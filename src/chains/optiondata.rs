/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use crate::chains::utils::{OptionDataPriceParams, default_empty_string, empty_string_round_to_2};
use crate::chains::{DeltasInStrike, OptionsInStrike};
use crate::error::ChainError;
use crate::greeks::{delta, gamma};
use crate::model::Position;
use crate::strategies::{BasicAble, FindOptimalSide};
use crate::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side, pos};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::fmt;
use tracing::{debug, error, trace};

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

    pub(crate) implied_volatility: Positive,

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

    #[serde(skip)]
    pub symbol: Option<Box<String>>,
    #[serde(skip)]
    pub expiration_date: Option<ExpirationDate>,
    #[serde(skip)]
    pub underlying_price: Option<Box<Positive>>,
    #[serde(skip)]
    pub risk_free_rate: Option<Decimal>,
    #[serde(skip)]
    pub dividend_yield: Option<Positive>,

    #[serde(skip)]
    pub extra_fields: Option<Value>,
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
        implied_volatility: Positive,
        delta_call: Option<Decimal>,
        delta_put: Option<Decimal>,
        gamma: Option<Decimal>,
        volume: Option<Positive>,
        open_interest: Option<u64>,
        symbol: Option<Box<String>>,
        expiration_date: Option<ExpirationDate>,
        underlying_price: Option<Box<Positive>>,
        risk_free_rate: Option<Decimal>,
        dividend_yield: Option<Positive>,
        extra_fields: Option<Value>,
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
            symbol,
            expiration_date,
            underlying_price,
            risk_free_rate,
            dividend_yield,
            extra_fields,
        }
    }

    /// Retrieves the implied volatility of the underlying asset or option.
    ///
    /// # Returns
    ///
    /// An `Option<Positive>` where:
    /// - `Some(Positive)` contains the implied volatility if it is available.
    /// - `None` if the implied volatility is not set or available.
    ///
    /// # Notes
    ///
    /// The implied volatility represents the market's forecast of a likely movement
    /// in an asset's price and is often used in option pricing models.
    ///
    /// Ensure that the `Positive` type enforces constraints to prevent invalid values
    /// such as negative volatility.
    pub fn get_volatility(&self) -> Positive {
        self.implied_volatility
    }

    pub fn set_volatility(&mut self, volatility: &Positive) {
        self.implied_volatility = *volatility;
    }

    pub fn set_extra_params(&mut self, params: OptionDataPriceParams) {
        if let Some(symbol) = params.underlying_symbol {
            self.symbol = Some(symbol);
        };

        if let Some(expiration_date) = params.expiration_date {
            self.expiration_date = Some(expiration_date);
        };

        if let Some(underlying_price) = params.underlying_price {
            self.underlying_price = Some(underlying_price);
        };

        if let Some(risk_free_rate) = params.risk_free_rate {
            self.risk_free_rate = Some(risk_free_rate);
        };

        if let Some(dividend_yield) = params.dividend_yield {
            self.dividend_yield = Some(dividend_yield);
        };
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
        if !self.valid_call() && !self.valid_put() {
            error!("Error: No valid prices for call or put options");
            return false;
        }
        true
    }

    /// Retrieves the strike price.
    ///
    /// This method returns the strike price associated with the object. The strike price
    /// is represented as a [`Positive`] value, ensuring that it is always greater than zero.
    ///
    /// # Returns
    /// * [`Positive`] - The strike price of the object.
    ///
    /// # Notes
    /// The method assumes that the strike price has been properly initialized and is
    /// a valid positive number.
    ///
    /// [`Positive`]: struct.Positive.html
    pub fn strike(&self) -> Positive {
        self.strike_price
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
        self.strike_price > Positive::ZERO && self.call_bid.is_some() && self.call_ask.is_some()
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
        self.strike_price > Positive::ZERO && self.put_bid.is_some() && self.put_ask.is_some()
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
        side: Side,
        option_style: OptionStyle,
    ) -> Result<Options, ChainError> {
        let mut option = Options::from(self);
        option.side = side;
        option.option_style = option_style;
        Ok(option)
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
        side: Side,
        option_style: OptionStyle,
        initial_iv: Positive,
    ) -> Result<Options, ChainError> {
        let mut option = self.get_option(side, option_style)?;
        option.set_implied_volatility(&initial_iv);
        Ok(option)
    }

    /// Retrieves a `Position` based on the provided parameters, calculating the option premium using the Black-Scholes model.
    ///
    /// This method fetches an option based on the provided parameters, calculates its theoretical
    /// premium using the Black-Scholes model, and constructs a `Position` struct containing the option
    /// details, premium, opening date, and associated fees.
    ///
    /// # Arguments
    ///
    /// * `price_params` - Option pricing parameters encapsulated in `OptionDataPriceParams`.
    /// * `side` - The side of the option, either `Side::Long` or `Side::Short`.
    /// * `option_style` - The style of the option, either `OptionStyle::Call` or `OptionStyle::Put`.
    /// * `date` - An optional `DateTime<Utc>` representing the opening date of the position.
    ///   If `None`, the current UTC timestamp is used.
    /// * `open_fee` - An optional `Positive` value representing the opening fee for the position.
    ///   If `None`, defaults to `Positive::ZERO`.
    /// * `close_fee` - An optional `Positive` value representing the closing fee for the position.
    ///   If `None`, defaults to `Positive::ZERO`.
    ///
    /// # Returns
    ///
    /// * `Result<Position, ChainError>` - A `Result` containing the constructed `Position` on success,
    ///   or a `ChainError` if any error occurred during option retrieval or premium calculation.
    ///
    /// # Errors
    ///
    /// This method can return a `ChainError` if:
    ///
    /// * The underlying option cannot be retrieved based on the provided parameters.
    /// * The Black-Scholes model fails to calculate a valid option premium.
    pub fn get_position(
        &self,
        side: Side,
        option_style: OptionStyle,
        date: Option<DateTime<Utc>>,
        open_fee: Option<Positive>,
        close_fee: Option<Positive>,
    ) -> Result<Position, ChainError> {
        let option = self.get_option(side, option_style)?;
        let premium = match (side, option_style) {
            (Side::Long, OptionStyle::Call) => self.get_call_buy_price(),
            (Side::Short, OptionStyle::Call) => self.get_call_sell_price(),
            (Side::Long, OptionStyle::Put) => self.get_put_buy_price(),
            (Side::Short, OptionStyle::Put) => self.get_put_sell_price(),
        };
        let premium = match premium {
            Some(premium) => premium,
            None => {
                let premium_dec = option.calculate_price_black_scholes()?.abs();
                Positive::from(premium_dec)
            }
        };
        let date = if let Some(date) = date {
            date
        } else {
            Utc::now()
        };
        let open_fee = if let Some(open_fee) = open_fee {
            open_fee
        } else {
            Positive::ZERO
        };
        let close_fee = if let Some(close_fee) = close_fee {
            close_fee
        } else {
            Positive::ZERO
        };

        Ok(Position::new(option, premium, date, open_fee, close_fee))
    }

    pub(super) fn get_options_in_strike(&self) -> Result<OptionsInStrike, ChainError> {
        let mut option: Options = self.get_option(Side::Long, OptionStyle::Call)?;
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
    pub fn calculate_prices(&mut self, spread: Option<Positive>) -> Result<(), ChainError> {
        let call_option = self.get_option(Side::Long, OptionStyle::Call)?;
        let price = call_option.calculate_price_black_scholes()?;
        self.call_middle = Some(price.into());
        let put_option = self.get_option(Side::Long, OptionStyle::Put)?;
        let prince = put_option.calculate_price_black_scholes()?;
        self.put_middle = Some(price.into());

        self.call_ask = self.call_middle;
        self.call_bid = self.call_middle;
        self.put_ask = self.put_middle;
        self.put_bid = self.put_middle;

        if spread.is_some() {
            self.apply_spread(spread.unwrap(), 2);
        }
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
    pub fn apply_spread(&mut self, spread: Positive, decimal_places: u32) {
        let half_spread: Decimal = (spread / Positive::TWO).into();

        if self.call_middle.is_some() {
            self.call_ask =
                Some((self.call_middle.unwrap() + half_spread).round_to(decimal_places));
            self.call_bid =
                Some((self.call_middle.unwrap() - half_spread).round_to(decimal_places));
        } else {
            assert!(self.call_ask.is_some() && self.call_bid.is_some());
            self.call_middle = Some(
                ((self.call_ask.unwrap() + self.call_bid.unwrap()) / Positive::TWO)
                    .round_to(decimal_places),
            );
        };

        if self.put_middle.is_some() {
            self.put_ask = Some((self.put_middle.unwrap() + half_spread).round_to(decimal_places));
            self.put_bid = Some((self.put_middle.unwrap() - half_spread).round_to(decimal_places));
        } else {
            assert!(self.put_ask.is_some() && self.put_bid.is_some());
            self.put_middle = Some(
                ((self.put_ask.unwrap() + self.put_bid.unwrap()) / Positive::TWO)
                    .round_to(decimal_places),
            );
        }
    }

    pub fn calculate_delta(&mut self) {
        let option: Options = match self.get_option(Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(e) => {
                debug!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta_call = Some(d),
            Err(e) => {
                debug!("Delta calculation failed: {}", e);
                self.delta_call = None;
            }
        }

        let option: Options = match self.get_option(Side::Long, OptionStyle::Put) {
            Ok(option) => option,
            Err(e) => {
                debug!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta_put = Some(d),
            Err(e) => {
                debug!("Delta calculation failed: {}", e);
                self.delta_put = None;
            }
        }
    }

    pub fn calculate_gamma(&mut self) {
        let option: Options = match self.get_option(Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(e) => {
                debug!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };
        match gamma(&option) {
            Ok(d) => self.gamma = Some(d),
            Err(e) => {
                debug!("Gamma calculation failed: {}", e);
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
    pub fn get_deltas(&self) -> Result<DeltasInStrike, ChainError> {
        let options_in_strike = self.get_options_in_strike()?;
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
        underlying_price: &Positive,
        side: &FindOptimalSide,
    ) -> bool {
        match side {
            FindOptimalSide::Upper => &self.strike_price >= underlying_price,
            FindOptimalSide::Lower => &self.strike_price <= underlying_price,
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
        assert!(self.call_ask.is_some() && self.call_bid.is_some());
        self.call_middle = match (self.call_bid, self.call_ask) {
            (Some(bid), Some(ask)) => Some(((bid + ask) / pos!(2.0)).round_to(4)),
            _ => None,
        };
        assert!(self.put_ask.is_some() && self.put_bid.is_some());
        self.put_middle = match (self.put_bid, self.put_ask) {
            (Some(bid), Some(ask)) => Some(((bid + ask) / pos!(2.0)).round_to(4)),
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

    /// Checks and corrects implied volatility if it's represented as a percentage greater than 1.0.
    ///
    /// This function checks if the `implied_volatility` field is present. If it is and its value
    /// is greater than 1.0, the function assumes it's represented as a percentage and divides it
    /// by 100.0 to convert it to a decimal value. This ensures that implied volatility is stored
    /// in the correct format, preventing potential misinterpretations and calculation errors.
    pub(super) fn check_and_convert_implied_volatility(&mut self) {
        if self.implied_volatility > pos!(1.0) {
            self.implied_volatility = self.implied_volatility / Positive::HUNDRED;
        }
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

    /// Returns the current gamma value.
    ///
    /// This function retrieves the optional `gamma` field of the struct.
    /// If the `gamma` field has been set, it returns a `Some(Decimal)` value;
    /// otherwise, it returns `None`.
    ///
    /// # Returns
    ///
    /// * `Option<Decimal>` - The current gamma value wrapped in `Some` if it exists,
    ///   or `None` if the gamma value is not set.
    ///
    pub fn current_gamma(&self) -> Option<Decimal> {
        self.gamma
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
            implied_volatility: Positive::ZERO,
            delta_call: None,
            delta_put: None,
            gamma: None,
            volume: None,
            open_interest: None,
            symbol: None,
            expiration_date: None,
            underlying_price: None,
            risk_free_rate: None,
            dividend_yield: None,
            extra_fields: None,
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
            self.implied_volatility.format_fixed_places(3),
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
            pos!(0.2),
            Some(dec!(-0.3)),
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0),
            Some(500),
            None,
            None,
            None,
            None,
            None,
            None,
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
        let mut option_data = create_test_option_data();
        option_data.set_volatility(&pos!(0.25));

        // Calculate prices with refresh flag set to true
        let result = option_data.calculate_prices(None);
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
        let mut option_data = create_test_option_data();
        option_data.set_volatility(&pos!(0.2));

        // Calculate gamma
        option_data.calculate_gamma();

        // Check that gamma was set
        assert!(option_data.gamma.is_some());
    }

    // Test for lines 1076-1077
    #[test]
    fn test_get_deltas() {
        let option_data = create_test_option_data();

        // Get deltas
        let result = option_data.get_deltas();
        assert!(result.is_ok());

        let deltas = result.unwrap();

        // Check that all deltas are present
        assert!(deltas.long_call != dec!(0.0));
        assert!(deltas.short_call != dec!(0.0));
        assert!(deltas.long_put != dec!(0.0));
        assert!(deltas.short_put != dec!(0.0));
    }
}

#[cfg(test)]
mod tests_get_position {
    use super::*;
    use crate::model::ExpirationDate;

    use crate::{assert_pos_relative_eq, pos, spos};
    use chrono::{Duration, Utc};
    use rust_decimal_macros::dec;

    // Helper function to create a standard test option data
    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            pos!(0.2),        // implied_volatility
            Some(dec!(-0.3)), // delta_call
            Some(dec!(0.7)),  // delta_put
            Some(dec!(0.5)),  // gamma
            spos!(1000.0),    // volume
            Some(500),        // open_interest
            None,             // symbol
            None,             // expiration_date
            None,             // underlying_price
            None,             // risk_free_rate
            None,             // dividend_yield
            None,             // extra_fields
        )
    }

    // Helper function to create standard price parameters
    fn create_test_price_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some(Box::new("AAPL".to_string())),
        )
    }

    #[test]
    fn test_get_position_long_call() {
        let option_data = create_test_option_data();
        let price_params = create_test_price_params();

        // Test getting a long call position
        let result = option_data.get_position(
            Side::Long,
            OptionStyle::Call,
            None, // Default to current date
            None, // Default to zero fees
            None, // Default to zero fees
        );

        assert!(result.is_ok(), "Should successfully create position");

        let position = result.unwrap();

        // Verify position properties
        assert_eq!(position.option.side, Side::Long);
        assert_eq!(position.option.option_style, OptionStyle::Call);
        assert_eq!(position.option.strike_price, pos!(100.0));
        assert!(
            position.premium > Positive::ZERO,
            "Premium should be positive"
        );

        // Verify fees are set to zero by default
        assert_eq!(position.open_fee, Positive::ZERO);
        assert_eq!(position.close_fee, Positive::ZERO);
    }

    #[test]
    fn test_get_position_short_put() {
        let option_data = create_test_option_data();
        let price_params = create_test_price_params();

        // Test getting a short put position
        let result = option_data.get_position(
            Side::Short,
            OptionStyle::Put,
            None, // Default to current date
            None, // Default to zero fees
            None, // Default to zero fees
        );

        assert!(result.is_ok(), "Should successfully create position");

        let position = result.unwrap();

        // Verify position properties
        assert_eq!(position.option.side, Side::Short);
        assert_eq!(position.option.option_style, OptionStyle::Put);
        assert_eq!(position.option.strike_price, pos!(100.0));
        assert!(
            position.premium > Positive::ZERO,
            "Premium should be positive"
        );
    }

    #[test]
    fn test_get_position_with_custom_date() {
        let option_data = create_test_option_data();
        let price_params = create_test_price_params();

        // Create a custom date (one week ago)
        let custom_date = Utc::now() - Duration::days(7);

        // Test with custom date
        let result =
            option_data.get_position(Side::Long, OptionStyle::Call, Some(custom_date), None, None);

        assert!(result.is_ok());
        let position = result.unwrap();

        // Verify the date was set correctly
        assert_eq!(position.date, custom_date);
    }

    #[test]
    fn test_get_position_with_fees() {
        let option_data = create_test_option_data();
        let price_params = create_test_price_params();

        // Custom fees
        let open_fee = pos!(1.5);
        let close_fee = pos!(2.0);

        // Test with custom fees
        let result = option_data.get_position(
            Side::Long,
            OptionStyle::Put,
            None,
            Some(open_fee),
            Some(close_fee),
        );

        assert!(result.is_ok());
        let position = result.unwrap();

        // Verify fees were set correctly
        assert_eq!(position.open_fee, open_fee);
        assert_eq!(position.close_fee, close_fee);
    }

    #[test]
    fn test_get_position_missing_volatility() {
        let mut option_data = create_test_option_data();

        // Test with missing volatility
        let result = option_data.get_position(Side::Long, OptionStyle::Call, None, None, None);

        // Should fail due to missing volatility
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ChainError::OptionDataError(detail) => {
                assert!(detail.to_string().contains("volatility"));
            }
            _ => panic!("Wrong error type returned"),
        }
    }

    #[test]
    fn test_get_position_in_the_money_call() {
        let mut option_data = create_test_option_data();

        // Create params with underlying price higher than strike (ITM call)
        let mut price_params = create_test_price_params();
        price_params.underlying_price = Some(Box::new(pos!(120.0)));
        option_data.set_extra_params(price_params);

        let result = option_data.get_position(Side::Long, OptionStyle::Call, None, None, None);

        assert!(result.is_ok());
        let position = result.unwrap();

        // An ITM call should have higher premium
        assert!(
            position.premium >= pos!(10.0),
            "ITM call premium should be significant"
        );
    }

    #[test]
    fn test_get_position_all_combinations() {
        let option_data = create_test_option_data();

        // Test all combinations of Side and OptionStyle
        let combinations = vec![
            (Side::Long, OptionStyle::Call),
            (Side::Long, OptionStyle::Put),
            (Side::Short, OptionStyle::Call),
            (Side::Short, OptionStyle::Put),
        ];

        for (side, style) in combinations {
            let result = option_data.get_position(side, style, None, None, None);

            assert!(
                result.is_ok(),
                "Failed to create position: {:?}, {:?}",
                side,
                style
            );
            let position = result.unwrap();

            // Verify position properties
            assert_eq!(position.option.side, side);
            assert_eq!(position.option.option_style, style);
            assert!(position.premium > Positive::ZERO);
        }
    }

    #[test]
    fn test_get_position_with_custom_all_params() {
        // This test checks that all custom parameters are correctly applied
        let option_data = create_test_option_data();

        // Create a custom date
        let custom_date = Utc::now() - Duration::days(14);

        // Custom fees
        let open_fee = pos!(2.5);
        let close_fee = pos!(1.75);

        // Test with all custom parameters
        let result = option_data.get_position(
            Side::Short,
            OptionStyle::Put,
            Some(custom_date),
            Some(open_fee),
            Some(close_fee),
        );

        assert!(result.is_ok());
        let position = result.unwrap();

        // Verify all parameters were applied correctly
        assert_eq!(position.option.side, Side::Short);
        assert_eq!(position.option.option_style, OptionStyle::Put);
        assert_eq!(position.date, custom_date);
        assert_eq!(position.open_fee, open_fee);
        assert_eq!(position.close_fee, close_fee);
    }

    #[test]
    fn test_get_position_uses_market_price_long_call() {
        let option_data = create_test_option_data();

        // Test getting a long call position
        let result = option_data.get_position(
            Side::Long,
            OptionStyle::Call,
            None, // Default to current date
            None, // Default to zero fees
            None, // Default to zero fees
        );

        assert!(result.is_ok(), "Should successfully create position");

        let position = result.unwrap();

        // For a long call, should use call_ask (10.0)
        assert_eq!(
            position.premium,
            pos!(10.0),
            "Should use call_ask price for long call"
        );
    }

    #[test]
    fn test_get_position_uses_market_price_short_call() {
        let option_data = create_test_option_data();

        // Test getting a short call position
        let result = option_data.get_position(
            Side::Short,
            OptionStyle::Call,
            None, // Default to current date
            None, // Default to zero fees
            None, // Default to zero fees
        );

        assert!(result.is_ok(), "Should successfully create position");

        let position = result.unwrap();

        // For a short call, should use call_bid (9.5)
        assert_eq!(
            position.premium,
            pos!(9.5),
            "Should use call_bid price for short call"
        );
    }

    #[test]
    fn test_get_position_uses_market_price_long_put() {
        let option_data = create_test_option_data();

        // Test getting a long put position
        let result = option_data.get_position(
            Side::Long,
            OptionStyle::Put,
            None, // Default to current date
            None, // Default to zero fees
            None, // Default to zero fees
        );

        assert!(result.is_ok(), "Should successfully create position");

        let position = result.unwrap();

        // For a long put, should use put_ask (9.0)
        assert_eq!(
            position.premium,
            pos!(9.0),
            "Should use put_ask price for long put"
        );
    }

    #[test]
    fn test_get_position_uses_market_price_short_put() {
        let option_data = create_test_option_data();

        // Test getting a short put position
        let result = option_data.get_position(
            Side::Short,
            OptionStyle::Put,
            None, // Default to current date
            None, // Default to zero fees
            None, // Default to zero fees
        );

        assert!(result.is_ok(), "Should successfully create position");

        let position = result.unwrap();

        // For a short put, should use put_bid (8.5)
        assert_eq!(
            position.premium,
            pos!(8.5),
            "Should use put_bid price for short put"
        );
    }

    #[test]
    fn test_get_position_fallback_to_black_scholes() {
        // Test with option data that doesn't have market prices
        let option_data = OptionData::new(
            pos!(100.0),      // strike_price
            None,             // call_bid (missing)
            None,             // call_ask (missing)
            None,             // put_bid (missing)
            None,             // put_ask (missing)
            pos!(0.2),        // implied_volatility
            Some(dec!(-0.3)), // delta_call
            Some(dec!(0.7)),  // delta_put
            Some(dec!(0.5)),  // gamma
            spos!(1000.0),    // volume
            Some(500),        // open_interest
            None,             // symbol
            None,             // expiration_date
            None,             // underlying_price
            None,             // risk_free_rate
            None,             // dividend_yield
            None,             // extra_fields
        );

        // Test getting a long call position
        let result = option_data.get_position(Side::Long, OptionStyle::Call, None, None, None);

        assert!(
            result.is_ok(),
            "Should successfully create position using Black-Scholes"
        );

        let position = result.unwrap();

        // Premium should be calculated using Black-Scholes
        assert!(
            position.premium > Positive::ZERO,
            "Should calculate premium using Black-Scholes"
        );

        // Let's verify it matches direct Black-Scholes calculation
        let option = option_data
            .get_option(Side::Long, OptionStyle::Call)
            .unwrap();
        let bs_price = option.calculate_price_black_scholes().unwrap().abs();
        let bs_price_positive = Positive::from(bs_price);

        assert_pos_relative_eq!(position.premium, bs_price_positive, pos!(0.00001));
    }

    #[test]
    fn test_get_position_with_custom_date_uses_market_price() {
        let option_data = create_test_option_data();

        // Create a custom date (one week ago)
        let custom_date = Utc::now() - Duration::days(7);

        // Test with custom date for long call
        let result =
            option_data.get_position(Side::Long, OptionStyle::Call, Some(custom_date), None, None);

        assert!(result.is_ok());
        let position = result.unwrap();

        // Verify the date was set correctly
        assert_eq!(position.date, custom_date);

        // Should still use market price (10.0 for long call)
        assert_eq!(
            position.premium,
            pos!(10.0),
            "Should use call_ask even with custom date"
        );
    }

    #[test]
    fn test_get_position_with_fees_uses_market_price() {
        let option_data = create_test_option_data();

        // Custom fees
        let open_fee = pos!(1.5);
        let close_fee = pos!(2.0);

        // Test with custom fees for short put
        let result = option_data.get_position(
            Side::Short,
            OptionStyle::Put,
            None,
            Some(open_fee),
            Some(close_fee),
        );

        assert!(result.is_ok());
        let position = result.unwrap();

        // Verify fees were set correctly
        assert_eq!(position.open_fee, open_fee);
        assert_eq!(position.close_fee, close_fee);

        // Should still use market price (8.5 for short put)
        assert_eq!(
            position.premium,
            pos!(8.5),
            "Should use put_bid even with custom fees"
        );
    }

    #[test]
    fn test_get_position_missing_specific_price() {
        // Test with option data missing just one price
        let mut option_data = create_test_option_data();
        option_data.call_ask = None; // Remove call ask price

        // Try to get a long call position which needs call_ask
        let result = option_data.get_position(Side::Long, OptionStyle::Call, None, None, None);

        // Should still succeed but fall back to Black-Scholes
        assert!(
            result.is_ok(),
            "Should fall back to Black-Scholes when specific price is missing"
        );

        let position = result.unwrap();

        // Let's verify it matches direct Black-Scholes calculation
        let option = option_data
            .get_option(Side::Long, OptionStyle::Call)
            .unwrap();
        let bs_price = option.calculate_price_black_scholes().unwrap().abs();
        let bs_price_positive = Positive::from(bs_price);

        assert_pos_relative_eq!(position.premium, bs_price_positive, pos!(0.00001));
    }
}

#[cfg(test)]
mod tests_check_convert_implied_volatility {
    use super::*;
    use crate::pos;

    #[test]
    fn test_check_and_convert_implied_volatility_over_one() {
        // Line 219: Test the conversion of implied volatility when it's greater than 1.0
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(20.0), // This is 2000% volatility, should be converted to 20%
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
            None,
        );

        // Call the method being tested
        option_data.check_and_convert_implied_volatility();

        // Assert that the volatility is now converted to a proper decimal (e.g., 0.2 instead of 20.0)
        assert_eq!(option_data.implied_volatility, pos!(0.2));
    }

    #[test]
    fn test_check_and_convert_implied_volatility_under_one() {
        // Test that volatility under 1.0 is not modified
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.15), // This is 15% volatility, should remain as is
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
            None,
        );

        // Original implied volatility
        let original_iv = option_data.implied_volatility;

        // Call the method being tested
        option_data.check_and_convert_implied_volatility();

        // Assert that the volatility is unchanged
        assert_eq!(option_data.implied_volatility, original_iv);
    }
}

#[cfg(test)]
mod tests_get_option_for_iv {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a standard OptionDataPriceParams for testing
    fn create_test_price_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some(Box::new("AAPL".to_string())),
        )
    }

    #[test]
    fn test_get_option_for_iv_success() {
        let mut option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos!(0.2),
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
            None,
        );

        let params = create_test_price_params();
        option_data.set_extra_params(params.clone());
        let initial_iv = pos!(0.25); // Different from the option_data IV to confirm it's using this value

        // Call the method being tested
        let result = option_data.get_option_for_iv(Side::Long, OptionStyle::Call, initial_iv);

        // Assert success and check properties
        assert!(result.is_ok());
        let option = result.unwrap();

        assert_eq!(option.option_type, OptionType::European);
        assert_eq!(option.side, Side::Long);
        assert_eq!(option.strike_price, pos!(100.0));
        assert_eq!(option.expiration_date, params.expiration_date.unwrap());
        assert_eq!(option.implied_volatility, initial_iv.to_f64()); // Should use the provided initial_iv
        assert_eq!(option.quantity, pos!(1.0));
        assert_eq!(option.underlying_price, *params.underlying_price.unwrap());
        assert_eq!(option.risk_free_rate, params.risk_free_rate.unwrap());
        assert_eq!(option.option_style, OptionStyle::Call);
        assert_eq!(option.dividend_yield, params.dividend_yield.unwrap());
    }

    #[test]
    fn test_get_option_for_iv_put() {
        // Test get_option_for_iv with Put option style
        let option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos!(0.2),
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
            None,
        );

        let params = create_test_price_params();
        let initial_iv = pos!(0.3);

        // Call the method with Put option style
        let result = option_data.get_option_for_iv(Side::Long, OptionStyle::Put, initial_iv);

        // Assert success and check option style
        assert!(result.is_ok());
        let option = result.unwrap();
        assert_eq!(option.option_style, OptionStyle::Put);
    }

    #[test]
    fn test_get_option_for_iv_short() {
        // Test get_option_for_iv with Short side
        let option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos!(0.2),
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
            None,
        );

        let params = create_test_price_params();
        let initial_iv = pos!(0.2);

        // Call the method with Short side
        let result = option_data.get_option_for_iv(Side::Short, OptionStyle::Call, initial_iv);

        // Assert success and check side
        assert!(result.is_ok());
        let option = result.unwrap();
        assert_eq!(option.side, Side::Short);
    }
}

