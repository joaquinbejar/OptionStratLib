/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/1/25
******************************************************************************/

use crate::Positive;
use crate::error::{GreeksError, OptionsError};
use std::error::Error;
use std::fmt;

/// Represents errors that can occur during volatility-related calculations.
///
/// This enum encapsulates various error conditions that may arise during implied volatility
/// calculations, volatility surface generation, or other volatility-related operations.
/// It provides detailed context about what went wrong, including invalid inputs, numerical
/// issues, and convergence failures.
///
/// `VolatilityError` is particularly useful for diagnosing problems in option pricing
/// models that rely on volatility parameters, such as Black-Scholes or binomial models.
#[derive(Debug)]
pub enum VolatilityError {
    /// Error indicating that a price value is invalid for volatility calculations.
    ///
    /// This variant is used when a price input doesn't meet the requirements
    /// for volatility calculation, such as being negative or outside valid bounds.
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
    ZeroVega,
    /// Error related to vega calculations or usage.
    ///
    /// This represents more general issues with vega calculations beyond just
    /// zero values.
    VegaError {
        /// A description of what went wrong with the vega calculation.
        reason: String,
    },
    /// Error related to option calculations or parameters.
    ///
    /// This represents issues with the underlying option model or parameters
    /// that prevent proper volatility calculation.
    OptionError {
        /// A description of what went wrong with the option calculation.
        reason: String,
    },
    /// Error indicating that an iterative volatility calculation failed to converge.
    ///
    /// This typically occurs in numerical methods like Newton-Raphson or bisection
    /// when trying to solve for implied volatility.
    NoConvergence {
        /// The number of iterations that were performed before giving up.
        iterations: u32,
        /// The last volatility value that was calculated before giving up.
        last_volatility: Positive,
    },
}

impl Error for VolatilityError {}

impl fmt::Display for VolatilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            VolatilityError::InvalidPrice { price, reason } => {
                write!(f, "Invalid price {price}: {reason}")
            }
            VolatilityError::InvalidTime { time, reason } => {
                write!(f, "Invalid time {time}: {reason}")
            }
            VolatilityError::ZeroVega => {
                write!(f, "Vega is zero, cannot calculate implied volatility")
            }
            VolatilityError::VegaError { reason } => {
                write!(f, "Error calculating vega: {reason}")
            }
            VolatilityError::OptionError { reason } => {
                write!(f, "Option error: {reason}")
            }
            VolatilityError::NoConvergence {
                iterations,
                last_volatility,
            } => {
                write!(
                    f,
                    "No convergence after {iterations} iterations. Last volatility: {last_volatility}"
                )
            }
        }
    }
}

impl From<GreeksError> for VolatilityError {
    fn from(error: GreeksError) -> Self {
        VolatilityError::VegaError {
            reason: error.to_string(),
        }
    }
}

impl From<OptionsError> for VolatilityError {
    fn from(error: OptionsError) -> Self {
        VolatilityError::OptionError {
            reason: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests_volatility_errors {
    use super::*;
    use crate::error::greeks::InputErrorKind;
    use crate::error::{GreeksError, OptionsError};
    use crate::pos;

    #[test]
    fn test_invalid_price_error() {
        let error = VolatilityError::InvalidPrice {
            price: pos!(0.0),
            reason: "Price cannot be zero".to_string(),
        };

        assert_eq!(error.to_string(), "Invalid price 0: Price cannot be zero");
    }

    #[test]
    fn test_invalid_time_error() {
        let error = VolatilityError::InvalidTime {
            time: pos!(0.0),
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
            last_volatility: pos!(0.5),
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
            VolatilityError::VegaError { reason } => {
                assert!(reason.contains("Volatility cannot be zero"));
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_from_options_error() {
        let greeks_error = OptionsError::OtherError {
            reason: "Invalid option parameters".to_string(),
        };

        let implied_vol_error: VolatilityError = greeks_error.into();

        match implied_vol_error {
            VolatilityError::OptionError { reason } => {
                assert!(reason.contains("Invalid option parameters"));
            }
            _ => panic!("Wrong error variant"),
        }
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
