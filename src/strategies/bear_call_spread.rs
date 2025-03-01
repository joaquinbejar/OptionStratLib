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
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyType, Validable,
};
use crate::Options;
use crate::Positive;
use crate::chains::StrategyLegs;
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::constants::{DARK_BLUE, DARK_GREEN};
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::probability::ProbabilityError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::error::{GreeksError, OperationErrorKind};
use crate::greeks::Greeks;
use crate::model::ProfitLossRange;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::strategies::{StrategyBasics, StrategyConstructor};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{RED, ShapeStyle};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::debug;

const BEAR_CALL_SPREAD_DESCRIPTION: &str = "A bear call spread is created by selling a call option with a lower strike price \
    and simultaneously buying a call option with a higher strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate decline in the underlying \
    asset's price. The maximum profit is limited to the net credit received, while the maximum \
    loss is limited to the difference between strike prices minus the net credit.";

#[derive(Clone, Debug)]
pub struct BearCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    short_call: Position,
    long_call: Position,
}

impl BearCallSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut short_strike: Positive,
        mut long_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        premium_long_call: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
    ) -> Self {
        if short_strike == Positive::ZERO {
            short_strike = underlying_price;
        }
        if long_strike == Positive::ZERO {
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
            expiration,
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

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for BearCallSpread {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a bear call spread
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bear Call Spread get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by strike price to identify short and long positions
        let mut sorted_options = vec_options.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let lower_strike_option = &sorted_options[0];
        let higher_strike_option = &sorted_options[1];

        // Validate options are calls
        if lower_strike_option.option.option_style != OptionStyle::Call
            || higher_strike_option.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bear Call Spread get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option sides
        if lower_strike_option.option.side != Side::Short
            || higher_strike_option.option.side != Side::Long
        {
            return Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "Bear Call Spread get_strategy".to_string(),
                reason: "Bear Call Spread requires a short lower strike call and a long higher strike call".to_string(),
            }));
        }

        // Validate expiration dates match
        if lower_strike_option.option.expiration_date != higher_strike_option.option.expiration_date
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bear Call Spread get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let short_call = Position::new(
            lower_strike_option.option.clone(),
            lower_strike_option.premium,
            Utc::now(),
            lower_strike_option.open_fee,
            lower_strike_option.close_fee,
        );

        let long_call = Position::new(
            higher_strike_option.option.clone(),
            higher_strike_option.premium,
            Utc::now(),
            higher_strike_option.open_fee,
            higher_strike_option.close_fee,
        );

        // Create strategy
        let mut strategy = BearCallSpread {
            name: "Bear Call Spread".to_string(),
            kind: StrategyType::BearCallSpread,
            description: BEAR_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call,
            long_call,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for BearCallSpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.short_call.option.strike_price
                + self.net_premium_received()? / self.short_call.option.quantity)
                .round_to(2),
        );

        Ok(())
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

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Call)
    /// * `side` - The side of the position (Long/Short)
    /// * `strike` - The strike price of the option
    ///
    /// # Returns
    /// * `Ok(Vec<&mut Position>)` - A vector containing mutable references to matching positions
    /// * `Err(PositionError)` - If there was an error retrieving positions
    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        match (side, option_style, strike) {
            (_, OptionStyle::Put, _) => Err(PositionError::invalid_position_type(
                *side,
                "Put is not valid for BearCallSpread".to_string(),
            )),
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Strike not found in positions".to_string(),
            )),
        }
    }

    /// Modifies an existing position in the strategy.
    ///
    /// # Arguments
    /// * `position` - The new position data to update
    ///
    /// # Returns
    /// * `Ok(())` if position was successfully modified
    /// * `Err(PositionError)` if position was not found or validation failed
    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        match (
            &position.option.side,
            &position.option.option_style,
            &position.option.strike_price,
        ) {
            (_, OptionStyle::Put, _) => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Put is not valid for PoorMansCoveredCall".to_string(),
                ));
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                self.long_call = position.clone();
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
            }
            _ => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Strike not found in positions".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Strategable for BearCallSpread {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for BearCallSpread {
    fn get_underlying_price(&self) -> Positive {
        self.short_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let net_premium_received = self.net_premium_received()?;
        if net_premium_received < Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        } else {
            Ok(net_premium_received)
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let width = self.long_call.option.strike_price - self.short_call.option.strike_price;
        let max_loss =
            (width * self.short_call.option.quantity).to_dec() - self.net_premium_received()?;
        if max_loss < Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        } else {
            Ok(Positive(max_loss))
        }
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.max_profit().unwrap_or(Positive::ZERO);
        let base = self.break_even_points[0] - self.short_call.option.strike_price;
        Ok((high * base / 200.0).into())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
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
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
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
        let mut best_value = Decimal::MIN;
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
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
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
            self.short_call.option.expiration_date,
            short.implied_volatility.unwrap() / 100.0,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            short.call_bid.unwrap(),
            long.call_ask.unwrap(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.long_call.open_fee,
            self.long_call.close_fee,
        )
    }
}

