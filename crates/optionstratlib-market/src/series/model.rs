use crate::chains::OptionChain;
use crate::error::ChainError;
use crate::series::params::OptionSeriesBuildParams;
use chrono::NaiveDate;
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Positive;
use optionstratlib_core::utils::Len;
use optionstratlib_core::{impl_json_debug_pretty, impl_json_display};
use rust_decimal::Decimal;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use std::collections::BTreeMap;
use std::fmt;

/// Represents a series of option chains for an underlying asset,
/// providing detailed information about its options market and related financial data.
///
/// # Serialization
///
/// The 0.22 contract (JSON shown; any serde format works) is an object with
/// `symbol`, `underlying_price`, `chains`, and the optional
/// `risk_free_rate` and `dividend_yield`, which are omitted when `None`.
/// `chains` is a map from an expiration date written as `YYYY-MM-DD` to the
/// [`OptionChain`] for that date, ordered by date. The dates are canonical:
/// deserialization reads each key back as an absolute
/// `ExpirationDate::DateTime` at 18:30 UTC on that date, the time the
/// `expiration_date` crate gives a date-only expiry, so write-then-read
/// returns the same `YYYY-MM-DD` keys whenever it is read. An expiry written
/// as `ExpirationDate::Days` therefore comes back as a `DateTime`. A key that
/// is not a `YYYY-MM-DD` date is an error, not a skipped chain.
#[derive(Clone)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct OptionSeries {
    /// The ticker symbol for the underlying asset (e.g., "AAPL", "SPY").
    pub symbol: String,

    /// The current market price of the underlying asset.
    pub underlying_price: Positive,

    /// A sorted collection of option chains, each corresponding to a different expiration date.
    pub chains: BTreeMap<ExpirationDate, OptionChain>,

    /// The risk-free interest rate used for option pricing models.
    pub risk_free_rate: Option<Decimal>,

    /// The annual dividend yield of the underlying asset.
    pub dividend_yield: Option<Positive>,
}

impl_json_debug_pretty!(OptionSeries);
impl_json_display!(OptionSeries);

impl OptionSeries {
    /// Creates a new instance of the struct with the specified symbol and underlying price.
    ///
    /// # Parameters
    /// - `symbol`: A `String` representing the symbol of the entity being created (e.g., a stock or asset).
    /// - `underlying_price`: A `Positive` value representing the current price of the underlying asset.
    ///   This must be a positive value.
    ///
    /// # Returns
    /// A new instance of the struct initialized with:
    /// - The provided `symbol` and `underlying_price`.
    /// - An empty `chains` field of type `BTreeMap`.
    /// - `None` for both `risk_free_rate` and `dividend_yield`.
    ///
    #[inline]
    #[must_use]
    pub fn new(symbol: String, underlying_price: Positive) -> Self {
        Self {
            symbol,
            underlying_price,
            chains: BTreeMap::new(),
            risk_free_rate: None,
            dividend_yield: None,
        }
    }

