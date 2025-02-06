/*
Strangle Strategy

A strangle involves simultaneously buying an out-of-the-money call and an out-of-the-money put option with the same expiration date.
This strategy is similar to a straddle but typically has a lower cost and requires a larger price move to become profitable.

Key characteristics:
- Unlimited profit potential
- Lower cost than a straddle
- Requires a larger price move to become profitable
*/
use super::base::{BreakEvenable, Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
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
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{Options, Positive};
use chrono::Utc;
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{debug, info, trace};

const SHORT_STRANGLE_DESCRIPTION: &str =
    "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable.";

#[derive(Clone, Debug)]
pub struct ShortStrangle {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    short_call: Position,
    short_put: Position,
}

impl ShortStrangle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut call_strike: Positive,
        mut put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        premium_short_put: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        if call_strike == Positive::ZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == Positive::ZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::Strangle,
            description: SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            call_strike,
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
            .expect("Invalid position");

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            put_strike,
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
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid position");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl BreakEvenable for ShortStrangle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_premium = self.net_premium_received()?;

        self.break_even_points.push(
            (self.short_put.option.strike_price - (total_premium / self.short_put.option.quantity))
                .round_to(2),
        );

        self.break_even_points.push(
            (self.short_call.option.strike_price
                + (total_premium / self.short_call.option.quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for ShortStrangle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side.clone(),
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.short_put])
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
            (Side::Long, _, _) => Err(PositionError::invalid_position_type(
                side.clone(),
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
            }
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
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

        if position.option.side == Side::Long {
            return Err(PositionError::invalid_position_type(
                position.option.side.clone(),
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            ));
        }

        if position.option.strike_price != self.short_call.option.strike_price
            && position.option.strike_price != self.short_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side.clone(),
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.short_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.short_put = position.clone();
        }

        Ok(())
    }
}

impl Strategies for ShortStrangle {
    fn get_underlying_price(&self) -> Positive {
        self.short_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.net_premium_received().unwrap().to_f64();
        if max_profit < ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        } else {
            Ok(max_profit.into())
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY)
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        if max_profit == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }
        let strike_diff = self.short_call.option.strike_price - self.short_put.option.strike_price;
        let inner_square = strike_diff * max_profit;
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * max_profit;
        let triangles = (outer_square - inner_square) / 2.0;
        let result =
            ((inner_square + triangles) / self.short_call.option.underlying_price).to_f64();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let result = match self.max_profit() {
            Ok(max_profit) => max_profit.to_f64() / break_even_diff * 100.0,
            Err(_) => ZERO,
        };
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        let start_price = first_option - max_profit;
        let end_price = last_option + max_profit;
        Ok(calculate_price_range(start_price, end_price, step))
    }
}

impl Validable for ShortStrangle {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.short_call.option.strike_price > self.short_put.option.strike_price
    }
}