#[cfg(test)]
mod tests_some_price_is_none {
    use super::*;
    use crate::{pos, spos};

    #[test]
    fn test_some_price_is_none_all_prices_present() {
        // Line 626: Test some_price_is_none when all prices are present
        let option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0), // call_bid
            spos!(5.5), // call_ask
            spos!(4.5), // put_bid
            spos!(5.0), // put_ask
            pos!(0.2),  // implied_volatility
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
            None,
        );

        // All prices are present, should return false
        assert!(!option_data.some_price_is_none());
    }

    #[test]
    fn test_some_price_is_none_with_missing_call_bid() {
        // Test with missing call_bid
        let option_data = OptionData::new(
            pos!(100.0),
            None,       // call_bid is None
            spos!(5.5), // call_ask
            spos!(4.5), // put_bid
            spos!(5.0), // put_ask
            pos!(0.2),  // implied_volatility
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
            None,
        );

        // One price is missing, should return true
        assert!(option_data.some_price_is_none());
    }

    #[test]
    fn test_some_price_is_none_with_missing_call_ask() {
        // Test with missing call_ask
        let option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0), // call_bid
            None,       // call_ask is None
            spos!(4.5), // put_bid
            spos!(5.0), // put_ask
            pos!(0.2),  // implied_volatility
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
            None,
        );

        // One price is missing, should return true
        assert!(option_data.some_price_is_none());
    }

    #[test]
    fn test_some_price_is_none_with_missing_put_bid() {
        // Test with missing put_bid
        let option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0), // call_bid
            spos!(5.5), // call_ask
            None,       // put_bid is None
            spos!(5.0), // put_ask
            pos!(0.2),  // implied_volatility
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
            None,
        );

        // One price is missing, should return true
        assert!(option_data.some_price_is_none());
    }

    #[test]
    fn test_some_price_is_none_with_missing_put_ask() {
        // Test with missing put_ask
        let option_data = OptionData::new(
            pos!(100.0),
            spos!(5.0), // call_bid
            spos!(5.5), // call_ask
            spos!(4.5), // put_bid
            None,       // put_ask is None
            pos!(0.2),  // implied_volatility
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
            None,
        );

        // One price is missing, should return true
        assert!(option_data.some_price_is_none());
    }

    #[test]
    fn test_some_price_is_none_with_all_prices_missing() {
        // Test with all prices missing
        let option_data = OptionData::new(
            pos!(100.0),
            None,      // call_bid is None
            None,      // call_ask is None
            None,      // put_bid is None
            None,      // put_ask is None
            pos!(0.2), // implied_volatility
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
            None,
        );

        // All prices are missing, should return true
        assert!(option_data.some_price_is_none());
    }
}

