/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/10/24
******************************************************************************/
use crate::chains::OptionData;
use crate::chains::chain::{SKEW_SLOPE, SKEW_SMILE_CURVE};
use crate::error::chains::ChainError;
use crate::model::ExpirationDate;
use crate::model::utils::ToRound;
use crate::{Positive, pos};
use num_traits::ToPrimitive;
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
/// * `smile_curve` - Controls the volatility skew pattern in the option chain. Positive values
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
    pub(crate) strike_interval: Option<Positive>,

    /// A field representing the volatility skew slope of a given parameter or function.
    pub(crate) skew_slope: Decimal,

    /// Factor controlling the volatility skew pattern (positive for smile, negative for skew)
    pub(crate) smile_curve: Decimal,

    /// Bid-ask spread to apply to option prices
    pub(crate) spread: Positive,

    /// Number of decimal places for price rounding
    pub(crate) decimal_places: u32,

    /// Core pricing parameters required for option valuation
    pub(crate) price_params: OptionDataPriceParams,

    pub(crate) implied_volatility: Positive,
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
    /// * `smile_curve` - A factor controlling the volatility skew pattern in the option chain.
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
        strike_interval: Option<Positive>,
        skew_slope: Decimal,
        smile_curve: Decimal,
        spread: Positive,
        decimal_places: u32,
        price_params: OptionDataPriceParams,
        implied_volatility: Positive,
    ) -> Self {
        Self {
            symbol,
            volume,
            chain_size,
            strike_interval,
            skew_slope,
            smile_curve,
            spread,
            decimal_places,
            price_params,
            implied_volatility,
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
    pub fn set_underlying_price(&mut self, price: Option<Box<Positive>>) {
        self.price_params.underlying_price = price;
    }

    /// Sets the implied volatility value for this option pricing parameter.
    ///
    /// # Arguments
    /// * `implied_vol` - A positive decimal value representing the implied volatility.
    pub fn set_implied_volatility(&mut self, implied_vol: Positive) {
        self.implied_volatility = implied_vol;
    }

    /// Returns the current implied volatility value.
    ///
    /// # Returns
    /// * `Positive` - The current implied volatility as a positive decimal value.
    pub fn get_implied_volatility(&self) -> Positive {
        self.implied_volatility
    }
}

impl Display for OptionChainBuildParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(pretty_json) => write!(f, "{pretty_json}"),
            Err(e) => write!(f, "Error serializing to JSON: {e}"),
        }
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct OptionDataPriceParams {
    /// The current price of the underlying asset
    pub(crate) underlying_price: Option<Box<Positive>>,

    /// When the option expires, either as days to expiration or as a specific datetime
    pub(crate) expiration_date: Option<ExpirationDate>,

    /// The risk-free interest rate used in pricing calculations
    pub(crate) risk_free_rate: Option<Decimal>,

    /// The dividend yield of the underlying asset
    pub(crate) dividend_yield: Option<Positive>,

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
        underlying_price: Option<Box<Positive>>,
        expiration_date: Option<ExpirationDate>,
        risk_free_rate: Option<Decimal>,
        dividend_yield: Option<Positive>,
        underlying_symbol: Option<String>,
    ) -> Self {
        Self {
            underlying_price,
            expiration_date,
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
    pub fn get_underlying_price(&self) -> Option<Box<Positive>> {
        self.underlying_price.clone()
    }

    /// Returns the expiration date of the option contract.
    ///
    /// # Returns
    ///
    /// An `ExpirationDate` representing when the option expires, either as days to expiration or a specific datetime
    pub fn get_expiration_date(&self) -> Option<ExpirationDate> {
        self.expiration_date
    }

    /// Returns the risk-free interest rate used in pricing calculations.
    ///
    /// # Returns
    ///
    /// A `Decimal` value representing the current risk-free rate
    pub fn get_risk_free_rate(&self) -> Option<Decimal> {
        self.risk_free_rate
    }

    /// Returns the dividend yield of the underlying asset.
    ///
    /// # Returns
    ///
    /// A `Positive` value representing the dividend yield of the underlying asset
    pub fn get_dividend_yield(&self) -> Option<Positive> {
        self.dividend_yield
    }

    /// Returns the symbol of the underlying asset.
    ///
    /// # Returns
    /// * `Option<String>` - The underlying symbol if available, or `None` if not set.
    pub fn get_symbol(&self) -> Option<String> {
        self.underlying_symbol.clone()
    }
}

impl Display for OptionDataPriceParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Underlying Price: {:.3}, Expiration: {:.4} Years, Risk-Free Rate: {:.2}%, Dividend Yield: {:.2}%, Symbol: {}",
            self.underlying_price
                .as_ref()
                .map_or_else(|| "None".to_string(), |p| p.value().to_string()),
            self.expiration_date.map_or_else(
                || "None".to_string(),
                |d| d.get_years().unwrap().to_string()
            ),
            self.risk_free_rate
                .map_or_else(|| "None".to_string(), |r| (r * dec!(100.0)).to_string()),
            self.dividend_yield.map_or_else(
                || "None".to_string(),
                |d| (d.value() * dec!(100.0)).to_string()
            ),
            self.underlying_symbol
                .as_ref()
                .map_or_else(|| "None".to_string(), |s| s.to_string()),
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
    /// Identifier for the position in an external system or platform
    pub epic: Option<String>,
    /// Additional custom data fields for the position stored as JSON
    pub extra_fields: Option<serde_json::Value>,
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
        epic: Option<String>,
        extra_fields: Option<serde_json::Value>,
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
            epic,
            extra_fields,
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

/// Adjust vol with skew/smile, using *relative* distance to ATM.
pub fn adjust_volatility(
    base_vol: &Option<Positive>,   // ATM vol (e.g. 0.17)
    skew_slope: &Option<Decimal>,  // slope per 10 % moneyness, e.g. -0.2
    smile_curve: &Option<Decimal>, // curvature, e.g. 0.4
    strike: &Positive,
    underlying_price: &Positive, // underlying_price
) -> Option<Positive> {
    if base_vol.is_none() {
        return None;
    }
    if strike.is_zero() {
        return None;
    }
    let base_vol = base_vol.unwrap();
    let skew_slope = skew_slope.unwrap_or(SKEW_SLOPE).to_f64().unwrap();
    let smile_curve = smile_curve.unwrap_or(SKEW_SMILE_CURVE).to_f64().unwrap();
    let m = (strike / underlying_price.to_f64()).ln();
    let factor: f64 = 1.0 + skew_slope * m + smile_curve * m * m;
    let clamped = factor.clamp(0.01, 3.0);

    (base_vol * clamped)
        .clamp(Positive::ZERO, Positive::ONE)
        .into()
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

pub(crate) fn empty_string_round_to_3<T: ToString + ToRound>(input: Option<T>) -> String {
    input.map_or_else(|| "".to_string(), |v| v.round_to(3).to_string())
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

/// Rounds an interval to clean market-friendly values like 0.25, 0.5, 1, 2.5, 5, 10, etc.
#[allow(dead_code)]
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
    } else if v <= 5.0 {
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

/// Return the strike interval that gives ~`size` strikes around ATM.
/// All units are in the same currency.
pub fn strike_step(
    underlying_price: Positive,
    implied_vol: Positive, // e.g. 0.25 for 25 %
    days_to_exp: Positive,
    size: usize,         // desired number of strikes
    k: Option<Positive>, // σ-multiplier you want to cover (2.0-3.0 typical)
) -> Positive {
    let k = k.unwrap_or_else(|| pos!(4.0));
    assert!(size > 1, "need at least two strikes");
    let t = days_to_exp / 365.0;
    let sigma = underlying_price * implied_vol * t.sqrt();
    let raw_step = pos!(2.0) * k * sigma / (size as f64 - 1.0);

    // Standard “nice” grids used by most exchanges
    let bins: &[Positive] = &[
        pos!(0.01),
        pos!(0.05),
        pos!(0.10),
        pos!(0.25),
        pos!(0.5),
        pos!(1.0),
        pos!(2.5),
        pos!(5.0),
        pos!(10.0),
        pos!(25.0),
        pos!(50.0),
        pos!(100.0),
        pos!(150.0),
        pos!(200.0),
        pos!(250.0),
    ];

    // Pick the closest one
    bins.iter()
        .copied()
        .min_by(|a, b| {
            ((a.to_dec() - raw_step.to_dec()).abs())
                .partial_cmp(&(b.to_dec() - raw_step.to_dec()).abs())
                .unwrap()
        })
        .unwrap_or(raw_step)
}

#[cfg(test)]
mod tests_strike_step {
    use super::*;
    use crate::chains::OptionChain;
    use crate::spos;
    use crate::utils::Len;
    #[test]
    fn basic() {
        let step = strike_step(pos!(100.0), pos!(0.2), pos!(30.0), 11, None);
        assert_eq!(step, 5.0);
    }

    #[test]
    fn long_days() {
        let step = strike_step(pos!(150.0), pos!(0.5), pos!(120.0), 30, spos!(3.0));

        assert_eq!(step, 10.0);
    }

    #[test]
    fn long_discrepancy() {
        let symbol = "AAPL".to_string();
        let risk_free_rate = dec!(0.02);
        let dividend_yield = pos!(0.0);
        let volume = Some(Positive::ONE);
        let spread = pos!(0.01);
        let decimal_places = 2;
        let skew_slope = dec!(-0.2);
        let smile_curve = dec!(0.1);

        let underlying_price = Some(Box::new(pos!(1547.0)));
        let days = pos!(45.0);
        let implied_volatility = pos!(0.17);
        let chain_size = 28;

        let strike_interval = strike_step(
            *underlying_price.clone().unwrap(),
            implied_volatility,
            days,
            chain_size,
            spos!(3.0),
        );

        assert_eq!(strike_interval, 25.0);

        let price_params = OptionDataPriceParams::new(
            underlying_price,
            Some(ExpirationDate::Days(days)),
            Some(risk_free_rate),
            Some(dividend_yield),
            Some(symbol.clone()),
        );
        let build_params = OptionChainBuildParams::new(
            symbol,
            volume,
            chain_size,
            Some(strike_interval),
            skew_slope,
            smile_curve,
            spread,
            decimal_places,
            price_params,
            implied_volatility,
        );
        let initial_chain = OptionChain::build_chain(&build_params);
        assert_eq!(initial_chain.len() - 1, chain_size);
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
            Some("Epic".to_string()),
            None,
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
            Some("Epic".to_string()),
            None,
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
            Some("Epic".to_string()),
            None,
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
        let debug_output = format!("{params:?}");
        assert!(debug_output.contains("RandomPositionsParams"));
    }
}

#[cfg(test)]
mod tests_adjust_volatility {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    /* 1 ─ base_vol = None → devuelve None */
    #[test]
    fn returns_none_when_base_is_none() {
        let strike = pos!(100.0);
        let spot = pos!(100.0);

        let out = adjust_volatility(
            &None, // base vol ausente
            &None, &None, &strike, &spot,
        );
        assert!(out.is_none());
    }

    /* 2 ─ sin skew/smile (defaults) la ATM vol no cambia */
    #[test]
    fn atm_unchanged_with_defaults() {
        let base = pos!(0.17);
        let strike = pos!(1500.0);
        let spot = pos!(1500.0);

        let out = adjust_volatility(
            &Some(base),
            &None,
            &None, // ambos -> 0
            &strike,
            &spot,
        )
        .unwrap();

        assert_eq!(out.to_dec(), base.to_dec());
    }

    /* 3 ─ factor > 1 se clampa al techo 1.0 */
    #[test]
    fn huge_positive_smile_clamps_upper() {
        let base = pos!(0.20);
        let strike = pos!(3000.0);
        let spot = pos!(1000.0);

        let smile = dec!(5.0);
        let out = adjust_volatility(&Some(base), &None, &Some(smile), &strike, &spot).unwrap();
        assert_eq!(out, base + 0.4);
    }

    /* 4 ─ factor < 0.01 se clampa al suelo 0.01 */
    /* factor < 0.01 se clampa al suelo 0.01 */
    #[test]
    fn extreme_moneyness_clamps_lower() {
        let base = pos!(0.30);
        // strike muy ITM → moneyness negativa grande
        let strike = pos!(10.0);
        let spot = pos!(1000.0);

        // pendiente positiva fuerte → 1 + (+)·(−) = 1 − algo grande < 0
        let skew = dec!(10.0);

        let out = adjust_volatility(
            &Some(base),
            &Some(skew),
            &None, // sin curvatura
            &strike,
            &spot,
        )
        .unwrap();

        let expected = base * pos!(0.01); // piso 1 %
        assert_relative_eq!(
            out.to_dec().to_f64().unwrap(),
            expected.to_dec().to_f64().unwrap(),
            epsilon = 1e-12
        );
    }

    #[test]
    fn negative_skew_increases_vol_below_atm() {
        let base = pos!(0.20);
        let strike = pos!(1000.0);
        let spot = pos!(1500.0);

        let skew = dec!(-1.0);
        let out = adjust_volatility(&Some(base), &Some(skew), &None, &strike, &spot).unwrap();

        assert!(out > base);
    }
}

#[cfg(test)]
mod tests_option_data_price_params {
    use super::*;
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn get_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("AAPL".to_string()),
        )
    }

    #[test]
    fn test_new_price_params() {
        let params = get_params();

        assert_eq!(*params.underlying_price.unwrap(), pos!(100.0));
        assert_eq!(
            params.expiration_date.unwrap().get_days().unwrap(),
            pos!(30.0)
        );
        assert_eq!(params.risk_free_rate.unwrap().to_f64().unwrap(), 0.05);
        assert_eq!(params.dividend_yield.unwrap().to_f64(), 0.02);
        assert_eq!(params.underlying_symbol.unwrap(), "AAPL");
    }

    #[test]
    fn test_default_price_params() {
        let params = OptionDataPriceParams::default();
        assert_eq!(params.underlying_price, None);
        assert_eq!(params.risk_free_rate, None);
        assert_eq!(params.dividend_yield, None);
        assert_eq!(params.underlying_symbol, None);
    }

    #[test]
    fn test_display_price_params() {
        let params = get_params();

        let display_string = format!("{params}");
        assert!(display_string.contains("Underlying Price: 100"));
        assert!(display_string.contains("Risk-Free Rate: 5"));
        assert!(display_string.contains("Dividend Yield: 2"));
        assert!(display_string.contains("Symbol: AAPL"));
        assert!(display_string.contains("Expiration: 0.08 Years"));
    }

    #[test]
    fn test_option_data_price_params_getters() {
        // Setup test parameters
        let underlying_price = Some(Box::new(pos!(100.0)));
        let expiration_date = Some(ExpirationDate::Days(pos!(30.0)));
        let risk_free_rate = Some(dec!(0.05));
        let dividend_yield = spos!(0.02);
        let underlying_symbol = Some("AAPL".to_string());

        let params = OptionDataPriceParams {
            underlying_price: underlying_price.clone(),
            expiration_date,
            risk_free_rate,
            dividend_yield,
            underlying_symbol: underlying_symbol.clone(),
        };

        // Test each getter
        assert_eq!(params.get_underlying_price(), underlying_price);
        assert_eq!(params.get_expiration_date(), expiration_date);
        assert_eq!(params.get_risk_free_rate(), risk_free_rate);
        assert_eq!(params.get_dividend_yield(), dividend_yield);
    }

    #[test]
    fn test_option_data_price_params_getters_with_datetime_expiration() {
        use chrono::{Duration, Utc};

        let future_date = Utc::now() + Duration::days(30);
        let expiration_date = Some(ExpirationDate::DateTime(future_date));

        let mut params = get_params();
        params.expiration_date = expiration_date;

        assert_eq!(params.get_expiration_date(), expiration_date);
    }

    #[test]
    fn test_option_data_price_params_getters_zero_values() {
        let mut params = get_params();
        params.underlying_price = Some(Box::new(pos!(0.0)));
        params.expiration_date = Some(ExpirationDate::Days(pos!(0.0)));
        params.risk_free_rate = Some(Decimal::ZERO);
        params.dividend_yield = Some(Positive::ZERO);

        assert_eq!(*params.get_underlying_price().unwrap(), Positive::ZERO);
        assert_eq!(
            params.get_expiration_date().unwrap(),
            ExpirationDate::Days(Positive::ZERO)
        );
        assert_eq!(params.get_risk_free_rate().unwrap(), Decimal::ZERO);
        assert_eq!(params.get_dividend_yield().unwrap(), Positive::ZERO);
    }
}

#[cfg(test)]
mod tests_option_chain_build_params {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn get_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("AAPL".to_string()),
        )
    }

    #[test]
    fn test_new_chain_build_params() {
        let price_params = get_params();

        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            spos!(1000.0),
            10,
            spos!(5.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
            pos!(0.25),
        );

        assert_eq!(params.symbol, "TEST");
        assert_eq!(params.volume, spos!(1000.0));
        assert_eq!(params.chain_size, 10);
        assert_eq!(params.strike_interval, spos!(5.0));
        assert_eq!(params.smile_curve, dec!(0.1));
        assert_eq!(params.spread, pos!(0.02));
        assert_eq!(params.decimal_places, 2);

        let display = format!("{params}");
        assert_eq!(
            display,
            r#"{"symbol":"TEST","volume":1000,"chain_size":10,"strike_interval":5,"skew_slope":"-0.2","smile_curve":"0.1","spread":0.02,"decimal_places":2,"price_params":{"underlying_price":100,"expiration_date":{"days":30.0},"risk_free_rate":"0.05","dividend_yield":0.02,"underlying_symbol":"AAPL"},"implied_volatility":0.25}"#
        );
    }

    #[test]
    fn test_chain_build_params_without_volume() {
        let price_params = OptionDataPriceParams::default();

        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            spos!(5.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
            pos!(0.25),
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
            Some("Epic".to_string()),
            None,
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
            Some("Epic".to_string()),
            None,
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
            Some("Epic".to_string()),
            None,
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
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_chain() {
        let chain = OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("AAPL".to_string()),
        );

        let params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            Some(Positive::ONE),
            5,
            Some(Positive::ONE),
            dec!(-0.2),
            dec!(0.0001),
            Positive::new(0.02).unwrap(),
            2,
            chain,
            pos!(0.25),
        );

        let built_chain = OptionChain::build_chain(&params);

        assert_eq!(built_chain.symbol, "AAPL");
        assert_eq!(built_chain.underlying_price, Positive::new(100.0).unwrap());
    }

    #[test]
    fn test_empty_string_round_to_2() {
        // Test with Some value
        let value = spos!(123.456);
        let result = empty_string_round_to_2(value);
        assert_eq!(result, "123.46");

        // Test with None
        let value: Option<Positive> = None;
        let result = empty_string_round_to_2(value);
        assert_eq!(result, "");
    }
}
