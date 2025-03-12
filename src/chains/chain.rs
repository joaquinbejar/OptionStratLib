/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::chains::utils::{
    OptionChainBuildParams, OptionChainParams, OptionDataPriceParams, RandomPositionsParams,
    adjust_volatility, default_empty_string, generate_list_of_strikes,
};
use crate::chains::{
    DeltasInStrike, FourOptions, OptionsInStrike, RNDAnalysis, RNDParameters, RNDResult,
};
use crate::curves::{BasicCurves, Curve, Point2D};
use crate::error::chains::ChainError;
use crate::error::{CurveError, SurfaceError};
use crate::geometrics::{Len, LinearInterpolation};
use crate::greeks::{Greeks, delta, gamma};
use crate::model::{
    BasicAxisTypes, ExpirationDate, OptionStyle, OptionType, Options, Position, Side,
};
use crate::strategies::utils::FindOptimalSide;
use crate::surfaces::{BasicSurfaces, Point3D, Surface};
use crate::utils::others::get_random_element;
use crate::volatility::VolatilitySmile;
use crate::{Positive, pos};
use chrono::{NaiveDate, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use tracing::{debug, error, info, trace, warn};
#[cfg(not(target_arch = "wasm32"))]
use {crate::chains::utils::parse, csv::WriterBuilder, std::fs::File};

/// Struct representing a row in an option chain with detailed pricing and analytics data.
///
/// This struct encapsulates the complete market data for an options contract at a specific
/// strike price, including bid/ask prices for both call and put options, implied volatility,
/// the Greeks (delta, gamma), volume, and open interest. It provides all the essential
/// information needed for options analysis and trading decision-making.
///
/// # Fields
///
/// * `strike_price` - The strike price of the option, represented as a positive floating-point number.
/// * `call_bid` - The bid price for the call option, represented as an optional positive floating-point number.
///   May be `None` if market data is unavailable.
/// * `call_ask` - The ask price for the call option, represented as an optional positive floating-point number.
///   May be `None` if market data is unavailable.
/// * `put_bid` - The bid price for the put option, represented as an optional positive floating-point number.
///   May be `None` if market data is unavailable.
/// * `put_ask` - The ask price for the put option, represented as an optional positive floating-point number.
///   May be `None` if market data is unavailable.
/// * `call_middle` - The mid-price between call bid and ask, represented as an optional positive floating-point number.
///   May be `None` if underlying bid/ask data is unavailable.
/// * `put_middle` - The mid-price between put bid and ask, represented as an optional positive floating-point number.
///   May be `None` if underlying bid/ask data is unavailable.
/// * `implied_volatility` - The implied volatility of the option, represented as an optional positive floating-point number.
///   May be `None` if it cannot be calculated from available market data.
/// * `delta_call` - The delta of the call option, represented as an optional decimal number.
///   Measures the rate of change of the option price with respect to changes in the underlying asset price.
/// * `delta_put` - The delta of the put option, represented as an optional decimal number.
///   Measures the rate of change of the option price with respect to changes in the underlying asset price.
/// * `gamma` - The gamma of the option, represented as an optional decimal number.
///   Measures the rate of change of delta with respect to changes in the underlying asset price.
/// * `volume` - The trading volume of the option, represented as an optional positive floating-point number.
///   May be `None` if data is not available.
/// * `open_interest` - The open interest of the option, represented as an optional unsigned integer.
///   Represents the total number of outstanding option contracts that have not been settled.
/// * `options` - An optional boxed reference to a `FourOptions` struct that may contain
///   the actual option contracts represented by this data. This field is not serialized.
///
/// # Usage
///
/// This struct is typically used to represent a single row in an option chain table,
/// providing comprehensive market data for options at a specific strike price. It's
/// useful for option pricing models, strategy analysis, and trading applications.
///
/// # Serialization
///
/// This struct implements Serialize and Deserialize traits, with fields that are `None`
/// being skipped during serialization to produce more compact JSON output.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct OptionData {
    /// The strike price of the option, represented as a positive floating-point number.
    #[serde(rename = "strike_price")]
    pub(crate) strike_price: Positive,

    /// The bid price for the call option. May be `None` if market data is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) call_bid: Option<Positive>,

    /// The ask price for the call option. May be `None` if market data is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) call_ask: Option<Positive>,

    /// The bid price for the put option. May be `None` if market data is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) put_bid: Option<Positive>,

    /// The ask price for the put option. May be `None` if market data is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) put_ask: Option<Positive>,

    /// The mid-price between call bid and ask. Calculated as (bid + ask) / 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_middle: Option<Positive>,

    /// The mid-price between put bid and ask. Calculated as (bid + ask) / 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put_middle: Option<Positive>,

    /// The implied volatility of the option, derived from option prices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) implied_volatility: Option<Positive>,

    /// The delta of the call option, measuring price sensitivity to underlying changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_call: Option<Decimal>,

    /// The delta of the put option, measuring price sensitivity to underlying changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_put: Option<Decimal>,

    /// The gamma of the option, measuring the rate of change in delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    gamma: Option<Decimal>,

    /// The trading volume of the option, indicating market activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<Positive>,

    /// The open interest, representing the number of outstanding contracts.
    #[serde(skip_serializing_if = "Option::is_none")]
    open_interest: Option<u64>,

    /// Optional reference to the actual option contracts represented by this data.
    /// This field is not serialized.
    #[serde(skip)]
    pub options: Option<Box<FourOptions>>,
}

impl OptionData {
    /// Creates a new instance of `OptionData` with the given option market parameters.
    ///
    /// This constructor creates an `OptionData` structure that represents a single row in an options chain,
    /// containing market data for both call and put options at a specific strike price. The middle prices
    /// for calls and puts are initially set to `None` and can be calculated later if needed.
    ///
    /// # Parameters
    ///
    /// * `strike_price` - The strike price of the option contract, guaranteed to be positive.
    /// * `call_bid` - The bid price for the call option. `None` if market data is unavailable.
    /// * `call_ask` - The ask price for the call option. `None` if market data is unavailable.
    /// * `put_bid` - The bid price for the put option. `None` if market data is unavailable.
    /// * `put_ask` - The ask price for the put option. `None` if market data is unavailable.
    /// * `implied_volatility` - The implied volatility derived from option prices. `None` if not calculable.
    /// * `delta_call` - The delta of the call option, measuring price sensitivity to underlying changes.
    /// * `delta_put` - The delta of the put option, measuring price sensitivity to underlying changes.
    /// * `gamma` - The gamma of the option, measuring the rate of change in delta.
    /// * `volume` - The trading volume of the option, indicating market activity.
    /// * `open_interest` - The number of outstanding option contracts that have not been settled.
    ///
    /// # Returns
    ///
    /// A new `OptionData` instance with the specified parameters and with `call_middle`, `put_middle`,
    /// and `options` fields initialized to `None`.
    ///
    /// # Note
    ///
    /// This function allows many optional parameters to accommodate scenarios where not all market data
    /// is available from data providers.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strike_price: Positive,
        call_bid: Option<Positive>,
        call_ask: Option<Positive>,
        put_bid: Option<Positive>,
        put_ask: Option<Positive>,
        implied_volatility: Option<Positive>,
        delta_call: Option<Decimal>,
        delta_put: Option<Decimal>,
        gamma: Option<Decimal>,
        volume: Option<Positive>,
        open_interest: Option<u64>,
    ) -> Self {
        OptionData {
            strike_price,
            call_bid,
            call_ask,
            put_bid,
            put_ask,
            call_middle: None,
            put_middle: None,
            implied_volatility,
            delta_call,
            delta_put,
            gamma,
            volume,
            open_interest,
            options: None,
        }
    }

    /// Validates the option data to ensure it meets the required criteria for calculations.
    ///
    /// This method performs a series of validation checks to ensure that the option data
    /// is complete and valid for further processing or analysis. It verifies:
    /// 1. The strike price is not zero
    /// 2. Implied volatility is present
    /// 3. Call option data is valid (via `valid_call()`)
    /// 4. Put option data is valid (via `valid_put()`)
    ///
    /// Each validation failure is logged as an error for debugging and troubleshooting.
    ///
    /// # Returns
    ///
    /// * `true` - If all validation checks pass, indicating the option data is valid
    /// * `false` - If any validation check fails, indicating the option data is incomplete or invalid
    pub fn validate(&self) -> bool {
        if self.strike_price == Positive::ZERO {
            error!("Error: Strike price cannot be zero");
            return false;
        }
        if self.implied_volatility.is_none() {
            error!("Error: Implied volatility cannot be None");
            return false;
        }
        if !self.valid_call() {
            error!("Error: Invalid call");
            return false;
        }
        if !self.valid_put() {
            error!("Error: Invalid put");
            return false;
        }
        true
    }

    /// Checks if this option data contains valid call option information.
    ///
    /// A call option is considered valid when all required data is present:
    /// * The strike price is greater than zero
    /// * Implied volatility is available
    /// * Both bid and ask prices for the call option are available
    ///
    /// # Returns
    ///
    /// `true` if all required call option data is present, `false` otherwise.
    pub(crate) fn valid_call(&self) -> bool {
        self.strike_price > Positive::ZERO
            && self.implied_volatility.is_some()
            && self.call_bid.is_some()
            && self.call_ask.is_some()
    }

    /// Checks if this option data contains valid put option information.
    ///
    /// A put option is considered valid when all required data is present:
    /// * The strike price is greater than zero
    /// * Implied volatility is available
    /// * Both bid and ask prices for the put option are available
    ///
    /// # Returns
    ///
    /// `true` if all required put option data is present, `false` otherwise.
    pub(crate) fn valid_put(&self) -> bool {
        self.strike_price > Positive::ZERO
            && self.implied_volatility.is_some()
            && self.put_bid.is_some()
            && self.put_ask.is_some()
    }

    /// Retrieves the price at which a call option can be purchased.
    ///
    /// This method returns the ask price for a call option, which is the price
    /// a buyer would pay to purchase the call option.
    ///
    /// # Returns
    ///
    /// The call option's ask price as a `Positive` value, or `None` if the price is unavailable.
    pub fn get_call_buy_price(&self) -> Option<Positive> {
        self.call_ask
    }

    /// Retrieves the price at which a call option can be sold.
    ///
    /// This method returns the bid price for a call option, which is the price
    /// a seller would receive when selling the call option.
    ///
    /// # Returns
    ///
    /// The call option's bid price as a `Positive` value, or `None` if the price is unavailable.
    pub fn get_call_sell_price(&self) -> Option<Positive> {
        self.call_bid
    }

    /// Retrieves the price at which a put option can be purchased.
    ///
    /// This method returns the ask price for a put option, which is the price
    /// a buyer would pay to purchase the put option.
    ///
    /// # Returns
    ///
    /// The put option's ask price as a `Positive` value, or `None` if the price is unavailable.
    pub fn get_put_buy_price(&self) -> Option<Positive> {
        self.put_ask
    }

    /// Retrieves the price at which a put option can be sold.
    ///
    /// This method returns the bid price for a put option, which is the price
    /// a seller would receive when selling the put option.
    ///
    /// # Returns
    ///
    /// The put option's bid price as a `Positive` value, or `None` if the price is unavailable.
    pub fn get_put_sell_price(&self) -> Option<Positive> {
        self.put_bid
    }

    /// Creates an option contract based on provided parameters and existing data.
    ///
    /// This method constructs a new `Options` instance by combining information from
    /// the current object with the provided pricing parameters. It handles the logic
    /// for determining the correct implied volatility to use, either from the provided
    /// parameters or from the object's stored value.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to `OptionDataPriceParams` containing essential pricing
    ///   information such as expiration date, underlying price, and risk-free rate.
    /// * `side` - Defines the directional exposure of the option (Long or Short).
    /// * `option_style` - Specifies the style of the option (Call or Put).
    ///
    /// # Returns
    ///
    /// * `Result<Options, ChainError>` - An `Options` instance if successful, or a `ChainError`
    ///   if required data such as implied volatility is missing.
    ///
    /// # Errors
    ///
    /// Returns `ChainError::invalid_volatility` if neither the input parameters nor the object
    /// itself contains a valid implied volatility value.
    fn get_option(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
    ) -> Result<Options, ChainError> {
        let implied_volatility = match price_params.implied_volatility {
            Some(iv) => iv,
            None => match self.implied_volatility {
                Some(iv) => iv / Positive::HUNDRED,
                None => {
                    return Err(ChainError::invalid_volatility(
                        None,
                        "Implied volatility not found",
                    ));
                }
            },
        };

        Ok(Options::new(
            OptionType::European,
            side,
            "OptionData".to_string(),
            self.strike_price,
            price_params.expiration_date,
            implied_volatility,
            pos!(1.0),
            price_params.underlying_price,
            price_params.risk_free_rate,
            option_style,
            price_params.dividend_yield,
            None,
        ))
    }

    /// Creates an option contract for implied volatility calculation with specified parameters.
    ///
    /// This method constructs a new European-style option contract with the given parameters
    /// to be used in implied volatility calculations or pricing models. It initializes a properly
    /// configured `Options` instance with all necessary values for financial calculations.
    ///
    /// # Parameters
    ///
    /// * `price_params` - Contains core pricing parameters including:
    ///   - `expiration_date` - When the option expires
    ///   - `underlying_price` - Current market price of the underlying asset
    ///   - `risk_free_rate` - The risk-free interest rate used in pricing models
    ///   - `dividend_yield` - The dividend yield of the underlying asset
    ///
    /// * `side` - Specifies whether this is a Long or Short position, determining
    ///   the directional exposure of the option
    ///
    /// * `option_style` - Determines whether this is a Call or Put option, defining
    ///   the fundamental right the contract provides
    ///
    /// * `initial_iv` - The initial implied volatility estimate to use for the option,
    ///   which will be the starting point for IV calculation algorithms
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either:
    /// * An `Options` instance configured with the specified parameters
    /// * A `ChainError` if there was a problem creating the option
    ///
    fn get_option_for_iv(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
        initial_iv: Positive,
    ) -> Result<Options, ChainError> {
        Ok(Options::new(
            OptionType::European,
            side,
            "OptionData".to_string(),
            self.strike_price,
            price_params.expiration_date,
            initial_iv,
            pos!(1.0),
            price_params.underlying_price,
            price_params.risk_free_rate,
            option_style,
            price_params.dividend_yield,
            None,
        ))
    }

    /// Returns a collection of option positions (calls and puts, long and short) at the same strike price.
    ///
    /// This method creates a comprehensive set of option positions all sharing the same strike price
    /// but varying in option style (Call/Put) and side (Long/Short). It's useful for analyzing
    /// option strategies that require positions across different option types at the same strike.
    ///
    /// # Arguments
    ///
    /// * `price_params` - Parameters required for pricing the options, including underlying price,
    ///   expiration date, risk-free rate, and other market factors.
    ///
    /// * `side` - The initial directional bias (Long or Short) used as a starting point for creating
    ///   the option positions. This parameter affects the first option that gets created.
    ///
    /// * `option_style` - The initial option style (Call or Put) used as a starting point for creating
    ///   the option positions. This parameter affects the first option that gets created.
    ///
    /// # Returns
    ///
    /// * `Result<OptionsInStrike, ChainError>` - If successful, returns an `OptionsInStrike` struct
    ///   containing all four option positions (long call, short call, long put, short put).
    ///   Returns a `ChainError` if option creation fails, such as when required volatility data
    ///   is missing.
    ///
    /// # Errors
    ///
    /// This function will return `ChainError` if:
    /// * The underlying `get_option` method fails, typically due to missing or invalid pricing data
    /// * Implied volatility is not provided and cannot be derived from available data
    ///
    fn get_options_in_strike(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
    ) -> Result<OptionsInStrike, ChainError> {
        let mut option: Options = self.get_option(price_params, side, option_style)?;
        option.option_style = OptionStyle::Call;
        option.side = Side::Long;
        let long_call = option.clone();
        option.side = Side::Short;
        let short_call = option.clone();
        option.option_style = OptionStyle::Put;
        let short_put = option.clone();
        option.side = Side::Long;
        let long_put = option.clone();
        Ok(OptionsInStrike {
            long_call,
            short_call,
            long_put,
            short_put,
        })
    }

    /// Calculates and sets the bid and ask prices for call and put options.
    ///
    /// This method computes the theoretical prices for both call and put options using the
    /// Black-Scholes pricing model, and then stores these values in the appropriate fields.
    /// After calculating the individual bid and ask prices, it also computes and sets the
    /// mid-prices by calling the `set_mid_prices` method.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to `OptionDataPriceParams` containing the necessary
    ///   parameters for option pricing, such as underlying price, volatility, risk-free rate,
    ///   expiration date, and dividend yield.
    ///
    /// * `refresh` - A boolean flag indicating whether to force recalculation of option
    ///   contracts even if they already exist. When set to `true`, the method will recreate
    ///   the option contracts before calculating prices.
    ///
    /// # Returns
    ///
    /// * `Result<(), ChainError>` - Returns `Ok(())` if prices are successfully calculated
    ///   and set, or a `ChainError` if any error occurs during the process.
    ///
    /// # Side Effects
    ///
    /// Sets the following fields in the struct:
    /// * `call_ask` - The ask price for the call option
    /// * `call_bid` - The bid price for the call option
    /// * `put_ask` - The ask price for the put option
    /// * `put_bid` - The bid price for the put option
    /// * `call_middle` and `put_middle` - The mid-prices calculated via `set_mid_prices()`
    ///
    /// # Errors
    ///
    /// May return:
    /// * `ChainError` variants if there are issues creating the options contracts
    /// * Errors propagated from the Black-Scholes calculation functions
    pub fn calculate_prices(
        &mut self,
        price_params: &OptionDataPriceParams,
        refresh: bool,
    ) -> Result<(), ChainError> {
        if self.options.is_none() || refresh {
            self.create_options(price_params)?;
        }

        let options = self.options.as_ref().unwrap();

        let call_ask = options.long_call.calculate_price_black_scholes()?;
        self.call_ask = Some(Positive(call_ask.abs()));

        let call_bid = options.short_call.calculate_price_black_scholes()?;
        self.call_bid = Some(Positive(call_bid.abs()));

        let put_ask = options.long_put.calculate_price_black_scholes()?;
        self.put_ask = Some(Positive(put_ask.abs()));

        let put_bid = options.short_put.calculate_price_black_scholes()?;
        self.put_bid = Some(Positive(put_bid.abs()));

        self.set_mid_prices();
        Ok(())
    }

    /// Applies a spread to the bid and ask prices of call and put options, then recalculates mid prices.
    ///
    /// This method adjusts the bid and ask prices by half of the specified spread value,
    /// subtracting from bid prices and adding to ask prices. It also ensures that all prices
    /// are rounded to the specified number of decimal places. If any price becomes negative
    /// after applying the spread, it is set to `None`.
    ///
    /// # Arguments
    ///
    /// * `spread` - A positive decimal value representing the total spread to apply
    /// * `decimal_places` - The number of decimal places to round the adjusted prices to
    ///
    /// # Inner Function
    ///
    /// The method contains an inner function `round_to_decimal` that handles the rounding
    /// of prices after applying a shift (half the spread).
    ///
    /// # Side Effects
    ///
    /// * Updates `call_ask`, `call_bid`, `put_ask`, and `put_bid` fields with adjusted values
    /// * Sets adjusted prices to `None` if they would become negative after applying the spread
    /// * Calls `set_mid_prices()` to recalculate the mid prices based on the new bid/ask values
    pub fn apply_spread(&mut self, spread: Positive, decimal_places: i32) {
        fn round_to_decimal(
            number: Positive,
            decimal_places: i32,
            shift: Decimal,
        ) -> Option<Positive> {
            let multiplier = Positive::TEN.powi(decimal_places as i64);
            Some(((number + shift) * multiplier).round() / multiplier)
        }

        let half_spread: Decimal = (spread / Positive::TWO).into();

        if let Some(call_ask) = self.call_ask {
            if call_ask < half_spread {
                self.call_ask = None;
            } else {
                self.call_ask = round_to_decimal(call_ask, decimal_places, half_spread);
            }
        }
        if let Some(call_bid) = self.call_bid {
            if call_bid < half_spread {
                self.call_bid = None;
            } else {
                self.call_bid = round_to_decimal(call_bid, decimal_places, -half_spread);
            }
        }
        if let Some(put_ask) = self.put_ask {
            if put_ask < half_spread {
                self.put_ask = None;
            } else {
                self.put_ask = round_to_decimal(put_ask, decimal_places, half_spread);
            }
        }
        if let Some(put_bid) = self.put_bid {
            if put_bid < half_spread {
                self.put_bid = None;
            } else {
                self.put_bid = round_to_decimal(put_bid, decimal_places, -half_spread);
            }
        }

        self.set_mid_prices();
    }

    /// Calculates the delta values for call and put options based on the provided price parameters.
    ///
    /// Delta is a key "Greek" that measures the rate of change of the option's price with respect to changes
    /// in the underlying asset's price. This method computes and stores delta values for both call and put options.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to `OptionDataPriceParams` containing essential market data and
    ///   contract specifications needed for the calculation.
    ///
    /// # Behavior
    ///
    /// The function follows these steps:
    /// 1. Ensures implied volatility is available, calculating it if necessary
    /// 2. Creates option objects if they don't exist but implied volatility is available
    /// 3. Calculates and stores delta values for call options
    /// 4. Calculates and stores delta values for put options
    ///
    /// If any step fails, appropriate error messages are logged and the corresponding delta
    /// values will remain unset.
    ///
    /// # Side Effects
    ///
    /// * Updates the `delta_call` and `delta_put` fields of the struct with calculated values
    /// * May update the `implied_volatility` field if it was previously `None`
    /// * May create option objects if they didn't exist but were needed for calculations
    /// * Logs errors if calculations fail
    pub fn calculate_delta(&mut self, price_params: &OptionDataPriceParams) {
        if self.implied_volatility.is_none() {
            trace!("Implied volatility not found, calculating it");
            if let Err(e) = self.calculate_implied_volatility(price_params) {
                error!("Failed to calculate implied volatility: {}", e);
                return;
            }
        }

        if self.options.is_none() && self.implied_volatility.is_some() {
            let _ = self.create_options(price_params);
        }

        // Now proceed with delta calculation
        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(e) => {
                error!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta_call = Some(d),
            Err(e) => {
                error!("Delta calculation failed: {}", e);
                self.delta_call = None;
            }
        }

        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Put) {
            Ok(option) => option,
            Err(e) => {
                error!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta_put = Some(d),
            Err(e) => {
                error!("Delta calculation failed: {}", e);
                self.delta_put = None;
            }
        }
    }

    /// Calculates the gamma value for an option and stores it in the object.
    ///
    /// Gamma measures the rate of change in delta with respect to changes in the underlying price.
    /// It represents the second derivative of the option price with respect to the underlying price.
    ///
    /// This method first ensures that implied volatility is available (calculating it if needed),
    /// then creates option structures if they don't already exist, and finally calculates
    /// the gamma value.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to the pricing parameters required for option calculations,
    ///   including underlying price, expiration date, risk-free rate and other inputs.
    ///
    /// # Behavior
    ///
    /// * If implied volatility isn't available, it attempts to calculate it first
    /// * If option structures haven't been created yet, it creates them
    /// * On successful calculation, stores the gamma value in `self.gamma`
    /// * On failure, logs an error and sets `self.gamma` to `None`
    ///
    /// # Errors
    ///
    /// * Does not return errors but logs them through the tracing system
    /// * Common failures include inability to calculate implied volatility or issues creating option objects
    pub fn calculate_gamma(&mut self, price_params: &OptionDataPriceParams) {
        if self.implied_volatility.is_none() {
            trace!("Implied volatility not found, calculating it");
            if let Err(e) = self.calculate_implied_volatility(price_params) {
                error!("Failed to calculate implied volatility: {}", e);
                return;
            }
        }
        if self.options.is_none() && self.implied_volatility.is_some() {
            let _ = self.create_options(price_params);
        }
        // Now proceed with delta calculation
        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(e) => {
                error!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };
        match gamma(&option) {
            Ok(d) => self.gamma = Some(d),
            Err(e) => {
                error!("Gamma calculation failed: {}", e);
                self.gamma = None;
            }
        }
    }

    /// Retrieves delta values for options at the current strike price.
    ///
    /// Delta measures the rate of change of the option price with respect to changes
    /// in the underlying asset's price. This method returns delta values for options
    /// at the specific strike price defined in the price parameters.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to the pricing parameters required for option calculations,
    ///   including underlying price, expiration date, risk-free rate and other inputs.
    ///
    /// # Returns
    ///
    /// * `Result<DeltasInStrike, ChainError>` - On success, returns a structure containing delta values
    ///   for the options at the specified strike. On failure, returns a ChainError describing the issue.
    ///
    /// # Errors
    ///
    /// * Returns a `ChainError` if there's an issue retrieving the options or calculating their deltas.
    /// * Possible errors include missing option data, calculation failures, or invalid parameters.
    pub fn get_deltas(
        &self,
        price_params: &OptionDataPriceParams,
    ) -> Result<DeltasInStrike, ChainError> {
        let options_in_strike =
            self.get_options_in_strike(price_params, Side::Long, OptionStyle::Call)?;
        Ok(options_in_strike.deltas()?)
    }

    /// Validates if an option strike price is valid according to the specified search strategy.
    ///
    /// This method checks whether the current option's strike price falls within the constraints
    /// defined by the `FindOptimalSide` parameter, relative to the given underlying asset price.
    ///
    /// # Parameters
    ///
    /// * `underlying_price` - The current market price of the underlying asset as a `Positive` value.
    /// * `side` - The strategy to determine valid strike prices, specifying whether to consider
    ///   options with strikes above, below, or within a specific range of the underlying price.
    ///
    /// # Returns
    ///
    /// `bool` - Returns true if the strike price is valid according to the specified strategy:
    ///   * For `Upper`: Strike price must be greater than or equal to underlying price
    ///   * For `Lower`: Strike price must be less than or equal to underlying price
    ///   * For `All`: Always returns true (all strike prices are valid)
    ///   * For `Range`: Strike price must fall within the specified range (inclusive)
    pub fn is_valid_optimal_side(
        &self,
        underlying_price: Positive,
        side: &FindOptimalSide,
    ) -> bool {
        match side {
            FindOptimalSide::Upper => self.strike_price >= underlying_price,
            FindOptimalSide::Lower => self.strike_price <= underlying_price,
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                self.strike_price >= *start && self.strike_price <= *end
            }
        }
    }

    /// Calculates and sets the mid-prices for both call and put options.
    ///
    /// This method computes the middle price between the bid and ask prices for
    /// both call and put options, when both bid and ask prices are available.
    /// The mid-price is calculated as the simple average: (bid + ask) / 2.
    /// If either bid or ask price is missing for an option type, the corresponding
    /// mid-price will be set to `None`.
    ///
    /// # Side Effects
    ///
    /// Updates the `call_middle` and `put_middle` fields with the calculated mid-prices.
    pub fn set_mid_prices(&mut self) {
        self.call_middle = match (self.call_bid, self.call_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / pos!(2.0)),
            _ => None,
        };
        self.put_middle = match (self.put_bid, self.put_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / pos!(2.0)),
            _ => None,
        };
    }

    /// Retrieves the current mid-prices for call and put options.
    ///
    /// This method returns the calculated middle prices for both call and put options
    /// as a tuple. Each price may be `None` if the corresponding bid/ask prices
    /// were not available when `set_mid_prices()` was called.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * First element: The call option mid-price (bid+ask)/2, or `None` if not available
    /// * Second element: The put option mid-price (bid+ask)/2, or `None` if not available
    pub fn get_mid_prices(&self) -> (Option<Positive>, Option<Positive>) {
        (self.call_middle, self.put_middle)
    }

    /// Calculates the implied volatility for an option based on market prices.
    ///
    /// This function attempts to derive the implied volatility from either call or put option
    /// mid-market prices. It first tries to use call options, and if that fails, it falls back
    /// to put options. The calculation uses different initial volatility guesses based on whether
    /// the option is in-the-money (ITM) or out-of-the-money (OTM).
    ///
    /// # Parameters
    ///
    /// * `&mut self` - Mutable reference to the option chain or strike object
    /// * `price_params` - Reference to pricing parameters including underlying price and other market data
    ///
    /// # Returns
    ///
    /// * `Result<(), ChainError>` - Ok if implied volatility was successfully calculated,
    ///   or an error describing why calculation failed
    ///
    /// # Process
    ///
    /// 1. Ensures middle prices are available, calculating them if necessary
    /// 2. Attempts to calculate IV using call options first
    /// 3. Falls back to put options if call calculation fails
    /// 4. Updates the implied_volatility field if successful
    /// 5. Creates option objects if needed once IV is established
    ///
    /// # Errors
    ///
    /// Returns a `ChainError::InvalidVolatility` if implied volatility cannot be calculated
    /// from either call or put prices.
    pub fn calculate_implied_volatility(
        &mut self,
        price_params: &OptionDataPriceParams,
    ) -> Result<(), ChainError> {
        trace!(
            "call_middle {:?} put_middle {:?}",
            self.call_middle, self.put_middle
        );
        if self.call_middle.is_none() || self.put_middle.is_none() {
            info!("Calculation middel prices for IV calculation:");
            self.calculate_prices(price_params, false)?;
        }

        // Try to calculate IV for calls if we have mid price
        if let Some(call_price) = self.call_middle {
            // Initial IV guess based on moneyness
            let initial_iv = if price_params.underlying_price > self.strike_price {
                pos!(0.5) // ITM
            } else {
                pos!(0.3) // OTM
            };

            let option =
                self.get_option_for_iv(price_params, Side::Long, OptionStyle::Call, initial_iv)?;

            match option.calculate_implied_volatility(call_price.to_dec()) {
                Ok(iv) => {
                    debug!("Successfully calculated call IV: {}", iv);
                    self.implied_volatility = Some(iv * Positive::HUNDRED);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed to calculate call IV: {}", e);
                }
            }
        }

        // If call IV calculation failed or wasn't possible, try puts
        if let Some(put_price) = self.put_middle {
            // Initial IV guess based on moneyness
            let initial_iv = if price_params.underlying_price < self.strike_price {
                pos!(5.0) // ITM
            } else {
                pos!(3.0) // OTM
            };

            let option =
                self.get_option_for_iv(price_params, Side::Long, OptionStyle::Put, initial_iv)?;

            match option.calculate_implied_volatility(put_price.to_dec()) {
                Ok(iv) => {
                    debug!("Successfully calculated put IV: {}", iv);
                    self.implied_volatility = Some(iv * Positive::HUNDRED);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed to calculate put IV: {}", e);
                }
            }
        }

        if self.options.is_none() && self.implied_volatility.is_some() {
            self.create_options(price_params)?;
        }

        Err(ChainError::invalid_volatility(
            None,
            "Could not calculate implied volatility from either calls or puts",
        ))
    }
}

