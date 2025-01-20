/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/1/25
******************************************************************************/

use crate::error::{GreeksError, OptionsError};
use crate::Positive;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ImpliedVolatilityError {
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

impl Error for ImpliedVolatilityError {}

impl fmt::Display for ImpliedVolatilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            ImpliedVolatilityError::InvalidPrice { price, reason } => {
                write!(f, "Invalid price {}: {}", price, reason)
            }
            ImpliedVolatilityError::InvalidTime { time, reason } => {
                write!(f, "Invalid time {}: {}", time, reason)
            }
            ImpliedVolatilityError::ZeroVega => {
                write!(f, "Vega is zero, cannot calculate implied volatility")
            }
            ImpliedVolatilityError::VegaError { reason } => {
                write!(f, "Error calculating vega: {}", reason)
            }
            ImpliedVolatilityError::OptionError { reason } => {
                write!(f, "Option error: {}", reason)
            }
            ImpliedVolatilityError::NoConvergence {
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

impl From<GreeksError> for ImpliedVolatilityError {
    fn from(error: GreeksError) -> Self {
        ImpliedVolatilityError::VegaError {
            reason: error.to_string(),
        }
    }
}

impl From<OptionsError> for ImpliedVolatilityError {
    fn from(error: OptionsError) -> Self {
        ImpliedVolatilityError::OptionError {
            reason: error.to_string(),
        }
    }
}


#[cfg(test)]
mod tests_volatility_errors {
    use super::*;
    use crate::pos;
    use crate::error::{GreeksError, OptionsError};
    use crate::error::greeks::InputErrorKind;

    #[test]
    fn test_invalid_price_error() {
        let error = ImpliedVolatilityError::InvalidPrice {
            price: pos!(0.0),
            reason: "Price cannot be zero".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Invalid price 0: Price cannot be zero"
        );
    }

    #[test]
    fn test_invalid_time_error() {
        let error = ImpliedVolatilityError::InvalidTime {
            time: pos!(0.0),
            reason: "Time cannot be zero".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Invalid time 0: Time cannot be zero"
        );
    }

    #[test]
    fn test_zero_vega_error() {
        let error = ImpliedVolatilityError::ZeroVega;

        assert_eq!(
            error.to_string(),
            "Vega is zero, cannot calculate implied volatility"
        );
    }

    #[test]
    fn test_vega_error() {
        let error = ImpliedVolatilityError::VegaError {
            reason: "Failed to calculate vega".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Error calculating vega: Failed to calculate vega"
        );
    }

    #[test]
    fn test_option_error() {
        let error = ImpliedVolatilityError::OptionError {
            reason: "Invalid option parameters".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Option error: Invalid option parameters"
        );
    }

    #[test]
    fn test_no_convergence_error() {
        let error = ImpliedVolatilityError::NoConvergence {
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
        let greeks_error = GreeksError::InputError(
            InputErrorKind::InvalidVolatility {
                value: 0.0,
                reason: "Volatility cannot be zero".to_string(),
            }
        );

        let implied_vol_error: ImpliedVolatilityError = greeks_error.into();

        match implied_vol_error {
            ImpliedVolatilityError::VegaError { reason } => {
                assert!(reason.contains("Volatility cannot be zero"));
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_from_options_error() {
        let greeks_error = OptionsError::OtherError{
            reason: "Invalid option parameters".to_string(),
        };

        let implied_vol_error: ImpliedVolatilityError = greeks_error.into();

        match implied_vol_error {
            ImpliedVolatilityError::OptionError { reason } => {
                assert!(reason.contains("Invalid option parameters"));
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_error_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ImpliedVolatilityError>();
    }

    #[test]
    fn test_error_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ImpliedVolatilityError>();
    }
}