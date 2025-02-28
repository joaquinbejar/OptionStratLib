/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyType, Validable,
};
use crate::Options;
use crate::chains::StrategyLegs;
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::constants::{DARK_BLUE, DARK_GREEN};
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::strategies::{BreakEvenErrorKind, ProfitLossErrorKind, StrategyError};
use crate::error::{GreeksError, OperationErrorKind, ProbabilityError};
use crate::greeks::Greeks;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::model::{Position, ProfitLossRange};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::strategies::{StrategyBasics, StrategyConstructor};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{Positive, spos};
use chrono::Utc;
use num_traits::ToPrimitive;
use plotters::prelude::{RED, ShapeStyle};
use plotters::style::full_palette::ORANGE;
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{error, info};

const CALL_BUTTERFLY_DESCRIPTION: &str = "A Ratio Call Spread involves buying one call option and selling multiple call options \
    at a higher strike price. This strategy is used when a moderate rise in the underlying \
    asset's price is expected, but with limited upside potential.";

#[derive(Clone, Debug)]
pub struct CallButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    long_call: Position,
    short_call_low: Position,
    short_call_high: Position,
}

impl CallButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        long_call_strike: Positive,
        short_call_low_strike: Positive,
        short_call_high_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_short_call_low: Positive,
        premium_short_call_high: Positive,
        open_fee_long: Positive,
        close_fee_long: Positive,
        open_fee_short_low: Positive,
        close_fee_short_low: Positive,
        open_fee_short_high: Positive,
        close_fee_short_high: Positive,
    ) -> Self {
        let mut strategy = CallButterfly {
            name: underlying_symbol.to_string(),
            kind: StrategyType::CallButterfly,
            description: CALL_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call_low: Position::default(),
            short_call_high: Position::default(),
        };
        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_call_strike,
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
            open_fee_long,
            close_fee_long,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid short call");
        strategy.long_call = long_call;

        let short_call_low_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_low_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_low = Position::new(
            short_call_low_option,
            premium_short_call_low,
            Utc::now(),
            open_fee_short_low,
            close_fee_short_low,
        );
        strategy
            .add_position(&short_call_low.clone())
            .expect("Invalid long call itm");
        strategy.short_call_low = short_call_low;

        let short_call_high_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_high = Position::new(
            short_call_high_option,
            premium_short_call_high,
            Utc::now(),
            open_fee_short_high,
            close_fee_short_high,
        );
        strategy
            .add_position(&short_call_high.clone())
            .expect("Invalid long call otm");
        strategy.short_call_high = short_call_high;

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for CallButterfly {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 3 options for a call butterfly
        if vec_options.len() != 3 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Call Butterfly get_strategy".to_string(),
                    reason: "Must have exactly 3 options".to_string(),
                },
            ));
        }

        // Sort options by strike price
        let mut sorted_options = vec_options.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let low_short_call = &sorted_options[0];
        let long_call = &sorted_options[1];
        let high_short_call = &sorted_options[2];

        // Validate options are calls
        if low_short_call.option.option_style != OptionStyle::Call
            || long_call.option.option_style != OptionStyle::Call
            || high_short_call.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Call Butterfly get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option sides
        if low_short_call.option.side != Side::Short
            || long_call.option.side != Side::Long
            || high_short_call.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Call Butterfly get_strategy".to_string(),
                    reason: "Call Butterfly requires one long call and two short calls".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if low_short_call.option.expiration_date != long_call.option.expiration_date
            || long_call.option.expiration_date != high_short_call.option.expiration_date
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Call Butterfly get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let short_call_low = Position::new(
            low_short_call.option.clone(),
            low_short_call.premium,
            Utc::now(),
            low_short_call.open_fee,
            low_short_call.close_fee,
        );

        let long_call = Position::new(
            long_call.option.clone(),
            long_call.premium,
            Utc::now(),
            long_call.open_fee,
            long_call.close_fee,
        );

        let short_call_high = Position::new(
            high_short_call.option.clone(),
            high_short_call.premium,
            Utc::now(),
            high_short_call.open_fee,
            high_short_call.close_fee,
        );

        // Create strategy
        let mut strategy = CallButterfly {
            name: "Call Butterfly".to_string(),
            kind: StrategyType::CallButterfly,
            description: CALL_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call,
            short_call_low,
            short_call_high,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for CallButterfly {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_call.option.strike_price
                - self.calculate_profit_at(self.long_call.option.strike_price)?
                    / self.long_call.option.quantity)
                .round_to(2),
        );

        self.break_even_points.push(
            (self.short_call_high.option.strike_price
                + self.calculate_profit_at(self.short_call_high.option.strike_price)?
                    / self.short_call_high.option.quantity)
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Default for CallButterfly {
    fn default() -> Self {
        CallButterfly::new(
            "".to_string(),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            ExpirationDate::Days(Positive::ZERO),
            Positive::ZERO,
            Decimal::ZERO,
            Positive::ZERO,
            Positive::ONE,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }
}

impl Positionable for CallButterfly {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.side {
            Side::Short => {
                if position.option.strike_price >= self.long_call.option.strike_price {
                    self.short_call_high = position.clone();
                    Ok(())
                } else {
                    self.short_call_low = position.clone();
                    Ok(())
                }
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.long_call,
            &self.short_call_low,
            &self.short_call_high,
        ])
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call_low.option.strike_price =>
            {
                Ok(vec![&mut self.short_call_low])
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call_high.option.strike_price =>
            {
                Ok(vec![&mut self.short_call_high])
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (_, OptionStyle::Put, _) => Err(PositionError::invalid_position_type(
                *side,
                "Put not found in positions".to_string(),
            )),
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

        if position.option.strike_price != self.long_call.option.strike_price
            && position.option.strike_price != self.short_call_low.option.strike_price
            && position.option.strike_price != self.short_call_high.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Put {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Put is not valid for CallButterfly".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call && position.option.side == Side::Long {
            self.long_call = position.clone();
        }

        if position.option.strike_price == self.short_call_low.option.strike_price {
            self.short_call_low = position.clone();
        }
        if position.option.strike_price == self.short_call_high.option.strike_price {
            self.short_call_high = position.clone();
        }

        Ok(())
    }
}

impl Strategable for CallButterfly {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for CallButterfly {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.calculate_profit_at(self.short_call_high.option.strike_price)?;
        if max_profit > Decimal::ZERO {
            Ok(max_profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY)
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let break_even = self.get_break_even_points()?;
        if break_even.len() != 2 {
            return Err(StrategyError::BreakEvenError(
                BreakEvenErrorKind::NoBreakEvenPoints,
            ));
        }
        let base_low = break_even[1] - break_even[0];
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let base_high =
            self.short_call_high.option.strike_price - self.short_call_low.option.strike_price;
        Ok(((base_low + base_high) * max_profit / 2.0).into())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_loss = match self.max_loss().unwrap() {
            value if value == Positive::ZERO => spos!(1.0),
            value if value == Positive::INFINITY => spos!(1.0),
            value => Some(value),
        };

        match (self.max_profit(), max_loss) {
            (Ok(max_profit), Some(ml)) => Ok((max_profit / ml * 100.0).into()),
            _ => Ok(Decimal::ZERO),
        }
    }
}

impl Validable for CallButterfly {
    fn validate(&self) -> bool {
        if self.name.is_empty() {
            error!("Symbol is required");
            return false;
        }
        if !self.long_call.validate() {
            return false;
        }
        if !self.short_call_low.validate() {
            return false;
        }
        if !self.short_call_high.validate() {
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call_low.option.strike_price {
            error!("Long call strike price must be less than short call strike price");
            return false;
        }
        if self.short_call_low.option.strike_price >= self.short_call_high.option.strike_price {
            error!("Short call low strike price must be less than short call high strike price");
            return false;
        }
        true
    }
}

impl Optimizable for CallButterfly {
    type Strategy = CallButterfly;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_triple_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(long, short_low, short_high)| {
                long.is_valid_optimal_side(underlying_price, &side)
                    && short_low.is_valid_optimal_side(underlying_price, &side)
                    && short_high.is_valid_optimal_side(underlying_price, &side)
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long, short_low, short_high)| {
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_low.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_high.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long, short_low, short_high)| {
                let legs = StrategyLegs::ThreeLegs {
                    first: long,
                    second: short_low,
                    third: short_high,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long, short_low, short_high)| {
                OptionDataGroup::Three(long, short_low, short_high)
            })
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
            let (long, short_low, short_high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::ThreeLegs {
                first: long,
                second: short_low,
                third: short_high,
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

    fn create_strategy(&self, option_chain: &OptionChain, legs: &StrategyLegs) -> CallButterfly {
        let (long_call, short_call_low, short_call_high) = match legs {
            StrategyLegs::ThreeLegs {
                first,
                second,
                third,
            } => (first, second, third),
            _ => panic!("Invalid number of legs for this strategy"),
        };

        if !long_call.validate() || !short_call_low.validate() || !short_call_high.validate() {
            panic!("Invalid options");
        }
        CallButterfly::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_call.strike_price,
            short_call_low.strike_price,
            short_call_high.strike_price,
            self.long_call.option.expiration_date,
            long_call.implied_volatility.unwrap(),
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long_call.call_ask.unwrap(),
            short_call_low.call_bid.unwrap(),
            short_call_high.call_bid.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call_low.open_fee,
            self.short_call_low.close_fee,
            self.short_call_high.open_fee,
            self.short_call_high.close_fee,
        )
    }
}

impl Profit for CallButterfly {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        let long_call_itm_profit = self.long_call.pnl_at_expiration(&price)?;
        let long_call_otm_profit = self.short_call_low.pnl_at_expiration(&price)?;
        let short_call_profit = self.short_call_high.pnl_at_expiration(&price)?;
        Ok(long_call_itm_profit + long_call_otm_profit + short_call_profit)
    }
}

impl Graph for CallButterfly {
    fn title(&self) -> String {
        let strategy_title = format!("Ratio Call Spread Strategy: {:?}", self.kind);
        let long_call_itm_title = self.long_call.title();
        let long_call_otm_title = self.short_call_low.title();
        let short_call_title = self.short_call_high.title();

        format!(
            "{}\n\t{}\n\t{}\n\t{}",
            strategy_title, long_call_itm_title, long_call_otm_title, short_call_title
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.to_f64(),
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (-24.0, -1.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(-55.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(3.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                self.calculate_profit_at(self.long_call.option.strike_price)
                    .unwrap()
                    .to_f64()
                    .unwrap(),
            ),
            label: format!(
                "Left Loss\n\n{:.2}",
                self.calculate_profit_at(self.long_call.option.strike_price)
                    .unwrap()
            ),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let lower_loss = self
            .calculate_profit_at(self.short_call_low.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        let upper_loss = self
            .calculate_profit_at(self.short_call_high.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();

        points.push(ChartPoint {
            coordinates: (self.short_call_low.option.strike_price.to_f64(), lower_loss),
            label: format!("Left High {:.2}", lower_loss),
            label_offset: LabelOffsetType::Relative(3.0, -3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call_high.option.strike_price.to_f64(),
                upper_loss,
            ),
            label: format!("Right High {:.2}", upper_loss),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let underlying_price = self.get_underlying_price();
        points.push(self.get_point_at_price(underlying_price));

        points
    }
}

impl ProbabilityAnalysis for CallButterfly {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.long_call.option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call_low.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            Positive::ZERO,
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
        let break_even_points = self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call_low.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ]);

        let mut loss_range_lower =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        let mut loss_range_upper =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        loss_range_lower.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        loss_range_upper.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![loss_range_lower, loss_range_upper])
    }
}

impl Greeks for CallButterfly {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.long_call.option,
            &self.short_call_low.option,
            &self.short_call_high.option,
        ])
    }
}

impl DeltaNeutrality for CallButterfly {}

impl PnLCalculator for CallButterfly {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self.short_call_low.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?
            + self.short_call_high.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_call_low
                .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_call_high
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

#[cfg(test)]
mod tests_call_butterfly {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn setup() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
            pos!(157.5),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(45.0),
            pos!(30.0),
            pos!(20.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "AAPL");
        assert_eq!(strategy.kind, StrategyType::CallButterfly);
        assert!(
            strategy
                .description
                .contains("A Ratio Call Spread involves")
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 150.1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 172.0;
        assert!(strategy.calculate_profit_at(pos!(price)).unwrap() < Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit() {
        let strategy = setup();
        assert!(strategy.max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_relative_eq!(
            strategy.net_premium_received().unwrap().to_f64(),
            4.9,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let strategy = setup();
        assert_relative_eq!(
            strategy.fees().unwrap().to_f64(),
            0.6,
            epsilon = f64::EPSILON
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150");

        let data = vec![
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
            pos!(165.0),
            pos!(170.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(
                values[i],
                strategy
                    .calculate_profit_at(price)
                    .unwrap()
                    .to_f64()
                    .unwrap()
            );
        }

        let title = strategy.title();
        assert!(title.contains("Ratio Call Spread Strategy"));
        assert!(title.contains("Call"));
    }
}

#[cfg(test)]
mod tests_call_butterfly_validation {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn setup_basic_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(7.0),
            pos!(5.0),
            pos!(3.0),
            pos!(4.0),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_empty_symbol() {
        let mut strategy = setup_basic_strategy();
        strategy.name = "".to_string();
        assert!(!strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_valid_strategy() {
        let strategy = setup_basic_strategy();
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_call_butterfly_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn setup_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(155.0),
            pos!(150.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(7.0),
            pos!(5.0),
            pos!(3.0),
            pos!(4.0),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vertical_lines() {
        let butterfly = setup_test_strategy();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1, "Should have exactly one vertical line");

        let line = &lines[0];
        assert_relative_eq!(line.x_coordinate, 150.0, epsilon = 0.001);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price: 150"));
        assert_eq!(line.font_size, 18);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_points_count_and_labels() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Should have 6 points: 2 break-even, 1 left loss, 2 max profit points, and current price
        assert_eq!(points.len(), 6, "Should have exactly 6 points");

        let labels: Vec<&str> = points.iter().map(|p| p.label.as_str()).collect();

        // Verify all required labels are present
        assert!(labels.iter().any(|&l| l.contains("Low Break Even")));
        assert!(labels.iter().any(|&l| l.contains("High Break Even")));
        assert!(labels.iter().any(|&l| l.contains("Left Loss")));
        assert!(labels.iter().any(|&l| l.contains("Left High")));
        assert!(labels.iter().any(|&l| l.contains("Right High")));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_points_coordinates() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Check for points at strike prices
        let strike_coordinates: Vec<f64> = points.iter().map(|p| p.coordinates.0).collect();

        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 145.0).abs() < 0.001),
            "Missing point near 145.0 strike"
        );
        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 150.0).abs() < 0.001),
            "Missing point near 150.0 strike"
        );
        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 155.0).abs() < 0.001),
            "Missing point near 155.0 strike"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_point_styling() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        for point in points {
            assert_eq!(point.point_size, 5, "Point size should be 5");
            assert_eq!(point.font_size, 18, "Font size should be 18");

            // Verify label offset is set
            match point.label_offset {
                LabelOffsetType::Relative(x, y) => {
                    assert!(x != 0.0, "X offset should not be 0");
                    assert!(y != 0.0, "Y offset should not be 0");
                }
                _ => panic!("Expected Relative label offset"),
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_points() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Find break-even points
        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();

        assert_eq!(
            break_even_points.len(),
            2,
            "Should have exactly 2 break-even points"
        );

        // Verify break-even points have y-coordinate = 0
        for point in break_even_points {
            assert_relative_eq!(point.coordinates.1, 0.0, epsilon = 0.001);
        }
    }
}

#[cfg(test)]
mod tests_call_butterfly_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::call_butterfly::CallButterfly;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5750.0),     // long_strike_itm
            pos!(5850.0),     // long_strike_otm
            pos!(5800.0),     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(95.8),     // short_quantity
            pos!(85.04),    // premium_long_itm
            pos!(31.65),    // premium_long_otm
            pos!(53.04),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5901.88));
        let size = dec!(-0.687410);
        let delta = pos!(0.7040502965074396);
        let k = pos!(5750.0);
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));
        let size = dec!(0.055904);
        let delta1 = pos!(0.0833378661861126);
        let delta2 = pos!(0.2835618144021385);
        let k1 = pos!(5750.0);
        let k2 = pos!(5850.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[0] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut short_call_low = strategy.short_call_low.option.clone();
        let short_call_high = strategy.short_call_high.option.clone();
        let long_call = strategy.long_call.option.clone();
        short_call_low.quantity += delta2;

        let delta = short_call_low.delta().unwrap()
            + short_call_high.delta().unwrap()
            + long_call.delta().unwrap();
        assert_decimal_eq!(delta, Decimal::ZERO, DELTA_THRESHOLD);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5794.4));

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
mod tests_call_butterfly_delta_size {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::call_butterfly::CallButterfly;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5750.0),     // long_strike_itm
            pos!(5850.0),     // long_strike_otm
            pos!(5800.0),     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(97.8),     // short_quantity
            pos!(85.04),    // premium_long_itm
            pos!(31.65),    // premium_long_otm
            pos!(53.04),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.73),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5881.88));
        let size = dec!(-0.5699325);
        let delta = pos!(0.5948524360242063);
        let k = pos!(5750.0);
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));
        let size = dec!(0.05590);
        let delta1 = pos!(0.0833378661861126);
        let delta2 = pos!(0.2835618144021385);
        let k1 = pos!(5750.0);
        let k2 = pos!(5850.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[0] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = delta2;
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
        let strategy = get_strategy(pos!(5794.4));

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
mod tests_call_butterfly_optimizable {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-19".to_string(), None, None);

        // Add options with different strikes
        chain.add_option(
            pos!(95.0),      // strike
            spos!(6.0),      // call_bid
            spos!(6.2),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.2),      // put_ask
            spos!(0.2),      // iv
            Some(dec!(0.4)), // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.2),
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    fn setup_test_butterfly() -> CallButterfly {
        CallButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(6.2), // long call ask
            pos!(3.0), // short call bid low
            pos!(1.0), // short call bid high
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_ratio() {
        let mut butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        // Verify the optimization resulted in valid strikes
        assert!(
            butterfly.long_call.option.strike_price < butterfly.short_call_low.option.strike_price
        );
        assert!(
            butterfly.short_call_low.option.strike_price
                < butterfly.short_call_high.option.strike_price
        );

        // Verify the strategy is valid
        assert!(butterfly.validate());
        assert!(butterfly.max_profit().is_ok());
        assert!(butterfly.max_loss().is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_area() {
        let mut butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        // Verify the optimization resulted in valid strikes
        assert!(
            butterfly.long_call.option.strike_price < butterfly.short_call_low.option.strike_price
        );
        assert!(
            butterfly.short_call_low.option.strike_price
                < butterfly.short_call_high.option.strike_price
        );

        // Verify the strategy is valid
        assert!(butterfly.validate());
        assert!(butterfly.max_profit().is_ok());
        assert!(butterfly.max_loss().is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_strategy() {
        let butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        let legs = StrategyLegs::ThreeLegs {
            first: chain.options.iter().next().unwrap(),
            second: chain.options.iter().nth(1).unwrap(),
            third: chain.options.iter().nth(2).unwrap(),
        };

        let new_strategy = butterfly.create_strategy(&chain, &legs);

        // Verify the new strategy has correct properties
        assert_relative_eq!(
            new_strategy.get_underlying_price().to_f64(),
            100.0,
            epsilon = 0.001
        );
        assert!(new_strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic(expected = "Invalid number of legs for this strategy")]
    fn test_create_strategy_invalid_legs() {
        let butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        let legs = StrategyLegs::TwoLegs {
            first: chain.options.iter().next().unwrap(),
            second: chain.options.iter().nth(1).unwrap(),
        };

        butterfly.create_strategy(&chain, &legs); // Should panic
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_combinations_empty_chain() {
        let butterfly = setup_test_butterfly();
        let empty_chain =
            OptionChain::new("TEST", pos!(100.0), "2024-12-19".to_string(), None, None);

        let combinations: Vec<_> = butterfly
            .filter_combinations(&empty_chain, FindOptimalSide::All)
            .collect();

        assert!(
            combinations.is_empty(),
            "Empty chain should yield no combinations"
        );
    }
}

#[cfg(test)]
mod tests_call_butterfly_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use crate::{assert_pos_relative_eq, pos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    /// Creates a test Call Butterfly with standard parameters based on SP500
    fn create_test_butterfly() -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // long_call_strike
            pos!(5800.0),  // short_call_low_strike
            pos!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long_itm
            pos!(53.04),    // premium_long_otm
            pos!(28.85),    // premium_short
            pos!(0.78),     // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.72),     // open_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let butterfly = create_test_butterfly();
        let result = butterfly.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 2.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let butterfly = create_test_butterfly();
        assert_eq!(
            butterfly.get_risk_free_rate().unwrap().to_f64().unwrap(),
            0.05
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let butterfly = create_test_butterfly();
        let result = butterfly.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 1);
        let range = &ranges[0];

        // Verify range bounds
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= pos!(1.0));

        // Verify bounds are within strike prices
        assert!(range.lower_bound.unwrap() >= butterfly.long_call.option.strike_price);
        assert!(range.upper_bound.unwrap() >= butterfly.short_call_high.option.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let butterfly = create_test_butterfly();
        let result = butterfly.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 2); // Should have two loss ranges

        // Test lower loss range
        let lower_range = &ranges[0];
        assert!(lower_range.lower_bound.is_none());
        assert!(lower_range.upper_bound.is_some());
        assert!(lower_range.probability > Positive::ZERO);

        // Test upper loss range
        let upper_range = &ranges[1];
        assert!(upper_range.lower_bound.is_some());
        assert!(upper_range.upper_bound.is_none());
        assert!(upper_range.probability > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_sum_to_one() {
        let butterfly = create_test_butterfly();

        let profit_ranges = butterfly.get_profit_ranges().unwrap();
        let loss_ranges = butterfly.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(total_profit_prob + total_loss_prob, pos!(1.0), pos!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_points_validity() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        // Break-even points should be within strike prices
        assert!(break_even_points[0] >= butterfly.long_call.option.strike_price);
        assert!(break_even_points[1] >= butterfly.short_call_high.option.strike_price);
        // Break-even points should be between adjacent strikes
        assert!(break_even_points[0] < butterfly.short_call_low.option.strike_price);
        assert!(break_even_points[1] > butterfly.short_call_low.option.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_volatility_adjustment() {
        let butterfly = create_test_butterfly();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let prob = butterfly.probability_of_profit(vol_adj, None);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_price_trend() {
        let butterfly = create_test_butterfly();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let prob = butterfly.probability_of_profit(None, trend);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_analyze_probabilities() {
        let butterfly = create_test_butterfly();
        let analysis = butterfly.analyze_probabilities(None, None).unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.expected_value > Positive::ZERO);
        assert_eq!(analysis.break_even_points.len(), 2);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_near_expiration() {
        let mut butterfly = create_test_butterfly();
        butterfly.long_call.option.expiration_date = ExpirationDate::Days(pos!(0.5));
        butterfly.short_call_low.option.expiration_date = ExpirationDate::Days(pos!(0.5));
        butterfly.short_call_high.option.expiration_date = ExpirationDate::Days(pos!(0.5));

        let prob = butterfly.probability_of_profit(None, None).unwrap();
        // Near expiration probabilities should be more extreme
        assert!(prob < pos!(0.3) || prob > pos!(0.7));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_high_volatility_scenario() {
        let mut butterfly = create_test_butterfly();
        butterfly.long_call.option.implied_volatility = pos!(0.5);
        butterfly.short_call_low.option.implied_volatility = pos!(0.5);
        butterfly.short_call_high.option.implied_volatility = pos!(0.5);

        let analysis = butterfly.analyze_probabilities(None, None).unwrap();
        // Higher volatility should increase potential profit/loss ranges
        assert!(analysis.expected_value > pos!(10.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_extreme_probabilities() {
        let butterfly = create_test_butterfly();
        let result = butterfly.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();

        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_call_butterfly_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_call_butterfly() -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // long_call_strike
            pos!(5800.0),  // short_call_low_strike
            pos!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long_itm
            pos!(53.04),    // premium_long_otm
            pos!(28.85),    // premium_short
            pos!(0.78),     // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.72),     // open_fee_short
        )
    }

    #[test]
    fn test_short_call_butterfly_get_position() {
        let mut call_butterfly = create_test_call_butterfly();

        // Test getting short call position
        let call_position =
            call_butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos!(5800.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5800.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position =
            call_butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos!(5850.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5850.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            call_butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos!(2715.0));
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
    fn test_long_call_butterfly_get_position() {
        let mut call_butterfly = create_test_call_butterfly();

        // Test getting short call position
        let call_position =
            call_butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos!(5750.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5750.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position =
            call_butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos!(2715.0));
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
    fn test_short_call_butterfly_modify_position() {
        let mut call_butterfly = create_test_call_butterfly();

        // Modify short call position
        let mut modified_call = call_butterfly.short_call_low.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = call_butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(call_butterfly.short_call_low.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = call_butterfly.short_call_high.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = call_butterfly.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(call_butterfly.short_call_high.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = call_butterfly.short_call_high.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = call_butterfly.modify_position(&invalid_position);
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

    #[test]
    fn test_long_call_butterfly_modify_position() {
        let mut call_butterfly = create_test_call_butterfly();

        // Modify long call position
        let mut modified_call = call_butterfly.long_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = call_butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(call_butterfly.long_call.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = call_butterfly.long_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = call_butterfly.modify_position(&invalid_position);
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
    fn create_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // long_call_strike
            pos!(5800.0),  // short_call_low_strike
            pos!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long_itm
            pos!(53.04),    // premium_long_otm
            pos!(28.85),    // premium_short
            pos!(0.78),     // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.72),     // open_fee_short
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call_low.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5800.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_call_low.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_long_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
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
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Call,
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
                assert_eq!(reason, "Strike not found in positions");
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
        let initial_quantity = strategy.long_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(5750.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.long_call.option.quantity, initial_quantity);
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
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = CallButterfly::get_strategy(&options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(95.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = CallButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Call Butterfly get_strategy" && reason == "Must have exactly 3 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(95.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(105.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        let result = CallButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Call Butterfly get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(95.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(105.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        let result = CallButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Call Butterfly get_strategy"
                && reason == "Call Butterfly requires one long call and two short calls"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(95.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(105.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        // Modify expiration date for second option
        options[1].option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let result = CallButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Call Butterfly get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_call_butterfly_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn setup_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(7.0),
            pos!(5.0),
            pos!(3.0),
            pos!(4.0),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    fn create_test_call_butterfly() -> Result<CallButterfly, StrategyError> {
        // Create short call at lower strike
        let short_call_low = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Lower short strike
            pos!(0.2),   // Implied volatility
        );

        // Create long call at middle strike
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Middle (long) strike price
            pos!(0.2),   // Implied volatility
        );

        // Create short call at higher strike
        let short_call_high = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Higher short strike
            pos!(0.2),   // Implied volatility
        );

        CallButterfly::get_strategy(&vec![short_call_low, long_call, short_call_high])
    }

    #[test]
    fn test_profit_below_lower_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(pos!(140.0)).unwrap();
        assert!(profit <= Decimal::ZERO);
    }

    #[test]
    fn test_profit_above_upper_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(pos!(160.0)).unwrap();
        assert!(profit <= Decimal::ZERO);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup_test_strategy();
        let ratio = strategy.profit_ratio().unwrap();
        assert!(ratio > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_pnl_below_strikes() {
        let butterfly = create_test_call_butterfly().unwrap();
        let market_price = pos!(90.0); // Below all strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = butterfly.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // All options OTM, should be close to max profit
        // Initial income from short calls
        // Initial costs: Premium for long call + total fees
        assert_pos_relative_eq!(pnl.initial_income, pos!(10.0), pos!(1e-6)); // Premiums from two short calls
        assert_pos_relative_eq!(pnl.initial_costs, pos!(8.0), pos!(1e-6)); // Premium for long call + fees
        assert!(pnl.unrealized.unwrap() > dec!(-2.0)); // Should be near max profit
    }

    #[test]
    fn test_calculate_pnl_between_strikes() {
        let butterfly = create_test_call_butterfly().unwrap();
        let market_price = pos!(100.0); // At middle strike
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.1);

        let result = butterfly.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At-the-money, some loss expected
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // But not max loss
    }

    #[test]
    fn test_calculate_pnl_above_strikes() {
        let butterfly = create_test_call_butterfly().unwrap();
        let market_price = pos!(110.0); // Above all strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = butterfly.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both short calls ITM, long call ITM
        // Expect significant loss, but capped
        assert!(pnl.unrealized.unwrap() < dec!(-5.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let butterfly = create_test_call_butterfly().unwrap();
        let underlying_price = pos!(95.0); // Below lower short strike

        let result = butterfly.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        assert_decimal_eq!(pnl.realized.unwrap(), dec!(2.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(10.0)); // Premiums from short calls
        assert_eq!(pnl.initial_costs, pos!(8.0)); // Premium for long call + fees
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let butterfly = create_test_call_butterfly().unwrap();
        let underlying_price = pos!(110.0); // Above highest short strike

        let result = butterfly.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss at expiration when price is above highest strike
        // Loss = width of spread - net premium received
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-8.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(10.0)); // Premiums from short calls
        assert_eq!(pnl.initial_costs, pos!(8.0)); // Premium for long call + fees
    }

    #[test]
    fn test_calculate_pnl_at_expiration_between_strikes() {
        let butterfly = create_test_call_butterfly().unwrap();
        let underlying_price = pos!(100.0); // At middle strike

        let result = butterfly.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At middle strike, some loss due to fees and net premium
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-3.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let butterfly = create_test_call_butterfly().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = butterfly.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, option values change
        // Net effect should be slightly negative
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still capped by the butterfly spread
        assert!(pnl.unrealized.unwrap() > dec!(-5.0));
    }
}