impl Optimizable for ShortStrangle {
    type Strategy = ShortStrangle;

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
            .filter(move |(short_put, short_call)| {
                short_put.is_valid_optimal_side(underlying_price, &side)
                    && short_call.is_valid_optimal_side(underlying_price, &side)
            })
            .filter(move |(short_put, short_call)| short_put.strike_price < short_call.strike_price)
            // Filter out options with invalid bid/ask prices
            .filter(|(short_put, short_call)| {
                short_put.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(short_put, short_call)| {
                let legs = StrategyLegs::TwoLegs {
                    first: short_put,
                    second: short_call,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(short_put, short_call)| OptionDataGroup::Two(short_put, short_call))
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
            let (short_put, short_call) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: short_put,
                second: short_call,
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
        let (put, call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        if !call.validate() || !put.validate() {
            panic!("Invalid options");
        }
        ShortStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.short_call.option.expiration_date.clone(),
            call.implied_volatility.unwrap() / 100.0,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            call.call_bid.unwrap(),
            put.put_bid.unwrap(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for ShortStrangle {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        trace!(
            "Price: {:?} Strike: {} Call: {:.2} Strike: {} Put: {:.2} Profit: {:.2}",
            price,
            self.short_call.option.strike_price,
            self.short_call.pnl_at_expiration(&price)?,
            self.short_put.option.strike_price,
            self.short_put.pnl_at_expiration(&price)?,
            self.short_call.pnl_at_expiration(&price)?
                + self.short_put.pnl_at_expiration(&price)?
        );
        Ok(
            self.short_call.pnl_at_expiration(&price)?
                + self.short_put.pnl_at_expiration(&price)?,
        )
    }
}

impl Graph for ShortStrangle {
    fn title(&self) -> String {
        let strategy_title = format!("Short {:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.title(), self.short_put.title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.to_f64(),
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!(
                "Current Price: {:.2}",
                self.short_call.option.underlying_price
            ),
            label_offset: (4.0, -1.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);

        let coordinates: (f64, f64) = (-3.0, 30.0);
        let font_size = 24;

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.to_f64(),
                max_profit.to_f64(),
            ),
            label: format!(
                "Max Profit ${:.2} at {:.0}",
                max_profit, self.short_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.to_f64(),
                max_profit.to_f64(),
            ),
            label: format!(
                "Max Profit ${:.2} at {:.0}",
                max_profit, self.short_put.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.underlying_price.to_f64(),
                self.calculate_profit_at(self.short_put.option.underlying_price)
                    .unwrap()
                    .to_f64()
                    .unwrap(),
            ),
            label: format!(
                "${:.2}",
                self.calculate_profit_at(self.short_put.option.underlying_price)
                    .unwrap(),
            ),
            label_offset: LabelOffsetType::Relative(-coordinates.0, coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size,
        });

        points
    }
}

impl ProbabilityAnalysis for ShortStrangle {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        let option = &self.short_call.option;
        Ok(option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.short_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.short_call.option;
        let break_even_points = &self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
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
        let option = &self.short_call.option;
        let break_even_points = self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut lower_loss_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        let mut upper_loss_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![lower_loss_range, upper_loss_range])
    }
}

impl Greeks for ShortStrangle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option, &self.short_put.option])
    }
}

impl DeltaNeutrality for ShortStrangle {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let call_delta = self.short_call.option.delta();
        let put_delta = self.short_put.option.delta();
        let threshold = DELTA_THRESHOLD;
        let c_delta = call_delta.unwrap();
        let p_delta = put_delta.unwrap();
        DeltaInfo {
            net_delta: c_delta + p_delta,
            individual_deltas: vec![c_delta, p_delta],
            is_neutral: (c_delta + p_delta).abs() < threshold,
            underlying_price: self.short_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.short_call.option.underlying_price
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
        let delta = self.short_put.option.delta().unwrap();
        let qty = if delta == Decimal::ZERO {
            Positive::ONE
        } else {
            Positive((net_delta.abs() / delta).abs())
        };
        vec![DeltaAdjustment::SellOptions {
            quantity: qty * self.short_put.option.quantity,
            strike: self.short_put.option.strike_price,
            option_type: OptionStyle::Put,
        }]
    }
}

const LONG_STRANGLE_DESCRIPTION: &str =
    "A long strangle involves buying an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when high volatility \
is expected and a significant move in the underlying asset's price is anticipated, but the \
direction is uncertain.";

#[derive(Clone, Debug)]
pub struct LongStrangle {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    long_call: Position,
    long_put: Position,
}

impl LongStrangle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut call_strike: Positive,
        mut put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_long_put: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
        open_fee_long_put: Positive,
        close_fee_long_put: Positive,
    ) -> Self {
        if call_strike == Positive::ZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == Positive::ZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::Strangle,
            description: LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            call_strike,
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
            .expect("Invalid position");

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            put_strike,
            expiration,
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
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid position");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");

        strategy
    }
}

