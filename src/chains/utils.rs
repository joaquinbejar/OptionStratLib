/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/10/24
******************************************************************************/
use crate::chains::OptionData;
use crate::error::chains::ChainError;
use crate::model::ExpirationDate;
use crate::model::utils::ToRound;
use crate::{Positive, pos};
use num_traits::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

/// Enum representing a grouping of option data references for analysis or display purposes.
///
/// This enum provides different ways to group option data references, from individual options
/// to collections of various sizes. It supports holding references to one, two, three, or four
/// specific options, or an arbitrary number of options through the `Any` variant.
///
/// # Variants
///
/// * `One` - Contains a reference to a single option data record.
///
/// * `Two` - Contains references to exactly two option data records, typically used
///   for comparison or spread analysis.
///
/// * `Three` - Contains references to exactly three option data records, useful for
///   analyzing multi-leg option strategies like butterflies.
///
/// * `Four` - Contains references to exactly four option data records, useful for
///   complex option strategies like condors or iron condors.
///
/// * `Any` - Contains a vector of option data references for more flexible grouping
///   when the number of options is variable or exceeds four.
///
/// # Type Parameters
///
/// * `'a` - The lifetime parameter ensuring that all referenced `OptionData` instances
///   live at least as long as this `OptionDataGroup`.
///
/// # Usage
///
/// This enum is typically used when analyzing multiple options together, displaying
/// related options in a UI, or processing option groups in trading strategies.
#[derive(Debug)]
pub enum OptionDataGroup<'a> {
    /// A single option data reference
    One(&'a OptionData),

    /// Two option data references, useful for spreads
    Two(&'a OptionData, &'a OptionData),

    /// Three option data references, useful for butterfly spreads
    Three(&'a OptionData, &'a OptionData, &'a OptionData),

    /// Four option data references, useful for condors and iron condors
    Four(
        &'a OptionData,
        &'a OptionData,
        &'a OptionData,
        &'a OptionData,
    ),

    /// A variable number of option data references
    Any(Vec<&'a OptionData>),
}

/// Parameters for building an option chain dataset.
///
/// This structure encapsulates all necessary configuration parameters to generate
/// a synthetic option chain for financial modeling and analysis. It controls various
/// aspects like size, pricing behavior, and volatility skew characteristics of the
/// resulting option chain.
///
/// # Fields
///
/// * `symbol` - The ticker symbol for the option chain's underlying asset.
///
/// * `volume` - Optional trading volume to assign to the generated options. If None,
///   default or random volumes may be used.
///
/// * `chain_size` - The number of strike prices to include above and below the at-the-money
///   strike in the generated chain.
///
/// * `strike_interval` - The fixed price difference between adjacent strike prices in the chain.
///
/// * `skew_factor` - Controls the volatility skew pattern in the option chain. Positive values
///   create a volatility smile, negative values create an inverted skew.
///
/// * `spread` - The bid-ask spread to apply to option prices in the chain.
///
/// * `decimal_places` - The number of decimal places to round prices to in the generated chain.
///
/// * `price_params` - Fundamental pricing parameters including underlying price, volatility,
///   expiration, and other inputs required for option pricing models.
///
/// # Usage
///
/// This structure is typically used as input to option chain generation functions to create
/// realistic synthetic option data for testing, simulation, or educational purposes.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct OptionChainBuildParams {
    /// The ticker symbol of the underlying asset
    pub(crate) symbol: String,

    /// Optional trading volume for the generated options
    pub(crate) volume: Option<Positive>,

    /// Number of strike prices to include above and below the at-the-money strike
    pub(crate) chain_size: usize,

    /// Price difference between adjacent strike prices
    pub(crate) strike_interval: Positive,

    /// Factor controlling the volatility skew pattern (positive for smile, negative for skew)
    pub(crate) skew_factor: Decimal,

    /// Bid-ask spread to apply to option prices
    pub(crate) spread: Positive,

    /// Number of decimal places for price rounding
    pub(crate) decimal_places: i32,

    /// Core pricing parameters required for option valuation
    pub(crate) price_params: OptionDataPriceParams,
}

#[allow(clippy::too_many_arguments)]
impl OptionChainBuildParams {
    /// Implementation of the constructor for `OptionChainBuildParams`.
    ///
    /// This implementation provides a constructor method `new()` to create instances of
    /// `OptionChainBuildParams` for generating synthetic option chains with customizable
    /// parameters.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The ticker symbol of the underlying asset for the option chain.
    ///
    /// * `volume` - Optional trading volume to assign to the generated options. When `None`,
    ///   default or random volumes may be used depending on the chain generation logic.
    ///
    /// * `chain_size` - Number of strike prices to include above and below the at-the-money strike,
    ///   determining the total size of the generated option chain.
    ///
    /// * `strike_interval` - The fixed price difference between adjacent strike prices in the chain,
    ///   represented as a positive decimal value.
    ///
    /// * `skew_factor` - A factor controlling the volatility skew pattern in the option chain.
    ///   Positive values create a volatility smile, negative values create an inverted skew.
    ///
    /// * `spread` - The bid-ask spread to apply to option prices in the chain, represented as a
    ///   positive decimal value.
    ///
    /// * `decimal_places` - The number of decimal places to round prices to in the generated chain.
    ///
    /// * `price_params` - Core pricing parameters required for option valuation, including
    ///   underlying price, expiration date, implied volatility, risk-free rate, and dividend yield.
    ///
    /// # Returns
    ///
    /// A new instance of `OptionChainBuildParams` with the specified configuration parameters.
    ///
    pub fn new(
        symbol: String,
        volume: Option<Positive>,
        chain_size: usize,
        strike_interval: Positive,
        skew_factor: Decimal,
        spread: Positive,
        decimal_places: i32,
        price_params: OptionDataPriceParams,
    ) -> Self {
        Self {
            symbol,
            volume,
            chain_size,
            strike_interval,
            skew_factor,
            spread,
            decimal_places,
            price_params,
        }
    }

    /// Sets the underlying asset price.
    ///
    /// This function updates the `underlying_price` field within the `price_params`
    /// structure.  The underlying price represents the current market price of the asset
    /// on which the option is based.  This value is crucial for option pricing calculations.
    ///
    /// # Arguments
    ///
    /// * `price` - A `Positive` value representing the new underlying asset price.  The
    ///   `Positive` type ensures that the price is always a non-negative value.
    ///
    pub fn set_underlying_price(&mut self, price: &Positive) {
        self.price_params.underlying_price = *price;
    }

    /// Retrieves the implied volatility.
    ///
    /// This function returns the implied volatility associated with the option,
    /// stored within the `price_params` structure. Implied volatility represents the
    /// market's expectation of the future volatility of the underlying asset.  It's
    /// a key input in option pricing models.  The function returns an `Option<Positive>`
    /// as the implied volatility might not always be available or calculated.
    ///
    /// # Returns
    ///
    /// * `Option<Positive>` - The implied volatility, wrapped in an `Option`.  If the
    ///   implied volatility has been set, the `Option` will contain a `Positive` value.
    ///   Otherwise, it will return `None`.
    pub fn get_implied_volatility(&self) -> Option<Positive> {
        self.price_params.implied_volatility
    }

    /// Sets the implied volatility.
    ///
    /// This function updates the `implied_volatility` field within the `price_params`
    /// structure. The implied volatility reflects the market's view on the future price
    /// fluctuations of the underlying asset. This parameter plays a significant role in
    /// determining option prices.
    ///
    /// # Arguments
    ///
    /// * `volatility` - An `Option<Positive>` representing the implied volatility.  Providing
    ///   `Some(Positive)` will set the volatility to the given value.  Providing `None`
    ///   clears any previously set implied volatility, useful when the volatility needs to be
    ///   recalculated or derived from other data.
    pub fn set_implied_volatility(&mut self, volatility: Option<Positive>) {
        self.price_params.implied_volatility = volatility
    }
}

impl Display for OptionChainBuildParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Option Chain Build Parameters:")?;
        writeln!(f, "  Symbol: {}", self.symbol)?;

        if let Some(volume) = self.volume {
            writeln!(f, "  Volume: {}", volume)?;
        } else {
            writeln!(f, "  Volume: None")?;
        }

        writeln!(f, "  Chain Size: {}", self.chain_size)?;
        writeln!(f, "  Strike Interval: {}", self.strike_interval)?;
        writeln!(f, "  Skew Factor: {}", self.skew_factor)?;
        writeln!(f, "  Spread: {}", self.spread.round_to(3))?;
        writeln!(f, "  Decimal Places: {}", self.decimal_places)?;
        writeln!(f, "  Price Parameters:")?;
        writeln!(
            f,
            "    Underlying Price: {}",
            self.price_params.underlying_price
        )?;
        writeln!(
            f,
            "    Expiration Date: {}",
            &self.price_params.expiration_date
        )?;

        if let Some(iv) = self.price_params.implied_volatility {
            writeln!(f, "    Implied Volatility: {:.2}%", iv * 100.0)?;
        } else {
            writeln!(f, "    Implied Volatility: None")?;
        }

        writeln!(
            f,
            "    Risk-Free Rate: {:.2}%",
            self.price_params.risk_free_rate * dec!(100.0)
        )?;
        writeln!(
            f,
            "    Dividend Yield: {:.2}%",
            self.price_params.dividend_yield * dec!(100.0)
        )?;

        Ok(())
    }
}