    /// Retrieves the nearest expiring option chain from the collection of option chains.
    ///
    /// # Returns
    /// - `Some(OptionChain)` if there is an option chain with the closest expiration date that is within 1 day or less.
    /// - `None` if there are no option chains or if the nearest expiration date is more than 1 day away.
    ///
    /// # Behavior
    /// The function checks the first key-value pair in the `chains` collection. If the expiration date is
    /// within one day (`ExpirationDate::Days(Positive::ONE)`), it returns a cloned instance of the corresponding
    /// `OptionChain`. Otherwise, it returns `None`.
    ///
    /// # Notes
    /// - The `chains` collection must be ordered by expiration date for this function to work correctly.
    /// - This method assumes that there is a defined threshold of "1 day" to determine closeness.
    ///
    /// # Errors
    /// This function does not return errors but may return `None` if no conditions are met.
    #[must_use]
    pub fn odte(&self) -> Option<OptionChain> {
        match self.chains.first_key_value() {
            Some((expiration_date, option_chain)) => {
                if expiration_date <= &ExpirationDate::Days(Positive::ONE) {
                    Some(option_chain.clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Retrieves the expiration dates associated with the chains.
    ///
    /// This function iterates through the keys of the `chains` field, attempting to extract
    /// expiration dates by calling the `get_days` method on each key. The expiration dates
    /// are collected into a `Vec<Positive>` and returned. If any error occurs during this
    /// process, a boxed error is returned.
    ///
    /// # Returns
    /// * `Ok(Vec<Positive>)` - A vector of expiration dates represented as `Positive` values
    ///   if all operations succeed.
    /// * `Err(Box<dyn Error>)` - A boxed error if any step in retrieving or mapping the keys fails.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The `get_days` method on any key fails.
    /// - The process of mapping and collecting the keys fails.
    ///
    pub fn get_expiration_dates(&self) -> Result<Vec<Positive>, ChainError> {
        self.chains
            .keys()
            .map(|e| e.get_days())
            .collect::<Result<Vec<Positive>, _>>()
            .map_err(|e| e.into())
    }

    /// Builds an option series object (`Self`) based on the provided parameters.
    ///
    /// This method takes in an `OptionSeriesBuildParams` object, clones its data, and constructs
    /// a series of option chains for each expiration date specified in the input parameters.
    /// Each option chain is built and associated with its corresponding expiration date, and the
    /// resulting data is stored in a `BTreeMap` for ordered access.
    ///
    /// # Parameters
    /// - `params`: A reference to an `OptionSeriesBuildParams` object, which contains configuration
    ///   details such as series to generate, price parameters, symbol, and chain parameters.
    ///
    /// # Returns
    /// A new instance of the object (`Self`) representing the constructed option series, which includes:
    /// - `symbol`: The symbol associated with the series.
    /// - `underlying_price`: The price of the underlying asset.
    /// - `chains`: A `BTreeMap` mapping expiration dates (`ExpirationDate`) to their corresponding
    ///   option chains (`OptionChain`).
    /// - `risk_free_rate`: The risk-free interest rate, extracted from the input parameters, if specified.
    /// - `dividend_yield`: The dividend yield of the underlying asset, extracted from the input parameters, if specified.
    ///
    /// # Process
    /// 1. Clones the input parameters for local modifications.
    /// 2. Iterates over each expiration date in the `series` field of the parameters.
    /// 3. For each expiration date:
    ///    - Converts it into an `ExpirationDate` type.
    ///    - Updates the chain parameters by setting the expiration date and resetting the strike interval.
    ///    - Builds an individual option chain using the updated chain parameters.
    ///    - Updates the expiration date string within the chain.
    ///    - Inserts the constructed chain into the `BTreeMap` with its associated expiration date.
    /// 4. Constructs and returns the resulting instance of the option series with all computed data.
    ///
    ///
    /// # Notes
    /// - This method assumes that valid expiration dates and series data are provided. Ensure proper
    ///   validation of `params` before calling this method.
    /// - The use of a `BTreeMap` ensures that the resulting chains are sorted based on the expiration dates.
    ///
    /// # Errors
    ///
    /// Returns `ChainError` if:
    /// - Failed to build any option chain in the series
    /// - Failed to get date string from expiration date
    /// - Missing underlying price in price params
    #[inline(never)]
    pub fn build_series(params: &OptionSeriesBuildParams) -> Result<Self, ChainError> {
        let mut params = params.clone();
        let mut chains: BTreeMap<ExpirationDate, OptionChain> = BTreeMap::new();
        for series in params.series.clone().into_iter() {
            let expiration_date: ExpirationDate = ExpirationDate::Days(series);
            params.chain_params.price_params.expiration_date = Some(expiration_date);
            params.chain_params.strike_interval = None;
            let mut chain: OptionChain = OptionChain::build_chain(&params.chain_params)?;
            let date_string = expiration_date.get_date_string().map_err(|e| {
                ChainError::invalid_parameters(
                    "expiration_date",
                    &format!("failed to get date string: {e}"),
                )
            })?;
            chain.update_expiration_date(date_string);
            chains.insert(expiration_date, chain);
        }
        let price_params = params.chain_params.price_params.clone();
        let underlying_price = *price_params.underlying_price.ok_or_else(|| {
            ChainError::invalid_parameters("underlying_price", "missing underlying price")
        })?;
        Ok(Self {
            symbol: params.chain_params.symbol.clone(),
            underlying_price,
            chains,
            risk_free_rate: price_params.risk_free_rate,
            dividend_yield: price_params.dividend_yield,
        })
    }

    /// Converts the current object to `OptionSeriesBuildParams`.
    ///
    /// This method performs the following steps:
    /// 1. Attempts to retrieve the first key-value pair from `self.chains`.
    /// 2. Fetches expiration dates by calling `self.get_expiration_dates()`.
    /// 3. Extracts chain parameters by calling `to_build_params` on the first
    ///    option chain (if found).
    /// 4. If no chains are available, returns an error indicating that no chains
    ///    were found.
    ///
    /// # Returns
    /// - On success, returns `Ok(OptionSeriesBuildParams)` which contains:
    ///     - Chain parameters (`chain_params`) obtained from the first option chain.
    ///     - Expiration dates (`series`).
    /// - On failure, returns an `Err` wrapped in a `Box<dyn Error>` with appropriate
    ///   error details.
    ///
    /// # Errors
    /// - Returns an error if there are no chains in `self.chains`.
    /// - Propagates any errors encountered by:
    ///     - `self.get_expiration_dates()`.
    ///     - `option_chain.to_build_params()`.
    /// # Related
    /// - `OptionSeriesBuildParams`: The resulting struct after conversion.
    /// - `to_build_params()`: Method on individual `option_chain` objects to extract parameters.
    pub fn to_build_params(&self) -> Result<OptionSeriesBuildParams, ChainError> {
        let chain_params = self.chains.first_key_value();
        let series = self.get_expiration_dates()?;
        let chain_params = match chain_params {
            Some((_, option_chain)) => option_chain.to_build_params()?,
            None => {
                return Err(ChainError::invalid_parameters(
                    "chains",
                    "no chains available to derive build parameters",
                ));
            }
        };

        Ok(OptionSeriesBuildParams {
            chain_params,
            series,
        })
    }
}

impl Default for OptionSeries {
    fn default() -> Self {
        Self::new("".to_string(), Positive::ZERO)
    }
}

impl From<&OptionSeries> for Positive {
    fn from(value: &OptionSeries) -> Self {
        value.underlying_price
    }
}

impl From<OptionSeries> for Positive {
    fn from(value: OptionSeries) -> Self {
        value.underlying_price
    }
}

impl Len for OptionSeries {
    fn len(&self) -> usize {
        self.chains.len()
    }

    fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }
}

// Custom serialization implementation
impl Serialize for OptionSeries {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OptionSeries", 5)?;

        state.serialize_field("symbol", &self.symbol)?;
        state.serialize_field("underlying_price", &self.underlying_price)?;

        // Serialize chains as a map of string dates to OptionChain.
        // Surface any date-formatting failure as a serde error rather
        // than panicking inside the closure.
        let mut chains_map: BTreeMap<String, &OptionChain> = BTreeMap::new();
        for (date, chain) in &self.chains {
            let key = date.get_date_string().map_err(serde::ser::Error::custom)?;
            chains_map.insert(key, chain);
        }
        state.serialize_field("chains", &chains_map)?;

        // Serialize optional fields
        if let Some(rate) = &self.risk_free_rate {
            state.serialize_field("risk_free_rate", rate)?;
        }

        if let Some(yield_val) = &self.dividend_yield {
            state.serialize_field("dividend_yield", yield_val)?;
        }

        state.end()
    }
}

/// Reads a `chains` key written by `ExpirationDate::get_date_string` back as
/// an absolute expiration at 18:30 UTC on that date, so formatting it again
/// gives the same key. The key is parsed here rather than through
/// `ExpirationDate::from_string`, which also overwrites the thread-local
/// reference datetime that `ExpirationDate::Days` resolves against.
fn expiration_from_key(key: &str) -> Result<ExpirationDate, String> {
    let date = NaiveDate::parse_from_str(key, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {key:?} is not YYYY-MM-DD: {e}"))?;
    let expiry = date
        .and_hms_opt(18, 30, 0)
        .ok_or_else(|| format!("Invalid date format: no 18:30 UTC on {key:?}"))?;
    Ok(ExpirationDate::DateTime(expiry.and_utc()))
}

// Custom deserialization implementation
impl<'de> Deserialize<'de> for OptionSeries {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define the fields we expect to see
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Symbol,
            UnderlyingPrice,
            Chains,
            RiskFreeRate,
            DividendYield,
        }

        // Create a visitor to handle the deserialization
        struct OptionSeriesVisitor;

        impl<'de> Visitor<'de> for OptionSeriesVisitor {
            type Value = OptionSeries;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct OptionSeries")
            }

            fn visit_map<V>(self, mut map: V) -> Result<OptionSeries, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut symbol = None;
                let mut underlying_price = None;
                let mut string_chains: Option<BTreeMap<String, OptionChain>> = None;
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
                                return Err(de::Error::duplicate_field("underlying_price"));
                            }
                            underlying_price = Some(map.next_value()?);
                        }
                        Field::Chains => {
                            if string_chains.is_some() {
                                return Err(de::Error::duplicate_field("chains"));
                            }
                            string_chains = Some(map.next_value()?);
                        }
                        Field::RiskFreeRate => {
                            if risk_free_rate.is_some() {
                                return Err(de::Error::duplicate_field("risk_free_rate"));
                            }
                            risk_free_rate = Some(map.next_value()?);
                        }
                        Field::DividendYield => {
                            if dividend_yield.is_some() {
                                return Err(de::Error::duplicate_field("dividend_yield"));
                            }
                            dividend_yield = Some(map.next_value()?);
                        }
                    }
                }

                let symbol = symbol.ok_or_else(|| de::Error::missing_field("symbol"))?;
                let underlying_price =
                    underlying_price.ok_or_else(|| de::Error::missing_field("underlying_price"))?;
                let string_chains =
                    string_chains.ok_or_else(|| de::Error::missing_field("chains"))?;

                // Read the `YYYY-MM-DD` keys back as absolute expirations.
                let mut chains = BTreeMap::new();
                for (date_str, chain) in string_chains {
                    let expiration_date =
                        expiration_from_key(&date_str).map_err(de::Error::custom)?;
                    chains.insert(expiration_date, chain);
                }

                Ok(OptionSeries {
                    symbol,
                    underlying_price,
                    chains,
                    risk_free_rate,
                    dividend_yield,
                })
            }
        }

        // Define the fields for our struct
        const FIELDS: &[&str] = &[
            "symbol",
            "underlying_price",
            "chains",
            "risk_free_rate",
            "dividend_yield",
        ];

        // Use our visitor to deserialize
        deserializer.deserialize_struct("OptionSeries", FIELDS, OptionSeriesVisitor)
    }
}