#[cfg(test)]
mod tests_is_valid_optimal_side_deltable {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_is_valid_optimal_side_deltable() {
        // Line 742-744: Test is_valid_optimal_side for Deltable threshold
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),        // implied_volatility
            Some(dec!(0.3)),  // delta_call
            Some(dec!(-0.3)), // delta_put
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

        // Deltable should always return true
        let result =
            option_data.is_valid_optimal_side(&pos!(100.0), &FindOptimalSide::Deltable(pos!(0.5)));

        assert!(result);
    }

    #[test]
    fn test_is_valid_optimal_side_center_panics() {
        // Lines 758-760: Test is_valid_optimal_side for Center (which should panic)
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2), // implied_volatility
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
            None,
        );

        // Testing for panic
        let result = std::panic::catch_unwind(|| {
            option_data.is_valid_optimal_side(&pos!(100.0), &FindOptimalSide::Center);
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_optimal_side_delta_range_valid_call() {
        // Lines 812-814: Test is_valid_optimal_side for DeltaRange with valid call delta
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),       // implied_volatility
            Some(dec!(0.3)), // delta_call within range
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

        // DeltaRange with min=0.2, max=0.4, which includes our delta_call=0.3
        let result = option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::DeltaRange(dec!(0.2), dec!(0.4)),
        );

        assert!(result);
    }

    #[test]
    fn test_is_valid_optimal_side_delta_range_valid_put() {
        // Test is_valid_optimal_side for DeltaRange with valid put delta
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2), // implied_volatility
            None,
            Some(dec!(0.3)), // delta_put within range
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

        // DeltaRange with min=0.2, max=0.4, which includes our delta_put=0.3
        let result = option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::DeltaRange(dec!(0.2), dec!(0.4)),
        );

        assert!(result);
    }

    #[test]
    fn test_is_valid_optimal_side_delta_range_invalid() {
        // Test is_valid_optimal_side for DeltaRange with invalid deltas
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),       // implied_volatility
            Some(dec!(0.1)), // delta_call outside range
            Some(dec!(0.5)), // delta_put outside range
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

        // DeltaRange with min=0.2, max=0.4, which excludes both delta values
        let result = option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::DeltaRange(dec!(0.2), dec!(0.4)),
        );

        assert!(!result);
    }

    #[test]
    fn test_is_valid_optimal_side_delta_range_no_deltas() {
        // Test is_valid_optimal_side for DeltaRange when no deltas are present
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2), // implied_volatility
            None,      // No delta_call
            None,      // No delta_put
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

        // DeltaRange with min=0.2, max=0.4, but no deltas to check
        let result = option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::DeltaRange(dec!(0.2), dec!(0.4)),
        );

        assert!(!result);
    }
}