/// Parameters required for pricing an option contract.
///
/// This structure encapsulates all necessary inputs for option pricing models
/// such as Black-Scholes or binomial tree models. It contains information about
/// the underlying asset, market conditions, and contract specifications needed
/// to calculate fair option values.
///
/// # Fields
///
/// * `underlying_price` - The current market price of the underlying asset.
///
/// * `expiration_date` - When the option contract expires, either as days to expiration
///   or as a specific datetime.
///
/// * `implied_volatility` - The expected volatility of the underlying asset price over
///   the life of the option. If None, it may be calculated from other parameters.
///
/// * `risk_free_rate` - The theoretical rate of return of an investment with zero risk,
///   used in option pricing models.
///
/// * `dividend_yield` - The dividend yield of the underlying asset, expressed as a positive
///   decimal value.
///
/// * `underlying_symbol` - Optional ticker or identifier for the underlying asset.
///
/// # Usage
///
/// This structure is typically used as input to option pricing functions to calculate
/// theoretical values, Greeks (delta, gamma, etc.), and other option metrics.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct OptionDataPriceParams {
    /// The current price of the underlying asset
    pub(crate) underlying_price: Positive,

    /// When the option expires, either as days to expiration or as a specific datetime
    pub(crate) expiration_date: ExpirationDate,

    /// The expected volatility of the underlying asset price, if known
    pub(crate) implied_volatility: Option<Positive>,

    /// The risk-free interest rate used in pricing calculations
    pub(crate) risk_free_rate: Decimal,

    /// The dividend yield of the underlying asset
    pub(crate) dividend_yield: Positive,

    /// Optional ticker symbol or identifier for the underlying asset
    pub(crate) underlying_symbol: Option<String>,
}

