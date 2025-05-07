/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::{OptionData, StrategyLegs};
use crate::constants::{STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER};
use crate::error::OperationErrorKind;
use crate::error::position::PositionError;
use crate::error::strategies::{BreakEvenErrorKind, StrategyError};
use crate::greeks::Greeks;
use crate::model::Trade;
use crate::model::position::Position;
use crate::model::types::{Action, OptionBasicType};
use crate::pnl::utils::PnLCalculator;
use crate::pricing::Profit;
use crate::strategies::probabilities::ProbabilityAnalysis;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria, calculate_price_range};
use crate::strategies::{DeltaNeutrality, StrategyConstructor};
use crate::visualization::utils::Graph;
use crate::{ExpirationDate, OptionStyle, OptionType, Positive, Side};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::{f64, fmt};
use tracing::error;

/// Represents basic information about a trading strategy.
///
/// This struct is used to store the name, type, and description of a strategy.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StrategyBasics {
    /// The name of the strategy.
    pub name: String,
    /// The type of the strategy.  See the [`StrategyType`] enum for possible values.
    pub kind: StrategyType,
    /// A description of the strategy.
    pub description: String,
}

/// This trait defines common functionalities for all trading strategies.
/// It combines several other traits, requiring implementations for methods related to strategy
/// information, construction, optimization, profit calculation, graphing, probability analysis,
/// Greeks calculation, delta neutrality, and P&L calculation.
pub trait Strategable:
    Strategies
    + StrategyConstructor
    + Profit
    + Graph
    + ProbabilityAnalysis
    + Greeks
    + DeltaNeutrality
    + PnLCalculator
{
    /// Returns basic information about the strategy, such as its name, type, and description.
    ///
    /// This method returns an error by default, as it is expected to be implemented by specific
    /// strategy types.
    /// The error indicates that the `info` operation is not supported for the given strategy type.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `StrategyBasics` struct if successful, or a `StrategyError`
    /// if the operation is not supported.
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "info",
            std::any::type_name::<Self>(),
        ))
    }

    /// Returns the type of the strategy.
    ///
    /// This method attempts to retrieve the strategy type from the `info()` method.
    /// If `info()` returns an error (indicating it's not implemented for the specific strategy),
    /// it asserts with a message and returns `StrategyType::Custom`.
    ///
    /// # Returns
    ///
    /// The `StrategyType` of the strategy.
    fn type_name(&self) -> StrategyType {
        match self.info() {
            Ok(info) => info.kind,
            Err(_) => {
                panic!("Invalid strategy type");
            }
        }
    }

    /// Returns the name of the strategy.
    ///
    /// This method attempts to retrieve the strategy name from the `info()` method.
    /// If `info()` returns an error (indicating it's not implemented for the specific strategy),
    /// it asserts with a message and returns "Unknown".
    ///
    /// # Returns
    ///
    /// The name of the strategy as a `String`.
    fn name(&self) -> String {
        match self.info() {
            Ok(info) => info.name,
            Err(_) => {
                panic!("Invalid strategy name");
            }
        }
    }
}

/// Represents different option trading strategies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StrategyType {
    /// Bull Call Spread strategy.
    BullCallSpread,
    /// Bear Call Spread strategy.
    BearCallSpread,
    /// Bull Put Spread strategy.
    BullPutSpread,
    /// Bear Put Spread strategy.
    BearPutSpread,
    /// Long Butterfly Spread strategy.
    LongButterflySpread,
    /// Short Butterfly Spread strategy.
    ShortButterflySpread,
    /// Iron Condor strategy.
    IronCondor,
    /// Iron Butterfly strategy.
    IronButterfly,
    /// Long Straddle strategy.
    LongStraddle,
    /// Short Straddle strategy.
    ShortStraddle,
    /// Long Strangle strategy.
    LongStrangle,
    /// Short Strangle strategy.
    ShortStrangle,
    /// Covered Call strategy.
    CoveredCall,
    /// Protective Put strategy.
    ProtectivePut,
    /// Collar strategy.
    Collar,
    /// Long Call strategy.
    LongCall,
    /// Long Put strategy.
    LongPut,
    /// Short Call strategy.
    ShortCall,
    /// Short Put strategy.
    ShortPut,
    /// Poor Man's Covered Call strategy.
    PoorMansCoveredCall,
    /// Call Butterfly strategy.
    CallButterfly,
    /// Custom strategy.
    Custom,
}

impl FromStr for StrategyType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BullCallSpread" => Ok(StrategyType::BullCallSpread),
            "BearCallSpread" => Ok(StrategyType::BearCallSpread),
            "BullPutSpread" => Ok(StrategyType::BullPutSpread),
            "BearPutSpread" => Ok(StrategyType::BearPutSpread),
            "LongButterflySpread" => Ok(StrategyType::LongButterflySpread),
            "ShortButterflySpread" => Ok(StrategyType::ShortButterflySpread),
            "IronCondor" => Ok(StrategyType::IronCondor),
            "IronButterfly" => Ok(StrategyType::IronButterfly),
            "LongStraddle" => Ok(StrategyType::LongStraddle),
            "ShortStraddle" => Ok(StrategyType::ShortStraddle),
            "LongStrangle" => Ok(StrategyType::LongStrangle),
            "ShortStrangle" => Ok(StrategyType::ShortStrangle),
            "CoveredCall" => Ok(StrategyType::CoveredCall),
            "ProtectivePut" => Ok(StrategyType::ProtectivePut),
            "Collar" => Ok(StrategyType::Collar),
            "LongCall" => Ok(StrategyType::LongCall),
            "LongPut" => Ok(StrategyType::LongPut),
            "ShortCall" => Ok(StrategyType::ShortCall),
            "ShortPut" => Ok(StrategyType::ShortPut),
            "PoorMansCoveredCall" => Ok(StrategyType::PoorMansCoveredCall),
            "CallButterfly" => Ok(StrategyType::CallButterfly),
            "Custom" => Ok(StrategyType::Custom),
            _ => Err(()),
        }
    }
}

impl StrategyType {
    /// Checks if a given string is a valid `StrategyType`.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A string slice representing the strategy type.
    ///
    /// # Returns
    ///
    /// `true` if the string is a valid `StrategyType`, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use optionstratlib::strategies::base::StrategyType;
    /// assert!(StrategyType::is_valid("BullCallSpread"));
    /// assert!(!StrategyType::is_valid("InvalidStrategy"));
    /// ```
    pub fn is_valid(strategy: &str) -> bool {
        StrategyType::from_str(strategy).is_ok()
    }
}
impl fmt::Display for StrategyType {
    /// Formats the `StrategyType` for display.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable formatter.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating whether the formatting was successful.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents a complete options trading strategy with risk-reward parameters.
///
/// A strategy encapsulates all the information needed to describe, analyze, and
/// trade a specific options strategy. It includes identifying information, the positions
/// that make up the strategy, and critical risk metrics such as maximum profit/loss
/// and break-even points.
///
/// This structure serves as the foundation for strategy analysis, visualization,
/// and trading execution within the options trading framework.
///
pub struct Strategy {
    /// The name of the strategy, which identifies it among other strategies.
    pub name: String,

    /// The type of the strategy, categorizing it according to standard options strategies.
    pub kind: StrategyType,

    /// A textual description explaining the strategy's purpose, construction, and typical market scenarios.
    pub description: String,

    /// A collection of positions (or legs) that together form the complete strategy.
    /// Each position represents an option contract or underlying asset position.
    pub legs: Vec<Position>,

    /// The maximum potential profit of the strategy, if limited and known.
    /// Expressed as an absolute value, not percentage.
    pub max_profit: Option<f64>,

    /// The maximum potential loss of the strategy, if limited and known.
    /// Expressed as an absolute value, not percentage.
    pub max_loss: Option<f64>,

