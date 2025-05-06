/*
Strangle Strategy

A strangle involves simultaneously buying an out-of-the-money call and an out-of-the-money put option with the same expiration date.
This strategy is similar to a straddle but typically has a lower cost and requires a larger price move to become profitable.

Key characteristics:
- Unlimited profit potential
- Lower cost than a straddle
- Requires a larger price move to become profitable
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyBasics,
    StrategyType, Validable,
};
use crate::chains::StrategyLegs;
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::constants::DARK_BLUE;
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::probability::ProbabilityError;
use crate::error::strategies::StrategyError;
use crate::error::{GreeksError, OperationErrorKind};
use crate::greeks::Greeks;
use crate::model::ProfitLossRange;
use crate::model::position::Position;
use crate::model::types::{OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::pnl::PnLCalculator;
use crate::pnl::utils::PnL;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria, calculate_price_range};
use crate::strategies::{DeltaAdjustment, StrategyConstructor};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{ExpirationDate, Options, Positive};
use chrono::Utc;
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{RED, ShapeStyle};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::error::Error;
use std::fmt;
use tracing::{debug, info};

const LONG_STRANGLE_DESCRIPTION: &str = "A long strangle involves buying an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when high volatility \
is expected and a significant move in the underlying asset's price is anticipated, but the \
direction is uncertain.";

/// # LongStrangle
///
/// Represents a Long Strangle options trading strategy.
///
/// A Long Strangle strategy involves simultaneously buying an out-of-the-money call option
/// and an out-of-the-money put option on the same underlying asset with the same expiration date
/// but different strike prices. This strategy benefits from significant price movements in either
/// direction.
///
/// ## Fields
///
/// * `name` - Custom name identifier for this specific strategy instance
/// * `kind` - The type of strategy (always `StrategyType::LongStrangle` for this struct)
/// * `description` - Detailed description of this specific strategy instance
/// * `break_even_points` - Vector containing the price points where the strategy breaks even
///   (typically two points: call strike + call premium and put strike - put premium)
/// * `long_call` - The long call position component of the strategy
/// * `long_put` - The long put position component of the strategy
///
/// ## Risk Profile
///
/// * Maximum Loss: Limited to the total premium paid (call premium + put premium + fees)
/// * Maximum Profit: Theoretically unlimited on the upside; on the downside, limited to
///   the put strike price minus total premium paid
/// * Break-even Points: Upper break-even is call strike plus total premium paid;
///   lower break-even is put strike minus total premium paid
///
/// ## Typical Usage
///
/// Used when an investor expects significant price movement in the underlying asset
/// but is uncertain about the direction of the move. This strategy is often employed:
///
/// * Ahead of major market events with uncertain outcomes (earnings reports, regulatory decisions)
/// * During periods of expected high volatility
/// * When anticipating a break from a trading range
///
/// ## Example Scenario
///
/// If the underlying asset is trading at $100, a long strangle might involve:
/// * Buying a put with a strike price of $90 for a premium of $2
/// * Buying a call with a strike price of $110 for a premium of $2
///
/// Total cost: $4 per share ($400 per contract)
/// Break-even points: $86 and $114
///
/// ## Advantages and Disadvantages
///
/// ### Advantages
/// * Benefits from price movements in either direction
/// * Limited risk (maximum loss is the premium paid)
/// * No margin requirements (beyond the premium paid)
///
/// ### Disadvantages
/// * Requires significant price movement to be profitable
/// * Suffers from time decay (theta) as both options lose value over time
/// * Generally more expensive than directional strategies due to purchasing two options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongStrangle {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Type of the strategy (always StrategyType::LongStrangle)
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy breaks even (typically two points)
    pub break_even_points: Vec<Positive>,
    /// The long call position component of the strategy
    long_call: Position,
    /// The long put position component of the strategy
    long_put: Position,
}

impl LongStrangle {
    /// ## Creation
    ///
    /// Creates a new Long Strangle strategy with two positions:
    /// 1. A long call option with strike typically above the current underlying price (OTM)
    /// 2. A long put option with strike typically below the current underlying price (OTM)
    ///
    /// If strike prices are not explicitly provided (passed as zero), the constructor will automatically set:
    /// - Call strike at 10% above the underlying price
    /// - Put strike at 10% below the underlying price
    ///
    /// ## Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset
    /// * `underlying_price` - Current price of the underlying asset
    /// * `call_strike` - Strike price for the call option (will be set to 110% of underlying price if zero)
    /// * `put_strike` - Strike price for the put option (will be set to 90% of underlying price if zero)
    /// * `expiration` - Expiration date for both options
    /// * `implied_volatility` - Implied volatility for pricing models
    /// * `risk_free_rate` - Risk-free interest rate for pricing models
    /// * `dividend_yield` - Dividend yield for the underlying asset
    /// * `quantity` - Number of contracts to open for each position
    /// * `premium_long_call` - Premium paid for the call option
    /// * `premium_long_put` - Premium paid for the put option
    /// * `open_fee_long_call` - Fee for opening the call position
    /// * `close_fee_long_call` - Fee for closing the call position
    /// * `open_fee_long_put` - Fee for opening the put position
    /// * `close_fee_long_put` - Fee for closing the put position
    ///
    /// ## Returns
    ///
    /// Returns a fully initialized `LongStrangle` strategy with properly configured positions and calculated
    /// break-even points.
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
            kind: StrategyType::LongStrangle,
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

impl Default for LongStrangle {
    fn default() -> Self {
        LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        }
    }
}

impl fmt::Display for LongStrangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match to_string_pretty(self) {
            Ok(pretty_json) => write!(f, "{}", pretty_json),
            Err(e) => write!(f, "Error serializing ShortStrangle to JSON: {}", e),
        }
    }
}

impl StrategyConstructor for LongStrangle {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a long strangle
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by option style to identify call and put
        let mut sorted_options = vec_options.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let put_option = &sorted_options[0]; // Put will be first
        let call_option = &sorted_options[1]; // Call will be second

        // Validate one option is call and other is put
        if call_option.option.option_style != OptionStyle::Call
            || put_option.option.option_style != OptionStyle::Put
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "One option must be a call and one must be a put".to_string(),
                },
            ));
        }

        // Validate both options are long
        if call_option.option.side != Side::Long || put_option.option.side != Side::Long {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Both options must be long positions".to_string(),
                },
            ));
        }

        // Validate call strike is higher than put strike
        if call_option.option.strike_price <= put_option.option.strike_price {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Call strike must be higher than put strike".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if call_option.option.expiration_date != put_option.option.expiration_date {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_call = Position::new(
            call_option.option.clone(),
            call_option.premium,
            Utc::now(),
            call_option.open_fee,
            call_option.close_fee,
        );

        let long_put = Position::new(
            put_option.option.clone(),
            put_option.premium,
            Utc::now(),
            put_option.open_fee,
            put_option.close_fee,
        );

        // Create strategy
        let mut strategy = LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::LongStrangle,
            description: LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call,
            long_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
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
                position.option.side,
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
                *side,
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
            let err_msg = format!("modify_position: Invalid position data: \n{}", position);
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition { reason: err_msg },
            ));
        }

        if position.option.side == Side::Short {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            ));
        }

        if position.option.strike_price != self.long_call.option.strike_price
            && position.option.strike_price != self.long_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
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

impl Strategable for LongStrangle {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for LongStrangle {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::from(self.long_call.option.calculate_price_black_scholes()?.abs());

        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::from(self.long_put.option.calculate_price_black_scholes()?.abs());

        Ok(())
    }

    fn volume(&mut self) -> Result<Positive, StrategyError> {
        let volume = self.long_call.option.quantity + self.long_put.option.quantity;
        Ok(volume)
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
        debug!("First: {} Last: {}", first_option, last_option);
        assert!(first_option < last_option);
        let diff = last_option - first_option.to_dec();
        debug!(
            "First break even point: {} Last break even point: {}",
            first_option, last_option
        );
        let start_price = first_option - diff.to_dec();
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
            .filter(move |(long_put, long_call)| match side {
                FindOptimalSide::DeltaRange(min, max) => {
                    let (_, delta_put) = long_put.current_deltas();
                    let (delta_call, _) = long_call.current_deltas();
                    delta_put.unwrap() > min
                        && delta_put.unwrap() < max
                        && delta_call.unwrap() > min
                        && delta_call.unwrap() < max
                }
                FindOptimalSide::Center => {
                    long_put.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && long_call
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                }
                _ => {
                    long_put.is_valid_optimal_side(underlying_price, &side)
                        && long_call.is_valid_optimal_side(underlying_price, &side)
                }
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

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        let (long_put, long_call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        long_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            && long_put.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (put, call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        let implied_volatility = call.implied_volatility.unwrap();
        assert!(implied_volatility <= Positive::ONE);
        LongStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.long_call.option.expiration_date,
            implied_volatility,
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
        let strategy_title = format!("{:?} Strategy: ", self.kind);
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

    fn get_x_values(&self) -> Vec<Positive> {
        self.best_range_to_show(Positive::from(1.0))
            .unwrap_or_else(|_| vec![self.long_call.option.strike_price])
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
        Ok(option.expiration_date)
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

impl DeltaNeutrality for LongStrangle {}

impl PnLCalculator for LongStrangle {
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
                .long_put
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
                .long_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }

    fn adjustments_pnl(&self, adjustment: &DeltaAdjustment) -> Result<PnL, Box<dyn Error>> {
        let pnl = {
            match adjustment {
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    match (side, option_style) {
                        (Side::Long, OptionStyle::Call) => {
                            let mut position = self.long_call.clone();
                            position.option.side = Side::Long; // Buy the call
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;
                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        (Side::Long, OptionStyle::Put) => {
                            let mut position = self.long_put.clone();
                            position.option.side = Side::Long; // Buy the put
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;
                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        _ => PnL::default(),
                    }
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    match (side, option_style) {
                        (Side::Long, OptionStyle::Call) => {
                            let mut position = self.long_call.clone();
                            position.option.side = Side::Short; // Sell the call 
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;
                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        (Side::Long, OptionStyle::Put) => {
                            let mut position = self.long_put.clone();
                            position.option.side = Side::Short; // Sell the put
                            position.option.quantity = *quantity;
                            position.option.strike_price = *strike;
                            PnL {
                                realized: None,
                                unrealized: None,
                                initial_costs: position.total_cost().unwrap(),
                                initial_income: position.premium_received().unwrap(),
                                date_time: Utc::now(),
                            }
                        }
                        _ => PnL::default(),
                    }
                }
                _ => PnL::default(),
            }
        };
        Ok(pnl)
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
            expiration,
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
        assert_eq!(strategy.kind, StrategyType::LongStrangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);

        let break_even_points = vec![128.0, 172.0];
        assert_eq!(strategy.break_even_points, break_even_points);
    }

    #[test]

    fn test_get_break_even_points() {
        let long_strangle = setup_long_strangle();
        assert_eq!(long_strangle.get_break_even_points().unwrap()[0], 128.0);
    }

    #[test]

    fn test_total_cost() {
        let long_strangle = setup_long_strangle();
        assert_eq!(
            long_strangle.total_cost().unwrap(),
            long_strangle.long_call.net_cost().unwrap()
                + long_strangle.long_put.net_cost().unwrap()
        );
    }

    #[test]

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

    fn test_new() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::LongStrangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);
    }

    #[test]

    fn test_validate() {
        let strategy = setup_long_strangle();
        let wrong_strategy = wrong_setup_long_strangle();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]

    fn test_max_profit() {
        let strategy = setup_long_strangle();
        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]

    fn test_max_loss() {
        let strategy = setup_long_strangle();
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            strategy.total_cost().unwrap()
        );
    }

    #[test]

    fn test_fees() {
        let strategy = setup_long_strangle();
        let expected_fees = 20.0; // 0.5 * 4 fees * 10 qty
        assert_eq!(strategy.fees().unwrap().to_f64(), expected_fees);
    }

    #[test]

    fn test_net_premium_received() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]

    fn test_profit_area() {
        let strategy = setup_long_strangle();
        let area = strategy.profit_area().unwrap();
        assert!(area > Decimal::ZERO);
    }

    #[test]

    fn test_profit_ratio() {
        let strategy = setup_long_strangle();
        let expected_ratio = 0.003666666666666666;
        assert_eq!(
            strategy.profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio
        );
    }

    #[test]

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

    fn test_graph_methods() {
        let strategy = setup_long_strangle();

        // Test vertical lines
        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150");

        // Test values calculation
        let data = strategy.get_x_values();
        let values = strategy.get_y_values();
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
        assert!(title.contains("LongStrangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]

    fn test_best_ratio() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_area() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::Upper);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_range_to_show() {
        let strategy = setup_long_strangle();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]

    fn test_is_valid_long_option() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.last().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::Upper));
        assert!(!strategy.is_valid_long_option(option_data, &FindOptimalSide::Lower));
        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::All));
        assert!(
            strategy
                .is_valid_long_option(option_data, &FindOptimalSide::Range(min_strike, max_strike))
        );
    }

    #[test]

    fn test_are_valid_prices() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.atm_option_data().unwrap();
        let put_option = option_chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: put_option,
            second: call_option,
        };

        assert!(strategy.are_valid_prices(&legs));

        let legs = StrategyLegs::TwoLegs {
            first: option_chain.options.first().unwrap(),
            second: put_option,
        };
        assert!(!strategy.are_valid_prices(&legs));
    }

    #[test]

    fn test_create_strategy() {
        let strategy = setup_long_strangle();
        let chain = create_test_option_chain();
        let call_option = chain
            .options
            .iter()
            .rev()
            .find(|option_data| option_data.valid_call())
            .unwrap();
        let put_option = chain.atm_option_data().unwrap();
        let legs = StrategyLegs::TwoLegs {
            first: put_option,
            second: call_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
        let call_option = chain
            .options
            .iter()
            .rev()
            .find(|option_data| option_data.valid_call())
            .unwrap();
        let put_option = chain.atm_option_data().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: put_option,
            second: call_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
    }

    #[test]

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
            None,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            spos!(5.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_long_strangle_probability {
    use super::*;
    use crate::model::ExpirationDate;
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

    fn test_get_risk_free_rate() {
        let strangle = create_test_long_strangle();
        assert_eq!(strangle.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]

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

    fn test_probability_of_profit() {
        let strangle = create_test_long_strangle();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

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
mod tests_long_strangle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::long_strangle::{LongStrangle, Positive};
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

    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(-0.0861);
        let delta = pos!(0.2070008842036104);
        let k = pos!(7450.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        info!("{:?}", strategy.delta_neutrality().unwrap());
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        info!("{:?}", suggestion);
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

    fn create_test_increasing_adjustments() {
        let strike = pos!(7050.0);
        let strategy = get_strategy(pos!(7150.0), strike);
        let size = dec!(0.1221);
        let delta = pos!(0.29240526858778937);
        let k = pos!(7050.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(option_style, OptionStyle::Put);
                assert_eq!(side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_put.option.clone();
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

    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

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
mod tests_long_strangle_delta_size {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{Action, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::long_strangle::{LongStrangle, Positive};
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

    fn create_test_apply_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let binding = strategy.delta_adjustments().unwrap();
        for suggestion in binding.iter() {
            match suggestion {
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.41400176840722),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7450.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Call);
                    assert_eq!(*side, Side::Long);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.3430003853562),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7250.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Put);
                    assert_eq!(*side, Side::Long);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        -quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SameSize(first, second) => {
                    let call_short_qty = match **first {
                        DeltaAdjustment::BuyOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.1875856830735),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7450.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Call);
                            assert_eq!(side, Side::Long);
                            quantity
                        }
                        _ => panic!("Invalid first adjustment"),
                    };
                    let put_short_qty = match **second {
                        DeltaAdjustment::SellOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.187585683073),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7250.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Put);
                            assert_eq!(side, Side::Long);
                            quantity
                        }
                        _ => {
                            panic!("Invalid suggestion")
                        }
                    };

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        call_short_qty.to_dec(),
                        &pos!(7450.0),
                        &OptionStyle::Call,
                        &Side::Long,
                    );
                    assert!(result.is_ok());
                    let result = temp_strategy.adjust_option_position(
                        -put_short_qty.to_dec(),
                        &pos!(7250.0),
                        &OptionStyle::Put,
                        &Side::Long,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                _ => panic!("Invalid suggestion"),
            }
        }
    }

    #[test]

    fn create_test_apply_delta_adjustments() {
        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Buy));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Sell));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(None);
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(-0.17221);
        let delta = pos!(0.4140017684072208);
        let k = pos!(7450.0);
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

    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0), pos!(7050.0));
        let size = dec!(0.244340);
        let delta = pos!(0.452497797238);

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let first_suggestion = binding.first().unwrap();

        match first_suggestion {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, pos!(7150.0), Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
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

    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

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
mod tests_strangle_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

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
mod tests_adjust_option_position_long {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
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
            adjustment.to_dec(),
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
            adjustment.to_dec(),
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
            Decimal::ONE,
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
            &pos!(110.0),
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
    use crate::{OptionStyle, Side, pos};

    mod long_strangle_tests {
        use super::*;

        #[test]
        fn test_valid_long_strangle() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(result.is_ok());

            let strategy = result.unwrap();
            assert_eq!(strategy.long_call.option.strike_price, pos!(110.0));
            assert_eq!(strategy.long_put.option.strike_price, pos!(90.0));
        }

        #[test]
        fn test_wrong_number_of_options() {
            let options = vec![create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            )];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_option_styles() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_sides() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_invalid_strikes() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }
    }
}

#[cfg(test)]
mod tests_long_strangle_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_strangle() -> Result<LongStrangle, StrategyError> {
        // Create long call position
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create long put position
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        LongStrangle::get_strategy(&vec![long_call, long_put])
    }

    #[test]
    fn test_calculate_pnl_at_money() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.3);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At the money, both options should have time value but no intrinsic value
        // Initial cost is 2 * (premium + fees) = 2 * (5.0 + 1.0) = 12.0
        assert_pos_relative_eq!(pnl.initial_costs, pos!(12.0), pos!(1e-6));
        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(0.748425), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(0.0));
        // Unrealized loss should be less than full premium paid (time value remains)
        assert!(pnl.unrealized.unwrap() > dec!(-12.0));
    }

    #[test]
    fn test_calculate_pnl_above_call_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(110.0); // Above call strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Call should be in the money by 5.0 (110 - 105)
        // Put should still have some time value
        assert!(pnl.unrealized.unwrap() > dec!(-7.0)); // Better than max loss
    }

    #[test]
    fn test_calculate_pnl_below_put_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(90.0); // Below put strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Put should be in the money by 5.0 (95 - 90)
        // Call should still have some time value
        assert!(pnl.unrealized.unwrap() > dec!(-7.0)); // Better than max loss
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(100.0); // At the money

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, both options expire worthless
        // Max loss is the total premium paid plus fees
        assert_eq!(pnl.realized.unwrap(), dec!(-12.0));
        assert_eq!(pnl.initial_costs, pos!(12.0));
        assert_eq!(pnl.initial_income, pos!(0.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_call_profit() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(115.0); // Well above call strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Call profit: 115 - 105 = 10
        // Put expires worthless
        // Total: 10 - initial costs (12)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_put_profit() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(85.0); // Well below put strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Put profit: 95 - 85 = 10
        // Call expires worthless
        // Total: 10 - initial costs (12)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, options should be worth more
        // Loss should be less than with lower volatility
        assert!(pnl.unrealized.unwrap() > dec!(-12.0));
    }
}