impl Profit for BearCallSpread {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(
            self.short_call.pnl_at_expiration(&price)?
                + self.long_call.pnl_at_expiration(&price)?,
        )
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
        let underlying_price = self.short_call.option.underlying_price.to_f64();
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
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Break Even {:.2}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.into(),
                self.max_profit().unwrap_or(Positive::ZERO).into(),
            ),
            label: format!(
                "Max Profit {:.2}",
                self.max_profit().unwrap_or(Positive::ZERO)
            ),
            label_offset: LabelOffsetType::Relative(-60.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -self.max_loss().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(Positive::ZERO)),
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
        Ok(self.short_call.option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.short_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call.option.implied_volatility,
            self.long_call.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(None, Some(break_even_point), Positive::ZERO)?;

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
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call.option.implied_volatility,
            self.long_call.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(None, Some(break_even_point), Positive::ZERO)?;

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
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option, &self.long_call.option])
    }
}

impl DeltaNeutrality for BearCallSpread {}

impl PnLCalculator for BearCallSpread {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .short_call
                .calculate_pnl(market_price, expiration_date, implied_volatility)?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_call
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

#[cfg(test)]
mod tests_bear_call_spread_strategies {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::{assert_pos_relative_eq, pos};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(95.0),                       // short_strike
            pos!(105.0),                      // long_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.20),                       // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(10.0),                       // premium_short_call
            pos!(5.0),                        // premium_long_call
            pos!(0.5),                        // open_fee_short_call
            pos!(0.5),                        // close_fee_short_call
            pos!(0.5),                        // open_fee_long_call
            pos!(0.5),                        // close_fee_long_call
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_underlying_price() {
        let spread = create_test_spread();
        assert_eq!(spread.get_underlying_price(), pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit_positive() {
        let spread = create_test_spread();
        let result = spread.max_profit();
        assert!(result.is_ok());
        assert_relative_eq!(
            result.unwrap().to_f64(),
            spread.net_premium_received().unwrap().to_f64(),
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit_zero() {
        let mut spread = create_test_spread();
        // Modify premiums to create negative net premium
        spread.short_call.premium = pos!(1.0);
        spread.long_call.premium = pos!(2.0);

        let result = spread.max_profit();
        assert!(result.is_ok());
        assert_relative_eq!(result.unwrap().to_f64(), 0.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss() {
        let spread = create_test_spread();
        let result = spread.max_loss();
        assert!(result.is_ok());

        let width =
            (spread.long_call.option.strike_price - spread.short_call.option.strike_price).to_f64();
        let expected_loss = width * spread.short_call.option.quantity.to_f64()
            - spread.net_premium_received().unwrap().to_f64();
        assert_relative_eq!(result.unwrap().to_f64(), expected_loss, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost() {
        let spread = create_test_spread();
        let expected_cost = 7.0;
        assert_relative_eq!(
            spread.total_cost().unwrap().to_f64(),
            expected_cost,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received() {
        let spread = create_test_spread();
        let expected_premium = spread.short_call.net_premium_received().unwrap()
            - spread.long_call.net_cost().unwrap();
        assert_pos_relative_eq!(
            spread.net_premium_received().unwrap(),
            expected_premium,
            pos!(0.0001)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let spread = create_test_spread();
        let expected_fees = (spread.short_call.open_fee
            + spread.short_call.close_fee
            + spread.long_call.open_fee
            + spread.long_call.close_fee)
            .to_f64();
        assert_relative_eq!(
            spread.fees().unwrap().to_f64(),
            expected_fees,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_area() {
        let spread = create_test_spread();
        let high = spread.max_profit().unwrap_or(Positive::ZERO);
        let base = spread.break_even_points[0] - spread.short_call.option.strike_price;
        let expected_area = (high * base / 200.0).to_f64();
        assert_relative_eq!(
            spread.profit_area().unwrap().to_f64().unwrap(),
            expected_area,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio_normal() {
        let spread = create_test_spread();
        let max_profit = spread.max_profit().unwrap();
        let max_loss = spread.max_loss().unwrap();
        let expected_ratio = (max_profit / max_loss * 100.0).to_f64();
        assert_relative_eq!(
            spread.profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio_zero_profit() {
        let mut spread = create_test_spread();
        // Modify premiums to create zero max profit
        spread.short_call.premium = Positive::ONE;
        spread.long_call.premium = Positive::ONE;

        assert_relative_eq!(
            spread.profit_ratio().unwrap().to_f64().unwrap(),
            0.0,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio_zero_loss() {
        let mut spread = create_test_spread();
        // Modify strikes to create zero max loss scenario
        spread.long_call.option.strike_price = spread.short_call.option.strike_price;

        assert_eq!(spread.profit_ratio().unwrap(), Decimal::MAX);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_break_even_points() {
        let spread = create_test_spread();
        let break_even_points = spread.get_break_even_points().unwrap();
        assert!(!break_even_points.is_empty());
        assert_eq!(break_even_points.len(), 1);

        // Break even should be short strike plus net premium received per contract
        let expected_break_even = spread.short_call.option.strike_price
            + pos!(
                spread.net_premium_received().unwrap().to_f64()
                    / spread.short_call.option.quantity.to_f64()
            );
        assert_relative_eq!(
            break_even_points[0].to_f64(),
            expected_break_even.to_f64(),
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_different_quantities() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.20),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(2.0),
            pos!(1.0),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
        );

        // Check that all calculations scale properly with quantity
        assert_relative_eq!(spread.max_profit().unwrap().to_f64(), 0.0, epsilon = 0.0001);
        assert_relative_eq!(spread.max_loss().unwrap().to_f64(), 20.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_different_strikes() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),  // wider spread
            pos!(110.0), // wider spread
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
        );

        // Check that strike width affects max loss calculation
        let base_spread = create_test_spread();
        assert!(spread.max_loss().unwrap() > base_spread.max_loss().unwrap());
    }
}

#[cfg(test)]
mod tests_bear_call_spread_positionable {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::{Options, pos};
    use chrono::Utc;
    use rust_decimal_macros::dec;

    // Helper function to create a test option
    fn create_test_option(side: Side) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            pos!(1.0),
            pos!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        )
    }

    // Helper function to create a test position
    fn create_test_position(side: Side) -> Position {
        Position::new(
            create_test_option(side),
            pos!(1.0),      // premium
            Utc::now(),     // timestamp
            Positive::ZERO, // open_fee
            Positive::ZERO, // close_fee
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_short_position() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let short_position = create_test_position(Side::Short);
        let result = spread.add_position(&short_position);

        assert!(result.is_ok());
        assert_eq!(spread.short_call.option.side, Side::Short);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_long_position() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let long_position = create_test_position(Side::Long);
        let result = spread.add_position(&long_position);

        assert!(result.is_ok());
        assert_eq!(spread.long_call.option.side, Side::Long);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_positions() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let result = spread.get_positions();
        assert!(result.is_ok());

        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].option.side, Side::Short);
        assert_eq!(positions[1].option.side, Side::Long);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_multiple_positions() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let short_position = create_test_position(Side::Short);
        let long_position = create_test_position(Side::Long);

        assert!(spread.add_position(&short_position).is_ok());
        assert!(spread.add_position(&long_position).is_ok());

        let positions = spread.get_positions().unwrap();
        assert_eq!(positions.len(), 2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_replace_positions() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        // Create new positions
        let new_short = create_test_position(Side::Short);
        let new_long = create_test_position(Side::Long);

        // Replace positions
        assert!(spread.add_position(&new_short).is_ok());
        assert!(spread.add_position(&new_long).is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_positions_integrity() {
        let mut spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
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
    use rust_decimal_macros::dec;

    fn create_valid_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(95.0),                       // short_strike
            pos!(105.0),                      // long_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(2.0),                        // premium_short_call
            pos!(1.0),                        // premium_long_call
            Positive::ZERO,                   // fees
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_spread() {
        let spread = create_valid_spread();
        assert!(spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_strike_order() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0), // short strike higher than long strike
            pos!(95.0),  // long strike lower than short strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );
        assert!(!spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_equal_strikes() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(100.0), // both strikes equal
            pos!(100.0), // both strikes equal
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );
        assert!(!spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_short_call() {
        let mut spread = create_valid_spread();
        // Invalidate short call by setting an invalid quantity
        spread.short_call.option.quantity = Positive::ZERO;
        assert!(!spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_long_call() {
        let mut spread = create_valid_spread();
        // Invalidate long call by setting an invalid quantity
        spread.long_call.option.quantity = Positive::ZERO;
        assert!(!spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic]
    fn test_invalid_expiration_dates() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(Positive::ZERO), // Invalid expiration (0 days)
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );
        assert!(!spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_underlying_price() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            Positive::ZERO, // Invalid underlying price
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );
        assert!(!spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strikes_too_close() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(99.999),  // Strikes very close to each other
            pos!(100.001), // but technically different
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );
        // Should still be valid as long as strikes are different
        assert!(spread.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validation_with_different_quantities() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // Different quantity
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );
        // Should be valid as quantity > 0
        assert!(spread.validate());
    }
}

#[cfg(test)]
mod tests_bear_call_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::pricing::payoff::Profit;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(95.0),                       // short_strike
            pos!(105.0),                      // long_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(2.0),                        // premium_short_call
            pos!(1.0),                        // premium_long_call
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_long_call
            Positive::ZERO,                   // close_fee_long_call
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_below_short_strike() {
        let spread = create_test_spread();
        let profit = spread
            .calculate_profit_at(pos!(90.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // When price is below short strike, both options expire worthless
        // Profit should be the net premium received
        let expected_profit = spread.net_premium_received().unwrap().to_f64();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_at_short_strike() {
        let spread = create_test_spread();
        let profit = spread
            .calculate_profit_at(pos!(95.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // At short strike, short call is at-the-money
        let expected_profit = spread.net_premium_received().unwrap().to_f64();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_between_strikes() {
        let spread = create_test_spread();
        let test_price = pos!(100.0);
        let profit = spread
            .calculate_profit_at(test_price)
            .unwrap()
            .to_f64()
            .unwrap();
        // Between strikes, only short call is in-the-money
        let intrinsic_value = test_price - spread.short_call.option.strike_price;
        let expected_profit =
            spread.net_premium_received().unwrap().to_f64() - intrinsic_value.to_f64();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_at_long_strike() {
        let spread = create_test_spread();
        let profit = spread
            .calculate_profit_at(pos!(105.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // At long strike, both options are in-the-money
        let short_intrinsic = pos!(105.0) - spread.short_call.option.strike_price;
        let long_intrinsic = pos!(105.0) - spread.long_call.option.strike_price;
        let expected_profit = spread.net_premium_received().unwrap().to_f64()
            - short_intrinsic.to_f64()
            + long_intrinsic.to_f64();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_above_long_strike() {
        let spread = create_test_spread();
        let profit = spread
            .calculate_profit_at(pos!(110.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // Maximum loss occurs when price is above long strike
        let expected_profit = -spread.max_loss().unwrap().to_f64();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_at_get_break_even_points() {
        let spread = create_test_spread();
        let break_even = spread.get_break_even_points().unwrap()[0];
        let profit = spread
            .calculate_profit_at(break_even)
            .unwrap()
            .to_f64()
            .unwrap();
        // At break-even point, profit should be zero
        assert_relative_eq!(profit, 0.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_with_different_quantities() {
        let spread = BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0),      // quantity = 2
            pos!(2.0),      // premium_short_call
            pos!(1.0),      // premium_long_call
            Positive::ZERO, // open_fee_short_call
            Positive::ZERO, // close_fee_short_call
            Positive::ZERO, // open_fee_long_call
            Positive::ZERO, // close_fee_long_call
        );

        let profit = spread
            .calculate_profit_at(pos!(90.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // With quantity = 2, profit should be double
        let expected_profit = spread.net_premium_received().unwrap().to_f64();
        assert_relative_eq!(profit, expected_profit, epsilon = 0.0001);
        assert_relative_eq!(
            profit,
            2.0 * create_test_spread()
                .calculate_profit_at(pos!(90.0))
                .unwrap()
                .to_f64()
                .unwrap(),
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_with_fees() {
        let spread = BearCallSpread::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // long_strike_itm
            pos!(5820.0),  // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        );

        let profit = spread
            .calculate_profit_at(pos!(90.0))
            .unwrap()
            .to_f64()
            .unwrap();
        let fees = 6.04;
        assert_eq!(spread.fees().unwrap().to_f64(), fees);

        assert_relative_eq!(profit, 104.34, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_bear_call_spread_optimizable {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    // Helper function to create a mock OptionChain for testing
    fn create_mock_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);

        // Add options with different strikes and prices
        chain.add_option(
            pos!(95.0),      // strike
            spos!(6.0),      // call_bid
            spos!(6.2),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.2),      // put_ask
            spos!(0.2),      // implied_vol
            Some(dec!(0.7)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
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
            Some(dec!(0.5)),
            Some(dec!(0.3)),
            Some(dec!(0.3)),
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
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            Some(dec!(0.3)),
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
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(3.0),
            pos!(1.2),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_ratio() {
        let mut strategy = create_test_strategy();
        let chain = create_mock_option_chain();

        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        // Verify the strategy was updated with optimal values
        assert!(strategy.validate());
        assert!(strategy.max_profit().is_ok());
        assert!(strategy.max_loss().is_ok());
        assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_area() {
        let mut strategy = create_test_strategy();
        let chain = create_mock_option_chain();

        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        // Verify the strategy was updated with optimal values
        assert!(strategy.validate());
        assert!(strategy.max_profit().is_ok());
        assert!(strategy.max_loss().is_ok());
        assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            Some(dec!(0.1)),
            Some(dec!(0.3)),
            Some(dec!(0.3)),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_no_valid_combinations() {
        let mut strategy = create_test_strategy();
        let mut empty_chain =
            OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);
        // Add invalid options
        empty_chain.add_option(
            pos!(95.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(105.0),                      // short_strike
            pos!(110.0),                      // long_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(2.0),
            pos!(1.0),
            Positive::ZERO, // open_fee_short_call
            Positive::ZERO, // close_fee_short_call
            Positive::ZERO, // open_fee_long_call
            Positive::ZERO, // close_fee_long_call
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title() {
        let spread = create_test_spread();
        let title = spread.title();
        assert!(title.contains("Bear Call Spread Strategy"));
        assert!(title.contains("Short"));
        assert!(title.contains("Long"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vertical_lines() {
        let spread = create_test_spread();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1); // Current price, short strike, long strike
        assert_eq!(lines[0].x_coordinate, 100.0);
        assert!(lines[0].label.contains("Current Price"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
        assert_eq!(
            values[0],
            spread
                .calculate_profit_at(pos!(95.0))
                .unwrap()
                .to_f64()
                .unwrap()
        );
        assert_eq!(
            values[1],
            spread
                .calculate_profit_at(pos!(100.0))
                .unwrap()
                .to_f64()
                .unwrap()
        );
        assert_eq!(
            values[2],
            spread
                .calculate_profit_at(pos!(105.0))
                .unwrap()
                .to_f64()
                .unwrap()
        );
        assert_eq!(
            values[3],
            spread
                .calculate_profit_at(pos!(110.0))
                .unwrap()
                .to_f64()
                .unwrap()
        );
        assert_eq!(
            values[4],
            spread
                .calculate_profit_at(pos!(115.0))
                .unwrap()
                .to_f64()
                .unwrap()
        );
        assert_eq!(
            values[0],
            spread.max_profit().unwrap_or(Positive::ZERO).to_f64()
        );
        assert_eq!(
            values[4],
            -spread.max_loss().unwrap_or(Positive::ZERO).to_f64()
        );
    }
}

#[cfg(test)]
mod tests_bear_call_spread_probability {
    use super::*;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearCallSpread {
        BearCallSpread::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // long_strike_itm
            pos!(5820.0),  // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let spread = create_test_spread();
        let result = spread.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 2.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let spread = create_test_spread();
        assert_eq!(spread.get_risk_free_rate().unwrap().to_f64().unwrap(), 0.05);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let spread = create_test_spread();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_none());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let spread = create_test_spread();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_none());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit() {
        let spread = create_test_spread();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_volatility_adjustment() {
        let spread = create_test_spread();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let result = spread.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_trend() {
        let spread = create_test_spread();
        let trend = Some(PriceTrend {
            drift_rate: -0.1,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_analyze_probabilities() {
        let spread = create_test_spread();
        let result = spread.analyze_probabilities(None, None);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert!(analysis.expected_value > Positive::ZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_extreme_probabilities() {
        let spread = create_test_spread();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bear_call_spread::BearCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BearCallSpread {
        let underlying_price = pos!(5781.88);
        BearCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strike = pos!(5870.0);
        let strategy = get_strategy(strike, pos!(5860.0));
        let size = dec!(0.0296);
        let delta = pos!(0.2210336595644664);
        let k = pos!(5870.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_increasing_adjustments() {
        let strike = pos!(5820.0);
        let strategy = get_strategy(pos!(5800.0), strike);
        let size = dec!(-0.0971);
        let delta = pos!(0.23253626);
        let k = pos!(5800.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta - strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_delta_size {
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bear_call_spread::BearCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{Positive, Side, assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BearCallSpread {
        let underlying_price = pos!(5781.88);
        BearCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strike = pos!(5840.4);
        let strategy = get_strategy(strike, pos!(5820.5));
        let size = dec!(0.2555);
        let delta = pos!(1.09617639141894);
        let k = pos!(5840.4);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();

        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_increasing_adjustments() {
        let strike = pos!(5820.414);
        let strategy = get_strategy(pos!(5800.0), strike);
        let size = dec!(-0.2971);
        let delta = pos!(0.932384393100519);
        let k = pos!(5820.414);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_bear_call_spread_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_short_bear_call_spread() -> BearCallSpread {
        BearCallSpread::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // short_call_strike
            pos!(5820.0),  // long_call_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    fn test_short_bear_call_spread_get_position() {
        let mut bear_call_spread = create_test_short_bear_call_spread();

        // Test getting short call position
        let call_position =
            bear_call_spread.get_position(&OptionStyle::Call, &Side::Long, &pos!(5820.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5820.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            bear_call_spread.get_position(&OptionStyle::Call, &Side::Short, &pos!(5750.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5750.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            bear_call_spread.get_position(&OptionStyle::Call, &Side::Short, &pos!(5821.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => {
                error!("Unexpected error: {:?}", invalid_position);
                panic!()
            }
        }
    }

    #[test]
    fn test_short_bear_call_spread_modify_position() {
        let mut bear_call_spread = create_test_short_bear_call_spread();

        // Modify short call position
        let mut modified_call = bear_call_spread.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = bear_call_spread.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(bear_call_spread.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = bear_call_spread.long_call.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = bear_call_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bear_call_spread.long_call.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = bear_call_spread.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = bear_call_spread.modify_position(&invalid_position);
        assert!(result.is_err());
        match result {
            Err(PositionError::ValidationError(kind)) => match kind {
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                } => {
                    assert_eq!(reason, "Strike not found in positions");
                }
                _ => panic!("Expected ValidationError::InvalidPosition"),
            },
            _ => panic!("Expected ValidationError"),
        }
    }
}

#[cfg(test)]
mod tests_adjust_option_position_short {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BearCallSpread {
        BearCallSpread::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // short_call_strike
            pos!(5820.0),  // long_call_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5820.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.long_call.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5750.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_call.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Put,
            &Side::Long,
        );

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<PositionError>() {
            Some(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Put is not valid for BearCallSpread");
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0), // Invalid strike price
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_quantity_adjustment() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(5750.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_strategy_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = BearCallSpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.short_call.option.strike_price, pos!(95.0));
        assert_eq!(strategy.long_call.option.strike_price, pos!(105.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        )];

        let result = BearCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Call Spread get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        option1.option.option_style = OptionStyle::Put;
        let option2 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        let options = vec![option1, option2];
        let result = BearCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Call Spread get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(115.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];
        let result = BearCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Call Spread get_strategy"
                && reason == "Bear Call Spread requires a short lower strike call and a long higher strike call"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let options = vec![option1, option2];
        let result = BearCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Call Spread get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_bear_call_spread_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_bear_call_spread() -> Result<BearCallSpread, StrategyError> {
        // Create short call with lower strike
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Strike price (ATM)
            pos!(0.2),   // Implied volatility
        );

        // Create long call with higher strike
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Higher strike price
            pos!(0.2),   // Implied volatility
        );

        BearCallSpread::get_strategy(&vec![short_call, long_call])
    }

    #[test]
    fn test_calculate_pnl_below_strikes() {
        let spread = create_test_bear_call_spread().unwrap();
        let market_price = pos!(95.0); // Below both strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options OTM, should be close to max profit
        // Initial income: Premium from short call (5.0)
        // Initial costs: Premium for long call (5.0) + total fees (2.0)
        assert_pos_relative_eq!(pnl.initial_income, pos!(5.0), pos!(1e-6));
        assert_pos_relative_eq!(pnl.initial_costs, pos!(7.0), pos!(1e-6));
        assert!(pnl.unrealized.unwrap() > dec!(-2.0)); // Should be near max profit
    }

    #[test]
    fn test_calculate_pnl_between_strikes() {
        let spread = create_test_bear_call_spread().unwrap();
        let market_price = pos!(102.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.1);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Short call ITM, long call OTM
        assert!(pnl.unrealized.unwrap() < dec!(-0.5)); // Some loss
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // But not max loss
    }

    #[test]
    fn test_calculate_pnl_above_strikes() {
        let spread = create_test_bear_call_spread().unwrap();
        let market_price = pos!(110.0); // Above both strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options ITM, should be near max loss
        assert!(pnl.unrealized.unwrap() < dec!(-2.0)); // Close to max loss
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // But not worse than max loss
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let spread = create_test_bear_call_spread().unwrap();
        let underlying_price = pos!(95.0); // Below both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, both options expire worthless
        // Max profit is the net premium received minus fees
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6)); // Premium received - costs
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let spread = create_test_bear_call_spread().unwrap();
        let underlying_price = pos!(110.0); // Well above both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss = spread width (5.0) - net premium received (0.0) + fees (2.0)
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-7.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_between_strikes() {
        let spread = create_test_bear_call_spread().unwrap();
        let underlying_price = pos!(102.5); // Between strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Loss should be: (102.5 - 100) = 2.5 intrinsic value of short call
        // Plus costs (7.0) minus income (5.0)
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-4.5), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let spread = create_test_bear_call_spread().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, both options are worth more
        // Net effect should be slightly negative as short gamma position
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still capped by the spread width
        assert!(pnl.unrealized.unwrap() > dec!(-5.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_at_short_strike() {
        let spread = create_test_bear_call_spread().unwrap();
        let underlying_price = pos!(100.0); // At short strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At the short strike, short call is ATM
        // Loss should be just the costs minus income
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6));
    }
}