    /// The price points of the underlying asset at which the strategy neither makes a profit nor a loss.
    /// These points are crucial for strategy planning and risk management.
    pub break_even_points: Vec<Positive>,
}

/// Creates a new `Strategy` instance.
///
/// This function initializes a new trading strategy with the given name, kind, and description.  The `legs`, `max_profit`, `max_loss`, and `break_even_points` are initialized as empty or `None`.
///
/// # Arguments
///
/// * `name` - The name of the strategy.
/// * `kind` - The type of the strategy  (e.g., BullCallSpread, LongStraddle).
/// * `description` - A description of the strategy.
///
/// # Returns
///
/// A new `Strategy` instance.
///
/// # Example
///
/// ```
/// use optionstratlib::strategies::base::{Strategy, StrategyType};
/// let strategy = Strategy::new(
///     "My Strategy".to_string(),
///     StrategyType::LongCall,
///     "A simple long call strategy".to_string(),
/// );
///
/// assert_eq!(strategy.name, "My Strategy");
/// assert_eq!(strategy.kind, StrategyType::LongCall);
/// assert_eq!(strategy.description, "A simple long call strategy");
/// assert!(strategy.legs.is_empty());
/// assert_eq!(strategy.max_profit, None);
/// assert_eq!(strategy.max_loss, None);
/// assert!(strategy.break_even_points.is_empty());
/// ```
impl Strategy {
    /// Creates a new `Strategy` instance.
    ///
    /// This function initializes a new trading strategy with the given name, kind, and description.
    /// The `legs`, `max_profit`, `max_loss`, and `break_even_points` are initialized as empty or `None`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the strategy.
    /// * `kind` - The type of the strategy (e.g., BullCallSpread, LongStraddle).
    /// * `description` - A description of the strategy.
    ///
    /// # Returns
    ///
    /// A new `Strategy` instance.
    ///
    /// # Example
    ///
    /// ```
    /// use optionstratlib::strategies::base::{Strategy, StrategyType};
    /// let strategy = Strategy::new(
    ///     "My Strategy".to_string(),
    ///     StrategyType::LongCall,
    ///     "A simple long call strategy".to_string(),
    /// );
    ///
    /// assert_eq!(strategy.name, "My Strategy");
    /// assert_eq!(strategy.kind, StrategyType::LongCall);
    /// assert_eq!(strategy.description, "A simple long call strategy");
    /// assert!(strategy.legs.is_empty());
    /// assert_eq!(strategy.max_profit, None);
    /// assert_eq!(strategy.max_loss, None);
    /// assert!(strategy.break_even_points.is_empty());
    /// ```
    pub fn new(name: String, kind: StrategyType, description: String) -> Self {
        Strategy {
            name,
            kind,
            description,
            legs: Vec::new(),
            max_profit: None,
            max_loss: None,
            break_even_points: Vec::new(),
        }
    }
}

pub trait BasicAble {
    /// Returns the title of the graph.
    fn get_title(&self) -> String {
        unimplemented!("get_title is not implemented for this strategy");
    }

    fn get_option_basic_type(&self) -> OptionBasicType {
        unimplemented!("get_option_basic_type is not implemented for this strategy");
    }
    fn get_symbol(&self) -> &str {
        unimplemented!("get_symbol is not implemented for this strategy");
    }
    fn get_strike(&self) -> HashMap<OptionBasicType, &Positive> {
        unimplemented!("get_strike is not implemented for this strategy");
    }
    fn get_side(&self) -> HashMap<OptionBasicType, &Side> {
        unimplemented!("get_side is not implemented for this strategy");
    }
    fn get_type(&self) -> &OptionType {
        unimplemented!("get_type is not implemented for this strategy");
    }
    fn get_style(&self) -> HashMap<OptionBasicType, &OptionStyle> {
        unimplemented!("get_style is not implemented for this strategy");
    }
    fn get_expiration(&self) -> HashMap<OptionBasicType, &ExpirationDate> {
        unimplemented!("get_expiration is not implemented for this strategy");
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType, &Positive> {
        unimplemented!("get_implied_volatility is not implemented for this strategy");
    }
    fn get_quantity(&self) -> HashMap<OptionBasicType, &Positive> {
        unimplemented!("get_quantity is not implemented for this strategy");
    }
    fn get_underlying_price(&self) -> &Positive {
        unimplemented!("get_underlying_price is not implemented for this strategy");
    }
    fn get_risk_free_rate(&self) -> HashMap<OptionBasicType, &Decimal> {
        unimplemented!("get_risk_free_rate is not implemented for this strategy");
    }
    fn get_dividend_yield(&self) -> HashMap<OptionBasicType, &Positive> {
        unimplemented!("get_dividend_yield is not implemented for this strategy");
    }
}

/// Defines a set of strategies for options trading.  Provides methods for calculating key metrics
/// such as profit/loss, cost, break-even points, and price ranges.  Implementations of this trait
/// must also implement the `Validable`, `Positionable`, and `BreakEvenable` traits.
pub trait Strategies: Validable + Positionable + BreakEvenable + BasicAble {
    /// Retrieves the current volume of the strategy as sum of quantities in their positions
    ///
    /// This function returns the volume as a `Positive` value, ensuring that the result
    /// is always greater than zero. If the method fails to retrieve the volume, an error
    /// of type `StrategyError` is returned.
    ///
    /// # Returns
    ///
    /// - `Ok(Positive)` - The current volume as a positive numeric value.
    /// - `Err(StrategyError)` - An error indicating why the volume could not be retrieved.
    ///
    /// # Errors
    ///
    /// This function may return a `StrategyError` in cases such as:
    /// - Internal issues within the strategy's calculation or storage.
    /// - Other implementation-specific failures.
    ///
    fn get_volume(&mut self) -> Result<Positive, StrategyError> {
        let quantities = self.get_quantity();
        let mut volume = Positive::ZERO;
        for (_, quantity) in quantities {
            volume += *quantity;
        }
        Ok(volume)
    }