#[cfg(test)]
mod tests_set_mid_prices {
    use super::*;
    use crate::{pos, spos};

    #[test]
    fn test_set_mid_prices_with_both_call_prices() {
        // Line 852: Test set_mid_prices with both call bid and ask present
        let mut option_data = OptionData::new(
            pos!(100.0),
            spos!(9.0),  // call_bid
            spos!(11.0), // call_ask
            None,
            None,
            pos!(0.2), // implied_volatility
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
            None,
        );

        // Call the method being tested
        option_data.set_mid_prices();

        // Assert that call_middle is set to (9.0 + 11.0) / 2 = 10.0
        assert_eq!(option_data.call_middle, Some(pos!(10.0)));
        // put_middle should still be None
        assert_eq!(option_data.put_middle, None);
    }

    #[test]
    fn test_set_mid_prices_with_both_put_prices() {
        // Test set_mid_prices with both put bid and ask present
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            spos!(8.0),  // put_bid
            spos!(12.0), // put_ask
            pos!(0.2),   // implied_volatility
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
            None,
        );

        // Call the method being tested
        option_data.set_mid_prices();

        // Assert that put_middle is set to (8.0 + 12.0) / 2 = 10.0
        assert_eq!(option_data.put_middle, Some(pos!(10.0)));
        // call_middle should still be None
        assert_eq!(option_data.call_middle, None);
    }

    #[test]
    fn test_set_mid_prices_with_all_prices() {
        // Test set_mid_prices with all prices present
        let mut option_data = OptionData::new(
            pos!(100.0),
            spos!(9.0),  // call_bid
            spos!(11.0), // call_ask
            spos!(8.0),  // put_bid
            spos!(12.0), // put_ask
            pos!(0.2),   // implied_volatility
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
            None,
        );

        // Call the method being tested
        option_data.set_mid_prices();

        // Assert that both middle prices are correctly calculated
        assert_eq!(option_data.call_middle, Some(pos!(10.0)));
        assert_eq!(option_data.put_middle, Some(pos!(10.0)));
    }

    #[test]
    fn test_set_mid_prices_with_missing_call_bid() {
        // Test set_mid_prices with missing call_bid
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,        // call_bid is missing
            spos!(11.0), // call_ask
            spos!(8.0),  // put_bid
            spos!(12.0), // put_ask
            pos!(0.2),   // implied_volatility
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
            None,
        );

        // Call the method being tested
        option_data.set_mid_prices();

        // Assert that call_middle is None due to missing bid
        assert_eq!(option_data.call_middle, None);
        // put_middle should still be calculated
        assert_eq!(option_data.put_middle, Some(pos!(10.0)));
    }

    #[test]
    fn test_set_mid_prices_with_missing_call_ask() {
        // Test set_mid_prices with missing call_ask
        let mut option_data = OptionData::new(
            pos!(100.0),
            spos!(9.0),  // call_bid
            None,        // call_ask is missing
            spos!(8.0),  // put_bid
            spos!(12.0), // put_ask
            pos!(0.2),   // implied_volatility
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
            None,
        );

        // Call the method being tested
        option_data.set_mid_prices();

        // Assert that call_middle is None due to missing ask
        assert_eq!(option_data.call_middle, None);
        // put_middle should still be calculated
        assert_eq!(option_data.put_middle, Some(pos!(10.0)));
    }
}