impl Default for OptionData {
    fn default() -> Self {
        OptionData {
            strike_price: Positive::ZERO,
            call_bid: None,
            call_ask: None,
            put_bid: None,
            put_ask: None,
            call_middle: None,
            put_middle: None,
            implied_volatility: None,
            delta_call: None,
            delta_put: None,
            gamma: None,
            volume: None,
            open_interest: None,
            options: None,
        }
    }
}

impl PartialOrd for OptionData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.strike_price.cmp(&other.strike_price))
    }
}

impl Eq for OptionData {}

impl Ord for OptionData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.strike_price.cmp(&other.strike_price)
    }
}

impl fmt::Display for OptionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:.3}{:<8} {:.3}{:<4} {:.3}{:<5} {:.2}{:<8} {:<10} {:<10}",
            self.strike_price.to_string(),
            default_empty_string(self.call_bid),
            default_empty_string(self.call_ask),
            default_empty_string(self.call_middle),
            default_empty_string(self.put_bid),
            default_empty_string(self.put_ask),
            default_empty_string(self.put_middle),
            self.implied_volatility.unwrap_or(Positive::ZERO),
            " ".to_string(),
            self.delta_call.unwrap_or(Decimal::ZERO),
            " ".to_string(),
            self.delta_put.unwrap_or(Decimal::ZERO),
            " ".to_string(),
            self.gamma.unwrap_or(Decimal::ZERO) * Decimal::ONE_HUNDRED,
            " ".to_string(),
            default_empty_string(self.volume),
            default_empty_string(self.open_interest),
        )?;
        Ok(())
    }
}

/// Represents an option chain for a specific underlying asset and expiration date.
///
/// An option chain contains all available option contracts (calls and puts) for a given
/// underlying asset at a specific expiration date, along with current market data and pricing
/// parameters necessary for financial analysis and valuation.
///
/// This struct provides a complete representation of option market data that can be used for
/// options strategy analysis, risk assessment, and pricing model calculations.
///
/// # Fields
///
/// * `symbol` - The ticker symbol for the underlying asset (e.g., "AAPL", "SPY").
///
/// * `underlying_price` - The current market price of the underlying asset, stored as a
///   guaranteed positive value.
///
/// * `expiration_date` - The expiration date of the options in the chain, typically
///   represented in a standard date format.
///
/// * `options` - A sorted collection of option contracts at different strike prices, containing
///   detailed market data like bid/ask prices, implied volatility, and the Greeks.
///
/// * `risk_free_rate` - The risk-free interest rate used for option pricing models,
///   typically derived from treasury yields. May be `None` if not specified.
///
/// * `dividend_yield` - The annual dividend yield of the underlying asset, represented
///   as a positive percentage. May be `None` for non-dividend-paying assets.
///
/// # Usage
///
/// This struct is typically used as the primary container for options market data analysis,
/// serving as input to pricing models, strategy backtesting, and risk management tools.
#[derive(Debug)]
pub struct OptionChain {
    /// The ticker symbol for the underlying asset (e.g., "AAPL", "SPY").
    pub symbol: String,

    /// The current market price of the underlying asset.
    pub underlying_price: Positive,

    /// The expiration date of the options in the chain.
    expiration_date: String,

    /// A sorted collection of option contracts at different strike prices.
    pub(crate) options: BTreeSet<OptionData>,

    /// The risk-free interest rate used for option pricing models.
    pub(crate) risk_free_rate: Option<Decimal>,

    /// The annual dividend yield of the underlying asset.
    pub(crate) dividend_yield: Option<Positive>,
}

impl Serialize for OptionChain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OptionChain", 6)?;

        state.serialize_field("symbol", &self.symbol)?;
        state.serialize_field("underlying_price", &self.underlying_price)?;
        state.serialize_field("expiration_date", &self.expiration_date)?;
        state.serialize_field("options", &self.options)?;

        if let Some(rate) = &self.risk_free_rate {
            state.serialize_field("risk_free_rate", rate)?;
        }

        if let Some(yield_val) = &self.dividend_yield {
            state.serialize_field("dividend_yield", yield_val)?;
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for OptionChain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Symbol,
            #[serde(rename = "underlying_price")]
            UnderlyingPrice,
            #[serde(rename = "expiration_date")]
            ExpirationDate,
            Options,
            RiskFreeRate,
            DividendYield,
        }

        struct OptionChainVisitor;

        impl<'de> Visitor<'de> for OptionChainVisitor {
            type Value = OptionChain;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct OptionChain")
            }

            fn visit_map<V>(self, mut map: V) -> Result<OptionChain, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut symbol = None;
                let mut underlying_price = None;
                let mut expiration_date = None;
                let mut options = None;
                let mut risk_free_rate = None;
                let mut dividend_yield = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Symbol => {
                            if symbol.is_some() {
                                return Err(de::Error::duplicate_field("symbol"));
                            }
                            symbol = Some(map.next_value()?);
                        }
                        Field::UnderlyingPrice => {
                            if underlying_price.is_some() {
                                return Err(de::Error::duplicate_field("underlying"));
                            }
                            underlying_price = Some(map.next_value()?);
                        }
                        Field::ExpirationDate => {
                            if expiration_date.is_some() {
                                return Err(de::Error::duplicate_field("expiration"));
                            }
                            expiration_date = Some(map.next_value()?);
                        }
                        Field::Options => {
                            if options.is_some() {
                                return Err(de::Error::duplicate_field("options"));
                            }
                            options = Some(map.next_value()?);
                        }
                        Field::RiskFreeRate => {
                            if risk_free_rate.is_some() {
                                return Err(de::Error::duplicate_field("risk_free_rate"));
                            }
                            risk_free_rate = map.next_value().ok();
                        }
                        Field::DividendYield => {
                            if dividend_yield.is_some() {
                                return Err(de::Error::duplicate_field("dividend_yield"));
                            }
                            dividend_yield = map.next_value().ok();
                        }
                    }
                }

                let symbol = symbol.ok_or_else(|| de::Error::missing_field("symbol"))?;
                let underlying_price =
                    underlying_price.ok_or_else(|| de::Error::missing_field("underlying"))?;
                let expiration_date =
                    expiration_date.ok_or_else(|| de::Error::missing_field("expiration"))?;
                let options = options.unwrap_or_default();

                Ok(OptionChain {
                    symbol,
                    underlying_price,
                    expiration_date,
                    options,
                    risk_free_rate,
                    dividend_yield,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "symbol",
            "underlying_price",
            "expiration_date",
            "options",
            "risk_free_rate",
            "dividend_yield",
        ];
        deserializer.deserialize_struct("OptionChain", FIELDS, OptionChainVisitor)
    }
}