    /// Calculates the maximum possible profit for the strategy.
    /// The default implementation returns an error indicating that the operation is not supported.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The maximum possible profit.
    /// * `Err(StrategyError)` - If the operation is not supported for this strategy.
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "max_profit",
            std::any::type_name::<Self>(),
        ))
    }

    /// Calculates the maximum possible profit for the strategy, potentially using an iterative approach.
    /// Defaults to calling `max_profit`.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The maximum possible profit.
    /// * `Err(StrategyError)` - If the operation is not supported for this strategy.
    fn get_max_profit_mut(&mut self) -> Result<Positive, StrategyError> {
        self.get_max_profit()
    }

    /// Calculates the maximum possible loss for the strategy.
    /// The default implementation returns an error indicating that the operation is not supported.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The maximum possible loss.
    /// * `Err(StrategyError)` - If the operation is not supported for this strategy.
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "max_loss",
            std::any::type_name::<Self>(),
        ))
    }

    /// Calculates the maximum possible loss for the strategy, potentially using an iterative approach.
    /// Defaults to calling `max_loss`.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The maximum possible loss.
    /// * `Err(StrategyError)` - If the operation is not supported for this strategy.
    fn get_max_loss_mut(&mut self) -> Result<Positive, StrategyError> {
        self.get_max_loss()
    }

    /// Calculates the total cost of the strategy, which is the sum of the absolute cost of all positions.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The total cost of the strategy.
    /// * `Err(PositionError)` - If there is an error retrieving the positions.
    fn get_total_cost(&self) -> Result<Positive, PositionError> {
        let positions = self.get_positions()?;
        let costs = positions
            .iter()
            .map(|p| p.total_cost().unwrap())
            .sum::<Positive>();
        Ok(costs)
    }

    /// Calculates the net cost of the strategy, which is the sum of the costs of all positions,
    /// considering premiums paid and received.
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The net cost of the strategy.
    /// * `Err(PositionError)` - If there is an error retrieving the positions.
    fn get_net_cost(&self) -> Result<Decimal, PositionError> {
        let positions = self.get_positions()?;
        let costs = positions
            .iter()
            .map(|p| p.net_cost().unwrap())
            .sum::<Decimal>();
        Ok(costs)
    }

    /// Calculates the net premium received for the strategy. This is the total premium received from short positions
    /// minus the total premium paid for long positions. If the result is negative, it returns zero.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The net premium received.
    /// * `Err(StrategyError)` - If there is an error retrieving the positions.
    fn get_net_premium_received(&self) -> Result<Positive, StrategyError> {
        let positions = self.get_positions()?;
        let costs = positions
            .iter()
            .filter(|p| p.option.side == Side::Long)
            .map(|p| p.net_cost().unwrap())
            .sum::<Decimal>();
        let premiums = positions
            .iter()
            .filter(|p| p.option.side == Side::Short)
            .map(|p| p.net_premium_received().unwrap())
            .sum::<Positive>();
        match premiums > costs {
            true => Ok(premiums - costs),
            false => Ok(Positive::ZERO),
        }
    }

    /// Calculates the total fees for the strategy by summing the fees of all positions.
    ///
    /// # Returns
    /// * `Ok(Positive)` - The total fees.
    /// * `Err(StrategyError)` - If there is an error retrieving positions or calculating fees.
    fn get_fees(&self) -> Result<Positive, StrategyError> {
        let mut fee = Positive::ZERO;
        let positions = match self.get_positions() {
            Ok(positions) => positions,
            Err(err) => {
                return Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "get_positions".to_string(),
                        reason: err.to_string(),
                    },
                ));
            }
        };
        for position in positions {
            fee += position.fees()?;
        }
        Ok(fee)
    }

    /// Calculates the profit area for the strategy. The default implementation returns an error
    /// indicating that the operation is not supported.
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The profit area.
    /// * `Err(StrategyError)` - If the operation is not supported.
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "profit_area",
            std::any::type_name::<Self>(),
        ))
    }

    /// Calculates the profit ratio for the strategy. The default implementation returns an error
    /// indicating that the operation is not supported.
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The profit ratio.
    /// * `Err(StrategyError)` - If the operation is not supported.
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "profit_ratio",
            std::any::type_name::<Self>(),
        ))
    }

    /// Determines the price range to display for the strategy's profit/loss graph.  This range is
    /// calculated based on the break-even points, the underlying price, and the maximum and minimum
    /// strike prices.  The range is expanded by applying `STRIKE_PRICE_LOWER_BOUND_MULTIPLIER` and
    /// `STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` to the minimum and maximum prices respectively.
    ///
    /// # Returns
    /// * `Ok((Positive, Positive))` - A tuple containing the start and end prices of the range.
    /// * `Err(StrategyError)` - If there is an error retrieving necessary data for the calculation.
    fn get_range_to_show(&self) -> Result<(Positive, Positive), StrategyError> {
        let mut all_points = self.get_break_even_points()?.clone();
        let (first_strike, last_strike) = self.get_max_min_strikes()?;
        let underlying_price = self.get_underlying_price();

        // Calculate the largest difference from the underlying price to furthest strike
        let max_diff = (last_strike.value() - underlying_price.value())
            .abs()
            .max((first_strike.value() - underlying_price.value()).abs());

        // Expand range by max_diff
        all_points.push(
            (*underlying_price - max_diff)
                .max(Positive::ZERO)
                .min(first_strike),
        );
        all_points.push((*underlying_price + max_diff).max(last_strike));

        // Sort to find min and max
        all_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let start_price = *all_points.first().unwrap() * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = *all_points.last().unwrap() * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Ok((start_price, end_price))
    }

    /// Generates a vector of prices within the display range, using a specified step.
    ///
    /// # Returns
    /// * `Ok(Vec<Positive>)` - A vector of prices.
    /// * `Err(StrategyError)` - If there is an error calculating the display range.
    fn get_best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let (start_price, end_price) = self.get_range_to_show()?;
        Ok(calculate_price_range(start_price, end_price, step))
    }

    /// Returns a sorted vector of unique strike prices for all positions in the strategy.
    ///
    /// # Returns
    /// * `Ok(Vec<Positive>)` - A vector of strike prices.
    /// * `Err(StrategyError)` - If there are no positions or an error occurs retrieving them.
    fn get_strikes(&self) -> Result<Vec<Positive>, StrategyError> {
        let positions = match self.get_positions() {
            Ok(positions) => positions,
            Err(_) => {
                return Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "get_positions".to_string(),
                        reason: "No positions found".to_string(),
                    },
                ));
            }
        };
        let strikes: Vec<Positive> = positions
            .iter()
            .map(|leg| leg.option.strike_price)
            .collect::<Vec<_>>()
            .into_iter()
            .sorted()
            .collect();
        Ok(strikes)
    }

    /// Returns the minimum and maximum strike prices from the positions in the strategy.
    /// Considers underlying price when applicable, ensuring the returned range includes it.
    ///
    /// # Returns
    /// * `Ok((Positive, Positive))` - A tuple containing the minimum and maximum strike prices.
    /// * `Err(StrategyError)` - If no strikes are found or if an error occurs retrieving positions.
    fn get_max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> {
        let strikes = self.get_strikes()?;
        if strikes.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "max_min_strikes".to_string(),
                    reason: "No strikes found".to_string(),
                },
            ));
        }
        let mut min = strikes
            .iter()
            .cloned()
            .fold(Positive::INFINITY, Positive::min);
        let mut max = strikes.iter().cloned().fold(Positive::ZERO, Positive::max);
        // If underlying_price is not Positive::ZERO, adjust min and max values
        let underlying_price = self.get_underlying_price();
        if underlying_price != &Positive::ZERO {
            // If min is greater than underlying_price, use underlying_price as min
            if min > *underlying_price {
                min = *underlying_price;
            }
            // If underlying_price is greater than max, use underlying_price as max
            if *underlying_price > max {
                max = *underlying_price;
            }
        }
        Ok((min, max))
    }

    /// Calculates the range of prices where the strategy is profitable, based on the break-even points.
    ///
    /// # Returns:
    /// * `Ok(Positive)` - The difference between the highest and lowest break-even points.  Returns
    ///   `Positive::INFINITY` if there is only one break-even point.
    /// * `Err(StrategyError)` - if there are no break-even points.
    fn get_range_of_profit(&self) -> Result<Positive, StrategyError> {
        let mut break_even_points = self.get_break_even_points()?.clone();
        match break_even_points.len() {
            0 => Err(StrategyError::BreakEvenError(
                BreakEvenErrorKind::NoBreakEvenPoints,
            )),
            1 => Ok(Positive::INFINITY),
            2 => Ok(break_even_points[1] - break_even_points[0]),
            _ => {
                // sort break even points and then get last minus first
                break_even_points.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Ok(*break_even_points.last().unwrap() - *break_even_points.first().unwrap())
            }
        }
    }

    /// Returns a vector of expiration dates for the strategy.
    ///
    /// # Returns
    /// * `Result<Vec<ExpirationDate>, StrategyError>` - A vector of expiration dates, or an error
    ///   if not implemented for the specific strategy.
    fn get_expiration_dates(&self) -> Result<Vec<ExpirationDate>, StrategyError> {
        unimplemented!("Expiration dates is not implemented for this strategy")
    }

    /// Sets the underlying price for a strategy.
    ///
    /// # Arguments
    ///
    /// * `price` - A reference to a `Positive` value representing the new price to set
    ///
    /// # Returns
    ///
    /// A `Result` that will always panic with an informative message
    ///
    /// # Errors
    ///
    /// This function always panics as it's not applicable for the current strategy type.
    /// It's implemented this way to fulfill a trait requirement but intentionally prevents
    /// usage for strategies where underlying price setting doesn't make sense.
    ///
    /// # Panics
    ///
    /// Always panics with the message "Set Underlying price is not applicable for this strategy"
    fn set_underlying_price(&mut self, _price: &Positive) -> Result<(), StrategyError> {
        panic!("Set Underlying price is not applicable for this strategy");
    }

    /// Sets the expiration date for the strategy.
    ///
    /// # Arguments
    /// * `expiration_date` - The new expiration date.
    ///
    /// # Returns
    /// * `Result<(), StrategyError>` -  An error if not implemented for the specific strategy.
    fn set_expiration_date(
        &mut self,
        _expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        unimplemented!("Set expiration date is not implemented for this strategy")
    }

    /// Updates the underlying price for the strategy.
    ///
    /// # Parameters
    /// - `_price`: A reference to a `Positive` value representing the new underlying price to update.
    ///
    /// # Returns
    /// - `Ok(())` if the operation succeeds.
    /// - `Err(StrategyError)` if there is an error during the operation.
    ///
    /// # Notes
    /// This function is currently unimplemented and will panic if called.
    /// It serves as a placeholder for strategies where updating the underlying
    /// price has not yet been implemented.
    ///
    /// # Panics
    /// Always panics with the message "Update underlying price is not implemented for this strategy".
    ///
    fn update_underlying_price(&mut self, _price: &Positive) -> Result<(), StrategyError> {
        unimplemented!("Update underlying price is not implemented for this strategy")
    }

    /// Updates the volatility for the strategy.
    ///
    /// # Parameters
    /// - `_volatility`: A reference to a `Positive` value representing the new volatility to set.
    ///
    /// # Returns
    /// - `Ok(())`: If the update operation succeeds (currently unimplemented).
    /// - `Err(StrategyError)`: If there is an error during the update process (place-holder as functionality is not implemented).
    ///
    /// # Notes
    /// This method is currently unimplemented, and calling it will result in the `unimplemented!` macro being triggered, which causes a panic.
    /// This function is a stub and should be implemented to handle setting the volatility specific to the strategy.
    ///
    fn update_volatility(&mut self, _volatility: &Positive) -> Result<(), StrategyError> {
        unimplemented!("Update volatility is not implemented for this strategy")
    }

    /// Updates the expiration date for the current strategy.
    ///
    /// This function is designed to be overridden or modified in specific strategy implementations.
    /// As it stands, calling this function will result in a panic, as it is not implemented for the
    /// current strategy by default.
    ///
    /// # Parameters
    /// - `_expiration_date`: The new `ExpirationDate` to update the strategy with. This argument is
    ///   currently unused in this default implementation.
    ///
    /// # Returns
    /// This function returns a `Result`:
    /// - `Ok(())` if the update is successful. However, as this method is unimplemented,
    ///   the success branch is not reachable for the default implementation.
    /// - `Err(StrategyError)` if an error occurs. In this implementation, it will not return an error
    ///   but rather panic.
    ///
    /// # Errors
    /// None are returned because the function panics with an unimplemented message in this base
    /// implementation.
    ///
    /// # Panics
    /// Always panics with the message `"Update expiration date is not implemented for this strategy"`.
    ///
    fn update_expiration_date(
        &mut self,
        _expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        unimplemented!("Update expiration date is not implemented for this strategy")
    }

    /// Attempts to execute the roll-in functionality for the strategy.
    ///
    /// # Parameters
    /// - `&mut self`: A mutable reference to the current instance of the strategy.
    /// - `_position: &Position`: A reference to the `Position` object, representing the current position
    ///   in the market. This parameter is currently unused in the implementation.
    ///
    /// # Returns
    /// - `Result<HashMap<Action, Trade>, StrategyError>`:
    ///   - `Ok(HashMap<Action, Trade>)`: On success, a map of actions to trades, representing the changes
    ///     made during the roll-in process.
    ///   - `Err(StrategyError)`: If an error occurs during the roll-in operation.
    ///
    /// # Errors
    /// - Returns a `StrategyError` if the roll-in operation fails (not currently implemented).
    ///
    /// # Panics
    /// - This function will panic if called, as it is currently unimplemented.
    ///
    /// # Note
    /// - This method is not implemented and will panic upon invocation. Future implementations should
    ///   define the specific logic for handling the roll-in operation for the associated strategy.
    fn roll_in(&mut self, _position: &Position) -> Result<HashMap<Action, Trade>, StrategyError> {
        unimplemented!("roll_in is not implemented for this strategy")
    }

    /// Executes the roll-out strategy for the provided position.
    ///
    /// This function is intended to evaluate and execute trading actions based
    /// on the given `Position`. It returns a mapping of `Action` to `Trade` that
    /// represents the proposed trades resulting from the strategy. However, this
    /// method currently is not implemented and will panic if called.
    ///
    /// # Arguments
    ///
    /// * `_position` - A reference to a `Position` object which represents the
    ///   current state of a trading position.
    ///
    /// # Returns
    ///
    /// * `Result<HashMap<Action, Trade>, StrategyError>` - A `Result` object
    ///   containing:
    ///   - `Ok(HashMap<Action, Trade>)` with the mapping of actions to trades if
    ///     successfully implemented in the future.
    ///   - `Err(StrategyError)` if an error occurs during execution (currently
    ///     always unimplemented).
    ///
    /// # Errors
    ///
    /// * Returns an error of type `StrategyError` if the strategy encounters
    ///   execution issues (in this case, always unimplemented).
    ///
    /// # Panics
    ///
    /// This function will panic with a message "roll_out is not implemented for this
    /// strategy" since it is currently not implemented.
    ///
    /// # Note
    ///
    /// Until implemented, calling this method will result in a runtime panic.
    fn roll_out(&mut self, _position: &Position) -> Result<HashMap<Action, Trade>, StrategyError> {
        unimplemented!("roll_out is not implemented for this strategy")
    }
}