#[cfg(test)]
mod tests_get_mid_prices {
    use super::*;
    use crate::{pos, spos};

    #[test]
    fn test_get_mid_prices_with_both_mid_prices() {
        // Lines 885, 887, 889-895: Test get_mid_prices when both mid prices are set
        let mut option_data = OptionData::new(
            pos!(100.0),
            spos!(9.0),
            spos!(11.0),
            spos!(8.0),
            spos!(12.0),
            pos!(0.2), // implied_volatility
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
            None,
        );

        // First set the mid prices
        option_data.set_mid_prices();

        // Then test getting them
        let (call_mid, put_mid) = option_data.get_mid_prices();

        // Verify returned values
        assert_eq!(call_mid, Some(pos!(10.0)));
        assert_eq!(put_mid, Some(pos!(10.0)));
    }

    #[test]
    fn test_get_mid_prices_with_only_call_mid() {
        // Test get_mid_prices when only call_middle is set
        let mut option_data = OptionData::new(
            pos!(100.0),
            spos!(9.0),
            spos!(11.0),
            None, // missing put_bid
            spos!(12.0),
            pos!(0.2), // implied_volatility
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
            None,
        );

        // First set the mid prices
        option_data.set_mid_prices();

        // Then test getting them
        let (call_mid, put_mid) = option_data.get_mid_prices();

        // Verify returned values
        assert_eq!(call_mid, Some(pos!(10.0)));
        assert_eq!(put_mid, None);
    }