impl BreakEvenable for LongStrangle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_premium = self.net_cost()?;

        self.break_even_points.push(
            (self.long_put.option.strike_price - (total_premium / self.long_put.option.quantity))
                .round_to(2),
        );

        self.break_even_points.push(
            (self.long_call.option.strike_price + (total_premium / self.long_call.option.quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for LongStrangle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side.clone(),
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_call, &self.long_put])
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
            (Side::Short, _, _) => Err(PositionError::invalid_position_type(
                side.clone(),
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            )),
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
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

        if position.option.side == Side::Short {
            return Err(PositionError::invalid_position_type(
                position.option.side.clone(),
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            ));
        }

        if position.option.strike_price != self.long_call.option.strike_price
            && position.option.strike_price != self.long_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side.clone(),
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.long_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.long_put = position.clone();
        }

        Ok(())
    }
}

impl Strategies for LongStrangle {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY) // Theoretically unlimited
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(self.total_cost()?)
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        if max_loss == Positive::ZERO {
            return Ok(Decimal::MAX);
        }
        let strike_diff = self.long_call.option.strike_price - self.long_put.option.strike_price;
        let inner_square = strike_diff * max_loss;
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * max_loss;
        let triangles = (outer_square - inner_square) / 2.0;
        let loss_area =
            ((inner_square + triangles) / self.long_call.option.underlying_price).to_f64();
        let result = 1.0 / loss_area; // Invert the value to get the profit area: the lower, the better
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        if max_loss == Positive::ZERO {
            return Ok(Decimal::MAX);
        }
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let ratio = max_loss / break_even_diff * 100.0;
        let result = 1.0 / ratio; // Invert the value to get the profit ratio: the lower, the better
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        info!("First: {} Last: {}", first_option, last_option);
        let diff = last_option - first_option;
        debug!(
            "First break even point: {} Last break even point: {}",
            first_option, last_option
        );
        let start_price = first_option - diff;
        debug!("Start price: {}", start_price);
        let end_price = last_option + diff;
        debug!("End price: {}", end_price);
        Ok(calculate_price_range(start_price, end_price, step))
    }
}

impl Validable for LongStrangle {
    fn validate(&self) -> bool {
        self.long_call.validate()
            && self.long_put.validate()
            && self.long_call.option.strike_price > self.long_put.option.strike_price
    }
}

impl Optimizable for LongStrangle {
    type Strategy = LongStrangle;

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
            .filter(move |(long_put, long_call)| {
                long_put.is_valid_optimal_side(underlying_price, &side)
                    && long_call.is_valid_optimal_side(underlying_price, &side)
            })
            .filter(move |(long_put, long_call)| long_put.strike_price < long_call.strike_price)
            // Filter out options with invalid bid/ask prices
            .filter(|(long_put, long_call)| {
                long_put.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && long_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long_put, long_call)| {
                let legs = StrategyLegs::TwoLegs {
                    first: long_put,
                    second: long_call,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long_put, long_call)| OptionDataGroup::Two(long_put, long_call))
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
            let (long_put, long_call) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: long_put,
                second: long_call,
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
        let (put, call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        LongStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.long_call.option.expiration_date.clone(),
            call.implied_volatility.unwrap() / 100.0,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            call.call_ask.unwrap(),
            put.put_ask.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.long_put.open_fee,
            self.long_put.close_fee,
        )
    }
}

impl Profit for LongStrangle {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(self.long_call.pnl_at_expiration(&price)? + self.long_put.pnl_at_expiration(&price)?)
    }
}

