/*
Bear Call Spread Strategy

A bear call spread involves selling a call option with a lower strike price and buying a call option with a higher strike price, both with the same expiration date.
This strategy is used when a moderate decline in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (net premium received)
- Limited risk (difference between strikes minus net premium)
- Generate income while maintaining a bearish outlook
- Both call options have the same expiration date
- Requires less margin than naked call selling
- Lower risk than naked call selling
- Maximum profit achieved when price stays below lower strike
- Also known as a vertical call credit spread
*/

/*
Bear Call Spread Strategy

A bear call spread, also known as a vertical call credit spread, is created by selling a call option with a lower strike price
and simultaneously buying a call option with a higher strike price, both with the same expiration date.

Key characteristics:
- Limited profit potential (net credit received)
- Limited risk (difference between strikes minus net credit)
- Bearish strategy that profits from price decline
- Both options have same expiration date
*/

use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::{debug, trace};

const BEAR_CALL_SPREAD_DESCRIPTION: &str =
    "A bear call spread is created by selling a call option with a lower strike price \
    and simultaneously buying a call option with a higher strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate decline in the underlying \
    asset's price. The maximum profit is limited to the net credit received, while the maximum \
    loss is limited to the difference between strike prices minus the net credit.";

#[derive(Clone, Debug)]
pub struct BearCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    short_call: Position,
    long_call: Position,
}

