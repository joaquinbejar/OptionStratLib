use std::error::Error;
use std::fmt;

/// Error type for option pricing operations.
///
/// This enum represents the various errors that can occur during option pricing,
/// providing domain-specific error handling for different pricing scenarios.
#[derive(Debug)]
pub enum PricingError {
    /// Error from a specific pricing method (e.g., Black-Scholes, Binomial).
    MethodError {
        /// Name of the pricing method that failed
        method: String,
        /// Detailed reason for the failure
        reason: String,
    },
    /// Error during Monte Carlo simulation.
    SimulationError {
        /// Detailed reason for the simulation failure
        reason: String,
    },
    /// Error due to invalid pricing engine configuration.
    InvalidEngine {
        /// Detailed reason for the invalid engine
        reason: String,
    },
    /// Generic pricing error.
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

impl fmt::Display for PricingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PricingError::MethodError { method, reason } => {
                write!(f, "Pricing method '{method}' failed: {reason}")
            }
            PricingError::SimulationError { reason } => {
                write!(f, "Pricing simulation failed: {reason}")
            }
            PricingError::InvalidEngine { reason } => {
                write!(f, "Invalid pricing engine: {reason}")
            }
            PricingError::OtherError { reason } => write!(f, "Pricing error: {reason}"),
        }
    }
}

impl Error for PricingError {}

impl From<Box<dyn Error>> for PricingError {
    fn from(err: Box<dyn Error>) -> Self {
        PricingError::OtherError {
            reason: err.to_string(),
        }
    }
}

/// Type alias for Results that may return a `PricingError`.
///
/// This is a convenience type for functions that return pricing results.
pub type PricingResult<T> = Result<T, PricingError>;