impl Graph for LongStrangle {
    fn title(&self) -> String {
        let strategy_title = format!("Long {:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_call.title(), self.long_put.title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let max_value = f64::INFINITY;
        let min_value = f64::NEG_INFINITY;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.to_f64(),
            y_range: (min_value, max_value),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (4.0, -50.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        let coordinates: (f64, f64) = (-3.0, 150.0);
        let font_size = 24;

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -max_loss.to_f64(),
            ),
            label: format!(
                "Max Loss high ${:.2} at {:.0}",
                max_loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.strike_price.to_f64(),
                -max_loss.to_f64(),
            ),
            label: format!(
                "Max Loss low ${:.2} at {:.0}",
                max_loss, self.long_put.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.underlying_price.to_f64(),
                self.calculate_profit_at(self.long_put.option.underlying_price)
                    .unwrap()
                    .to_f64()
                    .unwrap(),
            ),
            label: format!(
                "${:.2}",
                self.calculate_profit_at(self.long_put.option.underlying_price)
                    .unwrap(),
            ),
            label_offset: LabelOffsetType::Relative(-coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size,
        });

        points
    }
}

impl ProbabilityAnalysis for LongStrangle {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        let option = &self.long_call.option;
        Ok(option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.long_call.option;
        let break_even_points = &self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.long_put.option.implied_volatility,
        ]);

        let mut lower_profit_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        let mut upper_profit_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![lower_profit_range, upper_profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.long_call.option;
        let break_even_points = &self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.long_call.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            Positive::ZERO,
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

impl Greeks for LongStrangle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option, &self.long_put.option])
    }
}