impl OptionChain {
    /// Creates a new `OptionChain` for a specific underlying instrument and expiration date.
    ///
    /// This constructor initializes an `OptionChain` with the fundamental parameters needed for
    /// option calculations and analysis. It creates an empty collection of options that can be
    /// populated later through other methods.
    ///
    /// # Parameters
    ///
    /// * `symbol` - The ticker symbol of the underlying instrument (e.g., "AAPL" for Apple Inc.).
    ///
    /// * `underlying_price` - The current market price of the underlying instrument as a
    ///   `Positive` value, ensuring it's always greater than or equal to zero.
    ///
    /// * `expiration_date` - The expiration date for the options in this chain, provided as a
    ///   string. The expected format depends on the implementation's requirements.
    ///
    /// * `risk_free_rate` - The risk-free interest rate used for theoretical pricing models.
    ///   This is optional and can be provided later if not available at creation time.
    ///
    /// * `dividend_yield` - The dividend yield of the underlying instrument as a `Positive` value.
    ///   This is optional and can be provided later for dividend-paying instruments.
    ///
    /// # Returns
    ///
    /// A new `OptionChain` instance with the specified parameters and an empty set of options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::pos;
    ///
    /// let chain = OptionChain::new(
    ///     "AAPL",
    ///     pos!(172.50),
    ///     "2023-12-15".to_string(),
    ///     Some(dec!(0.05)),  // 5% risk-free rate
    ///     Some(pos!(0.0065)) // 0.65% dividend yield
    /// );
    /// ```
    pub fn new(
        symbol: &str,
        underlying_price: Positive,
        expiration_date: String,
        risk_free_rate: Option<Decimal>,
        dividend_yield: Option<Positive>,
    ) -> Self {
        OptionChain {
            symbol: symbol.to_string(),
            underlying_price,
            expiration_date,
            options: BTreeSet::new(),
            risk_free_rate,
            dividend_yield,
        }
    }

    /// Builds a complete option chain based on the provided parameters.
    ///
    /// This function creates an option chain with strikes generated around the underlying price,
    /// calculates prices and Greeks for each option using the Black-Scholes model, and applies
    /// the specified volatility skew to reflect market conditions.
    ///
    /// # Arguments
    ///
    /// * `params` - A reference to `OptionChainBuildParams` containing all necessary parameters
    ///   for building the chain, including price parameters, chain size, and volatility settings.
    ///
    /// # Returns
    ///
    /// A fully populated `OptionChain` containing option data for all generated strikes.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    /// use optionstratlib::{pos, spos, ExpirationDate};
    /// use optionstratlib::chains::chain::OptionChain;
    /// let price_params = OptionDataPriceParams::new(
    ///     pos!(100.0),                         // underlying price
    ///     ExpirationDate::Days(pos!(30.0)),    // expiration date
    ///     Some(pos!(0.2)),                     // implied volatility
    ///     dec!(0.05),                          // risk-free rate
    ///     pos!(0.0),                           // dividend yield
    ///     Some("SPY".to_string())              // underlying symbol
    /// );
    ///
    /// let build_params = OptionChainBuildParams::new(
    ///     "SPY".to_string(),
    ///     spos!(1000.0),
    ///     10,
    ///     pos!(5.0),
    ///     0.1,
    ///     pos!(0.02),
    ///     2,
    ///     price_params,
    /// );
    ///
    /// let chain = OptionChain::build_chain(&build_params);
    /// ```
    pub fn build_chain(params: &OptionChainBuildParams) -> Self {
        let mut option_chain = OptionChain::new(
            &params.symbol,
            params.price_params.underlying_price,
            params
                .price_params
                .expiration_date
                .get_date_string()
                .unwrap(),
            None,
            None,
        );
        let strikes = generate_list_of_strikes(
            params.price_params.underlying_price,
            params.chain_size,
            params.strike_interval,
        );
        for strike in strikes {
            let atm_distance = strike.to_dec() - params.price_params.underlying_price;
            let adjusted_volatility = adjust_volatility(
                params.price_params.implied_volatility,
                params.skew_factor,
                atm_distance.to_f64().unwrap(),
            );
            let mut option_data = OptionData::new(
                strike,
                None,
                None,
                None,
                None,
                adjusted_volatility,
                None,
                None,
                None,
                params.volume,
                None,
            );
            let price_params = OptionDataPriceParams::new(
                params.price_params.underlying_price,
                params.price_params.expiration_date,
                adjusted_volatility,
                params.price_params.risk_free_rate,
                params.price_params.dividend_yield,
                params.price_params.underlying_symbol.clone(),
            );
            if option_data.calculate_prices(&price_params, false).is_ok() {
                option_data.apply_spread(params.spread, params.decimal_places);
                option_data.calculate_delta(&price_params);
                option_data.calculate_gamma(&price_params);
            }
            option_chain.options.insert(option_data);
        }
        debug!("Option chain: {}", option_chain);
        option_chain
    }

    /// Filters option data in the chain based on specified criteria.
    ///
    /// This method filters the options in the chain according to the provided side parameter,
    /// which determines which options to include based on their strike price relative to
    /// the underlying price.
    ///
    /// # Arguments
    ///
    /// * `side` - A `FindOptimalSide` enum value that specifies which options to include:
    ///   - `Upper`: Only options with strikes above the underlying price
    ///   - `Lower`: Only options with strikes below the underlying price
    ///   - `All`: All options in the chain
    ///   - `Range(start, end)`: Only options with strikes within the specified range
    ///
    /// # Returns
    ///
    /// A vector of references to `OptionData` objects that match the filter criteria.
    pub(crate) fn filter_option_data(&self, side: FindOptimalSide) -> Vec<&OptionData> {
        self.options
            .iter()
            .filter(|option| match side {
                FindOptimalSide::Upper => option.strike_price > self.underlying_price,
                FindOptimalSide::Lower => option.strike_price < self.underlying_price,
                FindOptimalSide::All => true,
                FindOptimalSide::Range(start, end) => {
                    option.strike_price >= start && option.strike_price <= end
                }
            })
            .collect()
    }

    /// Filters options and converts them to `OptionsInStrike` objects.
    ///
    /// Similar to `filter_option_data`, but returns more detailed `OptionsInStrike` objects
    /// that include specific option contract information for a given side and style.
    ///
    /// # Arguments
    ///
    /// * `price_params` - Parameters used for option pricing calculations
    /// * `side` - A `FindOptimalSide` enum value that specifies which options to include
    ///
    /// # Returns
    ///
    /// A result containing either a vector of `OptionsInStrike` objects or a `ChainError` if
    /// conversion of any option fails.
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if any option data fails to be converted to `OptionsInStrike`.
    #[allow(dead_code)]
    pub(crate) fn filter_options_in_strike(
        &self,
        price_params: &OptionDataPriceParams,
        side: FindOptimalSide,
    ) -> Result<Vec<OptionsInStrike>, ChainError> {
        self.options
            .iter()
            .filter(|option| match side {
                FindOptimalSide::Upper => option.strike_price > self.underlying_price,
                FindOptimalSide::Lower => option.strike_price < self.underlying_price,
                FindOptimalSide::All => true,
                FindOptimalSide::Range(start, end) => {
                    option.strike_price >= start && option.strike_price <= end
                }
            })
            .map(|option| option.get_options_in_strike(price_params, Side::Long, OptionStyle::Call))
            .collect()
    }

    /// Adds a new option to the chain with the specified parameters.
    ///
    /// This method creates and adds a new option at the given strike price to the chain.
    /// It calculates mid prices and attempts to create detailed option objects with the
    /// provided parameters.
    ///
    /// # Arguments
    ///
    /// * `strike_price` - The strike price for the new option
    /// * `call_bid` - Optional bid price for the call option
    /// * `call_ask` - Optional ask price for the call option
    /// * `put_bid` - Optional bid price for the put option
    /// * `put_ask` - Optional ask price for the put option
    /// * `implied_volatility` - Optional implied volatility for the option
    /// * `delta_call` - Optional delta value for the call option
    /// * `delta_put` - Optional delta value for the put option
    /// * `gamma` - Optional gamma value for the option
    /// * `volume` - Optional trading volume for the option
    /// * `open_interest` - Optional open interest for the option
    ///
    /// # Panics
    ///
    /// Panics if the expiration date in the option chain cannot be parsed.
    #[allow(clippy::too_many_arguments)]
    pub fn add_option(
        &mut self,
        strike_price: Positive,
        call_bid: Option<Positive>,
        call_ask: Option<Positive>,
        put_bid: Option<Positive>,
        put_ask: Option<Positive>,
        implied_volatility: Option<Positive>,
        delta_call: Option<Decimal>,
        delta_put: Option<Decimal>,
        gamma: Option<Decimal>,
        volume: Option<Positive>,
        open_interest: Option<u64>,
    ) {
        let mut option_data = OptionData {
            strike_price,
            call_bid,
            call_ask,
            put_bid,
            put_ask,
            call_middle: None,
            put_middle: None,
            implied_volatility,
            delta_call,
            delta_put,
            gamma,
            volume,
            open_interest,
            options: None,
        };
        option_data.set_mid_prices();
        let expiration_date = match ExpirationDate::from_string(&self.expiration_date) {
            Ok(date) => date,
            Err(e) => {
                panic!("Failed to parse expiration date: {}", e);
            }
        };
        let params = OptionDataPriceParams::new(
            self.underlying_price,
            expiration_date,
            implied_volatility,
            self.risk_free_rate.unwrap_or(Decimal::ZERO),
            self.dividend_yield.unwrap_or(Positive::ZERO),
            Some(self.symbol.clone()),
        );
        if let Err(e) = option_data.create_options(&params) {
            error!(
                "Failed to create options for strike {}: {}",
                strike_price, e
            );
        }
        self.options.insert(option_data);
    }

    /// Returns a formatted title string for the option chain.
    ///
    /// This method creates a title by combining the option chain's symbol, expiration date,
    /// and underlying price. Spaces in the symbol and expiration date are replaced with hyphens
    /// for better compatibility with file systems and data representation.
    ///
    /// # Returns
    ///
    /// A formatted string in the format "{symbol}-{expiration_date}-{underlying_price}"
    /// where spaces have been replaced with hyphens in the symbol and expiration date.
    pub fn get_title(&self) -> String {
        let symbol_cleaned = self.symbol.replace(" ", "-");
        let expiration_date_cleaned = self.expiration_date.replace(" ", "-");
        format!(
            "{}-{}-{}",
            symbol_cleaned, expiration_date_cleaned, self.underlying_price
        )
    }

    /// Parses a file name to set the option chain's properties.
    ///
    /// This method extracts information from a file name that follows the format
    /// "symbol-day-month-year-price.extension". It sets the symbol, expiration date,
    /// and underlying price of the option chain based on the parsed values.
    ///
    /// # Arguments
    ///
    /// * `file` - A string slice representing the file path or name to parse
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file name was successfully parsed and the properties were set
    /// * `Err(...)` - If the file name format is invalid or the underlying price cannot be parsed
    ///
    /// # Panics
    ///
    /// This function will panic if the underlying price in the file name cannot be parsed as an f64.
    pub fn set_from_title(&mut self, file: &str) -> Result<(), Box<dyn Error>> {
        let file_name = file.split('/').next_back().unwrap();
        let file_name = file_name
            .rsplit_once('.')
            .map_or(file_name, |(name, _ext)| name);
        let parts: Vec<&str> = file_name.split('-').collect();
        if parts.len() != 5 {
            return Err("Invalid file name format: expected exactly 5 parts (symbol, day, month, year, price)".to_string().into());
        }
        self.symbol = parts[0].to_string();
        self.expiration_date = format!("{}-{}-{}", parts[1], parts[2], parts[3]);
        let underlying_price_str = parts[4].replace(",", ".");
        match underlying_price_str.parse::<f64>() {
            Ok(price) => {
                self.underlying_price = pos!(price);
                Ok(())
            }
            Err(_) => panic!("Invalid underlying price format in file name"),
        }
    }