/// Trait for strategies that can calculate and update break-even points.
///
/// This trait provides methods for retrieving and updating break-even points, which are
/// crucial for determining profitability in various trading scenarios.
pub trait BreakEvenable {
    /// Retrieves the break-even points for the strategy.
    ///
    /// Returns a `Result` containing a reference to a vector of `Positive` values representing
    /// the break-even points, or a `StrategyError` if the operation is not supported for the specific strategy.
    ///
    /// The default implementation returns a `StrategyError::OperationError` with `OperationErrorKind::NotSupported`.
    /// Strategies implementing this trait should override this method if they support break-even point calculations.
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "get_break_even_points",
            std::any::type_name::<Self>(),
        ))
    }

    /// Updates the break-even points for the strategy.
    ///
    /// This method is responsible for recalculating and updating the break-even points based on
    /// the current state of the strategy.
    ///
    /// The default implementation returns a `NotImplemented` error. Strategies implementing this trait
    /// should override this method to provide specific update logic.
    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        unimplemented!("Update break even points is not implemented for this strategy")
    }
}

/// This trait defines a way to validate a strategy.
///
/// The default implementation panics with a message indicating that validation
/// is not applicable for the specific strategy.  Implementors of this trait
/// should override the `validate` method to provide specific validation logic.
pub trait Validable {
    /// Validates the strategy.
    ///
    /// The default implementation panics, indicating that validation is not
    /// applicable.  Implementors should override this method to provide
    /// appropriate validation logic.
    ///
    /// Returns `true` if the strategy is valid, and `false` otherwise.
    fn validate(&self) -> bool {
        panic!("Validate is not applicable for this strategy");
    }
}

/// This trait defines methods for optimizing and validating option strategies.
/// It combines the `Validable` and `Strategies` traits, requiring implementors
/// to provide functionality for both validation and strategy generation.
pub trait Optimizable: Validable + Strategies {
    /// The type of strategy associated with this optimization.
    type Strategy: Strategies;

