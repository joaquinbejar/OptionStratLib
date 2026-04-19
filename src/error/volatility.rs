/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/1/25
******************************************************************************/

use crate::error::{GreeksError, OptionsError};
use positive::Positive;
use thiserror::Error;

/// Represents errors that can occur during volatility-related calculations.
///
/// This enum encapsulates various error conditions that may arise during implied volatility
/// calculations, volatility surface generation, or other volatility-related operations.
/// It provides detailed context about what went wrong, including invalid inputs, numerical
/// issues, and convergence failures.
///
/// `VolatilityError` is particularly useful for diagnosing problems in option pricing
/// models that rely on volatility parameters, such as Black-Scholes or binomial models.
#[derive(Error, Debug)]
pub enum VolatilityError {
    /// Error indicating that a price value is invalid for volatility calculations.
    ///
    /// This variant is used when a price input doesn't meet the requirements
    /// for volatility calculation, such as being negative or outside valid bounds.
    #[error("Invalid price {price}: {reason}")]
    InvalidPrice {
        /// The invalid price value that caused the error.
        price: Positive,
        /// A description explaining why the price is invalid.
        reason: String,
    },

    /// Error indicating that a time value is invalid for volatility calculations.
    ///
    /// This occurs when time parameters (such as time to expiration) are invalid
    /// for volatility calculations, for example being negative or too large.
    #[error("Invalid time {time}: {reason}")]
    InvalidTime {
        /// The invalid time value that caused the error.
        time: Positive,
        /// A description explaining why the time value is invalid.
        reason: String,
    },

    /// Error indicating that the vega value is zero.
    ///
    /// This occurs when attempting to calculate implied volatility using the
    /// Newton-Raphson method and vega is zero, making it impossible to converge.
    #[error("Vega is zero, cannot calculate implied volatility")]
    ZeroVega,

    /// Error related to Greeks calculations.
    #[error(transparent)]
    Greeks(#[from] GreeksError),

    /// Error related to option calculations.
    #[error(transparent)]
    Options(#[from] OptionsError),

    /// Error related to vega calculations or usage.
    ///
    /// This represents more general issues with vega calculations beyond just
    /// zero values.
    #[error("Error calculating vega: {reason}")]
    VegaError {
        /// A description of what went wrong with the vega calculation.
        reason: String,
    },

    /// Error related to option calculations or parameters.
    ///
    /// This represents issues with the underlying option model or parameters
    /// that prevent proper volatility calculation.
    #[error("Option error: {reason}")]
    OptionError {
        /// A description of what went wrong with the option calculation.
        reason: String,
    },

    /// Error indicating that an iterative volatility calculation failed to converge.
    ///
    /// This typically occurs in numerical methods like Newton-Raphson or bisection
    /// when trying to solve for implied volatility.
    #[error("No convergence after {iterations} iterations. Last volatility: {last_volatility}")]
    NoConvergence {
        /// The number of iterations that were performed before giving up.
        iterations: u32,
        /// The last volatility value that was calculated before giving up.
        last_volatility: Positive,
    },

    /// No implied-volatility candidate produced a price match after the grid search.
    #[error("implied volatility not found within search grid")]
    IvNotFound,

    /// None of the generated volatility samples produced a valid price.
    #[error("no valid volatility sample produced a finite price")]
    NoValidVolatility,

    /// A numerical precision or representation failure inside a Heston-style
    /// volatility simulation kernel (e.g. `sqrt` overflow, `f64`/`Decimal` bridge).
    #[error("volatility simulation numerical failure: {reason}")]
    NumericalFailure {
        /// Human-readable description of which numerical step failed.
        reason: String,
    },

    /// The ATM implied volatility is unavailable; the boxed inner source
    /// describes the failing lookup.
    #[error("ATM implied volatility is not available: {source}")]
    AtmIvUnavailable {
        /// Underlying error that explains why the ATM IV could not be retrieved.
        #[source]
        source: Box<VolatilityError>,
    },

    /// Positive value errors
    #[error(transparent)]
    PositiveError(#[from] positive::PositiveError),
}

#[cfg(test)]
mod tests_volatility_errors {
    use super::*;
    use crate::error::greeks::InputErrorKind;
    use crate::error::{GreeksError, OptionsError};
    use positive::pos_or_panic;

    #[test]
    fn test_invalid_price_error() {
        let error = VolatilityError::InvalidPrice {
            price: Positive::ZERO,
            reason: "Price cannot be zero".to_string(),
        };

        assert_eq!(error.to_string(), "Invalid price 0: Price cannot be zero");
    }

    #[test]
    fn test_invalid_time_error() {
        let error = VolatilityError::InvalidTime {
            time: Positive::ZERO,
            reason: "Time cannot be zero".to_string(),
        };

        assert_eq!(error.to_string(), "Invalid time 0: Time cannot be zero");
    }

    #[test]
    fn test_zero_vega_error() {
        let error = VolatilityError::ZeroVega;

        assert_eq!(
            error.to_string(),
            "Vega is zero, cannot calculate implied volatility"
        );
    }

    #[test]
    fn test_vega_error() {
        let error = VolatilityError::VegaError {
            reason: "Failed to calculate vega".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Error calculating vega: Failed to calculate vega"
        );
    }

    #[test]
    fn test_option_error() {
        let error = VolatilityError::OptionError {
            reason: "Invalid option parameters".to_string(),
        };

        assert_eq!(error.to_string(), "Option error: Invalid option parameters");
    }

    #[test]
    fn test_no_convergence_error() {
        let error = VolatilityError::NoConvergence {
            iterations: 100,
            last_volatility: pos_or_panic!(0.5),
        };

        assert_eq!(
            error.to_string(),
            "No convergence after 100 iterations. Last volatility: 0.5"
        );
    }

    #[test]
    fn test_from_greeks_error() {
        let greeks_error = GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value: 0.0,
            reason: "Volatility cannot be zero".to_string(),
        });

        let implied_vol_error: VolatilityError = greeks_error.into();

        match implied_vol_error {
            VolatilityError::Greeks(_) => {
                // Conversion successful
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_from_options_error() {
        let options_error = OptionsError::validation_error("strike", "Invalid option parameters");

        let implied_vol_error: VolatilityError = options_error.into();

        match implied_vol_error {
            VolatilityError::Options(_) => {
                // Conversion successful
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_iv_not_found() {
        let error = VolatilityError::IvNotFound;
        assert_eq!(
            error.to_string(),
            "implied volatility not found within search grid"
        );
    }

    #[test]
    fn test_no_valid_volatility() {
        let error = VolatilityError::NoValidVolatility;
        assert_eq!(
            error.to_string(),
            "no valid volatility sample produced a finite price"
        );
    }

    #[test]
    fn test_atm_iv_unavailable() {
        let error = VolatilityError::AtmIvUnavailable {
            source: Box::new(VolatilityError::IvNotFound),
        };
        assert!(
            error
                .to_string()
                .contains("ATM implied volatility is not available")
        );
    }

    #[test]
    fn test_error_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<VolatilityError>();
    }

    #[test]
    fn test_error_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<VolatilityError>();
    }
}