impl OptionDataPriceParams {
    /// Creates a new instance of `OptionDataPriceParams` with the provided parameters.
    ///
    /// This constructor initializes all the required fields for option pricing calculations,
    /// including asset price, expiration, volatility, and market rates.
    ///
    /// # Parameters
    ///
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `expiration_date` - When the option contract expires (either as days to expiration or as a specific datetime)
    /// * `implied_volatility` - The expected volatility of the underlying asset price (if known)
    /// * `risk_free_rate` - The theoretical risk-free interest rate used in pricing calculations
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `underlying_symbol` - Optional ticker or identifier for the underlying asset
    ///
    /// # Returns
    ///
    /// A new instance of `OptionDataPriceParams` containing the provided parameters
    pub fn new(
        underlying_price: Positive,
        expiration_date: ExpirationDate,
        implied_volatility: Option<Positive>,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        underlying_symbol: Option<String>,
    ) -> Self {
        if implied_volatility.is_some() {
            assert!(
                implied_volatility <= Some(Positive::ONE),
                "Implied volatility: {} must be between 0 and 1",
                implied_volatility.unwrap()
            );
        }
        Self {
            underlying_price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            underlying_symbol,
        }
    }

    /// Returns the current price of the underlying asset.
    ///
    /// # Returns
    ///
    /// A `Positive` value representing the underlying asset's current market price
    pub fn get_underlying_price(&self) -> Positive {
        self.underlying_price
    }

    /// Returns the expiration date of the option contract.
    ///
    /// # Returns
    ///
    /// An `ExpirationDate` representing when the option expires, either as days to expiration or a specific datetime
    pub fn get_expiration_date(&self) -> ExpirationDate {
        self.expiration_date
    }

    /// Returns the implied volatility of the underlying asset, if available.
    ///
    /// # Returns
    ///
    /// `Some(Positive)` containing the implied volatility if known, or `None` if not specified
    pub fn get_implied_volatility(&self) -> Option<Positive> {
        self.implied_volatility
    }

    /// Returns the risk-free interest rate used in pricing calculations.
    ///
    /// # Returns
    ///
    /// A `Decimal` value representing the current risk-free rate
    pub fn get_risk_free_rate(&self) -> Decimal {
        self.risk_free_rate
    }

    /// Returns the dividend yield of the underlying asset.
    ///
    /// # Returns
    ///
    /// A `Positive` value representing the dividend yield of the underlying asset
    pub fn get_dividend_yield(&self) -> Positive {
        self.dividend_yield
    }
}

impl Default for OptionDataPriceParams {
    fn default() -> Self {
        Self {
            underlying_price: Positive::ZERO,
            expiration_date: ExpirationDate::Days(Positive::ZERO),
            implied_volatility: None,
            risk_free_rate: Decimal::ZERO,
            dividend_yield: Positive::ZERO,
            underlying_symbol: None,
        }
    }
}

impl Display for OptionDataPriceParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Underlying Price: {:.3}, Expiration: {:.4} Years, Implied Volatility: {:.3}, Risk-Free Rate: {:.2}, Dividend Yield: {:.2}",
            self.underlying_price,
            self.expiration_date.get_years().unwrap(),
            self.implied_volatility.unwrap_or(Positive::ZERO).value(),
            self.risk_free_rate,
            self.dividend_yield
        )
    }
}

/// A trait for obtaining option pricing parameters based on a strike price.
///
/// This trait defines an interface for types that can provide the necessary parameters
/// for pricing options at a specific strike price. Implementations of this trait
/// handle the logic of determining appropriate pricing parameters such as underlying price,
/// expiration date, implied volatility, risk-free rate, dividend yield, and other relevant
/// values required for option pricing models.
///
/// # Type Parameters
///
/// The trait is generic over the implementing type, allowing various sources of option
/// parameters to conform to a single interface.
///
/// # Methods
///
/// * `get_params` - Retrieves the option pricing parameters for a given strike price.
///
/// # Errors
///
/// Returns a `ChainError` if the parameters cannot be determined or are invalid for
/// the specified strike price.
///
/// # Usage
///
/// This trait is typically implemented by types that represent sources of option chain data,
/// such as market data providers, model-based generators, or historical data repositories.
/// It provides a uniform way to access option pricing parameters regardless of their source.
pub trait OptionChainParams {
    /// Retrieves the option pricing parameters for a given strike price.
    ///
    /// This method calculates or retrieves all parameters necessary for pricing an option
    /// at the specified strike price, including the underlying price, expiration date,
    /// implied volatility (if available), risk-free rate, dividend yield, and underlying symbol.
    ///
    /// # Parameters
    ///
    /// * `strike_price` - A positive decimal value representing the strike price of the option
    ///   for which parameters are being requested.
    ///
    /// # Returns
    ///
    /// * `Ok(OptionDataPriceParams)` - A structure containing all necessary parameters for
    ///   option pricing calculations if the parameters could be successfully determined.
    /// * `Err(ChainError)` - An error if the parameters cannot be determined or are invalid
    ///   for the given strike price.
    ///
    /// # Errors
    ///
    /// This method may return various `ChainError` variants depending on the implementation,
    /// such as:
    /// - `ChainError::OptionDataError` for invalid option data
    /// - `ChainError::ChainBuildError` for problems constructing chain parameters
    /// - Other error types as appropriate for the specific implementation
    fn get_params(&self, strike_price: Positive) -> Result<OptionDataPriceParams, ChainError>;
}