    /// Finds the best ratio-based strategy within the given `OptionChain`.
    ///
    /// # Arguments
    /// * `option_chain` - A reference to the `OptionChain` containing option data.
    /// * `side` - A `FindOptimalSide` value specifying the filtering strategy.
    fn get_best_ratio(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Ratio);
    }

    /// Finds the best area-based strategy within the given `OptionChain`.
    ///
    /// # Arguments
    /// * `option_chain` - A reference to the `OptionChain` containing option data.
    /// * `side` - A `FindOptimalSide` value specifying the filtering strategy.
    fn get_best_area(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Area);
    }

    /// Filters and generates combinations of options data from the given `OptionChain`.
    ///
    /// # Parameters
    /// - `&self`: A reference to the current object/context that holds the filtering logic or required data.
    /// - `_option_chain`: A reference to an `OptionChain` object that contains relevant financial information
    ///   such as options data, underlying price, and expiration date.
    /// - `_side`: A `FindOptimalSide` value that specifies the filtering strategy for finding combinations of
    ///   options. It can specify:
    ///     - `Upper`: Consider options higher than a certain threshold.
    ///     - `Lower`: Consider options lower than a certain threshold.
    ///     - `All`: Include all options.
    ///     - `Range(start, end)`: Consider options within a specified range.
    ///
    /// # Returns
    /// - An iterator that yields `OptionDataGroup` items. These items represent combinations of options data filtered
    ///   based on the given criteria. The `OptionDataGroup` can represent combinations of 2, 3, 4, or any number
    ///   of options depending on the grouping logic.
    ///
    /// **Note**:
    /// - The current implementation returns an empty iterator (`std::iter::empty()`) as a placeholder.
    /// - You may modify this method to implement the actual filtering and combination logic based on the
    ///   provided `OptionChain` and `FindOptimalSide` criteria.
    ///
    /// # See Also
    /// - `FindOptimalSide` for the strategy enumeration.
    /// - `OptionDataGroup` for the structure of grouped combinations.
    /// - `OptionChain` for the full structure being processed.
    fn filter_combinations<'a>(
        &'a self,
        _option_chain: &'a OptionChain,
        _side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        error!("Filter combinations is not applicable for this strategy");
        std::iter::empty()
    }

    /// Finds the optimal strategy based on the given criteria.
    /// The default implementation panics.  Specific strategies should override
    /// this method to provide their own optimization logic.
    ///
    /// # Arguments
    /// * `_option_chain` - A reference to the `OptionChain` containing option data.
    /// * `_side` - A `FindOptimalSide` value specifying the filtering strategy.
    /// * `_criteria` - An `OptimizationCriteria` value indicating the optimization goal (e.g., ratio, area).
    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        panic!("Find optimal is not applicable for this strategy");
    }

    /// Checks if a long option is valid based on the given criteria.
    ///
    /// # Arguments
    /// * `option` - A reference to the `OptionData` to validate.
    /// * `side` - A reference to the `FindOptimalSide` specifying the filtering strategy.
    fn is_valid_optimal_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        match side {
            FindOptimalSide::Upper => option.strike_price >= *self.get_underlying_price(),
            FindOptimalSide::Lower => option.strike_price <= *self.get_underlying_price(),
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                option.strike_price >= *start && option.strike_price <= *end
            }
            FindOptimalSide::Deltable(_threshold) => true,
            FindOptimalSide::Center => {
                panic!("Center should be managed by the strategy");
            }
            FindOptimalSide::DeltaRange(min, max) => {
                let (delta_call, delta_put) = option.current_deltas();
                (delta_put.is_some() && delta_put.unwrap() >= *min && delta_put.unwrap() <= *max)
                    || (delta_call.is_some()
                        && delta_call.unwrap() >= *min
                        && delta_call.unwrap() <= *max)
            }
        }
    }

    /// Checks if the prices in the given `StrategyLegs` are valid.
    /// Assumes the strategy consists of one long call and one short call by default.
    ///
    /// # Arguments
    /// * `legs` - A reference to the `StrategyLegs` containing the option data.
    fn are_valid_legs(&self, legs: &StrategyLegs) -> bool {
        // by default, we assume Options are one long call and one short call
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
    }

    /// Creates a new strategy from the given `OptionChain` and `StrategyLegs`.
    /// The default implementation panics. Specific strategies must override this.
    ///
    /// # Arguments
    /// * `_chain` - A reference to the `OptionChain` providing option data.
    /// * `_legs` - A reference to the `StrategyLegs` defining the strategy's components.
    fn create_strategy(&self, _chain: &OptionChain, _legs: &StrategyLegs) -> Self::Strategy {
        unimplemented!("Create strategy is not applicable for this strategy");
    }
}

/// The `Positionable` trait defines methods for managing positions within a trading strategy.
/// These methods allow for adding, retrieving, and modifying positions, providing a common
/// interface for different strategies to interact with position data.
pub trait Positionable {
    /// Adds a position to the strategy.
    ///
    /// # Arguments
    ///
    /// * `_position` - A reference to the `Position` to be added.
    ///
    /// # Returns
    ///
    /// * `Result<(), PositionError>` - Returns `Ok(())` if the position was successfully added,
    ///   or a `PositionError` if the operation is not supported by the strategy.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns an error indicating that adding a position is not
    /// supported. Strategies that support adding positions should override this method.
    fn add_position(&mut self, _position: &Position) -> Result<(), PositionError> {
        Err(PositionError::unsupported_operation(
            std::any::type_name::<Self>(),
            "add_position",
        ))
    }

    /// Retrieves all positions held by the strategy.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<&Position>, PositionError>` - A `Result` containing a vector of references to
    ///   the `Position` objects held by the strategy, or a `PositionError` if the operation is
    ///   not supported.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns an error indicating that getting positions is not
    /// supported. Strategies that manage positions should override this method.
    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Err(PositionError::unsupported_operation(
            std::any::type_name::<Self>(),
            "get_positions",
        ))
    }

    /// Retrieves a specific position based on option style, side, and strike.
    ///
    /// # Arguments
    ///
    /// * `_option_style` - The style of the option (Call or Put).
    /// * `_side` - The side of the position (Long or Short).
    /// * `_strike` - The strike price of the option.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<&mut Position>, PositionError>` - A `Result` containing a vector of mutable
    ///   references to the matching `Position` objects, or a `PositionError` if the operation is not supported.
    ///   This function currently uses `unimplemented!()`.
    fn get_position(
        &mut self,
        _option_style: &OptionStyle,
        _side: &Side,
        _strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        unimplemented!("Modify position is not implemented for this strategy")
    }

    /// Modifies an existing position.
    ///
    /// # Arguments
    ///
    /// * `_position` - A reference to the `Position` to be modified.
    ///
    /// # Returns
    ///
    /// * `Result<(), PositionError>` - A `Result` indicating success or failure of the
    ///   modification, or a `PositionError` if the operation is not supported.
    ///   This function currently uses `unimplemented!()`.
    fn modify_position(&mut self, _position: &Position) -> Result<(), PositionError> {
        unimplemented!("Modify position is not implemented for this strategy")
    }

    ///
    /// Attempts to replace the current position with a new position.
    ///
    /// # Parameters
    /// - `_position`: A reference to a `Position` object that represents the new position to replace the current one.
    ///
    /// # Returns
    /// - `Ok(())`: If the position replacement is successful.
    /// - `Err(PositionError)`: If an error occurs while replacing the position.
    ///
    /// # Notes
    /// This function is currently not implemented for this strategy and will panic with a `not implemented` message when called.
    ///
    /// # Panics
    /// This function will always panic with `unimplemented!()` since it hasn't been implemented yet.
    ///
    fn replace_position(&mut self, _position: &Position) -> Result<(), PositionError> {
        unimplemented!("Replace position is not implemented for this strategy")
    }
}

