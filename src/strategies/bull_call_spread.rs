/*
Bull Call Spread Strategy

A bull call spread involves buying a call option with a lower strike price and selling a call option with a higher strike price,
both with the same expiration date. This strategy is used when a moderate rise in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (difference between strikes minus net debit)
- Limited risk (net debit paid)
- Bullish strategy that profits from price increase
- Both options have same expiration date
- Lower cost than buying calls outright
- Lower risk than naked call buying
- Maximum profit achieved when price rises above higher strike
- Also known as a vertical call debit spread
*/
use super::base::{BreakEvenable, Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN};
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::probability::ProbabilityError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::error::GreeksError;
use crate::greeks::Greeks;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{pos, Options, Positive};
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{debug, error, info};

const BULL_CALL_SPREAD_DESCRIPTION: &str =
    "A bull call spread is created by buying a call option with a lower strike price \
    and simultaneously selling a call option with a higher strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate increase in the underlying \
    asset's price. The maximum profit is limited to the difference between strike prices minus \
    the net debit paid, while the maximum loss is limited to the net debit paid.";

#[derive(Clone, Debug)]
pub struct BullCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    long_call: Position,
    short_call: Position,
}

impl BullCallSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut long_strike: Positive,
        mut short_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_short_call: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
    ) -> Self {
        if long_strike == Positive::ZERO {
            long_strike = underlying_price;
        }
        if short_strike == Positive::ZERO {
            short_strike = underlying_price;
        }

        let mut strategy = BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: BULL_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike,
            expiration.clone(),
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
            .expect("Failed to add long call");

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
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
            .expect("Failed to add short call");

        strategy.validate();

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl BreakEvenable for BullCallSpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_call.option.strike_price
                + self.net_cost()? / self.long_call.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Positionable for BullCallSpread {
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
        Ok(vec![&self.long_call, &self.short_call])
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
                side.clone(),
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
                side.clone(),
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
                    position.option.side.clone(),
                    "Put is not valid for PoorMansCoveredCall".to_string(),
                ))
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
                    position.option.side.clone(),
                    "Strike not found in positions".to_string(),
                ))
            }
        }

        Ok(())
    }
}