/// Parameters for generating random positions in an option chain
#[derive(Clone, Debug)]
pub struct RandomPositionsParams {
    /// Number of long put positions to generate
    pub qty_puts_long: Option<usize>,
    /// Number of short put positions to generate
    pub qty_puts_short: Option<usize>,
    /// Number of long call positions to generate
    pub qty_calls_long: Option<usize>,
    /// Number of short call positions to generate
    pub qty_calls_short: Option<usize>,
    /// Expiration date for the options
    pub expiration_date: ExpirationDate,
    /// Quantity for each option position
    pub option_qty: Positive,
    /// Risk free interest rate
    pub risk_free_rate: Decimal,
    /// Dividend yield of the underlying
    pub dividend_yield: Positive,
    /// Fee for opening put positions
    pub open_put_fee: Positive,
    /// Fee for opening call positions
    pub open_call_fee: Positive,
    /// Fee for closing put positions
    pub close_put_fee: Positive,
    /// Fee for closing call positions
    pub close_call_fee: Positive,
}

impl RandomPositionsParams {
    /// Creates a new instance of `RandomPositionsParams` with the specified parameters.
    ///
    /// This constructor initializes a configuration object that defines parameters for
    /// generating random option positions in an option chain. It allows specifying the
    /// quantity of different option types (puts/calls, long/short), expiration settings,
    /// and various fee structures.
    ///
    /// # Parameters
    ///
    /// * `qty_puts_long` - Optional number of long put positions to generate
    /// * `qty_puts_short` - Optional number of short put positions to generate
    /// * `qty_calls_long` - Optional number of long call positions to generate
    /// * `qty_calls_short` - Optional number of short call positions to generate
    /// * `expiration_date` - The expiration date for the options (can be specified as days from now or absolute date)
    /// * `option_qty` - The quantity of contracts for each option position
    /// * `risk_free_rate` - The risk-free interest rate used for option pricing calculations
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `open_put_fee` - The fee charged when opening put positions
    /// * `open_call_fee` - The fee charged when opening call positions
    /// * `close_put_fee` - The fee charged when closing put positions
    /// * `close_call_fee` - The fee charged when closing call positions
    ///
    /// # Returns
    ///
    /// A new `RandomPositionsParams` instance with the specified configuration.
    ///
    /// # Note
    ///
    /// This function has many parameters, but this is justified by the complex nature
    /// of option position generation which requires detailed configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qty_puts_long: Option<usize>,
        qty_puts_short: Option<usize>,
        qty_calls_long: Option<usize>,
        qty_calls_short: Option<usize>,
        expiration_date: ExpirationDate,
        option_qty: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        open_put_fee: Positive,
        open_call_fee: Positive,
        close_put_fee: Positive,
        close_call_fee: Positive,
    ) -> Self {
        Self {
            qty_puts_long,
            qty_puts_short,
            qty_calls_long,
            qty_calls_short,
            expiration_date,
            option_qty,
            risk_free_rate,
            dividend_yield,
            open_put_fee,
            open_call_fee,
            close_put_fee,
            close_call_fee,
        }
    }
    /// Returns the total number of positions to generate.
    ///
    /// This method calculates the sum of all option position types (puts long/short and calls long/short)
    /// that need to be generated based on the current configuration. If any position type is not specified
    /// (None), it is treated as zero.
    ///
    /// # Returns
    ///
    /// The total number of option positions to be generated.
    ///
    pub fn total_positions(&self) -> usize {
        self.qty_puts_long.unwrap_or(0)
            + self.qty_puts_short.unwrap_or(0)
            + self.qty_calls_long.unwrap_or(0)
            + self.qty_calls_short.unwrap_or(0)
    }
}

pub(crate) fn adjust_volatility(
    volatility: Option<Positive>,
    skew_factor: Decimal,
    atm_distance: f64,
) -> Option<Positive> {
    volatility?;
    let skew: Decimal = skew_factor * Decimal::from_f64(atm_distance.abs()).unwrap();
    let smile: Decimal = skew_factor * Decimal::from_f64(atm_distance.powi(2)).unwrap();

    let volatility_skew = volatility.unwrap() * (Decimal::ONE + skew + smile);
    Some(volatility_skew.min(Positive::ONE))
}