#[cfg(test)]
mod tests_strategies {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]

    fn test_strategy_new() {
        let strategy = Strategy::new(
            "Test Strategy".to_string(),
            StrategyType::Custom,
            "Test Description".to_string(),
        );

        assert_eq!(strategy.name, "Test Strategy");
        assert_eq!(strategy.kind, StrategyType::Custom);
        assert_eq!(strategy.description, "Test Description");
        assert!(strategy.legs.is_empty());
        assert_eq!(strategy.max_profit, None);
        assert_eq!(strategy.max_loss, None);
        assert!(strategy.break_even_points.is_empty());
    }

    struct MockStrategy {
        legs: Vec<Position>,
        break_even_points: Vec<Positive>,
    }

    impl Validable for MockStrategy {}

    impl Positionable for MockStrategy {
        fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
            self.legs.push(position.clone());
            Ok(())
        }

        fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
            Ok(self.legs.iter().collect())
        }
    }

    impl BreakEvenable for MockStrategy {
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
    }

    impl BasicAble for MockStrategy {}

    impl Strategies for MockStrategy {
        fn get_volume(&mut self) -> Result<Positive, StrategyError> {
            unreachable!()
        }

        fn get_max_profit(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::THOUSAND)
        }

        fn get_max_loss(&self) -> Result<Positive, StrategyError> {
            Ok(pos!(500.0))
        }

        fn get_total_cost(&self) -> Result<Positive, PositionError> {
            Ok(pos!(200.0))
        }

        fn get_net_premium_received(&self) -> Result<Positive, StrategyError> {
            Ok(pos!(300.0))
        }

        fn get_fees(&self) -> Result<Positive, StrategyError> {
            Ok(pos!(50.0))
        }

        fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(5000.0))
        }

        fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(2.0))
        }
    }

    #[test]
    fn test_strategies_trait() {
        let mut mock_strategy = MockStrategy {
            legs: Vec::new(),
            break_even_points: vec![Positive::HUNDRED],
        };

        // Test add_leg and get_legs
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            Positive::ONE,
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );
        mock_strategy
            .add_position(&position.clone())
            .expect("Error adding position");

        // Test other methods
        assert_eq!(
            mock_strategy.get_break_even_points().unwrap(),
            &vec![Positive::HUNDRED]
        );
        assert_eq!(
            mock_strategy.get_max_profit().unwrap_or(Positive::ZERO),
            1000.0
        );
        assert_eq!(
            mock_strategy.get_max_loss().unwrap_or(Positive::ZERO),
            500.0
        );
        assert_eq!(mock_strategy.get_total_cost().unwrap().to_f64(), 200.0);
        assert_eq!(
            mock_strategy.get_net_premium_received().unwrap(),
            dec!(300.0)
        );
        assert_eq!(mock_strategy.get_fees().unwrap(), dec!(50.0));
        assert_eq!(mock_strategy.get_profit_area().unwrap(), dec!(5000.0));
        assert_eq!(mock_strategy.get_profit_ratio().unwrap(), dec!(2.0));
    }

    #[test]
    fn test_strategies_default_methods() {
        struct DefaultStrategy;
        impl Validable for DefaultStrategy {
            fn validate(&self) -> bool {
                true
            }
        }
        impl Positionable for DefaultStrategy {}
        impl BreakEvenable for DefaultStrategy {}
        impl BasicAble for DefaultStrategy {}
        impl Strategies for DefaultStrategy {
            fn get_volume(&mut self) -> Result<Positive, StrategyError> {
                unreachable!()
            }
        }

        let strategy = DefaultStrategy;

        assert_eq!(
            strategy.get_max_profit().unwrap_or(Positive::ZERO),
            Positive::ZERO
        );
        assert_eq!(
            strategy.get_max_loss().unwrap_or(Positive::ZERO),
            Positive::ZERO
        );
        assert!(strategy.get_total_cost().is_err());
        assert!(strategy.get_profit_area().is_err());
        assert!(strategy.get_profit_ratio().is_err());
        assert!(strategy.validate());
    }

    #[test]
    fn test_strategies_add_leg_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl BreakEvenable for PanicStrategy {}
        impl BasicAble for PanicStrategy {}
        impl Strategies for PanicStrategy {
            fn get_volume(&mut self) -> Result<Positive, StrategyError> {
                unreachable!()
            }
        }

        let mut strategy = PanicStrategy;
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            Positive::ONE,
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );

        assert!(strategy.add_position(&position).is_err());
    }
}

#[cfg(test)]
mod tests_strategies_extended {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;

    #[test]
    fn test_strategy_enum() {
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
        assert_eq!(StrategyType::Custom, StrategyType::Custom);
    }

    #[test]
    fn test_strategy_new_with_legs() {
        let mut strategy = Strategy::new(
            "Test Strategy".to_string(),
            StrategyType::BullCallSpread,
            "Test Description".to_string(),
        );
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            Positive::ONE,
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );

        strategy.legs.push(position);

