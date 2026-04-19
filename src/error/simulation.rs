use crate::error::{ChainError, DecimalError, OptionsError, PricingError, StrategyError};
use crate::prelude::GraphError;
use expiration_date::error::ExpirationDateError;
use positive::Positive;
use rust_decimal::Decimal;
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

    /// The walk type in the parameters does not match the generator being invoked.
    ///
    /// Raised from inside each `WalkTypeAble` implementation when the supplied
    /// `walk_type` discriminator is not the one expected by the method.
    #[error("invalid walk type: expected {expected}")]
    InvalidWalkType {
        /// Human-readable description of the expected walk type, e.g. `"Brownian"`.
        expected: &'static str,
    },

    /// Autocorrelation parameter is outside the required `[-1, 1]` interval.
    #[error("autocorrelation {value} must lie in [-1, 1]")]
    InvalidAutocorrelation {
        /// The offending autocorrelation value.
        value: Decimal,
    },

    /// GARCH stationarity constraint `alpha + beta < 1` violated.
    #[error("GARCH stationarity violated: alpha ({alpha}) + beta ({beta}) must be < 1")]
    GarchStationarity {
        /// The GARCH alpha coefficient.
        alpha: Positive,
        /// The GARCH beta coefficient.
        beta: Positive,
    },

    /// Heston correlation `rho` is outside the valid `[-1, 1]` interval.
    #[error("Heston correlation rho {rho} must lie in [-1, 1]")]
    InvalidCorrelation {
        /// The offending correlation value.
        rho: Decimal,
    },

    /// Not enough historical price observations to generate the requested walk.
    #[error("historical walk requires at least {required} observations, found {found}")]
    InsufficientHistoricalData {
        /// Minimum number of observations required.
        required: usize,
        /// Number of observations actually available.
        found: usize,
    },

    /// Failed to convert the x-axis step index into a `Decimal`.
    #[error("cannot convert x-axis step index to Decimal")]
    IndexConversion,

    /// The simulated expiration has already been reached, no further steps
    /// can be generated.
    #[error("cannot generate next step: expiration date already reached")]
    ExpirationReached,

    /// Decimal arithmetic error surfaced from pricing or Greek calculations.
    #[error(transparent)]
    Decimal(#[from] DecimalError),

    /// Options domain error surfaced during simulation.
    #[error(transparent)]
    Options(#[from] OptionsError),

    /// Pricing error surfaced during simulation.
    #[error(transparent)]
    Pricing(#[from] PricingError),

    /// Expiration-date conversion error.
    #[error(transparent)]
    ExpirationDate(#[from] ExpirationDateError),

    /// Strategy-layer error surfaced during simulation.
    #[error(transparent)]
    Strategy(Box<StrategyError>),

    /// Chain domain error surfaced during simulation.
    #[error(transparent)]
    Chain(Box<ChainError>),

    /// Error during graph generation.
    #[error(transparent)]
    GraphError(#[from] GraphError),

    /// Positive value errors
    #[error(transparent)]
    PositiveError(#[from] positive::PositiveError),

    /// A simulation kernel produced a non-finite `f64` value (`NaN` /
    /// `±∞`) at an `f64` → `Decimal` boundary.
    ///
    /// Emitted by Brownian / geometric Brownian motion, Heston,
    /// telegraph, and general random-walk path generators whenever
    /// an intermediate `f64` would otherwise be silently cast into
    /// `Decimal::ZERO`. `context` is a static call-site tag
    /// following the same convention as
    /// [`crate::error::DecimalError::Overflow`].
    #[error("simulation non-finite {context}: {value}")]
    NonFinite {
        /// Static tag identifying the kernel and step that produced
        /// the non-finite value.
        context: &'static str,
        /// The offending `f64` value (`NaN`, `+∞`, or `-∞`).
        value: f64,
    },
}

impl SimulationError {
    /// Creates a new `WalkError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the walk generation failure
    #[must_use]
    #[inline]
    pub fn walk_error(reason: &str) -> Self {
        SimulationError::WalkError {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `InvalidParameters` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the invalid parameters
    #[must_use]
    #[inline]
    pub fn invalid_parameters(reason: &str) -> Self {
        SimulationError::InvalidParameters {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `StepError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the step calculation failure
    #[must_use]
    #[inline]
    pub fn step_error(reason: &str) -> Self {
        SimulationError::StepError {
            reason: reason.to_string(),
        }
    }

    /// Creates a [`SimulationError::NonFinite`] from a static call-site
    /// tag and the offending `f64` value.
    #[must_use]
    #[inline]
    #[cold]
    pub fn non_finite(context: &'static str, value: f64) -> Self {
        SimulationError::NonFinite { context, value }
    }
}

impl From<StrategyError> for SimulationError {
    #[inline]
    fn from(err: StrategyError) -> Self {
        SimulationError::Strategy(Box::new(err))
    }
}

impl From<ChainError> for SimulationError {
    #[inline]
    fn from(err: ChainError) -> Self {
        SimulationError::Chain(Box::new(err))
    }
}

/// Type alias for Results that may return a `SimulationError`.
///
/// This is a convenience type for functions that return simulation results.
pub type SimulationResult<T> = Result<T, SimulationError>;