#[allow(dead_code)]
pub(crate) fn parse<T: std::str::FromStr>(s: &str) -> Option<T> {
    let trimmed = s.trim();
    let input: Result<T, String> = match trimmed.parse::<T>() {
        Ok(value) => Ok(value),
        Err(_) => {
            return None;
        }
    };

    input.ok()
}

pub(crate) fn empty_string_round_to_2<T: ToString + ToRound>(input: Option<T>) -> String {
    input.map_or_else(|| "".to_string(), |v| v.round_to(2).to_string())
}

pub(crate) fn default_empty_string<T: ToString>(input: Option<T>) -> String {
    input.map_or_else(|| "".to_string(), |v| v.to_string())
}

pub(crate) fn rounder(reference_price: Positive, strike_interval: Positive) -> Positive {
    if strike_interval == Positive::ZERO {
        return reference_price;
    }
    let price = reference_price.value();
    let interval = strike_interval.value();

    let remainder = price % interval;
    let base = price - remainder;

    let rounded = if remainder >= interval / Decimal::TWO {
        base + interval
    } else {
        base
    };

    rounded.into()
}

/// Calculates the optimal strike interval for an option chain to achieve exactly `chain_size` strikes,
/// scaling the interval with both expected move and time to expiration.
///
/// This function:
/// 1. Computes expected move at 95% confidence using underlying price, implied volatility, and time.
/// 2. Derives a base interval based on the underlying price, scaled by a time factor to adjust for longer expiries.
/// 3. Determines a raw interval needed to span the expected move across the desired number of strikes.
/// 4. Takes the maximum of base and raw intervals, and rounds to a clean market-friendly value.
///
/// # Arguments
/// * `params` - Build parameters containing pricing inputs and desired chain size.
///
/// # Returns
/// `(strike_interval, num_strikes)`:
/// - `strike_interval`: calculated spacing between strikes.
/// - `num_strikes`: always equals `params.chain_size`.
///
/// # Errors
/// Returns `ChainError` if the expiration date cannot convert to days.
pub fn calculate_optimal_chain_params(
    params: &OptionChainBuildParams,
) -> Result<(Positive, usize), ChainError> {
    let p = &params.price_params;
    let price = p.underlying_price;

    // Use default 20% vol if none provided
    let iv = p.implied_volatility.unwrap_or(pos!(0.2));

    // Time to expiration in days and years
    let days = p.expiration_date.get_days()?;
    let t_years = days / pos!(365.0);

    // Expected move at 95% confidence (1.96 sigma)
    let expected_move = price * iv * t_years.sqrt() * pos!(1.96);

    // Time scaling factor: sqrt(days/30)
    // Larger for longer expiries, smaller for short ones
    let time_factor = (days / pos!(30.0)).sqrt();

    // Static base interval based on underlying price tiers
    let base_static = if price < pos!(25.0) {
        if price < pos!(10.0) {
            pos!(1.0)
        } else {
            pos!(2.5)
        }
    } else if price < pos!(100.0) {
        pos!(5.0)
    } else if price < pos!(1000.0) {
        pos!(10.0)
    } else {
        // For very high-priced assets, use 1% of price
        price * pos!(0.01)
    };

    // Adjust base interval by time factor and round
    let base_interval = (base_static * time_factor).round();

    // Calculate half the number of intervals for the desired strikes
    let num_strikes = params.chain_size;
    let half_intervals = ((num_strikes - 1) as f64) / 2.0;

    // Raw interval needed to span the expected move
    let raw_interval = expected_move / pos!(half_intervals);

    // Choose the larger of raw and base intervals
    let target_interval = raw_interval.max(base_interval);

    // println!(
    //     "Base interval: {}, half_intervals: {}, raw_interval:{}, target_interval: {}",
    //     base_interval,
    //     half_intervals,
    //     raw_interval.ceiling(),
    //     target_interval.ceiling()
    // );

    // Round to a clean market-friendly interval
    let strike_interval = round_to_clean_interval(target_interval, price);

    Ok((strike_interval, num_strikes))
}

/// Rounds an interval to clean market-friendly values like 0.25, 0.5, 1, 2.5, 5, 10, etc.
fn round_to_clean_interval(interval: Positive, price: Positive) -> Positive {
    let v = interval.to_f64();

    if price < pos!(25.0) {
        if v <= 0.25 {
            pos!(0.25)
        } else if v <= 0.5 {
            pos!(0.5)
        } else if v <= 1.0 {
            pos!(1.0)
        } else if v <= 2.5 {
            pos!(2.5)
        } else {
            pos!(5.0)
        }
    } else if price < pos!(100.0) {
        if v <= 1.0 {
            pos!(1.0)
        } else if v <= 2.5 {
            pos!(2.5)
        } else if v <= 5.0 {
            pos!(5.0)
        } else {
            pos!(10.0)
        }
    } else {
        if v <= 5.0 {
            pos!(1.0)
        } else if v <= 8.0 {
            pos!(2.0)
        } else if v <= 12.5 {
            pos!(5.0)
        } else if v <= 15.0 {
            pos!(10.0)
        } else if v <= 20.0 {
            pos!(15.0)
        } else if v <= 25.0 {
            pos!(20.0)
        } else if v <= 35.0 {
            pos!(25.0)
        } else if v <= 50.0 {
            pos!(50.0)
        } else {
            pos!(100.0)
        }
    }
}