    #[test]
    fn test_get_mid_prices_with_only_put_mid() {
        // Test get_mid_prices when only put_middle is set
        let mut option_data = OptionData::new(
            pos!(100.0),
            None, // missing call_bid
            spos!(11.0),
            spos!(8.0),
            spos!(12.0),
            pos!(0.2), // implied_volatility
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
            None,
        );

        // First set the mid prices
        option_data.set_mid_prices();

        // Then test getting them
        let (call_mid, put_mid) = option_data.get_mid_prices();

        // Verify returned values
        assert_eq!(call_mid, None);
        assert_eq!(put_mid, Some(pos!(10.0)));
    }

    #[test]
    fn test_get_mid_prices_with_no_mid_prices() {
        // Test get_mid_prices when no mid prices are set
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2), // implied_volatility
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
            None,
        );

        // Mid prices haven't been set, should both be None
        let (call_mid, put_mid) = option_data.get_mid_prices();

        // Verify returned values
        assert_eq!(call_mid, None);
        assert_eq!(put_mid, None);
    }
}

#[cfg(test)]
mod tests_current_deltas {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_current_deltas_with_both_deltas() {
        // Lines 933-934: Test current_deltas method when both deltas are present
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),        // implied_volatility
            Some(dec!(0.5)),  // delta_call
            Some(dec!(-0.5)), // delta_put
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