        assert_eq!(strategy.legs.len(), 1);
    }

    #[test]
    fn test_strategies_get_legs_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl BreakEvenable for PanicStrategy {}
        impl BasicAble for PanicStrategy {}
        impl Strategies for PanicStrategy {
            fn get_volume(&mut self) -> Result<Positive, StrategyError> {
                unreachable!()
            }
        }

        let strategy = PanicStrategy;
        assert!(strategy.get_positions().is_err());
    }

    #[test]
    fn test_strategies_break_even_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl BreakEvenable for PanicStrategy {}
        impl BasicAble for PanicStrategy {}
        impl Strategies for PanicStrategy {
            fn get_volume(&mut self) -> Result<Positive, StrategyError> {
                unreachable!()
            }
        }

        let strategy = PanicStrategy;
        assert!(strategy.get_break_even_points().is_err());
    }

    #[test]
    fn test_strategies_net_premium_received_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl BreakEvenable for PanicStrategy {}
        impl BasicAble for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.get_net_premium_received().is_err());
    }

    #[test]
    fn test_strategies_fees_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl BreakEvenable for PanicStrategy {}
        impl BasicAble for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.get_fees().is_err());
    }

    #[test]

    fn test_strategies_max_profit_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl BreakEvenable for TestStrategy {}
        impl BasicAble for TestStrategy {}
        impl Strategies for TestStrategy {
            fn get_max_profit(&self) -> Result<Positive, StrategyError> {
                Ok(pos!(100.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.get_max_profit_mut().unwrap().to_f64(), 100.0);
    }

    #[test]

    fn test_strategies_max_loss_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl BreakEvenable for TestStrategy {}
        impl BasicAble for TestStrategy {}
        impl Strategies for TestStrategy {
            fn get_max_loss(&self) -> Result<Positive, StrategyError> {
                Ok(pos!(50.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.get_max_loss_mut().unwrap().to_f64(), 50.0);
    }

    #[test]

    fn test_strategies_empty_strikes() {
        struct EmptyStrategy;
        impl Validable for EmptyStrategy {}
        impl Positionable for EmptyStrategy {
            fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
                Ok(vec![])
            }
        }
        impl BreakEvenable for EmptyStrategy {}
        impl BasicAble for EmptyStrategy {}
        impl Strategies for EmptyStrategy {}

        let strategy = EmptyStrategy;
        assert_eq!(strategy.get_strikes().unwrap(), Vec::<Positive>::new());
        assert!(strategy.get_max_min_strikes().is_err());
    }
}

#[cfg(test)]
mod tests_strategy_type {
    use super::*;

    #[test]

    fn test_strategy_type_equality() {
        assert_eq!(StrategyType::BullCallSpread, StrategyType::BullCallSpread);
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
    }

    #[test]

    fn test_strategy_type_clone() {
        let strategy = StrategyType::IronCondor;
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }

    #[test]

    fn test_strategy_type_debug() {
        let strategy = StrategyType::ShortStraddle;
        let debug_string = format!("{:?}", strategy);
        assert_eq!(debug_string, "ShortStraddle");
    }

    #[test]

    fn test_all_strategy_types() {
        let strategies = [
            StrategyType::BullCallSpread,
            StrategyType::BearCallSpread,
            StrategyType::BullPutSpread,
            StrategyType::BearPutSpread,
            StrategyType::IronCondor,
            StrategyType::LongStraddle,
            StrategyType::ShortStraddle,
            StrategyType::LongStrangle,
            StrategyType::ShortStrangle,
            StrategyType::CoveredCall,
            StrategyType::ProtectivePut,
            StrategyType::Collar,
            StrategyType::LongCall,
            StrategyType::LongPut,
            StrategyType::ShortCall,
            StrategyType::ShortPut,
            StrategyType::PoorMansCoveredCall,
            StrategyType::CallButterfly,
            StrategyType::Custom,
        ];

        for (i, strategy) in strategies.iter().enumerate() {
            for (j, other_strategy) in strategies.iter().enumerate() {
                if i == j {
                    assert_eq!(strategy, other_strategy);
                } else {
                    assert_ne!(strategy, other_strategy);
                }
            }
        }
    }

    #[test]
    fn test_strategy_type_from_str() {
        assert_eq!(
            StrategyType::from_str("ShortStrangle"),
            Ok(StrategyType::ShortStrangle)
        );
        assert_eq!(
            StrategyType::from_str("LongCall"),
            Ok(StrategyType::LongCall)
        );
        assert_eq!(
            StrategyType::from_str("BullCallSpread"),
            Ok(StrategyType::BullCallSpread)
        );
        assert_eq!(StrategyType::from_str("InvalidStrategy"), Err(()));
    }

    #[test]
    fn test_strategy_type_is_valid() {
        assert!(StrategyType::is_valid("ShortStrangle"));
        assert!(StrategyType::is_valid("LongPut"));
        assert!(StrategyType::is_valid("CoveredCall"));
        assert!(!StrategyType::is_valid("InvalidStrategy"));
        assert!(!StrategyType::is_valid("Random"));
    }

    #[test]
    fn test_strategy_type_serialization() {
        let strategy = StrategyType::IronCondor;
        let serialized = serde_json::to_string(&strategy).unwrap();
        assert_eq!(serialized, "\"IronCondor\"");

        let deserialized: StrategyType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, StrategyType::IronCondor);
    }

    #[test]
    fn test_strategy_type_deserialization() {
        let json_data = "\"ShortStraddle\"";
        let deserialized: StrategyType = serde_json::from_str(json_data).unwrap();
        assert_eq!(deserialized, StrategyType::ShortStraddle);
    }

    #[test]
    fn test_invalid_strategy_type_deserialization() {
        let json_data = "\"InvalidStrategy\"";
        let deserialized: Result<StrategyType, _> = serde_json::from_str(json_data);
        assert!(deserialized.is_err());
    }
}

#[cfg(test)]
mod tests_max_min_strikes {
    use super::*;
    use crate::{Side, pos};

    struct TestStrategy {
        strikes: Vec<Positive>,
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(
            strikes: Vec<Positive>,
            underlying_price: Positive,
            break_even_points: Vec<Positive>,
        ) -> Self {
            Self {
                strikes,
                underlying_price,
                break_even_points,
            }
        }
    }

    impl Validable for TestStrategy {
        fn validate(&self) -> bool {
            true
        }
    }

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
    }

    impl BasicAble for TestStrategy {
        fn get_underlying_price(&self) -> &Positive {
            &self.underlying_price
        }
    }

    impl Strategies for TestStrategy {
        fn get_max_profit(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn get_max_loss(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn get_total_cost(&self) -> Result<Positive, PositionError> {
            Ok(Positive::ZERO)
        }
        fn get_net_premium_received(&self) -> Result<Positive, StrategyError> {
            let positions = self.get_positions()?;
            let costs = positions
                .iter()
                .filter(|p| p.option.side == Side::Long)
                .map(|p| p.net_cost().unwrap())
                .sum::<Decimal>();

            let premiums = positions
                .iter()
                .filter(|p| p.option.side == Side::Short)
                .map(|p| p.net_premium_received().unwrap())
                .sum::<Positive>();

            match premiums > costs {
                true => Ok(premiums - costs),
                false => Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "Net premium received".to_string(),
                        reason: "Net premium received is negative".to_string(),
                    },
                )),
            }
        }
        fn get_fees(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn get_best_range_to_show(&self, _step: Positive) -> Result<Vec<Positive>, StrategyError> {
            Ok(vec![])
        }
        fn get_strikes(&self) -> Result<Vec<Positive>, StrategyError> {
            Ok(self.strikes.clone())
        }
    }

    #[test]

    fn test_empty_strikes() {
        let strategy = TestStrategy::new(vec![], Positive::ZERO, vec![]);
        assert!(strategy.get_max_min_strikes().is_err());
    }

    #[test]

    fn test_single_strike() {
        let strike = pos!(100.0);
        let strategy = TestStrategy::new(vec![strike], Positive::ZERO, vec![]);
        assert_eq!(strategy.get_max_min_strikes().unwrap(), (strike, strike));
    }

    #[test]

    fn test_multiple_strikes_no_underlying() {
        let strikes = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let strategy = TestStrategy::new(strikes.clone(), Positive::ZERO, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (*strikes.first().unwrap(), *strikes.last().unwrap())
        );
    }

    #[test]

    fn test_underlying_price_between_strikes() {
        let strikes = vec![pos!(90.0), pos!(110.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }

    #[test]

    fn test_underlying_price_below_min_strike() {
        let strikes = vec![pos!(100.0), pos!(110.0)];
        let underlying = pos!(90.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }

    #[test]

    fn test_underlying_price_above_max_strike() {
        let strikes = vec![pos!(90.0), pos!(100.0)];
        let underlying = pos!(110.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }

    #[test]

    fn test_strikes_with_duplicates() {
        let strikes = vec![pos!(100.0), pos!(100.0), pos!(110.0)];
        let strategy = TestStrategy::new(strikes, Positive::ZERO, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(100.0), pos!(110.0))
        );
    }

    #[test]

    fn test_underlying_equals_min_strike() {
        let strikes = vec![pos!(100.0), pos!(110.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(100.0), pos!(110.0))
        );
    }

    #[test]

    fn test_underlying_equals_max_strike() {
        let strikes = vec![pos!(90.0), pos!(100.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(90.0), pos!(100.0))
        );
    }

    #[test]

    fn test_unordered_strikes() {
        let strikes = vec![pos!(110.0), pos!(90.0), pos!(100.0)];
        let strategy = TestStrategy::new(strikes, Positive::ZERO, vec![]);
        assert_eq!(
            strategy.get_max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
        strikes: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(
            underlying_price: Positive,
            break_even_points: Vec<Positive>,
            strikes: Vec<Positive>,
        ) -> Self {
            Self {
                underlying_price,
                break_even_points,
                strikes,
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
    }

    impl BasicAble for TestStrategy {
        fn get_underlying_price(&self) -> &Positive {
            &self.underlying_price
        }
    }

    impl Strategies for TestStrategy {
        fn get_strikes(&self) -> Result<Vec<Positive>, StrategyError> {
            Ok(self.strikes.clone())
        }
    }

    #[test]

    fn test_basic_range_with_step() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let range = strategy.get_best_range_to_show(pos!(5.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], pos!(5.0));
    }

    #[test]

    fn test_range_with_small_step() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(95.0), pos!(105.0)],
            vec![pos!(97.0), pos!(103.0)],
        );
        let range = strategy.get_best_range_to_show(pos!(1.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], pos!(1.0));
    }

    #[test]

    fn test_range_boundaries() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let range = strategy.get_best_range_to_show(pos!(5.0)).unwrap();
        assert!(range.first().unwrap() < &pos!(90.0));
        assert!(range.last().unwrap() > &pos!(110.0));
    }

    #[test]

    fn test_range_step_size() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let step = pos!(5.0);
        let range = strategy.get_best_range_to_show(step).unwrap();

        for i in 1..range.len() {
            assert_eq!(range[i] - range[i - 1], step);
        }
    }

    #[test]

    fn test_range_includes_underlying() {
        let underlying_price = pos!(100.0);
        let strategy = TestStrategy::new(
            underlying_price,
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let range = strategy.get_best_range_to_show(pos!(5.0)).unwrap();

        assert!(range.iter().any(|&price| price <= underlying_price));
        assert!(range.iter().any(|&price| price >= underlying_price));
    }

    #[test]

    fn test_range_with_extreme_values() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(50.0), pos!(150.0)],
            vec![pos!(75.0), pos!(125.0)],
        );
        let range = strategy.get_best_range_to_show(pos!(10.0)).unwrap();

        assert!(range.first().unwrap() <= &pos!(50.0));
        assert!(range.last().unwrap() >= &pos!(150.0));
    }
}