impl Strategies for BullCallSpread {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(self.short_call.option.strike_price)?;
        if profit >= Decimal::ZERO {
            Ok(profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(self.long_call.option.strike_price)?;
        if loss <= Decimal::ZERO {
            Ok(loss.abs().into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        }
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.max_profit().unwrap_or(Positive::ZERO);
        let base = self.short_call.option.strike_price - self.break_even_points[0];
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

impl Validable for BullCallSpread {
    fn validate(&self) -> bool {
        if !self.long_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        if !self.short_call.validate() {
            debug!("Short call is invalid");
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call.option.strike_price {
            error!(
                "Long call strike price {} must be lower than short call strike price {}",
                self.long_call.option.strike_price, self.short_call.option.strike_price
            );
            return false;
        }

        true
    }
}

impl Optimizable for BullCallSpread {
    type Strategy = BullCallSpread;

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
            .filter(|(long, short)| {
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long, short)| {
                let legs = StrategyLegs::TwoLegs {
                    first: long,
                    second: short,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long, short)| OptionDataGroup::Two(long, short))
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
            let (long, short) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: long,
                second: short,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        BullCallSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_call.option.expiration_date.clone(),
            long.implied_volatility.unwrap() / 100.0,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long.call_ask.unwrap(),
            short.call_bid.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
        )
    }
}

impl Profit for BullCallSpread {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(
            self.long_call.pnl_at_expiration(&price)?
                + self.short_call.pnl_at_expiration(&price)?,
        )
    }
}

impl Graph for BullCallSpread {
    fn title(&self) -> String {
        format!(
            "{} Strategy:\n\t{}\n\t{}",
            self.name,
            self.long_call.title(),
            self.short_call.title()
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let underlying_price = self.long_call.option.underlying_price.to_f64();
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
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Break Even {:.2}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Profit Point (at higher strike price)
        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.to_f64(),
                self.max_profit().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!(
                "Max Profit {:.2}",
                self.max_profit().unwrap_or(Positive::ZERO)
            ),
            label_offset: LabelOffsetType::Relative(10.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Loss Point (at lower strike price)
        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -self.max_loss().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(Positive::ZERO)),
            label_offset: LabelOffsetType::Relative(-120.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Current Price Point
        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for BullCallSpread {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.long_call.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_point),
            Some(self.short_call.option.strike_price),
            pos!(self.max_profit()?.to_f64()),
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
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(self.long_call.option.strike_price),
            Some(break_even_point),
            pos!(self.max_loss()?.to_f64()),
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

impl Greeks for BullCallSpread {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option, &self.short_call.option])
    }
}

impl DeltaNeutrality for BullCallSpread {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_delta = self.long_call.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let threshold = DELTA_THRESHOLD;

        let s_c_delta = short_call_delta.unwrap();
        let l_c_delta = long_call_delta.unwrap();
        DeltaInfo {
            net_delta: l_c_delta + s_c_delta,
            individual_deltas: vec![l_c_delta, s_c_delta],
            is_neutral: (l_c_delta + s_c_delta).abs() < threshold,
            underlying_price: self.long_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta = self.short_call.option.delta().unwrap();
        let qty = if delta == Decimal::ZERO {
            Positive::ONE
        } else {
            Positive((net_delta.abs() / delta).abs())
        };
        vec![DeltaAdjustment::SellOptions {
            quantity: qty * self.short_call.option.quantity,
            strike: self.short_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta = self.long_call.option.delta().unwrap();
        let qty = if delta == Decimal::ZERO {
            Positive::ONE
        } else {
            Positive((net_delta.abs() / delta).abs())
        };

        vec![DeltaAdjustment::BuyOptions {
            quantity: qty * self.long_call.option.quantity,
            strike: self.long_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }
}

#[cfg(test)]
fn bull_call_spread_test() -> BullCallSpread {
    use rust_decimal_macros::dec;
    let underlying_price = pos!(5781.88);
    BullCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5820.0),     // short_strike
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

#[cfg(test)]
mod tests_bull_call_spread_strategy {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_bull_call_spread() {
        let spread = bull_call_spread_test();

        assert_eq!(spread.name, "Bull Call Spread");
        assert_eq!(spread.kind, StrategyType::BullCallSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(spread.get_underlying_price(), pos!(5781.88));
        assert_eq!(spread.long_call.option.strike_price, pos!(5750.0));
        assert_eq!(spread.short_call.option.strike_price, pos!(5820.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_leg() {
        let mut spread = bull_call_spread_test();
        let new_long_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos!(1.5),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        );

        spread
            .add_position(&new_long_call.clone())
            .expect("Failed to add long call");
        assert_eq!(spread.long_call.option.strike_price, pos!(90.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_legs() {
        let spread = bull_call_spread_test();
        let legs = spread.get_positions().expect("Failed to get positions");

        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].option.side, Side::Long);
        assert_eq!(legs[1].option.side, Side::Short);
        assert_eq!(legs[0].option.option_style, OptionStyle::Call);
        assert_eq!(legs[1].option.option_style, OptionStyle::Call);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit() {
        let spread = bull_call_spread_test();
        let max_profit = spread.max_profit().unwrap();
        assert_eq!(max_profit, pos!(35.37));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss() {
        let spread = bull_call_spread_test();
        let max_loss = spread.max_loss().unwrap();
        assert_eq!(max_loss, pos!(174.63));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost() {
        let spread = bull_call_spread_test();
        assert_eq!(spread.total_cost().unwrap(), pos!(264.18));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::TWO,
            Positive::ONE,
            pos!(0.5), // open_fee_long_call
            pos!(0.5), // close_fee_long_call
            pos!(0.5), // open_fee_short_call
            pos!(0.5), // close_fee_short_call
        );

        assert_eq!(spread.fees().unwrap().to_f64(), 2.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_points() {
        let spread = bull_call_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        assert_eq!(break_even_points[0], pos!(5808.21));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_area() {
        let spread = bull_call_spread_test();
        let area = spread.profit_area().unwrap().to_f64().unwrap();
        assert_eq!(area, 2.0850615);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio() {
        let spread = bull_call_spread_test();
        let ratio = spread.profit_ratio().unwrap().to_f64().unwrap();
        assert_relative_eq!(ratio, 20.25425, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_default_strikes() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            Positive::ZERO, // long_strike = default
            Positive::ZERO, // short_strike = default
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

        assert_eq!(spread.long_call.option.strike_price, pos!(100.0));
        assert_eq!(spread.short_call.option.strike_price, pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_strikes() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(100.0),
            pos!(95.0),
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
}

#[cfg(test)]
mod tests_bull_call_spread_validation {
    use super::*;
    use crate::model::types::ExpirationDate;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_valid_position(
        side: Side,
        strike_price: Positive,
        expiration: ExpirationDate,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                expiration,
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            Positive::ONE,
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_bull_call_spread() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(spread.validate(), "Valid spread should pass validation");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_long_call() {
        let mut invalid_long =
            create_valid_position(Side::Long, pos!(95.0), ExpirationDate::Days(pos!(30.0)));
        invalid_long.option.quantity = Positive::ZERO;

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: invalid_long,
            short_call: create_valid_position(
                Side::Short,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with invalid long call should fail validation"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_short_call() {
        let mut invalid_short =
            create_valid_position(Side::Short, pos!(100.0), ExpirationDate::Days(pos!(30.0)));
        invalid_short.option.quantity = Positive::ZERO;

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_call: invalid_short,
        };

        assert!(
            !spread.validate(),
            "Spread with invalid short call should fail validation"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with long strike price >= short strike price should fail validation"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_equal_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with equal strike prices should fail validation"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_different_expiration_dates_same_day() {
        let date1 = ExpirationDate::DateTime(Utc::now() + chrono::Duration::days(30));
        let date2 = ExpirationDate::Days(pos!(30.0));

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, pos!(95.0), date1),
            short_call: create_valid_position(Side::Short, pos!(100.0), date2),
        };

        assert!(
            spread.validate(),
            "Spread with different ExpirationDate types but same date should pass validation"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_boundary_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos!(94.99),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            spread.validate(),
            "Spread with very close but valid strike prices should pass validation"
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_optimization {
    use super::*;
    use crate::chains::chain::OptionData;
    use crate::model::types::ExpirationDate;
    use crate::spos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        chain.add_option(
            pos!(85.0),      // strike
            spos!(16.0),     // call_bid
            spos!(16.2),     // call_ask
            None,            // put_bid
            None,            // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.8)), // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(90.0),
            spos!(11.5),
            spos!(11.7),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.7)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.5),
            spos!(3.7),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(250.0),
            Some(125),
        );

        chain.add_option(
            pos!(105.0),
            spos!(1.0),
            spos!(1.2),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(300.0),
            Some(150),
        );

        chain
    }

    fn create_base_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(7.2), // premium_long_call
            pos!(3.5), // premium_short_call
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_ratio() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.profit_ratio().unwrap().to_f64().unwrap() > 0.0,
            "Profit ratio should be positive"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_area() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.profit_area().unwrap().to_f64().unwrap() > 0.0,
            "Profit area should be positive"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_upper_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        assert!(spread.short_call.option.strike_price >= chain.underlying_price);
        assert!(spread.long_call.option.strike_price >= chain.underlying_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_lower_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Lower, OptimizationCriteria::Ratio);

        assert!(spread.short_call.option.strike_price <= chain.underlying_price);
        assert!(spread.long_call.option.strike_price <= chain.underlying_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_range() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(90.0), pos!(100.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(spread.short_call.option.strike_price <= pos!(100.0));
        assert!(spread.short_call.option.strike_price >= pos!(90.0));
        assert!(spread.long_call.option.strike_price <= pos!(100.0));
        assert!(spread.long_call.option.strike_price >= pos!(90.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_valid_long_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        assert!(spread.is_valid_long_option(&option, &FindOptimalSide::All));
        assert!(spread.is_valid_long_option(&option, &FindOptimalSide::Lower));
        assert!(!spread.is_valid_long_option(&option, &FindOptimalSide::Upper));
        assert!(
            spread.is_valid_long_option(&option, &FindOptimalSide::Range(pos!(90.0), pos!(100.0)))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_valid_short_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos!(105.0),
            spos!(1.0),
            spos!(1.2),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        assert!(spread.is_valid_short_option(&option, &FindOptimalSide::All));
        assert!(!spread.is_valid_short_option(&option, &FindOptimalSide::Lower));
        assert!(spread.is_valid_short_option(&option, &FindOptimalSide::Upper));
        assert!(!spread
            .is_valid_short_option(&option, &FindOptimalSide::Range(pos!(90.0), pos!(100.0))));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_are_valid_prices() {
        let spread = create_base_spread();
        let long_option = OptionData::new(
            pos!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );
        let short_option = OptionData::new(
            pos!(100.0),
            spos!(3.5),
            spos!(3.7),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::TwoLegs {
            first: &long_option,
            second: &short_option,
        };
        assert!(spread.are_valid_prices(&legs));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_prices() {
        let spread = create_base_spread();
        let long_option = OptionData::new(
            pos!(95.0),
            spos!(7.2),
            Some(Positive::ZERO),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );
        let short_option = OptionData::new(
            pos!(100.0),
            Some(Positive::ZERO),
            spos!(3.5),
            None,
            None,
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::TwoLegs {
            first: &long_option,
            second: &short_option,
        };
        assert!(!spread.are_valid_prices(&legs));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_strategy() {
        let spread = create_base_spread();
        let chain = create_test_chain();
        let long_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(95.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(100.0))
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs);

        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_call.option.strike_price, pos!(95.0));
        assert_eq!(new_strategy.short_call.option.strike_price, pos!(100.0));
        assert_eq!(
            new_strategy.long_call.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(
            new_strategy.short_call.option.option_style,
            OptionStyle::Call
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_below_long_strike() {
        let spread = bull_call_spread_test();
        let price = pos!(5800.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -24.63
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_at_long_strike() {
        let spread = bull_call_spread_test();
        let price = pos!(5807.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -3.63
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_between_strikes() {
        let spread = bull_call_spread_test();
        let price = pos!(5810.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            5.37
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_at_get_break_even_points() {
        let spread = bull_call_spread_test();
        let price = pos!(5808.21);
        assert!(spread.calculate_profit_at(price).unwrap().abs() < dec!(0.001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_at_short_strike() {
        let spread = bull_call_spread_test();
        let price = pos!(5818.21);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            30.0
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_above_short_strike() {
        let spread = bull_call_spread_test();
        let price = pos!(5908.21);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            35.37
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_with_multiple_contracts() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0),
            pos!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let price = pos!(105.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            6.0
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_with_fees() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(4.0),
            Positive::TWO,
            pos!(0.5), // open_fee_long_call
            pos!(0.5), // close_fee_long_call
            pos!(0.5), // open_fee_short_call
            pos!(0.5), // close_fee_short_call
        );

        let price = pos!(105.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            1.0
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_maximum_profit() {
        let spread = bull_call_spread_test();
        let price = pos!(5858.21);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            35.37
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_maximum_loss() {
        let spread = bull_call_spread_test();
        let price = pos!(5708.21);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -174.63
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title_format() {
        let spread = bull_call_spread_test();
        let title = spread.title();

        assert!(title.contains("Bull Call Spread Strategy"));
        assert!(title.contains("Long"));
        assert!(title.contains("Short"));
        assert!(title.contains("SP500")); // symbol
        assert!(title.contains("$5750")); // long strike
        assert!(title.contains("$5820")); // short strike
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_vertical_lines() {
        let spread = bull_call_spread_test();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1);

        let line = &lines[0];
        assert_eq!(line.x_coordinate, 5781.88);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price"));
        assert!(line.label.contains("5781.88"));
        assert_eq!(line.label_offset, (4.0, 0.0));
        assert_eq!(line.line_color, ORANGE);
        assert_eq!(line.label_color, ORANGE);
        assert_eq!(line.font_size, 18);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_points() {
        let spread = bull_call_spread_test();
        let points = spread.get_points();

        assert_eq!(points.len(), 4);

        let break_even = &points[0];
        assert_eq!(break_even.coordinates.1, 0.0);
        assert_eq!(break_even.coordinates.0, 5808.21);
        assert!(break_even.label.contains("Break Even"));
        assert_eq!(break_even.point_color, DARK_BLUE);
        assert_eq!(break_even.label_color, DARK_BLUE);
        assert_eq!(break_even.point_size, 5);
        assert_eq!(break_even.font_size, 18);

        let max_profit = &points[1];
        assert_eq!(max_profit.coordinates.0, 5820.0);
        assert_eq!(max_profit.coordinates.1, 35.37);
        assert!(max_profit.label.contains("Max Profit"));
        assert_eq!(max_profit.point_color, DARK_GREEN);
        assert_eq!(max_profit.label_color, DARK_GREEN);

        let max_loss = &points[2];
        assert_eq!(max_loss.coordinates.0, 5750.0);
        assert_eq!(max_loss.coordinates.1, -174.63);
        assert!(max_loss.label.contains("Max Loss"));
        assert_eq!(max_loss.point_color, RED);
        assert_eq!(max_loss.label_color, RED);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_points_coordinates() {
        let spread = bull_call_spread_test();
        let points = spread.get_points();

        // Break-even point
        assert_eq!(points[0].coordinates.0, 5808.21);
        assert_eq!(points[0].coordinates.1, 0.0);

        // Maximum profit point
        assert_eq!(points[1].coordinates.0, 5820.0);
        assert_eq!(points[1].coordinates.1, 35.37);

        // Maximum loss point
        assert_eq!(points[2].coordinates.0, 5750.0);
        assert_eq!(points[2].coordinates.1, -174.63);

        // Current price point
        assert_eq!(points[3].coordinates.0, 5781.88);
        let current_profit = spread
            .calculate_profit_at(pos!(5781.88))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(points[3].coordinates.1, current_profit);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_point_labels() {
        let spread = bull_call_spread_test();
        let points = spread.get_points();
        assert!(points[0].label.contains("5808.21")); // Break-even
        assert!(points[1].label.contains("35.37")); // Max profit
        assert!(points[2].label.contains("-174.63")); // Max loss
        assert!(points[3].label.contains("-78.99")); // Current price
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_points_style() {
        let spread = bull_call_spread_test();
        let points = spread.get_points();

        for point in points.iter() {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert!(matches!(point.point_color, DARK_BLUE | DARK_GREEN | RED));
            assert_eq!(point.point_color, point.label_color);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_with_zero_profits() {
        let mut spread = bull_call_spread_test();
        spread.short_call.premium = pos!(1.0);
        spread.long_call.premium = pos!(6.0);

        let points = spread.get_points();
        let max_profit_point = &points[1];

        assert_eq!(max_profit_point.coordinates.1, 185.94);
        assert!(max_profit_point.label.contains("185.94"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_with_different_quantities() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let points = spread.get_points();
        let max_profit_point = &points[1];
        let max_loss_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 6.0); // 2 * 3.0
        assert_eq!(max_loss_point.coordinates.1, -4.0); // 2 * -2.0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_at_extremes() {
        let spread = bull_call_spread_test();
        let profit_at_zero = spread.calculate_profit_at(Positive::ZERO).unwrap();
        let profit_at_high = spread.calculate_profit_at(pos!(1000.0)).unwrap();

        assert_eq!(profit_at_zero, dec!(-174.63));
        assert_eq!(profit_at_high, dec!(-174.63));
    }
}

#[cfg(test)]
mod tests_bull_call_spread_probability {
    use super::*;
    use crate::assert_pos_relative_eq;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let spread = bull_call_spread_test();
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
        let spread = bull_call_spread_test();
        assert_eq!(spread.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let spread = bull_call_spread_test();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.lower_bound.unwrap(), pos!(5808.21)); // Break-even
        assert_eq!(range.upper_bound.unwrap(), pos!(5820.0)); // Short strike
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let spread = bull_call_spread_test();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.lower_bound.unwrap(), pos!(5750.0)); // Long strike
        assert_eq!(range.upper_bound.unwrap(), pos!(5808.21)); // Break-even
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit() {
        let spread = bull_call_spread_test();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_volatility_adjustment() {
        let spread = bull_call_spread_test();
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
    fn test_probability_with_uptrend() {
        let spread = bull_call_spread_test();
        let trend = Some(PriceTrend {
            drift_rate: 0.8,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob < pos!(0.5));
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_downtrend() {
        let spread = bull_call_spread_test();
        let trend = Some(PriceTrend {
            drift_rate: -0.1,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob < pos!(0.5));
        assert!(prob > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_analyze_probabilities() {
        let spread = bull_call_spread_test();
        let result = spread.analyze_probabilities(None, None);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert_pos_relative_eq!(analysis.expected_value, Positive::ZERO, pos!(0.000001));
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_extreme_probabilities() {
        let spread = bull_call_spread_test();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_near_expiration() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(1.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());
        let prob = result.unwrap();

        assert!(prob < pos!(0.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_high_volatility() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.50), // Alta volatilidad
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());
        let prob = result.unwrap();

        assert!(prob < pos!(0.3));
        assert!(prob < pos!(0.7));
    }
}

#[cfg(test)]
mod tests_delta {
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bull_call_spread::BullCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos, Positive};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullCallSpread {
        let underlying_price = pos!(5781.88);
        BullCallSpread::new(
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
        let strike = pos!(5820.0);
        let strategy = get_strategy(pos!(5750.0), strike);
        let size = dec!(0.3502);
        let delta = pos!(1.092269393430898);
        let k = pos!(5820.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();

        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5850.0), pos!(5820.0));
        let size = dec!(-0.1234671);
        let delta = pos!(0.626251716937553);
        let k = pos!(5850.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_delta_size {
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bull_call_spread::BullCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos, Positive};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullCallSpread {
        let underlying_price = pos!(5781.88);
        BullCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
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
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5750.0), pos!(5820.9));
        let size = dec!(0.7086);
        let delta = pos!(2.239306943523854);
        let k = pos!(5820.9);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5850.0), pos!(5820.0));
        let size = dec!(-0.246934);
        let delta = pos!(1.252503433875106);
        let k = pos!(5850.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_bull_call_spread_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_bull_call_spread() -> BullCallSpread {
        BullCallSpread::new(
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
    fn test_short_bull_call_spread_get_position() {
        let mut bull_call_spread = create_test_bull_call_spread();

        // Test getting short call position
        let call_position =
            bull_call_spread.get_position(&OptionStyle::Call, &Side::Long, &pos!(5750.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5750.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            bull_call_spread.get_position(&OptionStyle::Call, &Side::Short, &pos!(5820.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5820.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            bull_call_spread.get_position(&OptionStyle::Call, &Side::Short, &pos!(5821.0));
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
    fn test_short_bull_call_spread_modify_position() {
        let mut bull_call_spread = create_test_bull_call_spread();

        // Modify short call position
        let mut modified_call = bull_call_spread.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = bull_call_spread.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(bull_call_spread.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = bull_call_spread.long_call.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = bull_call_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bull_call_spread.long_call.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = bull_call_spread.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = bull_call_spread.modify_position(&invalid_position);
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
mod tests_adjust_option_position {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullCallSpread {
        BullCallSpread::new(
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
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment,
            &pos!(5750.0),
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
            adjustment,
            &pos!(5820.0),
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
            pos!(1.0),
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
            pos!(1.0),
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
            Positive::ZERO,
            &pos!(5820.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}
