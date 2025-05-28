use crate::Positive;
use crate::chains::OptionChainBuildParams;
use serde::{Deserialize, Serialize};
use std::fmt;

/// `OptionSeriesBuildParams` is a struct that represents the parameters required to
/// generate multiple option chains (series) along with their respective expiration details.
///
/// # Fields
///
/// * `chain_params` (`OptionChainBuildParams`) - Contains the configuration and parameters needed
///   for building the option chains. This might include parameters like underlying asset details,
///   strike prices, and other relevant metadata. This field is private but accessible
///   within the same module.
///
/// * `series` (`Vec<Positive>`) - A collection of positive values indicating the number
///   of option chains to build and their associated days to expiration. Each value in the vector
///   specifies a particular series to generate with its distinct expiration timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionSeriesBuildParams {
    /// Parameters for building option chains
    pub(crate) chain_params: OptionChainBuildParams,

    /// Number of options chain to build and its days to expiration
    pub(crate) series: Vec<Positive>,
}

impl OptionSeriesBuildParams {
    /// Constructs a new instance of the `Self` type.
    ///
    /// # Parameters
    /// - `chain_params`: An instance of `OptionChainBuildParams` representing the parameters
    ///   for building an option chain. It can be `Some` value with valid
    ///   parameters or `None`.
    /// - `series`: A vector of `Positive` values representing the series data used for initialization.
    ///
    /// # Returns
    /// - A new `Self` instance initialized with the provided `chain_params` and `series`.
    ///
    pub fn new(chain_params: OptionChainBuildParams, series: Vec<Positive>) -> Self {
        Self {
            chain_params,
            series,
        }
    }

    /// Sets the underlying price for the chain parameters in the current context.
    ///
    /// This method updates the underlying price by delegating it to the `set_underlying_price`
    /// method of the `chain_params` object. The `price` provided must be of type `Positive`,
    /// which ensures the value is valid and strictly positive.
    ///
    /// # Parameters
    /// - `price`: A reference to a `Positive` value representing the new underlying price
    ///   to be set. The `Positive` type guarantees that only valid positive values are accepted.
    ///
    /// # Panics
    /// This function does not explicitly panic, but any implementation detail within
    /// the `set_underlying_price` method of `chain_params` that could panic needs to
    /// be considered by the caller.
    ///
    pub fn set_underlying_price(&mut self, price: &Positive) {
        let price = Some(Box::new(*price));
        self.chain_params.set_underlying_price(price);
    }

    /// Sets the implied volatility for the chain parameters.
    ///
    /// # Parameters
    /// - `volatility`: An `Option` containing a `Positive` value representing the implied volatility.
    ///   If `Some`, updates the implied volatility; if `None`, clears the previously set value.
    ///
    /// # Description
    /// This function allows updating or clearing the implied volatility value used by the `chain_params`.
    /// Implied volatility is often used in financial calculations and represents the market's forecast
    /// of a likely movement in a security's price. This method delegates the update to the
    /// `set_implied_volatility` method of the `chain_params` field.
    ///
    /// # Notes
    /// - `Positive` is a type that enforces the input to always be positive.
    /// - Use `Some(value)` to set a new positive implied volatility.
    /// - Pass `None` to clear the existing implied volatility.
    ///
    pub fn set_implied_volatility(&mut self, volatility: Positive) {
        self.chain_params.set_implied_volatility(volatility);
    }
}

impl fmt::Display for OptionSeriesBuildParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let series = self
            .series
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "chain_params: {} , series: {}",
            self.chain_params, series
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::utils::OptionDataPriceParams;
    use crate::utils::time::get_tomorrow_formatted;
    use crate::{ExpirationDate, pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_display_empty_series() {
        let tomorrow = get_tomorrow_formatted();
        let expiration = ExpirationDate::from_string_to_days(&tomorrow).unwrap();
        let expiration_as_string = expiration.to_string();
        let price_params = OptionDataPriceParams::new(
            Some(Box::new(pos!(100.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("AAPL".to_string()),
        );
        let chain_params = OptionChainBuildParams::new(
            "TEST".to_string(),
            spos!(1000.0),
            10,
            spos!(5.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            price_params,
            pos!(0.2)
        );

        let params = OptionSeriesBuildParams {
            chain_params,
            series: vec![],
        };

        assert_eq!(
            params.to_string(),
            format!(
                "chain_params: Option Chain Build Parameters:\n  Symbol: TEST\n  Volume: 1000\n  Chain Size: 10\n  Strike Interval: 5\n  Skew Factor: 0.1\n  Spread: 0.02\n  Decimal Places: 2\n  Price Parameters:\n    Underlying Price: 100\n    Expiration Date: {}\n    Implied Volatility: 20%\n    Risk-Free Rate: 5.00%\n    Dividend Yield: 2.00%\n , series: ",
                expiration_as_string
            )
        );
    }
}
