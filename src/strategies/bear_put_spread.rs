/*
Bear Put Spread Strategy

A bear put spread involves buying a put option with a higher strike price and selling a put option with a lower strike price,
both with the same expiration date. This strategy is used when a moderate decline in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (difference between strikes minus net premium paid)
- Limited risk (net premium paid)
- Bearish strategy that profits from price decrease
- Both options have same expiration date
- Lower cost than buying puts outright
- Maximum profit achieved when price falls below lower strike
- Also known as a vertical put debit spread
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
use tracing::debug;

const BEAR_PUT_SPREAD_DESCRIPTION: &str =
    "A bear put spread is created by buying a put option with a higher strike price \
    and simultaneously selling a put option with a lower strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate decrease in the underlying \
    asset's price. The maximum profit is limited to the difference between strike prices minus \
    the net premium paid, while the maximum loss is limited to the net premium paid.";

#[derive(Clone, Debug)]
pub struct BearPutSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_put: Position,
    short_put: Position,
}

impl BearPutSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        mut long_strike: PositiveF64,
        mut short_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_long_put: f64,
        premium_short_put: f64,
        open_fee_long_put: f64,
        close_fee_long_put: f64,
        open_fee_short_put: f64,
        close_fee_short_put: f64,
    ) -> Self {
        if long_strike == PZERO {
            long_strike = underlying_price;
        }
        if short_strike == PZERO {
            short_strike = underlying_price;
        }

        let mut strategy = BearPutSpread {
            name: "Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: BEAR_PUT_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put: Position::default(),
            short_put: Position::default(),
        };

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let long_put = Position::new(
            long_put_option,
            premium_long_put,
            Utc::now(),
            open_fee_long_put,
            close_fee_long_put,
        );
        strategy.add_position(&long_put.clone()).expect("Error adding long put");

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let short_put = Position::new(
            short_put_option,
            premium_short_put,
            Utc::now(),
            open_fee_short_put,
            close_fee_short_put,
        );
        strategy.add_position(&short_put.clone()).expect("Error adding short put");

        strategy.validate();

        // Calculate break-even point
        strategy
            .break_even_points
            .push(long_strike - strategy.net_premium_received() / quantity);

        strategy
    }
}

impl Positionable for BearPutSpread {
    fn add_position(&mut self, position: &Position) -> Result<(), String> {
        match position.option.side {
            Side::Short => {
                self.short_put = position.clone();
                Ok(())
            },
            Side::Long => {
                self.long_put = position.clone();
                Ok(())
            },
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, String> {
        Ok(vec![&self.long_put, &self.short_put])
    }
}

impl Strategies for BearPutSpread {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.long_put.option.underlying_price
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let profit = self.calculate_profit_at(self.short_put.option.strike_price);
        if profit >= ZERO {
            Ok(pos!(profit))
        } else {
            Err("Max profit must be greater than zero")
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let loss = self.calculate_profit_at(self.long_put.option.strike_price);
        if loss <= ZERO {
            Ok(pos!(loss.abs()))
        } else {
            Err("Max loss must be less than zero")
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(self.long_put.net_cost() - self.short_put.net_premium_received())
    }

    fn net_premium_received(&self) -> f64 {
        // self.short_put.net_premium_received() - self.long_put.net_cost()

        self.long_put.net_cost() - self.short_put.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.long_put.open_fee
            + self.long_put.close_fee
            + self.short_put.open_fee
            + self.short_put.close_fee
    }

    fn profit_area(&self) -> f64 {
        let high = self.max_profit().unwrap_or(PZERO);
        let base = self.break_even_points[0] - self.short_put.option.strike_price;
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

impl Validable for BearPutSpread {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long put is invalid");
            return false;
        }
        if !self.short_put.validate() {
            debug!("Short put is invalid");
            return false;
        }
        if self.long_put.option.strike_price <= self.short_put.option.strike_price {
            debug!("Long put strike price must be higher than short put strike price");
            return false;
        }
        true
    }
}

impl Optimizable for BearPutSpread {
    type Strategy = BearPutSpread;

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
                        "Invalid options Asset {} - Long({}) Short({})",
                        option_chain.underlying_price,
                        long_option.strike_price,
                        short_option.strike_price,
                    );
                    continue;
                }

                let legs = StrategyLegs::TwoLegs {
                    first: long_option,
                    second: short_option,
                };

                if !self.are_valid_prices(&legs) {
                    debug!(
                        "Invalid prices - Long({}): {:?} Short({}): {:?}",
                        long_option.strike_price,
                        long_option.put_ask,
                        short_option.strike_price,
                        short_option.put_bid
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

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        long.put_ask.unwrap_or(PZERO) > PZERO && short.put_bid.unwrap_or(PZERO) > PZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        BearPutSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_put.option.expiration_date.clone(),
            long.implied_volatility.unwrap().value() / 100.0,
            self.long_put.option.risk_free_rate,
            self.long_put.option.dividend_yield,
            self.long_put.option.quantity,
            long.put_ask.unwrap().value(),
            short.put_bid.unwrap().value(),
            self.long_put.open_fee,
            self.long_put.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for BearPutSpread {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_put.pnl_at_expiration(&price) + self.short_put.pnl_at_expiration(&price)
    }
}

impl Graph for BearPutSpread {
    fn title(&self) -> String {
        format!(
            "{} Strategy:\n\t{}\n\t{}",
            self.name,
            self.long_put.title(),
            self.short_put.title()
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let underlying_price = self.long_put.option.underlying_price.value();
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

        // Break Even Point
        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Break Even {:.2}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Profit Point (at lower strike price)
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.value(),
                self.max_profit().unwrap_or(PZERO).value(),
            ),
            label: format!("Max Profit {:.2}", self.max_profit().unwrap_or(PZERO)),
            label_offset: LabelOffsetType::Relative(10.0, 5.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Loss Point (at higher strike price)
        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.strike_price.value(),
                -self.max_loss().unwrap_or(PZERO).value(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(PZERO)),
            label_offset: LabelOffsetType::Relative(-60.0, -5.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Current Price Point
        points.push(self.get_point_at_price(self.long_put.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for BearPutSpread {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        Ok(self.long_put.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.long_put.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_point = self.get_break_even_points()[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.long_put.option.implied_volatility),
            pos!(self.short_put.option.implied_volatility),
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(self.short_put.option.strike_price), // Price below short strike is max profit
            Some(break_even_point),                   // Upper bound is break even point
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
            pos!(self.long_put.option.implied_volatility),
            pos!(self.short_put.option.implied_volatility),
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_point), // Lower bound is break even point
            None,                   // No upper bound (theoretically)
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
mod tests_bear_put_spread_strategy {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(105.0),                // long_strike
            pos!(95.0),                 // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            4.0,                        // premium_long_put
            2.0,                        // premium_short_put
            0.0,                        // open_fee_long_put
            0.0,                        // close_fee_long_put
            0.0,                        // open_fee_short_put
            0.0,                        // close_fee_short_put
        )
    }

    #[test]
    fn test_new_bear_put_spread() {
        let spread = create_test_spread();

        assert_eq!(spread.name, "Bear Put Spread");
        assert_eq!(spread.kind, StrategyType::BearPutSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(spread.get_underlying_price(), pos!(100.0));
        assert_eq!(spread.long_put.option.strike_price, pos!(105.0));
        assert_eq!(spread.short_put.option.strike_price, pos!(95.0));
    }

    #[test]
    fn test_add_leg() {
        let mut spread = create_test_spread();
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos!(110.0),
                ExpirationDate::Days(30.0),
                0.20,
                pos!(1.0),
                pos!(100.0),
                0.05,
                OptionStyle::Put,
                0.0,
                None,
            ),
            5.0,
            Utc::now(),
            0.0,
            0.0,
        );

        spread.add_position(&new_long_put.clone()).expect("Error adding long put");
        assert_eq!(spread.long_put.option.strike_price, pos!(110.0));
    }

    #[test]
    fn test_get_legs() {
        let spread = create_test_spread();
        let legs = spread.get_positions().expect("Error getting legs");

        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].option.side, Side::Long);
        assert_eq!(legs[1].option.side, Side::Short);
    }

    #[test]
    fn test_max_profit() {
        let spread = create_test_spread();
        let max_profit = spread.max_profit().unwrap();
        // Width (105 - 95 = 10) - Net Premium (4 - 2 = 2)
        assert_eq!(max_profit, pos!(8.0));
    }

    #[test]
    fn test_max_loss() {
        let spread = create_test_spread();
        let max_loss = spread.max_loss().unwrap();
        // Net Premium Paid (4 - 2 = 2)
        assert_eq!(max_loss, pos!(2.0));
    }

    #[test]
    fn test_total_cost() {
        let spread = create_test_spread();
        // Long Premium - Short Premium (4 - 2 = 2)
        assert_eq!(spread.total_cost(), pos!(2.0));
    }

    #[test]
    fn test_net_premium_received() {
        let spread = create_test_spread();
        // Net Premium Received is actually Net Premium Paid in this case
        assert_eq!(spread.net_premium_received(), 2.0);
    }

    #[test]
    fn test_fees() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            4.0,
            2.0,
            0.5, // open_fee_long_put
            0.5, // close_fee_long_put
            0.5, // open_fee_short_put
            0.5, // close_fee_short_put
        );

        assert_eq!(spread.fees(), 2.0); // Total fees = 0.5 * 4
    }

    #[test]
    fn test_break_even_points() {
        let spread = create_test_spread();
        let break_even_points = spread.get_break_even_points();

        assert_eq!(break_even_points.len(), 1);
        // Break-even = Long Strike - Net Premium / Quantity
        assert_eq!(break_even_points[0], pos!(103.0)); // 105 - 2/1
    }

    #[test]
    fn test_profit_area() {
        let spread = create_test_spread();
        let area = spread.profit_area();
        assert!(area > 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let spread = create_test_spread();
        let ratio = spread.profit_ratio();
        // Ratio = (max_profit / max_loss) * 100
        // = (8.0 / 2.0) * 100 = 400.0
        assert_eq!(ratio, 400.0);
    }

    #[test]
    fn test_default_strikes() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            PZERO, // long_strike = default
            PZERO, // short_strike = default
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        assert_eq!(spread.long_put.option.strike_price, pos!(100.0));
        assert_eq!(spread.short_put.option.strike_price, pos!(100.0));
    }

    #[test]
    fn test_with_different_quantities() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let max_profit = spread.max_profit().unwrap();
        let max_loss = spread.max_loss().unwrap();

        // Max Profit = (Width * Quantity) - Net Premium
        assert_eq!(max_profit, pos!(16.0)); // (10 * 2) - (4 - 2)
                                            // Max Loss = Net Premium
        assert_eq!(max_loss, pos!(4.0));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_validation {
    use super::*;
    use crate::model::types::ExpirationDate;
    use chrono::Utc;

    fn create_valid_position(
        side: Side,
        strike_price: PositiveF64,
        expiration: ExpirationDate,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                expiration,
                0.20,
                pos!(1.0),
                pos!(100.0),
                0.05,
                OptionStyle::Put,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_invalid_long_put() {
        let mut invalid_long =
            create_valid_position(Side::Long, pos!(105.0), ExpirationDate::Days(30.0));
        invalid_long.option.quantity = PZERO;

        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: invalid_long,
            short_put: create_valid_position(Side::Short, pos!(95.0), ExpirationDate::Days(30.0)),
        };

        assert!(!spread.validate());
    }

    #[test]
    fn test_invalid_short_put() {
        let mut invalid_short =
            create_valid_position(Side::Short, pos!(95.0), ExpirationDate::Days(30.0));
        invalid_short.option.quantity = PZERO;

        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(Side::Long, pos!(105.0), ExpirationDate::Days(30.0)),
            short_put: invalid_short,
        };

        assert!(!spread.validate());
    }

    #[test]
    fn test_invalid_strike_prices() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(Side::Long, pos!(95.0), ExpirationDate::Days(30.0)),
            short_put: create_valid_position(Side::Short, pos!(105.0), ExpirationDate::Days(30.0)),
        };

        assert!(!spread.validate());
    }

    #[test]
    fn test_equal_strike_prices() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(Side::Long, pos!(100.0), ExpirationDate::Days(30.0)),
            short_put: create_valid_position(Side::Short, pos!(100.0), ExpirationDate::Days(30.0)),
        };

        assert!(!spread.validate());
    }

    #[test]
    fn test_valid_strike_prices() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(Side::Long, pos!(105.0), ExpirationDate::Days(30.0)),
            short_put: create_valid_position(Side::Short, pos!(95.0), ExpirationDate::Days(30.0)),
        };

        assert!(spread.validate());
    }

    #[test]
    fn test_different_expiration_dates() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(Side::Long, pos!(105.0), ExpirationDate::Days(30.0)),
            short_put: create_valid_position(Side::Short, pos!(95.0), ExpirationDate::Days(60.0)),
        };

        assert!(spread.validate());
    }
}

#[cfg(test)]
mod tests_bear_put_spread_optimization {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::spos;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(90.0), "2024-12-31".to_string());

        // Add options with increasing strikes around the current price
        chain.add_option(
            pos!(85.0),   // strike
            None,         // call_bid
            None,         // call_ask
            spos!(8.0),   // put_bid
            spos!(8.2),   // put_ask
            spos!(0.2),   // implied_volatility
            Some(-0.8),   // delta
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(90.0),
            None,
            None,
            spos!(6.0),
            spos!(6.2),
            spos!(0.2),
            Some(-0.7),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(95.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(-0.6),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(100.0),
            None,
            None,
            spos!(2.5),
            spos!(2.7),
            spos!(0.2),
            Some(-0.5),
            spos!(250.0),
            Some(125),
        );

        chain.add_option(
            pos!(105.0),
            None,
            None,
            spos!(1.5),
            spos!(1.7),
            spos!(0.2),
            Some(-0.4),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(110.0),
            None,
            None,
            spos!(0.8),
            spos!(1.0),
            spos!(0.2),
            Some(-0.3),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(115.0),
            None,
            None,
            spos!(0.4),
            spos!(0.6),
            spos!(0.2),
            Some(-0.2),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    fn create_base_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.7, // premium_long_put
            4.0, // premium_short_put
            0.0,
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.profit_ratio() > 0.0,
            "Profit ratio should be positive"
        );

        // The optimal ratio should choose strikes with maximum difference while minimizing cost
        assert!(spread.long_put.option.strike_price > spread.short_put.option.strike_price);
    }

    #[test]
    fn test_find_optimal_area() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(spread.profit_area() > 0.0, "Profit area should be positive");

        // Area optimization should favor wider spreads with good probability of profit
        assert!(spread.long_put.option.strike_price > chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_upper_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        // Both strikes should be above the underlying price
        assert!(spread.short_put.option.strike_price > chain.underlying_price);
        assert!(spread.long_put.option.strike_price > chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_range() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
            OptimizationCriteria::Ratio,
        );

        // Strikes should be within the specified range
        assert!(spread.short_put.option.strike_price >= pos!(95.0));
        assert!(spread.long_put.option.strike_price <= pos!(105.0));
    }

    #[test]
    fn test_are_valid_prices() {
        let spread = create_base_spread();

        // Test with valid prices
        let valid_option1 = OptionData::new(
            pos!(105.0),
            None,
            None,
            spos!(1.5),
            spos!(1.7),
            spos!(0.2),
            Some(-0.4),
            spos!(100.0),
            Some(50),
        );

        let valid_option2 = OptionData::new(
            pos!(95.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(-0.6),
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::TwoLegs {
            first: &valid_option1,
            second: &valid_option2,
        };
        assert!(spread.are_valid_prices(&legs));

        // Test with invalid prices (zero or None)
        let invalid_option = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            Some(-0.5),
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::TwoLegs {
            first: &invalid_option,
            second: &valid_option2,
        };
        assert!(!spread.are_valid_prices(&legs));

        let legs = StrategyLegs::TwoLegs {
            first: &valid_option1,
            second: &invalid_option,
        };
        assert!(!spread.are_valid_prices(&legs));
    }

    #[test]
    fn test_create_strategy() {
        let spread = create_base_spread();
        let chain = create_test_chain();

        let long_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(105.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(95.0))
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs);

        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_put.option.strike_price, pos!(105.0));
        assert_eq!(new_strategy.short_put.option.strike_price, pos!(95.0));
        assert_eq!(new_strategy.long_put.option.implied_volatility, 0.002); // 0.2 / 100

        // Verify premiums are set correctly
        assert_eq!(new_strategy.long_put.premium, 1.7); // put_ask from long option
        assert_eq!(new_strategy.short_put.premium, 4.0); // put_bid from short option
    }

    #[test]
    fn test_optimization_with_invalid_options() {
        let mut spread = create_base_spread();
        let mut chain = create_test_chain();

        // Add some invalid options to the chain
        chain.add_option(
            pos!(120.0),
            None,
            None,
            None, // Invalid: no put_bid
            None, // Invalid: no put_ask
            spos!(0.2),
            Some(-0.1),
            spos!(50.0),
            Some(25),
        );

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        // Should still find a valid optimization ignoring invalid options
        assert!(spread.validate());
        assert!(spread.max_profit().is_ok());
    }

    #[test]
    fn test_optimization_with_different_quantities() {
        let mut spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            1.7,
            4.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(spread.validate());
        assert_eq!(spread.long_put.option.quantity, pos!(2.0));
        assert_eq!(spread.short_put.option.quantity, pos!(2.0));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(105.0),                // long_strike
            pos!(95.0),                 // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            4.0,                        // premium_long_put
            2.0,                        // premium_short_put
            0.0,                        // open_fee_long_put
            0.0,                        // close_fee_long_put
            0.0,                        // open_fee_short_put
            0.0,                        // close_fee_short_put
        )
    }

    #[test]
    fn test_profit_at_max_profit() {
        let spread = create_test_spread();
        let price = pos!(90.0);

        // Max Profit = Width (105 - 95 = 10) - Net Premium (4 - 2 = 2) = 8
        assert_eq!(spread.calculate_profit_at(price), 8.0);
    }

    #[test]
    fn test_profit_at_max_loss() {
        let spread = create_test_spread();
        let price = pos!(110.0);

        // Max Loss = Net Premium = 4 - 2 = 2
        assert_eq!(spread.calculate_profit_at(price), -2.0);
    }

    #[test]
    fn test_profit_at_short_strike() {
        let spread = create_test_spread();
        let price = pos!(95.0);

        // Profit at short strike = Max Profit = 8
        assert_eq!(spread.calculate_profit_at(price), 8.0);
    }

    #[test]
    fn test_profit_at_long_strike() {
        let spread = create_test_spread();
        let price = pos!(105.0);

        // Loss at long strike = Max Loss = -2
        assert_eq!(spread.calculate_profit_at(price), -2.0);
    }

    #[test]
    fn test_profit_at_break_even() {
        let spread = create_test_spread();
        let price = pos!(103.0); // Break even = long strike - net premium = 105 - 2

        assert!(spread.calculate_profit_at(price).abs() < 0.01);
    }

    #[test]
    fn test_profit_between_strikes() {
        let spread = create_test_spread();
        let price = pos!(100.0);

        let profit = spread.calculate_profit_at(price);

        assert!(profit > -2.0);
        assert!(profit < 8.0);
    }

    #[test]
    fn test_profit_with_different_quantities() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let max_profit_price = pos!(90.0);
        let max_loss_price = pos!(110.0);

        // Con quantity = 2:
        // Max Profit = 2 * (Width - Net Premium) = 2 * (10 - 2) = 16
        assert_eq!(spread.calculate_profit_at(max_profit_price), 16.0);

        // Max Loss = 2 * Net Premium = 2 * 2 = 4
        assert_eq!(spread.calculate_profit_at(max_loss_price), -4.0);
    }

    #[test]
    fn test_profit_with_fees() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            4.0,
            2.0,
            0.5, // open_fee_long_put
            0.5, // close_fee_long_put
            0.5, // open_fee_short_put
            0.5, // close_fee_short_put
        );

        let max_profit_price = pos!(90.0);

        // Max Profit = Width - Net Premium - Total Fees
        // = (105 - 95) - (4 - 2) - (0.5 + 0.5 + 0.5 + 0.5)
        // = 10 - 2 - 2 = 6
        assert_eq!(spread.calculate_profit_at(max_profit_price), 6.0);
    }

    #[test]
    fn test_profit_far_below_strikes() {
        let spread = create_test_spread();
        let price = pos!(80.0);

        // El profit debería ser igual al max profit
        assert_eq!(spread.calculate_profit_at(price), 8.0);
    }

    #[test]
    fn test_profit_far_above_strikes() {
        let spread = create_test_spread();
        let price = pos!(120.0);

        assert_eq!(spread.calculate_profit_at(price), -2.0);
    }
}

#[cfg(test)]
mod tests_bear_put_spread_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(105.0),                // long_strike
            pos!(95.0),                 // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            4.0,                        // premium_long_put
            2.0,                        // premium_short_put
            0.0,                        // open_fee_long_put
            0.0,                        // close_fee_long_put
            0.0,                        // open_fee_short_put
            0.0,                        // close_fee_short_put
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
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert_eq!(range.lower_bound.unwrap(), pos!(95.0)); // short strike
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
        assert!(range.upper_bound.is_none());
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
            drift_rate: -0.1, // Negative drift for bearish trend
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
        assert!(analysis.expected_value != PZERO);
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

#[cfg(test)]
mod tests_bear_put_spread_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(105.0),                // long_strike
            pos!(95.0),                 // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            4.0,                        // premium_long_put
            2.0,                        // premium_short_put
            0.0,                        // fees
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_title_format() {
        let spread = create_test_spread();
        let title = spread.title();

        assert!(title.contains("Bear Put Spread Strategy"));
        assert!(title.contains("Long"));
        assert!(title.contains("Short"));
        assert!(title.contains("TEST")); // symbol
        assert!(title.contains("105")); // long strike
        assert!(title.contains("95")); // short strike
    }

    #[test]
    fn test_get_vertical_lines() {
        let spread = create_test_spread();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1);

        let line = &lines[0];
        assert_eq!(line.x_coordinate, 100.0);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price"));
        assert!(line.label.contains("100.00"));
        assert_eq!(line.label_offset, (4.0, 0.0));
        assert_eq!(line.line_color, ORANGE);
        assert_eq!(line.label_color, ORANGE);
        assert_eq!(line.font_size, 18);
    }

    #[test]
    fn test_get_points() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert_eq!(points.len(), 4); // Break even, max profit, max loss, current price

        // Break even point
        let break_even = &points[0];
        assert_eq!(break_even.coordinates.1, 0.0);
        assert!(break_even.label.contains("Break Even"));
        assert_eq!(break_even.point_color, DARK_BLUE);
        assert_eq!(break_even.label_color, DARK_BLUE);
        assert_eq!(break_even.point_size, 5);
        assert_eq!(break_even.font_size, 18);

        // Max profit point at short strike
        let max_profit = &points[1];
        assert_eq!(max_profit.coordinates.0, 95.0);
        assert!(max_profit.label.contains("Max Profit"));
        assert_eq!(max_profit.point_color, DARK_GREEN);
        assert_eq!(max_profit.label_color, DARK_GREEN);

        // Max loss point at long strike
        let max_loss = &points[2];
        assert_eq!(max_loss.coordinates.0, 105.0);
        assert!(max_loss.label.contains("Max Loss"));
        assert_eq!(max_loss.point_color, RED);
        assert_eq!(max_loss.label_color, RED);
    }

    #[test]
    fn test_points_coordinates() {
        let spread = create_test_spread();
        let points = spread.get_points();

        // Break even point
        assert_eq!(points[0].coordinates.1, 0.0);

        // Maximum profit point at short strike
        assert_eq!(points[1].coordinates.0, 95.0);
        assert_eq!(points[1].coordinates.1, 8.0); // Width (10) - Net Premium (2.0)

        // Maximum loss point at long strike
        assert_eq!(points[2].coordinates.0, 105.0);
        assert_eq!(points[2].coordinates.1, -2.0); // -Net Premium

        // Current price point
        assert_eq!(points[3].coordinates.0, 100.0);
        let current_profit = spread.calculate_profit_at(pos!(100.0));
        assert_eq!(points[3].coordinates.1, current_profit);
    }

    #[test]
    fn test_point_labels() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert_eq!(points.len(), 4);
        assert!(points[0].label.contains("Break Even"));
        assert!(points[1].label.contains("Max Profit"));
        assert!(points[2].label.contains("Max Loss"));
        assert!(points[3].label.contains("3.00"));
    }

    #[test]
    fn test_points_style() {
        let spread = create_test_spread();
        let points = spread.get_points();

        for point in points.iter() {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert!(matches!(point.point_color, DARK_BLUE | DARK_GREEN | RED));
            assert_eq!(point.point_color, point.label_color);
        }
    }

    #[test]
    fn test_graph_with_different_quantities() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let points = spread.get_points();
        let max_profit_point = &points[1];
        let max_loss_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 16.0); // 2 * (10.0 - 2.0)
        assert_eq!(max_loss_point.coordinates.1, -4.0); // 2 * -2.0
    }
}
