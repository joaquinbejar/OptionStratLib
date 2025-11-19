use crate::error::{DecimalError, GreeksError, OptionsError, PositionError};
use thiserror::Error;

/// Error type for option pricing operations.
///
/// This enum represents the various errors that can occur during option pricing,
/// providing domain-specific error handling for different pricing scenarios.
#[derive(Error, Debug)]
pub enum PricingError {
    /// Error from a specific pricing method (e.g., Black-Scholes, Binomial).
    #[error("Pricing method '{method}' failed: {reason}")]
    MethodError {
        /// Name of the pricing method that failed
        method: String,
        /// Detailed reason for the failure
        reason: String,
    },

    /// Error during Monte Carlo simulation.
    #[error("Pricing simulation failed: {reason}")]
    SimulationError {
        /// Detailed reason for the simulation failure
        reason: String,
    },

    /// Error due to invalid pricing engine configuration.
    #[error("Invalid pricing engine: {reason}")]
    InvalidEngine {
        /// Detailed reason for the invalid engine
        reason: String,
    },

    /// Error from Greeks calculations.
    #[error(transparent)]
    Greeks(#[from] GreeksError),

    /// Error from Options operations.
    #[error(transparent)]
    Options(#[from] OptionsError),

    /// Error from Position operations.
    #[error(transparent)]
    Position(#[from] PositionError),

    /// Error from Decimal operations.
    #[error(transparent)]
    Decimal(#[from] DecimalError),

    /// Generic pricing error.
    #[error("Pricing error: {reason}")]
    OtherError {
        /// Detailed reason for the error
        reason: String,
    },
}

impl PricingError {
    /// Creates a new `MethodError` variant.
    ///
    /// # Arguments
    /// * `method` - Name of the pricing method that failed
    /// * `reason` - Detailed reason for the failure
    pub fn method_error(method: &str, reason: &str) -> Self {
        PricingError::MethodError {
            method: method.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `SimulationError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the simulation failure
    pub fn simulation_error(reason: &str) -> Self {
        PricingError::SimulationError {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `InvalidEngine` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the invalid engine
    pub fn invalid_engine(reason: &str) -> Self {
        PricingError::InvalidEngine {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `OtherError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the error
    pub fn other(reason: &str) -> Self {
        PricingError::OtherError {
            reason: reason.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for PricingError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        PricingError::OtherError {
            reason: err.to_string(),
        }
    }
}

impl From<String> for PricingError {
    fn from(s: String) -> Self {
        PricingError::OtherError { reason: s }
    }
}

impl From<&str> for PricingError {
    fn from(s: &str) -> Self {
        PricingError::OtherError {
            reason: s.to_string(),
        }
    }
}

/// Type alias for Results that may return a `PricingError`.
///
/// This is a convenience type for functions that return pricing results.
pub type PricingResult<T> = Result<T, PricingError>;
