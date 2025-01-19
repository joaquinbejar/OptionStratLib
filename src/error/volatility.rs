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