#[cfg(test)]
mod tests_range_to_show {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
        strikes: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(
            underlying_price: Positive,
            break_even_points: Vec<Positive>,
            strikes: Vec<Positive>,
        ) -> Self {
            Self {
                underlying_price,
                break_even_points,
                strikes,
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
    }

    impl BasicAble for TestStrategy {
        fn get_underlying_price(&self) -> &Positive {
            &self.underlying_price
        }
    }

    impl Strategies for TestStrategy {
        fn get_strikes(&self) -> Result<Vec<Positive>, StrategyError> {
            Ok(self.strikes.clone())
        }
    }

    #[test]

    fn test_basic_range() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let (start, end) = strategy.get_range_to_show().unwrap();
        assert!(start < pos!(90.0));
        assert!(end > pos!(110.0));
    }

    #[test]

    fn test_range_with_far_strikes() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(80.0), pos!(120.0)],
        );
        let (start, end) = strategy.get_range_to_show().unwrap();
        assert!(start < pos!(80.0));
        assert!(end > pos!(120.0));
    }

    #[test]

    fn test_range_with_underlying_outside_strikes() {
        let strategy = TestStrategy::new(
            pos!(150.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let (_start, end) = strategy.get_range_to_show().unwrap();
        assert!(end > pos!(150.0));
    }
}

#[cfg(test)]
mod tests_range_of_profit {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        break_even_points: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(break_even_points: Vec<Positive>) -> Self {
            Self { break_even_points }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
    }

    impl BasicAble for TestStrategy {}

    impl Strategies for TestStrategy {}

    #[test]

    fn test_no_break_even_points() {
        let strategy = TestStrategy::new(vec![]);
        assert!(strategy.get_range_of_profit().is_err());
    }

    #[test]

    fn test_single_break_even_point() {
        let strategy = TestStrategy::new(vec![pos!(100.0)]);
        assert_eq!(strategy.get_range_of_profit().unwrap(), Positive::INFINITY);
    }

    #[test]

    fn test_two_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(90.0), pos!(110.0)]);
        assert_eq!(strategy.get_range_of_profit().unwrap(), pos!(20.0));
    }

    #[test]

    fn test_multiple_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(80.0), pos!(100.0), pos!(120.0)]);
        assert_eq!(strategy.get_range_of_profit().unwrap(), pos!(40.0));
    }

    #[test]

    fn test_unordered_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(120.0), pos!(80.0), pos!(100.0)]);
        assert_eq!(strategy.get_range_of_profit().unwrap(), pos!(40.0));
    }
}

#[cfg(test)]
mod tests_strategy_methods {
    use super::*;

    #[test]
    fn test_get_underlying_price_panic() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl BreakEvenable for TestStrategy {}
        impl BasicAble for TestStrategy {}
        impl Strategies for TestStrategy {}
        let strategy = TestStrategy;
        let result = std::panic::catch_unwind(|| strategy.get_underlying_price());
        assert!(result.is_err());
    }

    #[test]

    fn test_strategy_type_debug_all_variants() {
        let variants = vec![
            StrategyType::BullCallSpread,
            StrategyType::BearCallSpread,
            StrategyType::BullPutSpread,
            StrategyType::BearPutSpread,
            StrategyType::LongButterflySpread,
            StrategyType::ShortButterflySpread,
            StrategyType::IronCondor,
            StrategyType::IronButterfly,
            StrategyType::LongStraddle,
            StrategyType::ShortStraddle,
            StrategyType::LongStrangle,
            StrategyType::ShortStrangle,
            StrategyType::CoveredCall,
            StrategyType::ProtectivePut,
            StrategyType::Collar,
            StrategyType::LongCall,
            StrategyType::LongPut,
            StrategyType::ShortCall,
            StrategyType::ShortPut,
            StrategyType::PoorMansCoveredCall,
            StrategyType::CallButterfly,
            StrategyType::Custom,
        ];

        for variant in variants {
            let debug_string = format!("{:?}", variant);
            assert!(!debug_string.is_empty());
        }
    }
}

#[cfg(test)]
mod tests_optimizable {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    struct TestOptimizableStrategy;

    impl Validable for TestOptimizableStrategy {
        fn validate(&self) -> bool {
            true
        }
    }

    impl Positionable for TestOptimizableStrategy {}

    impl BreakEvenable for TestOptimizableStrategy {}

    impl BasicAble for TestOptimizableStrategy {}

    impl Strategies for TestOptimizableStrategy {}

    impl Optimizable for TestOptimizableStrategy {
        type Strategy = Self;
    }

    #[test]

    fn test_is_valid_long_option() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(100),     // open_interest
        );
        assert!(strategy.is_valid_optimal_option(&option_data, &FindOptimalSide::All));
        assert!(strategy.is_valid_optimal_option(
            &option_data,
            &FindOptimalSide::Range(pos!(90.0), pos!(110.0))
        ));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_long_option_upper_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(100),     // open_interest
        );
        assert!(strategy.is_valid_optimal_option(&option_data, &FindOptimalSide::Upper));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_long_option_lower_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(100),     // open_interest
        );
        assert!(strategy.is_valid_optimal_option(&option_data, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_is_valid_short_option() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(100),     // open_interest
        );
        assert!(strategy.is_valid_optimal_option(&option_data, &FindOptimalSide::All));
        assert!(strategy.is_valid_optimal_option(
            &option_data,
            &FindOptimalSide::Range(pos!(90.0), pos!(110.0))
        ));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_short_option_upper_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(100),     // open_interest
        );
        assert!(strategy.is_valid_optimal_option(&option_data, &FindOptimalSide::Upper));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_short_option_lower_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(100),     // open_interest
        );
        assert!(strategy.is_valid_optimal_option(&option_data, &FindOptimalSide::Lower));
    }
}

#[cfg(test)]
mod tests_strategy_net_operations {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use chrono::{TimeZone, Utc};

    struct TestStrategy {
        positions: Vec<Position>,
    }

    impl TestStrategy {
        fn new() -> Self {
            Self {
                positions: Vec::new(),
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {
        fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
            self.positions.push(position.clone());
            Ok(())
        }

        fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
            Ok(self.positions.iter().collect())
        }
    }

    impl BreakEvenable for TestStrategy {}

    impl BasicAble for TestStrategy {}

    impl Strategies for TestStrategy {}

    #[test]

    fn test_net_cost_calculation() {
        let mut strategy = TestStrategy::new();
        let option_long = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let option_short = create_sample_option_simplest(OptionStyle::Call, Side::Short);

        let position_long = Position::new(
            option_long,
            Positive::ONE,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            pos!(1.0),
            pos!(0.5),
        );
        let position_short = Position::new(
            option_short,
            Positive::ONE,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            pos!(1.0),
            pos!(0.5),
        );

        strategy.add_position(&position_long).unwrap();
        strategy.add_position(&position_short).unwrap();

        let result = strategy.get_net_cost().unwrap();
        assert!(result > Decimal::ZERO);
    }

    #[test]

    fn test_net_premium_received_calculation() {
        let mut strategy = TestStrategy::new();
        let option_long = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let option_short = create_sample_option_simplest(OptionStyle::Call, Side::Short);

        let fixed_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let position_long =
            Position::new(option_long, Positive::ONE, fixed_date, pos!(1.0), pos!(0.5));
        let position_short = Position::new(
            option_short,
            Positive::ONE,
            fixed_date,
            pos!(1.0),
            pos!(0.5),
        );

        strategy.add_position(&position_long).unwrap();
        strategy.add_position(&position_short).unwrap();

        let result = strategy.get_net_premium_received().unwrap();
        assert!(result == Positive::ZERO);
    }

    #[test]

    fn test_fees_calculation() {
        let mut strategy = TestStrategy::new();
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let fixed_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let position = Position::new(option, Positive::ONE, fixed_date, pos!(1.0), pos!(0.5));

        strategy.add_position(&position).unwrap();

        let result = strategy.get_fees().unwrap();
        assert!(result > Positive::ZERO);
    }
}