        // Get current deltas
        let (call_delta, put_delta) = option_data.current_deltas();

        // Verify returned values
        assert_eq!(call_delta, Some(dec!(0.5)));
        assert_eq!(put_delta, Some(dec!(-0.5)));
    }

    #[test]
    fn test_current_deltas_with_only_call_delta() {
        // Test current_deltas with only call delta
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),       // implied_volatility
            Some(dec!(0.5)), // delta_call
            None,            // No delta_put
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

        // Get current deltas
        let (call_delta, put_delta) = option_data.current_deltas();

        // Verify returned values
        assert_eq!(call_delta, Some(dec!(0.5)));
        assert_eq!(put_delta, None);
    }

    #[test]
    fn test_current_deltas_with_only_put_delta() {
        // Test current_deltas with only put delta
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),        // implied_volatility
            None,             // No delta_call
            Some(dec!(-0.5)), // delta_put
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

        // Get current deltas
        let (call_delta, put_delta) = option_data.current_deltas();

        // Verify returned values
        assert_eq!(call_delta, None);
        assert_eq!(put_delta, Some(dec!(-0.5)));
    }

    #[test]
    fn test_current_deltas_with_no_deltas() {
        // Test current_deltas with no deltas
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2), // implied_volatility
            None,      // No delta_call
            None,      // No delta_put
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

        // Get current deltas
        let (call_delta, put_delta) = option_data.current_deltas();

        // Verify returned values
        assert_eq!(call_delta, None);
        assert_eq!(put_delta, None);
    }
}
