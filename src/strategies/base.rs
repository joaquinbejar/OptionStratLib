use crate::chains::OptionData;
use crate::constants::{STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER};
use crate::error::strategies::BreakEvenErrorKind;
use crate::{
    ExpirationDate, Options, Positive,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
    error::{OperationErrorKind, position::PositionError, strategies::StrategyError},
    greeks::Greeks,
    model::{
        Trade,
        position::Position,
        types::{Action, OptionBasicType, OptionStyle, OptionType, Side},
    },
    pnl::PnLCalculator,
    pricing::payoff::Profit,
    strategies::{
        StrategyConstructor,
        delta_neutral::DeltaNeutrality,
        probabilities::core::ProbabilityAnalysis,
        utils::{FindOptimalSide, OptimizationCriteria, calculate_price_range},
    },
    visualization::Graph,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;
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

/// A trait that defines basic operations and attributes for managing options-related strategies.
///
/// This trait provides methods to retrieve various properties and mappings
/// associated with options, such as title, symbol, strike prices, sides, styles,
/// expiration dates, and implied volatility.
///
/// # Note
/// Several methods in this trait are unimplemented and will panic if invoked
/// without being explicitly implemented for the specific strategy.
///
/// # Methods
/// - `get_title`: Returns the title of the strategy.
/// - `get_option_basic_type`: Retrieves a set of basic option types.
/// - `get_symbol`: Returns the symbol associated with the option.
/// - `get_strike`: Maps option basic types to their positive strike values.
/// - `get_strikes`: Returns a vector of strike prices.
/// - `get_side`: Maps option basic types to their respective sides.
/// - `get_type`: Retrieves the type of the option.
/// - `get_style`: Maps option basic types to their corresponding styles.
/// - `get_expiration`: Maps option basic types to their expiration dates.
/// - `get_implied_volatility`: Retrieves implied volatility (currently unimplemented).
///
/// # Panics
/// All methods with `unimplemented!` will panic when called unless properly implemented.
/// Refer to the documentation for individual methods for more details.
///
pub trait BasicAble {
    /// Retrieves the title associated with the current instance of the strategy.
    ///
    /// # Returns
    /// A `String` representing the title.  
    ///
    /// # Panics
    /// This method is not yet implemented and will panic with the message
    /// `"get_title is not implemented for this strategy"`. Ensure this method
    /// is properly implemented before using it.
    ///
    fn get_title(&self) -> String {
        unimplemented!("get_title is not implemented for this strategy");
    }
    /// Retrieves a `HashSet` of `OptionBasicType` values associated with the current strategy.
    ///
    /// # Returns
    /// A `HashSet` containing the `OptionBasicType` elements relevant to the strategy.
    /// However, this method is currently not implemented and will panic with the message
    /// `"get_option_basic_type is not implemented for this strategy"`.
    ///
    /// # Panics
    /// This function will panic with the message:
    /// `"get_option_basic_type is not implemented for this strategy"` if called.
    ///
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType> {
        unimplemented!("get_option_basic_type is not implemented for this strategy");
    }
    /// Retrieves the symbol associated with the current instance by delegating the call to the `get_symbol`
    /// method of the `one_option` object.
    ///
    /// # Returns
    /// A string slice (`&str`) that represents the symbol.
    ///
    /// # Notes
    /// - Assumes that `one_option()` is a method that returns an object or reference which implements
    ///   a `get_symbol()` method.
    /// - The returned `&str` is borrowed from the referenced object, and its lifetime is tied to the `self` instance.
    fn get_symbol(&self) -> &str {
        self.one_option().get_symbol()
    }
    /// Retrieves a mapping of option basic types to their associated positive strike values.
    ///
    /// This method delegates the call to the `get_strike` method of the `one_option` object,
    /// returning a `HashMap` where each key is an `OptionBasicType` and each value is a reference
    /// to a `Positive` strike value.
    ///
    /// # Returns
    /// A `HashMap` where:
    /// - `OptionBasicType` represents the type of the option,
    /// - `&Positive` is a reference to the positive strike value associated with the option.
    ///
    /// # Notes
    /// - Ensure that the `one_option` method returns a valid object that implements a `get_strike` method.
    /// - The values in the returned map are references, so their lifetime is tied to the ownership of `self`.
    ///
    fn get_strike(&self) -> HashMap<OptionBasicType, &Positive> {
        self.one_option().get_strike()
    }
    /// Retrieves a vector of strike prices from the option types.
    ///
    /// This function accesses the `OptionBasicType` objects associated with the instance,
    /// extracts their `strike_price` fields, and collects those values into a `Vec<&Positive>`.
    ///
    /// # Returns
    ///
    /// A vector containing references to the strike prices (`&Positive`) of the associated option types.
    ///
    /// # Notes
    ///
    /// - The method assumes that `self.get_option_basic_type()` returns a collection of
    ///   objects that have a `strike_price` field.
    /// - The `strike_price` type is `&Positive`, which implies it references a type with
    ///   positive constraints.
    fn get_strikes(&self) -> Vec<&Positive> {
        self.get_option_basic_type()
            .iter()
            .map(|option_type| option_type.strike_price)
            .collect()
    }
    /// Retrieves a `HashMap` that maps each `OptionBasicType` to its corresponding `Side`.
    ///
    /// This method generates a mapping by iterating over the result of `get_option_basic_type`
    /// and pairing each `OptionBasicType` with its associated `Side`.
    ///
    /// # Returns
    ///
    /// A `HashMap` where:
    /// - The keys are `OptionBasicType` values.
    /// - The values are `Side` references corresponding to each `OptionBasicType`.
    ///
    /// # Panics
    ///
    /// This function assumes that `option_type.side` is valid for all elements
    /// in the iterator returned by `get_option_basic_type`. If this assumption is violated,
    /// the behavior is undefined.
    ///
    /// # Notes
    ///
    /// Ensure that `get_option_basic_type` is properly implemented and returns
    /// an iterable collection of `OptionBasicType` elements that have valid `Side` associations.
    fn get_side(&self) -> HashMap<OptionBasicType, &Side> {
        self.get_option_basic_type()
            .iter()
            .map(|option_type| (*option_type, option_type.side))
            .collect()
    }
    /// Retrieves the type of the option.
    ///
    /// This method provides access to the `OptionType` of the associated option
    /// by calling the `get_type` method on the result of `self.one_option()`.
    ///
    /// # Returns
    /// A reference to the `OptionType` of the associated option.
    ///
    /// # Notes
    /// - Ensure `self.one_option()` returns a valid object with a callable `get_type`
    ///   method to avoid runtime errors.
    fn get_type(&self) -> &OptionType {
        self.one_option().get_type()
    }
    /// Retrieves a mapping of `OptionBasicType` to their corresponding `OptionStyle`.
    ///
    /// This function generates a `HashMap` where each `OptionBasicType` returned by
    /// the `get_option_basic_type` method is associated with its respective `OptionStyle`.
    ///
    /// # Returns
    /// A `HashMap` where:
    /// - The keys are `OptionBasicType` values (basic option types).
    /// - The values are references to the associated `OptionStyle` for each option type.
    ///
    /// # Note
    /// Ensure that `get_option_basic_type` returns a valid iterator of `OptionBasicType`
    /// items before calling this function, as the result depends on its output.
    ///
    /// # Panics
    /// This function will panic if the `OptionStyle` reference is invalid or not properly
    /// initialized for any `OptionBasicType`.
    fn get_style(&self) -> HashMap<OptionBasicType, &OptionStyle> {
        self.get_option_basic_type()
            .iter()
            .map(|option_type| (*option_type, option_type.option_style))
            .collect()
    }
    /// Retrieves a map of option basic types to their corresponding expiration dates.
    ///
    /// This method iterates over the collection of option basic types and creates a `HashMap`
    /// where each key is an `OptionBasicType` and the value is a reference to its associated
    /// `ExpirationDate`.
    ///
    /// # Returns
    /// A `HashMap` where:
    /// - The key is of type `OptionBasicType`.
    /// - The value is a reference to the `ExpirationDate` associated with the option basic type.
    ///
    /// # Panics
    /// This method may panic if `expiration_date` is unexpectedly `None` within the option type,
    /// depending on your implementation of `get_option_basic_type`.
    ///
    /// # Notes
    /// - Ensure `self.get_option_basic_type()` returns a valid iterable of `OptionBasicType` instances.
    /// - The lifetime of the returned `ExpirationDate` references is tied to the lifetime of `self`.
    fn get_expiration(&self) -> HashMap<OptionBasicType, &ExpirationDate> {
        self.get_option_basic_type()
            .iter()
            .map(|option_type| (*option_type, option_type.expiration_date))
            .collect()
    }
    /// Retrieves the implied volatility for the current strategy.
    ///
    /// # Returns
    ///
    /// A `HashMap` where the key is of type `OptionBasicType` and
    /// the value is a reference to a `Positive` value. Each key-value
    /// pair corresponds to the implied volatility associated with a
    /// specific option type.
    ///
    /// # Notes
    ///
    /// This method is not yet implemented for the specific strategy
    /// and will panic when invoked.
    ///
    /// # Panics
    ///
    /// This function will unconditionally panic with the message
    /// `"get_implied_volatility is not implemented for this strategy"`.
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType, &Positive> {
        unimplemented!("get_implied_volatility is not implemented for this strategy");
    }
    /// Retrieves the quantity information associated with the strategy.
    ///
    /// # Returns
    /// A `HashMap` that holds pairs of `OptionBasicType` (the key) and a reference
    /// to a `Positive` value (the value). This map represents the mapping of
    /// option basic types to their respective quantities.
    ///
    /// # Notes
    /// This method is not implemented in the current strategy and will
    /// panic when called.
    ///
    /// # Panics
    /// This function will panic with the message `"get_quantity is not implemented for this strategy"`.
    ///
    /// # Example
    /// The function currently serves as a placeholder and should be implemented
    /// in a specific strategy that defines its behavior.
    fn get_quantity(&self) -> HashMap<OptionBasicType, &Positive> {
        unimplemented!("get_quantity is not implemented for this strategy");
    }
    /// Retrieves the underlying price of the financial instrument (e.g., option).
    ///
    /// This method fetches the underlying price from the associated `one_option`
    /// instance, ensuring that the value is positive.
    ///
    /// # Returns
    ///
    /// A reference to a `Positive` value representing the underlying price.
    ///
    /// # Notes
    ///
    /// This method assumes that the underlying price is always available and valid.
    fn get_underlying_price(&self) -> &Positive {
        self.one_option().get_underlying_price()
    }
    /// Retrieves the risk-free interest rate associated with a given set of options.
    ///
    /// This function retrieves the risk-free rate from a single option
    /// and returns it as a `HashMap`, where the keys correspond to the `OptionBasicType`
    /// and the values are references to the respective `Decimal` values.
    ///
    /// # Returns
    ///
    /// A `HashMap` where:
    /// - The key is of type `OptionBasicType`, representing the unique identifier or type of option.
    /// - The value is a reference (`&Decimal`) to the corresponding risk-free rate.
    ///
    /// # Notes
    ///
    /// - The method relies on the `one_option()` function to retrieve the required data.
    /// - Ensure that the `one_option()` method is implemented correctly to fetch the necessary risk-free rates.
    ///
    /// # Errors
    ///
    /// This function assumes that `one_option` and its underlying functionality
    /// are error-free. Errors, if any, must be handled within `one_option`.
    fn get_risk_free_rate(&self) -> HashMap<OptionBasicType, &Decimal> {
        self.one_option().get_risk_free_rate()
    }
    /// Retrieves the dividend yield of a financial option.
    ///
    /// This method calls the `get_dividend_yield` function of the associated `one_option()`
    /// method and returns a `HashMap` containing the dividend yield information. The keys
    /// of the map are of type `OptionBasicType`, and the values are references to instances
    /// of `Positive`.
    ///
    /// # Returns
    /// * `HashMap<OptionBasicType, &Positive>`: A mapping of option basic types to their
    ///   respective positive dividend yield values.
    ///
    /// # Note
    /// Ensure that the associated `one_option()` method is correctly implemented
    /// and provides the desired dividend yield information.
    fn get_dividend_yield(&self) -> HashMap<OptionBasicType, &Positive> {
        self.one_option().get_dividend_yield()
    }
    /// This method, `one_option`, is designed to retrieve a reference to an `Options` object.
    /// However, in this implementation, the function is not currently functional, as it
    /// explicitly triggers an unimplemented error when called.
    ///
    /// # Returns
    /// * `&Options` - A reference to an `Options` object. However, this is not currently
    ///   available due to the method being unimplemented.
    ///
    /// # Panics
    /// This method will unconditionally panic with the message
    /// "one_option is not implemented for this strategy" whenever it is invoked.
    ///
    /// # Note
    /// This is a placeholder implementation and should be overridden or implemented in
    /// a concrete type where this function is required.
    fn one_option(&self) -> &Options {
        unimplemented!("one_option is not implemented for this strategy");
    }
    /// Provides a mutable reference to an `Options` instance.
    ///
    /// This function is intended to allow mutation of a single
    /// `Options` instance managed within the strategy. It is
    /// a stub and not currently implemented. When called,
    /// it will panic with the message "one_option_mut is not implemented
    /// for this strategy".
    ///
    /// # Errors
    ///
    /// Panics if this function is called since it is unimplemented.
    ///
    /// # Returns
    ///
    /// A mutable reference to an `Options` instance (in a fully
    /// implemented version of this function).
    ///
    fn one_option_mut(&mut self) -> &mut Options {
        unimplemented!("one_option_mut is not implemented for this strategy");
    }

    /// Sets the expiration date for the strategy.
    ///
    /// This method is intended to allow the user to define an expiration date
    /// for a given strategy. However, it is currently unimplemented for this
    /// specific strategy and will result in a panic with a message indicating
    /// that it is not supported.
    ///
    /// # Parameters
    ///
    /// - `_expiration_date`: The expiration date to set for the strategy,
    ///   represented as an `ExpirationDate` object. This parameter is accepted
    ///   but not utilized, as the method is not implemented.
    ///
    /// # Returns
    ///
    /// This function returns a `Result`:
    /// - `Ok(())` if the operation is successful (not applicable here as the
    ///   function is unimplemented).
    /// - `Err(StrategyError)` if an error occurs (though, in this case, the
    ///   method only panics as it is unimplemented).
    ///
    /// # Errors
    ///
    /// Always returns a panic with the message:
    /// `"set_expiration_date is not implemented for this strategy"`. No actual
    /// `StrategyError` is produced by this method, as it is incomplete.
    ///
    /// # Panics
    ///
    /// This function always panics when called with the message:
    /// `"set_expiration_date is not implemented for this strategy"`.
    ///
    /// Note: Avoid using this method until it is fully implemented for this
    /// specific strategy.
    fn set_expiration_date(
        &mut self,
        _expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        unimplemented!("set_expiration_date is not implemented for this strategy");
    }
    /// Sets the underlying price for this strategy.
    ///
    /// # Parameters
    /// - `_price`: A reference to a `Positive` value representing the new underlying price
    ///   to be set.
    ///
    /// # Returns
    /// - `Ok(())` if the operation is successful.
    /// - `Err(StrategyError)` if an error occurs during the operation.
    ///
    /// # Note
    /// This function is currently not implemented for this strategy. Calling
    /// this function will result in a runtime panic with the message
    /// "set_underlying_price is not implemented for this strategy".
    ///
    /// # Panics
    /// Always panics with the message `"set_underlying_price is not implemented for this strategy"`.
    ///
    fn set_underlying_price(&mut self, _price: &Positive) -> Result<(), StrategyError> {
        unimplemented!("set_underlying_price is not implemented for this strategy");
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
    fn set_implied_volatility(&mut self, _volatility: &Positive) -> Result<(), StrategyError> {
        unimplemented!("set_implied_volatility is not implemented for this strategy");
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

    /// Returns the minimum and maximum strike prices from the positions in the strategy.
    /// Considers underlying price when applicable, ensuring the returned range includes it.
    ///
    /// # Returns
    /// * `Ok((Positive, Positive))` - A tuple containing the minimum and maximum strike prices.
    /// * `Err(StrategyError)` - If no strikes are found or if an error occurs retrieving positions.
    fn get_max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> {
        let strikes: Vec<&Positive> = self.get_strikes();
        if strikes.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "max_min_strikes".to_string(),
                    reason: "No strikes found".to_string(),
                },
            ));
        }

        let min = strikes.iter().fold(Positive::INFINITY, |acc, &strike| {
            Positive::min(acc, *strike)
        });
        let max = strikes
            .iter()
            .fold(Positive::ZERO, |acc, &strike| Positive::max(acc, *strike));

        let underlying_price = self.get_underlying_price();
        let mut min_value = min;
        let mut max_value = max;

        if underlying_price != &Positive::ZERO {
            if min_value > *underlying_price {
                min_value = *underlying_price;
            }
            if *underlying_price > max_value {
                max_value = *underlying_price;
            }
        }

        Ok((min_value, max_value))
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
mod tests_strategies_extended {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;

    #[test]
    fn test_strategy_enum() {
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
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
        impl BasicAble for EmptyStrategy {
            fn get_option_basic_type(&self) -> HashSet<OptionBasicType> {
                HashSet::new()
            }
        }
        impl Strategies for EmptyStrategy {}

        let strategy = EmptyStrategy;
        assert_eq!(strategy.get_strikes(), Vec::<&Positive>::new());
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
mod tests_best_range_to_show {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(underlying_price: Positive, break_even_points: Vec<Positive>) -> Self {
            Self {
                underlying_price,
                break_even_points,
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
        fn get_option_basic_type(&self) -> HashSet<OptionBasicType> {
            HashSet::new()
        }
    }

    impl Strategies for TestStrategy {
        fn get_max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> {
            Ok((pos!(90.0), pos!(100.0)))
        }
    }

    #[test]
    fn test_basic_range_with_step() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(90.0), pos!(110.0)]);
        let range = strategy.get_best_range_to_show(pos!(5.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], pos!(5.0));
    }

    #[test]
    fn test_range_with_small_step() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(95.0), pos!(105.0)]);
        let range = strategy.get_best_range_to_show(pos!(1.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], pos!(1.0));
    }

    #[test]
    fn test_range_boundaries() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(90.0), pos!(110.0)]);
        let range = strategy.get_best_range_to_show(pos!(5.0)).unwrap();
        assert!(range.first().unwrap() < &pos!(90.0));
        assert!(range.last().unwrap() > &pos!(110.0));
    }

    #[test]
    fn test_range_step_size() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(90.0), pos!(110.0)]);
        let step = pos!(5.0);
        let range = strategy.get_best_range_to_show(step).unwrap();

        for i in 1..range.len() {
            assert_eq!(range[i] - range[i - 1], step);
        }
    }

    #[test]
    fn test_range_includes_underlying() {
        let underlying_price = pos!(100.0);
        let strategy = TestStrategy::new(underlying_price, vec![pos!(90.0), pos!(110.0)]);
        let range = strategy.get_best_range_to_show(pos!(5.0)).unwrap();

        assert!(range.iter().any(|&price| price <= underlying_price));
        assert!(range.iter().any(|&price| price >= underlying_price));
    }

    #[test]
    fn test_range_with_extreme_values() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(50.0), pos!(150.0)]);
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
    }

    impl TestStrategy {
        fn new(underlying_price: Positive, break_even_points: Vec<Positive>) -> Self {
            Self {
                underlying_price,
                break_even_points,
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
        fn get_option_basic_type(&self) -> HashSet<OptionBasicType> {
            HashSet::new()
        }
        fn get_underlying_price(&self) -> &Positive {
            &self.underlying_price
        }
    }

    impl Strategies for TestStrategy {
        fn get_max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> {
            Ok((pos!(90.0), pos!(110.0)))
        }
    }

    #[test]
    fn test_basic_range() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(90.0), pos!(110.0)]);
        let (start, end) = strategy.get_range_to_show().unwrap();
        assert!(start < pos!(90.0));
        assert!(end > pos!(110.0));
    }

    #[test]
    fn test_range_with_far_strikes() {
        let strategy = TestStrategy::new(pos!(100.0), vec![pos!(90.0), pos!(110.0)]);
        let (start, end) = strategy.get_range_to_show().unwrap();
        assert!(start < pos!(90.0));
        assert!(end > pos!(110.0));
    }

    #[test]
    fn test_range_with_underlying_outside_strikes() {
        let strategy = TestStrategy::new(pos!(150.0), vec![pos!(90.0), pos!(110.0)]);
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
}

#[cfg(test)]
mod tests_optimizable {
    use super::*;
    use crate::chains::OptionData;
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