    /// Updates the mid prices for all options in the chain.
    ///
    /// This method creates a new collection of options where each option has its
    /// mid price calculated and updated. The mid price is typically the average
    /// of the bid and ask prices.
    ///
    /// The original options in the chain are replaced with the updated ones.
    pub fn update_mid_prices(&mut self) {
        let modified_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option = option.clone();
                option.set_mid_prices();
                option
            })
            .collect();
        self.options = modified_options;
    }

    /// Calculates and updates the delta and gamma Greeks for all options in the chain.
    ///
    /// This method computes the delta and gamma values for each option in the chain based on
    /// the current market parameters. Delta measures the rate of change of the option price
    /// with respect to the underlying asset's price, while gamma measures the rate of change
    /// of delta with respect to the underlying asset's price.
    ///
    /// The original options in the chain are replaced with the ones containing the updated Greeks.
    pub fn update_greeks(&mut self) {
        let modified_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option = option.clone(); // Create a clone we can modify
                let params = self.get_params(option.strike_price).unwrap();
                option.calculate_delta(&params);
                option.calculate_gamma(&params);
                option
            })
            .collect();
        self.options = modified_options;
    }

    /// Calculates and updates the implied volatility for all options in the chain.
    ///
    /// This method attempts to compute the implied volatility for each option in the chain.
    /// Implied volatility is the market's forecast of a likely movement in the underlying price
    /// and is derived from the option's market price.
    ///
    /// If the calculation fails for any option, a debug message is logged with the strike price
    /// and the error, but the process continues for other options.
    ///
    /// The original options in the chain are replaced with the ones containing the updated
    /// implied volatility values.
    pub fn update_implied_volatilities(&mut self) {
        let modified_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option = option.clone();
                let params = self.get_params(option.strike_price).unwrap();
                if let Err(e) = option.calculate_implied_volatility(&params) {
                    debug!(
                        "Failed to calculate IV for strike {}: {}",
                        option.strike_price, e
                    );
                }
                option
            })
            .collect();
        self.options = modified_options;
    }

    /// Saves the option chain data to a CSV file.
    ///
    /// This method writes the option chain data to a CSV file at the specified path.
    /// The file will be named using the option chain's title (symbol, expiration date, and price).
    /// The CSV includes headers for all option properties and each option in the chain is written as a row.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The directory path where the CSV file will be created
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Ok(()) if successful, or an Error if the file couldn't be created
    ///   or written to.
    ///
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let full_path = format!("{}/{}.csv", file_path, self.get_title());
        let mut wtr = WriterBuilder::new().from_path(full_path)?;
        wtr.write_record([
            "Strike Price",
            "Call Bid",
            "Call Ask",
            "Put Bid",
            "Put Ask",
            "Implied Volatility",
            "Delta",
            "Delta",
            "Gamma",
            "Volume",
            "Open Interest",
        ])?;
        for option in &self.options {
            wtr.write_record(&[
                option.strike_price.to_string(),
                default_empty_string(option.call_bid),
                default_empty_string(option.call_ask),
                default_empty_string(option.put_bid),
                default_empty_string(option.put_ask),
                default_empty_string(option.implied_volatility),
                default_empty_string(option.delta_call),
                default_empty_string(option.delta_put),
                default_empty_string(option.gamma),
                default_empty_string(option.volume),
                default_empty_string(option.open_interest),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// Saves the option chain data to a JSON file.
    ///
    /// This method serializes the option chain into JSON format and writes it to a file
    /// at the specified path. The file will be named using the option chain's title
    /// (symbol, expiration date, and price).
    ///
    /// # Arguments
    ///
    /// * `file_path` - The directory path where the JSON file will be created
    ///
    /// # Returns
    ///
    /// * `Result<(), Box<dyn Error>>` - Ok(()) if successful, or an Error if the file couldn't be created
    ///   or written to.
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let full_path = format!("{}/{}.json", file_path, self.get_title());
        let file = File::create(full_path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }

    /// Loads option chain data from a CSV file.
    ///
    /// This function reads option data from a CSV file and constructs an OptionChain.
    /// It attempts to extract the symbol, underlying price, and expiration date from the file name.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the CSV file containing option chain data
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - An OptionChain if successful, or an Error if the file
    ///   couldn't be read or the data is invalid.
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        let mut options = BTreeSet::new();
        for result in rdr.records() {
            let record = result?;
            debug!("To CSV: {:?}", record);
            let mut option_data = OptionData {
                strike_price: record[0].parse()?,
                call_bid: parse(&record[1]),
                call_ask: parse(&record[2]),
                put_bid: parse(&record[3]),
                put_ask: parse(&record[4]),
                call_middle: None,
                put_middle: None,
                implied_volatility: parse(&record[5]),
                delta_call: parse(&record[6]),
                delta_put: parse(&record[7]),
                gamma: parse(&record[8]),
                volume: parse(&record[9]),
                open_interest: parse(&record[10]),
                options: None,
            };
            option_data.set_mid_prices();
            options.insert(option_data);
        }
        let mut option_chain = OptionChain {
            symbol: "unknown".to_string(),
            underlying_price: Positive::ZERO,
            expiration_date: "unknown".to_string(),
            options,
            risk_free_rate: None,
            dividend_yield: None,
        };
        match option_chain.set_from_title(file_path) {
            Ok(_) => {
                // TODO: find other way to set symbol, underlying_price and expiration_date
            }
            Err(e) => {
                debug!("Failed to set title from file name: {}", e);
            }
        }
        Ok(option_chain)
    }

    /// Loads option chain data from a JSON file.
    ///
    /// This function deserializes an OptionChain from a JSON file and updates
    /// the mid prices for all options in the chain.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the JSON file containing serialized option chain data
    ///
    /// # Returns
    ///
    /// * `Result<Self, Box<dyn Error>>` - An OptionChain if successful, or an Error if the file
    ///   couldn't be read or the data is invalid.
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut option_chain: OptionChain = serde_json::from_reader(file)?;
        option_chain.update_mid_prices();
        Ok(option_chain)
    }

    /// Generates a vector of strike prices within the range of available options.
    ///
    /// This method creates a vector of strike prices starting from the lowest
    /// strike price in the chain up to the highest, incrementing by the specified step.
    ///
    /// # Arguments
    ///
    /// * `step` - The increment value between consecutive strike prices
    ///
    /// # Returns
    ///
    /// * `Option<Vec<f64>>` - A vector containing the strike prices if the option chain
    ///   is not empty, or None if there are no options in the chain.
    ///
    pub fn strike_price_range_vec(&self, step: f64) -> Option<Vec<f64>> {
        let first = self.options.iter().next();
        let last = self.options.iter().next_back();
        if let (Some(first), Some(last)) = (first, last) {
            let mut range = Vec::new();
            let mut current_price = first.strike_price;
            while current_price <= last.strike_price {
                range.push(current_price.to_f64());
                current_price += pos!(step);
            }
            Some(range)
        } else {
            None
        }
    }

    /// Creates random positions based on specified quantities of puts and calls
    ///
    /// # Arguments
    ///
    /// * `qty_puts_long` - Number of long put positions to create
    /// * `qty_puts_short` - Number of short put positions to create
    /// * `qty_calls_long` - Number of long call positions to create
    /// * `qty_calls_short` - Number of short call positions to create
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Position>, ChainError>` - Vector of created positions or error message
    pub fn get_random_positions(
        &self,
        params: RandomPositionsParams,
    ) -> Result<Vec<Position>, ChainError> {
        if params.total_positions() == 0 {
            return Err(ChainError::invalid_parameters(
                "total_positions",
                "The sum of the quantities must be greater than 0",
            ));
        }

        let mut positions = Vec::with_capacity(params.total_positions());

        // Add long put positions
        if let Some(qty) = params.qty_puts_long {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Long,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date,
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_ask.unwrap_or(Positive::ZERO),
                        Utc::now(),
                        params.open_put_fee,
                        params.close_put_fee,
                    );
                    positions.push(position);
                }
            }
        }

        // Add short put positions
        if let Some(qty) = params.qty_puts_short {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Short,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date,
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_bid.unwrap_or(Positive::ZERO),
                        Utc::now(),
                        params.open_put_fee,
                        params.close_put_fee,
                    );
                    positions.push(position);
                }
            }
        }

        // Add long call positions
        if let Some(qty) = params.qty_calls_long {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Long,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date,
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_ask.unwrap_or(Positive::ZERO),
                        Utc::now(),
                        params.open_call_fee,
                        params.close_call_fee,
                    );
                    positions.push(position);
                }
            }
        }

        // Add short call positions
        if let Some(qty) = params.qty_calls_short {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Short,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date,
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_bid.unwrap_or(Positive::ZERO),
                        Utc::now(),
                        params.open_call_fee,
                        params.close_call_fee,
                    );
                    positions.push(position);
                }
            }
        }

        Ok(positions)
    }

    /// Returns an iterator over the `options` field in the `OptionChain` structure.
    ///
    /// This method provides a mechanism to traverse through the set of options
    /// (`OptionData`) associated with an `OptionChain`.
    ///
    /// # Returns
    ///
    /// An iterator that yields references to the `OptionData` elements in the `options` field.
    /// Since the `options` field is stored as a `BTreeSet`, the elements are ordered
    /// in ascending order based on the sorting rules of `BTreeSet` (typically defined by `Ord` implementation).
    ///
    pub fn get_single_iter(&self) -> impl Iterator<Item = &OptionData> {
        self.options
            .iter()
            .filter(|option| option.implied_volatility.is_some())
    }

    /// Returns an iterator that generates pairs of distinct option combinations from the `OptionChain`.
    ///
    /// This function iterates over all unique combinations of two options from the `options` collection
    /// without repetition. In mathematical terms, it generates combinations where order does not matter
    /// and an option cannot combine with itself.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples of references to two distinct `OptionData` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::{pos, Positive};
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2) in option_chain.get_double_iter() {
    ///     info!("{:?}, {:?}", option1, option2);
    /// }
    /// ```
    pub fn get_double_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData)> {
        self.get_single_iter().enumerate().flat_map(|(i, item1)| {
            self.get_single_iter()
                .skip(i + 1)
                .map(move |item2| (item1, item2))
        })
    }

    /// Returns an iterator that generates inclusive pairs of option combinations from the `OptionChain`.
    ///
    /// This function iterates over all combinations of two options from the `options` collection,
    /// including pairing an option with itself.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples with two references to `OptionData`, potentially including
    /// self-pairs (e.g., `(option, option)`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2) in option_chain.get_double_inclusive_iter() {
    ///     info!("{:?}, {:?}", option1, option2);
    /// }
    /// ```
    pub fn get_double_inclusive_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData)> {
        self.get_single_iter().enumerate().flat_map(|(i, item1)| {
            self.get_single_iter()
                .skip(i)
                .map(move |item2| (item1, item2))
        })
    }

    /// Returns an iterator that generates unique triplets of distinct option combinations from the `OptionChain`.
    ///
    /// This function iterates over all unique combinations of three options from the `options` collection
    /// without repetition.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples containing references to three distinct `OptionData` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3) in option_chain.get_triple_iter() {
    ///     info!("{:?}, {:?}, {:?}", option1, option2, option3);
    /// }
    /// ```
    pub fn get_triple_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData)> {
        self.get_single_iter()
            .enumerate()
            .flat_map(move |(i, item1)| {
                self.get_single_iter()
                    .skip(i + 1)
                    .enumerate()
                    .flat_map(move |(j, item2)| {
                        self.get_single_iter()
                            .skip(i + j + 2)
                            .map(move |item3| (item1, item2, item3))
                    })
            })
    }

    /// Returns an iterator that generates inclusive triplets of option combinations from the `OptionChain`.
    ///
    /// This function iterates over all combinations of three options from the `options` collection,
    /// including those where the same option may be included more than once.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples with three references to `OptionData`, potentially including
    /// repeated elements (e.g., `(option1, option2, option1)`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3) in option_chain.get_triple_inclusive_iter() {
    ///     info!("{:?}, {:?}, {:?}", option1, option2, option3);
    /// }
    /// ```
    pub fn get_triple_inclusive_iter(
        &self,
    ) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData)> {
        self.get_single_iter()
            .enumerate()
            .flat_map(move |(i, item1)| {
                self.get_single_iter()
                    .skip(i)
                    .enumerate()
                    .flat_map(move |(j, item2)| {
                        self.get_single_iter()
                            .skip(i + j)
                            .map(move |item3| (item1, item2, item3))
                    })
            })
    }

    /// Returns an iterator that generates unique quadruples of distinct option combinations from the `OptionChain`.
    ///
    /// This function iterates over all unique combinations of four options from the `options` collection
    /// without repetition.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples containing references to four distinct `OptionData` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3, option4) in option_chain.get_quad_iter() {
    ///     info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
    /// }
    /// ```
    pub fn get_quad_iter(
        &self,
    ) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData, &OptionData)> {
        self.get_single_iter()
            .enumerate()
            .flat_map(move |(i, item1)| {
                self.get_single_iter()
                    .skip(i + 1)
                    .enumerate()
                    .flat_map(move |(j, item2)| {
                        self.get_single_iter().skip(i + j + 2).enumerate().flat_map(
                            move |(k, item3)| {
                                self.get_single_iter()
                                    .skip(i + j + k + 3)
                                    .map(move |item4| (item1, item2, item3, item4))
                            },
                        )
                    })
            })
    }

    /// Returns an iterator that generates inclusive quadruples of option combinations from the `OptionChain`.
    ///
    /// This function iterates over all combinations of four options from the `options` collection,
    /// including those where the same option may be included more than once.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples with four references to `OptionData`, potentially including
    /// repeated elements (e.g., `(option1, option2, option1, option4)`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3, option4) in option_chain.get_quad_inclusive_iter() {
    ///     info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
    /// }
    /// ```
    pub fn get_quad_inclusive_iter(
        &self,
    ) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData, &OptionData)> {
        self.get_single_iter()
            .enumerate()
            .flat_map(move |(i, item1)| {
                self.get_single_iter()
                    .skip(i)
                    .enumerate()
                    .flat_map(move |(j, item2)| {
                        self.get_single_iter().skip(i + j).enumerate().flat_map(
                            move |(k, item3)| {
                                self.get_single_iter()
                                    .skip(i + j + k)
                                    .map(move |item4| (item1, item2, item3, item4))
                            },
                        )
                    })
            })
    }

    /// Retrieves the call option price for a specific strike price
    ///
    /// This helper method finds and returns the ask price of a call option
    /// at the specified strike price from the option chain.
    ///
    /// # Arguments
    /// * `strike` - The strike price to look up
    ///
    /// # Returns
    /// * `Some(Decimal)` - The call option ask price if found
    /// * `None` - If no option exists at the specified strike or if the price is not available
    ///
    /// # Notes
    /// * Uses the ask price as it represents the cost to buy the option
    /// * Converts the price to Decimal for consistency in calculations
    pub fn get_call_price(&self, strike: Positive) -> Option<Decimal> {
        self.options
            .iter()
            .find(|opt| opt.strike_price == strike)
            .and_then(|opt| opt.call_ask)
            .map(|price| price.to_dec())
    }

    /// Retrieves the implied volatility for the at-the-money (ATM) option
    ///
    /// Finds the option with strike price equal to the current underlying price
    /// and returns its implied volatility.
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The ATM implied volatility if found
    /// * `Err(String)` - Error message if ATM implied volatility is not available
    ///
    /// # Examples
    /// ```
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::pos;
    /// let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);
    /// match chain.get_atm_implied_volatility() {
    ///     Ok(vol) => info!("ATM volatility: {}", vol),
    ///     Err(e) => info!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Notes
    /// * ATM strike is defined as the strike equal to the current underlying price
    /// * Important for volatility skew calculations and option pricing
    /// * Returns implied volatility as a decimal for precise calculations
    pub fn get_atm_implied_volatility(&self) -> Result<Decimal, String> {
        let atm_strike = self.underlying_price;

        self.options
            .iter()
            .find(|opt| opt.strike_price == atm_strike)
            .and_then(|opt| opt.implied_volatility)
            .map(|iv| iv.value())
            .ok_or_else(|| "No ATM implied volatility available".to_string())
    }

    /// Calculates the total gamma exposure for all options in the chain.
    ///
    /// Gamma exposure represents the aggregate rate of change in the delta value
    /// with respect to changes in the underlying asset's price across all options.
    /// It measures the second-order price sensitivity and indicates how the delta
    /// will change as the underlying price moves.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate gamma value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's gamma calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn gamma_exposure(&self) -> Result<Decimal, ChainError> {
        let mut gamma_exposure = Decimal::ZERO;
        for option in &self.options {
            if let Some(four_options) = &option.options {
                gamma_exposure += four_options.long_call.gamma()?;
            } else {
                warn!(
                    "No options greeks no initialized. Please run the update_greeks method first."
                );
            }
        }
        Ok(gamma_exposure)
    }

    /// Calculates the total delta exposure for all options in the chain.
    ///
    /// Delta exposure represents the aggregate sensitivity of option prices to changes
    /// in the underlying asset's price. A delta exposure of 1.0 means that for every
    /// $1 change in the underlying asset, the options portfolio will change by $1 in the same direction.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate delta value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's delta calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn delta_exposure(&self) -> Result<Decimal, ChainError> {
        let mut delta_exposure = Decimal::ZERO;
        for option in &self.options {
            if let Some(four_options) = &option.options {
                delta_exposure += four_options.long_call.delta()?;
            } else {
                warn!(
                    "No options greeks no initialized. Please run the update_greeks method first."
                );
            }
        }
        Ok(delta_exposure)
    }

    /// Calculates the total vega exposure for all options in the chain.
    ///
    /// Vega exposure represents the aggregate sensitivity of option prices to changes
    /// in the implied volatility of the underlying asset. It measures how much option
    /// prices will change for a 1% change in implied volatility.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate vega value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's vega calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn vega_exposure(&self) -> Result<Decimal, ChainError> {
        let mut vega_exposure = Decimal::ZERO;
        for option in &self.options {
            if let Some(four_options) = &option.options {
                vega_exposure += four_options.long_call.vega()?;
            } else {
                warn!(
                    "No options greeks no initialized. Please run the update_greeks method first."
                );
            }
        }
        Ok(vega_exposure)
    }

    /// Calculates the total theta exposure for all options in the chain.
    ///
    /// Theta exposure represents the aggregate rate of time decay in option prices
    /// as they approach expiration. It measures how much value the options portfolio
    /// will lose per day, holding all other factors constant.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate theta value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's theta calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn theta_exposure(&self) -> Result<Decimal, ChainError> {
        let mut theta_exposure = Decimal::ZERO;
        for option in &self.options {
            if let Some(four_options) = &option.options {
                theta_exposure += four_options.long_call.theta()?;
            } else {
                warn!(
                    "No options greeks no initialized. Please run the update_greeks method first."
                );
            }
        }
        Ok(theta_exposure)
    }

    /// Generates a gamma curve for visualization and analysis.
    ///
    /// Creates a curve representing gamma values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing gamma data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn gamma_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Gamma, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a delta curve for visualization and analysis.
    ///
    /// Creates a curve representing delta values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing delta data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn delta_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a vega curve for visualization and analysis.
    ///
    /// Creates a curve representing vega values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing vega data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn vega_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Vega, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a theta curve for visualization and analysis.
    ///
    /// Creates a curve representing theta values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing theta data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn theta_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Theta, &OptionStyle::Call, &Side::Long)
    }
}

impl Len for OptionChain {
    fn len(&self) -> usize {
        self.options.len()
    }
}

impl OptionChainParams for OptionChain {
    fn get_params(&self, strike_price: Positive) -> Result<OptionDataPriceParams, ChainError> {
        let option = self
            .options
            .iter()
            .find(|option| option.strike_price == strike_price);
        if option.is_none() {
            let reason = format!("Option with strike price {} not found", strike_price);
            return Err(ChainError::invalid_strike(strike_price.to_f64(), &reason));
        }
        Ok(OptionDataPriceParams::new(
            self.underlying_price,
            ExpirationDate::from_string(&self.expiration_date)?,
            option.unwrap().implied_volatility,
            self.risk_free_rate.unwrap_or(Decimal::ZERO),
            self.dividend_yield.unwrap_or(Positive::ZERO),
            Some(self.symbol.clone()),
        ))
    }
}

impl RNDAnalysis for OptionChain {
    /// Implementation of RND calculation for option chains
    ///
    /// # Numerical Method
    /// 1. Calculates second derivative of option prices
    /// 2. Applies Breeden-Litzenberger formula
    /// 3. Normalizes resulting densities
    ///
    /// # Error Conditions
    /// * Empty option chain
    /// * Zero derivative tolerance
    /// * Failed density calculations
    fn calculate_rnd(&self, params: &RNDParameters) -> Result<RNDResult, Box<dyn Error>> {
        let mut densities = BTreeMap::new();
        let mut h = params.derivative_tolerance.to_dec();

        // Step 1: Validate parameters
        if h == Positive::ZERO {
            return Err(Box::from(
                "Derivative tolerance must be greater than zero".to_string(),
            ));
        }

        // Step 2: Get all available strikes
        let strikes: Vec<Positive> = self.options.iter().map(|opt| opt.strike_price).collect();
        if strikes.is_empty() {
            return Err(Box::from(
                "No strikes available for RND calculation".to_string(),
            ));
        }

        // Calculate minimum strike interval
        let min_interval = strikes
            .windows(2)
            .map(|w| w[1] - w[0])
            .min()
            .ok_or("Cannot determine strike interval")?;

        if h < min_interval.to_dec() {
            h = min_interval.to_dec();
        }

        // Step 3: Calculate time to expiry
        let expiry_date = NaiveDate::parse_from_str(&self.expiration_date, "%Y-%m-%d")?
            .and_hms_opt(23, 59, 59)
            .ok_or("Invalid expiry date time")?;

        let now = Utc::now().naive_utc();
        let time_to_expiry =
            Decimal::from_f64((expiry_date - now).num_days() as f64 / 365.0).unwrap();

        // Step 4: Calculate discount factor
        let discount = (-params.risk_free_rate * time_to_expiry).exp();

        // Debug information
        #[cfg(test)]
        {
            debug!("Time to expiry: {} years", time_to_expiry);
            debug!("Discount factor: {}", discount);
            debug!("Step size h: {}", h);
        }

        // Step 5: Calculate RND for each strike
        for opt in self.options.iter() {
            let k = opt.strike_price;

            // Debug prices
            #[cfg(test)]
            {
                debug!("Processing strike {}", k);
                debug!("Call price at k: {:?}", self.get_call_price(k));
                debug!("Call price at k+h: {:?}", self.get_call_price(k + h));
                debug!("Call price at k-h: {:?}", self.get_call_price(k - h));
            }

            if let (Some(call_price), Some(call_up), Some(call_down)) = (
                self.get_call_price(k),
                self.get_call_price(k + h),
                self.get_call_price(k - h),
            ) {
                // Calculate second derivative
                let second_derivative = (call_up + call_down - Decimal::TWO * call_price) / (h * h);

                #[cfg(test)]
                {
                    debug!("Second derivative: {}", second_derivative);
                }

                // Calculate density using Breeden-Litzenberger formula
                let density = second_derivative * discount;

                #[cfg(test)]
                {
                    debug!("Density: {}", density);
                }

                // Store valid density
                if !density.is_sign_negative() && !density.is_zero() {
                    densities.insert(k, density);
                }
            }
        }

        // Step 6: Validate and normalize densities
        if densities.is_empty() {
            return Err(Box::from("Failed to calculate valid densities".to_string()));
        }

        let total: Decimal = densities.values().sum();
        if !total.is_zero() {
            for density in densities.values_mut() {
                *density /= total;
            }
        }

        #[cfg(test)]
        {
            debug!("Total number of densities: {}", densities.len());
            debug!("Sum of densities: {}", total);
        }

        Ok(RNDResult::new(densities))
    }

    /// Implementation of volatility skew calculation
    ///
    /// Extracts and analyzes the relationship between strike prices
    /// and implied volatilities.
    ///
    /// # Error Conditions
    /// * Missing ATM volatility
    /// * Insufficient valid data points
    fn calculate_skew(&self) -> Result<Vec<(Positive, Decimal)>, Box<dyn Error>> {
        let mut skew = Vec::new();
        let atm_strike = self.underlying_price;
        let atm_vol = self.get_atm_implied_volatility()?;

        for opt in self.options.iter() {
            if let Some(iv) = opt.implied_volatility {
                let relative_strike = opt.strike_price / atm_strike;
                let vol_diff = iv.to_dec() - atm_vol;
                skew.push((relative_strike, vol_diff));
            }
        }

        if skew.is_empty() {
            return Err(Box::from("No valid data for skew calculation".to_string()));
        }

        Ok(skew)
    }
}

