use crate::ExpirationDate;
use crate::chains::OptionChain;
use crate::error::ChainError;
use crate::series::params::OptionSeriesBuildParams;
use crate::utils::Len;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};
use std::collections::BTreeMap;
use std::fmt;
use positive::Positive;
use utoipa::ToSchema;

/// Represents a series of option chains for an underlying asset,
/// providing detailed information about its options market and related financial data.
#[derive(DebugPretty, DisplaySimple, Clone, ToSchema)]
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
    pub fn build_series(params: &OptionSeriesBuildParams) -> Self {
        let mut params = params.clone();
        let mut chains: BTreeMap<ExpirationDate, OptionChain> = BTreeMap::new();
        for series in params.series.clone().into_iter() {
            let expiration_date: ExpirationDate = ExpirationDate::Days(series);
            params.chain_params.price_params.expiration_date = Some(expiration_date);
            params.chain_params.strike_interval = None;
            let mut chain: OptionChain = OptionChain::build_chain(&params.chain_params);
            chain.update_expiration_date(expiration_date.get_date_string().unwrap());
            chains.insert(expiration_date, chain);
        }
        let price_params = params.chain_params.price_params.clone();
        let underlying_price = *price_params.underlying_price.unwrap();
        Self {
            symbol: params.chain_params.symbol.clone(),
            underlying_price,
            chains,
            risk_free_rate: price_params.risk_free_rate,
            dividend_yield: price_params.dividend_yield,
        }
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
                return Err("No chains found".into());
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

        // Serialize chains as a map of string dates to OptionChain
        let chains_map: BTreeMap<String, &OptionChain> = self
            .chains
            .iter()
            .map(|(date, chain)| (date.get_date_string().unwrap(), chain))
            .collect();
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

                // Convert string dates back to ExpirationDate objects
                let mut chains = BTreeMap::new();
                for (date_str, chain) in string_chains {
                    let expiration_date = ExpirationDate::from_string_to_days(&date_str)
                        .map_err(|e| de::Error::custom(format!("Invalid date format: {e}")))?;
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
    use positive::{Positive, pos_or_panic, spos};

    use crate::chains::OptionChain;
    use crate::series::params::OptionSeriesBuildParams;
    use crate::utils::Len;
    use crate::utils::time::get_x_days_formatted_pos;

    use rust_decimal_macros::dec;

    // Helper function to create a simple OptionChain for testing
    fn create_test_chain(expiration_days: Positive) -> OptionChain {
        let date = get_x_days_formatted_pos(expiration_days);
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
            let series = OptionSeries::build_series(&series_params);

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
            assert!(error.to_string().contains("No chains found"));
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
            assert!(displaying.contains("dividend_yield\":0.02"));

            let date = get_x_days_formatted_pos(Positive::ONE);
            let matches = date.to_string();
            assert!(displaying.contains(&matches));

            let date = get_x_days_formatted_pos(pos_or_panic!(7.0));
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
            assert!(displaying.contains("underlying_price\":450"));

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

        #[test]
        fn test_serialization_minimal() {
            let original = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);

            // Serialize
            let serialized = serde_json::to_string(&original).unwrap();

            // Verify serialized structure
            assert!(serialized.contains("\"symbol\":\"TEST\""));
            assert!(serialized.contains("\"underlying_price\":100"));
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
            assert!(serialized_string.contains("\"underlying_price\":100"));
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
