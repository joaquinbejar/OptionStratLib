use crate::error::DecimalError;
use crate::prelude::GraphError;
use thiserror::Error;

/// Error type for simulation operations.
///
/// This enum represents the various errors that can occur during simulation operations,
/// such as random walks, Monte Carlo simulations, and other stochastic processes.
#[derive(Error, Debug)]
pub enum SimulationError {
    /// Error during walk generation.
    #[error("Walk generation failed: {reason}")]
    WalkError {
        /// Detailed reason for the walk generation failure
        reason: String,
    },

    /// Error due to invalid simulation parameters.
    #[error("Invalid simulation parameters: {reason}")]
    InvalidParameters {
        /// Detailed reason for the invalid parameters
        reason: String,
    },

    /// Error during step calculation.
    #[error("Step calculation failed: {reason}")]
    StepError {
        /// Detailed reason for the step calculation failure
        reason: String,
    },

    /// Generic simulation error.
    #[error("Simulation error: {reason}")]
    OtherError {
        /// Detailed reason for the error
        reason: String,
    },

    /// Error during graph generation.
    #[error(transparent)]
    GraphError(#[from] GraphError),
}

impl SimulationError {
    /// Creates a new `WalkError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the walk generation failure
    pub fn walk_error(reason: &str) -> Self {
        SimulationError::WalkError {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `InvalidParameters` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the invalid parameters
    pub fn invalid_parameters(reason: &str) -> Self {
        SimulationError::InvalidParameters {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `StepError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the step calculation failure
    pub fn step_error(reason: &str) -> Self {
        SimulationError::StepError {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `OtherError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the error
    pub fn other(reason: &str) -> Self {
        SimulationError::OtherError {
            reason: reason.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for SimulationError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SimulationError::OtherError {
            reason: err.to_string(),
        }
    }
}

impl From<String> for SimulationError {
    fn from(s: String) -> Self {
        SimulationError::OtherError { reason: s }
    }
}

impl From<&str> for SimulationError {
    fn from(s: &str) -> Self {
        SimulationError::OtherError {
            reason: s.to_string(),
        }
    }
}

impl From<DecimalError> for SimulationError {
    fn from(err: DecimalError) -> Self {
        SimulationError::OtherError {
            reason: err.to_string(),
        }
    }
}

impl From<crate::error::OptionsError> for SimulationError {
    fn from(err: crate::error::OptionsError) -> Self {
        SimulationError::OtherError {
            reason: err.to_string(),
        }
    }
}

impl From<crate::error::PricingError> for SimulationError {
    fn from(err: crate::error::PricingError) -> Self {
        SimulationError::OtherError {
            reason: err.to_string(),
        }
    }
}

/// Type alias for Results that may return a `SimulationError`.
///
/// This is a convenience type for functions that return simulation results.
pub type SimulationResult<T> = Result<T, SimulationError>;
