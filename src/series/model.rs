use crate::chains::OptionChain;
use crate::series::params::OptionSeriesBuildParams;
use crate::utils::Len;
use crate::{ExpirationDate, Positive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

/// Represents a series of option chains for an underlying asset,
/// providing detailed information about its options market and related financial data.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn get_expiration_dates(&self) -> Result<Vec<Positive>, Box<dyn Error>> {
        let keys: Result<Vec<Positive>, Box<dyn Error>> =
            self.chains.keys().map(|e| e.get_days()).collect();

        keys
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
    ///   validation of `params` prior to calling this method.
    /// - The use of a `BTreeMap` ensures that the resulting chains are sorted based on the expiration dates.
    /// ```
    pub fn build_series(params: &OptionSeriesBuildParams) -> Self {
        let mut params = params.clone();
        let mut chains: BTreeMap<ExpirationDate, OptionChain> = BTreeMap::new();
        for series in params.series.clone().into_iter() {
            let expiration_date: ExpirationDate = ExpirationDate::Days(series);
            params.chain_params.price_params.expiration_date = expiration_date;
            params.chain_params.strike_interval = None;
            let mut chain: OptionChain = OptionChain::build_chain(&params.chain_params);

            chain.update_expiration_date(expiration_date.get_date_string().unwrap());
            chains.insert(expiration_date, chain);
        }

        Self {
            symbol: params.chain_params.symbol.clone(),
            underlying_price: params.chain_params.price_params.underlying_price,
            chains,
            risk_free_rate: Some(params.chain_params.price_params.risk_free_rate),
            dividend_yield: Some(params.chain_params.price_params.dividend_yield),
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
    pub fn to_build_params(&self) -> Result<OptionSeriesBuildParams, Box<dyn Error>> {
        let chain_params = self.chains.first_key_value();
        let series = self.get_expiration_dates()?;
        let chain_params = match chain_params {
            Some((_, option_chain)) => option_chain.to_build_params()?,
            None => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No chains found",
                )));
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

impl fmt::Display for OptionSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chains: String = self
            .chains
            .iter()
            .map(|(e, o)| format!("{}:\n{}", e, o))
            .collect();

        let risk_free_rate = match &self.risk_free_rate {
            Some(r) => format!(" risk_free_rate: {}", r),
            None => "".to_string(),
        };
        let dividend_yield = match &self.dividend_yield {
            Some(d) => format!(" dividend_yield: {}", d),
            None => "".to_string(),
        };

        write!(
            f,
            "OptionSeries {{ symbol: {}, underlying_price: {}{}{}\n{} }}",
            self.symbol, self.underlying_price, risk_free_rate, dividend_yield, chains
        )
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
