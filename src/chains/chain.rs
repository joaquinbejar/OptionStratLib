/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::chains::utils::{
    OptionChainBuildParams, OptionChainParams, OptionDataPriceParams, RandomPositionsParams,
    adjust_volatility, default_empty_string, rounder,
};
use crate::chains::{OptionData, OptionsInStrike, RNDAnalysis, RNDParameters, RNDResult};
use crate::curves::{BasicCurves, Curve, Point2D};
use crate::error::chains::ChainError;
use crate::error::{CurveError, SurfaceError};
use crate::geometrics::{LinearInterpolation, MetricsExtractor};
use crate::greeks::Greeks;
use crate::model::{
    BasicAxisTypes, ExpirationDate, OptionStyle, OptionType, Options, Position, Side,
};
use crate::strategies::utils::FindOptimalSide;
use crate::surfaces::{BasicSurfaces, Point3D, Surface};
use crate::utils::Len;
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
use tracing::{debug, error, warn};
use {crate::chains::utils::parse, csv::WriterBuilder, std::fs::File};

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
#[derive(Debug, Clone)]
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
    ///     dec!(0.1),
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
            Some(params.price_params.risk_free_rate),
            Some(params.price_params.dividend_yield),
        );

        fn create_chain_data(s: &Positive, p: &OptionChainBuildParams) -> OptionData {
            let atm_distance = s.to_dec() - p.price_params.underlying_price;
            let adjusted_volatility = adjust_volatility(
                p.price_params.implied_volatility,
                p.skew_factor,
                atm_distance.to_f64().unwrap(),
            );
            let mut option_data = OptionData::new(
                *s,
                None,
                None,
                None,
                None,
                adjusted_volatility,
                None,
                None,
                None,
                p.volume,
                None,
            );
            let price_params = OptionDataPriceParams::new(
                p.price_params.underlying_price,
                p.price_params.expiration_date,
                adjusted_volatility,
                p.price_params.risk_free_rate,
                p.price_params.dividend_yield,
                p.price_params.underlying_symbol.clone(),
            );

            let result_calculate_prices = option_data.calculate_prices(&price_params, false);
            if result_calculate_prices.is_ok() {
                option_data.apply_spread(p.spread, p.decimal_places);
                option_data.calculate_delta(&price_params);
                option_data.calculate_gamma(&price_params);
            } else {
                warn!(
                    "Failed to calculate prices for strike: {} error: {}",
                    s,
                    result_calculate_prices.unwrap_err()
                );
            }
            option_data
        }
        let atm_strike = rounder(params.price_params.underlying_price, params.strike_interval);
        let atm_strike_option_data = create_chain_data(&atm_strike.clone(), params);
        option_chain.options.insert(atm_strike_option_data);

        let mut counter = Positive::ONE;
        loop {
            let next_upper_strike = atm_strike + (params.strike_interval * counter);
            let next_upper_option_data = create_chain_data(&next_upper_strike, params);
            option_chain.options.insert(next_upper_option_data.clone());

            let strike_step = (params.strike_interval * counter).to_dec();
            if strike_step > atm_strike.to_dec() {
                break;
            }
            let next_lower_strike = atm_strike - (params.strike_interval * counter).to_dec();
            let next_lower_option_data = create_chain_data(&next_lower_strike, params);
            option_chain.options.insert(next_lower_option_data.clone());

            if next_upper_option_data.some_price_is_none()
                && next_lower_option_data.some_price_is_none()
            {
                break;
            }
            counter += Positive::ONE;
            if counter > pos!(200.0) {
                break;
            }
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
    pub fn to_build_params(&self) -> Result<OptionChainBuildParams, Box<dyn Error>> {
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
        let implied_volatility = match self.atm_implied_volatility() {
            Ok(Some(iv)) => {
                assert!(*iv >= pos!(0.0) && *iv <= pos!(1.0));
                Some(*iv)
            }
            _ => Some(pos!(0.2)), // 20% is a reasonable default IV
        };

        let volatility_curve =
            self.curve(&BasicAxisTypes::Volatility, &OptionStyle::Call, &Side::Long)?;
        let skew_factor =
            volatility_curve.compute_shape_metrics()?.skewness / Decimal::from(100000);
        // let skew_factor = Decimal::ZERO;

        // Create the price parameters
        let price_params = OptionDataPriceParams::new(
            self.underlying_price,
            ExpirationDate::from_string(&self.expiration_date)?,
            implied_volatility,
            self.risk_free_rate.unwrap_or(Decimal::ZERO),
            self.dividend_yield.unwrap_or(Positive::ZERO),
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
            strike_interval,
            skew_factor,
            spread,
            decimal_places,
            price_params,
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
                FindOptimalSide::Deltable(_threshold) => true,
                FindOptimalSide::Center => {
                    panic!("Center should be managed by the strategy");
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
    /// * `Err(Box<dyn Error>)` - Error if the option chain is empty or if the operation fails
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
    pub fn atm_strike(&self) -> Result<&Positive, Box<dyn Error>> {
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
    /// * `Err(Box<dyn Error>)` - If the option chain is empty or no ATM option can be found,
    ///   returns an error describing the failure.
    ///
    /// # Errors
    ///
    /// This function returns an error in the following cases:
    ///
    /// * The option chain (`self.options`) is empty.
    /// * No option with a strike price close to the underlying price can be found.
    pub fn atm_option_data(&self) -> Result<&OptionData, Box<dyn Error>> {
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
    pub fn load_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut option_chain: OptionChain = serde_json::from_reader(file)?;
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
                if option_clone.implied_volatility.is_some() {
                    option_clone.check_and_convert_implied_volatility();
                }
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
        // keep it for back compatibility
        match self.atm_implied_volatility() {
            Ok(iv) => match iv {
                Some(iv) => Ok(iv.value()),
                None => Err("No ATM implied volatility available".to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
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
    /// * `Err(Box<dyn Error>)` - If the ATM option cannot be found, returns an error.
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying `atm_option_data()` call fails,
    /// which can happen if the option chain is empty or no suitable ATM option is found.
    pub fn atm_implied_volatility(&self) -> Result<&Option<Positive>, Box<dyn Error>> {
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
                let four = match opt.options.as_ref() {
                    Some(four) => four,
                    None => {
                        error!("No options greeks initialized. Please run the update_greeks method first.");
                        return None;
                    }
                };

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
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            pos!(1.0),
            Decimal::ZERO,
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
        assert!(chain.options.len() >= 21);
        assert_eq!(chain.underlying_price, pos!(100.0));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 14.01);
        assert_eq!(first.call_bid.unwrap(), 13.99);
        assert_eq!(first.put_ask, None);
        assert_eq!(first.put_bid, None);
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, None);
        assert_eq!(last.call_bid, None);
        assert_eq!(last.put_ask, spos!(14.02));
        assert_eq!(last.put_bid, spos!(14.0));
    }

    #[test]

    fn test_new_option_chain_build_chain_long() {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            25,
            pos!(25.0),
            dec!(0.000002),
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
        assert!(chain.options.len() > 1);
        assert_eq!(chain.underlying_price, pos!(5878.10));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 303.11);
        assert_eq!(first.call_bid.unwrap(), 303.09);
        assert_eq!(first.put_ask, None);
        assert_eq!(first.put_bid, None);
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, None);
        assert_eq!(last.call_bid, None);
        assert_eq!(last.put_ask, spos!(296.92));
        assert_eq!(last.put_bid, spos!(296.90));
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

    fn test_validate_valid_option() {
        setup_logger();
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

    fn test_validate_no_implied_volatility() {
        let mut option_data = create_valid_option_data();
        option_data.implied_volatility = None;
        assert!(!option_data.validate());
    }

    #[test]

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
    use crate::{pos, spos};

    #[test]

    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        assert_eq!(chain.strike_price_range_vec(5.0), None);
    }

    #[test]

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

    fn test_get_double_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]

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
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]

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

    fn test_get_triple_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]

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
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]

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
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]

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
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]

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
            assert_eq!(
                result.unwrap_err().to_string(),
                "Derivative tolerance must be greater than zero"
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
            assert_eq!(
                result.unwrap_err().to_string(),
                "Derivative tolerance must be greater than zero"
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
            assert_eq!(
                result.unwrap_err().to_string(),
                "Cannot find ATM OptionData for empty option chain: TEST"
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
            option_data.implied_volatility.unwrap(),
            pos!(0.13008),
            pos!(0.0001)
        );

        let iv = option_data.implied_volatility.unwrap();
        assert!(iv > pos!(0.0) && iv <= pos!(1.0));
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
                option.implied_volatility.unwrap() ,
                original_iv.unwrap(),
                pos!(0.1)
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
            spos!(0.35),
            Some(dec!(0.3)),
            Some(dec!(-0.7)),
            None, // No gamma value
            spos!(60.0),
            Some(30),
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
        assert_eq!(initial_delta, dec!(31.0));

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

#[cfg(test)]
mod tests_atm_strike {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::model::types::ExpirationDate;
    use crate::utils::logger::setup_logger;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_standard_chain() -> OptionChain {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            pos!(1.0),
            Decimal::ZERO,
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

        // The lowest strike in the standard chain should be around 90.0
        assert_eq!(
            *strike,
            pos!(86.0),
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
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
        ));

        options.insert(OptionData::new(
            pos!(101.0),
            spos!(0.9),
            spos!(1.0),
            spos!(1.1),
            spos!(1.2),
            spos!(0.2),
            Some(dec!(0.55)),
            Some(dec!(-0.45)),
            Some(dec!(0.1)),
            spos!(100.0),
            Some(50),
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
    use crate::chains::utils::OptionChainBuildParams;
    use crate::chains::utils::OptionDataPriceParams;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::spos;
    use crate::utils::logger::setup_logger;
    use rust_decimal_macros::dec;

    // Helper function to create a standard option chain for testing
    fn create_standard_chain() -> OptionChain {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            pos!(1.0),
            Decimal::ZERO,
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

        OptionChain::build_chain(&params)
    }

    // Helper function to create a chain with custom strikes for specific tests
    fn create_custom_strike_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-01-01".to_string(),
            Some(dec!(0.05)),
            Some(pos!(0.02)),
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
                spos!(vols[i]),
                Some(Decimal::from_f64(deltas_call[i]).unwrap()),
                Some(Decimal::from_f64(deltas_put[i]).unwrap()),
                Some(Decimal::from_f64(gammas[i]).unwrap()),
                spos!(100.0),
                Some(50),
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
        let chain = OptionChain::new("EMPTY", pos!(100.0), "2024-01-01".to_string(), None, None);

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
            OptionChain::new("SINGLE", pos!(100.0), "2024-01-01".to_string(), None, None);

        chain.add_option(
            pos!(100.0),
            spos!(5.0),
            spos!(5.5),
            spos!(4.0),
            spos!(4.5),
            spos!(0.2),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.04)),
            spos!(100.0),
            Some(50),
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
            "2024-01-01".to_string(),
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
                spos!(0.2),
                Some(dec!(0.5)),
                Some(dec!(-0.5)),
                Some(dec!(0.04)),
                spos!(100.0),
                Some(50),
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
mod tests_to_build_params {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::model::types::ExpirationDate;
    use crate::utils::logger::setup_logger;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    use tracing::info;

    fn create_standard_chain() -> OptionChain {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            22,
            pos!(25.0),
            dec!(0.000001),
            pos!(0.03),
            2,
            OptionDataPriceParams::new(
                pos!(5000.0),
                ExpirationDate::Days(pos!(30.0)),
                spos!(0.1),
                dec!(0.05),
                pos!(0.05),
                Some("SP500".to_string()),
            ),
        );

        OptionChain::build_chain(&params)
    }

    #[test]
    fn test_to_build_params_simple() {
        let chain = create_standard_chain();
        info!("{}", chain);
        let mut params = chain.to_build_params().unwrap();

        params.skew_factor = dec!(0.000001);
        params.price_params.underlying_price =
            pos!(params.price_params.underlying_price.to_f64() * f64::exp(0.2)).max(Positive::ZERO);
        params.price_params.implied_volatility = Some(
            pos!(params.price_params.implied_volatility.unwrap().to_f64() * f64::exp(0.2))
                .max(Positive::ZERO),
        );
        info!("{}", params);

        let new_chain = OptionChain::build_chain(&params);
        info!("{}", new_chain);
    }
}