#[cfg(test)]
mod tests_rounder {
    use super::*;
    use crate::pos;

    #[test]

    fn test_rounder() {
        assert_eq!(rounder(pos!(151.0), pos!(5.0)), pos!(150.0));
        assert_eq!(rounder(pos!(154.0), pos!(5.0)), pos!(155.0));
        assert_eq!(rounder(pos!(152.5), pos!(5.0)), pos!(155.0));
        assert_eq!(rounder(pos!(152.4), pos!(5.0)), pos!(150.0));

        assert_eq!(rounder(pos!(151.0), pos!(10.0)), pos!(150.0));
        assert_eq!(rounder(pos!(156.0), pos!(10.0)), pos!(160.0));
        assert_eq!(rounder(pos!(155.0), pos!(10.0)), pos!(160.0));
        assert_eq!(rounder(pos!(154.9), pos!(10.0)), pos!(150.0));

        assert_eq!(rounder(pos!(17.0), pos!(15.0)), pos!(15.0));
        assert_eq!(rounder(pos!(43.0), pos!(15.0)), pos!(45.0));
        assert_eq!(rounder(pos!(37.5), pos!(15.0)), pos!(45.0));
        assert_eq!(rounder(pos!(37.4), pos!(15.0)), pos!(30.0));
    }
}

#[cfg(test)]
mod tests_parse {
    use super::*;
    use crate::spos;
    use std::f64::consts::PI;

    #[test]

    fn test_parse_valid_integer() {
        let input = "42";
        let result: Option<i32> = parse(input);
        assert_eq!(result, Some(42));
    }

    #[test]

    fn test_parse_invalid_integer() {
        let input = "not_a_number";
        let result: Option<i32> = parse(input);
        assert_eq!(result, None);
    }

    #[test]

    fn test_parse_valid_float() {
        let input = &*PI.to_string();
        let result: Option<f64> = parse(input);
        assert_eq!(result, Some(PI));
    }

    #[test]

    fn test_positive_f64() {
        let input = "42.01";
        let result: Option<Positive> = parse(input);
        assert_eq!(result, spos!(42.01));
    }
}

