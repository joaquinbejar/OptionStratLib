/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/1/25
******************************************************************************/

use crate::Positive;
use crate::error::{GreeksError, OptionsError};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum VolatilityError {
    InvalidPrice {
        price: Positive,
        reason: String,
    },
    InvalidTime {
        time: Positive,
        reason: String,
    },
    ZeroVega,
    VegaError {
        reason: String,
    },
    OptionError {
        reason: String,
    },
    NoConvergence {
        iterations: u32,
        last_volatility: Positive,
    },
}

impl Error for VolatilityError {}

impl fmt::Display for VolatilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            VolatilityError::InvalidPrice { price, reason } => {
                write!(f, "Invalid price {}: {}", price, reason)
            }
            VolatilityError::InvalidTime { time, reason } => {
                write!(f, "Invalid time {}: {}", time, reason)
            }
            VolatilityError::ZeroVega => {
                write!(f, "Vega is zero, cannot calculate implied volatility")
            }
            VolatilityError::VegaError { reason } => {
                write!(f, "Error calculating vega: {}", reason)
            }
            VolatilityError::OptionError { reason } => {
                write!(f, "Option error: {}", reason)
            }
            VolatilityError::NoConvergence {
                iterations,
                last_volatility,
            } => {
                write!(
                    f,
                    "No convergence after {} iterations. Last volatility: {}",
                    iterations, last_volatility
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