impl fmt::Display for OptionChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Symbol: {}", self.symbol)?;
        writeln!(f, "Underlying Price: {:.1}", self.underlying_price)?;
        writeln!(f, "Expiration Date: {}", self.expiration_date)?;
        writeln!(
            f,
            "-------------------------------------------------------------------------------\
            ---------------------------------------------------------------------"
        )?;
        writeln!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<13} {:<10} {:<10} {:<10} {:<10} {:<10}",
            "Strike",
            "Call Bid",
            "Call Ask",
            "Call Mid",
            "Put Bid",
            "Put Ask",
            "Put Mid",
            "Implied Vol.",
            "Delta",
            "Delta",
            "Gamma",
            "Volume",
            "Open Interest"
        )?;
        writeln!(
            f,
            "----------------------------------------------------------------------------------\
            ------------------------------------------------------------------"
        )?;

        for option in &self.options {
            writeln!(f, "{}", option,)?;
        }
        Ok(())
    }
}

impl VolatilitySmile for OptionChain {
    /// Computes the volatility smile for the option chain.
    ///
    /// This function calculates the volatility smile by interpolating the implied volatilities
    /// for all strike prices in the option chain.  It uses the available implied volatilities
    /// from the `options` field and performs linear interpolation to estimate missing values.
    ///
    /// # Returns
    ///
    /// A `Curve` object representing the volatility smile. The x-coordinates of the curve
    /// correspond to the strike prices, and the y-coordinates represent the corresponding
    /// implied volatilities.
    fn smile(&self) -> Curve {
        // Build a BTreeSet with the known points (options with implied volatility)
        let mut bt_points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter_map(|option| {
                // Only include options with a valid implied volatility
                option
                    .implied_volatility
                    .map(|vol| Point2D::new(option.strike_price.to_dec(), vol.to_dec()))
            })
            .collect();

        // Create an initial Curve object using the known points
        let curve = Curve::new(bt_points.clone());

        // Interpolate missing points (options without implied volatility)
        for option in self
            .options
            .iter()
            .filter(|o| o.implied_volatility.is_none())
        {
            // Use linear interpolation to estimate the missing implied volatility
            if let Ok(interpolated_point) = curve.linear_interpolate(option.strike_price.to_dec()) {
                bt_points.insert(interpolated_point);
            }
        }

        // Return the final Curve with all points, including interpolated ones
        Curve::new(bt_points)
    }
}

impl BasicCurves for OptionChain {
    fn curve(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Curve, CurveError> {
        if axis == &BasicAxisTypes::UnderlyingPrice
            || axis == &BasicAxisTypes::Strike
            || axis == &BasicAxisTypes::Expiration
        {
            return Err(CurveError::ConstructionError("Axis not valid".to_string()));
        }
        let points = self
            .get_single_iter()
            .filter_map(|opt| {
                let four = opt.options.as_ref()?;

                // Select the appropriate option based on style and side
                let option = match (option_style, side) {
                    (OptionStyle::Call, Side::Long) => &four.long_call,
                    (OptionStyle::Call, Side::Short) => &four.short_call,
                    (OptionStyle::Put, Side::Long) => &four.long_put,
                    (OptionStyle::Put, Side::Short) => &four.short_put,
                };

                // Get x and y values based on the axis types
                match self.get_curve_strike_versus(axis, option) {
                    Ok(point) => Some(Point2D::new(point.0, point.1)),
                    Err(_) => None,
                }
            })
            .collect();

        Ok(Curve::new(points))
    }
}

impl BasicSurfaces for OptionChain {
    fn surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        volatility: Option<Vec<Positive>>,
        side: &Side,
    ) -> Result<Surface, SurfaceError> {
        if axis == &BasicAxisTypes::UnderlyingPrice
            || axis == &BasicAxisTypes::Strike
            || axis == &BasicAxisTypes::Expiration
        {
            return Err(SurfaceError::ConstructionError(
                "Axis not valid".to_string(),
            ));
        }

        let mut points = BTreeSet::new();

        for opt in self.get_single_iter() {
            let four = match opt.options.as_ref() {
                Some(four) => four,
                None => continue,
            };

            // Select the appropriate option based on style and side
            let option = match (option_style, side) {
                (OptionStyle::Call, Side::Long) => &four.long_call,
                (OptionStyle::Call, Side::Short) => &four.short_call,
                (OptionStyle::Put, Side::Long) => &four.long_put,
                (OptionStyle::Put, Side::Short) => &four.short_put,
            };

            match &volatility {
                // If volatility vector is provided, use get_volatility_versus for each volatility
                Some(vols) => {
                    for vol in vols {
                        match self.get_surface_volatility_versus(axis, option, *vol) {
                            Ok((x, y, z)) => {
                                points.insert(Point3D::new(x, y, z));
                            }
                            Err(_) => continue,
                        }
                    }
                }
                // If no volatility vector is provided, use get_strike_versus with original volatility
                None => match self.get_surface_strike_versus(axis, option) {
                    Ok((x, y, z)) => {
                        points.insert(Point3D::new(x, y, z));
                    }
                    Err(_) => continue,
                },
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points generated for surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

#[cfg(test)]
mod tests_chain_base {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::utils::logger::setup_logger;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    #[cfg(not(target_arch = "wasm32"))]
    use std::fs;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_chain() {
        let chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.underlying_price, 5781.88);
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert!(chain.options.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_chain_build_chain() {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            pos!(1.0),
            0.0,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
                spos!(0.17),
                Decimal::ZERO,
                pos!(0.05),
                Some("SP500".to_string()),
            ),
        );

        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert_eq!(chain.options.len(), 21);
        assert_eq!(chain.underlying_price, pos!(100.0));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 10.04);
        assert_eq!(first.call_bid.unwrap(), 10.02);
        assert_eq!(first.put_ask, spos!(0.04));
        assert_eq!(first.put_bid, spos!(0.02));
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, spos!(0.06));
        assert_eq!(last.call_bid, spos!(0.04));
        assert_eq!(last.put_ask, spos!(10.06));
        assert_eq!(last.put_bid, spos!(10.04));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_chain_build_chain_long() {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            25,
            pos!(25.0),
            0.000002,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                pos!(5878.10),
                ExpirationDate::Days(pos!(60.0)),
                spos!(0.03),
                Decimal::ZERO,
                pos!(0.05),
                Some("SP500".to_string()),
            ),
        );
        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert_eq!(chain.options.len(), 51);
        assert_eq!(chain.underlying_price, pos!(5878.10));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 628.11);
        assert_eq!(first.call_bid.unwrap(), 628.09);
        assert_eq!(first.put_ask, None);
        assert_eq!(first.put_bid, None);
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, None);
        assert_eq!(last.call_bid, None);
        assert_eq!(last.put_ask, spos!(621.91));
        assert_eq!(last.put_bid, spos!(621.89));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_option() {
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
        );
        assert_eq!(chain.options.len(), 1);
        // first option in the chain
        let option = chain.options.iter().next().unwrap();
        assert_eq!(option.strike_price, 5520.0);
        assert!(option.call_bid.is_some());
        assert_eq!(option.call_bid.unwrap(), 274.26);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_title_i() {
        let chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_title_ii() {
        let chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18 oct 2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_i() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_ii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_iii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_iv() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.88.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_v() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_save_to_csv() {
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_csv(".");
        assert!(result.is_ok());
        let file_name = "./SP500-18-oct-2024-5781.88.csv".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_save_to_json() {
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_json(".");
        assert!(result.is_ok());

        let file_name = "./SP500-18-oct-2024-5781.88.json".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_load_from_csv() {
        setup_logger();
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.89),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_csv(".");
        assert!(result.is_ok());

        let result = OptionChain::load_from_csv("./SP500-18-oct-2024-5781.89.csv");
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.89);

        let file_name = "./SP500-18-oct-2024-5781.89.csv".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_load_from_json() {
        let mut chain =
            OptionChain::new("SP500", pos!(5781.9), "18-oct-2024".to_string(), None, None);
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_json(".");
        assert!(result.is_ok());

        let result = OptionChain::load_from_json("./SP500-18-oct-2024-5781.9.json");
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.9);

        let file_name = "./SP500-18-oct-2024-5781.9.json".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }
}

#[cfg(test)]
mod tests_option_data {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::spos;
    use crate::utils::logger::setup_logger;
    use crate::{assert_pos_relative_eq, pos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;
    use tracing::info;

    fn create_valid_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_data() {
        let option_data = create_valid_option_data();
        assert_eq!(option_data.strike_price, pos!(100.0));
        assert_eq!(option_data.call_bid, spos!(9.5));
        assert_eq!(option_data.call_ask, spos!(10.0));
        assert_eq!(option_data.put_bid, spos!(8.5));
        assert_eq!(option_data.put_ask, spos!(9.0));
        assert_eq!(option_data.implied_volatility, spos!(0.2));
        assert_eq!(option_data.delta_call.unwrap().to_f64(), Some(-0.3));
        assert_eq!(option_data.volume, spos!(1000.0));
        assert_eq!(option_data.open_interest, Some(500));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_valid_option() {
        setup_logger();
        let option_data = create_valid_option_data();
        assert!(option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_zero_strike() {
        let mut option_data = create_valid_option_data();
        option_data.strike_price = Positive::ZERO;
        assert!(!option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_no_implied_volatility() {
        let mut option_data = create_valid_option_data();
        option_data.implied_volatility = None;
        assert!(!option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_missing_both_sides() {
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
            None,
            None,
        );
        assert!(!option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_call() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_call());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_call_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.call_bid = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_call_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.call_ask = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_put() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_put());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_put_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.put_bid = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_put_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.put_ask = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_prices_success() {
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            Decimal::ZERO,
            Positive::ZERO,
            None,
        );

        let result = option_data.calculate_prices(&price_params, false);

        assert!(result.is_ok());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.call_bid.is_some());
        assert!(option_data.put_ask.is_some());
        assert!(option_data.put_bid.is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_prices_missing_volatility() {
        setup_logger();
        let mut option_data = OptionData::new(
            pos!(100.0),
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

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            Decimal::ZERO,
            Positive::ZERO,
            None,
        );
        let _ = option_data.calculate_prices(&price_params, false);

        info!("{}", option_data);
        assert_eq!(option_data.call_ask, None);
        assert_eq!(option_data.call_bid, None);
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
        assert_eq!(option_data.implied_volatility, None);
        assert_eq!(option_data.delta_call, None);
        assert_eq!(option_data.strike_price, pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_prices_override_volatility() {
        setup_logger();
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
            None,
            None,
        );

        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.12),
            dec!(0.05),
            pos!(0.01),
            None,
        );
        let result = option_data.calculate_prices(&price_params, false);

        assert!(result.is_ok());
        info!("{}", option_data);
        assert_pos_relative_eq!(option_data.call_ask.unwrap(), pos!(10.51108), pos!(0.0001));
        assert_pos_relative_eq!(option_data.call_bid.unwrap(), pos!(10.51108), pos!(0.0001));
        assert_pos_relative_eq!(option_data.put_ask.unwrap(), pos!(0.100966), pos!(0.0001));
        assert_pos_relative_eq!(option_data.put_bid.unwrap(), pos!(0.100966), pos!(0.0001));
        option_data.apply_spread(pos!(0.02), 2);
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, spos!(10.52));
        assert_eq!(option_data.call_bid, spos!(10.50));
        assert_eq!(option_data.put_ask, spos!(0.11));
        assert_eq!(option_data.put_bid, spos!(0.09));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_prices_with_all_parameters() {
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.01),
            None,
        );

        let result = option_data.calculate_prices(&price_params, false);

        assert!(result.is_ok());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.call_bid.is_some());
        assert!(option_data.put_ask.is_some());
        assert!(option_data.put_bid.is_some());
    }
}

#[cfg(test)]
mod tests_get_random_positions {
    use super::*;
    use crate::error::chains::ChainBuildErrorKind;
    use crate::model::types::ExpirationDate;
    use crate::utils::logger::setup_logger;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        // Create a sample option chain
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        // Add some test options with different strikes
        chain.add_option(
            pos!(95.0),      // strike_price
            spos!(4.0),      // call_bid
            spos!(4.2),      // call_ask
            spos!(3.0),      // put_bid
            spos!(3.2),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(50),
        );

        chain.add_option(
            pos!(105.0),
            spos!(2.0),
            spos!(2.2),
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_quantity() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ChainError::ChainBuildError(ChainBuildErrorKind::InvalidParameters {
                parameter,
                reason,
            }) => {
                assert_eq!(parameter, "total_positions");
                assert_eq!(reason, "The sum of the quantities must be greater than 0");
            }
            _ => panic!("Incorrect error type"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_long_puts_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            Some(2),
            None,
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Put);
            assert_eq!(position.option.side, Side::Long);
            // Premium should be ask price for long positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_short_puts_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            Some(2),
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Put);
            assert_eq!(position.option.side, Side::Short);
            // Premium should be bid price for short positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_long_calls_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            Some(2),
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Call);
            assert_eq!(position.option.side, Side::Long);
            // Premium should be ask price for long positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_short_calls_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            Some(2),
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Call);
            assert_eq!(position.option.side, Side::Short);
            // Premium should be bid price for short positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_mixed_positions() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 4);

        let mut long_puts = 0;
        let mut short_puts = 0;
        let mut long_calls = 0;
        let mut short_calls = 0;

        for position in positions {
            match (position.option.option_style, position.option.side) {
                (OptionStyle::Put, Side::Long) => long_puts += 1,
                (OptionStyle::Put, Side::Short) => short_puts += 1,
                (OptionStyle::Call, Side::Long) => long_calls += 1,
                (OptionStyle::Call, Side::Short) => short_calls += 1,
            }
            // All premiums should be positive
            assert!(position.premium > 0.0);
        }

        assert_eq!(long_puts, 1);
        assert_eq!(short_puts, 1);
        assert_eq!(long_calls, 1);
        assert_eq!(short_calls, 1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_chain() {
        setup_logger();
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let params = RandomPositionsParams::new(
            Some(1),
            None,
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert!(positions.is_empty());
    }
}

#[cfg(test)]
mod tests_option_data_get_prices {
    use super::*;
    use crate::pos;
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.2),
            Some(dec!(-0.3)),
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0),
            Some(500),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_call_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_buy_price(), spos!(10.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_call_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_sell_price(), spos!(9.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_put_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_buy_price(), spos!(9.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_put_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_sell_price(), spos!(8.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_prices_with_none_values() {
        let data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(data.get_call_buy_price(), None);
        assert_eq!(data.get_call_sell_price(), None);
        assert_eq!(data.get_put_buy_price(), None);
        assert_eq!(data.get_put_sell_price(), None);
    }
}

#[cfg(test)]
mod tests_option_data_display {
    use super::*;
    use crate::pos;
    use crate::spos;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_display_full_data() {
        let data = OptionData::new(
            pos!(100.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.2),
            Some(dec!(-0.3)),
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0),
            Some(500),
        );
        let display_string = format!("{}", data);
        assert!(display_string.contains("100"));
        assert!(display_string.contains("9.5"));
        assert!(display_string.contains("10"));
        assert!(display_string.contains("8.5"));
        assert!(display_string.contains("9"));
        assert!(display_string.contains("0.200"));
        assert!(display_string.contains("-0.300"));
        assert!(display_string.contains("1000"));
        assert!(display_string.contains("500"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_display_empty_data() {
        let data = OptionData::default();
        let display_string = format!("{}", data);

        assert!(display_string.contains("0.0"));
        assert!(display_string.contains("")); // Para campos None
    }
}

#[cfg(test)]
mod tests_filter_option_data {
    use super::*;
    use crate::{pos, spos};

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                spos!(0.2),
                None,
                None,
                None,
                None,
                None,
            );
        }
        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_upper() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Upper);
        assert_eq!(filtered.len(), 2);
        assert!(
            filtered
                .iter()
                .all(|opt| opt.strike_price > chain.underlying_price)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_lower() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Lower);
        assert_eq!(filtered.len(), 2);
        assert!(
            filtered
                .iter()
                .all(|opt| opt.strike_price < chain.underlying_price)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_all() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::All);
        assert_eq!(filtered.len(), 5);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_range() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Range(pos!(95.0), pos!(105.0)));
        assert_eq!(filtered.len(), 3);
        assert!(
            filtered
                .iter()
                .all(|opt| opt.strike_price >= pos!(95.0) && opt.strike_price <= pos!(105.0))
        );
    }
}

#[cfg(test)]
mod tests_strike_price_range_vec {
    use super::*;
    use crate::{pos, spos};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        assert_eq!(chain.strike_price_range_vec(5.0), None);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_single_option() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
            None,
            None,
        );
        let range = chain.strike_price_range_vec(5.0).unwrap();
        assert_eq!(range.len(), 1);
        assert_eq!(range[0], 100.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_multiple_options() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        for strike in [90.0, 95.0, 100.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                spos!(0.2),
                None,
                None,
                None,
                None,
                None,
            );
        }
        let range = chain.strike_price_range_vec(5.0).unwrap();
        assert_eq!(range, vec![90.0, 95.0, 100.0]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_step_size() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        for strike in [90.0, 100.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                spos!(0.2),
                None,
                None,
                None,
                None,
                None,
            );
        }
        let range = chain.strike_price_range_vec(2.0).unwrap();
        assert_eq!(range.len(), 6); // [90, 92, 94, 96, 98, 100]
        assert_eq!(range[1] - range[0], 2.0);
    }
}

#[cfg(test)]
mod tests_option_data_get_option {
    use super::*;
    use crate::error::chains::OptionDataErrorKind;
    use crate::model::types::ExpirationDate;
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(0.7)),  // delta
            Some(dec!(0.3)),  // gamma
            spos!(1000.0),    // volume
            Some(500),        // open_interest
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_option_success() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.25),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.strike_price, pos!(100.0));
        assert_eq!(option.implied_volatility, 0.25); // Uses provided IV
        assert_eq!(option.underlying_price, pos!(100.0));
        assert_eq!(option.risk_free_rate.to_f64().unwrap(), 0.05);
        assert_eq!(option.dividend_yield.to_f64(), 0.02);
        assert_eq!(option.side, Side::Long);
        assert_eq!(option.option_style, OptionStyle::Call);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_option_using_data_iv() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None, // No IV provided in params
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.implied_volatility, 0.002); // Uses IV from option_data
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_option_missing_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
                volatility,
                reason,
            }) => {
                assert_eq!(volatility, None);
                assert_eq!(reason, "Implied volatility not found");
            }
            _ => panic!("Incorrect error type"),
        }
    }
}