#[cfg(test)]
mod tests_parse_bis {
    use super::*;
    use crate::{Positive, spos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_decimal() {
        let input = "42.5";
        let result: Option<Decimal> = parse(input);
        assert_eq!(result, Some(dec!(42.5)));

        let invalid = "not_a_decimal";
        let result: Option<Decimal> = parse(invalid);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_empty_string() {
        let input = "";
        let result: Option<i32> = parse(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_whitespace() {
        let input = "  ";
        let result: Option<i32> = parse(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_bool() {
        let input = "true";
        let result: Option<bool> = parse(input);
        assert_eq!(result, Some(true));

        let input = "false";
        let result: Option<bool> = parse(input);
        assert_eq!(result, Some(false));

        let input = "not_a_bool";
        let result: Option<bool> = parse(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_positive() {
        let input = "42.5";
        let result: Option<Positive> = parse(input);
        assert_eq!(result, spos!(42.5));

        // Negative numbers should return None for Positive type
        let input = "-42.5";
        let result: Option<Positive> = parse(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_different_number_formats() {
        // Integer
        let result: Option<i32> = parse("123");
        assert_eq!(result, Some(123));

        // Float
        let result: Option<f64> = parse("123.456");
        assert_eq!(result, Some(123.456));

        // Scientific notation
        let result: Option<f64> = parse("1.23e2");
        assert_eq!(result, Some(123.0));
    }

    #[test]
    fn test_parse_with_leading_trailing_spaces() {
        let input = "  42  ";
        let result: Option<i32> = parse(input);
        assert_eq!(result, Some(42));

        let input = "  42.5  ";
        let result: Option<f64> = parse(input);
        assert_eq!(result, Some(42.5));
    }

    #[test]
    fn test_parse_invalid_formats() {
        // Partial number
        let result: Option<i32> = parse("42abc");
        assert_eq!(result, None);

        // Multiple decimal points
        let result: Option<f64> = parse("42.3.4");
        assert_eq!(result, None);

        // Invalid scientific notation
        let result: Option<f64> = parse("1.23e");
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod tests_default_empty_string {
    use super::*;

    #[test]

    fn test_default_empty_string_with_some_value() {
        let input = Some(42);
        let result = default_empty_string(input);
        assert_eq!(result, "42");
    }

    #[test]

    fn test_default_empty_string_with_float() {
        let input = Some(42.01223);
        let result = default_empty_string(input);
        assert_eq!(result, "42.01223");
    }

    #[test]

    fn test_default_empty_string_with_none() {
        let input: Option<i32> = None;
        let result = default_empty_string(input);
        assert_eq!(result, "");
    }

    #[test]

    fn test_default_empty_string_with_string_value() {
        let input = Some("Hello");
        let result = default_empty_string(input);
        assert_eq!(result, "Hello");
    }
}

#[cfg(test)]
mod tests_random_positions_params {
    use super::*;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_params() -> RandomPositionsParams {
        RandomPositionsParams::new(
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
        )
    }

    #[test]

    fn test_new_params() {
        let params = create_test_params();
        assert_eq!(params.qty_puts_long, Some(1));
        assert_eq!(params.qty_puts_short, Some(1));
        assert_eq!(params.qty_calls_long, Some(1));
        assert_eq!(params.qty_calls_short, Some(1));
        assert_eq!(params.option_qty, 1.0);
        assert_eq!(params.risk_free_rate.to_f64().unwrap(), 0.05);
        assert_eq!(params.dividend_yield.to_f64(), 0.02);
        assert_eq!(params.open_put_fee, 1.0);
        assert_eq!(params.close_put_fee, 1.0);
        assert_eq!(params.open_call_fee, 1.0);
        assert_eq!(params.close_call_fee, 1.0);
    }

    #[test]

    fn test_total_positions() {
        let params = create_test_params();
        assert_eq!(params.total_positions(), 4);

        let params = RandomPositionsParams::new(
            Some(2),
            None,
            Some(3),
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
        assert_eq!(params.total_positions(), 5);

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
        assert_eq!(params.total_positions(), 0);
    }

    #[test]

    fn test_clone() {
        let params = create_test_params();
        let cloned = params.clone();
        assert_eq!(params.total_positions(), cloned.total_positions());
    }

    #[test]

    fn test_debug() {
        let params = create_test_params();
        let debug_output = format!("{:?}", params);
        assert!(debug_output.contains("RandomPositionsParams"));
    }
}

#[cfg(test)]
mod tests_adjust_volatility {
    use super::*;
    use crate::spos;

    #[test]

    fn test_adjust_volatility_none() {
        let result = adjust_volatility(None, dec!(0.1), 10.0);
        assert_eq!(result, None);
    }

    #[test]

    fn test_adjust_volatility_zero_skew() {
        let vol = spos!(0.2);
        let result = adjust_volatility(vol, dec!(0.0), 10.0);
        assert_eq!(result, vol);
    }

    #[test]

    fn test_adjust_volatility_positive_distance() {
        let vol = spos!(0.2);
        let result = adjust_volatility(vol, dec!(0.1), 10.0);
        assert!(result.is_some());
        assert!(result.unwrap() > vol.unwrap());
    }

    #[test]

    fn test_adjust_volatility_negative_distance() {
        let vol = spos!(0.2);
        let result = adjust_volatility(vol, dec!(0.1), -10.0);
        assert!(result.is_some());
        assert!(result.unwrap() > vol.unwrap());
    }
}

#[cfg(test)]
mod tests_option_data_price_params {
    use super::*;
    use crate::constants::ZERO;
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]

    fn test_new_price_params() {
        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        assert_eq!(params.underlying_price, pos!(100.0));
        assert_eq!(params.risk_free_rate.to_f64().unwrap(), 0.05);
        assert_eq!(params.dividend_yield.to_f64(), 0.02);
        assert_eq!(params.implied_volatility, spos!(0.2));
    }

    #[test]

    fn test_default_price_params() {
        let params = OptionDataPriceParams::default();
        assert_eq!(params.underlying_price, Positive::ZERO);
        assert_eq!(params.risk_free_rate.to_f64().unwrap(), ZERO);
        assert_eq!(params.dividend_yield.to_f64(), ZERO);
        assert_eq!(params.implied_volatility, None);
    }

    #[test]

    fn test_display_price_params() {
        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );
        let display_string = format!("{}", params);
        assert!(display_string.contains("Underlying Price: 100"));
        assert!(display_string.contains("Implied Volatility: 0.200"));
        assert!(display_string.contains("Risk-Free Rate: 0.05"));
        assert!(display_string.contains("Dividend Yield: 0.02"));
    }

    #[test]

    fn test_display_price_params_no_volatility() {
        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );
        let display_string = format!("{}", params);
        assert!(display_string.contains("Implied Volatility: 0.000"));
    }

    #[test]

    fn test_option_data_price_params_getters() {
        // Setup test parameters
        let underlying_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = spos!(0.2);
        let risk_free_rate = dec!(0.05);
        let dividend_yield = pos!(0.02);

        let params = OptionDataPriceParams::new(
            underlying_price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            None,
        );

        // Test each getter
        assert_eq!(params.get_underlying_price(), underlying_price);
        assert_eq!(params.get_expiration_date(), expiration_date);
        assert_eq!(params.get_implied_volatility(), implied_volatility);
        assert_eq!(params.get_risk_free_rate(), risk_free_rate);
        assert_eq!(params.get_dividend_yield(), dividend_yield);
    }

    #[test]

    fn test_option_data_price_params_getters_with_none_volatility() {
        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None, // No implied volatility
            dec!(0.05),
            pos!(0.02),
            None,
        );

        assert_eq!(params.get_implied_volatility(), None);
    }

    #[test]

    fn test_option_data_price_params_getters_with_datetime_expiration() {
        use chrono::{Duration, Utc};

        let future_date = Utc::now() + Duration::days(30);
        let expiration_date = ExpirationDate::DateTime(future_date);

        let params = OptionDataPriceParams::new(
            pos!(100.0),
            expiration_date,
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        assert_eq!(params.get_expiration_date(), expiration_date);
    }

    #[test]

    fn test_option_data_price_params_getters_zero_values() {
        let params = OptionDataPriceParams::new(
            Positive::ZERO,
            ExpirationDate::Days(Positive::ZERO),
            Some(Positive::ZERO),
            Decimal::ZERO,
            Positive::ZERO,
            None,
        );

        assert_eq!(params.get_underlying_price(), Positive::ZERO);
        assert_eq!(
            params.get_expiration_date(),
            ExpirationDate::Days(Positive::ZERO)
        );
        assert_eq!(params.get_implied_volatility(), Some(Positive::ZERO));
        assert_eq!(params.get_risk_free_rate(), Decimal::ZERO);
        assert_eq!(params.get_dividend_yield(), Positive::ZERO);
    }
}

#[cfg(test)]
mod tests_option_chain_build_params {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_chain_build_params() {
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            spos!(1000.0),
            10,
            pos!(5.0),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
        );

        assert_eq!(params.symbol, "TEST");
        assert_eq!(params.volume, spos!(1000.0));
        assert_eq!(params.chain_size, 10);
        assert_eq!(params.strike_interval, pos!(5.0));
        assert_eq!(params.skew_factor, dec!(0.1));
        assert_eq!(params.spread, pos!(0.02));
        assert_eq!(params.decimal_places, 2);

        let display = format!("{}", params);
        assert!(display.contains("TEST"));
        assert!(display.contains("Volume: 1000"));
        assert!(display.contains("Chain Size: 10"));
        assert!(display.contains("Strike Interval: 5"));
        assert!(display.contains("Skew Factor: 0.1"));
        assert!(display.contains("Spread: 0.02"));
        assert!(display.contains("Decimal Places: 2"));
    }

    #[test]
    fn test_chain_build_params_without_volume() {
        let price_params = OptionDataPriceParams::default();

        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            pos!(5.0),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
        );

        assert_eq!(params.volume, None);
    }
}

#[cfg(test)]
mod tests_random_positions_params_extended {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]

    fn test_partial_positions() {
        let params = RandomPositionsParams::new(
            Some(2),
            None,
            Some(1),
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

        assert_eq!(params.qty_puts_long, Some(2));
        assert_eq!(params.qty_puts_short, None);
        assert_eq!(params.qty_calls_long, Some(1));
        assert_eq!(params.qty_calls_short, None);
        assert_eq!(params.total_positions(), 3);
    }

    #[test]

    fn test_no_positions() {
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

        assert_eq!(params.total_positions(), 0);
    }

    #[test]

    fn test_expiration_date() {
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

        match params.expiration_date {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }
}

#[cfg(test)]
mod tests_sample {
    use super::*;
    use crate::chains::chain::OptionChain;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]

    fn test_chain() {
        let chain = OptionDataPriceParams::new(
            Positive::new(2000.0).unwrap(),
            ExpirationDate::Days(pos!(10.0)),
            Some(Positive::new(0.01).unwrap()),
            dec!(0.01),
            Positive::ZERO,
            None,
        );

        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            Some(Positive::ONE),
            5,
            Positive::ONE,
            dec!(0.0001),
            Positive::new(0.02).unwrap(),
            2,
            chain,
        );

        let built_chain = OptionChain::build_chain(&params);

        assert_eq!(built_chain.symbol, "SP500");
        assert_eq!(built_chain.underlying_price, Positive::new(2000.0).unwrap());
    }
}

#[cfg(test)]
mod utils_coverage_tests {
    use super::*;
    use crate::chains::utils::{adjust_volatility, empty_string_round_to_2};
    use crate::{pos, spos};

    // Test for lines 218-219, 246, 269, 271
    #[test]
    fn test_option_chain_build_params_getters_setters() {
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            Some("TEST".to_string()),
        );

        let mut params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            pos!(5.0),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
        );

        // Test get_implied_volatility
        let iv = params.get_implied_volatility();
        assert_eq!(iv, spos!(0.2));

        // Test set_underlying_price
        params.set_underlying_price(&pos!(110.0));
        assert_eq!(params.price_params.underlying_price, pos!(110.0));

        // Test set_implied_volatility
        params.set_implied_volatility(spos!(0.25));
        assert_eq!(params.get_implied_volatility(), spos!(0.25));

        // Test setting to None
        params.set_implied_volatility(None);
        assert_eq!(params.get_implied_volatility(), None);
    }

    // Test for lines 368-369
    #[test]
    fn test_empty_string_round_to_2() {
        // Test with Some value
        let value = Some(pos!(123.456));
        let result = empty_string_round_to_2(value);
        assert_eq!(result, "123.46");

        // Test with None
        let value: Option<Positive> = None;
        let result = empty_string_round_to_2(value);
        assert_eq!(result, "");
    }

    // Test for lines 636-637, 642, 655
    #[test]
    fn test_adjust_volatility_edge_cases() {
        // Test with None volatility
        let result = adjust_volatility(None, dec!(0.1), 10.0);
        assert_eq!(result, None);

        // Test with zero skew factor
        let result = adjust_volatility(spos!(0.2), dec!(0.0), 10.0);
        assert_eq!(result, spos!(0.2));

        // Test when adjusted volatility would exceed 1.0
        let result = adjust_volatility(spos!(0.9), dec!(0.5), 10.0); // This would make vol > 1.0
        assert_eq!(result, Some(Positive::ONE)); // Should cap at 1.0
    }
}