#[cfg(test)]
mod tests_option_series {
    use super::*;
    use optionstratlib_core::{model::Positive, pos_or_panic, spos};

    use crate::chains::OptionChain;
    use crate::series::params::OptionSeriesBuildParams;
    use optionstratlib_core::utils::Len;
    use optionstratlib_core::utils::time::get_x_days_formatted_pos;

    use rust_decimal_macros::dec;

    // Helper function to create a simple OptionChain for testing
    fn create_test_chain(expiration_days: Positive) -> OptionChain {
        let date =
            get_x_days_formatted_pos(expiration_days).expect("test expiration fits the calendar");
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            date,
            Some(dec!(0.05)),
            spos!(0.02),
        );

        // Add a simple option to the chain
        chain.add_option(
            Positive::HUNDRED,
            spos!(5.0),
            spos!(5.5),
            spos!(4.5),
            spos!(5.0),
            pos_or_panic!(0.2),
            None,
            None,
            None,
            spos!(100.0),
            Some(50),
            None,
        );

        chain
    }

    // Helper function to create a basic OptionSeries for testing
    fn create_test_series() -> OptionSeries {
        let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

        // Add chains with different expiration dates
        series.chains.insert(
            ExpirationDate::Days(Positive::ONE),
            create_test_chain(Positive::ONE),
        );
        series.chains.insert(
            ExpirationDate::Days(pos_or_panic!(7.0)),
            create_test_chain(pos_or_panic!(7.0)),
        );
        series.chains.insert(
            ExpirationDate::Days(pos_or_panic!(30.0)),
            create_test_chain(pos_or_panic!(30.0)),
        );

        series.risk_free_rate = Some(dec!(0.05));
        series.dividend_yield = spos!(0.02);

        series
    }

    mod tests_construction {
        use super::*;

        #[test]
        fn test_new_construction() {
            let series = OptionSeries::new("SPY".to_string(), pos_or_panic!(450.0));

            assert_eq!(series.symbol, "SPY");
            assert_eq!(series.underlying_price, pos_or_panic!(450.0));
            assert!(series.chains.is_empty());
            assert_eq!(series.risk_free_rate, None);
            assert_eq!(series.dividend_yield, None);
        }

        #[test]
        fn test_default_construction() {
            let series = OptionSeries::default();

            assert_eq!(series.symbol, "");
            assert_eq!(series.underlying_price, Positive::ZERO);
            assert!(series.chains.is_empty());
            assert_eq!(series.risk_free_rate, None);
            assert_eq!(series.dividend_yield, None);
        }
    }

    mod tests_odte_method {
        use super::*;

        #[test]
        fn test_odte_with_valid_chain() {
            let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // Add a chain with expiration of 1 day or less
            series.chains.insert(
                ExpirationDate::Days(pos_or_panic!(0.5)),
                create_test_chain(pos_or_panic!(0.5)),
            );

            let odte_chain = series.odte();
            assert!(odte_chain.is_some());
            assert_eq!(odte_chain.unwrap().symbol, "TEST");
        }

        #[test]
        fn test_odte_with_invalid_chain() {
            let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // Add a chain with expiration longer than 1 day
            series.chains.insert(
                ExpirationDate::Days(Positive::TWO),
                create_test_chain(Positive::TWO),
            );

            let odte_chain = series.odte();
            assert!(odte_chain.is_none());
        }

        #[test]
        fn test_odte_with_empty_chains() {
            let series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // No chains added
            let odte_chain = series.odte();
            assert!(odte_chain.is_none());
        }

        #[test]
        fn test_odte_with_exact_one_day() {
            let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // Add a chain with exactly 1 day expiration
            series.chains.insert(
                ExpirationDate::Days(Positive::ONE),
                create_test_chain(Positive::ONE),
            );

            let odte_chain = series.odte();
            assert!(odte_chain.is_some());
        }
    }

    mod tests_get_expiration_dates {
        use super::*;

        #[test]
        fn test_get_expiration_dates_normal_case() {
            let series = create_test_series();

            let result = series.get_expiration_dates();
            assert!(result.is_ok());

            let dates = result.unwrap();
            assert_eq!(dates.len(), 3);

            // Verify the dates are in the correct order (BTreeMap sorts keys)
            assert_eq!(dates[0], Positive::ONE);
            assert_eq!(dates[1], pos_or_panic!(7.0));
            assert_eq!(dates[2], pos_or_panic!(30.0));
        }

        #[test]
        fn test_get_expiration_dates_empty_chains() {
            let series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            let result = series.get_expiration_dates();
            assert!(result.is_ok());

            let dates = result.unwrap();
            assert!(dates.is_empty());
        }
    }

    mod tests_build_series {
        use super::*;

        use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};

        #[test]
        fn test_build_series_basic() {
            // Create price params
            let price_params = OptionDataPriceParams::new(
                Some(Box::new(Positive::HUNDRED)),
                Some(ExpirationDate::Days(pos_or_panic!(30.0))),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("TEST".to_string()),
            );

            // Create chain build params
            let chain_params = OptionChainBuildParams::new(
                "TEST".to_string(),
                None,
                5,
                spos!(5.0),
                dec!(-0.2),
                dec!(0.0),
                pos_or_panic!(0.01),
                2,
                price_params,
                pos_or_panic!(0.2),
            );

            // Create series build params with multiple expiration dates
            let series_params = OptionSeriesBuildParams {
                chain_params,
                series: vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)],
            };

            // Build the series
            let series = OptionSeries::build_series(&series_params).unwrap();

            // Verify the series properties
            assert_eq!(series.symbol, "TEST");
            assert_eq!(series.underlying_price, Positive::HUNDRED);
            assert_eq!(series.chains.len(), 3);
            assert_eq!(series.risk_free_rate, Some(dec!(0.05)));
            assert_eq!(series.dividend_yield, spos!(0.02));

            // Verify chain expiration dates
            let expirations = series.get_expiration_dates().unwrap();
            assert_eq!(
                expirations,
                vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)]
            );
        }
    }

    mod tests_to_build_params {
        use super::*;

        #[test]
        fn test_to_build_params_normal_case() {
            let series = create_test_series();

            let result = series.to_build_params();
            assert!(result.is_ok());

            let params = result.unwrap();
            assert_eq!(params.series.len(), 3);
            assert_eq!(params.chain_params.symbol, "TEST");
        }

        #[test]
        fn test_to_build_params_empty_series() {
            let series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            let result = series.to_build_params();
            assert!(result.is_err());

            // Verify the error message
            let error = result.unwrap_err();
            assert!(error.to_string().contains("chains"));
        }
    }

    mod tests_display {
        use super::*;

        use tracing::info;

        #[test]
        fn test_display_full_series() {
            let series = create_test_series();

            let displaying = format!("{series}");
            info!("{}", displaying);
            // Verify the display string contains the important parts
            assert!(displaying.contains("symbol"));
            assert!(displaying.contains("100"));
            assert!(displaying.contains("risk_free_rate\":\"0.05"));
            assert!(displaying.contains("dividend_yield\":\"0.02\""));

            let date = get_x_days_formatted_pos(Positive::ONE).expect("one day ahead");
            let matches = date.to_string();
            assert!(displaying.contains(&matches));

            let date = get_x_days_formatted_pos(pos_or_panic!(7.0)).expect("seven days ahead");
            let matches = date.to_string();
            assert!(displaying.contains(&matches));

            let matches = "expiration_date".to_string();
            assert!(displaying.contains(&matches));
        }

        #[test]
        fn test_display_minimal_series() {
            let series = OptionSeries::new("SPY".to_string(), pos_or_panic!(450.0));

            let displaying = format!("{series}");
            // Verify the minimal display string
            assert!(displaying.contains("symbol\":\"SPY"));
            assert!(displaying.contains("underlying_price\":\"450\""));

            // Should not include optional fields
            assert!(!displaying.contains("risk_free_rate"));
            assert!(!displaying.contains("dividend_yield"));
        }
    }

    mod tests_len {
        use super::*;

        #[test]
        fn test_len_normal_case() {
            let series = create_test_series();

            assert_eq!(series.len(), 3);
            assert!(!series.is_empty());
        }

        #[test]
        fn test_len_empty_chains() {
            let series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            assert_eq!(series.len(), 0);
            assert!(series.is_empty());
        }
    }

    mod tests_serialization {
        use super::*;

        use serde_json;

        fn chain(symbol: &str, expiration: &str) -> OptionChain {
            OptionChain::new(
                symbol,
                Positive::HUNDRED,
                expiration.to_string(),
                None,
                None,
            )
        }

        #[test]
        fn test_round_trip_keeps_every_chain_and_date() {
            let mut original = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);
            original.chains.insert(
                ExpirationDate::Days(pos_or_panic!(30.0)),
                chain("TEST", "2030-01-15"),
            );
            original.chains.insert(
                ExpirationDate::Days(pos_or_panic!(60.0)),
                chain("TEST", "2030-02-14"),
            );
            original.risk_free_rate = Some(dec!(0.05));

            let json = match serde_json::to_string(&original) {
                Ok(json) => json,
                Err(error) => panic!("serialize: {error}"),
            };
            let back: OptionSeries = match serde_json::from_str(&json) {
                Ok(series) => series,
                Err(error) => panic!("deserialize: {error}"),
            };

            assert_eq!(back.symbol, original.symbol);
            assert_eq!(back.underlying_price, original.underlying_price);
            assert_eq!(back.risk_free_rate, original.risk_free_rate);
            assert_eq!(back.dividend_yield, None);
            assert_eq!(back.chains.len(), 2);
            // Compare against the keys actually written rather than
            // re-deriving them from `original`, which would race a midnight
            // between serializing and asserting.
            let value: serde_json::Value = match serde_json::from_str(&json) {
                Ok(value) => value,
                Err(error) => panic!("parse: {error}"),
            };
            let written: Vec<String> = match value.get("chains").and_then(|c| c.as_object()) {
                Some(chains) => chains.keys().cloned().collect(),
                None => panic!("no chains object in {json}"),
            };
            let read: Vec<String> = back
                .chains
                .keys()
                .map(|date| match date.get_date_string() {
                    Ok(date) => date,
                    Err(error) => panic!("date string: {error}"),
                })
                .collect();
            assert_eq!(written.len(), 2);
            assert_eq!(read, written);
            assert!(
                back.chains
                    .keys()
                    .all(|date| matches!(date, ExpirationDate::DateTime(_)))
            );
            let expirations: Vec<String> = back
                .chains
                .values()
                .map(OptionChain::get_expiration_date)
                .collect();
            assert_eq!(expirations, vec!["2030-01-15", "2030-02-14"]);
        }

        #[test]
        fn test_round_trip_keeps_absolute_dates_exactly() {
            let expiry = match NaiveDate::from_ymd_opt(2030, 1, 15)
                .and_then(|date| date.and_hms_opt(18, 30, 0))
            {
                Some(expiry) => expiry.and_utc(),
                None => panic!("valid fixture date"),
            };
            let mut original = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);
            original.chains.insert(
                ExpirationDate::DateTime(expiry),
                chain("TEST", "2030-01-15"),
            );

            let json = match serde_json::to_string(&original) {
                Ok(json) => json,
                Err(error) => panic!("serialize: {error}"),
            };
            assert!(json.contains("\"2030-01-15\":"), "{json}");
            let back: OptionSeries = match serde_json::from_str(&json) {
                Ok(series) => series,
                Err(error) => panic!("deserialize: {error}"),
            };
            let keys: Vec<&ExpirationDate> = back.chains.keys().collect();
            assert!(matches!(keys.as_slice(), [ExpirationDate::DateTime(read)] if *read == expiry));
            match serde_json::to_string(&back) {
                Ok(again) => assert_eq!(again, json),
                Err(error) => panic!("serialize again: {error}"),
            }
        }

        /// Two expiries that have both passed stay two chains, in date order,
        /// in memory and through a JSON round trip. With `expiration_date`
        /// 0.4.1 every past date compared equal to every other, so the second
        /// insert replaced the first and the series kept one chain (#825).
        #[test]
        fn test_round_trip_keeps_two_past_expiries() {
            let expiry = |year, month, day| match NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|date| date.and_hms_opt(18, 30, 0))
            {
                Some(expiry) => ExpirationDate::DateTime(expiry.and_utc()),
                None => panic!("valid fixture date"),
            };
            let mut original = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);
            original
                .chains
                .insert(expiry(2020, 6, 19), chain("TEST", "2020-06-19"));
            original
                .chains
                .insert(expiry(2020, 3, 20), chain("TEST", "2020-03-20"));
            assert_eq!(original.chains.len(), 2);

            let json = match serde_json::to_string(&original) {
                Ok(json) => json,
                Err(error) => panic!("serialize: {error}"),
            };
            assert!(json.contains("\"2020-03-20\":"), "{json}");
            assert!(json.contains("\"2020-06-19\":"), "{json}");
            let back: OptionSeries = match serde_json::from_str(&json) {
                Ok(series) => series,
                Err(error) => panic!("deserialize: {error}"),
            };

            let keys: Vec<ExpirationDate> = back.chains.keys().copied().collect();
            assert_eq!(keys, vec![expiry(2020, 3, 20), expiry(2020, 6, 19)]);
            let expirations: Vec<String> = back
                .chains
                .values()
                .map(OptionChain::get_expiration_date)
                .collect();
            assert_eq!(expirations, vec!["2020-03-20", "2020-06-19"]);
        }

        #[test]
        fn test_deserialize_rejects_a_key_in_another_date_format() {
            let chain_json = match serde_json::to_string(&chain("TEST", "2030-01-15")) {
                Ok(json) => json,
                Err(error) => panic!("serialize chain: {error}"),
            };
            for key in ["30", "20300115", "15-01-2030"] {
                let json = format!(
                    r#"{{"symbol":"TEST","underlying_price":"100","chains":{{"{key}":{chain_json}}}}}"#
                );
                match serde_json::from_str::<OptionSeries>(&json) {
                    Err(error) => assert!(error.to_string().contains("Invalid date format")),
                    Ok(_) => panic!("key {key} must be rejected"),
                }
            }
        }

        #[test]
        fn test_deserialize_rejects_a_key_that_is_not_a_date() {
            let chain_json = match serde_json::to_string(&chain("TEST", "2030-01-15")) {
                Ok(json) => json,
                Err(error) => panic!("serialize chain: {error}"),
            };
            let json = format!(
                r#"{{"symbol":"TEST","underlying_price":"100","chains":{{"not-a-date":{chain_json}}}}}"#
            );
            match serde_json::from_str::<OptionSeries>(&json) {
                Err(error) => assert!(error.to_string().contains("Invalid date format")),
                Ok(_) => panic!("a non-date chain key must be rejected"),
            }
        }

        #[test]
        fn test_serialization_minimal() {
            let original = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // Serialize
            let serialized = serde_json::to_string(&original).unwrap();

            // Verify serialized structure
            assert!(serialized.contains("\"symbol\":\"TEST\""));
            assert!(serialized.contains("\"underlying_price\":\"100\""));
            assert!(serialized.contains("\"chains\":{}"));

            // Deserialize
            let deserialized: OptionSeries = serde_json::from_str(&serialized).unwrap();

            // Verify key properties
            assert_eq!(deserialized.symbol, original.symbol);
            assert_eq!(deserialized.underlying_price, original.underlying_price);
            assert_eq!(deserialized.chains.len(), 0);
            assert_eq!(deserialized.risk_free_rate, None);
            assert_eq!(deserialized.dividend_yield, None);
        }

        #[test]
        fn test_serialization_empty_series() {
            let original = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // Serialize
            let serialized = serde_json::to_string(&original);
            assert!(
                serialized.is_ok(),
                "Serialization failed: {:?}",
                serialized.err()
            );

            // Verify serialized structure contains expected fields
            let serialized_string = serialized.unwrap();
            assert!(serialized_string.contains("\"symbol\":\"TEST\""));
            assert!(serialized_string.contains("\"underlying_price\":\"100\""));
            assert!(serialized_string.contains("\"chains\":{}"));

            // Deserialize
            let deserialized: Result<OptionSeries, _> = serde_json::from_str(&serialized_string);
            assert!(
                deserialized.is_ok(),
                "Deserialization failed: {:?}",
                deserialized.err()
            );

            let deserialized = deserialized.unwrap();

            // Verify key properties
            assert_eq!(deserialized.symbol, original.symbol);
            assert_eq!(deserialized.underlying_price, original.underlying_price);
            assert_eq!(deserialized.chains.len(), 0);
            assert_eq!(deserialized.risk_free_rate, None);
            assert_eq!(deserialized.dividend_yield, None);
        }

        #[test]
        fn test_serialization_individual_chain() {
            // This test verifies if individual OptionChain serialization works
            let chain = create_test_chain(pos_or_panic!(7.0));

            // Serialize just the chain
            let serialized = serde_json::to_string(&chain);
            assert!(
                serialized.is_ok(),
                "Chain serialization failed: {:?}",
                serialized.err()
            );

            // Deserialize the chain
            let deserialized: Result<OptionChain, _> = serde_json::from_str(&serialized.unwrap());
            assert!(
                deserialized.is_ok(),
                "Chain deserialization failed: {:?}",
                deserialized.err()
            );

            let deserialized_chain = deserialized.unwrap();
            assert_eq!(deserialized_chain.symbol, chain.symbol);
            assert_eq!(deserialized_chain.underlying_price, chain.underlying_price);
        }
    }

    mod tests_clone {
        use super::*;

        #[test]
        fn test_clone() {
            let original = create_test_series();
            let cloned = original.clone();

            // Verify key properties
            assert_eq!(cloned.symbol, original.symbol);
            assert_eq!(cloned.underlying_price, original.underlying_price);
            assert_eq!(cloned.chains.len(), original.chains.len());
            assert_eq!(cloned.risk_free_rate, original.risk_free_rate);
            assert_eq!(cloned.dividend_yield, original.dividend_yield);

            // Verify chains are properly cloned
            let original_expirations = original.get_expiration_dates().unwrap();
            let cloned_expirations = cloned.get_expiration_dates().unwrap();
            assert_eq!(cloned_expirations, original_expirations);
        }
    }
}
