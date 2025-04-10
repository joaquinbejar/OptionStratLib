/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use crate::chains::utils::{OptionDataPriceParams, default_empty_string, empty_string_round_to_2};
use crate::chains::{DeltasInStrike, FourOptions, OptionsInStrike};
use crate::error::ChainError;
use crate::greeks::{delta, gamma};
use crate::strategies::FindOptimalSide;
use crate::{OptionStyle, OptionType, Options, Positive, Side, pos};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

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
    pub(super) delta_call: Option<Decimal>,

    /// The delta of the put option, measuring price sensitivity to underlying changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) delta_put: Option<Decimal>,

    /// The gamma of the option, measuring the rate of change in delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) gamma: Option<Decimal>,

    /// The trading volume of the option, indicating market activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) volume: Option<Positive>,

    /// The open interest, representing the number of outstanding contracts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) open_interest: Option<u64>,

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

    /// Checks if any of the bid or ask prices for call or put options are None.
    ///
    /// This function returns `true` if any of the `call_bid`, `call_ask`, `put_bid`, or `put_ask`
    /// fields of the `OptionData` struct are `None`, indicating missing price information.
    /// It returns `false` if all four fields have valid price data.
    ///
    pub fn some_price_is_none(&self) -> bool {
        self.call_bid.is_none()
            || self.call_ask.is_none()
            || self.put_bid.is_none()
            || self.put_ask.is_none()
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
    pub(super) fn get_option(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
    ) -> Result<Options, ChainError> {
        let implied_volatility = match price_params.implied_volatility {
            Some(iv) => iv,
            None => match self.implied_volatility {
                Some(iv) => {
                    assert!(iv <= Positive::ONE, "Implied volatility must be <= 1");
                    iv
                }
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
    pub(super) fn get_options_in_strike(
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
        trace!("Options: {:?}", options);
        match options.long_call.calculate_price_black_scholes() {
            Ok(call_ask) => {
                trace!("Call Ask: {}", call_ask);
                self.call_ask = Some(Positive(call_ask.abs()));
            }
            Err(_) => self.call_ask = None,
        }

        match options.short_call.calculate_price_black_scholes() {
            Ok(call_bid) => {
                trace!("Call Bid: {}", call_bid);
                self.call_bid = Some(Positive(call_bid.abs()));
            }
            Err(_) => self.call_bid = None,
        }

        match options.long_put.calculate_price_black_scholes() {
            Ok(put_ask) => {
                trace!("Put Ask: {}", put_ask);
                self.put_ask = Some(Positive(put_ask.abs()));
            }
            Err(_) => self.put_ask = None,
        }

        match options.short_put.calculate_price_black_scholes() {
            Ok(put_bid) => {
                trace!("Put Bid: {}", put_bid);
                self.put_bid = Some(Positive(put_bid.abs()));
            }
            Err(_) => self.put_bid = None,
        }

        trace!(
            "Prices: {:?} {:?} {:?} {:?}",
            self.call_ask, self.call_bid, self.put_ask, self.put_bid
        );
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
        side: &FindOptimalSide, // Note: now mutable
    ) -> bool {
        match side {
            FindOptimalSide::Upper => self.strike_price >= underlying_price,
            FindOptimalSide::Lower => self.strike_price <= underlying_price,
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                self.strike_price >= *start && self.strike_price <= *end
            }
            FindOptimalSide::Deltable(_threshold) => true,
            FindOptimalSide::Center => {
                panic!("Center should be managed by the strategy");
            }
            FindOptimalSide::DeltaRange(min, max) => {
                (self.delta_put.is_some()
                    && self.delta_put.unwrap() >= *min
                    && self.delta_put.unwrap() <= *max)
                    || (self.delta_call.is_some()
                        && self.delta_call.unwrap() >= *min
                        && self.delta_call.unwrap() <= *max)
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
            trace!(
                "call_middle {:?} put_middle {:?}",
                self.call_middle, self.put_middle
            );
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
                    assert!(
                        iv <= Positive::ONE,
                        "Volatility should be <= 1 and is: {}",
                        iv
                    );
                    self.implied_volatility = Some(iv);
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
                    self.implied_volatility = Some(iv);
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

    /// Checks and corrects implied volatility if it's represented as a percentage greater than 1.0.
    ///
    /// This function checks if the `implied_volatility` field is present. If it is and its value
    /// is greater than 1.0, the function assumes it's represented as a percentage and divides it
    /// by 100.0 to convert it to a decimal value. This ensures that implied volatility is stored
    /// in the correct format, preventing potential misinterpretations and calculation errors.
    pub(super) fn check_and_convert_implied_volatility(&mut self) {
        if let Some(iv) = self.implied_volatility {
            if iv > pos!(1.0) {
                self.implied_volatility = Some(iv / Positive::HUNDRED);
            }
        }
    }

    /// Creates a complete set of four standard option contracts based on specified pricing parameters.
    ///
    /// This method constructs four option contracts (long call, short call, long put, short put)
    /// with identical strike prices and expiration dates, all based on the same underlying asset.
    /// The resulting options are stored within the `OptionData` instance for further analysis
    /// or trading strategy evaluation.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to `OptionDataPriceParams` containing essential pricing inputs
    ///   including underlying price, expiration date, risk-free rate, dividend yield, and optionally
    ///   the underlying symbol and implied volatility.
    ///
    /// # Returns
    ///
    /// * `Result<(), ChainError>` - Returns `Ok(())` if option creation succeeds, or a `ChainError`
    ///   if any issues occur during creation.
    ///
    pub fn create_options(
        &mut self,
        price_params: &OptionDataPriceParams,
    ) -> Result<(), ChainError> {
        let symbol = if let Some(underlying_symbol) = price_params.underlying_symbol.clone() {
            underlying_symbol
        } else {
            "NA".to_string()
        };
        let long_call = Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date,
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Call,
            price_params.dividend_yield,
            None,
        ));
        let short_call = Arc::new(Options::new(
            OptionType::European,
            Side::Short,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date,
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Call,
            price_params.dividend_yield,
            None,
        ));
        let long_put = Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date,
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Put,
            price_params.dividend_yield,
            None,
        ));
        let short_put = Arc::new(Options::new(
            OptionType::European,
            Side::Short,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date,
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Put,
            price_params.dividend_yield,
            None,
        ));
        self.options = Some(Box::new(FourOptions {
            long_call,
            short_call,
            long_put,
            short_put,
        }));
        Ok(())
    }

    /// Returns a tuple containing the current delta values for both call and put options.
    ///
    /// This method provides access to the option's delta values, which measure the rate of change
    /// of the option price with respect to changes in the underlying asset price. Delta values
    /// typically range from -1 to 1 and are a key metric for understanding option price sensitivity.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * First element: `Option<Decimal>` - The delta value for the call option. May be `None` if
    ///   the delta value is not available or could not be calculated.
    /// * Second element: `Option<Decimal>` - The delta value for the put option. May be `None` if
    ///   the delta value is not available or could not be calculated.
    ///
    pub fn current_deltas(&self) -> (Option<Decimal>, Option<Decimal>) {
        (self.delta_call, self.delta_put)
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
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<6}{:<7} {:.3}{:<4} {:.3}{:<5} {:.4}{:<8} {:<10} {:<10}",
            self.strike_price.to_string(),
            empty_string_round_to_2(self.call_bid),
            empty_string_round_to_2(self.call_ask),
            empty_string_round_to_2(self.call_middle),
            empty_string_round_to_2(self.put_bid),
            empty_string_round_to_2(self.put_ask),
            empty_string_round_to_2(self.put_middle),
            self.implied_volatility
                .unwrap_or(Positive::ZERO)
                .format_fixed_places(3),
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

#[cfg(test)]
mod optiondata_coverage_tests {
    use super::*;
    use crate::utils::logger::setup_logger;
    use crate::{ExpirationDate, spos};
    use rust_decimal_macros::dec;

    // Helper function to create test option data
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
    fn test_current_deltas() {
        let option_data = create_test_option_data();

        // Test current deltas
        let (call_delta, put_delta) = option_data.current_deltas();

        assert!(call_delta.is_some());
        assert!(put_delta.is_some());
        assert_eq!(call_delta.unwrap(), dec!(-0.3));
        assert_eq!(put_delta.unwrap(), dec!(0.7));
    }

    #[test]
    fn test_calculate_prices_with_refresh() {
        setup_logger();
        let mut option_data = create_test_option_data();

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.25), // Different IV to force recalculation
            dec!(0.05),
            pos!(0.02),
            Some("TEST".to_string()),
        );

        // Calculate prices with refresh flag set to true
        let result = option_data.calculate_prices(&price_params, true);
        assert!(result.is_ok());

        // Check that prices were updated
        assert!(option_data.call_bid.is_some());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.put_bid.is_some());
        assert!(option_data.put_ask.is_some());

        // Check that mid prices were set
        assert!(option_data.call_middle.is_some());
        assert!(option_data.put_middle.is_some());
    }

    #[test]
    fn test_apply_spread() {
        let mut option_data = create_test_option_data();

        // Record original values
        let original_call_bid = option_data.call_bid;
        let original_call_ask = option_data.call_ask;

        // Apply a spread
        option_data.apply_spread(pos!(0.5), 2);

        // Check that values were updated
        assert_ne!(option_data.call_bid, original_call_bid);
        assert_ne!(option_data.call_ask, original_call_ask);

        // Test with a spread that would make bid negative (should set to None)
        let mut option_data = create_test_option_data();
        option_data.call_bid = spos!(0.1);
        option_data.apply_spread(pos!(1.0), 2);

        // Bid should be None as it would be negative
        assert_eq!(option_data.call_bid, None);
    }

    #[test]
    fn test_calculate_gamma_no_implied_volatility() {
        setup_logger();
        let mut option_data = create_test_option_data();

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            Some("TEST".to_string()),
        );

        // Calculate gamma
        option_data.calculate_gamma(&price_params);

        // Check that gamma was set
        assert!(option_data.gamma.is_some());

        // Test with missing implied volatility
        let mut option_data_no_iv = create_test_option_data();
        option_data_no_iv.implied_volatility = None;

        option_data_no_iv.calculate_gamma(&price_params);
    }

    // Test for lines 1076-1077
    #[test]
    fn test_get_deltas() {
        let option_data = create_test_option_data();

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            Some("TEST".to_string()),
        );

        // Get deltas
        let result = option_data.get_deltas(&price_params);
        assert!(result.is_ok());

        let deltas = result.unwrap();

        // Check that all deltas are present
        assert!(deltas.long_call != dec!(0.0));
        assert!(deltas.short_call != dec!(0.0));
        assert!(deltas.long_put != dec!(0.0));
        assert!(deltas.short_put != dec!(0.0));
    }
}