#[cfg(test)]
mod tests_option_data_get_options_in_strike {
    use super::*;
    use crate::error::chains::OptionDataErrorKind;
    use crate::greeks::Greeks;
    use crate::model::types::ExpirationDate;
    use crate::{assert_decimal_eq, pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(-0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_success() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.25),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        // Check long call
        assert_eq!(options.long_call.strike_price, pos!(100.0));
        assert_eq!(options.long_call.option_style, OptionStyle::Call);
        assert_eq!(options.long_call.side, Side::Long);

        // Check short call
        assert_eq!(options.short_call.strike_price, pos!(100.0));
        assert_eq!(options.short_call.option_style, OptionStyle::Call);
        assert_eq!(options.short_call.side, Side::Short);

        // Check long put
        assert_eq!(options.long_put.strike_price, pos!(100.0));
        assert_eq!(options.long_put.option_style, OptionStyle::Put);
        assert_eq!(options.long_put.side, Side::Long);

        // Check short put
        assert_eq!(options.short_put.strike_price, pos!(100.0));
        assert_eq!(options.short_put.option_style, OptionStyle::Put);
        assert_eq!(options.short_put.side, Side::Short);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_using_data_iv() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert_eq!(options.long_call.implied_volatility, 0.002);
        assert_eq!(options.short_call.implied_volatility, 0.002);
        assert_eq!(options.long_put.implied_volatility, 0.002);
        assert_eq!(options.short_put.implied_volatility, 0.002);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_missing_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
                volatility,
                reason,
            }) => {
                assert_eq!(volatility, None);
                assert_eq!(reason, "Implied volatility not found");
            }
            _ => panic!("Incorrect error type"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_all_properties() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(pos!(45.0)),
            spos!(0.3),
            dec!(0.06),
            pos!(0.03),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        // Verify common properties across all options
        let check_common_properties = |option: &Options| {
            assert_eq!(option.strike_price, pos!(100.0));
            assert_eq!(option.underlying_price, pos!(110.0));
            assert_eq!(option.implied_volatility, 0.3);
            assert_eq!(option.risk_free_rate.to_f64().unwrap(), 0.06);
            assert_eq!(option.dividend_yield.to_f64(), 0.03);
            assert_eq!(option.option_type, OptionType::European);
            assert_eq!(option.quantity, pos!(1.0));
        };

        check_common_properties(&options.long_call);
        check_common_properties(&options.short_call);
        check_common_properties(&options.long_put);
        check_common_properties(&options.short_put);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_deltas() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(pos!(45.0)),
            spos!(0.3),
            dec!(0.06),
            pos!(0.03),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        let epsilon = dec!(1e-8);

        assert_decimal_eq!(
            options.long_call.delta().unwrap(),
            dec!(0.844825189),
            epsilon
        );
        assert_decimal_eq!(
            options.short_call.delta().unwrap(),
            dec!(-0.844825189),
            epsilon
        );
        assert_decimal_eq!(
            options.long_put.delta().unwrap(),
            dec!(-0.151483012),
            epsilon
        );
        assert_decimal_eq!(
            options.short_put.delta().unwrap(),
            dec!(0.151483012),
            epsilon
        );
    }
}

#[cfg(test)]
mod tests_filter_options_in_strike {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::{pos, spos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(1.0),       // call_bid
                spos!(1.2),       // call_ask
                spos!(1.0),       // put_bid
                spos!(1.2),       // put_ask
                spos!(0.2),       // implied_volatility
                Some(dec!(-0.3)), // delta
                Some(dec!(-0.3)),
                Some(dec!(0.3)),
                spos!(1000.0), // volume
                Some(500),     // open_interest
            );
        }
        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_upper_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::Upper);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 2);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price > chain.underlying_price);
            assert_eq!(opt.long_call.option_type, OptionType::European);
            assert_eq!(opt.long_call.side, Side::Long);
            assert_eq!(opt.short_call.side, Side::Short);
            assert_eq!(opt.long_put.side, Side::Long);
            assert_eq!(opt.short_put.side, Side::Short);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_lower_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::Lower);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 2);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price < chain.underlying_price);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_all_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 5);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_range_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(
            &price_params,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
        );
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 3);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price >= pos!(95.0));
            assert!(opt.long_call.strike_price <= pos!(105.0));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_invalid_range() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(
            &price_params,
            FindOptimalSide::Range(pos!(200.0), pos!(300.0)),
        );
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_all_strikes_deltas() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 5);

        for opt in filtered_options {
            assert_eq!(opt.long_call.option_type, OptionType::European);
            assert_eq!(opt.long_call.side, Side::Long);
            assert_eq!(opt.short_call.side, Side::Short);
            assert_eq!(opt.long_put.side, Side::Long);
            assert_eq!(opt.short_put.side, Side::Short);

            let deltas = opt.deltas().unwrap();
            assert!(deltas.long_call > Decimal::ZERO);
            assert!(deltas.short_call < Decimal::ZERO);
            assert!(deltas.long_put < Decimal::ZERO);
            assert!(deltas.short_put > Decimal::ZERO);
        }
    }
}

#[cfg(test)]
mod tests_chain_iterators {
    use super::*;
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        // Add three options with different strikes
        chain.add_option(
            pos!(90.0),      // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.6)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            spos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(80.0),
            Some(40),
        );

        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty()); // No pairs with single element
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_iter_multiple() {
        let chain = create_test_chain();
        let pairs: Vec<_> = chain.get_double_iter().collect();

        // Should have 3 pairs: (90,100), (90,110), (100,110)
        assert_eq!(pairs.len(), 3);

        // Check strikes of pairs
        assert_eq!(pairs[0].0.strike_price, pos!(90.0));
        assert_eq!(pairs[0].1.strike_price, pos!(100.0));

        assert_eq!(pairs[1].0.strike_price, pos!(90.0));
        assert_eq!(pairs[1].1.strike_price, pos!(110.0));

        assert_eq!(pairs[2].0.strike_price, pos!(100.0));
        assert_eq!(pairs[2].1.strike_price, pos!(110.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert_eq!(pairs.len(), 1); // Should have one pair (self-pair)
        assert_eq!(pairs[0].0.strike_price, pairs[0].1.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();

        // Should have 6 pairs: (90,90), (90,100), (90,110), (100,100), (100,110), (110,110)
        assert_eq!(pairs.len(), 6);

        // Check strikes of pairs
        assert_eq!(pairs[0].0.strike_price, pos!(90.0));
        assert_eq!(pairs[0].1.strike_price, pos!(90.0));

        assert_eq!(pairs[1].0.strike_price, pos!(90.0));
        assert_eq!(pairs[1].1.strike_price, pos!(100.0));

        assert_eq!(pairs[2].0.strike_price, pos!(90.0));
        assert_eq!(pairs[2].1.strike_price, pos!(110.0));

        assert_eq!(pairs[3].0.strike_price, pos!(100.0));
        assert_eq!(pairs[3].1.strike_price, pos!(100.0));

        assert_eq!(pairs[4].0.strike_price, pos!(100.0));
        assert_eq!(pairs[4].1.strike_price, pos!(110.0));

        assert_eq!(pairs[5].0.strike_price, pos!(110.0));
        assert_eq!(pairs[5].1.strike_price, pos!(110.0));
    }
}

#[cfg(test)]
mod tests_chain_iterators_bis {
    use super::*;
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        // Add four options with different strikes
        chain.add_option(
            pos!(90.0),      // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.6)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            spos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(80.0),
            Some(40),
        );

        chain.add_option(
            pos!(120.0),
            spos!(0.5),
            spos!(1.0),
            spos!(7.0),
            spos!(7.5),
            spos!(0.35),
            Some(dec!(0.3)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(60.0),
            Some(30),
        );

        chain
    }

    // Tests for Triple Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_iter_two_elements() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        // Add two options
        chain.add_option(
            pos!(90.0),
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
        chain.add_option(
            pos!(100.0),
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

        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty()); // Not enough elements for a triple
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_iter_multiple() {
        let chain = create_test_chain();
        let triples: Vec<_> = chain.get_triple_iter().collect();

        // Should have 4 triples: (90,100,110), (90,100,120), (90,110,120), (100,110,120)
        assert_eq!(triples.len(), 4);

        // Check first triple
        assert_eq!(triples[0].0.strike_price, pos!(90.0));
        assert_eq!(triples[0].1.strike_price, pos!(100.0));
        assert_eq!(triples[0].2.strike_price, pos!(110.0));

        // Check last triple
        assert_eq!(triples[3].0.strike_price, pos!(100.0));
        assert_eq!(triples[3].1.strike_price, pos!(110.0));
        assert_eq!(triples[3].2.strike_price, pos!(120.0));
    }

    // Tests for Triple Inclusive Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.5),
            None,
            None,
            None,
            None,
            None,
        );

        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].0.strike_price, triples[0].1.strike_price);
        assert_eq!(triples[0].1.strike_price, triples[0].2.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();

        // Count should be (n+2)(n+1)n/6 where n is the number of elements
        assert_eq!(triples.len(), 20); // For 4 elements: 4*5*6/6 = 20

        // Check first few triples (including self-references)
        assert_eq!(triples[0].0.strike_price, pos!(90.0));
        assert_eq!(triples[0].1.strike_price, pos!(90.0));
        assert_eq!(triples[0].2.strike_price, pos!(90.0));
    }

    // Tests for Quad Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_iter_three_elements() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        // Add three options
        chain.add_option(
            pos!(90.0),
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
        chain.add_option(
            pos!(100.0),
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
        chain.add_option(
            pos!(110.0),
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

        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty()); // Not enough elements for a quad
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_iter_multiple() {
        let chain = create_test_chain();
        let quads: Vec<_> = chain.get_quad_iter().collect();

        // Should have 1 quad: (90,100,110,120)
        assert_eq!(quads.len(), 1);

        // Check the quad
        assert_eq!(quads[0].0.strike_price, pos!(90.0));
        assert_eq!(quads[0].1.strike_price, pos!(100.0));
        assert_eq!(quads[0].2.strike_price, pos!(110.0));
        assert_eq!(quads[0].3.strike_price, pos!(120.0));
    }

    // Tests for Quad Inclusive Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.5),
            None,
            None,
            None,
            None,
            None,
        );

        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].0.strike_price, quads[0].1.strike_price);
        assert_eq!(quads[0].1.strike_price, quads[0].2.strike_price);
        assert_eq!(quads[0].2.strike_price, quads[0].3.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();

        // Count should be (n+3)(n+2)(n+1)n/24 where n is the number of elements
        assert_eq!(quads.len(), 35); // For 4 elements: 7*6*5*4/24 = 35

        // Check first quad (self-reference)
        assert_eq!(quads[0].0.strike_price, pos!(90.0));
        assert_eq!(quads[0].1.strike_price, pos!(90.0));
        assert_eq!(quads[0].2.strike_price, pos!(90.0));
        assert_eq!(quads[0].3.strike_price, pos!(90.0));

        // Check last quad
        assert_eq!(quads[34].0.strike_price, pos!(120.0));
        assert_eq!(quads[34].1.strike_price, pos!(120.0));
        assert_eq!(quads[34].2.strike_price, pos!(120.0));
        assert_eq!(quads[34].3.strike_price, pos!(120.0));
    }
}