impl BearCallSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        mut short_strike: PositiveF64,
        mut long_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_short_call: f64,
        premium_long_call: f64,
        open_fee_short_call: f64,
        close_fee_short_call: f64,
        open_fee_long_call: f64,
        close_fee_long_call: f64,
    ) -> Self {
        if short_strike == PZERO {
            short_strike = underlying_price;
        }
        if long_strike == PZERO {
            long_strike = underlying_price;
        }

        let mut strategy = BearCallSpread {
            name: "Bear Call Spread".to_string(),
            kind: StrategyType::BearCallSpread,
            description: BEAR_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            long_call: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short_call,
            Utc::now(),
            open_fee_short_call,
            close_fee_short_call,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Error adding short call");

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call = Position::new(
            long_call_option,
            premium_long_call,
            Utc::now(),
            open_fee_long_call,
            close_fee_long_call,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Error adding long call");

        strategy.validate();

        // Calculate break-even point
        strategy
            .break_even_points
            .push(short_strike + strategy.net_premium_received() / quantity);

        strategy
    }
}

impl Positionable for BearCallSpread {
    fn add_position(&mut self, position: &Position) -> Result<(), String> {
        match position.option.side {
            Side::Short => {
                self.short_call = position.clone();
                Ok(())
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, String> {
        Ok(vec![&self.short_call, &self.long_call])
    }
}

impl Strategies for BearCallSpread {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.short_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let net_premium_received = self.net_premium_received();
        if net_premium_received < ZERO {
            trace!("Net premium received is negative {}", net_premium_received);
            Err("Net premium received is negative")
        } else {
            Ok(pos!(net_premium_received))
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let width = self.long_call.option.strike_price - self.short_call.option.strike_price;
        let mas_loss =
            (width * self.short_call.option.quantity).value() - self.net_premium_received();
        if mas_loss < ZERO {
            trace!("Max loss is negative {}", mas_loss);
            Err("Max loss is negative")
        } else {
            Ok(pos!(mas_loss))
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(self.short_call.net_cost() + self.long_call.net_cost())
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received() - self.long_call.net_cost()
    }

    fn fees(&self) -> f64 {
        self.short_call.open_fee
            + self.short_call.close_fee
            + self.long_call.open_fee
            + self.long_call.close_fee
    }

    fn profit_area(&self) -> f64 {
        let high = self.max_profit().unwrap_or(PZERO);
        let base = self.break_even_points[0] - self.short_call.option.strike_price;
        (high * base / 200.0).value()
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit().unwrap_or(PZERO);
        let max_loss = self.max_loss().unwrap_or(PZERO);
        match (max_profit, max_loss) {
            (PZERO, _) => ZERO,
            (_, PZERO) => f64::INFINITY,
            _ => (max_profit / max_loss * 100.0).value(),
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

impl Validable for BearCallSpread {
    fn validate(&self) -> bool {
        if !self.short_call.validate() {
            debug!("Short call is invalid");
            return false;
        }
        if !self.long_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        if self.short_call.option.strike_price >= self.long_call.option.strike_price {
            debug!("Short call strike price must be lower than long call strike price");
            return false;
        }
        true
    }
}

impl Optimizable for BearCallSpread {
    type Strategy = BearCallSpread;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for short_index in 0..options.len() {
            let short_option = &options[short_index];

            for long_option in &options[short_index + 1..] {
                if !self.is_valid_short_option(short_option, &side)
                    || !self.is_valid_long_option(long_option, &side)
                {
                    debug!(
                        "Invalid options Asset {} - Short({}) Long({})",
                        option_chain.underlying_price,
                        short_option.strike_price,
                        long_option.strike_price,
                    );
                    continue;
                }

                let legs = StrategyLegs::TwoLegs {
                    first: short_option,
                    second: long_option,
                };

                if !self.are_valid_prices(&legs) {
                    debug!(
                        "Invalid prices - Short({}): {:?} Long({}): {:?}",
                        short_option.strike_price,
                        short_option.call_bid,
                        long_option.strike_price,
                        long_option.call_ask
                    );
                    continue;
                }

                let strategy = self.create_strategy(option_chain, &legs);

                if !strategy.validate() {
                    debug!("Invalid strategy");
                    continue;
                }

                if strategy.max_profit().is_err() || strategy.max_loss().is_err() {
                    debug!(
                        "Invalid profit {} loss {}",
                        strategy.max_profit().unwrap_or(PZERO),
                        strategy.max_loss().unwrap_or(PZERO)
                    );
                    continue;
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio(),
                    OptimizationCriteria::Area => strategy.profit_area(),
                };

                if current_value > best_value {
                    best_value = current_value;
                    *self = strategy.clone();
                }
            }
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (short, long) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        BearCallSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            short.strike_price,
            long.strike_price,
            self.short_call.option.expiration_date.clone(),
            short.implied_volatility.unwrap().value() / 100.0,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            short.call_bid.unwrap().value(),
            long.call_ask.unwrap().value(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.long_call.open_fee,
            self.long_call.close_fee,
        )
    }
}

impl Profit for BearCallSpread {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price) + self.long_call.pnl_at_expiration(&price)
    }
}

impl Graph for BearCallSpread {
    fn title(&self) -> String {
        format!(
            "{} Strategy:\n\t{}\n\t{}",
            self.name,
            self.short_call.title(),
            self.long_call.title()
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let underlying_price = self.short_call.option.underlying_price.value();
        vec![ChartVerticalLine {
            x_coordinate: underlying_price,
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!("Current Price: {:.2}", underlying_price),
            label_offset: (4.0, 0.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Break Even {:.2}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.value(),
                self.max_profit().unwrap_or(PZERO).value(),
            ),
            label: format!("Max Profit {:.2}", self.max_profit().unwrap_or(PZERO)),
            label_offset: LabelOffsetType::Relative(-60.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.value(),
                -self.max_loss().unwrap_or(PZERO).value(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(PZERO)),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.short_call.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for BearCallSpread {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        Ok(self.short_call.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.short_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_point = self.get_break_even_points()[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.short_call.option.implied_volatility),
            pos!(self.long_call.option.implied_volatility),
        ]);

        let mut profit_range = ProfitLossRange::new(
            None,
            Some(break_even_point),
            pos!(self.max_profit()?.value()),
        )?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_point = self.get_break_even_points()[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.short_call.option.implied_volatility),
            pos!(self.long_call.option.implied_volatility),
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_point),
            Some(self.long_call.option.strike_price),
            pos!(self.max_loss()?.value()),
        )?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![loss_range])
    }
}

#[cfg(test)]
mod tests_bear_call_spread_graph {
    use super::*;
    use crate::pos;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(105.0),                // short_strike
            pos!(110.0),                // long_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            2.0,                        // premium_short_call
            1.0,                        // premium_long_call
            0.0,                        // fees
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_title() {
        let spread = create_test_spread();
        let title = spread.title();
        assert!(title.contains("Bear Call Spread Strategy"));
        assert!(title.contains("Short"));
        assert!(title.contains("Long"));
    }

    #[test]
    fn test_vertical_lines() {
        let spread = create_test_spread();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1); // Current price, short strike, long strike
        assert_eq!(lines[0].x_coordinate, 100.0);
        assert!(lines[0].label.contains("Current Price"));
    }

    #[test]
    fn test_get_points() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert_eq!(points.len(), 4); // Break-even, max profit, max loss, current price, profit zone

        assert_eq!(points[0].coordinates.1, 0.0);
        assert!(points[0].label.contains("Break Even"));

        assert_eq!(points[1].coordinates.0, 105.0); // short strike
        assert!(points[1].label.contains("Max Profit"));

        assert_eq!(points[2].coordinates.0, 110.0); // long strike
        assert!(points[2].label.contains("Max Loss"));
    }

    #[test]
    fn test_get_values() {
        let spread = create_test_spread();
        let test_prices = vec![
            pos!(95.0),
            pos!(100.0),
            pos!(105.0),
            pos!(110.0),
            pos!(115.0),
        ];

        let values = spread.get_values(&test_prices);
        assert_eq!(values.len(), 5);
        assert_eq!(values[0], spread.calculate_profit_at(pos!(95.0)));
        assert_eq!(values[1], spread.calculate_profit_at(pos!(100.0)));
        assert_eq!(values[2], spread.calculate_profit_at(pos!(105.0)));
        assert_eq!(values[3], spread.calculate_profit_at(pos!(110.0)));
        assert_eq!(values[4], spread.calculate_profit_at(pos!(115.0)));
        assert_eq!(values[0], spread.max_profit().unwrap_or(PZERO).value());
        assert_eq!(values[4], -spread.max_loss().unwrap_or(PZERO).value());
    }
}

#[cfg(test)]
mod tests_bear_call_spread_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(105.0),                // short_strike
            pos!(110.0),                // long_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            2.0,                        // premium_short_call
            1.0,                        // premium_long_call
            0.0,                        // open_fee_short_call
            0.0,                        // close_fee_short_call
            0.0,                        // open_fee_long_call
            0.0,                        // close_fee_long_call
        )
    }

    #[test]
    fn test_get_expiration() {
        let spread = create_test_spread();
        let result = spread.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    fn test_get_risk_free_rate() {
        let spread = create_test_spread();
        assert_eq!(spread.get_risk_free_rate(), Some(0.05));
    }

    #[test]
    fn test_get_profit_ranges() {
        let spread = create_test_spread();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_none());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > PZERO);
    }

    #[test]
    fn test_get_loss_ranges() {
        let spread = create_test_spread();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > PZERO);
    }

    #[test]
    fn test_probability_of_profit() {
        let spread = create_test_spread();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let spread = create_test_spread();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let result = spread.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_trend() {
        let spread = create_test_spread();
        let trend = Some(PriceTrend {
            drift_rate: -0.1,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_analyze_probabilities() {
        let spread = create_test_spread();
        let result = spread.analyze_probabilities(None, None);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > PZERO);
        assert!(analysis.probability_of_max_profit >= PZERO);
        assert!(analysis.probability_of_max_loss >= PZERO);
        assert!(analysis.expected_value > PZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > PZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let spread = create_test_spread();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= PZERO);
        assert!(max_loss_prob >= PZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}