impl DeltaNeutrality for LongStrangle {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let call_delta = self.long_call.option.delta();
        let put_delta = self.long_put.option.delta();
        let threshold = DELTA_THRESHOLD;
        let c_delta = call_delta.unwrap();
        let p_delta = put_delta.unwrap();
        DeltaInfo {
            net_delta: c_delta + p_delta,
            individual_deltas: vec![c_delta, p_delta],
            is_neutral: (c_delta + p_delta).abs() < threshold,
            underlying_price: self.long_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta = self.long_put.option.delta().unwrap();
        let qty = if delta == Decimal::ZERO {
            Positive::ONE
        } else {
            Positive((net_delta.abs() / delta).abs())
        };
        vec![DeltaAdjustment::BuyOptions {
            quantity: qty * self.long_call.option.quantity,
            strike: self.long_put.option.strike_price,
            option_type: OptionStyle::Put,
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
mod tests_short_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    fn wrong_setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(155.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
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
        assert_eq!(strategy.name, "Short Strangle");
        assert_eq!(strategy.kind, StrategyType::Strangle);
        assert_eq!(
            strategy.description,
            "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate() {
        let strategy = setup();
        let wrong_strategy = wrong_setup();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 141.9);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 150.0;
        assert_eq!(
            strategy
                .calculate_profit_at(pos!(price))
                .unwrap()
                .to_f64()
                .unwrap(),
            310.0
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit() {
        let strategy = setup();
        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            strategy.net_premium_received().unwrap().to_f64()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost() {
        let strategy = setup();
        assert_eq!(strategy.total_cost().unwrap(), 40.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(
            strategy.net_premium_received().unwrap().to_f64(),
            strategy.short_call.net_premium_received().unwrap()
                + strategy.short_put.net_premium_received().unwrap()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let strategy = setup();
        let expected_fees = 40.0;
        assert_eq!(strategy.fees().unwrap().to_f64(), expected_fees);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_area() {
        let strategy = setup();
        assert_eq!(
            strategy.profit_area().unwrap().to_f64().unwrap(),
            27.07333333333334
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        let data = vec![
            pos!(140.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
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
        assert!(title.contains("Short Strangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_leg() {
        let mut strategy = setup();
        let original_call = strategy.short_call.clone();
        let original_put = strategy.short_put.clone();

        // Test adding a new call leg
        strategy
            .add_position(&original_call.clone())
            .expect("Invalid position");
        assert_eq!(strategy.short_call, original_call);

        // Test adding a new put leg
        strategy
            .add_position(&original_put.clone())
            .expect("Invalid position");
        assert_eq!(strategy.short_put, original_put);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio() {
        let strategy = setup();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio =
            strategy.max_profit().unwrap_or(Positive::ZERO) / break_even_diff * 100.0;
        assert_relative_eq!(
            strategy.profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio.to_f64(),
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_ratio() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_area() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic]
    fn test_best_range_to_show() {
        let strategy = setup();
        let step = pos!(1.0);
        let _ = strategy.best_range_to_show(step).unwrap();
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_valid_short_option() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.first().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        // Test FindOptimalSide::Upper
        assert!(strategy.is_valid_short_option(option_data, &FindOptimalSide::Upper));

        // Test FindOptimalSide::Lower
        assert!(!strategy.is_valid_short_option(option_data, &FindOptimalSide::Lower));

        // Test FindOptimalSide::All
        assert!(strategy.is_valid_short_option(option_data, &FindOptimalSide::All));

        // Test FindOptimalSide::Range
        assert!(strategy
            .is_valid_short_option(option_data, &FindOptimalSide::Range(min_strike, max_strike)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_strategy() {
        let strategy = setup();
        let chain = create_test_option_chain();
        let call_option = chain.options.first().unwrap();
        let put_option = chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };

        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());

        let call_option = chain.options.last().unwrap();
        let put_option = chain.options.first().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };

        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(!new_strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_points() {
        let strategy = setup();
        let points = strategy.get_points();

        assert_eq!(points.len(), 5);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].to_f64()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].to_f64()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(1150.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.01),
            pos!(0.02),
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(10.0),
            0.00001,
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_long_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_long_strangle_new() {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let call_strike = pos!(160.0);
        let put_strike = pos!(140.0);
        let expiration = ExpirationDate::default();
        let implied_volatility = pos!(0.25);
        let risk_free_rate = dec!(0.01);
        let dividend_yield = pos!(0.02);
        let quantity = pos!(10.0);
        let premium_long_call = pos!(5.0);
        let premium_long_put = pos!(5.0);
        let open_fee_long_call = pos!(0.5);
        let close_fee_long_call = pos!(0.5);
        let open_fee_long_put = pos!(0.5);
        let close_fee_long_put = pos!(0.5);

        let strategy = LongStrangle::new(
            underlying_symbol.clone(),
            underlying_price,
            call_strike,
            put_strike,
            expiration.clone(),
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            quantity,
            premium_long_call,
            premium_long_put,
            open_fee_long_call,
            close_fee_long_call,
            open_fee_long_put,
            close_fee_long_put,
        );

        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::Strangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);

        let break_even_points = vec![128.0, 172.0];
        assert_eq!(strategy.break_even_points, break_even_points);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_break_even_points() {
        let long_strangle = setup_long_strangle();
        assert_eq!(long_strangle.get_break_even_points().unwrap()[0], 128.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost() {
        let long_strangle = setup_long_strangle();
        assert_eq!(
            long_strangle.total_cost().unwrap(),
            long_strangle.long_call.net_cost().unwrap()
                + long_strangle.long_put.net_cost().unwrap()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_profit_at() {
        let long_strangle = setup_long_strangle();
        let price = pos!(150.0);
        let expected_profit = long_strangle
            .long_call
            .pnl_at_expiration(&Some(&price))
            .unwrap()
            + long_strangle
                .long_put
                .pnl_at_expiration(&Some(&price))
                .unwrap();
        assert_eq!(
            long_strangle.calculate_profit_at(price).unwrap(),
            expected_profit
        );
    }

    fn setup_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.25),
            dec!(0.01),
            pos!(0.02),
            pos!(10.0),
            pos!(5.0),
            pos!(5.0),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
        )
    }

    fn wrong_setup_long_strangle() -> LongStrangle {
        // Setup with put strike higher than call strike
        LongStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0), // Call strike lower than put
            pos!(160.0), // Put strike higher than call
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.25),
            dec!(0.01),
            pos!(0.02),
            pos!(10.0),
            pos!(5.0),
            pos!(5.0),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::Strangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate() {
        let strategy = setup_long_strangle();
        let wrong_strategy = wrong_setup_long_strangle();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit() {
        let strategy = setup_long_strangle();
        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss() {
        let strategy = setup_long_strangle();
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            strategy.total_cost().unwrap()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let strategy = setup_long_strangle();
        let expected_fees = 20.0; // 0.5 * 4 fees * 10 qty
        assert_eq!(strategy.fees().unwrap().to_f64(), expected_fees);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_area() {
        let strategy = setup_long_strangle();
        let area = strategy.profit_area().unwrap();
        assert!(area > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio() {
        let strategy = setup_long_strangle();
        let expected_ratio = 0.003666666666666666;
        assert_eq!(
            strategy.profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_leg() {
        let mut strategy = setup_long_strangle();
        let original_call = strategy.long_call.clone();
        let original_put = strategy.long_put.clone();

        strategy
            .add_position(&original_call.clone())
            .expect("Invalid position");
        assert_eq!(strategy.long_call, original_call);

        strategy
            .add_position(&original_put.clone())
            .expect("Invalid position");
        assert_eq!(strategy.long_put, original_put);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_methods() {
        let strategy = setup_long_strangle();

        // Test vertical lines
        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        // Test values calculation
        let data = vec![
            pos!(130.0),
            pos!(140.0),
            pos!(150.0),
            pos!(160.0),
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

        // Test title
        let title = strategy.title();
        assert!(title.contains("Long Strangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_ratio() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_area() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_to_show() {
        let strategy = setup_long_strangle();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_valid_long_option() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.last().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::Upper));
        assert!(!strategy.is_valid_long_option(option_data, &FindOptimalSide::Lower));
        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::All));
        assert!(strategy
            .is_valid_long_option(option_data, &FindOptimalSide::Range(min_strike, max_strike)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_are_valid_prices() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.options.first().unwrap();
        let put_option = option_chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        assert!(strategy.are_valid_prices(&legs));

        let mut invalid_call = call_option.clone();
        invalid_call.call_ask = Some(Positive::ZERO);

        let legs = StrategyLegs::TwoLegs {
            first: &invalid_call,
            second: put_option,
        };
        assert!(!strategy.are_valid_prices(&legs));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_strategy() {
        let strategy = setup_long_strangle();
        let chain = create_test_option_chain();
        let call_option = chain.options.first().unwrap();
        let put_option = chain.options.last().unwrap();
        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
        let call_option = chain.options.last().unwrap();
        let put_option = chain.options.first().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(!new_strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_points() {
        let strategy = setup_long_strangle();
        let points = strategy.get_points();

        // Should have 5 points: 2 break-even, 2 max loss, 1 current price
        assert_eq!(points.len(), 5);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].to_f64()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].to_f64()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(150.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.65),
            dec!(0.01),
            pos!(0.02),
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(5.0),
            0.00001,
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_short_strangle_probability {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    /// Helper function that creates a basic short strangle for testing purposes
    /// Returns a ShortStrangle instance with predefined test values
    fn create_test() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit_basic() {
        let strangle = create_test();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok(), "Probability calculation should succeed");
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit_with_volatility_adjustment() {
        let strangle = create_test();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        };

        let result = strangle.probability_of_profit(Some(vol_adj), None);

        assert!(
            result.is_ok(),
            "Probability calculation with volatility adjustment should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit_with_trend() {
        let strangle = create_test();
        let trend = PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        };

        let result = strangle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit_with_downward_trend() {
        let strangle = create_test();
        let trend = PriceTrend {
            drift_rate: -0.1,
            confidence: 0.90,
        };

        let result = strangle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with downward trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_reference_price() {
        let strangle = create_test();
        let result = strangle.get_underlying_price();

        assert_eq!(
            result,
            pos!(100.0),
            "Reference price should match underlying price"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let strangle = create_test();
        let result = strangle.get_expiration();

        assert!(result.is_ok(), "Expiration retrieval should succeed");
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let strangle = create_test();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok(), "Profit ranges calculation should succeed");
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1, "Should have exactly one profit range");

        let range = &ranges[0];
        assert!(range.lower_bound.is_some(), "Lower bound should be defined");
        assert!(range.upper_bound.is_some(), "Upper bound should be defined");
        assert!(
            range.probability > Positive::ZERO,
            "Probability should be positive"
        );
    }
}

#[cfg(test)]
mod tests_short_strangle_probability_bis {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let strangle = create_test();
        let result = strangle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let strangle = create_test();
        assert_eq!(strangle.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let strangle = create_test();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Short strangle has one profit range

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let strangle = create_test();
        let result = strangle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Short strangle has two loss ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].lower_bound.is_none()); // First loss range extends to negative infinity
        assert!(ranges[1].upper_bound.is_none()); // Second loss range extends to positive infinity
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit() {
        let strangle = create_test();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_volatility_adjustment() {
        let strangle = create_test();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = strangle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_trend() {
        let strangle = create_test();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strangle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_analyze_probabilities() {
        let strangle = create_test();
        let result = strangle.analyze_probabilities(None, None);

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
        let strangle = create_test();
        let result = strangle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_long_strangle_probability {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_long_call
            Positive::TWO,                    // premium_long_put
            Positive::ZERO,                   // open_fee_long_call
            Positive::ZERO,                   // close_fee_long_call
            Positive::ZERO,                   // open_fee_long_put
            Positive::ZERO,                   // close_fee_long_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let strangle = create_test_long_strangle();
        let result = strangle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let strangle = create_test_long_strangle();
        assert_eq!(strangle.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let strangle = create_test_long_strangle();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Long strangle has two profit ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].upper_bound.is_some());
        assert!(ranges[1].lower_bound.is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let strangle = create_test_long_strangle();
        let result = strangle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Long strangle has one loss range
        assert!(ranges[0].lower_bound.is_some());
        assert!(ranges[0].upper_bound.is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_of_profit() {
        let strangle = create_test_long_strangle();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_volatility_adjustment() {
        let strangle = create_test_long_strangle();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = strangle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_with_trend() {
        let strangle = create_test_long_strangle();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strangle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_expected_value_calculation() {
        let strangle = create_test_long_strangle();
        let result = strangle.expected_value(None, None);

        assert!(result.is_ok());
        let ev = result.unwrap();
        assert!(
            ev >= Positive::ZERO,
            "Expected value should be non-negative"
        );

        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });
        let result_with_vol = strangle.expected_value(vol_adj, None);
        assert!(result_with_vol.is_ok());
        assert!(result_with_vol.unwrap() >= Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_extreme_probabilities() {
        let strangle = create_test_long_strangle();
        let result = strangle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_short_strangle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::Positive;
    use crate::strategies::strangle::ShortStrangle;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let size = dec!(0.086108);
        let delta = pos!(0.207_000_884_203_610_32);
        let k = pos!(7450.0);
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
        let strike = pos!(7050.0);
        let strategy = get_strategy(pos!(7150.0), strike);
        let size = dec!(-0.1221);
        let delta = pos!(0.2924052685877894);
        let k = pos!(7050.0);
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
                assert_eq!(*option_type, OptionStyle::Put);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
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
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

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
mod tests_long_strangle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::{LongStrangle, Positive};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> LongStrangle {
        let underlying_price = pos!(7138.5);
        LongStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(-0.0861);
        let delta = pos!(0.2070008842036104);
        let k = pos!(7450.0);
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
    fn create_test_increasing_adjustments() {
        let strike = pos!(7050.0);
        let strategy = get_strategy(pos!(7150.0), strike);
        let size = dec!(0.1221);
        let delta = pos!(0.29240526858778937);
        let k = pos!(7050.0);
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
                assert_eq!(*option_type, OptionStyle::Put);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_put.option.clone();
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
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

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
mod tests_short_strangle_delta_size {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::Positive;
    use crate::strategies::strangle::ShortStrangle;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(0.1722);
        let delta = pos!(0.4140017684072208);
        let k = pos!(7450.0);
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
        let strategy = get_strategy(pos!(7150.0), pos!(7050.0));
        let size = dec!(-0.24434);
        let delta = pos!(0.5848105371755788);
        let k = pos!(7050.0);
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
                assert_eq!(*option_type, OptionStyle::Put);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
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
        let strategy = get_strategy(pos!(7450.7), pos!(7045.0));

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
mod tests_long_strangle_delta_size {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::{LongStrangle, Positive};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> LongStrangle {
        let underlying_price = pos!(7138.5);
        LongStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(-0.17221);
        let delta = pos!(0.4140017684072208);
        let k = pos!(7450.0);
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0), pos!(7050.0));
        let size = dec!(0.244340);
        let delta = pos!(0.5848105371755788);
        let k = pos!(7050.0);
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
                assert_eq!(*option_type, OptionStyle::Put);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_put.option.clone();
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
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

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
mod tests_strangle_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_short_strangle() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    fn create_test_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_long_call
            pos!(2.0),      // premium_long_put
            pos!(0.1),      // open_fee_long_call
            pos!(0.1),      // close_fee_long_call
            pos!(0.1),      // open_fee_long_put
            pos!(0.1),      // close_fee_long_put
        )
    }

    #[test]
    fn test_short_strangle_get_position() {
        let mut strangle = create_test_short_strangle();

        // Test getting short call position
        let call_position = strangle.get_position(&OptionStyle::Call, &Side::Short, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position = strangle.get_position(&OptionStyle::Put, &Side::Short, &pos!(90.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(90.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            strangle.get_position(&OptionStyle::Call, &Side::Short, &pos!(100.0));
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
                panic!()
            }
        }
    }

    #[test]
    fn test_short_strangle_modify_position() {
        let mut strangle = create_test_short_strangle();

        // Modify short call position
        let mut modified_call = strangle.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(strangle.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = strangle.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(strangle.short_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = strangle.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = strangle.modify_position(&invalid_position);
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
    fn test_long_strangle_get_position() {
        let mut strangle = create_test_long_strangle();

        // Test getting long call position
        let call_position = strangle.get_position(&OptionStyle::Call, &Side::Long, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting long put position
        let put_position = strangle.get_position(&OptionStyle::Put, &Side::Long, &pos!(90.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(90.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position = strangle.get_position(&OptionStyle::Call, &Side::Long, &pos!(100.0));
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
                panic!()
            }
        }
    }

    #[test]
    fn test_long_strangle_modify_position() {
        let mut strangle = create_test_long_strangle();

        // Modify long call position
        let mut modified_call = strangle.long_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(strangle.long_call.option.quantity, pos!(2.0));

        // Modify long put position
        let mut modified_put = strangle.long_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(strangle.long_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = strangle.long_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = strangle.modify_position(&invalid_position);
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
    fn create_test_strategy() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment,
            &pos!(110.0),
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
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment,
            &pos!(90.0),
            &OptionStyle::Put,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_put.option.quantity,
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
                assert_eq!(
                    reason,
                    "Position side is Long, it is not valid for ShortStrangle"
                );
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
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_adjust_option_position_long {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> LongStrangle {
        LongStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment,
            &pos!(110.0),
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
        let initial_quantity = strategy.long_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment,
            &pos!(90.0),
            &OptionStyle::Put,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.long_put.option.quantity,
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
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<PositionError>() {
            Some(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(
                    reason,
                    "Position side is Short, it is not valid for LongStrangle"
                );
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
        let initial_quantity = strategy.long_call.option.quantity;

        let result = strategy.adjust_option_position(
            Positive::ZERO,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.long_call.option.quantity, initial_quantity);
    }
}