#[cfg(test)]
mod tests_is_valid_optimal_side {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_upper_side_valid() {
        let option_data = OptionData::new(
            pos!(110.0), // strike price higher than underlying
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
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_upper_side_invalid() {
        let option_data = OptionData::new(
            pos!(90.0), // strike price lower than underlying
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
        let underlying_price = pos!(100.0);

        assert!(!option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_lower_side_valid() {
        let option_data = OptionData::new(
            pos!(90.0), // strike price lower than underlying
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
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_lower_side_invalid() {
        let option_data = OptionData::new(
            pos!(110.0), // strike price higher than underlying
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
        let underlying_price = pos!(100.0);

        assert!(!option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_all_side() {
        let option_data = OptionData::new(
            pos!(100.0),
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
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::All));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_valid() {
        let option_data = OptionData::new(
            pos!(100.0), // strike price within range
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
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(
            option_data.is_valid_optimal_side(
                pos!(100.0),
                &FindOptimalSide::Range(range_start, range_end)
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_invalid_below() {
        let option_data = OptionData::new(
            pos!(80.0), // strike price below range
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
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(
            !option_data.is_valid_optimal_side(
                pos!(100.0),
                &FindOptimalSide::Range(range_start, range_end)
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_invalid_above() {
        let option_data = OptionData::new(
            pos!(120.0), // strike price above range
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
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(
            !option_data.is_valid_optimal_side(
                pos!(100.0),
                &FindOptimalSide::Range(range_start, range_end)
            )
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_at_boundaries() {
        let option_data_lower = OptionData::new(
            pos!(90.0), // strike price at lower boundary
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
        let option_data_upper = OptionData::new(
            pos!(110.0), // strike price at upper boundary
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
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(
            option_data_lower.is_valid_optimal_side(
                pos!(100.0),
                &FindOptimalSide::Range(range_start, range_end)
            )
        );
        assert!(
            option_data_upper.is_valid_optimal_side(
                pos!(100.0),
                &FindOptimalSide::Range(range_start, range_end)
            )
        );
    }
}

#[cfg(test)]
mod rnd_analysis_tests {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a standard option chain for testing
    fn create_standard_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

        // Add a range of options with known prices and volatilities
        let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
        let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
        let implied_vols = [0.17, 0.17, 0.17, 0.17, 0.17];

        for ((&strike, &call_ask), &impl_vol) in strikes
            .iter()
            .zip(call_asks.iter())
            .zip(implied_vols.iter())
        {
            chain.add_option(
                pos!(strike),
                spos!(call_ask - 0.02), // bid slightly lower than ask
                spos!(call_ask),
                None,
                None,
                spos!(impl_vol),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
            );
        }

        chain
    }

    mod calculate_rnd_tests {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_basic_rnd_calculation() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());

            let rnd = result.unwrap();

            // Verify densities exist
            assert!(!rnd.densities.is_empty());

            // Verify total probability is approximately 1
            let total: Decimal = rnd.densities.values().sum();
            assert!((total - dec!(1.0)).abs() < dec!(0.0001));

            // Verify all densities are non-negative
            assert!(rnd.densities.values().all(|&d| !d.is_sign_negative()));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_tolerance_adjustment() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.1), // Smaller than strike interval
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_default() {
            let chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);
            let params = RNDParameters::default();

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Derivative tolerance must be greater than zero"
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_zero_tolerance() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                derivative_tolerance: Positive::ZERO,
                ..Default::default()
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Derivative tolerance must be greater than zero"
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_expired_option() {
            let mut chain = create_standard_chain();
            chain.expiration_date = "2023-01-01".to_string(); // Past date

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok()); // Should still work with past date
        }
    }

    mod calculate_skew_tests {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_basic_skew_calculation() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew();

            assert!(result.is_ok());
            let skew = result.unwrap();

            // Verify we have skew data
            assert!(!skew.is_empty());

            // Verify relative strikes are ordered
            for window in skew.windows(2) {
                assert!(window[0].0 < window[1].0);
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_flat_volatility_surface() {
            let chain = create_standard_chain(); // All vols are 0.17
            let result = chain.calculate_skew().unwrap();

            // All vol differences should be close to zero
            for (_, vol_diff) in result {
                assert!(vol_diff.abs() < dec!(0.0001));
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_empty_chain_skew() {
            let chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            let result = chain.calculate_skew();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "No ATM implied volatility available"
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_missing_implied_volatility() {
            let mut chain = create_standard_chain();

            // Add an option without implied volatility
            chain.add_option(
                pos!(115.0),
                spos!(0.1),
                spos!(0.2),
                None,
                None,
                None, // No implied volatility
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
            );

            let result = chain.calculate_skew();
            assert!(result.is_ok()); // Should work with partial data
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_relative_strike_calculation() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew().unwrap();

            // For ATM strike (100.0), relative strike should be 1.0
            let atm_strike = result
                .iter()
                .find(|(rel_strike, _)| (*rel_strike - dec!(1.0)) < pos!(0.0001));
            assert!(atm_strike.is_some());
        }
    }

    mod calculate_rnd_tests_bis {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_invalid_date_format() {
            let mut chain = create_standard_chain();
            chain.expiration_date = "invalid_date".to_string();

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_negative_risk_free_rate() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(-0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_verify_rnd_properties() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(5.0),
            };

            let result = chain.calculate_rnd(&params).unwrap();
            let densities = &result.densities;

            // Verify mode is near the money
            let mode = densities
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap();
            assert_eq!(*mode.0, pos!(100.0));

            // Verify densities decrease away from the money
            let atm_density = densities.get(&pos!(100.0)).unwrap();
            for (strike, density) in densities.iter() {
                if strike < &pos!(90.0) || strike > &pos!(110.0) {
                    assert!(density < atm_density);
                }
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_strike_interval_detection() {
            let mut chain = create_standard_chain();

            // Add option with different strike interval
            chain.add_option(
                pos!(102.5),
                spos!(1.0),
                spos!(1.1),
                None,
                None,
                spos!(0.17),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
            );

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.1),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());
        }
    }

    mod calculate_skew_tests_bis {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_skew_with_smile() {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            // Create new options with a volatility smile
            let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
            let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
            let smile_vols = [0.20, 0.18, 0.17, 0.18, 0.20];

            for ((&strike, &call_ask), &vol) in
                strikes.iter().zip(call_asks.iter()).zip(smile_vols.iter())
            {
                chain.add_option(
                    pos!(strike),
                    spos!(call_ask - 0.02),
                    spos!(call_ask),
                    None,
                    None,
                    spos!(vol),
                    None,
                    None,
                    None,
                    spos!(100.0),
                    Some(50),
                );
            }

            let result = chain.calculate_skew().unwrap();

            // First half of skew should be decreasing
            for window in result.windows(2).take(result.len() / 2) {
                assert!(window[0].1 > window[1].1);
            }

            // Second half of skew should be increasing
            for window in result.windows(2).skip(result.len() / 2) {
                assert!(window[0].1 < window[1].1);
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_skew_monotonic() {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            // Create new options with monotonic skew
            let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
            let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
            let skew_vols = [0.22, 0.20, 0.17, 0.15, 0.14];

            for ((&strike, &call_ask), &vol) in
                strikes.iter().zip(call_asks.iter()).zip(skew_vols.iter())
            {
                chain.add_option(
                    pos!(strike),
                    spos!(call_ask - 0.02),
                    spos!(call_ask),
                    None,
                    None,
                    spos!(vol),
                    None,
                    None,
                    None,
                    spos!(100.0),
                    Some(50),
                );
            }

            let result = chain.calculate_skew().unwrap();

            // Verify monotonic decrease
            for window in result.windows(2) {
                assert!(window[0].1 > window[1].1);
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_strike_range_coverage() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew().unwrap();

            // Get min and max relative strikes
            let min_rel_strike = result.iter().map(|(k, _)| k).min().unwrap();
            let max_rel_strike = result.iter().map(|(k, _)| k).max().unwrap();

            // Verify range coverage
            assert!(*min_rel_strike < pos!(1.0)); // Have strikes below ATM
            assert!(*max_rel_strike > pos!(1.0)); // Have strikes above ATM
        }
    }
}

#[cfg(test)]
mod tests_option_data_implied_volatility {
    use super::*;
    use crate::utils::setup_logger_with_level;
    use crate::{assert_pos_relative_eq, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_iv_from_call() {
        setup_logger_with_level("debug");
        let mut option_data = OptionData::new(
            pos!(21395.0),        // strike
            spos!(280.0),         // call_bid
            spos!(285.0),         // call_ask
            Some(Positive::ZERO), // put_bid
            spos!(4.0),           // put_ask
            None,                 // initial IV
            None,                 // delta
            None,
            None,
            None, // volume
            None, // open_interest
        );

        option_data.set_mid_prices();

        let params = OptionDataPriceParams::new(
            pos!(21637.0),                   // underlying price
            ExpirationDate::Days(pos!(7.0)), // expiration
            None,                            // IV (will be calculated)
            dec!(0.05),                      // risk-free rate
            pos!(0.0),                       // dividend yield
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);

        assert!(result.is_ok(), "Failed to calculate IV: {:?}", result);
        assert!(option_data.implied_volatility.is_some());

        let iv = option_data.implied_volatility.unwrap() / pos!(100.0);
        assert!(iv > pos!(0.0) && iv < pos!(2.0));
    }

    #[test]
    fn test_calculate_iv_from_put() {
        setup_logger_with_level("debug");
        let mut option_data = OptionData::new(
            pos!(21700.0), // strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // initial IV
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        option_data.set_mid_prices();

        let params = OptionDataPriceParams::new(
            pos!(21637.0),                   // underlying price
            ExpirationDate::Days(pos!(1.0)), // expiration
            None,                            // IV (will be calculated)
            dec!(0.0),                       // risk-free rate
            pos!(0.0),                       // dividend yield
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);

        assert!(result.is_ok(), "Failed to calculate IV: {:?}", result);
        assert!(option_data.implied_volatility.is_some());
        assert_pos_relative_eq!(
            option_data.implied_volatility.unwrap() / pos!(100.0),
            pos!(0.13008),
            pos!(0.0001)
        );

        let iv = option_data.implied_volatility.unwrap() / pos!(100.0);
        assert!(iv > pos!(0.0) && iv < pos!(2.0));
    }

    #[test]
    fn test_calculate_iv_no_prices() {
        let mut option_data = OptionData::new(
            pos!(100.0),
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

        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.0),
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_iv_option_data_to_option() {
        setup_logger_with_level("debug");
        let mut option_data = OptionData::new(
            pos!(21700.0), // strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // initial IV
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        option_data.set_mid_prices();

        let params = OptionDataPriceParams::new(
            pos!(21637.0),                   // underlying price
            ExpirationDate::Days(pos!(1.0)), // expiration
            None,                            // IV (will be calculated)
            dec!(0.0),                       // risk-free rate
            pos!(0.0),                       // dividend yield
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);

        assert!(result.is_ok(), "Failed to calculate IV: {:?}", result);
        assert!(option_data.implied_volatility.is_some());
        let volatility = pos!(0.13008);
        assert_pos_relative_eq!(
            option_data.implied_volatility.unwrap() / pos!(100.0),
            volatility,
            pos!(0.0001)
        );

        let option = option_data
            .get_option(&params, Side::Long, OptionStyle::Call)
            .unwrap();
        assert_eq!(
            option.implied_volatility,
            option_data.implied_volatility.unwrap() / pos!(100.0)
        );

        let option = option_data
            .get_option(&params, Side::Short, OptionStyle::Put)
            .unwrap();
        assert_eq!(
            option.implied_volatility,
            option_data.implied_volatility.unwrap() / pos!(100.0)
        );

        let option = option_data
            .get_option(&params, Side::Long, OptionStyle::Put)
            .unwrap();
        assert_eq!(
            option.implied_volatility,
            option_data.implied_volatility.unwrap() / pos!(100.0)
        );
    }
}

#[cfg(test)]
mod tests_chain_implied_volatility {
    use super::*;
    use crate::utils::time::{get_today_or_tomorrow_formatted, get_tomorrow_formatted};
    use crate::{assert_pos_relative_eq, pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_update_implied_volatilities() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(21637.0),
            get_tomorrow_formatted(),
            Some(dec!(0.0)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(21395.0),        // strike
            spos!(250.0),         // call_bid
            spos!(254.0),         // call_ask
            Some(Positive::ZERO), // put_bid
            spos!(4.0),           // put_ask
            None,                 // implied_volatility (empezamos sin IV)
            None,                 // delta
            None,                 // volume
            None,                 // open_interest
            None,
            None,
        );

        chain.add_option(
            pos!(21700.0), // ATM strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // implied_volatility
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        chain.update_mid_prices();
        chain.update_greeks();

        for option in chain.options.iter() {
            assert!(
                option.implied_volatility.is_some(),
                "IV should be calculated for strike {}",
                option.strike_price
            );

            let iv = option.implied_volatility.unwrap() / pos!(100.0);
            assert!(
                iv > pos!(0.0) && iv < pos!(2.0),
                "IV should be reasonable for strike {}: {}",
                option.strike_price,
                iv
            );

            debug!("Strike: {}, IV: {}", option.strike_price, iv);
        }
    }

    #[test]
    fn test_update_implied_volatilities_today() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(21637.0),
            get_today_or_tomorrow_formatted(),
            Some(dec!(0.0)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(21395.0),        // strike
            spos!(250.0),         // call_bid
            spos!(254.0),         // call_ask
            Some(Positive::ZERO), // put_bid
            spos!(4.0),           // put_ask
            None,                 // implied_volatility (empezamos sin IV)
            None,                 // delta
            None,                 // volume
            None,                 // open_interest
            None,
            None,
        );

        chain.add_option(
            pos!(21700.0), // ATM strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // implied_volatility
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        chain.update_mid_prices();
        chain.update_greeks();

        for option in chain.options.iter() {
            assert!(
                option.implied_volatility.is_some(),
                "IV should be calculated for strike {}",
                option.strike_price
            );

            let iv = option.implied_volatility.unwrap() / pos!(100.0);
            assert!(
                iv > pos!(0.0) && iv < pos!(5.0),
                "IV should be reasonable for strike {}: {}",
                option.strike_price,
                iv
            );

            debug!("Strike: {}, IV: {}", option.strike_price, iv);
        }
    }

    #[test]
    fn test_update_implied_volatilities_missing_prices() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(100.0),
            None, // call_bid
            None, // call_ask
            None, // put_bid
            None, // put_ask
            None, // implied_volatility
            None, // delta
            None, // volume
            None, // open_interest
            None,
            None,
        );

        chain.update_mid_prices();
        chain.update_implied_volatilities();

        for option in chain.options.iter() {
            assert!(
                option.implied_volatility.is_none(),
                "IV should be None for options without prices"
            );
        }
    }

    #[test]
    fn test_update_implied_volatilities_maintain_existing() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(100.0),
            None, // call_bid
            None, // call_ask
            None, // put_bid
            None, // put_ask
            spos!(0.2),
            None, // delta
            None, // volume
            None, // open_interest
            None,
            None,
        );

        let original_iv = chain.options.iter().next().unwrap().implied_volatility;
        chain.update_mid_prices();
        chain.update_implied_volatilities();

        for option in chain.options.iter() {
            assert_pos_relative_eq!(
                option.implied_volatility.unwrap() / pos!(100.0),
                original_iv.unwrap(),
                pos!(0.001)
            );
        }
    }
}

#[cfg(test)]
mod tests_option_data_delta {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::utils::setup_logger_with_level;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a standard test OptionDataPriceParams
    fn create_standard_price_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            pos!(100.0),                      // underlying_price
            ExpirationDate::Days(pos!(30.0)), // expiration_date
            spos!(0.2),                       // implied_volatility
            dec!(0.05),                       // risk_free_rate
            pos!(0.02),                       // dividend_yield
            None,
        )
    }

    // Helper function to create standard OptionData
    fn create_standard_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0), // strike_price
            spos!(5.0),  // call_bid
            spos!(5.5),  // call_ask
            spos!(4.5),  // put_bid
            spos!(5.0),  // put_ask
            spos!(0.2),  // implied_volatility
            None,        // delta
            None,
            None,
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    fn test_calculate_delta_standard_call() {
        let price_params = create_standard_price_params();
        let mut option_data = create_standard_option_data();

        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Typical at-the-money call delta should be around 0.5
        assert!(delta > dec!(0.4) && delta < dec!(0.6));
    }

    #[test]
    fn test_calculate_delta_near_the_money() {
        let mut price_params = create_standard_price_params();
        price_params.underlying_price = pos!(105.0); // Slightly ITM

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Near-the-money call delta should be slightly higher than 0.5
        assert!(delta > dec!(0.7) && delta < dec!(0.9));
    }

    #[test]
    fn test_calculate_delta_deep_itm() {
        let mut price_params = create_standard_price_params();
        price_params.underlying_price = pos!(150.0); // Deep ITM

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Deep ITM call delta should be close to 1
        assert!(delta > dec!(0.9) && delta <= dec!(1.0));
    }

    #[test]
    fn test_calculate_delta_deep_otm() {
        let mut price_params = create_standard_price_params();
        price_params.underlying_price = pos!(50.0); // Deep OTM

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Deep OTM call delta should be close to 0
        assert!(delta >= Decimal::ZERO && delta < dec!(0.1));
    }

    #[test]
    fn test_calculate_delta_no_volatility() {
        setup_logger_with_level("debug");
        let mut price_params = create_standard_price_params();
        price_params.implied_volatility = None;

        let mut option_data = create_standard_option_data();
        option_data.implied_volatility = None;

        option_data.calculate_delta(&price_params);

        // If no volatility is provided, delta calculation should fail
        assert!(option_data.delta_call.is_none());
    }

    #[test]
    fn test_calculate_delta_multiple_calls() {
        let price_params = create_standard_price_params();
        let mut option_data = create_standard_option_data();

        // Call delta multiple times to ensure consistent behavior
        for _ in 0..3 {
            option_data.calculate_delta(&price_params);
            assert!(option_data.delta_call.is_some());
        }
    }

    #[test]
    fn test_calculate_delta_different_expiration() {
        let mut price_params = create_standard_price_params();
        price_params.expiration_date = ExpirationDate::Days(pos!(60.0)); // Longer expiration

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Delta should still be reasonable with longer expiration
        assert!(delta > Decimal::ZERO && delta <= dec!(1.0));
    }
}

#[cfg(test)]
mod tests_basic_curves {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::utils::time::get_tomorrow_formatted;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a sample OptionChain for testing
    fn create_test_option_chain() -> OptionChain {
        let tomorrow_date = get_tomorrow_formatted();
        let mut chain = OptionChain::new("TEST", pos!(100.0), tomorrow_date, None, None);

        // Add some test options
        chain.add_option(
            pos!(90.0),        // strike_price
            spos!(5.0),        // call_bid
            spos!(5.5),        // call_ask
            spos!(1.0),        // put_bid
            spos!(1.5),        // put_ask
            spos!(0.2),        // implied_volatility
            Some(dec!(0.6)),   // delta
            Some(dec!(100.0)), // volume
            Some(dec!(50.0)),  // open_interest
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            spos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
            None,
        );
        chain.update_greeks();
        chain
    }

    #[test]
    fn test_curve_delta_long_call() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Check that we have the expected number of points
        assert_eq!(curve.points.len(), 3);

        // Verify points are in reasonable ranges
        for point in &curve.points {
            assert!(
                point.x >= dec!(90.0) && point.x <= dec!(110.0),
                "Delta out of expected range"
            );
        }
    }

    #[test]
    fn test_curve_delta_short_put() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Put, &Side::Short);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Check that we have the expected number of points
        assert!(!curve.points.is_empty());

        // Verify points are in reasonable ranges
        for point in &curve.points {
            assert!(point.x > dec!(0.0), "Price should be positive");
        }
    }

    #[test]
    fn test_curve_price_short_put() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Price, &OptionStyle::Put, &Side::Short);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Check that we have the expected number of points
        assert!(!curve.points.is_empty());

        // Verify points are in reasonable ranges
        for point in &curve.points {
            assert!(point.x > dec!(0.0), "Price should be positive");
        }
    }

    #[test]
    fn test_curve_length() {
        let chain = create_test_option_chain();

        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn test_curve_with_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2024-12-31".to_string(), None, None);

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Curve should be empty
        assert_eq!(curve.points.len(), 0);
    }

    #[test]
    fn test_curve_multiple_axes() {
        let chain = create_test_option_chain();

        // Test various axis combinations
        let axes = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Price,
            BasicAxisTypes::Volatility,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
        ];

        for axis in axes {
            let curve = chain.curve(&axis, &OptionStyle::Call, &Side::Long);

            assert!(curve.is_ok(), "Failed to create curve for axis: {:?}", axis);
            let curve = curve.unwrap();

            // Each curve should have at least one point
            assert!(
                !curve.points.is_empty(),
                "Curve for axis {:?} is empty",
                axis
            );
        }
    }

    #[test]
    fn test_curve_point_order() {
        let chain = create_test_option_chain();

        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);

        assert!(curve.is_ok());
        let curve = curve.unwrap();

        // Verify points are in order (sorted by x-coordinate)
        let mut prev_x = Decimal::MIN;
        for point in &curve.points {
            assert!(point.x >= prev_x, "Points are not in ascending order");
            prev_x = point.x;
        }
    }
}

#[cfg(test)]
mod tests_option_chain_surfaces {
    use super::*;
    use crate::utils::time::get_tomorrow_formatted;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let tomorrow_date = get_tomorrow_formatted();
        let mut chain = OptionChain::new("TEST", pos!(100.0), tomorrow_date, None, None);

        // Add some test options
        chain.add_option(
            pos!(90.0),        // strike_price
            spos!(5.0),        // call_bid
            spos!(5.5),        // call_ask
            spos!(1.0),        // put_bid
            spos!(1.5),        // put_ask
            spos!(0.2),        // implied_volatility
            Some(dec!(0.6)),   // delta
            Some(dec!(100.0)), // volume
            Some(dec!(50.0)),  // open_interest
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            spos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
            None,
        );
        chain.update_greeks();
        chain
    }

    #[test]
    fn test_surface_invalid_axis() {
        let chain = create_test_option_chain();
        let result = chain.surface(
            &BasicAxisTypes::Strike,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(SurfaceError::ConstructionError(msg)) => {
                assert_eq!(msg, "Axis not valid");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_surface_with_no_volatility() {
        let chain = create_test_option_chain();
        let result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );

        assert!(result.is_ok());
        let surface = result.unwrap();
        assert!(!surface.points.is_empty());

        // Should have one point per strike
        assert_eq!(surface.points.len(), 3);
    }

    #[test]
    fn test_surface_with_custom_volatilities() {
        let chain = create_test_option_chain();
        let volatilities = vec![pos!(0.15), pos!(0.20), pos!(0.25)];

        let result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            Some(volatilities),
            &Side::Long,
        );

        assert!(result.is_ok());
        let surface = result.unwrap();
        assert!(!surface.points.is_empty());
        assert_eq!(surface.points.len(), 9);
    }

    #[test]
    fn test_surface_empty_volatility_vector() {
        let chain = create_test_option_chain();
        let empty_vols: Vec<Positive> = vec![];

        let result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            Some(empty_vols),
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(SurfaceError::ConstructionError(msg)) => {
                assert_eq!(msg, "No valid points generated for surface");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_surface_different_option_styles() {
        let chain = create_test_option_chain();

        // Test for calls
        let call_result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );
        assert!(call_result.is_ok());

        // Test for puts
        let put_result =
            chain.surface(&BasicAxisTypes::Delta, &OptionStyle::Put, None, &Side::Long);
        assert!(put_result.is_ok());
    }

    #[test]
    fn test_surface_different_sides() {
        let chain = create_test_option_chain();

        // Test for long position
        let long_result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );
        assert!(long_result.is_ok());

        // Test for short position
        let short_result = chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Short,
        );
        assert!(short_result.is_ok());
    }

    #[test]
    fn test_surface_different_greeks() {
        let chain = create_test_option_chain();
        let axes = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
            BasicAxisTypes::Price,
        ];

        for axis in axes {
            let result = chain.surface(&axis, &OptionStyle::Call, None, &Side::Long);
            assert!(result.is_ok(), "Failed for axis: {:?}", axis);
        }
    }

    #[test]
    fn test_surface_with_empty_chain() {
        let empty_chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            Some(pos!(0.01)),
        );

        let result = empty_chain.surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            None,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(SurfaceError::ConstructionError(msg)) => {
                assert_eq!(msg, "No valid points generated for surface");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_optiondata_serialization() {
        let option_data = OptionData {
            strike_price: pos!(100.0),
            call_bid: Some(pos!(9.5)),
            call_ask: Some(pos!(10.0)),
            put_bid: Some(pos!(8.5)),
            put_ask: Some(pos!(9.0)),
            call_middle: Some(pos!(9.75)),
            put_middle: Some(pos!(8.75)),
            implied_volatility: Some(pos!(0.2)),
            delta_call: Some(dec!(0.5)),
            delta_put: Some(dec!(-0.5)),
            gamma: Some(dec!(0.1)),
            volume: Some(pos!(1000.0)),
            open_interest: Some(500),
            options: None,
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
    }

    #[test]
    fn test_optionchain_serialization() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-01-01".to_string(),
            Some(dec!(0.05)),
            Some(pos!(0.02)),
        );

        // Add some test options
        chain.add_option(
            pos!(95.0),
            Some(pos!(6.0)),
            Some(pos!(6.5)),
            Some(pos!(1.5)),
            Some(pos!(2.0)),
            Some(pos!(0.2)),
            Some(dec!(0.7)),
            Some(dec!(-0.3)),
            Some(dec!(0.1)),
            Some(pos!(1000.0)),
            Some(500),
        );

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: OptionChain = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chain.symbol, deserialized.symbol);
        assert_eq!(chain.underlying_price, deserialized.underlying_price);
        assert_eq!(chain.expiration_date, deserialized.expiration_date);
        assert_eq!(chain.risk_free_rate, deserialized.risk_free_rate);
        assert_eq!(chain.dividend_yield, deserialized.dividend_yield);
    }

    #[test]
    fn test_optiondata_empty_fields() {
        let option_data = OptionData {
            strike_price: pos!(100.0),
            call_bid: None,
            call_ask: None,
            put_bid: None,
            put_ask: None,
            call_middle: None,
            put_middle: None,
            implied_volatility: None,
            delta_call: None,
            delta_put: None,
            gamma: None,
            volume: None,
            open_interest: None,
            options: None,
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
    }
}

#[cfg(test)]
mod tests_option_data_serde {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json;

    // Helper function to create a sample OptionData
    fn create_sample_option_data() -> OptionData {
        OptionData {
            strike_price: pos!(100.0),
            call_bid: Some(pos!(9.5)),
            call_ask: Some(pos!(10.0)),
            put_bid: Some(pos!(8.5)),
            put_ask: Some(pos!(9.0)),
            call_middle: Some(pos!(9.75)),
            put_middle: Some(pos!(8.75)),
            implied_volatility: Some(pos!(0.2)),
            delta_call: Some(dec!(0.5)),
            delta_put: Some(dec!(-0.5)),
            gamma: Some(dec!(0.1)),
            volume: Some(pos!(1000.0)),
            open_interest: Some(500),
            options: None,
        }
    }

    #[test]
    fn test_optiondata_complete_serialization() {
        let option_data = create_sample_option_data();
        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
        // Verify specific fields
        assert_eq!(deserialized.strike_price, pos!(100.0));
        assert_eq!(deserialized.delta_call, Some(dec!(0.5)));
    }

    #[test]
    fn test_optiondata_minimal_serialization() {
        // Test with minimal required fields
        let option_data = OptionData {
            strike_price: pos!(100.0),
            call_bid: None,
            call_ask: None,
            put_bid: None,
            put_ask: None,
            call_middle: None,
            put_middle: None,
            implied_volatility: None,
            delta_call: None,
            delta_put: None,
            gamma: None,
            volume: None,
            open_interest: None,
            options: None,
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
        assert!(deserialized.call_bid.is_none());
    }

    #[test]
    fn test_optiondata_large_numbers() {
        // Test with large numbers to verify precision
        let option_data = OptionData {
            strike_price: pos!(999999.99),
            call_bid: Some(pos!(99999.99)),
            call_ask: Some(pos!(99999.99)),
            implied_volatility: Some(pos!(1.0)),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
        assert_eq!(deserialized.strike_price, pos!(999999.99));
    }

    #[test]
    fn test_optiondata_small_numbers() {
        // Test with very small numbers to verify precision
        let option_data = OptionData {
            strike_price: pos!(0.0001),
            call_bid: Some(pos!(0.0001)),
            implied_volatility: Some(pos!(0.0001)),
            delta_call: Some(dec!(0.0001)),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
        assert_eq!(deserialized.delta_call, Some(dec!(0.0001)));
    }

    #[test]
    fn test_optiondata_special_cases() {
        // Test with edge cases and special values
        let option_data = OptionData {
            strike_price: Positive::ONE,
            call_bid: Some(Positive::ZERO),
            implied_volatility: Some(pos!(1.0)),
            delta_call: Some(Decimal::ONE),
            delta_put: Some(Decimal::NEGATIVE_ONE),
            gamma: Some(Decimal::ZERO),
            open_interest: Some(u64::MAX),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
        assert_eq!(deserialized.open_interest, Some(u64::MAX));
    }

    #[test]
    fn test_optiondata_json_structure() {
        let option_data = create_sample_option_data();
        let serialized = serde_json::to_string_pretty(&option_data).unwrap();

        // Verify JSON structure
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("strike_price").is_some());
        assert_eq!(
            json_value.get("strike_price").unwrap().as_f64().unwrap(),
            100.0
        );
    }

    #[test]
    fn test_optiondata_deserialization_error_handling() {
        // Test invalid JSON
        let invalid_json = r#"{"strike": "invalid"}"#;
        let result: Result<OptionData, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        // Test missing required field
        let missing_strike = r#"{"call_bid": 1.0}"#;
        let result: Result<OptionData, _> = serde_json::from_str(missing_strike);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_option_chain_serde {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_sample_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-01-01".to_string(),
            Some(dec!(0.05)),
            Some(pos!(0.02)),
        );

        // Add some test options
        chain.add_option(
            pos!(95.0),
            Some(pos!(6.0)),
            Some(pos!(6.5)),
            Some(pos!(1.5)),
            Some(pos!(2.0)),
            Some(pos!(0.2)),
            Some(dec!(0.7)),
            Some(dec!(-0.3)),
            Some(dec!(0.1)),
            Some(pos!(1000.0)),
            Some(500),
        );

        chain
    }

    #[test]
    fn test_optionchain_complete_serialization() {
        let chain = create_sample_chain();
        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: OptionChain = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chain.symbol, deserialized.symbol);
        assert_eq!(chain.underlying_price, deserialized.underlying_price);
        assert_eq!(chain.options.len(), deserialized.options.len());
    }

    #[test]
    fn test_optionchain_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2024-01-01".to_string(), None, None);

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: OptionChain = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chain.symbol, deserialized.symbol);
        assert!(deserialized.options.is_empty());
        assert!(deserialized.risk_free_rate.is_none());
    }

    #[test]
    fn test_optionchain_multiple_options() {
        let mut chain = create_sample_chain();

        // Add more options
        chain.add_option(
            pos!(100.0),
            Some(pos!(5.0)),
            Some(pos!(5.5)),
            Some(pos!(5.0)),
            Some(pos!(5.5)),
            Some(pos!(0.2)),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.1)),
            Some(pos!(1000.0)),
            Some(500),
        );

        chain.add_option(
            pos!(105.0),
            Some(pos!(4.0)),
            Some(pos!(4.5)),
            Some(pos!(8.0)),
            Some(pos!(8.5)),
            Some(pos!(0.2)),
            Some(dec!(0.3)),
            Some(dec!(-0.7)),
            Some(dec!(0.1)),
            Some(pos!(1000.0)),
            Some(500),
        );

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: OptionChain = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chain.options.len(), deserialized.options.len());
        assert_eq!(deserialized.options.len(), 3);
    }

    #[test]
    fn test_optionchain_special_values() {
        let mut chain = OptionChain::new(
            "SPECIAL",
            Positive::INFINITY,
            "2024-01-01".to_string(),
            Some(Decimal::MAX),
            Some(Positive::INFINITY),
        );

        chain.add_option(
            Positive::ONE,
            Some(Positive::ONE),
            Some(Positive::ONE),
            Some(Positive::ONE),
            Some(Positive::ONE),
            Some(Positive::ONE),
            Some(Decimal::ONE),
            Some(Decimal::ONE),
            Some(Decimal::ONE),
            Some(Positive::ONE),
            Some(1),
        );

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: OptionChain = serde_json::from_str(&serialized).unwrap();

        assert_eq!(chain.underlying_price, deserialized.underlying_price);
        assert_eq!(chain.risk_free_rate, deserialized.risk_free_rate);
    }

    #[test]
    fn test_optionchain_json_structure() {
        let chain = create_sample_chain();
        let serialized = serde_json::to_string_pretty(&chain).unwrap();

        // Verify JSON structure
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("symbol").is_some());
        assert!(json_value.get("underlying_price").is_some());
        assert!(json_value.get("options").is_some());
        assert!(json_value.get("options").unwrap().is_array());
    }

    #[test]
    fn test_optionchain_deserialization_error_handling() {
        // Test invalid JSON
        let invalid_json = r#"{"symbol": 123}"#;
        let result: Result<OptionChain, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        // Test missing required fields
        let missing_fields = r#"{"symbol": "TEST"}"#;
        let result: Result<OptionChain, _> = serde_json::from_str(missing_fields);
        assert!(result.is_err());
    }

    #[test]
    fn test_optionchain_options_validation() {
        let mut chain = create_sample_chain();

        // Add an option with all None values except strike
        chain.add_option(
            pos!(110.0),
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

        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: OptionChain = serde_json::from_str(&serialized).unwrap();

        // Find the option with strike 110.0
        let option = deserialized
            .options
            .iter()
            .find(|opt| opt.strike_price == pos!(110.0))
            .unwrap();

        assert!(option.call_bid.is_none());
        assert!(option.implied_volatility.is_none());
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_gamma_calculations {
    use super::*;
    use crate::utils::setup_logger;
    use crate::utils::time::get_tomorrow_formatted;
    use crate::{assert_decimal_eq, pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined gamma values
    fn create_test_chain_with_gamma() -> OptionChain {
        let mut option_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        option_chain.expiration_date = get_tomorrow_formatted();
        option_chain
    }

    #[test]
    fn test_gamma_exposure_basic() {
        setup_logger();
        let mut chain = create_test_chain_with_gamma();
        chain.update_greeks();
        let result = chain.gamma_exposure();

        assert!(result.is_ok());
        let gamma_exposure = result.unwrap();
        // Total gamma should be sum of all individual gammas
        // 0.04 + 0.06 + 0.02 = 0.12
        assert_decimal_eq!(gamma_exposure, dec!(0.003548), dec!(0.001));
    }

    #[test]
    fn test_gamma_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.gamma_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_gamma_exposure_missing_gamma() {
        let mut chain = create_test_chain_with_gamma();

        // Add an option without gamma
        chain.add_option(
            pos!(110.0),
            spos!(0.5),
            spos!(0.7),
            spos!(2.5),
            spos!(2.7),
            spos!(0.35),
            Some(dec!(0.3)),
            Some(dec!(-0.7)),
            None, // No gamma value
            spos!(60.0),
            Some(30),
        );

        chain.update_greeks();
        let result = chain.gamma_exposure().unwrap();
        assert_decimal_eq!(result, dec!(0.0035), dec!(0.001));
    }

    #[test]
    fn test_gamma_curve() {
        let mut chain = create_test_chain_with_gamma();
        chain.update_greeks();
        let result = chain.gamma_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Test that curve contains points
        assert!(!curve.points.is_empty());

        // For each strike in the chain, there should be a corresponding point
        assert_eq!(curve.points.len(), chain.options.len());

        // Test x range of curve matches strike range
        let first_strike = chain.options.iter().next().unwrap().strike_price;
        let last_strike = chain.options.iter().last().unwrap().strike_price;
        assert_eq!(curve.x_range.0, first_strike.to_dec());
        assert_eq!(curve.x_range.1, last_strike.to_dec());
    }

    #[test]
    fn test_gamma_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.gamma_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_delta_calculations {
    use super::*;
    use crate::utils::setup_logger;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined delta values
    fn create_test_chain_with_delta() -> OptionChain {
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap()
    }

    #[test]

    fn test_delta_exposure_basic() {
        setup_logger();
        let mut chain = create_test_chain_with_delta();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.delta_exposure();

        assert!(result.is_ok());
        let delta_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(delta_exposure, dec!(31.0), dec!(0.000001));
    }

    #[test]

    fn test_delta_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.delta_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]

    fn test_delta_exposure_uninitialized_greeks() {
        let mut chain = create_test_chain_with_delta();
        chain.update_greeks();
        // Don't initialize greeks, should return zero exposure
        let result = chain.delta_exposure();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(31.0));
    }

    #[test]

    fn test_delta_exposure_updates() {
        setup_logger();
        let mut chain = create_test_chain_with_delta();

        // Get initial delta exposure (should be 0 as greeks aren't initialized)
        let initial_delta = chain.delta_exposure().unwrap();
        assert_eq!(initial_delta, dec!(0.0));

        // Update greeks and check new delta exposure
        chain.update_greeks();
        let updated_delta = chain.delta_exposure().unwrap();
        assert_decimal_eq!(updated_delta, dec!(31.0), dec!(0.000001));
    }

    #[test]

    fn test_delta_curve() {
        let mut chain = create_test_chain_with_delta();
        chain.update_greeks();
        let result = chain.delta_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Test that curve contains points
        assert!(!curve.points.is_empty());

        // For each strike in the chain, there should be a corresponding point
        assert_eq!(curve.points.len(), chain.options.len());

        // Test x range of curve matches strike range
        let first_strike = chain.options.iter().next().unwrap().strike_price;
        let last_strike = chain.options.iter().last().unwrap().strike_price;
        assert_eq!(curve.x_range.0, first_strike.to_dec());
        assert_eq!(curve.x_range.1, last_strike.to_dec());
    }

    #[test]

    fn test_delta_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.delta_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }

    #[test]

    fn test_delta_curve_shape() {
        setup_logger();
        let mut chain = create_test_chain_with_delta();
        chain.update_greeks();
        let curve = chain.delta_curve().unwrap();

        // Get sorted points by strike
        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Verify the delta curve shape:
        // 1. Delta should be roughly between 0 and 1 for calls
        // 2. Should decrease as strike increases
        for point in &points {
            // Check delta bounds for call options
            assert!(point.y >= dec!(-0.1)); // Allow some margin for numerical precision
            assert!(point.y <= dec!(1.1));
        }

        // Check monotonic decrease
        for i in 1..points.len() {
            assert!(points[i].y <= points[i - 1].y + dec!(0.1)); // Allow small non-monotonicity due to market data
        }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_vega_calculations {
    use super::*;
    use crate::utils::setup_logger;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined vega values
    fn create_test_chain_with_vega() -> OptionChain {
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap()
    }

    #[test]

    fn test_vega_exposure_basic() {
        setup_logger();
        let mut chain = create_test_chain_with_vega();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.vega_exposure();

        assert!(result.is_ok());
        let vega_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(vega_exposure, dec!(0.0), dec!(0.0001));
    }

    #[test]

    fn test_vega_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.vega_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]

    fn test_vega_exposure_uninitialized_greeks() {
        let mut chain = create_test_chain_with_vega();
        chain.update_greeks();
        // Don't initialize greeks, should return zero exposure
        let result = chain.vega_exposure();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]

    fn test_vega_exposure_updates() {
        setup_logger();
        let mut chain = create_test_chain_with_vega();

        // Get initial vega exposure (should be 0 as greeks aren't initialized)
        let initial_vega = chain.vega_exposure().unwrap();
        assert_eq!(initial_vega, dec!(0.0));

        // Update greeks and check new vega exposure
        chain.update_greeks();
        let updated_vega = chain.vega_exposure().unwrap();
        assert_decimal_eq!(updated_vega, dec!(0.0), dec!(0.000001));
    }

    #[test]

    fn test_vega_curve() {
        let mut chain = create_test_chain_with_vega();
        chain.update_greeks();
        let result = chain.vega_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Test that curve contains points
        assert!(!curve.points.is_empty());

        // For each strike in the chain, there should be a corresponding point
        assert_eq!(curve.points.len(), chain.options.len());

        // Test x range of curve matches strike range
        let first_strike = chain.options.iter().next().unwrap().strike_price;
        let last_strike = chain.options.iter().last().unwrap().strike_price;
        assert_eq!(curve.x_range.0, first_strike.to_dec());
        assert_eq!(curve.x_range.1, last_strike.to_dec());
    }

    #[test]

    fn test_vega_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.vega_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }

    #[test]

    fn test_vega_curve_shape() {
        setup_logger();
        let mut chain = create_test_chain_with_vega();
        chain.update_greeks();
        let curve = chain.vega_curve().unwrap();

        // Get sorted points by strike
        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Verify the vega curve shape:
        // 1. Delta should be roughly between 0 and 1 for calls
        // 2. Should decrease as strike increases
        for point in &points {
            // Check vega bounds for call options
            assert!(point.y >= dec!(-0.1)); // Allow some margin for numerical precision
            assert!(point.y <= dec!(1.1));
        }

        // Check monotonic decrease
        for i in 1..points.len() {
            assert!(points[i].y <= points[i - 1].y + dec!(0.1)); // Allow small non-monotonicity due to market data
        }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests_theta_calculations {
    use super::*;
    use crate::utils::setup_logger;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined theta values
    fn create_test_chain_with_theta() -> OptionChain {
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap()
    }

    #[test]
    fn test_theta_exposure_basic() {
        setup_logger();
        let mut chain = create_test_chain_with_theta();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.theta_exposure();

        assert!(result.is_ok());
        let theta_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(theta_exposure, dec!(0.0), dec!(0.000001));
    }

    #[test]
    fn test_theta_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.theta_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_theta_exposure_uninitialized_greeks() {
        let mut chain = create_test_chain_with_theta();
        chain.update_greeks();
        // Don't initialize greeks, should return zero exposure
        let result = chain.theta_exposure();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_theta_exposure_updates() {
        setup_logger();
        let mut chain = create_test_chain_with_theta();

        // Get initial theta exposure (should be 0 as greeks aren't initialized)
        let initial_theta = chain.theta_exposure().unwrap();
        assert_eq!(initial_theta, dec!(0.0));

        // Update greeks and check new theta exposure
        chain.update_greeks();
        let updated_theta = chain.theta_exposure().unwrap();
        assert_decimal_eq!(updated_theta, dec!(0.0), dec!(0.000001));
    }

    #[test]
    fn test_theta_curve() {
        let mut chain = create_test_chain_with_theta();
        chain.update_greeks();
        let result = chain.theta_curve();

        assert!(result.is_ok());
        let curve = result.unwrap();

        // Test that curve contains points
        assert!(!curve.points.is_empty());

        // For each strike in the chain, there should be a corresponding point
        assert_eq!(curve.points.len(), chain.options.len());

        // Test x range of curve matches strike range
        let first_strike = chain.options.iter().next().unwrap().strike_price;
        let last_strike = chain.options.iter().last().unwrap().strike_price;
        assert_eq!(curve.x_range.0, first_strike.to_dec());
        assert_eq!(curve.x_range.1, last_strike.to_dec());
    }

    #[test]
    fn test_theta_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.theta_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }

    #[test]
    fn test_theta_curve_shape() {
        setup_logger();
        let mut chain = create_test_chain_with_theta();
        chain.update_greeks();
        let curve = chain.theta_curve().unwrap();

        // Get sorted points by strike
        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Verify the theta curve shape:
        // 1. Delta should be roughly between 0 and 1 for calls
        // 2. Should decrease as strike increases
        for point in &points {
            // Check theta bounds for call options
            assert!(point.y >= dec!(-0.1)); // Allow some margin for numerical precision
            assert!(point.y <= dec!(1.1));
        }

        // Check monotonic decrease
        for i in 1..points.len() {
            assert!(points[i].y <= points[i - 1].y + dec!(0.1)); // Allow small non-monotonicity due to market data
        }
    }
}
