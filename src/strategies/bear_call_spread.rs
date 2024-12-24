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
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::probability::ProbabilityError;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::debug;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};

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
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
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

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.long_call])
    }
}

impl Strategies for BearCallSpread {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.short_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<PositiveF64, StrategyError> {
        let net_premium_received = self.net_premium_received();
        if net_premium_received < ZERO {
            Err(StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
                reason: "Net premium received is negative".to_string(),
            }))
        } else {
            Ok(pos!(net_premium_received))
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, StrategyError> {
        let width = self.long_call.option.strike_price - self.short_call.option.strike_price;
        let mas_loss =
            (width * self.short_call.option.quantity).value() - self.net_premium_received();
        if mas_loss < ZERO {
            Err(StrategyError::ProfitLossError(ProfitLossErrorKind::MaxLossError {
                reason: "Max loss is negative".to_string(),
            }))
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

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_double_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |&option| {
                option.0.is_valid_optimal_side(underlying_price, &side)
                    && option.1.is_valid_optimal_side(underlying_price, &side)
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(short, long)| {
                long.call_ask.unwrap_or(PZERO) > PZERO && short.call_bid.unwrap_or(PZERO) > PZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(short_option, long_option)| {
                let legs = StrategyLegs::TwoLegs {
                    first: short_option,
                    second: long_option,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(short, long)| OptionDataGroup::Two(short, long))
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = f64::NEG_INFINITY;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (short_option, long_option) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: short_option,
                second: long_option,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio(),
                OptimizationCriteria::Area => strategy.profit_area(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                debug!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
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
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.short_call.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.short_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
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

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
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

impl Greeks for BearCallSpread {
    fn greeks(&self) -> Greek {
        let long_call_greek = self.long_call.greeks();
        let short_call_greek = self.short_call.greeks();

        Greek {
            delta: long_call_greek.delta + short_call_greek.delta,
            gamma: long_call_greek.gamma + short_call_greek.gamma,
            theta: long_call_greek.theta + short_call_greek.theta,
            vega: long_call_greek.vega + short_call_greek.vega,
            rho: long_call_greek.rho + short_call_greek.rho,
            rho_d: long_call_greek.rho_d + short_call_greek.rho_d,
        }
    }
}

impl DeltaNeutrality for BearCallSpread {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_delta = self.long_call.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let threshold = DELTA_THRESHOLD;
        DeltaInfo {
            net_delta: long_call_delta + short_call_delta,
            individual_deltas: vec![long_call_delta, short_call_delta],
            is_neutral: (long_call_delta + short_call_delta).abs() < threshold,
            underlying_price: self.long_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> PositiveF64 {
        self.long_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        vec![DeltaAdjustment::SellOptions {
            quantity: pos!((net_delta.abs() / self.short_call.option.delta()).abs())
                * self.short_call.option.quantity,
            strike: self.short_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        vec![DeltaAdjustment::BuyOptions {
            quantity: pos!((net_delta.abs() / self.long_call.option.delta()).abs())
                * self.long_call.option.quantity,
            strike: self.long_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }
}

#[cfg(test)]
mod tests_bear_call_spread_strategies {
    use super::*;
    use crate::model::types::ExpirationDate;
    use approx::assert_relative_eq;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(95.0),                 // short_strike
            pos!(105.0),                // long_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            10.0,                       // premium_short_call
            5.0,                        // premium_long_call
            0.5,                        // open_fee_short_call
            0.5,                        // close_fee_short_call
            0.5,                        // open_fee_long_call
            0.5,                        // close_fee_long_call
        )
    }

    #[test]
    fn test_get_underlying_price() {
        let spread = create_test_spread();
        assert_eq!(spread.get_underlying_price(), pos!(100.0));
    }

    #[test]
    fn test_max_profit_positive() {
        let spread = create_test_spread();
        let result = spread.max_profit();
        assert!(result.is_ok());
        assert_relative_eq!(
            result.unwrap().value(),
            spread.net_premium_received(),
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_max_profit_negative() {
        let mut spread = create_test_spread();
        // Modify premiums to create negative net premium
        spread.short_call.premium = 1.0;
        spread.long_call.premium = 2.0;

        let result = spread.max_profit();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "Profit/Loss error: Maximum profit calculation error: Net premium received is negative");
    }

    #[test]
    fn test_max_loss() {
        let spread = create_test_spread();
        let result = spread.max_loss();
        assert!(result.is_ok());

        let width =
            (spread.long_call.option.strike_price - spread.short_call.option.strike_price).value();
        let expected_loss =
            width * spread.short_call.option.quantity.value() - spread.net_premium_received();
        assert_relative_eq!(result.unwrap().value(), expected_loss, epsilon = 0.0001);
    }

    #[test]
    fn test_max_loss_negative() {
        let mut spread = create_test_spread();
        // Modify strikes to create invalid width
        spread.short_call.option.strike_price = pos!(105.0);
        spread.long_call.option.strike_price = pos!(95.0);

        let result = spread.max_loss();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Profit/Loss error: Maximum loss calculation error: Max loss is negative");
    }

    #[test]
    fn test_total_cost() {
        let spread = create_test_spread();
        let expected_cost = spread.short_call.net_cost() + spread.long_call.net_cost();
        assert_relative_eq!(spread.total_cost().value(), expected_cost, epsilon = 0.0001);
    }

    #[test]
    fn test_net_premium_received() {
        let spread = create_test_spread();
        let expected_premium =
            spread.short_call.net_premium_received() - spread.long_call.net_cost();
        assert_relative_eq!(
            spread.net_premium_received(),
            expected_premium,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_fees() {
        let spread = create_test_spread();
        let expected_fees = spread.short_call.open_fee
            + spread.short_call.close_fee
            + spread.long_call.open_fee
            + spread.long_call.close_fee;
        assert_relative_eq!(spread.fees(), expected_fees, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_area() {
        let spread = create_test_spread();
        let high = spread.max_profit().unwrap_or(PZERO);
        let base = spread.break_even_points[0] - spread.short_call.option.strike_price;
        let expected_area = (high * base / 200.0).value();
        assert_relative_eq!(spread.profit_area(), expected_area, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_ratio_normal() {
        let spread = create_test_spread();
        let max_profit = spread.max_profit().unwrap();
        let max_loss = spread.max_loss().unwrap();
        let expected_ratio = (max_profit / max_loss * 100.0).value();
        assert_relative_eq!(spread.profit_ratio(), expected_ratio, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_ratio_zero_profit() {
        let mut spread = create_test_spread();
        // Modify premiums to create zero max profit
        spread.short_call.premium = 1.0;
        spread.long_call.premium = 1.0;

        assert_relative_eq!(spread.profit_ratio(), 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_ratio_zero_loss() {
        let mut spread = create_test_spread();
        // Modify strikes to create zero max loss scenario
        spread.long_call.option.strike_price = spread.short_call.option.strike_price;

        assert!(spread.profit_ratio().is_infinite());
    }

    #[test]
    fn test_get_break_even_points() {
        let spread = create_test_spread();
        let break_even_points = spread.get_break_even_points();
        assert!(!break_even_points.is_empty());
        assert_eq!(break_even_points.len(), 1);

        // Break even should be short strike plus net premium received per contract
        let expected_break_even = spread.short_call.option.strike_price
            + pos!(spread.net_premium_received() / spread.short_call.option.quantity.value());
        assert_relative_eq!(
            break_even_points[0].value(),
            expected_break_even.value(),
            epsilon = 0.0001
        );
    }

    #[test]
    #[should_panic]
    fn test_with_different_quantities() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            2.0,
            1.0,
            0.5,
            0.5,
            0.5,
            0.5,
        );

        // Check that all calculations scale properly with quantity
        let base_spread = create_test_spread();
        assert_relative_eq!(
            spread.max_profit().unwrap().value(),
            base_spread.max_profit().unwrap().value() * 2.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            spread.max_loss().unwrap().value(),
            base_spread.max_loss().unwrap().value() * 2.0,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_with_different_strikes() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),  // wider spread
            pos!(110.0), // wider spread
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.5,
            0.5,
            0.5,
            0.5,
        );

        // Check that strike width affects max loss calculation
        let base_spread = create_test_spread();
        assert!(spread.max_loss().unwrap() > base_spread.max_loss().unwrap());
    }
}

#[cfg(test)]
mod tests_bear_call_spread_positionable {
    use super::*;
    use crate::model::option::Options;
    use crate::model::position::Position;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use chrono::Utc;

    // Helper function to create a test option
    fn create_test_option(side: Side) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(30.0),
            0.2,
            pos!(1.0),
            pos!(100.0),
            0.05,
            OptionStyle::Call,
            0.0,
            None,
        )
    }

    // Helper function to create a test position
    fn create_test_position(side: Side) -> Position {
        Position::new(
            create_test_option(side),
            1.0,        // premium
            Utc::now(), // timestamp
            0.0,        // open_fee
            0.0,        // close_fee
        )
    }

    #[test]
    fn test_add_short_position() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let short_position = create_test_position(Side::Short);
        let result = spread.add_position(&short_position);

        assert!(result.is_ok());
        assert_eq!(spread.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_add_long_position() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let long_position = create_test_position(Side::Long);
        let result = spread.add_position(&long_position);

        assert!(result.is_ok());
        assert_eq!(spread.long_call.option.side, Side::Long);
    }

    #[test]
    fn test_get_positions() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let result = spread.get_positions();
        assert!(result.is_ok());

        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].option.side, Side::Short);
        assert_eq!(positions[1].option.side, Side::Long);
    }

    #[test]
    fn test_add_multiple_positions() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let short_position = create_test_position(Side::Short);
        let long_position = create_test_position(Side::Long);

        assert!(spread.add_position(&short_position).is_ok());
        assert!(spread.add_position(&long_position).is_ok());

        let positions = spread.get_positions().unwrap();
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_replace_positions() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        // Create new positions
        let new_short = create_test_position(Side::Short);
        let new_long = create_test_position(Side::Long);

        // Replace positions
        assert!(spread.add_position(&new_short).is_ok());
        assert!(spread.add_position(&new_long).is_ok());
    }

    #[test]
    fn test_positions_integrity() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let short_position = create_test_position(Side::Short);
        let long_position = create_test_position(Side::Long);

        spread.add_position(&short_position).unwrap();
        spread.add_position(&long_position).unwrap();

        let positions = spread.get_positions().unwrap();

        // Verify position integrity
        assert_eq!(positions[0].option.side, Side::Short);
        assert_eq!(positions[1].option.side, Side::Long);
        assert_eq!(positions[0].premium, 1.0);
        assert_eq!(positions[1].premium, 1.0);
        assert_eq!(positions[0].open_fee, 0.0);
        assert_eq!(positions[1].open_fee, 0.0);
    }
}
#[cfg(test)]
mod tests_bear_call_spread_validable {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_valid_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(95.0),                 // short_strike
            pos!(105.0),                // long_strike
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
    fn test_valid_spread() {
        let spread = create_valid_spread();
        assert!(spread.validate());
    }

    #[test]
    fn test_invalid_strike_order() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0), // short strike higher than long strike
            pos!(95.0),  // long strike lower than short strike
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!spread.validate());
    }

    #[test]
    fn test_equal_strikes() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(100.0), // both strikes equal
            pos!(100.0), // both strikes equal
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!spread.validate());
    }

    #[test]
    fn test_invalid_short_call() {
        let mut spread = create_valid_spread();
        // Invalidate short call by setting an invalid quantity
        spread.short_call.option.quantity = pos!(0.0);
        assert!(!spread.validate());
    }

    #[test]
    fn test_invalid_long_call() {
        let mut spread = create_valid_spread();
        // Invalidate long call by setting an invalid quantity
        spread.long_call.option.quantity = pos!(0.0);
        assert!(!spread.validate());
    }

    #[test]
    #[should_panic]
    fn test_invalid_expiration_dates() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(0.0), // Invalid expiration (0 days)
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!spread.validate());
    }

    #[test]
    fn test_invalid_volatility() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            -0.20, // Invalid negative volatility
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!spread.validate());
    }

    #[test]
    fn test_invalid_underlying_price() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(0.0), // Invalid underlying price
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!spread.validate());
    }

    #[test]
    fn test_strikes_too_close() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(99.999),  // Strikes very close to each other
            pos!(100.001), // but technically different
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        // Should still be valid as long as strikes are different
        assert!(spread.validate());
    }

    #[test]
    fn test_validation_with_different_quantities() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // Different quantity
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        // Should be valid as quantity > 0
        assert!(spread.validate());
    }
}
#[cfg(test)]
mod tests_bear_call_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pricing::payoff::Profit;
    use approx::assert_relative_eq;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(95.0),                 // short_strike
            pos!(105.0),                // long_strike
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
    fn test_profit_below_short_strike() {
        let spread = create_test_spread();
        let profit = spread.calculate_profit_at(pos!(90.0));
        // When price is below short strike, both options expire worthless
        // Profit should be the net premium received
        let expected_profit = spread.net_premium_received();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_at_short_strike() {
        let spread = create_test_spread();
        let profit = spread.calculate_profit_at(pos!(95.0));
        // At short strike, short call is at-the-money
        let expected_profit = spread.net_premium_received();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_between_strikes() {
        let spread = create_test_spread();
        let test_price = pos!(100.0);
        let profit = spread.calculate_profit_at(test_price);
        // Between strikes, only short call is in-the-money
        let intrinsic_value = test_price - spread.short_call.option.strike_price;
        let expected_profit = spread.net_premium_received() - intrinsic_value.value();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_at_long_strike() {
        let spread = create_test_spread();
        let profit = spread.calculate_profit_at(pos!(105.0));
        // At long strike, both options are in-the-money
        let short_intrinsic = pos!(105.0) - spread.short_call.option.strike_price;
        let long_intrinsic = pos!(105.0) - spread.long_call.option.strike_price;
        let expected_profit =
            spread.net_premium_received() - short_intrinsic.value() + long_intrinsic.value();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_above_long_strike() {
        let spread = create_test_spread();
        let profit = spread.calculate_profit_at(pos!(110.0));
        // Maximum loss occurs when price is above long strike
        let expected_profit = -spread.max_loss().unwrap().value();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_at_break_even() {
        let spread = create_test_spread();
        let break_even = spread.get_break_even_points()[0];
        let profit = spread.calculate_profit_at(break_even);
        // At break-even point, profit should be zero
        assert_relative_eq!(profit, 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_with_different_quantities() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let profit = spread.calculate_profit_at(pos!(90.0));
        // With quantity = 2, profit should be double
        let expected_profit = spread.net_premium_received();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
        assert_relative_eq!(
            profit,
            2.0 * create_test_spread().calculate_profit_at(pos!(90.0)),
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_profit_with_fees() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            2.0,
            1.0,
            0.5, // open_fee_short_call
            0.5, // close_fee_short_call
            0.5, // open_fee_long_call
            0.5, // close_fee_long_call
        );

        let profit = spread.calculate_profit_at(pos!(90.0));
        // Net premium should be reduced by total fees
        let expected_profit = spread.net_premium_received();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
        assert!(profit < create_test_spread().calculate_profit_at(pos!(90.0)));
    }
}

#[cfg(test)]
mod tests_bear_call_spread_optimizable {
    use super::*;
    use crate::model::types::{ExpirationDate, PositiveF64};
    use crate::spos;
    use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};

    // Helper function to create a mock OptionChain for testing
    fn create_mock_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);

        // Add options with different strikes and prices
        chain.add_option(
            pos!(95.0),   // strike
            spos!(6.0),   // call_bid
            spos!(6.2),   // call_ask
            spos!(1.0),   // put_bid
            spos!(1.2),   // put_ask
            spos!(0.2),   // implied_vol
            Some(0.7),    // delta
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.2),
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(0.5),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(105.0),
            spos!(1.0),
            spos!(1.2),
            spos!(6.0),
            spos!(6.2),
            spos!(0.2),
            Some(0.3),
            spos!(150.0),
            Some(75),
        );

        chain
    }

    // Helper function to create a basic BearCallSpread for testing
    fn create_test_strategy() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            1.2,
            0.0,
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_filter_combinations_valid() {
        let strategy = create_test_strategy();
        let chain = create_mock_option_chain();
        let combinations: Vec<_> = strategy
            .filter_combinations(&chain, FindOptimalSide::Upper)
            .collect();

        assert!(!combinations.is_empty());

        // Test some properties of the filtered combinations
        for combination in combinations {
            match combination {
                OptionDataGroup::Two(short, long) => {
                    // Short strike should be lower than long strike
                    assert!(short.strike_price < long.strike_price);

                    // Both options should have valid prices
                    assert!(short.call_bid.is_some());
                    assert!(long.call_ask.is_some());

                    // Both options should have valid implied volatility
                    assert!(short.implied_volatility.is_some());
                    assert!(long.implied_volatility.is_some());
                }
                _ => panic!("Expected Two-leg combination"),
            }
        }
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut strategy = create_test_strategy();
        let chain = create_mock_option_chain();

        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        // Verify the strategy was updated with optimal values
        assert!(strategy.validate());
        assert!(strategy.max_profit().is_ok());
        assert!(strategy.max_loss().is_ok());
        assert!(strategy.profit_ratio() > 0.0);
    }

    #[test]
    fn test_find_optimal_area() {
        let mut strategy = create_test_strategy();
        let chain = create_mock_option_chain();

        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        // Verify the strategy was updated with optimal values
        assert!(strategy.validate());
        assert!(strategy.max_profit().is_ok());
        assert!(strategy.max_loss().is_ok());
        assert!(strategy.profit_area() > 0.0);
    }

    #[test]
    fn test_create_strategy() {
        let strategy = create_test_strategy();
        let chain = create_mock_option_chain();

        // Get two option data entries from the chain
        let short_option = chain.options.iter().next().unwrap();
        let long_option = chain.options.iter().last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: short_option,
            second: long_option,
        };

        let new_strategy = strategy.create_strategy(&chain, &legs);

        // Verify the new strategy
        assert!(new_strategy.validate());
        assert_eq!(
            new_strategy.short_call.option.strike_price,
            short_option.strike_price
        );
        assert_eq!(
            new_strategy.long_call.option.strike_price,
            long_option.strike_price
        );
        assert!(new_strategy.max_profit().is_ok());
        assert!(new_strategy.max_loss().is_ok());
    }

    #[test]
    fn test_filter_combinations_empty_chain() {
        let strategy = create_test_strategy();
        let empty_chain =
            OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);
        let combinations: Vec<_> = strategy
            .filter_combinations(&empty_chain, FindOptimalSide::All)
            .collect();

        assert!(combinations.is_empty());
    }

    #[test]
    fn test_filter_combinations_invalid_prices() {
        let mut chain = create_mock_option_chain();
        // Add an option with invalid prices
        chain.add_option(
            pos!(110.0),
            None, // Invalid call_bid
            None, // Invalid call_ask
            spos!(1.0),
            spos!(1.2),
            spos!(0.2),
            Some(0.1),
            spos!(50.0),
            Some(25),
        );

        let strategy = create_test_strategy();
        let combinations: Vec<_> = strategy
            .filter_combinations(&chain, FindOptimalSide::All)
            .collect();

        // Verify that invalid options are filtered out
        for combination in combinations {
            match combination {
                OptionDataGroup::Two(short, long) => {
                    assert!(short.call_bid.is_some());
                    assert!(long.call_ask.is_some());
                }
                _ => panic!("Expected Two-leg combination"),
            }
        }
    }

    #[test]
    fn test_find_optimal_no_valid_combinations() {
        let mut strategy = create_test_strategy();
        let mut empty_chain =
            OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);
        // Add invalid options
        empty_chain.add_option(pos!(95.0), None, None, None, None, None, None, None, None);

        // Should not panic when no valid combinations exist
        strategy.find_optimal(
            &empty_chain,
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
        );

        // Strategy should remain unchanged
        assert!(strategy.validate());
    }

    #[test]
    #[should_panic]
    fn test_create_strategy_invalid_legs() {
        let strategy = create_test_strategy();
        let chain = create_mock_option_chain();

        // Test with invalid leg configuration
        let result = std::panic::catch_unwind(|| {
            strategy.create_strategy(
                &chain,
                &StrategyLegs::TwoLegs {
                    first: chain.options.iter().next().unwrap(),
                    second: chain.options.iter().next().unwrap(),
                },
            );
        });

        assert!(result.is_err());
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

#[cfg(test)]
mod tests_delta {
    use crate::model::types::{ExpirationDate, OptionStyle, PositiveF64};
    use crate::pos;
    use crate::strategies::bear_call_spread::BearCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use approx::assert_relative_eq;

    fn get_strategy(long_strike: PositiveF64, short_strike: PositiveF64) -> BearCallSpread {
        let underlying_price = pos!(5781.88);
        BearCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(2.0),
            0.18,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // long quantity
            85.04,     // premium_long
            29.85,     // premium_short
            0.78,      // open_fee_long
            0.78,      // open_fee_long
            0.73,      // close_fee_long
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5840.0), pos!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.08591,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.3660429216960246),
                strike: pos!(5840.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = pos!(0.3660429216960246);
        assert_relative_eq!(option.delta(), -0.0859127, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5800.0), pos!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.097145,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.3029931694406367),
                strike: pos!(5820.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = pos!(0.3029931694406367);
        assert_relative_eq!(option.delta(), 0.09714, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_delta_size {
    use crate::model::types::{ExpirationDate, OptionStyle, PositiveF64};
    use crate::pos;
    use crate::strategies::bear_call_spread::BearCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use approx::assert_relative_eq;

    fn get_strategy(long_strike: PositiveF64, short_strike: PositiveF64) -> BearCallSpread {
        let underlying_price = pos!(5781.88);
        BearCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(2.0),
            0.18,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(3.0), // long quantity
            85.04,     // premium_long
            29.85,     // premium_short
            0.78,      // open_fee_long
            0.78,      // open_fee_long
            0.73,      // close_fee_long
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5840.0), pos!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.2577383682099583,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(1.0981287650880742),
                strike: pos!(5840.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = pos!(1.0981287650880742);
        assert_relative_eq!(option.delta(), -0.257738, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5800.0), pos!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.291436,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.9089795083219099),
                strike: pos!(5820.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = pos!(0.9089795083219099);
        assert_relative_eq!(option.delta(), 0.29143, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}
