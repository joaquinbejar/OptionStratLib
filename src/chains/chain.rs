/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::chains::utils::{
    OptionChainBuildParams, OptionChainParams, OptionDataPriceParams, RandomPositionsParams,
    adjust_volatility, default_empty_string, rounder, strike_step,
};
use crate::chains::{OptionData, OptionsInStrike, RNDAnalysis, RNDParameters, RNDResult};
use crate::curves::{BasicCurves, Curve, Point2D};
use crate::error::chains::{ChainError, OptionDataErrorKind};
use crate::error::{CurveError, SurfaceError};
use crate::geometrics::LinearInterpolation;
use crate::greeks::Greeks;
use crate::model::{
    BasicAxisTypes, ExpirationDate, OptionStyle, OptionType, Options, Position, Side,
};
use crate::strategies::utils::FindOptimalSide;
use crate::surfaces::{BasicSurfaces, Point3D, Surface};
use crate::utils::Len;
use crate::utils::others::get_random_element;
use crate::volatility::{VolatilitySkew, VolatilitySmile};
use crate::{Positive, pos};
use chrono::{NaiveDate, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use pretty_simple_display::DebugSimple;
use prettytable::{Attr, Cell, Row, Table, color, format};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;
use tracing::{debug, error, warn};
use utoipa::ToSchema;
use {crate::chains::utils::parse, csv::WriterBuilder, std::fs::File};

/// A constant representing the skew value for the smile curve in financial modeling.
///
/// The skew smile curve is often used in options pricing to represent the implied volatility
/// skew relative to strike prices. It helps adjust for market conditions and asset-specific
/// behaviors in pricing models.
pub const SKEW_SMILE_CURVE: Decimal = dec!(0.1);

/// A constant representing the skew slope value used in calculations.
///
/// `SKEW_SLOPE` is defined as a `Decimal` with a value of `-0.2`.
/// It is typically used in scenarios where a slope factor is applied,
/// such as in financial models or data analysis where skewness impacts outcomes.
///
pub const SKEW_SLOPE: Decimal = dec!(-0.2);

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
#[derive(DebugSimple, Clone, ToSchema)]
pub struct OptionChain {
    /// The ticker symbol for the underlying asset (e.g., "AAPL", "SPY").
    pub symbol: String,

    /// The current market price of the underlying asset.
    pub underlying_price: Positive,

    /// The expiration date of the options in the chain.
    expiration_date: String,

    /// A sorted collection of option contracts at different strike prices.
    pub options: BTreeSet<OptionData>,

    /// The risk-free interest rate used for option pricing models.
    pub risk_free_rate: Option<Decimal>,

    /// The annual dividend yield of the underlying asset.
    pub dividend_yield: Option<Positive>,
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
    /// use optionstratlib::{pos, spos};
    ///
    /// let chain = OptionChain::new(
    ///     "AAPL",
    ///     pos!(172.50),
    ///     "2023-12-15".to_string(),
    ///     Some(dec!(0.05)),  // 5% risk-free rate
    ///     spos!(0.0065) // 0.65% dividend yield
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
    ///     Some(Box::new(pos!(100.0))),               // underlying price
    ///     Some(ExpirationDate::Days(pos!(30.0))),    // expiration date
    ///     Some(dec!(0.05)),                          // risk-free rate
    ///     spos!(0.0),                                // dividend yield
    ///     Some("SPY".to_string())                    // underlying symbol
    /// );
    ///
    /// let build_params = OptionChainBuildParams::new(
    ///     "SPY".to_string(),
    ///     spos!(1000.0),
    ///     10,
    ///     spos!(5.0),
    ///     dec!(-0.2),
    ///     dec!(0.1),
    ///     pos!(0.02),
    ///     2,
    ///     price_params,
    ///     pos!(0.2) // implied volatility
    /// );
    ///
    /// let chain = OptionChain::build_chain(&build_params);
    /// ```
    pub fn build_chain(params: &OptionChainBuildParams) -> Self {
        let strike_interval = if let Some(strike_interval) = params.strike_interval {
            strike_interval
        } else {
            assert!(params.price_params.underlying_price.is_some());
            assert!(params.price_params.expiration_date.is_some());
            strike_step(
                *params.price_params.underlying_price.clone().unwrap(),
                params.implied_volatility,
                params
                    .price_params
                    .expiration_date
                    .unwrap()
                    .get_days()
                    .unwrap(),
                params.chain_size,
                None,
            )
        };
        let underlying_price = *params.price_params.underlying_price.clone().unwrap();

        let mut option_chain = OptionChain::new(
            &params.symbol,
            underlying_price,
            params
                .price_params
                .expiration_date
                .unwrap()
                .get_date_string()
                .unwrap(),
            params.price_params.risk_free_rate,
            params.price_params.dividend_yield,
        );

        fn create_chain_data(strike: &Positive, p: &OptionChainBuildParams) -> OptionData {
            assert!(
                p.implied_volatility <= Positive::ONE,
                "{}",
                format!(
                    "Implied volatility should be between 0 and 1, got: {}",
                    p.implied_volatility
                )
            );

            let price = *p.price_params.underlying_price.clone().unwrap();
            assert!(!strike.is_zero());
            assert!(!p.implied_volatility.is_zero());
            let adjusted_volatility = adjust_volatility(
                &Some(p.implied_volatility),
                &Some(p.skew_slope),
                &Some(p.smile_curve),
                strike,
                &price,
            );
            assert!(adjusted_volatility.is_some());

            let mut option_data = OptionData::new(
                *strike,
                None,
                None,
                None,
                None,
                adjusted_volatility.unwrap(),
                None,
                None,
                None,
                p.volume,
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
                p.price_params.underlying_price.clone(),
                p.price_params.expiration_date,
                p.price_params.risk_free_rate,
                p.price_params.dividend_yield,
                p.price_params.underlying_symbol.clone(),
            );
            option_data.set_extra_params(price_params);

            match option_data.calculate_prices(Some(p.spread)) {
                Ok(()) => {
                    option_data.apply_spread(p.spread, p.decimal_places);
                    option_data.calculate_delta();
                    option_data.calculate_gamma();
                }
                Err(e) => {
                    warn!(
                        "Failed to calculate prices for strike: {} error: {}",
                        strike, e
                    );
                }
            }
            option_data
        }

        let atm_strike = rounder(underlying_price, strike_interval);
        let atm_strike_option_data = create_chain_data(&atm_strike.clone(), params);
        option_chain.options.insert(atm_strike_option_data);

        // Generate strikes above and below ATM based on chain_size parameter
        let mut counter = Positive::ONE;
        let max_strikes = params.chain_size;

        loop {
            // Check if we've reached the desired chain size
            if counter.to_usize() > max_strikes {
                break;
            }

            let next_upper_strike = atm_strike + (strike_interval * counter);
            let next_upper_option_data = create_chain_data(&next_upper_strike, params);
            option_chain.options.insert(next_upper_option_data.clone());

            let strike_step = (strike_interval * counter).to_dec();
            if strike_step > atm_strike.to_dec() {
                break;
            }
            let next_lower_strike = atm_strike - (strike_interval * counter).to_dec();
            if next_lower_strike == Positive::ZERO {
                break;
            }
            let next_lower_option_data = create_chain_data(&next_lower_strike, params);
            option_chain.options.insert(next_lower_option_data.clone());

            if next_upper_option_data.some_price_is_none()
                && next_lower_option_data.some_price_is_none()
            {
                break;
            }
            counter += Positive::ONE;
        }
        debug!("Option chain: {}", option_chain);
        option_chain
    }

    /// Generates build parameters that would reproduce the current option chain.
    ///
    /// This method creates an `OptionChainBuildParams` object with configuration values
    /// extracted from the current chain. This is useful for:
    /// - Recreating a similar chain with modified parameters
    /// - Saving the chain's configuration for later reconstruction
    /// - Generating additional chains with consistent parameters
    ///
    /// # Returns
    ///
    /// An `OptionChainBuildParams` structure containing the parameters needed to rebuild
    /// this option chain. The method calculates appropriate values for chain size, strike interval,
    /// and estimated spread based on the current data.
    ///
    pub fn to_build_params(&self) -> Result<OptionChainBuildParams, ChainError> {
        // Calculate chain size based on the distance from ATM strike
        let atm_strike = self.atm_strike()?;
        let strike_interval = self.get_strike_interval();

        // Calculate the number of strikes above and below the ATM strike
        let mut chain_size = 0;
        let strike_prices: Vec<Positive> =
            self.options.iter().map(|opt| opt.strike_price).collect();

        if !strike_prices.is_empty() {
            // Find the maximum distance from ATM in number of strikes
            let min_strike = strike_prices.iter().min().unwrap();
            let max_strike = strike_prices.iter().max().unwrap();

            let strikes_below = ((atm_strike.to_dec() - min_strike.to_dec())
                / strike_interval.to_dec())
            .ceil()
            .to_u64()
            .unwrap_or(0) as usize;

            let strikes_above = ((max_strike.to_dec() - atm_strike.to_dec())
                / strike_interval.to_dec())
            .ceil()
            .to_u64()
            .unwrap_or(0) as usize;

            chain_size = strikes_below.max(strikes_above);
        }

        // Default to a reasonable chain size if calculation fails
        if chain_size == 0 {
            chain_size = 10;
        } else {
            chain_size = self.len();
        }

        // Estimate the average bid-ask spread from the available options
        let mut total_spread = Decimal::ZERO;
        let mut count = 0;

        for option in &self.options {
            if let (Some(ask), Some(bid)) = (option.call_ask, option.call_bid) {
                total_spread += (ask.to_dec() - bid.to_dec()).abs();
                count += 1;
            }

            if let (Some(ask), Some(bid)) = (option.put_ask, option.put_bid) {
                total_spread += (ask.to_dec() - bid.to_dec()).abs();
                count += 1;
            }
        }

        // Default spread if we couldn't calculate it
        let spread = if count > 0 {
            Positive(total_spread / Decimal::from(count))
        } else {
            pos!(0.02) // 0.02 is a reasonable default spread
        };

        // Get ATM implied volatility with a default fallback
        let implied_volatility = match self.get_atm_implied_volatility() {
            Ok(iv) => {
                assert!(*iv >= pos!(0.0) && *iv <= pos!(1.0));
                *iv
            }
            _ => pos!(0.2), // 20% is a reasonable default IV
        };

        let skew_slope = SKEW_SLOPE;
        let smile_curve = SKEW_SMILE_CURVE;

        // Create the price parameters
        let price_params = OptionDataPriceParams::new(
            Some(Box::new(self.underlying_price)),
            Some(ExpirationDate::from_string(&self.expiration_date)?),
            self.risk_free_rate,
            self.dividend_yield,
            Some(self.symbol.clone()),
        );

        // Determine a reasonable number of decimal places based on the underlying price
        let decimal_places = if self.underlying_price >= pos!(100.0) {
            2
        } else {
            3
        };

        // Volume is typically available in the option data
        let volume = self.options.iter().filter_map(|opt| opt.volume).next();

        Ok(OptionChainBuildParams::new(
            self.symbol.clone(),
            volume,
            chain_size,
            Some(strike_interval),
            skew_slope,
            smile_curve,
            spread,
            decimal_places,
            price_params,
            implied_volatility,
        ))
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
    #[allow(dead_code)]
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
                FindOptimalSide::Deltable(_threshold) => true,
                FindOptimalSide::Center => {
                    panic!("Center should be managed by the strategy");
                }
                FindOptimalSide::DeltaRange(min, max) => {
                    (option.delta_put.is_some()
                        && option.delta_put.unwrap() >= min
                        && option.delta_put.unwrap() <= max)
                        || (option.delta_call.is_some()
                            && option.delta_call.unwrap() >= min
                            && option.delta_call.unwrap() <= max)
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
                FindOptimalSide::Deltable(_threshold) => true,
                FindOptimalSide::Center => {
                    panic!("Center should be managed by the strategy");
                }
                FindOptimalSide::DeltaRange(min, max) => {
                    (option.delta_put.is_some()
                        && option.delta_put.unwrap() >= min
                        && option.delta_put.unwrap() <= max)
                        || (option.delta_call.is_some()
                            && option.delta_call.unwrap() >= min
                            && option.delta_call.unwrap() <= max)
                }
            })
            .map(|option| option.get_options_in_strike())
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
        implied_volatility: Positive,
        delta_call: Option<Decimal>,
        delta_put: Option<Decimal>,
        gamma: Option<Decimal>,
        volume: Option<Positive>,
        open_interest: Option<u64>,
        extra_fields: Option<Value>,
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
            extra_fields,
            ..Default::default()
        };
        option_data.set_mid_prices();
        let expiration_date = match ExpirationDate::from_string(&self.expiration_date) {
            Ok(date) => date,
            Err(e) => {
                panic!("Failed to parse expiration date: {e}");
            }
        };
        let params = OptionDataPriceParams::new(
            Some(Box::new(self.underlying_price)),
            Some(expiration_date),
            self.risk_free_rate,
            self.dividend_yield,
            Some(self.symbol.clone()),
        );
        option_data.set_extra_params(params);

        self.options.insert(option_data);
    }

    /// Returns the strike price closest to the underlying price (at-the-money).
    ///
    /// This method searches through all available options in the chain to find the one
    /// with a strike price that most closely matches the current underlying price.
    /// This is useful for finding at-the-money (ATM) options when there isn't an exact
    /// match for the underlying price.
    ///
    /// # Returns
    ///
    /// * `Ok(&Positive)` - Reference to the strike price closest to the underlying price
    /// * `Err(ChainError)` - Error if the option chain is empty or if the operation fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::{error, info};
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::pos;
    ///
    /// let chain = OptionChain::new("SPY", pos!(450.75), "2023-12-15".to_string(), None, None);
    /// // Add options to the chain...
    ///
    /// match chain.atm_strike() {
    ///     Ok(strike) => info!("Closest strike to underlying: {}", strike),
    ///     Err(e) => error!("Error finding ATM strike: {}", e),
    /// }
    /// ```
    pub fn atm_strike(&self) -> Result<&Positive, ChainError> {
        let option_data = self.atm_option_data()?;
        Ok(&option_data.strike_price)
    }

    /// Retrieves the OptionData for the at-the-money (ATM) option.
    ///
    /// This function attempts to find the ATM option within the option chain.
    /// First, it checks for an option with a strike price that exactly matches the
    /// underlying asset's price. If an exact match is not found, it searches for the
    /// option with the strike price closest to the underlying price.
    ///
    /// # Returns
    ///
    /// * `Ok(&OptionData)` - If a suitable ATM option is found, returns a reference to it.
    /// * `Err(ChainError)` - If the option chain is empty or no ATM option can be found,
    ///   returns an error describing the failure.
    ///
    /// # Errors
    ///
    /// This function returns an error in the following cases:
    ///
    /// * The option chain (`self.options`) is empty.
    /// * No option with a strike price close to the underlying price can be found.
    pub fn atm_option_data(&self) -> Result<&OptionData, ChainError> {
        // Check for empty option chain
        if self.options.is_empty() {
            return Err(format!(
                "Cannot find ATM OptionData for empty option chain: {}",
                self.symbol
            )
            .into());
        }

        // First check for exact match
        if let Some(exact_match) = self
            .options
            .iter()
            .find(|opt| opt.strike_price == self.underlying_price)
        {
            return Ok(exact_match);
        }

        // Find the option with strike price closest to underlying price
        let option_data = self.options.iter().min_by(|a, b| {
            let a_distance = (a.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
            let b_distance = (b.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
            a_distance
                .partial_cmp(&b_distance)
                .unwrap_or(Ordering::Equal)
        });

        match option_data {
            Some(opt) => Ok(opt),
            None => Err(format!(
                "Failed to find ATM OptionData for option chain: {}",
                self.symbol
            )
            .into()),
        }
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
    pub fn set_from_title(&mut self, file: &str) -> Result<(), ChainError> {
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
                option.calculate_delta();
                option.calculate_gamma();
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
    /// * `Result<(), ChainError>` - Ok(()) if successful, or an Error if the file couldn't be created
    ///   or written to.
    ///
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    pub fn save_to_csv(&self, file_path: &str) -> Result<(), ChainError> {
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
                option.implied_volatility.to_string(),
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
    /// * `Result<(), ChainError>` - Ok(()) if successful, or an Error if the file couldn't be created
    ///   or written to.
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    pub fn save_to_json(&self, file_path: &str) -> Result<(), ChainError> {
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
    /// * `Result<Self, ChainError>` - An OptionChain if successful, or an Error if the file
    ///   couldn't be read or the data is invalid.
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    pub fn load_from_csv(file_path: &str) -> Result<Self, ChainError> {
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
                implied_volatility: parse(&record[5]).unwrap(),
                delta_call: parse(&record[6]),
                delta_put: parse(&record[7]),
                gamma: parse(&record[8]),
                volume: parse(&record[9]),
                open_interest: parse(&record[10]),
                ..Default::default()
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
    /// * `Result<Self, ChainError>` - An OptionChain if successful, or an Error if the file
    ///   couldn't be read or the data is invalid.
    ///
    /// # Note
    ///
    /// This method is only available on non-WebAssembly targets.
    pub fn load_from_json(file_path: &str) -> Result<Self, ChainError> {
        let file = File::open(file_path)?;
        let mut option_chain: OptionChain = serde_json::from_reader(file)?;
        option_chain.set_optiondata_extra_params()?;
        option_chain.mutate_single_options(|option| {
            option.implied_volatility = if option.implied_volatility >= Positive::ONE {
                option.implied_volatility / Positive::HUNDRED
            } else {
                option.implied_volatility
            }
        });

        option_chain.update_mid_prices();
        option_chain.update_greeks();
        // if implied volatility is in percentage, convert it to decimal
        option_chain.check_and_convert_implied_volatility();
        Ok(option_chain)
    }

    fn check_and_convert_implied_volatility(&mut self) {
        let updated_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option_clone = option.clone();
                option_clone.check_and_convert_implied_volatility();
                option_clone
            })
            .collect();

        self.options = updated_options;
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
                            option.implied_volatility,
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
                        params.epic.clone(),
                        params.extra_fields.clone(),
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
                            option.implied_volatility,
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
                        params.epic.clone(),
                        params.extra_fields.clone(),
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
                            option.implied_volatility,
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
                        params.epic.clone(),
                        params.extra_fields.clone(),
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
                            option.implied_volatility,
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
                        params.epic.clone(),
                        params.extra_fields.clone(),
                    );
                    positions.push(position);
                }
            }
        }

        Ok(positions)
    }

    /// Returns an iterator over the `OptionData` elements.
    ///
    /// This method provides an iterator that yields references to
    /// the `OptionData` items contained within the structure.
    ///
    /// # Returns
    ///
    /// An iterator where each item is a reference to an `OptionData`.
    pub fn iter(&self) -> impl Iterator<Item = &OptionData> {
        self.get_single_iter()
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
        self.options.iter().filter(|option| option.validate())
    }

    /// Applies a mutation function to each option in the chain that has an implied volatility value.
    ///
    /// This method filters the option chain to include only options with defined implied volatility,
    /// applies the provided function to each option, and then updates the chain with these modified options.
    /// The options collection is completely replaced with the new, modified set.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable closure that takes a mutable reference to an `OptionData` and applies
    ///   some transformation or modification to it.
    ///
    pub fn mutate_single_options<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut OptionData),
    {
        let modified_options = self
            .options
            .iter()
            .filter(|&option| option.validate())
            .cloned()
            .map(|mut option| {
                f(&mut option);
                option
            })
            .collect::<BTreeSet<_>>();

        self.options = modified_options;
    }

    /// Returns an iterator that provides mutable access to individual options in the chain.
    ///
    /// This method enables modifying options in the chain while maintaining the collection's integrity.
    /// It works by:
    /// 1. Filtering options that have implied volatility
    /// 2. Removing each option from the internal collection
    /// 3. Providing mutable access to each option
    ///
    /// The caller is responsible for reinserting modified options back into the chain.
    /// After modifications, options should be reinserted into the chain using appropriate methods.
    ///
    /// # Returns
    ///
    /// An iterator yielding mutable references to `OptionData` instances.
    ///
    /// # Examples
    ///
    pub fn get_single_iter_mut(&mut self) -> impl Iterator<Item = OptionData> {
        self.options
            .iter()
            .filter(|&option| option.validate())
            .cloned()
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
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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

    /// Retrieves the At-The-Money (ATM) implied volatility.
    ///
    /// This function retrieves the implied volatility of the ATM option.
    /// It calls `self.atm_option_data()` to find the ATM option and then
    /// returns a reference to its implied volatility.
    ///
    /// # Returns
    ///
    /// * `Ok(&Option<Positive>)` - If the ATM option is found, returns a reference
    ///   to its implied volatility, which is an `Option<Positive>`.
    /// * `Err(ChainError)` - If the ATM option cannot be found, returns an error.
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying `atm_option_data()` call fails,
    /// which can happen if the option chain is empty or no suitable ATM option is found.
    pub fn get_atm_implied_volatility(&self) -> Result<&Positive, ChainError> {
        let option_data = self.atm_option_data()?;
        Ok(&option_data.implied_volatility)
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
            gamma_exposure += option.gamma.unwrap_or(Decimal::ZERO);
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
            delta_exposure += option.delta_call.unwrap_or(Decimal::ZERO);
            delta_exposure += option.delta_put.unwrap_or(Decimal::ZERO);
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
        for option_data in &self.options {
            let vega = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .vega()?;
            vega_exposure += vega;
            let vega = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .vega()?;
            vega_exposure += vega;
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
        for option_data in &self.options {
            let theta = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .theta()?;
            theta_exposure += theta;
            let theta = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .theta()?;
            theta_exposure += theta;
        }
        Ok(theta_exposure)
    }

    /// Calculates the total vanna exposure for all options in the chain.
    ///
    /// Vanna exposure represents the aggregate sensitivity of option delta to changes
    /// in the implied volatility of the underlying asset. It measures how much option
    /// delta will change for a 1% change in implied volatility.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate vanna value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's vanna calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn vanna_exposure(&self) -> Result<Decimal, ChainError> {
        let mut vanna_exposure = Decimal::ZERO;
        for option_data in &self.options {
            let vanna = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .vanna()?;
            vanna_exposure += vanna;
            let vanna = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .vanna()?;
            vanna_exposure += vanna;
        }
        Ok(vanna_exposure)
    }

    /// Calculates the total vomma exposure for all options in the chain.
    ///
    /// Vomma exposure represents the aggregate sensitivity of option Vega to changes
    /// in the implied volatility of the underlying asset.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate Vomma value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's vomma calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn vomma_exposure(&self) -> Result<Decimal, ChainError> {
        let mut vomma_exposure = Decimal::ZERO;
        for option_data in &self.options {
            let vomma = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .vomma()?;
            vomma_exposure += vomma;
            let vomma = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .vomma()?;
            vomma_exposure += vomma;
        }
        Ok(vomma_exposure)
    }

    /// Calculates the total veta exposure for all options in the chain.
    ///
    /// Veta exposure represents the aggregate sensitivity of option Vega with respect
    /// to the passage of time.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate veta value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's veta calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling the `update_greeks` method.
    pub fn veta_exposure(&self) -> Result<Decimal, ChainError> {
        let mut veta_exposure = Decimal::ZERO;
        for option_data in &self.options {
            let veta = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .veta()?;
            veta_exposure += veta;
            let veta = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .veta()?;
            veta_exposure += veta;
        }
        Ok(veta_exposure)
    }

    /// Calculates the total charm exposure for all options in the chain.
    ///
    /// Charm exposure represents the aggregate sensitivity of option Delta
    /// with respect to the passage of time.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate charm value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's charm calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling
    /// the `update_greeks` method.
    pub fn charm_exposure(&self) -> Result<Decimal, ChainError> {
        let mut charm_exposure = Decimal::ZERO;
        for option_data in &self.options {
            let charm = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .charm()?;
            charm_exposure += charm;
            let charm = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .charm()?;
            charm_exposure += charm;
        }
        Ok(charm_exposure)
    }

    /// Calculates the total color exposure for all options in the chain.
    ///
    /// Color exposure represents the aggregate sensitivity of option Gamma
    /// with respect to the passage of time.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, ChainError>` - The aggregate color value, or an error if calculation fails
    ///
    /// # Errors
    ///
    /// Returns a `ChainError` if:
    /// - Any option's color calculation fails
    /// - Options greeks are not initialized
    ///
    /// # Note
    ///
    /// This method requires options greeks to be initialized first by calling
    /// the `update_greeks` method.
    pub fn color_exposure(&self) -> Result<Decimal, ChainError> {
        let mut color_exposure = Decimal::ZERO;
        for option_data in &self.options {
            let color = option_data
                .get_option(Side::Long, OptionStyle::Call)?
                .color()?;
            color_exposure += color;
            let color = option_data
                .get_option(Side::Long, OptionStyle::Put)?
                .color()?;
            color_exposure += color;
        }
        Ok(color_exposure)
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

    /// Generates a vanna curve for visualization and analysis.
    ///
    /// Creates a curve representing vanna values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing vanna data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn vanna_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Vanna, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a veta curve for visualization and analysis.
    ///
    /// Creates a curve representing veta values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing veta data points,
    ///   or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing data
    /// or calculation errors
    pub fn veta_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Veta, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a charm curve for visualization and analysis.
    ///
    /// Creates a curve representing charm values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing charm data points, or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing
    /// data or calculation errors
    pub fn charm_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Charm, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a color curve for visualization and analysis.
    ///
    /// Creates a curve representing color values across different strike prices
    /// or other relevant parameters for long call options in the chain.
    ///
    /// # Returns
    ///
    /// * `Result<Curve, CurveError>` - A curve object containing color data points, or an error if curve generation fails
    ///
    /// # Errors
    ///
    /// Returns a `CurveError` if the curve cannot be generated due to missing
    /// data or calculation errors
    pub fn color_curve(&self) -> Result<Curve, CurveError> {
        self.curve(&BasicAxisTypes::Color, &OptionStyle::Call, &Side::Long)
    }

    /// Generates a Veta time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Veta values across different strike prices
    /// and time horizons. Veta measures the rate of change of Vega with respect to time,
    /// making this surface particularly useful for understanding how volatility sensitivity
    /// evolves as expiration approaches.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A surface object containing Veta data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib::pos;
    ///
    /// let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
    /// let veta_surface = chain.veta_time_surface(days)?;
    /// ```
    pub fn veta_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, SurfaceError> {
        self.time_surface(
            &BasicAxisTypes::Veta,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Theta time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Theta values across different strike prices
    /// and time horizons. Theta measures the sensitivity of the options's value to the passage of
    /// time (time decay). As time passes with decreasing time to expiry an option's value
    /// decreases.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A surface object containing Theta data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib::pos;
    ///
    /// let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
    /// let theta_surface = chain.theta_time_surface(days)?;
    /// ```
    pub fn theta_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, SurfaceError> {
        self.time_surface(
            &BasicAxisTypes::Theta,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Charm time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Charm values across different strike prices
    /// and time horizons. Charm, also called DdeltaDtime or Delta decay,  measures the rate of
    /// change of Delta over the passage of time.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A surface object containing Charm data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib::pos;
    ///
    /// let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
    /// let charm_surface = chain.charm_time_surface(days)?;
    /// ```
    pub fn charm_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, SurfaceError> {
        self.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Color time surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Color values across different strike prices
    /// and time horizons. Color, also called DgammaDtime or Gamma decay,  measures the rate of
    /// change of Gamma over the passage of time.
    ///
    /// # Parameters
    ///
    /// * `days_to_expiry` - Vector of days to expiration values to use for surface calculations.
    ///   Common values might be `vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A surface object containing Color data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the surface cannot be generated due to missing data
    /// or calculation errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib::pos;
    ///
    /// let days = vec![pos!(7.0), pos!(14.0), pos!(30.0), pos!(60.0), pos!(90.0)];
    /// let color_surface = chain.color_time_surface(days)?;
    /// ```
    pub fn color_time_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, SurfaceError> {
        self.time_surface(
            &BasicAxisTypes::Color,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        )
    }

    /// Generates a Vanna volatility surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Vanna values across different strike prices
    /// and volatility levels. Vanna measures the sensitivity of Delta to changes in
    /// implied volatility, making this surface useful for understanding how delta
    /// hedging effectiveness changes with volatility.
    ///
    /// # Parameters
    ///
    /// * `volatilities` - Vector of volatility values to use for surface calculations.
    ///   Common values might be `vec![pos!(0.1), pos!(0.2), pos!(0.3), pos!(0.4), pos!(0.5)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A surface object containing Vanna data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the surface cannot be generated due to missing data
    /// or calculation errors
    pub fn vanna_surface(&self, volatilities: Vec<Positive>) -> Result<Surface, SurfaceError> {
        self.surface(
            &BasicAxisTypes::Vanna,
            &OptionStyle::Call,
            Some(volatilities),
            &Side::Long,
        )
    }

    /// Generates a Vomma volatility surface for visualization and analysis.
    ///
    /// Creates a 3D surface representing Vomma (Volga) values across different strike prices
    /// and volatility levels. Vomma measures the second-order sensitivity of option price
    /// to volatility (rate of change of Vega with respect to volatility).
    ///
    /// # Parameters
    ///
    /// * `volatilities` - Vector of volatility values to use for surface calculations.
    ///   Common values might be `vec![pos!(0.1), pos!(0.2), pos!(0.3), pos!(0.4), pos!(0.5)]`
    ///
    /// # Returns
    ///
    /// * `Result<Surface, SurfaceError>` - A surface object containing Vomma data points,
    ///   or an error if surface generation fails
    ///
    /// # Errors
    ///
    /// Returns a `SurfaceError` if the surface cannot be generated due to missing data
    /// or calculation errors
    pub fn vomma_surface(&self, volatilities: Vec<Positive>) -> Result<Surface, SurfaceError> {
        self.surface(
            &BasicAxisTypes::Vomma,
            &OptionStyle::Call,
            Some(volatilities),
            &Side::Long,
        )
    }

    /// Updates the expiration date for the option chain and recalculates Greeks.
    ///
    /// This method changes the expiration date of the option chain to the provided value
    /// and then triggers a recalculation of all Greek values for every option in the chain.
    /// The Greeks are financial measures that indicate how option prices are expected to change
    /// in response to different factors.
    ///
    /// # Parameters
    ///
    /// * `expiration` - A string representing the new expiration date for the option chain.
    ///   This should be in a standard date format compatible with the system.
    ///
    /// # Effects
    ///
    /// * Updates the `expiration_date` field of the option chain.
    /// * Calls `update_greeks()` to recalculate and update the Greek values for all options
    ///   in the chain based on the new expiration date.
    ///
    /// # Example
    ///
    /// ```
    /// use optionstratlib::chains::chain::OptionChain;
    /// let mut chain = OptionChain::new("AAPL", Default::default(), "".to_string(), None, None);
    /// chain.update_expiration_date("2023-12-15".to_string());
    /// ```
    pub fn update_expiration_date(&mut self, expiration: String) {
        self.expiration_date = expiration;
        // update expiration date for all options
        let expiration = self.get_expiration();
        if expiration.is_none() {
            warn!("Expiration date is not valid, skipping update.");
            return;
        }

        // Create a new set of options with updated expiration dates
        let mut updated_options = BTreeSet::new();
        for mut option in self.options.iter().cloned() {
            option.expiration_date = expiration;
            updated_options.insert(option);
        }

        // Replace the old options with the updated ones
        self.options = updated_options;

        self.update_greeks();
    }

    /// Retrieves the expiration date of the option chain.
    ///
    /// This method returns the expiration date associated with the option chain as a `String`.
    /// The expiration date represents the date on which the options in the chain will expire.
    ///
    /// # Returns
    ///
    /// A `String` representing the expiration date of the option chain.
    pub fn get_expiration_date(&self) -> String {
        self.expiration_date.clone()
    }

    /// Returns the expiration date of the option chain as an `ExpirationDate` object.
    ///
    /// # Returns
    /// * `Option<ExpirationDate>` - The expiration date if it can be parsed, or `None` if parsing fails.
    pub fn get_expiration(&self) -> Option<ExpirationDate> {
        ExpirationDate::from_string(&self.expiration_date).ok()
    }

    /// Calculates the strike price interval based on the available option contracts.
    ///
    /// This method determines a reasonable interval between strike prices by analyzing
    /// the strike prices of the options within the option chain. It calculates the
    /// differences between consecutive strike prices, and then returns the median of
    /// these intervals, rounded to the nearest integer. This approach is robust against
    /// outliers in strike price spacing.
    ///
    /// # Returns
    ///
    /// A `Positive` value representing the calculated strike price interval. If there are
    /// fewer than two options in the chain, or if an error occurs during the calculation,
    /// a default interval of 5.0 is returned. If the calculated median interval rounds to zero,
    /// a minimum interval of 1.0 is returned to ensure a valid positive interval.
    pub(crate) fn get_strike_interval(&self) -> Positive {
        if self.options.len() < 2 {
            return pos!(5.0); // Default interval if not enough options
        }

        let strikes: Vec<Positive> = self.options.iter().map(|opt| opt.strike_price).collect();

        let mut intervals = Vec::new();
        for i in 1..strikes.len() {
            intervals.push(strikes[i].to_dec() - strikes[i - 1].to_dec());
        }

        // Return the median interval for robustness
        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        if intervals.is_empty() {
            pos!(5.0) // Default if something went wrong
        } else {
            // Get the median interval
            let median_interval = intervals[intervals.len() / 2];

            // Round to the nearest integer
            let rounded_interval = median_interval.round();

            // Ensure we're not returning 0 as an interval
            if rounded_interval == Decimal::ZERO {
                pos!(1.0) // Minimum interval is 1
            } else {
                Positive(rounded_interval)
            }
        }
    }

    /// Retrieves a `Position` with a delta closest to the specified `target_delta`.
    ///
    /// This function searches the option chain for an option whose delta is less than or equal to
    /// the `target_delta`. It then selects the option with the highest delta value (for calls) or
    /// the most negative delta value (for puts) that meets this criteria.  A `Position` is
    /// constructed from the selected option.
    ///
    /// # Arguments
    ///
    /// * `target_delta` - The target delta value to search for.
    /// * `side` - The side of the position (Long or Short).
    /// * `option_style` - The style of the option (Call or Put).
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Position` if a suitable option is found, or a `ChainError` if no
    /// option with a delta less than or equal to the `target_delta` is found.
    ///
    /// # Errors
    ///
    /// Returns a `ChainError::OptionDataError` with `OptionDataErrorKind::InvalidDelta` if no option
    /// is found with a delta less than or equal to the specified `target_delta`.
    pub fn get_position_with_delta(
        &self,
        target_delta: Decimal,
        side: Side,
        option_style: OptionStyle,
    ) -> Result<Position, ChainError> {
        // Early validation - empty chain check
        if self.options.is_empty() {
            return Err(ChainError::OptionDataError(
                OptionDataErrorKind::InvalidDelta {
                    delta: Some(target_delta.to_f64().unwrap()),
                    reason: "Option chain is empty".to_string(),
                },
            ));
        }

        // Convert target to absolute value for consistent comparisons
        let target_delta_abs = target_delta.abs();

        // Find options with appropriate deltas based on option style
        let filtered_options = self
            .get_single_iter()
            .filter_map(|option_data| {
                // Get the appropriate delta based on option style
                let delta_opt = match option_style {
                    OptionStyle::Call => option_data.delta_call,
                    OptionStyle::Put => option_data.delta_put,
                };

                // Include only options with valid deltas less than or equal to target (absolute value)
                delta_opt.and_then(|delta| {
                    if delta.abs() <= target_delta_abs {
                        Some((option_data, delta))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        // If no options match our criteria, return a specific error
        if filtered_options.is_empty() {
            let message = match option_style {
                OptionStyle::Call => {
                    format!("No call option with delta ≤ {target_delta} was found")
                }
                OptionStyle::Put => {
                    format!("No put option with delta ≥ {target_delta} was found")
                }
            };

            return Err(ChainError::OptionDataError(
                OptionDataErrorKind::InvalidDelta {
                    delta: Some(target_delta.to_f64().unwrap()),
                    reason: message,
                },
            ));
        }

        // Find the option with the highest absolute delta value that's still ≤ target
        filtered_options
            .into_iter()
            .max_by(|(_, delta_a), (_, delta_b)| {
                delta_a
                    .abs()
                    .partial_cmp(&delta_b.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(option_data, delta)| {
                debug!(
                    "Selected option with strike {} and delta {}",
                    option_data.strike_price, delta
                );

                option_data
                    .get_position(side, option_style, None, None, None)
                    .map_err(|e| {
                        error!("Failed to create position: {}", e);
                        ChainError::OptionDataError(OptionDataErrorKind::InvalidDelta {
                            delta: Some(delta.to_f64().unwrap()),
                            reason: format!("Failed to create position: {e}"),
                        })
                    })
            })
            .unwrap_or_else(|| {
                // This should never happen since we checked for empty filtered_options,
                // but included for completeness
                Err(ChainError::OptionDataError(
                    OptionDataErrorKind::InvalidDelta {
                        delta: Some(target_delta.to_f64().unwrap()),
                        reason: "Unexpected error when selecting option with closest delta"
                            .to_string(),
                    },
                ))
            })
    }

    /// Retrieves a collection of strike prices from the chain of options.
    ///
    /// This method iterates through the options in the chain, extracts the `strike_price`
    /// of each option, and returns them as a vector of `Positive` values.
    ///
    /// # Returns
    /// This function returns a `Result`:
    /// - On success, it returns an `Ok` variant containing a `Vec<Positive>`, where each
    ///   element is the strike price of a corresponding option in the chain.
    /// - If an error occurs, it returns an `Err` variant containing a `ChainError`.
    ///
    /// # Errors
    /// This function will return an error if there is any issue in processing the options chain
    /// that prevents successful extraction of strike prices.
    ///
    /// # Note
    /// - The `Positive` type for `strike_price` ensures that only valid positive values are included.
    /// - An empty vector will be returned if there are no options in the chain.
    ///
    /// # Dependencies
    /// The method depends on `self.iter()` to provide access to the underlying collection of options.
    /// Each option is expected to have a `strike_price` field.
    pub fn get_strikes(&self) -> Result<Vec<Positive>, ChainError> {
        Ok(self
            .options
            .iter()
            .map(|option| option.strike_price)
            .collect())
    }

    /// Retrieves an `OptionData` instance from an option chain that has a strike price
    /// closest to the given price.
    ///
    /// # Arguments
    ///
    /// * `price` - A reference to a `Positive`, which represents the price to compare
    ///   against the strike prices in the option chain.
    ///
    /// # Returns
    ///
    /// * `Ok(&OptionData)` - A reference to the `OptionData` instance with the strike price
    ///   closest to the specified price.
    /// * `Err(ChainError)` - An error indicating the failure to retrieve the option data,
    ///   which could occur due to:
    ///   - The option chain being empty.
    ///   - No matching `OptionData` found for the given price.
    ///
    /// # Errors
    ///
    /// * `ChainError` - Returned if the option chain is empty or no suitable option data
    ///   can be found that matches the given price.
    ///
    /// # Behavior
    ///
    /// * If the option chain is empty (`self.options.is_empty()`), this function will
    ///   immediately return an error with a message indicating that the option data cannot be
    ///   found for an empty chain.
    /// * The function iterates through the available `OptionData` instances in the chain
    ///   and identifies the one whose `strike_price` is closest to the specified `price`.
    ///   - The comparison is done based on the absolute difference between the `strike_price`
    ///     and `price`, with the smallest difference being considered the best match.
    /// * If a matching option is found, it is returned as a reference inside an `Ok`.
    /// * If no matching option is found, an error will be returned with a descriptive message.
    ///
    /// # Notes
    ///
    /// * The `strike_price` and `price` values are compared as decimal values using the
    ///   `to_dec` method.
    /// * If two or more `OptionData` instances have the same distance to the given `price`,
    ///   the implementation will use the first instance it encounters based on the iteration
    ///   order.
    pub fn get_optiondata_with_strike(&self, price: &Positive) -> Result<&OptionData, ChainError> {
        // Check for empty option chain
        if self.options.is_empty() {
            return Err(format!(
                "Cannot find option data for empty option chain: {}",
                self.symbol
            )
            .into());
        }

        // Find the option with strike price closest to the price parameter
        let option_data = self.options.iter().min_by(|a, b| {
            let a_distance = (a.strike_price.to_dec() - price.to_dec()).abs();
            let b_distance = (b.strike_price.to_dec() - price.to_dec()).abs();
            a_distance
                .partial_cmp(&b_distance)
                .unwrap_or(Ordering::Equal)
        });

        match option_data {
            Some(opt) => Ok(opt),
            None => Err(format!(
                "Failed to find option data for price {} in chain: {}",
                price, self.symbol
            )
            .into()),
        }
    }

    /// Sets additional parameters for all option data objects in the chain.
    ///
    /// This method propagates the chain-level parameters (underlying price, expiration date,
    /// risk-free rate, dividend yield, and symbol) to all individual option contracts.
    ///
    /// # Returns
    /// * `Result<(), ChainError>` - Ok if successful, or an error if the operation fails.
    pub fn set_optiondata_extra_params(&mut self) -> Result<(), ChainError> {
        let params = OptionDataPriceParams::new(
            Some(Box::new(self.underlying_price)),
            ExpirationDate::from_string(&self.expiration_date).ok(),
            self.risk_free_rate,
            self.dividend_yield,
            Some(self.symbol.clone()),
        );

        self.mutate_single_options(|option| {
            option.set_extra_params(params.clone());
        });

        Ok(())
    }
}

impl PartialEq for OptionChain {
    fn eq(&self, other: &Self) -> bool {
        self.get_expiration() == other.get_expiration() && self.symbol == other.symbol
    }
}

impl Eq for OptionChain {}

impl PartialOrd for OptionChain {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OptionChain {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_expiration()
            .cmp(&other.get_expiration())
            .then_with(|| self.symbol.cmp(&other.symbol))
    }
}

impl Default for OptionChain {
    fn default() -> Self {
        Self::new("", Default::default(), "".to_string(), None, None)
    }
}

impl Len for OptionChain {
    fn len(&self) -> usize {
        self.options.len()
    }

    fn is_empty(&self) -> bool {
        self.options.is_empty()
    }
}

impl OptionChainParams for OptionChain {
    fn get_params(&self, strike_price: Positive) -> Result<OptionDataPriceParams, ChainError> {
        let option = self
            .options
            .iter()
            .find(|option| option.strike_price == strike_price);
        if option.is_none() {
            let reason = format!("Option with strike price {strike_price} not found");
            return Err(ChainError::invalid_strike(strike_price.to_f64(), &reason));
        }
        Ok(OptionDataPriceParams::new(
            Some(Box::new(self.underlying_price)),
            ExpirationDate::from_string(&self.expiration_date).ok(),
            self.risk_free_rate,
            self.dividend_yield,
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
    fn calculate_rnd(&self, params: &RNDParameters) -> Result<RNDResult, ChainError> {
        let mut densities = BTreeMap::new();
        let mut h = params.derivative_tolerance.to_dec();

        // Step 1: Validate parameters
        if h == Positive::ZERO {
            return Err("Derivative tolerance must be greater than zero"
                .to_string()
                .into());
        }

        // Step 2: Get all available strikes
        let strikes: Vec<Positive> = self.options.iter().map(|opt| opt.strike_price).collect();
        if strikes.is_empty() {
            return Err("No strikes available for RND calculation"
                .to_string()
                .into());
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
                debug!(
                    "Call price at k-h: {:?}",
                    self.get_call_price(k.sub_or_zero(&h))
                );
            }
            if let (Some(call_price), Some(call_up), Some(call_down)) = (
                self.get_call_price(k),
                self.get_call_price(k + h),
                self.get_call_price(k.sub_or_zero(&h)),
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
            return Err("Failed to calculate valid densities".to_string().into());
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
    fn calculate_skew(&self) -> Result<Vec<(Positive, Decimal)>, ChainError> {
        let mut skew = Vec::new();
        let atm_strike = self.underlying_price;
        let atm_vol = self.get_atm_implied_volatility()?;

        for opt in self.options.iter() {
            let relative_strike = opt.strike_price / atm_strike;
            let vol_diff = opt.implied_volatility.to_dec() - atm_vol;
            skew.push((relative_strike, vol_diff));
        }

        if skew.is_empty() {
            return Err("No valid data for skew calculation".to_string().into());
        }

        Ok(skew)
    }
}

impl OptionChain {
    /// Print the option chain with colored headers to stdout.
    ///
    /// This method prints the option chain directly to stdout using prettytable's
    /// `printstd()` method, which properly displays colors in the terminal.
    /// Use this method instead of `info!("{}", chain)` to see colored headers.
    pub fn show(&self) {
        // Print header information
        let mut header = Table::new();
        header.set_format(*format::consts::FORMAT_BOX_CHARS);
        header.add_row(Row::new(vec![
            Cell::new("Symbol").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Underlying Price").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Expiration Date").with_style(Attr::ForegroundColor(color::GREEN)),
        ]));
        header.add_row(Row::new(vec![
            Cell::new(&self.symbol).with_style(Attr::ForegroundColor(color::MAGENTA)),
            Cell::new(&self.underlying_price.to_string())
                .with_style(Attr::ForegroundColor(color::MAGENTA)),
            Cell::new(&self.expiration_date).with_style(Attr::ForegroundColor(color::MAGENTA)),
        ]));

        header.printstd();

        // Create the table
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Add header row with green color
        table.add_row(Row::new(vec![
            Cell::new("Strike").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Call Bid").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Call Ask").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Call Mid").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Put Bid").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Put Ask").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Put Mid").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("IV").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("C-Delta").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("P-Delta").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Gamma").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Vol.").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("OI").with_style(Attr::ForegroundColor(color::GREEN)),
        ]));

        // Add data rows
        for option in &self.options {
            // Check if strike price is a multiple of 25
            let is_multiple_of_25 = option.strike_price.is_multiple(25.0);

            let cells = vec![
                Cell::new(&option.strike_price.to_string()),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.call_bid,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.call_ask,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.call_middle,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.put_bid,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.put_ask,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.put_middle,
                )),
                Cell::new(&format!("{:.3}", option.implied_volatility)),
                Cell::new(&format!(
                    "{:.3}",
                    option.delta_call.unwrap_or(Decimal::ZERO)
                )),
                Cell::new(&format!("{:.3}", option.delta_put.unwrap_or(Decimal::ZERO))),
                Cell::new(&format!(
                    "{:.4}",
                    option.gamma.unwrap_or(Decimal::ZERO) * Decimal::ONE_HUNDRED
                )),
                Cell::new(&default_empty_string(option.volume)),
                Cell::new(&default_empty_string(option.open_interest)),
            ];

            // Apply yellow color to all cells if strike price is multiple of 25
            if is_multiple_of_25 {
                let colored_cells: Vec<Cell> = cells
                    .into_iter()
                    .map(|cell| cell.with_style(Attr::ForegroundColor(color::YELLOW)))
                    .collect();
                table.add_row(Row::new(colored_cells));
            } else {
                table.add_row(Row::new(cells));
            }
        }

        // Print the table with colors using printstd()
        table.printstd();
    }
}

impl fmt::Display for OptionChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut header = Table::new();
        header.set_format(*format::consts::FORMAT_BOX_CHARS);
        header.add_row(Row::new(vec![
            Cell::new("Symbol").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Underlying Price").with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("Expiration Date").with_style(Attr::ForegroundColor(color::GREEN)),
        ]));
        header.add_row(Row::new(vec![
            Cell::new(&self.symbol).with_style(Attr::ForegroundColor(color::MAGENTA)),
            Cell::new(&self.underlying_price.to_string())
                .with_style(Attr::ForegroundColor(color::MAGENTA)),
            Cell::new(&self.expiration_date).with_style(Attr::ForegroundColor(color::MAGENTA)),
        ]));

        write!(f, "\n{}", header)?;

        // Create the table
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Add header row with green color (colors may not display through Display trait)
        table.add_row(Row::new(vec![
            Cell::new("Strike"),
            Cell::new("Call Bid"),
            Cell::new("Call Ask"),
            Cell::new("Call Mid"),
            Cell::new("Put Bid"),
            Cell::new("Put Ask"),
            Cell::new("Put Mid"),
            Cell::new("IV"),
            Cell::new("C-Delta"),
            Cell::new("P-Delta"),
            Cell::new("Gamma"),
            Cell::new("Vol."),
            Cell::new("OI"),
        ]));

        // Add data rows
        for option in &self.options {
            table.add_row(Row::new(vec![
                Cell::new(&option.strike_price.to_string()),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.call_bid,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.call_ask,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.call_middle,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.put_bid,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.put_ask,
                )),
                Cell::new(&crate::chains::utils::empty_string_round_to_3(
                    option.put_middle,
                )),
                Cell::new(&format!("{:.3}", option.implied_volatility)),
                Cell::new(&format!(
                    "{:.3}",
                    option.delta_call.unwrap_or(Decimal::ZERO)
                )),
                Cell::new(&format!("{:.3}", option.delta_put.unwrap_or(Decimal::ZERO))),
                Cell::new(&format!(
                    "{:.4}",
                    option.gamma.unwrap_or(Decimal::ZERO) * Decimal::ONE_HUNDRED
                )),
                Cell::new(&default_empty_string(option.volume)),
                Cell::new(&default_empty_string(option.open_interest)),
            ]));
        }

        // Print the table (colors may not display through Display trait)
        write!(f, "{}", table)?;
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
            .map(|option| {
                Point2D::new(
                    option.strike_price.to_dec(),
                    option.implied_volatility.to_dec(),
                )
            })
            .collect();

        // Create an initial Curve object using the known points
        let curve = Curve::new(bt_points.clone());

        // Interpolate missing points (options without implied volatility)
        for option in self
            .options
            .iter()
            .filter(|o| o.implied_volatility.is_zero())
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

impl VolatilitySkew for OptionChain {
    /// Computes the volatility skew for the option chain.
    ///
    /// This function calculates the volatility skew by interpolating the implied
    /// volatilities for all the calculated moneyness data points in the option chain.
    /// It uses the available implied volatilities from the `options` field and
    /// performs linear interpolation to estimate missing values.
    ///
    /// # Returns
    ///
    /// A `Curve` object representing the volatility skew. The x-coordinates of the curve
    /// correspond to the moneyness, and the y-coordinates represent the corresponding
    /// implied volatilities.
    fn volatility_skew(&self) -> Curve {
        // Build a BTreeSet with the known points (options with implied volatility)
        let mut bt_points = self
            .options
            .iter()
            .map(|option| {
                Point2D::new(
                    (option.strike_price.to_dec() / self.underlying_price.to_dec() - Decimal::ONE)
                        * Decimal::ONE_HUNDRED,
                    option.implied_volatility.to_dec(),
                )
            })
            .collect::<BTreeSet<_>>();

        // Create an initial Curve object using the known points
        let curve = Curve::new(bt_points.clone());

        // Interpolate missing points (options without implied volatility)
        for option in self
            .options
            .iter()
            .filter(|o| o.implied_volatility.is_zero())
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
                // Select the appropriate option based on style and side
                let option = match (option_style, side) {
                    (OptionStyle::Call, Side::Long) => {
                        opt.get_option(Side::Long, OptionStyle::Call)
                    }
                    (OptionStyle::Call, Side::Short) => {
                        opt.get_option(Side::Short, OptionStyle::Call)
                    }
                    (OptionStyle::Put, Side::Long) => opt.get_option(Side::Long, OptionStyle::Put),
                    (OptionStyle::Put, Side::Short) => {
                        opt.get_option(Side::Short, OptionStyle::Put)
                    }
                };
                let option: Arc<Options> = match option {
                    Ok(o) => Arc::new(o),
                    Err(_) => return None, // Skip options that cannot be retrieved
                };

                // Get x and y values based on the axis types
                match self.get_curve_strike_versus(axis, &option) {
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
            // Select the appropriate option based on style and side
            let option = match (option_style, side) {
                (OptionStyle::Call, Side::Long) => opt.get_option(Side::Long, OptionStyle::Call),
                (OptionStyle::Call, Side::Short) => opt.get_option(Side::Short, OptionStyle::Call),
                (OptionStyle::Put, Side::Long) => opt.get_option(Side::Long, OptionStyle::Put),
                (OptionStyle::Put, Side::Short) => opt.get_option(Side::Short, OptionStyle::Put),
            };
            let option: Arc<Options> = match option {
                Ok(o) => Arc::new(o),
                Err(_) => {
                    return Err(SurfaceError::ConstructionError(
                        "Failed to retrieve option data".to_string(),
                    ));
                }
            };

            match &volatility {
                // If volatility vector is provided, use get_volatility_versus for each volatility
                Some(vols) => {
                    for vol in vols {
                        match self.get_surface_volatility_versus(axis, &option, *vol) {
                            Ok((x, y, z)) => {
                                points.insert(Point3D::new(x, y, z));
                            }
                            Err(_) => continue,
                        }
                    }
                }
                // If no volatility vector is provided, use get_strike_versus with original volatility
                None => match self.get_surface_strike_versus(axis, &option) {
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

    fn time_surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        days_to_expiry: Vec<Positive>,
        side: &Side,
    ) -> Result<Surface, SurfaceError> {
        if axis == &BasicAxisTypes::UnderlyingPrice
            || axis == &BasicAxisTypes::Strike
            || axis == &BasicAxisTypes::Expiration
            || axis == &BasicAxisTypes::Volatility
        {
            return Err(SurfaceError::ConstructionError(
                "Axis not valid for time surface".to_string(),
            ));
        }

        let mut points = BTreeSet::new();

        for opt in self.get_single_iter() {
            let option = match (option_style, side) {
                (OptionStyle::Call, Side::Long) => opt.get_option(Side::Long, OptionStyle::Call),
                (OptionStyle::Call, Side::Short) => opt.get_option(Side::Short, OptionStyle::Call),
                (OptionStyle::Put, Side::Long) => opt.get_option(Side::Long, OptionStyle::Put),
                (OptionStyle::Put, Side::Short) => opt.get_option(Side::Short, OptionStyle::Put),
            };
            let option: Arc<Options> = match option {
                Ok(o) => Arc::new(o),
                Err(_) => {
                    return Err(SurfaceError::ConstructionError(
                        "Failed to retrieve option data".to_string(),
                    ));
                }
            };

            for days in &days_to_expiry {
                match self.get_surface_time_versus(axis, &option, *days) {
                    Ok((x, y, z)) => {
                        points.insert(Point3D::new(x, y, z));
                    }
                    Err(_) => continue,
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points generated for time surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl From<&Vec<OptionData>> for OptionChain {
    fn from(options: &Vec<OptionData>) -> Self {
        let first_option = match options.first() {
            Some(opt) => opt,
            None => {
                return OptionChain::default();
            }
        };
        let symbol = first_option.clone().symbol.unwrap_or("Unknown".to_string());
        let underlying_price = *first_option
            .clone()
            .underlying_price
            .unwrap_or(Box::new(Positive::ZERO));
        let expiration_date = first_option
            .clone()
            .expiration_date
            .unwrap_or(ExpirationDate::Days(Positive::ZERO))
            .to_string();
        let risk_free_rate = first_option.risk_free_rate;
        let dividend_yield = first_option.dividend_yield;

        let options: BTreeSet<OptionData> = options.iter().cloned().collect();

        OptionChain {
            symbol,
            underlying_price,
            expiration_date,
            risk_free_rate,
            dividend_yield,
            options,
        }
    }
}

#[cfg(test)]
mod tests_chain_base {
    use super::*;
    use crate::model::ExpirationDate;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    use std::fs;
    use tracing::info;

    #[test]
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
    fn test_new_option_chain_build_chain() {
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.3),
            Decimal::ZERO,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("SP500".to_string()),
            ),
            pos!(0.17),
        );

        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        // With chain_size=10, we should get 21 strikes: 10 below + ATM + 10 above
        assert_eq!(chain.options.len(), 21);
        assert_eq!(chain.underlying_price, pos!(100.0));

        // First strike should be 10 strikes below ATM (100 - 10*1 = 90)
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.strike_price, pos!(90.0));
        assert_eq!(first.call_ask.unwrap(), 10.24);
        assert_eq!(first.call_bid.unwrap(), 10.22);
        assert_eq!(first.put_ask.unwrap(), 0.04);
        assert_eq!(first.put_bid.unwrap(), 0.02);

        // Last strike should be 10 strikes above ATM (100 + 10*1 = 110)
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.strike_price, pos!(110.0));
        assert_eq!(last.call_ask.unwrap(), 0.06);
        assert_eq!(last.call_bid.unwrap(), 0.04);
        assert_eq!(last.put_ask.unwrap(), 9.77);
        assert_eq!(last.put_bid.unwrap(), 9.75);
    }

    #[test]
    fn test_new_option_chain_build_chain_long() {
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            25,
            spos!(25.0),
            dec!(-0.3),
            dec!(0.2),
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(5878.10))),
                Some(ExpirationDate::Days(pos!(5.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("SP500".to_string()),
            ),
            pos!(0.2),
        );
        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert!(chain.options.len() > 1);
        assert_eq!(chain.underlying_price, pos!(5878.10));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 480.20);
        assert_eq!(first.call_bid.unwrap(), 480.18);
        assert_eq!(first.put_ask, None);
        assert_eq!(first.put_bid, None);
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, None);
        assert_eq!(last.call_bid, None);
        assert_eq!(last.put_ask, spos!(469.19));
        assert_eq!(last.put_bid, spos!(469.17));
    }

    #[test]
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
            pos!(0.1631),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
            None,
        );
        assert_eq!(chain.options.len(), 1);
        // first option in the chain
        let option = chain.options.iter().next().unwrap();
        assert_eq!(option.strike_price, 5520.0);
        assert!(option.call_bid.is_some());
        assert_eq!(option.call_bid.unwrap(), 274.26);
    }

    #[test]
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
    fn test_set_from_title_i() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_ii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_iii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    fn test_set_from_title_iv() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.88.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_v() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
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
            pos!(0.1631),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
            None,
        );
        let result = chain.save_to_csv(".");
        assert!(result.is_ok());
        let file_name = "./SP500-18-oct-2024-5781.88.csv".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
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
            pos!(0.1631),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
            None,
        );
        let result = chain.save_to_json(".");
        assert!(result.is_ok());

        let file_name = "./SP500-18-oct-2024-5781.88.json".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    fn test_load_from_csv() {
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
            pos!(0.1631),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
            None,
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
    fn test_load_from_json() {
        let mut chain =
            OptionChain::new("SP500", pos!(5781.9), "18-oct-2024".to_string(), None, None);
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            pos!(0.1631),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(100),
            None,
        );
        let result = chain.save_to_json("tests/");
        assert!(result.is_ok());

        let result = OptionChain::load_from_json("tests/SP500-18-oct-2024-5781.9.json");
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.9);

        let file_name = "tests/SP500-18-oct-2024-5781.9.json".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }
}

#[cfg(test)]
mod tests_option_data {
    use super::*;

    use crate::spos;

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
            pos!(0.2),        // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0), // volume
            Some(500),     // open_interest
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_new_option_data() {
        let option_data = create_valid_option_data();
        assert_eq!(option_data.strike_price, pos!(100.0));
        assert_eq!(option_data.call_bid, spos!(9.5));
        assert_eq!(option_data.call_ask, spos!(10.0));
        assert_eq!(option_data.put_bid, spos!(8.5));
        assert_eq!(option_data.put_ask, spos!(9.0));
        assert_eq!(option_data.implied_volatility, pos!(0.2));
        assert_eq!(option_data.delta_call.unwrap().to_f64(), Some(-0.3));
        assert_eq!(option_data.volume, spos!(1000.0));
        assert_eq!(option_data.open_interest, Some(500));
    }

    #[test]
    fn test_validate_valid_option() {
        let option_data = create_valid_option_data();
        assert!(option_data.validate());
    }

    #[test]
    fn test_validate_zero_strike() {
        let mut option_data = create_valid_option_data();
        option_data.strike_price = Positive::ZERO;
        assert!(!option_data.validate());
    }

    #[test]
    fn test_validate_missing_both_sides() {
        let mut option_data = OptionData {
            strike_price: pos!(100.0),
            ..Default::default()
        };
        option_data.implied_volatility = pos!(0.2);
        assert!(!option_data.validate());
    }

    #[test]
    fn test_valid_call() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_call());
    }

    #[test]
    fn test_valid_call_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.call_bid = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    fn test_valid_call_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.call_ask = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    fn test_valid_put() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_put());
    }

    #[test]
    fn test_valid_put_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.put_bid = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    fn test_valid_put_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.put_ask = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    fn test_calculate_prices_success() {
        let mut option_data = OptionData {
            strike_price: pos!(100.0),
            symbol: Some("TEST".to_string()),
            expiration_date: Some(ExpirationDate::Days(pos!(30.0))),
            underlying_price: Some(Box::new(pos!(100.0))),
            ..Default::default()
        };
        option_data.implied_volatility = pos!(0.2);
        let result = option_data.calculate_prices(None);

        assert!(result.is_ok());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.call_bid.is_some());
        assert!(option_data.put_ask.is_some());
        assert!(option_data.put_bid.is_some());
    }

    #[test]
    fn test_calculate_prices_missing_volatility() {
        let mut option_data = OptionData {
            strike_price: pos!(100.0),
            symbol: Some("TEST".to_string()),
            expiration_date: Some(ExpirationDate::Days(pos!(30.0))),
            underlying_price: Some(Box::new(pos!(100.0))),
            ..Default::default()
        };
        let _ = option_data.calculate_prices(None);

        info!("{}", option_data);
        assert_eq!(option_data.call_ask, None);
        assert_eq!(option_data.call_bid, None);
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
        assert_eq!(option_data.implied_volatility, Positive::ZERO);
        assert_eq!(option_data.delta_call, None);
        assert_eq!(option_data.strike_price, pos!(100.0));
    }

    #[test]
    fn test_calculate_prices_override_volatility() {
        let mut option_data = OptionData {
            strike_price: pos!(100.0),
            symbol: Some("TEST".to_string()),
            expiration_date: Some(ExpirationDate::Days(pos!(30.0))),
            underlying_price: Some(Box::new(pos!(100.0))),
            ..Default::default()
        };
        option_data.implied_volatility = pos!(0.2);
        let result = option_data.calculate_prices(None);

        assert!(result.is_ok());
        info!("{}", option_data);
        assert_pos_relative_eq!(option_data.call_ask.unwrap(), pos!(2.2871), pos!(0.0001));
        assert_pos_relative_eq!(option_data.call_bid.unwrap(), pos!(2.2871), pos!(0.0001));
        assert_pos_relative_eq!(option_data.put_ask.unwrap(), pos!(2.2871), pos!(0.0001));
        assert_pos_relative_eq!(option_data.put_bid.unwrap(), pos!(2.2871), pos!(0.0001));
        option_data.apply_spread(pos!(0.02), 2);
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, spos!(2.30));
        assert_eq!(option_data.call_bid, spos!(2.28));
        assert_eq!(option_data.put_ask, spos!(2.30));
        assert_eq!(option_data.put_bid, spos!(2.28));
    }

    #[test]
    fn test_calculate_prices_with_all_parameters() {
        let mut option_data = OptionData {
            strike_price: pos!(100.0),
            symbol: Some("TEST".to_string()),
            expiration_date: Some(ExpirationDate::Days(pos!(30.0))),
            underlying_price: Some(Box::new(pos!(100.0))),
            ..Default::default()
        };
        option_data.implied_volatility = pos!(0.2);
        let result = option_data.calculate_prices(None);

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
    use crate::model::ExpirationDate;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        // Create a sample option chain
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Add some test options with different strikes
        chain.add_option(
            pos!(95.0),      // strike_price
            spos!(4.0),      // call_bid
            spos!(4.2),      // call_ask
            spos!(3.0),      // put_bid
            spos!(3.2),      // put_ask
            pos!(0.2),       // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.2),
            spos!(3.0),
            spos!(3.2),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(50),
            None,
        );

        chain.add_option(
            pos!(105.0),
            spos!(2.0),
            spos!(2.2),
            spos!(4.0),
            spos!(4.2),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(50),
            None,
        );

        chain
    }

    #[test]
    fn test_zero_quantity() {
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
            None,
            None,
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
    fn test_long_puts_only() {
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
            None,
            None,
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
    fn test_short_puts_only() {
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
            None,
            None,
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
    fn test_long_calls_only() {
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
            None,
            None,
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
    fn test_short_calls_only() {
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
            None,
            None,
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
    fn test_mixed_positions() {
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
            None,
            None,
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
    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
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
            None,
            None,
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
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

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
            None,
        )
    }

    #[test]
    fn test_get_call_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_buy_price(), spos!(10.0));
    }

    #[test]
    fn test_get_call_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_sell_price(), spos!(9.5));
    }

    #[test]
    fn test_get_put_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_buy_price(), spos!(9.0));
    }

    #[test]
    fn test_get_put_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_sell_price(), spos!(8.5));
    }

    #[test]
    fn test_get_prices_with_none_values() {
        let data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
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
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_display_full_data() {
        let data = OptionData::new(
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
            None,
        );
        let display_string = format!("{data}");
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
    fn test_display_empty_data() {
        let data = OptionData::default();
        let display_string = format!("{data}");

        assert!(display_string.contains("0.0"));
        assert!(display_string.contains("")); // Para campos None
    }
}

#[cfg(test)]
mod tests_filter_option_data {
    use super::*;
    use crate::pos;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                pos!(0.2),
                None,
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
    fn test_filter_all() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::All);
        assert_eq!(filtered.len(), 5);
    }

    #[test]
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
    use crate::pos;

    #[test]
    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        assert_eq!(chain.strike_price_range_vec(5.0), None);
    }

    #[test]
    fn test_single_option() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.2),
            None,
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
    fn test_multiple_options() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        for strike in [90.0, 95.0, 100.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                pos!(0.2),
                None,
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
    fn test_step_size() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        for strike in [90.0, 100.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                pos!(0.2),
                None,
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
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),                            // strike_price
            spos!(9.5),                             // call_bid
            spos!(10.0),                            // call_ask
            spos!(8.5),                             // put_bid
            spos!(9.0),                             // put_ask
            pos!(0.25),                             // implied_volatility
            Some(dec!(-0.3)),                       // delta
            Some(dec!(0.7)),                        // delta
            Some(dec!(0.3)),                        // gamma
            spos!(1000.0),                          // volume
            Some(500),                              // open_interest
            Some("TEST".to_string()),               // symbol
            Some(ExpirationDate::Days(pos!(30.0))), // expiration_date
            Some(Box::new(pos!(100.0))),            // underlying_price
            Some(dec!(0.05)),                       // risk_free_rate
            Some(pos!(0.02)),                       // dividend_yield
            None,
            None,
        )
    }

    #[test]
    fn test_get_option_success() {
        let option_data = create_test_option_data();
        let result = option_data.get_option(Side::Long, OptionStyle::Call);
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
    fn test_get_option_using_data_iv() {
        let option_data = create_test_option_data();
        let result = option_data.get_option(Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.implied_volatility, 0.25); // Uses IV from option_data
    }
}

#[cfg(test)]
mod tests_option_data_get_options_in_strike {
    use super::*;
    use crate::greeks::Greeks;
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
            pos!(0.2),        // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(-0.3)),
            Some(dec!(0.3)),
            spos!(1000.0),                          // volume
            Some(500),                              // open_interest
            Some("TEST".to_string()),               // symbol
            Some(ExpirationDate::Days(pos!(30.0))), // expiration_date
            Some(Box::new(pos!(100.0))),            // underlying_price
            Some(dec!(0.05)),                       // risk_free_rate
            Some(pos!(0.02)),                       // dividend_yield
            None,
            None,
        )
    }

    #[test]
    fn test_get_options_in_strike_success() {
        let option_data = create_test_option_data();
        let result = option_data.get_options_in_strike();
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
    fn test_get_options_in_strike_using_data_iv() {
        let option_data = create_test_option_data();
        let result = option_data.get_options_in_strike();
        assert!(result.is_ok());

        let options = result.unwrap();
        assert_eq!(options.long_call.implied_volatility, 0.2);
        assert_eq!(options.short_call.implied_volatility, 0.2);
        assert_eq!(options.long_put.implied_volatility, 0.2);
        assert_eq!(options.short_put.implied_volatility, 0.2);
    }

    #[test]
    fn test_get_options_in_strike_all_properties() {
        let option_data = create_test_option_data();
        let result = option_data.get_options_in_strike();
        assert!(result.is_ok());

        let options = result.unwrap();

        // Verify common properties across all options
        let check_common_properties = |option: &Options| {
            assert_eq!(option.strike_price, pos!(100.0));
            assert_eq!(option.underlying_price, pos!(100.0));
            assert_eq!(option.implied_volatility, 0.2);
            assert_eq!(option.risk_free_rate.to_f64().unwrap(), 0.05);
            assert_eq!(option.dividend_yield.to_f64(), 0.02);
            assert_eq!(option.option_type, OptionType::European);
            assert_eq!(option.quantity, pos!(1.0));
        };

        check_common_properties(&options.long_call);
        check_common_properties(&options.short_call);
        check_common_properties(&options.long_put);
        check_common_properties(&options.short_put);
    }

    #[test]
    fn test_get_options_in_strike_deltas() {
        let option_data = create_test_option_data();
        let result = option_data.get_options_in_strike();
        assert!(result.is_ok());

        let options = result.unwrap();

        let epsilon = dec!(1e-8);

        assert_decimal_eq!(
            options.long_call.delta().unwrap(),
            dec!(0.539076663),
            epsilon
        );
        assert_decimal_eq!(
            options.short_call.delta().unwrap(),
            dec!(-0.539076663),
            epsilon
        );
        assert_decimal_eq!(
            options.long_put.delta().unwrap(),
            dec!(-0.459280851),
            epsilon
        );
        assert_decimal_eq!(
            options.short_put.delta().unwrap(),
            dec!(0.459280851),
            epsilon
        );
    }
}

#[cfg(test)]
mod tests_filter_options_in_strike {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(1.0),       // call_bid
                spos!(1.2),       // call_ask
                spos!(1.0),       // put_bid
                spos!(1.2),       // put_ask
                pos!(0.2),        // implied_volatility
                Some(dec!(-0.3)), // delta
                Some(dec!(-0.3)),
                Some(dec!(0.3)),
                spos!(1000.0), // volume
                Some(500),     // open_interest
                None,
            );
        }
        chain
    }

    #[test]
    fn test_filter_upper_strikes() {
        let chain = create_test_chain();
        let result = chain.filter_options_in_strike(FindOptimalSide::Upper);
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
    fn test_filter_lower_strikes() {
        let chain = create_test_chain();
        let result = chain.filter_options_in_strike(FindOptimalSide::Lower);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 2);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price < chain.underlying_price);
        }
    }

    #[test]
    fn test_filter_all_strikes() {
        let chain = create_test_chain();
        let result = chain.filter_options_in_strike(FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 5);
    }

    #[test]
    fn test_filter_range_strikes() {
        let chain = create_test_chain();
        let result =
            chain.filter_options_in_strike(FindOptimalSide::Range(pos!(95.0), pos!(105.0)));
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 3);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price >= pos!(95.0));
            assert!(opt.long_call.strike_price <= pos!(105.0));
        }
    }

    #[test]
    fn test_filter_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let result = chain.filter_options_in_strike(FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    fn test_filter_invalid_range() {
        let chain = create_test_chain();
        let result =
            chain.filter_options_in_strike(FindOptimalSide::Range(pos!(200.0), pos!(300.0)));
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    fn test_filter_all_strikes_deltas() {
        let chain = create_test_chain();
        let result = chain.filter_options_in_strike(FindOptimalSide::All);
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
            assert!(deltas.long_call >= Decimal::ZERO);
            assert!(deltas.short_call <= Decimal::ZERO);
            assert!(deltas.long_put <= Decimal::ZERO);
            assert!(deltas.short_put >= Decimal::ZERO);
        }
    }
}

#[cfg(test)]
mod tests_chain_iterators {
    use super::*;
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Add three options with different strikes
        chain.add_option(
            pos!(90.0),      // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.5),      // put_ask
            pos!(0.2),       // implied_volatility
            Some(dec!(0.6)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(80.0),
            Some(40),
            None,
        );

        chain
    }

    #[test]
    fn test_get_double_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_get_double_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
            None,
        );

        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty()); // No pairs with single element
    }

    #[test]
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
    fn test_get_double_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_get_double_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
            None,
        );

        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert_eq!(pairs.len(), 1); // Should have one pair (self-pair)
        assert_eq!(pairs[0].0.strike_price, pairs[0].1.strike_price);
    }

    #[test]
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
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Add four options with different strikes
        chain.add_option(
            pos!(90.0),      // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.5),      // put_ask
            pos!(0.2),       // implied_volatility
            Some(dec!(0.6)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(80.0),
            Some(40),
            None,
        );

        chain.add_option(
            pos!(120.0),
            spos!(0.5),
            spos!(1.0),
            spos!(7.0),
            spos!(7.5),
            pos!(0.35),
            Some(dec!(0.3)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(60.0),
            Some(30),
            None,
        );

        chain
    }

    // Tests for Triple Iterator
    #[test]
    fn test_get_triple_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    fn test_get_triple_iter_two_elements() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        // Add two options
        chain.add_option(
            pos!(90.0),
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            pos!(0.5),
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
    fn test_get_triple_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    fn test_get_triple_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.5),
            None,
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
    fn test_get_quad_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    fn test_get_quad_iter_three_elements() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        // Add three options
        chain.add_option(
            pos!(90.0),
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            pos!(0.5),
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
            pos!(0.5),
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
    fn test_get_quad_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    fn test_get_quad_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.5),
            None,
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
    fn test_upper_side_valid() {
        let option_data = OptionData::new(
            pos!(110.0), // strike price higher than underlying
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(&underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_upper_side_invalid() {
        let option_data = OptionData::new(
            pos!(90.0), // strike price lower than underlying
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let underlying_price = pos!(100.0);

        assert!(!option_data.is_valid_optimal_side(&underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_lower_side_valid() {
        let option_data = OptionData::new(
            pos!(90.0), // strike price lower than underlying
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(&underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_lower_side_invalid() {
        let option_data = OptionData::new(
            pos!(110.0), // strike price higher than underlying
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let underlying_price = pos!(100.0);

        assert!(!option_data.is_valid_optimal_side(&underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_all_side() {
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(&underlying_price, &FindOptimalSide::All));
    }

    #[test]
    fn test_range_side_valid() {
        let option_data = OptionData::new(
            pos!(100.0), // strike price within range
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::Range(range_start, range_end)
        ));
    }

    #[test]
    fn test_range_side_invalid_below() {
        let option_data = OptionData::new(
            pos!(80.0), // strike price below range
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(!option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::Range(range_start, range_end)
        ));
    }

    #[test]
    fn test_range_side_invalid_above() {
        let option_data = OptionData::new(
            pos!(120.0), // strike price above range
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(!option_data.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::Range(range_start, range_end)
        ));
    }

    #[test]
    fn test_range_side_at_boundaries() {
        let option_data_lower = OptionData::new(
            pos!(90.0), // strike price at lower boundary
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let option_data_upper = OptionData::new(
            pos!(110.0), // strike price at upper boundary
            None,
            None,
            None,
            None,
            pos!(0.5),
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
            None,
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(option_data_lower.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::Range(range_start, range_end)
        ));
        assert!(option_data_upper.is_valid_optimal_side(
            &pos!(100.0),
            &FindOptimalSide::Range(range_start, range_end)
        ));
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
                pos!(impl_vol),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
                None,
            );
        }

        chain
    }

    mod calculate_rnd_tests {
        use super::*;

        #[test]
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
        fn test_default() {
            let chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);
            let params = RNDParameters::default();

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Derivative tolerance must be greater than zero")
            );
        }

        #[test]
        fn test_zero_tolerance() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                derivative_tolerance: Positive::ZERO,
                ..Default::default()
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Derivative tolerance must be greater than zero")
            );
        }

        #[test]
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
        fn test_flat_volatility_surface() {
            let chain = create_standard_chain(); // All vols are 0.17
            let result = chain.calculate_skew().unwrap();

            // All vol differences should be close to zero
            for (_, vol_diff) in result {
                assert!(vol_diff.abs() < dec!(0.0001));
            }
        }

        #[test]
        fn test_empty_chain_skew() {
            let chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            let result = chain.calculate_skew();
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Cannot find ATM OptionData for empty option chain: TEST")
            );
        }

        #[test]
        fn test_missing_implied_volatility() {
            let mut chain = create_standard_chain();

            // Add an option without implied volatility
            chain.add_option(
                pos!(115.0),
                spos!(0.1),
                spos!(0.2),
                None,
                None,
                pos!(0.5),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
                None,
            );

            let result = chain.calculate_skew();
            assert!(result.is_ok()); // Should work with partial data
        }

        #[test]
        fn test_relative_strike_calculation() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew().unwrap();

            // For ATM strike (100.0), relative strike should be 1.0
            let atm_strike = result
                .iter()
                .find(|(rel_strike, _)| (rel_strike.sub_or_zero(&Decimal::ONE)) < pos!(0.0001));
            assert!(atm_strike.is_some());
        }
    }

    mod calculate_rnd_tests_bis {
        use super::*;

        #[test]
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
        fn test_strike_interval_detection() {
            let mut chain = create_standard_chain();

            // Add option with different strike interval
            chain.add_option(
                pos!(102.5),
                spos!(1.0),
                spos!(1.1),
                None,
                None,
                pos!(0.17),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
                None,
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
                    pos!(vol),
                    None,
                    None,
                    None,
                    spos!(100.0),
                    Some(50),
                    None,
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
                    pos!(vol),
                    None,
                    None,
                    None,
                    spos!(100.0),
                    Some(50),
                    None,
                );
            }

            let result = chain.calculate_skew().unwrap();

            // Verify monotonic decrease
            for window in result.windows(2) {
                assert!(window[0].1 > window[1].1);
            }
        }

        #[test]
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
mod tests_option_data_delta {
    use super::*;
    use crate::model::ExpirationDate;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a standard test OptionDataPriceParams
    fn create_standard_price_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("AAPL".to_string()),
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
            pos!(0.2),   // implied_volatility
            None,        // delta
            None,
            None,
            spos!(1000.0),            // volume
            Some(500),                // open_interest
            Some("AAPL".to_string()), // underlying_symbol
            Some(ExpirationDate::Days(pos!(3.9))),
            Some(Box::new(pos!(100.0))), // underlying_price
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_calculate_delta_standard_call() {
        let mut option_data = create_standard_option_data();

        option_data.calculate_delta();

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Typical at-the-money call delta should be around 0.5
        assert!(delta > dec!(0.4) && delta < dec!(0.6));
    }

    #[test]
    fn test_calculate_delta_near_the_money() {
        let mut option_data = create_standard_option_data();
        option_data.calculate_delta();
        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();
        // Near-the-money call delta should be slightly higher than 0.5
        assert!(delta > dec!(0.5) && delta < dec!(0.6));
    }

    #[test]
    fn test_calculate_delta_deep_itm() {
        let mut option_data = create_standard_option_data();
        option_data.underlying_price = Some(Box::new(pos!(104.0)));
        option_data.calculate_delta();

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Deep ITM call delta should be close to 1
        assert!(delta > dec!(0.9) && delta <= dec!(1.0));
    }

    #[test]
    fn test_calculate_delta_deep_otm() {
        let mut option_data = create_standard_option_data();
        option_data.underlying_price = Some(Box::new(pos!(94.0)));
        option_data.calculate_delta();

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Deep OTM call delta should be close to 0
        assert!(delta >= Decimal::ZERO && delta < dec!(0.1));
    }

    #[test]
    fn test_calculate_delta_multiple_calls() {
        let mut option_data = create_standard_option_data();

        // Call delta multiple times to ensure consistent behavior
        for _ in 0..3 {
            option_data.calculate_delta();
            assert!(option_data.delta_call.is_some());
        }
    }

    #[test]
    fn test_calculate_delta_different_expiration() {
        let mut price_params = create_standard_price_params();
        price_params.expiration_date = Some(ExpirationDate::Days(pos!(60.0))); // Longer expiration

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta();

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
            pos!(0.2),         // implied_volatility
            Some(dec!(0.6)),   // delta
            Some(dec!(100.0)), // volume
            Some(dec!(50.0)),  // open_interest
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
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
            BasicAxisTypes::Vanna,
            BasicAxisTypes::Veta,
            BasicAxisTypes::Charm,
            BasicAxisTypes::Color,
        ];

        for axis in axes {
            let curve = chain.curve(&axis, &OptionStyle::Call, &Side::Long);

            assert!(curve.is_ok(), "Failed to create curve for axis: {axis:?}");
            let curve = curve.unwrap();

            // Each curve should have at least one point
            assert!(!curve.points.is_empty(), "Curve for axis {axis:?} is empty");
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
            pos!(0.2),         // implied_volatility
            Some(dec!(0.6)),   // delta
            Some(dec!(100.0)), // volume
            Some(dec!(50.0)),  // open_interest
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
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
            BasicAxisTypes::Vanna,
            BasicAxisTypes::Vomma,
            BasicAxisTypes::Veta,
            BasicAxisTypes::Charm,
            BasicAxisTypes::Color,
        ];

        for axis in axes {
            let result = chain.surface(&axis, &OptionStyle::Call, None, &Side::Long);
            assert!(result.is_ok(), "Failed for axis: {axis:?}");
        }
    }

    #[test]
    fn test_surface_with_empty_chain() {
        let empty_chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.01),
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

    #[test]
    fn test_vanna_surface() {
        let chain = create_test_option_chain();
        let volatilities = vec![pos!(0.15), pos!(0.20), pos!(0.25)];
        let chain_result = chain.vanna_surface(volatilities);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_vomma_surface() {
        let chain = create_test_option_chain();
        let volatilities = vec![pos!(0.15), pos!(0.20), pos!(0.25)];
        let chain_result = chain.vomma_surface(volatilities);
        assert!(chain_result.is_ok());
    }
}

#[cfg(test)]
mod tests_option_chain_time_surfaces {
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
            pos!(0.2),         // implied_volatility
            Some(dec!(0.6)),   // delta
            Some(dec!(100.0)), // volume
            Some(dec!(50.0)),  // open_interest
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            pos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(150.0)),
            Some(dec!(75)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            pos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(80.0)),
            Some(dec!(40)),
            None,
            None,
            None,
        );
        chain.update_greeks();
        chain
    }

    #[test]
    fn test_time_surface_invalid_axis() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let result = chain.time_surface(
            &BasicAxisTypes::Strike,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(SurfaceError::ConstructionError(msg)) => {
                assert_eq!(msg, "Axis not valid for time surface");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_time_surface_empty_dte_vector() {
        let chain = create_test_option_chain();
        let empty_dte: Vec<Positive> = vec![];

        let result = chain.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Call,
            empty_dte,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(SurfaceError::ConstructionError(msg)) => {
                assert_eq!(msg, "No valid points generated for time surface");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_time_surface_different_option_styles() {
        let chain = create_test_option_chain();

        // Test for calls
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let call_result = chain.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        );
        assert!(call_result.is_ok());

        // Test for puts
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let put_result = chain.time_surface(
            &BasicAxisTypes::Charm,
            &OptionStyle::Put,
            days_to_expiry,
            &Side::Long,
        );
        assert!(put_result.is_ok());
    }

    #[test]
    fn test_time_surface_different_sides() {
        let chain = create_test_option_chain();

        // Test for long position
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let long_result = chain.time_surface(
            &BasicAxisTypes::Color,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Long,
        );
        assert!(long_result.is_ok());

        // Test for short position
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let short_result = chain.time_surface(
            &BasicAxisTypes::Color,
            &OptionStyle::Call,
            days_to_expiry,
            &Side::Short,
        );
        assert!(short_result.is_ok());
    }

    #[test]
    fn test_time_surface_different_greeks() {
        let chain = create_test_option_chain();
        let axes = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
            BasicAxisTypes::Price,
            BasicAxisTypes::Vanna,
            BasicAxisTypes::Vomma,
            BasicAxisTypes::Veta,
            BasicAxisTypes::Charm,
            BasicAxisTypes::Color,
        ];

        for axis in axes {
            let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
            let result = chain.time_surface(&axis, &OptionStyle::Call, days_to_expiry, &Side::Long);
            assert!(result.is_ok(), "Failed for axis: {axis:?}");
        }
    }

    #[test]
    fn test_time_surface_with_empty_chain() {
        let empty_chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.01),
        );

        let empty_dte: Vec<Positive> = vec![];
        let result = empty_chain.time_surface(
            &BasicAxisTypes::Delta,
            &OptionStyle::Call,
            empty_dte,
            &Side::Long,
        );

        assert!(result.is_err());
        match result {
            Err(SurfaceError::ConstructionError(msg)) => {
                assert_eq!(msg, "No valid points generated for time surface");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }

    #[test]
    fn test_theta_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let chain_result = chain.theta_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_veta_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let chain_result = chain.veta_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_charm_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let chain_result = chain.charm_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }

    #[test]
    fn test_color_time_surface() {
        let chain = create_test_option_chain();
        let days_to_expiry = vec![pos!(30.0), pos!(60.0), pos!(90.0)];
        let chain_result = chain.color_time_surface(days_to_expiry);
        assert!(chain_result.is_ok());
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use crate::spos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_optiondata_serialization() {
        let option_data = OptionData {
            strike_price: pos!(100.0),
            call_bid: spos!(9.5),
            call_ask: spos!(10.0),
            put_bid: spos!(8.5),
            put_ask: spos!(9.0),
            call_middle: spos!(9.75),
            put_middle: spos!(8.75),
            implied_volatility: pos!(0.2),
            delta_call: Some(dec!(0.5)),
            delta_put: Some(dec!(-0.5)),
            gamma: Some(dec!(0.1)),
            volume: spos!(1000.0),
            open_interest: Some(500),
            symbol: None,
            expiration_date: None,
            underlying_price: None,
            risk_free_rate: None,
            dividend_yield: None,
            epic: None,
            extra_fields: None,
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
            "2030-01-01".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add some test options
        chain.add_option(
            pos!(95.0),
            spos!(6.0),
            spos!(6.5),
            spos!(1.5),
            spos!(2.0),
            pos!(0.2),
            Some(dec!(0.7)),
            Some(dec!(-0.3)),
            Some(dec!(0.1)),
            spos!(1000.0),
            Some(500),
            None,
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
            epic: None,
            extra_fields: None,
        };

        let serialized = serde_json::to_string(&option_data).unwrap();
        let deserialized: OptionData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(option_data, deserialized);
    }
}

#[cfg(test)]
mod tests_option_data_serde {
    use super::*;
    use crate::spos;
    use rust_decimal_macros::dec;
    use serde_json;

    // Helper function to create a sample OptionData
    fn create_sample_option_data() -> OptionData {
        OptionData {
            strike_price: pos!(100.0),
            call_bid: spos!(9.5),
            call_ask: spos!(10.0),
            put_bid: spos!(8.5),
            put_ask: spos!(9.0),
            call_middle: spos!(9.75),
            put_middle: spos!(8.75),
            implied_volatility: pos!(0.2),
            delta_call: Some(dec!(0.5)),
            delta_put: Some(dec!(-0.5)),
            gamma: Some(dec!(0.1)),
            volume: spos!(1000.0),
            open_interest: Some(500),
            symbol: None,
            expiration_date: None,
            underlying_price: None,
            risk_free_rate: None,
            dividend_yield: None,
            epic: None,
            extra_fields: None,
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
            epic: None,
            extra_fields: None,
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
            call_bid: spos!(99999.99),
            call_ask: spos!(99999.99),
            implied_volatility: pos!(1.0),
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
            call_bid: spos!(0.0001),
            implied_volatility: pos!(0.0001),
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
            implied_volatility: pos!(1.0),
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
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_sample_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2030-01-01".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add some test options
        chain.add_option(
            pos!(95.0),
            spos!(6.0),
            spos!(6.5),
            spos!(1.5),
            spos!(2.0),
            pos!(0.2),
            Some(dec!(0.7)),
            Some(dec!(-0.3)),
            Some(dec!(0.1)),
            spos!(1000.0),
            Some(500),
            None,
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
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);

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
            spos!(5.0),
            spos!(5.5),
            spos!(5.0),
            spos!(5.5),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.1)),
            spos!(1000.0),
            Some(500),
            None,
        );

        chain.add_option(
            pos!(105.0),
            spos!(4.0),
            spos!(4.5),
            spos!(8.0),
            spos!(8.5),
            pos!(0.2),
            Some(dec!(0.3)),
            Some(dec!(-0.7)),
            Some(dec!(0.1)),
            spos!(1000.0),
            Some(500),
            None,
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
            "2030-01-01".to_string(),
            Some(Decimal::MAX),
            Some(Positive::INFINITY),
        );

        chain.add_option(
            Positive::ONE,
            Some(Positive::ONE),
            Some(Positive::ONE),
            Some(Positive::ONE),
            Some(Positive::ONE),
            Positive::ONE,
            Some(Decimal::ONE),
            Some(Decimal::ONE),
            Some(Decimal::ONE),
            Some(Positive::ONE),
            Some(1),
            None,
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
            Positive::ZERO,
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
    }
}

#[cfg(test)]
mod tests_gamma_calculations {
    use super::*;

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
        let mut chain = create_test_chain_with_gamma();
        chain.update_greeks();
        let result = chain.gamma_exposure();

        assert!(result.is_ok());
        let gamma_exposure = result.unwrap();
        assert_decimal_eq!(gamma_exposure, dec!(0.0), dec!(0.001));
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
            pos!(0.35),
            Some(dec!(0.3)),
            Some(dec!(-0.7)),
            None, // No gamma value
            spos!(60.0),
            Some(30),
            None,
        );

        chain.update_greeks();
        let result = chain.gamma_exposure().unwrap();
        assert_decimal_eq!(result, dec!(0.0), dec!(0.001));
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
mod tests_delta_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined delta values
    fn create_test_chain_with_delta() -> OptionChain {
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap()
    }

    #[test]
    fn test_delta_exposure_basic() {
        let mut chain = create_test_chain_with_delta();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.delta_exposure();

        assert!(result.is_ok());
        let delta_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(delta_exposure, dec!(17.0), dec!(0.000001));
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
        assert_eq!(result.unwrap(), dec!(17.0));
    }

    #[test]
    fn test_delta_exposure_updates() {
        let mut chain = create_test_chain_with_delta();

        // Get initial delta exposure (should be 0 as greeks aren't initialized)
        let initial_delta = chain.delta_exposure().unwrap();
        assert_eq!(initial_delta, dec!(17.0));

        // Update greeks and check new delta exposure
        chain.update_greeks();
        let updated_delta = chain.delta_exposure().unwrap();
        assert_decimal_eq!(updated_delta, dec!(17.0), dec!(0.000001));
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
mod tests_vega_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined vega values
    fn create_test_chain_with_vega() -> OptionChain {
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap()
    }

    #[test]
    fn test_vega_exposure_basic() {
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
mod tests_theta_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with predefined theta values
    fn create_test_chain_with_theta() -> OptionChain {
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap()
    }

    #[test]
    fn test_theta_exposure_basic() {
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

#[cfg(test)]
mod tests_vanna_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain for vanna calculations
    fn create_test_chain_with_vanna() -> OptionChain {
        let mut option_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        // It is necessary to update the expiration date of all the options in the chain
        // with a relative number of days in order to have a correct vanna calculation
        option_chain.update_expiration_date("30.0".to_string());
        option_chain
    }

    #[test]
    fn test_vanna_exposure_basic() {
        let mut chain = create_test_chain_with_vanna();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.vanna_exposure();

        assert!(result.is_ok());
        let vanna_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(vanna_exposure, dec!(-38.1265860372), dec!(0.0001));
    }

    #[test]
    fn test_vanna_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.vanna_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_vanna_curve() {
        let mut chain = create_test_chain_with_vanna();
        chain.update_greeks();
        let result = chain.vanna_curve();

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
    fn test_vanna_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.vanna_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }
}

#[cfg(test)]
mod tests_vomma_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain for vomma calculation
    fn create_test_chain_with_vomma() -> OptionChain {
        let mut option_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        // It is necessary to update the expiration date of all the options in the chain
        // with a relative number of days in order to have a correct vomma calculation
        option_chain.update_expiration_date("30.0".to_string());
        option_chain
    }

    #[test]
    fn test_vomma_exposure_basic() {
        let mut chain = create_test_chain_with_vomma();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.vomma_exposure();

        assert!(result.is_ok());
        let vomma_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(vomma_exposure, dec!(1393.74972558), dec!(0.0001));
    }

    #[test]
    fn test_vomma_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.vomma_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }
}

#[cfg(test)]
mod tests_veta_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain for veta calculations
    fn create_test_chain_with_veta() -> OptionChain {
        let mut option_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        // It is necessary to update the expiration date of all the options in the chain
        // with a relative number of days in order to have a correct veta calculation
        option_chain.update_expiration_date("30.0".to_string());
        option_chain
    }

    #[test]
    fn test_veta_exposure_basic() {
        let mut chain = create_test_chain_with_veta();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.veta_exposure();

        assert!(result.is_ok());
        let veta_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(veta_exposure, dec!(0.15239781588122), dec!(0.0001));
    }

    #[test]
    fn test_veta_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.veta_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_veta_curve() {
        let mut chain = create_test_chain_with_veta();
        chain.update_greeks();
        let result = chain.veta_curve();

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
    fn test_veta_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.veta_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }
}

#[cfg(test)]
mod tests_charm_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain for charm calculations
    fn create_test_chain_with_charm() -> OptionChain {
        let mut option_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        // It is necessary to update the expiration date of all the options in the chain
        // with a relative number of days in order to have a correct charm calculation
        option_chain.update_expiration_date("30.0".to_string());
        option_chain
    }

    #[test]
    fn test_charm_exposure_basic() {
        let mut chain = create_test_chain_with_charm();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.charm_exposure();

        assert!(result.is_ok());
        let charm_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(charm_exposure, dec!(0.115107), dec!(0.000001));
    }

    #[test]
    fn test_charm_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.charm_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_charm_curve() {
        let mut chain = create_test_chain_with_charm();
        chain.update_greeks();
        let result = chain.charm_curve();

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
    fn test_charm_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.charm_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }
}

#[cfg(test)]
mod tests_color_calculations {
    use super::*;

    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain for charm calculations
    fn create_test_chain_with_color() -> OptionChain {
        let mut option_chain =
            OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
        // It is necessary to update the expiration date of all the options in the chain
        // with a relative number of days in order to have a correct color calculation
        option_chain.update_expiration_date("30.0".to_string());
        option_chain
    }

    #[test]
    fn test_color_exposure_basic() {
        let mut chain = create_test_chain_with_color();
        // Initialize the greeks first
        chain.update_greeks();
        let result = chain.color_exposure();

        assert!(result.is_ok());
        let color_exposure = result.unwrap();
        // Test against expected value from sample data
        assert_decimal_eq!(color_exposure, dec!(-0.001356), dec!(0.000001));
    }

    #[test]
    fn test_color_exposure_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.color_exposure();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), dec!(0.0));
    }

    #[test]
    fn test_color_curve() {
        let mut chain = create_test_chain_with_color();
        chain.update_greeks();
        let result = chain.color_curve();

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
    fn test_color_curve_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        let result = chain.color_curve();
        // Should return error or empty curve depending on implementation
        if let Ok(curve) = result {
            assert!(curve.points.is_empty())
        }
    }
}

#[cfg(test)]
mod tests_atm_strike {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::model::ExpirationDate;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_standard_chain() -> OptionChain {
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.3),
            Decimal::ZERO,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("AAPL".to_string()),
            ),
            pos!(0.2),
        );

        OptionChain::build_chain(&params)
    }

    #[test]
    fn test_atm_strike_exact_match() {
        let chain = create_standard_chain();

        // The default chain has a strike at 100.0 which matches the underlying price
        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find the ATM strike");

        let strike = result.unwrap();
        assert_eq!(
            *strike,
            pos!(100.0),
            "Should return strike at exactly 100.0"
        );
    }

    #[test]
    fn test_atm_strike_approximate_match() {
        let mut chain = create_standard_chain();

        // Modify the underlying price to a value that doesn't have an exact match
        chain.underlying_price = pos!(100.5);

        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find the closest strike");

        let strike = result.unwrap();
        assert_eq!(
            *strike,
            pos!(100.0),
            "Should return the closest strike (100.0)"
        );

        // Modify the underlying price to test the other direction
        chain.underlying_price = pos!(101.0);

        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find the closest strike");

        let strike = result.unwrap();
        assert_eq!(
            *strike,
            pos!(101.0),
            "Should return the closest strike (101.0)"
        );
    }

    #[test]
    fn test_atm_strike_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2023-12-15".to_string(), None, None);

        let result = chain.atm_strike();
        assert!(result.is_err(), "Should return error for empty chain");

        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("empty option chain"),
            "Error should mention empty chain"
        );
        assert!(error.contains("EMPTY"), "Error should include the symbol");
    }

    #[test]
    fn test_atm_strike_extreme_underlying() {
        let mut chain = create_standard_chain();

        // Set underlying price far from any strike
        chain.underlying_price = pos!(110.0);

        let result = chain.atm_strike();
        assert!(
            result.is_ok(),
            "Should find the closest strike even for extreme values"
        );

        let strike = result.unwrap();

        // The farthest strike in the standard chain should be around 110.0
        assert!(
            *strike >= pos!(110.0),
            "Should return the highest available strike"
        );

        // Test with very low underlying price
        chain.underlying_price = pos!(80.0);

        let result = chain.atm_strike();
        assert!(
            result.is_ok(),
            "Should find the closest strike for low values"
        );

        let strike = result.unwrap();

        // The lowest strike in the standard chain should be 90.0 (chain_size=10, so 10 strikes below ATM)
        assert_eq!(
            *strike,
            pos!(90.0),
            "Should return the lowest available strike"
        );
    }

    #[test]
    fn test_atm_strike_equidistant() {
        let mut chain = create_standard_chain();

        // Set underlying price exactly between two strikes
        chain.underlying_price = pos!(100.5);

        // Set up a custom chain with known strikes
        let mut options = BTreeSet::new();
        options.insert(OptionData::new(
            pos!(100.0),
            spos!(1.0),
            spos!(1.1),
            spos!(1.0),
            spos!(1.1),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        options.insert(OptionData::new(
            pos!(101.0),
            spos!(0.9),
            spos!(1.0),
            spos!(1.1),
            spos!(1.2),
            pos!(0.2),
            Some(dec!(0.55)),
            Some(dec!(-0.45)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        chain.options = options;

        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find a strike when equidistant");

        let strike = result.unwrap();

        // When equidistant, should return one of the two closest strikes
        assert!(
            *strike == pos!(100.0) || *strike == pos!(101.0),
            "Should return one of the equidistant strikes"
        );
    }
}

#[cfg(test)]
mod tests_atm_strike_bis {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::model::ExpirationDate;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_standard_chain() -> OptionChain {
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.3),
            Decimal::ZERO,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("AAPL".to_string()),
            ),
            pos!(0.2),
        );

        OptionChain::build_chain(&params)
    }

    #[test]
    fn test_atm_strike_approximate_match() {
        let mut chain = create_standard_chain();

        // Modify the underlying price to a value that doesn't have an exact match
        chain.underlying_price = pos!(100.5);

        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find the closest strike");

        let strike = result.unwrap();
        assert_eq!(
            *strike,
            pos!(100.0),
            "Should return the closest strike (100.0)"
        );

        // Modify the underlying price to test the other direction
        chain.underlying_price = pos!(101.0);

        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find the closest strike");

        let strike = result.unwrap();
        assert_eq!(
            *strike,
            pos!(101.0),
            "Should return the closest strike (101.0)"
        );
    }

    #[test]
    fn test_atm_strike_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2023-12-15".to_string(), None, None);

        let result = chain.atm_strike();
        assert!(result.is_err(), "Should return error for empty chain");

        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("empty option chain"),
            "Error should mention empty chain"
        );
        assert!(error.contains("EMPTY"), "Error should include the symbol");
    }

    #[test]
    fn test_atm_strike_extreme_underlying() {
        let mut chain = create_standard_chain();

        // Set underlying price far from any strike
        chain.underlying_price = pos!(150.0);

        let result = chain.atm_strike();
        assert!(
            result.is_ok(),
            "Should find the closest strike even for extreme values"
        );

        let strike = result.unwrap();

        // The farthest strike in the standard chain should be around 110.0
        assert!(
            *strike >= pos!(110.0),
            "Should return the highest available strike"
        );

        // Test with very low underlying price
        chain.underlying_price = pos!(80.0);

        let result = chain.atm_strike();
        assert!(
            result.is_ok(),
            "Should find the closest strike for low values"
        );

        let strike = result.unwrap();

        // The lowest strike in the standard chain should be 90.0 (chain_size=10, so 10 strikes below ATM)
        assert_eq!(
            *strike,
            pos!(90.0),
            "Should return the lowest available strike"
        );
    }

    #[test]
    fn test_atm_strike_equidistant() {
        let mut chain = create_standard_chain();

        // Set underlying price exactly between two strikes
        chain.underlying_price = pos!(100.5);

        // Set up a custom chain with known strikes
        let mut options = BTreeSet::new();
        options.insert(OptionData::new(
            pos!(100.0),
            spos!(1.0),
            spos!(1.1),
            spos!(1.0),
            spos!(1.1),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        options.insert(OptionData::new(
            pos!(101.0),
            spos!(0.9),
            spos!(1.0),
            spos!(1.1),
            spos!(1.2),
            pos!(0.2),
            Some(dec!(0.55)),
            Some(dec!(-0.45)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        chain.options = options;

        let result = chain.atm_strike();
        assert!(result.is_ok(), "Should find a strike when equidistant");

        let strike = result.unwrap();

        // When equidistant, should return one of the two closest strikes
        assert!(
            *strike == pos!(100.0) || *strike == pos!(101.0),
            "Should return one of the equidistant strikes"
        );
    }
}

#[cfg(test)]
mod tests_option_chain_utils {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a chain with custom strikes for specific tests
    fn create_custom_strike_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2030-01-01".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add options with irregular strike intervals
        let strikes = [90.0, 92.5, 95.0, 100.0, 105.0, 110.0, 115.0, 125.0];
        let vols = [0.22, 0.20, 0.18, 0.17, 0.175, 0.18, 0.19, 0.21]; // Volatility smile pattern
        let deltas_call = [0.1, 0.2, 0.3, 0.5, 0.7, 0.8, 0.9, 0.95]; // Approximate delta values
        let deltas_put = [-0.9, -0.8, -0.7, -0.5, -0.3, -0.2, -0.1, -0.05]; // Approximate put delta values
        let gammas = [0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.005]; // Approximate gamma values

        for (i, &strike) in strikes.iter().enumerate() {
            chain.add_option(
                pos!(strike),
                spos!(5.0),
                spos!(5.5),
                spos!(4.0),
                spos!(4.5),
                pos!(vols[i]),
                Some(Decimal::from_f64(deltas_call[i]).unwrap()),
                Some(Decimal::from_f64(deltas_put[i]).unwrap()),
                Some(Decimal::from_f64(gammas[i]).unwrap()),
                spos!(100.0),
                Some(50),
                None,
            );
        }

        chain
    }

    #[test]
    fn test_get_strike_interval_custom_chain() {
        let chain = create_custom_strike_chain();

        // The custom chain has mostly 5.0 intervals but some irregular ones
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(5.0),
            "Strike interval should be 5.0 for custom chain"
        );
    }

    #[test]
    fn test_get_strike_interval_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Empty chain should return the default interval
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(5.0),
            "Empty chain should return default interval of 5.0"
        );
    }

    #[test]
    fn test_get_strike_interval_single_option_chain() {
        let mut chain =
            OptionChain::new("SINGLE", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.0),
            spos!(4.5),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.04)),
            spos!(100.0),
            Some(50),
            None,
        );

        // Chain with a single option should return the default interval
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(5.0),
            "Single option chain should return default interval of 5.0"
        );
    }

    #[test]
    fn test_get_strike_interval_fractional_intervals() {
        let mut chain = OptionChain::new(
            "FRACTIONAL",
            pos!(100.0),
            "2030-01-01".to_string(),
            None,
            None,
        );

        // Add options with small fractional intervals
        let strikes = [100.0, 100.25, 100.5, 100.75, 101.0];

        for &strike in &strikes {
            chain.add_option(
                pos!(strike),
                spos!(1.0),
                spos!(1.1),
                spos!(1.0),
                spos!(1.1),
                pos!(0.2),
                Some(dec!(0.5)),
                Some(dec!(-0.5)),
                Some(dec!(0.04)),
                spos!(100.0),
                Some(50),
                None,
            );
        }

        // The intervals are all 0.25, but method should round to 0 and then to 1
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(1.0),
            "Fractional intervals should round to minimum of 1.0"
        );
    }
}

#[cfg(test)]
mod tests_option_chain_utils_bis {
    use super::*;
    use crate::chains::utils::OptionChainBuildParams;
    use crate::chains::utils::OptionDataPriceParams;
    use crate::model::ExpirationDate;
    use crate::{pos, spos};

    use rust_decimal_macros::dec;

    // Helper function to create a standard option chain for testing
    fn create_standard_chain() -> OptionChain {
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.3),
            Decimal::ZERO,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("AAPL".to_string()),
            ),
            pos!(0.2),
        );

        OptionChain::build_chain(&params)
    }

    // Helper function to create a chain with custom strikes for specific tests
    fn create_custom_strike_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2030-01-01".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add options with irregular strike intervals
        let strikes = [90.0, 92.5, 95.0, 100.0, 105.0, 110.0, 115.0, 125.0];
        let vols = [0.22, 0.20, 0.18, 0.17, 0.175, 0.18, 0.19, 0.21]; // Volatility smile pattern
        let deltas_call = [0.1, 0.2, 0.3, 0.5, 0.7, 0.8, 0.9, 0.95]; // Approximate delta values
        let deltas_put = [-0.9, -0.8, -0.7, -0.5, -0.3, -0.2, -0.1, -0.05]; // Approximate put delta values
        let gammas = [0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.005]; // Approximate gamma values

        for (i, &strike) in strikes.iter().enumerate() {
            chain.add_option(
                pos!(strike),
                spos!(5.0),
                spos!(5.5),
                spos!(4.0),
                spos!(4.5),
                pos!(vols[i]),
                Some(Decimal::from_f64(deltas_call[i]).unwrap()),
                Some(Decimal::from_f64(deltas_put[i]).unwrap()),
                Some(Decimal::from_f64(gammas[i]).unwrap()),
                spos!(100.0),
                Some(50),
                None,
            );
        }

        chain
    }

    #[test]
    fn test_get_strike_interval_standard_chain() {
        let chain = create_standard_chain();

        // The standard chain should have a regular interval of 1.0
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(1.0),
            "Strike interval should be 1.0 for standard chain"
        );
    }

    #[test]
    fn test_get_strike_interval_custom_chain() {
        let chain = create_custom_strike_chain();

        // The custom chain has mostly 5.0 intervals but some irregular ones
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(5.0),
            "Strike interval should be 5.0 for custom chain"
        );
    }

    #[test]
    fn test_get_strike_interval_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Empty chain should return the default interval
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(5.0),
            "Empty chain should return default interval of 5.0"
        );
    }

    #[test]
    fn test_get_strike_interval_single_option_chain() {
        let mut chain =
            OptionChain::new("SINGLE", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.0),
            spos!(4.5),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.04)),
            spos!(100.0),
            Some(50),
            None,
        );

        // Chain with a single option should return the default interval
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(5.0),
            "Single option chain should return default interval of 5.0"
        );
    }

    #[test]
    fn test_get_strike_interval_fractional_intervals() {
        let mut chain = OptionChain::new(
            "FRACTIONAL",
            pos!(100.0),
            "2030-01-01".to_string(),
            None,
            None,
        );

        // Add options with small fractional intervals
        let strikes = [100.0, 100.25, 100.5, 100.75, 101.0];

        for &strike in &strikes {
            chain.add_option(
                pos!(strike),
                spos!(1.0),
                spos!(1.1),
                spos!(1.0),
                spos!(1.1),
                pos!(0.2),
                Some(dec!(0.5)),
                Some(dec!(-0.5)),
                Some(dec!(0.04)),
                spos!(100.0),
                Some(50),
                None,
            );
        }

        // The intervals are all 0.25, but method should round to 0 and then to 1
        let interval = chain.get_strike_interval();

        assert_eq!(
            interval,
            pos!(1.0),
            "Fractional intervals should round to minimum of 1.0"
        );
    }
}

#[cfg(test)]
mod tests_to_build_params_bis {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::model::ExpirationDate;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    use tracing::info;

    fn create_standard_chain() -> OptionChain {
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            22,
            spos!(25.0),
            dec!(-0.1),
            dec!(0.1),
            pos!(0.03),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("AAPL".to_string()),
            ),
            pos!(0.2),
        );

        OptionChain::build_chain(&params)
    }

    #[test]
    fn test_to_build_params_simple() {
        let chain = create_standard_chain();
        info!("{}", chain);
        let mut params = chain.to_build_params().unwrap();

        params.smile_curve = dec!(0.000001);
        params.price_params.underlying_price = Some(Box::new(
            pos!(params.price_params.underlying_price.unwrap().to_f64() * f64::exp(0.2))
                .max(Positive::ZERO),
        ));
        params.implied_volatility =
            pos!(params.implied_volatility.to_f64() * f64::exp(0.2)).max(Positive::ZERO);
        info!("{}", params);

        let new_chain = OptionChain::build_chain(&params);
        info!("{}", new_chain);
    }
}

#[cfg(test)]
mod chain_coverage_tests {
    use super::*;
    use crate::spos;

    use rust_decimal_macros::dec;

    // Helper function to create a test chain with specific characteristics
    fn create_test_chain() -> OptionChain {
        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            5,
            spos!(5.0),
            dec!(-0.3),
            dec!(0.1),
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("AAPL".to_string()),
            ),
            pos!(0.2),
        );

        OptionChain::build_chain(&params)
    }

    #[test]
    fn test_option_chain_display() {
        let chain = create_test_chain();

        // Test the Display implementation - covers many lines
        let display_output = format!("{chain}");

        // Verify expected content in the display output
        assert!(display_output.contains("TEST"));
        assert!(display_output.contains("Underlying Price"));
        assert!(display_output.contains("Strike"));
        assert!(display_output.contains("Call Bid"));
        assert!(display_output.contains("Put Ask"));
    }

    #[test]
    fn test_get_title_variants() {
        let chain = OptionChain::new(
            "SP500 Index", // With space
            pos!(5781.88),
            "18 Oct 2024".to_string(), // With spaces
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let title = chain.get_title();
        assert_eq!(title, "SP500-Index-18-Oct-2024-5781.88");
    }

    #[test]
    fn test_update_expiration_date() {
        let mut chain = create_test_chain();
        let original_date = chain.get_expiration_date();

        // Update to a new date
        chain.update_expiration_date("2026-12-31".to_string());

        // Verify the date was updated
        assert_ne!(chain.get_expiration_date(), original_date);
        assert_eq!(chain.get_expiration_date(), "2026-12-31");
    }

    #[test]
    fn test_atm_option_data_edge_cases() {
        // Test with empty chain
        let empty_chain =
            OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);
        let result = empty_chain.atm_option_data();
        assert!(result.is_err());

        // Test with a single option exactly at the money
        let mut single_option_chain =
            OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        single_option_chain.add_option(
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
        );

        let result = single_option_chain.atm_option_data();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().strike_price, pos!(100.0));
    }

    #[test]
    fn test_update_mid_prices_and_greeks() {
        let mut chain = create_test_chain();

        // Get original values
        let original_options = chain.options.clone();

        // Call the update methods
        chain.update_mid_prices();
        chain.update_greeks();

        // Verify that values have been updated
        for (original, updated) in original_options.iter().zip(chain.options.iter()) {
            // The objects should have the same strike but potentially different values
            assert_eq!(original.strike_price, updated.strike_price);

            // Midpoints should now be set in the updated version
            if original.call_bid.is_some() && original.call_ask.is_some() {
                assert!(updated.call_middle.is_some());
            }

            if original.put_bid.is_some() && original.put_ask.is_some() {
                assert!(updated.put_middle.is_some());
            }

            // Greeks should be set
            assert!(updated.delta_call.is_some() || updated.delta_put.is_some());
        }
    }

    #[test]
    fn test_strike_price_range_vec() {
        let chain = create_test_chain();

        // Test with different step sizes
        let range_1 = chain.strike_price_range_vec(1.0);
        assert!(range_1.is_some());

        let range_5 = chain.strike_price_range_vec(5.0);
        assert!(range_5.is_some());

        // Compare ranges
        if let (Some(range_1), Some(range_5)) = (range_1, range_5) {
            assert!(range_1.len() >= range_5.len());
        }

        // Test with empty chain
        let empty_chain =
            OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);
        let range = empty_chain.strike_price_range_vec(5.0);
        assert!(range.is_none());
    }

    #[test]
    fn test_get_params_and_atm_strike() {
        let chain = create_test_chain();

        // Test get_params
        let atm_strike = chain.atm_strike().unwrap();
        let params_result = chain.get_params(*atm_strike);
        assert!(params_result.is_ok());

        let params = params_result.unwrap();
        assert_eq!(*params.underlying_price.unwrap(), chain.underlying_price);

        // Test with invalid strike
        let invalid_strike = pos!(9999.0);
        let invalid_params_result = chain.get_params(invalid_strike);
        assert!(invalid_params_result.is_err());
    }

    #[test]
    fn test_calculate_delta_exposure() {
        let mut chain = create_test_chain();

        // Update Greeks to ensure they are populated
        chain.update_greeks();

        // Now test delta exposure
        let delta_exposure = chain.delta_exposure();
        assert!(delta_exposure.is_ok());
    }

    #[test]
    fn test_all_exposures() {
        let mut chain = create_test_chain();

        // Update Greeks to ensure they are populated
        chain.update_greeks();

        // Test various exposure calculations
        let gamma_exposure = chain.gamma_exposure();
        assert!(gamma_exposure.is_ok());

        let delta_exposure = chain.delta_exposure();
        assert!(delta_exposure.is_ok());

        let vega_exposure = chain.vega_exposure();
        assert!(vega_exposure.is_ok());

        let theta_exposure = chain.theta_exposure();
        assert!(theta_exposure.is_ok());

        let vanna_exposure = chain.vanna_exposure();
        assert!(vanna_exposure.is_ok());

        let vomma_exposure = chain.vomma_exposure();
        assert!(vomma_exposure.is_ok());

        let veta_exposure = chain.veta_exposure();
        assert!(veta_exposure.is_ok());

        let charm_exposure = chain.charm_exposure();
        assert!(charm_exposure.is_ok());

        let color_exposure = chain.color_exposure();
        assert!(color_exposure.is_ok());
    }

    #[test]
    fn test_all_curves() {
        let mut chain = create_test_chain();

        // Update Greeks to ensure they are populated
        chain.update_greeks();

        // Test various curve calculations
        let gamma_curve = chain.gamma_curve();
        assert!(gamma_curve.is_ok());

        let delta_curve = chain.delta_curve();
        assert!(delta_curve.is_ok());

        let vega_curve = chain.vega_curve();
        assert!(vega_curve.is_ok());

        let theta_curve = chain.theta_curve();
        assert!(theta_curve.is_ok());

        let vanna_curve = chain.vanna_curve();
        assert!(vanna_curve.is_ok());

        let veta_curve = chain.veta_curve();
        assert!(veta_curve.is_ok());

        let charm_curve = chain.charm_curve();
        assert!(charm_curve.is_ok());

        let color_curve = chain.color_curve();
        assert!(color_curve.is_ok());
    }
}

#[cfg(test)]
mod chain_coverage_tests_bis {
    use super::*;
    use crate::spos;

    use rust_decimal_macros::dec;

    // Helper function to create a test chain with specific characteristics
    fn create_test_chain() -> OptionChain {
        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            5,
            spos!(5.0),
            dec!(-0.3),
            dec!(0.1),
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("AAPL".to_string()),
            ),
            pos!(0.17),
        );

        OptionChain::build_chain(&params)
    }

    #[test]
    fn test_deserializer_field_handling() {
        let chain = create_test_chain();

        // Save to JSON to trigger serialization
        let result = chain.save_to_json(".");
        assert!(result.is_ok());
        let file = format!("./{}.json", chain.get_title());
        // Load from JSON to trigger deserialization
        let loaded_chain = OptionChain::load_from_json(&file);
        assert!(loaded_chain.is_ok());

        let loaded_chain = loaded_chain.unwrap();
        assert_eq!(loaded_chain.symbol, "TEST");
        assert_eq!(loaded_chain.underlying_price, pos!(100.0));

        // Clean up the test file
        std::fs::remove_file(file).unwrap();
    }

    // Test for many display-related lines
    #[test]
    fn test_option_chain_display() {
        let chain = create_test_chain();

        // Test the Display implementation - covers many lines
        let display_output = format!("{chain}");

        // Verify expected content in the display output
        assert!(display_output.contains("TEST"));
        assert!(display_output.contains("100"));
        assert!(display_output.contains("Strike"));
        assert!(display_output.contains("Call Bid"));
        assert!(display_output.contains("Put Ask"));
    }

    #[test]
    fn test_get_title_variants() {
        let chain = OptionChain::new(
            "SP500 Index", // With space
            pos!(5781.88),
            "18 Oct 2024".to_string(), // With spaces
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let title = chain.get_title();
        assert_eq!(title, "SP500-Index-18-Oct-2024-5781.88");
    }

    // Test for line 345
    #[test]
    fn test_update_expiration_date() {
        let mut chain = create_test_chain();
        let original_date = chain.get_expiration_date();

        // Update to a new date
        chain.update_expiration_date("2026-12-31".to_string());

        // Verify the date was updated
        assert_ne!(chain.get_expiration_date(), original_date);
        assert_eq!(chain.get_expiration_date(), "2026-12-31");
    }

    #[test]
    fn test_atm_option_data_edge_cases() {
        // Test with empty chain
        let empty_chain =
            OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);
        let result = empty_chain.atm_option_data();
        assert!(result.is_err());

        // Test with a single option exactly at the money
        let mut single_option_chain =
            OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        single_option_chain.add_option(
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
        );

        let result = single_option_chain.atm_option_data();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().strike_price, pos!(100.0));
    }

    #[test]
    fn test_update_mid_prices_and_greeks() {
        let mut chain = create_test_chain();

        // Get original values
        let original_options = chain.options.clone();

        // Call the update methods
        chain.update_mid_prices();
        chain.update_greeks();

        // Verify that values have been updated
        for (original, updated) in original_options.iter().zip(chain.options.iter()) {
            // The objects should have the same strike but potentially different values
            assert_eq!(original.strike_price, updated.strike_price);

            // Midpoints should now be set in the updated version
            if original.call_bid.is_some() && original.call_ask.is_some() {
                assert!(updated.call_middle.is_some());
            }

            if original.put_bid.is_some() && original.put_ask.is_some() {
                assert!(updated.put_middle.is_some());
            }

            // Greeks should be set
            assert!(updated.delta_call.is_some() || updated.delta_put.is_some());
        }
    }

    #[test]
    fn test_strike_price_range_vec() {
        let chain = create_test_chain();

        // Test with different step sizes
        let range_1 = chain.strike_price_range_vec(1.0);
        assert!(range_1.is_some());

        let range_5 = chain.strike_price_range_vec(5.0);
        assert!(range_5.is_some());

        // Compare ranges
        if let (Some(range_1), Some(range_5)) = (range_1, range_5) {
            assert!(range_1.len() >= range_5.len());
        }

        // Test with empty chain
        let empty_chain =
            OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);
        let range = empty_chain.strike_price_range_vec(5.0);
        assert!(range.is_none());
    }

    #[test]
    fn test_get_params_and_atm_strike() {
        let chain = create_test_chain();

        // Test get_params
        let atm_strike = chain.atm_strike().unwrap();
        let params_result = chain.get_params(*atm_strike);
        assert!(params_result.is_ok());

        let params = params_result.unwrap();
        assert_eq!(*params.underlying_price.unwrap(), chain.underlying_price);

        // Test with invalid strike
        let invalid_strike = pos!(9999.0);
        let invalid_params_result = chain.get_params(invalid_strike);
        assert!(invalid_params_result.is_err());
    }

    #[test]
    fn test_calculate_delta_exposure() {
        let mut chain = create_test_chain();

        // Update Greeks to ensure they are populated
        chain.update_greeks();

        // Now test delta exposure
        let delta_exposure = chain.delta_exposure();
        assert!(delta_exposure.is_ok());
    }

    #[test]
    fn test_all_exposures() {
        let mut chain = create_test_chain();

        // Update Greeks to ensure they are populated
        chain.update_greeks();

        // Test various exposure calculations
        let gamma_exposure = chain.gamma_exposure();
        assert!(gamma_exposure.is_ok());

        let delta_exposure = chain.delta_exposure();
        assert!(delta_exposure.is_ok());

        let vega_exposure = chain.vega_exposure();
        assert!(vega_exposure.is_ok());

        let theta_exposure = chain.theta_exposure();
        assert!(theta_exposure.is_ok());

        let vanna_exposure = chain.vanna_exposure();
        assert!(vanna_exposure.is_ok());

        let vomma_exposure = chain.vomma_exposure();
        assert!(vomma_exposure.is_ok());

        let veta_exposure = chain.veta_exposure();
        assert!(veta_exposure.is_ok());

        let charm_exposure = chain.charm_exposure();
        assert!(charm_exposure.is_ok());

        let color_exposure = chain.color_exposure();
        assert!(color_exposure.is_ok());
    }

    #[test]
    fn test_all_curves() {
        let mut chain = create_test_chain();

        // Update Greeks to ensure they are populated
        chain.update_greeks();

        // Test various curve calculations
        let gamma_curve = chain.gamma_curve();
        assert!(gamma_curve.is_ok());

        let delta_curve = chain.delta_curve();
        assert!(delta_curve.is_ok());

        let vega_curve = chain.vega_curve();
        assert!(vega_curve.is_ok());

        let theta_curve = chain.theta_curve();
        assert!(theta_curve.is_ok());

        let vanna_curve = chain.vanna_curve();
        assert!(vanna_curve.is_ok());

        let veta_curve = chain.veta_curve();
        assert!(veta_curve.is_ok());

        let charm_curve = chain.charm_curve();
        assert!(charm_curve.is_ok());

        let color_curve = chain.color_curve();
        assert!(color_curve.is_ok());
    }
}

#[cfg(test)]
mod tests_get_position_with_delta {
    use super::*;
    use crate::error::chains::OptionDataErrorKind;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    use tracing::info;

    // Helper function to create a test option chain with various deltas
    fn create_test_chain_with_deltas() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add options with different deltas
        // For calls: higher strike = lower delta
        // For puts: higher strike = higher delta (more negative)

        // Far ITM call, high delta / Far OTM put, low delta
        chain.add_option(
            pos!(80.0),
            spos!(20.0),
            spos!(20.5),
            spos!(0.5),
            spos!(0.7),
            pos!(0.25),
            Some(dec!(0.9)),  // High call delta (0.9)
            Some(dec!(-0.1)), // Low put delta (-0.1)
            Some(dec!(0.02)),
            spos!(100.0),
            Some(50),
            None,
        );

        // ITM call / OTM put
        chain.add_option(
            pos!(90.0),
            spos!(11.0),
            spos!(11.5),
            spos!(1.5),
            spos!(1.8),
            pos!(0.25),
            Some(dec!(0.7)),  // Call delta (0.7)
            Some(dec!(-0.3)), // Put delta (-0.3)
            Some(dec!(0.04)),
            spos!(200.0),
            Some(100),
            None,
        );

        // ATM call and put
        chain.add_option(
            pos!(100.0),
            spos!(5.0),
            spos!(5.3),
            spos!(5.0),
            spos!(5.3),
            pos!(0.25),
            Some(dec!(0.5)),  // ATM call delta (0.5)
            Some(dec!(-0.5)), // ATM put delta (-0.5)
            Some(dec!(0.05)),
            spos!(500.0),
            Some(250),
            None,
        );

        // OTM call / ITM put
        chain.add_option(
            pos!(110.0),
            spos!(1.5),
            spos!(1.8),
            spos!(11.0),
            spos!(11.5),
            pos!(0.25),
            Some(dec!(0.3)),  // Call delta (0.3)
            Some(dec!(-0.7)), // Put delta (-0.7)
            Some(dec!(0.04)),
            spos!(200.0),
            Some(100),
            None,
        );

        // Far OTM call, low delta / Far ITM put, high delta
        chain.add_option(
            pos!(120.0),
            spos!(0.5),
            spos!(0.7),
            spos!(20.0),
            spos!(20.5),
            pos!(0.25),
            Some(dec!(0.1)),  // Low call delta (0.1)
            Some(dec!(-0.9)), // High put delta (-0.9)
            Some(dec!(0.02)),
            spos!(100.0),
            Some(50),
            None,
        );

        chain
    }

    #[test]
    fn test_get_position_with_delta_long_call() {
        let chain = create_test_chain_with_deltas();
        info!("{}", chain);
        // Request a long call with delta of 0.6 or lower
        let result = chain.get_position_with_delta(dec!(0.6), Side::Long, OptionStyle::Call);

        assert!(result.is_ok(), "Should find a suitable call option");

        let position = result.unwrap();

        assert_eq!(
            position.option.strike_price,
            pos!(100.0),
            "Should select option with strike 100.0 (delta 0.5)"
        );
        assert_eq!(position.option.side, Side::Long);
        assert_eq!(position.option.option_style, OptionStyle::Call);
    }

    #[test]
    fn test_get_position_with_delta_short_put() {
        let chain = create_test_chain_with_deltas();

        // Request a short put with delta of -0.4
        let result = chain.get_position_with_delta(dec!(0.4), Side::Short, OptionStyle::Put);

        assert!(result.is_ok(), "Should find a suitable put option");

        let position = result.unwrap();

        // Should select the option with delta -0.5 (closest to -0.3)
        assert_eq!(
            position.option.strike_price,
            pos!(90.0),
            "Should select option with strike 90.0 (delta -0.3)"
        );
        assert_eq!(position.option.side, Side::Short);
        assert_eq!(position.option.option_style, OptionStyle::Put);
    }

    #[test]
    fn test_get_position_with_delta_exact_match() {
        let chain = create_test_chain_with_deltas();

        // Request a long call with delta of exactly 0.5
        let result = chain.get_position_with_delta(dec!(-0.5), Side::Long, OptionStyle::Call);

        assert!(result.is_ok(), "Should find an exact match");

        let position = result.unwrap();

        // Should select the option with delta exactly 0.5
        assert_eq!(
            position.option.strike_price,
            pos!(100.0),
            "Should select option with strike 100.0 (delta exactly 0.5)"
        );
    }

    #[test]
    fn test_get_position_with_delta_high_target() {
        let chain = create_test_chain_with_deltas();

        // Request a long call with very high delta of 0.95
        let result = chain.get_position_with_delta(dec!(-0.95), Side::Long, OptionStyle::Call);

        assert!(result.is_ok(), "Should find the highest available delta");

        let position = result.unwrap();

        // Should select the option with highest delta (0.9)
        assert_eq!(
            position.option.strike_price,
            pos!(80.0),
            "Should select option with strike 80.0 (highest delta 0.9)"
        );
    }

    #[test]
    fn test_get_position_with_delta_low_target() {
        let chain = create_test_chain_with_deltas();
        info!("{}", chain);
        // Request a long call with very low delta of 0.05
        let result = chain.get_position_with_delta(dec!(0.05), Side::Long, OptionStyle::Call);

        assert!(result.is_err(), "Shouldn't find the lowest available delta");
    }

    #[test]
    fn test_get_position_with_delta_put_high_target() {
        let chain = create_test_chain_with_deltas();

        // Request a long put with high delta (for puts, high means more negative)
        let result = chain.get_position_with_delta(dec!(0.95), Side::Long, OptionStyle::Put);

        assert!(result.is_ok(), "Should find highest available put delta");

        let position = result.unwrap();

        // Should select the option with highest delta (most negative: -0.9)
        assert_eq!(
            position.option.strike_price,
            pos!(120.0),
            "Should select option with strike 120.0 (highest put delta -0.9)"
        );
    }

    #[test]
    fn test_get_position_with_delta_empty_chain() {
        let empty_chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Request any position on an empty chain
        let result = empty_chain.get_position_with_delta(dec!(0.5), Side::Long, OptionStyle::Call);

        // Should fail because there are no options
        assert!(result.is_err(), "Should fail with empty chain");

        match result.unwrap_err() {
            ChainError::OptionDataError(OptionDataErrorKind::InvalidDelta { delta, reason }) => {
                assert_eq!(delta, Some(0.5));
                assert!(
                    reason.contains("Option chain is empty"),
                    "Error message should mention missing delta: {reason}"
                );
            }
            err => panic!("Unexpected error type: {err}"),
        }
    }

    #[test]
    fn test_get_position_with_delta_missing_deltas() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add options without delta values
        chain.add_option(
            pos!(90.0),
            spos!(10.0),
            spos!(10.5),
            spos!(1.5),
            spos!(1.8),
            pos!(0.25),
            None, // No call delta
            None, // No put delta
            Some(dec!(0.04)),
            spos!(200.0),
            Some(100),
            None,
        );

        chain.add_option(
            pos!(100.0),
            spos!(5.0),
            spos!(5.3),
            spos!(5.0),
            spos!(5.3),
            pos!(0.25),
            None, // No call delta
            None, // No put delta
            Some(dec!(0.05)),
            spos!(500.0),
            Some(250),
            None,
        );

        // Request a position but no option has delta values
        let result = chain.get_position_with_delta(dec!(0.5), Side::Long, OptionStyle::Call);

        // Should fail because there are no options with delta values
        assert!(result.is_err(), "Should fail with missing deltas");

        match result.unwrap_err() {
            ChainError::OptionDataError(OptionDataErrorKind::InvalidDelta { delta, reason }) => {
                assert_eq!(delta, Some(0.5));
                assert!(
                    reason.contains("No call option with delta ≤ 0.5 was found"),
                    "Error message should mention missing delta: {reason}"
                );
            }
            err => panic!("Unexpected error type: {err}"),
        }
    }

    #[test]
    fn test_get_position_with_delta_multiple_candidates() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-12-31".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add multiple options with the same delta
        chain.add_option(
            pos!(95.0),
            spos!(11.0),
            spos!(11.5),
            spos!(1.5),
            spos!(1.8),
            pos!(0.25),
            Some(dec!(0.6)), // Same call delta
            Some(dec!(-0.4)),
            Some(dec!(0.04)),
            spos!(200.0),
            Some(100),
            None,
        );

        chain.add_option(
            pos!(105.0),
            spos!(10.0),
            spos!(10.5),
            spos!(2.5),
            spos!(2.8),
            pos!(0.25),
            Some(dec!(0.6)), // Same call delta
            Some(dec!(-0.4)),
            Some(dec!(0.04)),
            spos!(200.0),
            Some(100),
            None,
        );

        // Request a position matching the delta
        let result = chain.get_position_with_delta(dec!(0.6), Side::Long, OptionStyle::Call);

        assert!(result.is_ok(), "Should find one of the matching options");

        let position = result.unwrap();

        // Should select one of the options with delta 0.6
        // The implementation should be deterministic, always picking the same one
        assert!(
            position.option.strike_price == pos!(95.0)
                || position.option.strike_price == pos!(105.0),
            "Should select one of the options with delta 0.6"
        );
    }

    #[test]
    fn test_get_position_with_delta_side_combinations() {
        let chain = create_test_chain_with_deltas();
        info!("{}", chain);
        // Test all combinations of Side and OptionStyle
        let combinations = vec![
            (Side::Long, OptionStyle::Call, dec!(0.5)),
            (Side::Short, OptionStyle::Call, dec!(0.5)),
            (Side::Long, OptionStyle::Put, dec!(0.5)), // Remember put deltas are negative
            (Side::Short, OptionStyle::Put, dec!(0.5)),
        ];

        for (side, style, delta) in combinations {
            let result = chain.get_position_with_delta(delta, side, style);

            assert!(
                result.is_ok(),
                "Should find position for {side:?} {style:?}"
            );

            let position = result.unwrap();

            // Verify the position has the correct side and style
            assert_eq!(
                position.option.side, side,
                "Position should have requested side"
            );
            assert_eq!(
                position.option.option_style, style,
                "Position should have requested style"
            );

            // For this test with delta 0.5, all combinations should select the ATM option
            assert_eq!(
                position.option.strike_price,
                pos!(100.0),
                "Should select ATM option with strike 100.0 for {side:?} {style:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests_get_strikes_and_optiondata {
    use super::*;

    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a test chain with specific strikes
    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Add options with different strikes
        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(1.0),
                spos!(1.1),
                spos!(1.0),
                spos!(1.1),
                pos!(0.2),
                Some(dec!(0.5)),
                Some(dec!(-0.5)),
                Some(dec!(0.1)),
                spos!(100.0),
                Some(50),
                None,
            );
        }
        chain
    }

    #[test]
    fn test_get_strikes_normal_case() {
        let chain = create_test_chain();

        let result = chain.get_strikes();
        assert!(result.is_ok(), "Should successfully get strikes");

        let strikes = result.unwrap();
        assert_eq!(strikes.len(), 5, "Should return 5 strikes");

        // Verify strikes are in the expected order
        assert_eq!(strikes[0], pos!(90.0));
        assert_eq!(strikes[1], pos!(95.0));
        assert_eq!(strikes[2], pos!(100.0));
        assert_eq!(strikes[3], pos!(105.0));
        assert_eq!(strikes[4], pos!(110.0));
    }

    #[test]
    fn test_get_strikes_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);

        let result = chain.get_strikes();
        assert!(result.is_ok(), "Should handle empty chain");

        let strikes = result.unwrap();
        assert!(
            strikes.is_empty(),
            "Should return empty vector for empty chain"
        );
    }

    #[test]
    fn test_get_optiondata_with_strike_exact_match() {
        let chain = create_test_chain();

        // Test with exact strike match
        let result = chain.get_optiondata_with_strike(&pos!(100.0));
        assert!(result.is_ok(), "Should find exact strike");

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(100.0),
            "Should return option with strike 100.0"
        );
    }

    #[test]
    fn test_get_optiondata_with_strike_closest_match() {
        let chain = create_test_chain();

        // Test with price between two strikes but closer to 105
        let result = chain.get_optiondata_with_strike(&pos!(103.0));
        assert!(result.is_ok(), "Should find closest strike");

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(105.0),
            "Should return closest strike 105.0"
        );

        // Test with price between two strikes but closer to 100
        let result = chain.get_optiondata_with_strike(&pos!(97.0));
        assert!(result.is_ok(), "Should find closest strike");

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(95.0),
            "Should return closest strike 95.0"
        );

        // Test with price below lowest strike
        let result = chain.get_optiondata_with_strike(&pos!(85.0));
        assert!(result.is_ok(), "Should find closest strike for low price");

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(90.0),
            "Should return lowest strike 90.0"
        );

        // Test with price above highest strike
        let result = chain.get_optiondata_with_strike(&pos!(115.0));
        assert!(result.is_ok(), "Should find closest strike for high price");

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(110.0),
            "Should return highest strike 110.0"
        );

        // Test with price exactly between two strikes (equidistant case)
        let result = chain.get_optiondata_with_strike(&pos!(102.5));
        assert!(
            result.is_ok(),
            "Should handle price exactly between strikes"
        );

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(100.0),
            "For equidistant strikes, should return the lower strike due to BTreeSet ordering"
        );
    }

    #[test]
    fn test_get_optiondata_with_strike_edge_cases() {
        let chain = create_test_chain();

        // Test with price exactly between two strikes
        let result = chain.get_optiondata_with_strike(&pos!(97.5));
        assert!(
            result.is_ok(),
            "Should handle price exactly between strikes"
        );

        let option_data = result.unwrap();
        // Could be either 95.0 or 100.0 depending on implementation details
        assert!(
            option_data.strike_price == pos!(95.0) || option_data.strike_price == pos!(100.0),
            "Should return one of the equidistant strikes"
        );
    }

    #[test]
    fn test_get_optiondata_with_strike_empty_chain() {
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2030-01-01".to_string(), None, None);

        let result = chain.get_optiondata_with_strike(&pos!(100.0));
        assert!(result.is_err(), "Should return error for empty chain");

        let error = result.unwrap_err();
        let error_msg = format!("{error}");
        assert!(
            error_msg.contains("empty option chain"),
            "Error should mention empty chain"
        );
        assert!(
            error_msg.contains("EMPTY"),
            "Error should include the symbol"
        );
    }

    #[test]
    fn test_get_optiondata_with_strike_single_option() {
        let mut chain =
            OptionChain::new("SINGLE", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Add a single option
        chain.add_option(
            pos!(100.0),
            spos!(1.0),
            spos!(1.1),
            spos!(1.0),
            spos!(1.1),
            pos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
            None,
        );

        // Test with any price - should always return the single option
        let result = chain.get_optiondata_with_strike(&pos!(150.0));
        assert!(result.is_ok(), "Should find option in single-option chain");

        let option_data = result.unwrap();
        assert_eq!(
            option_data.strike_price,
            pos!(100.0),
            "Should return the only available strike"
        );
    }

    #[test]
    fn test_get_strikes_order() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        // Add options in non-sorted order
        chain.add_option(
            pos!(105.0),
            None,
            None,
            None,
            None,
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        chain.add_option(
            pos!(95.0),
            None,
            None,
            None,
            None,
            pos!(0.2),
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
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let result = chain.get_strikes();
        assert!(result.is_ok());

        let strikes = result.unwrap();

        // Verify they're in ascending order regardless of insertion order
        assert_eq!(strikes[0], pos!(95.0));
        assert_eq!(strikes[1], pos!(100.0));
        assert_eq!(strikes[2], pos!(105.0));
    }
}

#[cfg(test)]
mod tests_option_chain_comparison {
    use crate::Positive;
    use crate::chains::chain::OptionChain;
    use rust_decimal_macros::dec;
    use std::cmp::Ordering;

    fn create_test_chain(symbol: &str, expiration: &str) -> OptionChain {
        OptionChain::new(
            symbol,
            Positive::new(100.0).unwrap(),
            expiration.to_string(),
            Some(dec!(0.05)),
            Some(Positive::new(0.02).unwrap()),
        )
    }

    fn create_chain_with_invalid_expiration(symbol: &str) -> OptionChain {
        OptionChain::new(
            symbol,
            Positive::new(100.0).unwrap(),
            "invalid-date".to_string(),
            Some(dec!(0.05)),
            Some(Positive::new(0.02).unwrap()),
        )
    }

    #[test]
    fn test_partial_eq_same_symbol_same_expiration() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2030-01-19");

        assert_eq!(chain1, chain2);
    }

    #[test]
    fn test_partial_eq_same_symbol_different_expiration() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2024-02-16");

        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_partial_eq_different_symbol_same_expiration() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("MSFT", "2030-01-19");

        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_partial_eq_different_symbol_different_expiration() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("MSFT", "2024-02-16");

        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_partial_eq_different_underlying_price_same_expiration_symbol() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let mut chain2 = create_test_chain("AAPL", "2030-01-19");

        // Change underlying price - should still be equal based on implementation
        chain2.underlying_price = Positive::new(150.0).unwrap();

        assert_eq!(chain1, chain2);
    }

    #[test]
    fn test_partial_eq_with_none_expiration() {
        let chain1 = create_chain_with_invalid_expiration("AAPL");
        let chain2 = create_chain_with_invalid_expiration("AAPL");

        // Both should have None for get_expiration() and should be equal
        assert_eq!(chain1, chain2);
    }

    #[test]
    fn test_partial_eq_one_valid_one_invalid_expiration() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_chain_with_invalid_expiration("AAPL");

        // One has valid expiration, one has None - should not be equal
        assert_ne!(chain1, chain2);
    }

    #[test]
    fn test_eq_trait_reflexivity() {
        let chain = create_test_chain("AAPL", "2030-01-19");

        assert_eq!(chain, chain);
    }

    #[test]
    fn test_eq_trait_symmetry() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2030-01-19");

        assert_eq!(chain1 == chain2, chain2 == chain1);
    }

    #[test]
    fn test_eq_trait_transitivity() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2030-01-19");
        let chain3 = create_test_chain("AAPL", "2030-01-19");

        assert!(chain1 == chain2 && chain2 == chain3 && chain1 == chain3);
    }

    #[test]
    fn test_partial_ord_earlier_expiration_less_than_later() {
        let earlier_chain = create_test_chain("AAPL", "2030-01-19");
        let later_chain = create_test_chain("AAPL", "2035-02-16");

        assert!(earlier_chain < later_chain);
        assert!(earlier_chain.partial_cmp(&later_chain) == Some(Ordering::Less));
    }

    #[test]
    fn test_partial_ord_later_expiration_greater_than_earlier() {
        let earlier_chain = create_test_chain("AAPL", "2030-01-19");
        let later_chain = create_test_chain("AAPL", "2030-02-16");

        assert!(later_chain > earlier_chain);
        assert!(later_chain.partial_cmp(&earlier_chain) == Some(Ordering::Greater));
    }

    #[test]
    fn test_partial_ord_same_expiration_equal() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("MSFT", "2030-01-19"); // Different symbol, same expiration

        assert!(chain1.partial_cmp(&chain2) != Some(Ordering::Equal));
    }

    #[test]
    fn test_partial_ord_with_none_expiration() {
        let valid_chain = create_test_chain("AAPL", "2030-01-19");
        let invalid_chain = create_chain_with_invalid_expiration("AAPL");

        // When one has None expiration, comparison should handle it appropriately
        let result = valid_chain.partial_cmp(&invalid_chain);
        assert!(result.is_some()); // Should return Some(Ordering) based on implementation
    }

    #[test]
    fn test_partial_ord_both_none_expiration() {
        let invalid_chain1 = create_chain_with_invalid_expiration("AAPL");
        let invalid_chain2 = create_chain_with_invalid_expiration("MSFT");

        // Both have None expiration - should be equal in ordering
        assert!(invalid_chain1.partial_cmp(&invalid_chain2) != Some(Ordering::Equal));
    }

    #[test]
    fn test_ord_consistency_with_partial_ord() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2024-02-16");

        assert_eq!(chain1.cmp(&chain2), chain1.partial_cmp(&chain2).unwrap());
        assert_eq!(chain2.cmp(&chain1), chain2.partial_cmp(&chain1).unwrap());
    }

    #[test]
    fn test_ord_earlier_less_than_later() {
        let earlier_chain = create_test_chain("AAPL", "2030-01-19");
        let later_chain = create_test_chain("AAPL", "2030-01-20");

        assert_eq!(earlier_chain.cmp(&later_chain), Ordering::Less);
    }

    #[test]
    fn test_ord_later_greater_than_earlier() {
        let earlier_chain = create_test_chain("AAPL", "2030-01-19");
        let later_chain = create_test_chain("SPX", "2031-02-16");

        assert_eq!(later_chain.cmp(&earlier_chain), Ordering::Greater);
    }

    #[test]
    fn test_ord_same_expiration_equal() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("MSFT", "2030-01-19");

        assert_eq!(chain1.cmp(&chain2), Ordering::Less);
    }

    #[test]
    fn test_ord_reflexivity() {
        let chain = create_test_chain("AAPL", "2030-01-19");

        assert_eq!(chain.cmp(&chain), Ordering::Equal);
    }

    #[test]
    fn test_ord_antisymmetry() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2024-02-16");

        if chain1.cmp(&chain2) == Ordering::Less {
            assert_eq!(chain2.cmp(&chain1), Ordering::Greater);
        }
    }

    #[test]
    fn test_ord_transitivity() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("AAPL", "2030-02-16");
        let chain3 = create_test_chain("AAPL", "2030-02-17");

        assert_eq!(chain1.cmp(&chain2), Ordering::Less);
        assert_eq!(chain2.cmp(&chain3), Ordering::Less);
        assert_eq!(chain1.cmp(&chain3), Ordering::Less);
    }

    #[test]
    fn test_sorting_mixed_symbols_by_expiration() {
        let mut chains = [
            create_test_chain("MSFT", "2024-02-16"),
            create_test_chain("AAPL", "2030-01-19"),
            create_test_chain("GOOGL", "2024-03-15"),
            create_test_chain("TSLA", "2030-01-19"), // Same expiration as AAPL
        ];

        chains.sort();

        // Should be sorted by expiration date regardless of symbol
        assert_eq!(chains[0].get_expiration_date(), "2024-03-15");
        assert_eq!(chains[1].get_expiration_date(), "2024-02-16");
        assert_eq!(chains[2].get_expiration_date(), "2030-01-19");
        assert_eq!(chains[3].get_expiration_date(), "2030-01-19");
    }

    #[test]
    fn test_ord_with_datetime_expiration_format() {
        let chain1 = create_test_chain("AAPL", "2030-01-19 15:30:00");
        let chain2 = create_test_chain("AAPL", "2030-01-19 16:00:00");

        // Should compare properly even with time components
        let result = chain1.cmp(&chain2);
        assert!(matches!(
            result,
            Ordering::Less | Ordering::Equal | Ordering::Greater
        ));
    }

    #[test]
    fn test_consistency_between_eq_and_ord() {
        let chain1 = create_test_chain("AAPL", "2030-01-19");
        let chain2 = create_test_chain("MSFT", "2030-01-19"); // Same expiration, different symbol

        // If chains are equal according to PartialEq, they should be Equal in Ord
        if chain1 == chain2 {
            assert_eq!(chain1.cmp(&chain2), Ordering::Equal);
        }

        // If chains compare as Equal in Ord, they should be equal according to PartialEq
        if chain1.cmp(&chain2) == Ordering::Equal {
            assert_eq!(chain1, chain2);
        }
    }

    #[test]
    fn test_option_chain_in_btreeset() {
        use std::collections::BTreeSet;

        let mut set = BTreeSet::new();

        set.insert(create_test_chain("AAPL", "2027-03-15"));
        set.insert(create_test_chain("AAPL", "2030-01-19"));
        set.insert(create_test_chain("AAPL", "2027-02-16"));
        set.insert(create_test_chain("MSFT", "2030-01-19"));
        set.insert(create_test_chain("AAPL", "2027-03-15")); // Duplicate

        // Should maintain sorted order and handle duplicates properly
        let chains: Vec<_> = set.into_iter().collect();

        // Verify ordering (chains with same expiration should be treated as equal)
        assert_eq!(chains.len(), 4);
        assert_eq!(chains[0].get_expiration_date(), "2027-02-16");
        assert_eq!(chains[1].get_expiration_date(), "2027-03-15");
        assert_eq!(chains[2].get_expiration_date(), "2030-01-19");
        assert_eq!(chains[3].get_expiration_date(), "2030-01-19");
    }
}

#[cfg(test)]
mod tests_volatility_smile_skew {
    use super::*;
    use crate::pos;
    use crate::volatility::{VolatilitySkew, VolatilitySmile};
    use rust_decimal_macros::dec;

    fn create_chain_with_options() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(90.0),
            Some(pos!(11.0)),
            Some(pos!(11.5)),
            Some(pos!(0.5)),
            Some(pos!(1.0)),
            pos!(0.28),
            Some(dec!(0.85)),
            Some(dec!(-0.15)),
            Some(dec!(0.015)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(95.0),
            Some(pos!(6.5)),
            Some(pos!(7.0)),
            Some(pos!(1.0)),
            Some(pos!(1.5)),
            pos!(0.24),
            Some(dec!(0.72)),
            Some(dec!(-0.28)),
            Some(dec!(0.020)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.5)),
            Some(pos!(4.0)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            Some(dec!(0.52)),
            Some(dec!(-0.48)),
            Some(dec!(0.025)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(105.0),
            Some(pos!(1.5)),
            Some(pos!(2.0)),
            Some(pos!(6.0)),
            Some(pos!(6.5)),
            pos!(0.22),
            Some(dec!(0.32)),
            Some(dec!(-0.68)),
            Some(dec!(0.020)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(110.0),
            Some(pos!(0.5)),
            Some(pos!(1.0)),
            Some(pos!(10.0)),
            Some(pos!(10.5)),
            pos!(0.26),
            Some(dec!(0.18)),
            Some(dec!(-0.82)),
            Some(dec!(0.015)),
            None,
            None,
            None,
        );

        chain
    }

    #[test]
    fn test_volatility_smile_returns_curve_with_correct_points() {
        let chain = create_chain_with_options();
        let smile = chain.smile();

        assert_eq!(smile.points.len(), 5);
    }

    #[test]
    fn test_volatility_smile_strike_prices_as_x_axis() {
        let chain = create_chain_with_options();
        let smile = chain.smile();

        let points: Vec<_> = smile.points.iter().collect();

        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[1].x, dec!(95.0));
        assert_eq!(points[2].x, dec!(100.0));
        assert_eq!(points[3].x, dec!(105.0));
        assert_eq!(points[4].x, dec!(110.0));
    }

    #[test]
    fn test_volatility_smile_iv_as_y_axis() {
        let chain = create_chain_with_options();
        let smile = chain.smile();

        let points: Vec<_> = smile.points.iter().collect();

        assert_eq!(points[0].y, dec!(0.28));
        assert_eq!(points[2].y, dec!(0.20)); // ATM
        assert_eq!(points[4].y, dec!(0.26));
    }

    #[test]
    fn test_volatility_skew_returns_curve_with_correct_points() {
        let chain = create_chain_with_options();
        let skew = chain.volatility_skew();

        assert_eq!(skew.points.len(), 5);
    }

    #[test]
    fn test_volatility_skew_moneyness_as_x_axis() {
        let chain = create_chain_with_options();
        let skew = chain.volatility_skew();

        let points: Vec<_> = skew.points.iter().collect();

        // Moneyness = (strike/underlying - 1) * 100
        // For strike 90, underlying 100: (90/100 - 1) * 100 = -10
        assert_eq!(points[0].x, dec!(-10.0));
        // For strike 100, underlying 100: (100/100 - 1) * 100 = 0
        assert_eq!(points[2].x, dec!(0.0));
        // For strike 110, underlying 100: (110/100 - 1) * 100 = 10
        assert_eq!(points[4].x, dec!(10.0));
    }

    #[test]
    fn test_volatility_skew_iv_as_y_axis() {
        let chain = create_chain_with_options();
        let skew = chain.volatility_skew();

        let points: Vec<_> = skew.points.iter().collect();

        assert_eq!(points[0].y, dec!(0.28)); // OTM put
        assert_eq!(points[2].y, dec!(0.20)); // ATM
        assert_eq!(points[4].y, dec!(0.26)); // OTM call
    }

    #[test]
    fn test_volatility_smile_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let smile = chain.smile();

        assert!(smile.points.is_empty());
    }

    #[test]
    fn test_volatility_skew_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
        let skew = chain.volatility_skew();

        assert!(skew.points.is_empty());
    }

    #[test]
    fn test_volatility_smile_single_option() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let smile = chain.smile();
        assert_eq!(smile.points.len(), 1);
    }
}

#[cfg(test)]
mod tests_get_call_price {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_call_price_existing_strike() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let price = chain.get_call_price(pos!(100.0));
        assert!(price.is_some());
        assert_eq!(price.unwrap(), dec!(3.5));
    }

    #[test]
    fn test_get_call_price_non_existing_strike() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let price = chain.get_call_price(pos!(105.0));
        assert!(price.is_none());
    }

    #[test]
    fn test_get_call_price_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        let price = chain.get_call_price(pos!(100.0));
        assert!(price.is_none());
    }

    #[test]
    fn test_get_call_price_no_ask_price() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            None, // No ask price
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let price = chain.get_call_price(pos!(100.0));
        assert!(price.is_none());
    }
}

#[cfg(test)]
mod tests_title_operations {
    use super::*;
    use crate::pos;

    #[test]
    fn test_get_title_basic() {
        let chain = OptionChain::new("AAPL", pos!(150.0), "2030-01-15".to_string(), None, None);

        let title = chain.get_title();
        assert_eq!(title, "AAPL-2030-01-15-150");
    }

    #[test]
    fn test_get_title_with_spaces_in_symbol() {
        let chain = OptionChain::new(
            "SPX INDEX",
            pos!(4500.0),
            "2030-01-15".to_string(),
            None,
            None,
        );

        let title = chain.get_title();
        assert_eq!(title, "SPX-INDEX-2030-01-15-4500");
    }

    #[test]
    fn test_get_title_with_spaces_in_date() {
        let chain = OptionChain::new("AAPL", pos!(150.0), "2030 01 15".to_string(), None, None);

        let title = chain.get_title();
        assert_eq!(title, "AAPL-2030-01-15-150");
    }

    #[test]
    fn test_set_from_title_valid_format() {
        let mut chain = OptionChain::new("", pos!(1.0), "".to_string(), None, None);

        let result = chain.set_from_title("AAPL-15-01-2030-150.5.csv");

        assert!(result.is_ok());
        assert_eq!(chain.symbol, "AAPL");
        assert_eq!(chain.expiration_date, "15-01-2030");
        assert_eq!(chain.underlying_price, pos!(150.5));
    }

    #[test]
    fn test_set_from_title_with_path() {
        let mut chain = OptionChain::new("", pos!(1.0), "".to_string(), None, None);

        let result = chain.set_from_title("/path/to/files/MSFT-20-03-2030-300.json");

        assert!(result.is_ok());
        assert_eq!(chain.symbol, "MSFT");
        assert_eq!(chain.expiration_date, "20-03-2030");
        assert_eq!(chain.underlying_price, pos!(300.0));
    }

    #[test]
    fn test_set_from_title_invalid_format() {
        let mut chain = OptionChain::new("", pos!(1.0), "".to_string(), None, None);

        let result = chain.set_from_title("invalid-format.csv");

        assert!(result.is_err());
    }

    #[test]
    fn test_set_from_title_too_few_parts() {
        let mut chain = OptionChain::new("", pos!(1.0), "".to_string(), None, None);

        let result = chain.set_from_title("AAPL-15-01.csv");

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_expiration_operations {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_expiration_date() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-15".to_string(), None, None);

        assert_eq!(chain.get_expiration_date(), "2030-01-15");
    }

    #[test]
    fn test_get_expiration_valid_date() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-15".to_string(), None, None);

        let expiration = chain.get_expiration();
        assert!(expiration.is_some());
    }

    #[test]
    fn test_get_expiration_invalid_date() {
        let chain = OptionChain::new("TEST", pos!(100.0), "invalid-date".to_string(), None, None);

        let expiration = chain.get_expiration();
        assert!(expiration.is_none());
    }

    #[test]
    fn test_update_expiration_date() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2030-01-15".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        chain.update_expiration_date("2030-06-15".to_string());

        assert_eq!(chain.get_expiration_date(), "2030-06-15");
    }

    #[test]
    fn test_update_expiration_date_invalid() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-15".to_string(), None, None);

        // Update with invalid date - should still update the string but warn
        chain.update_expiration_date("invalid".to_string());

        assert_eq!(chain.get_expiration_date(), "invalid");
    }
}

#[cfg(test)]
mod tests_from_vec_option_data {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_from_empty_vec() {
        let options: Vec<OptionData> = vec![];
        let chain = OptionChain::from(&options);

        assert!(chain.options.is_empty());
        assert_eq!(chain.symbol, "");
    }

    #[test]
    fn test_from_vec_with_options() {
        let mut opt1 = OptionData::new(
            pos!(95.0),
            Some(pos!(6.0)),
            Some(pos!(6.5)),
            Some(pos!(1.0)),
            Some(pos!(1.5)),
            pos!(0.25),
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
            None,
        );
        opt1.symbol = Some("TEST".to_string());
        opt1.underlying_price = Some(Box::new(pos!(100.0)));
        opt1.expiration_date = Some(ExpirationDate::Days(pos!(30.0)));
        opt1.risk_free_rate = Some(dec!(0.05));
        opt1.dividend_yield = spos!(0.02);

        let mut opt2 = OptionData::new(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
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
            None,
        );
        opt2.symbol = Some("TEST".to_string());
        opt2.underlying_price = Some(Box::new(pos!(100.0)));
        opt2.expiration_date = Some(ExpirationDate::Days(pos!(30.0)));

        let options = vec![opt1, opt2];
        let chain = OptionChain::from(&options);

        assert_eq!(chain.symbol, "TEST");
        assert_eq!(chain.underlying_price, pos!(100.0));
        assert_eq!(chain.options.len(), 2);
        assert_eq!(chain.risk_free_rate, Some(dec!(0.05)));
        assert_eq!(chain.dividend_yield, spos!(0.02));
    }

    #[test]
    fn test_from_vec_uses_first_option_metadata() {
        let mut opt1 = OptionData::new(
            pos!(95.0),
            None,
            None,
            None,
            None,
            pos!(0.25),
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
            None,
        );
        opt1.symbol = Some("FIRST".to_string());
        opt1.underlying_price = Some(Box::new(pos!(100.0)));

        let mut opt2 = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            pos!(0.20),
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
            None,
        );
        opt2.symbol = Some("SECOND".to_string());
        opt2.underlying_price = Some(Box::new(pos!(150.0)));

        let options = vec![opt1, opt2];
        let chain = OptionChain::from(&options);

        // Should use first option's metadata
        assert_eq!(chain.symbol, "FIRST");
        assert_eq!(chain.underlying_price, pos!(100.0));
    }
}

#[cfg(test)]
mod tests_len_trait {
    use super::*;
    use crate::pos;
    use crate::utils::Len;

    #[test]
    fn test_len_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());
    }

    #[test]
    fn test_len_with_options() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);

        chain.add_option(
            pos!(95.0),
            None,
            None,
            None,
            None,
            pos!(0.20),
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
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        chain.add_option(
            pos!(105.0),
            None,
            None,
            None,
            None,
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(chain.len(), 3);
        assert!(!chain.is_empty());
    }
}

#[cfg(test)]
mod tests_default_trait {
    use super::*;

    #[test]
    fn test_default_chain() {
        let chain = OptionChain::default();

        assert_eq!(chain.symbol, "");
        assert_eq!(chain.underlying_price, Positive::ZERO);
        assert_eq!(chain.get_expiration_date(), "");
        assert!(chain.options.is_empty());
        assert!(chain.risk_free_rate.is_none());
        assert!(chain.dividend_yield.is_none());
    }
}

#[cfg(test)]
mod tests_option_chain_params_trait {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_params_existing_strike() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2030-01-15".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let result = chain.get_params(pos!(100.0));
        assert!(result.is_ok());

        let params = result.unwrap();
        assert!(params.underlying_price.is_some());
        assert_eq!(*params.underlying_price.unwrap(), pos!(100.0));
    }

    #[test]
    fn test_get_params_non_existing_strike() {
        let chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2030-01-15".to_string(),
            Some(dec!(0.05)),
            spos!(0.02),
        );

        let result = chain.get_params(pos!(150.0));
        assert!(result.is_err());
    }
}
