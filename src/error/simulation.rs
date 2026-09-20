//! Target crate (ADR-0001 D6, roadmap M1-14): **simulation**. Owns `SimulationError`; the `Strategy`, `Chain` and `GraphError` variants are upper-layer references removed in the batch behind the 0.22.0 bump.

use crate::error::{DecimalError, OptionsError, PricingError};
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
    /// A volatility failure raised by the pricing capability this layer
    /// depends on. Boxed because `VolatilityError` is the larger enum.
    #[error(transparent)]
    Volatility(Box<crate::error::VolatilityError>),
}

impl SimulationError {
    /// Creates a new `WalkError` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the walk generation failure
    #[must_use]
    #[cold]
    #[inline(never)]
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
    #[cold]
    #[inline(never)]
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
    #[cold]
    #[inline(never)]
    pub fn step_error(reason: &str) -> Self {
        SimulationError::StepError {
            reason: reason.to_string(),
        }
    }

    /// Creates a [`SimulationError::NonFinite`] from a static call-site
    /// tag and the offending `f64` value.
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn non_finite(context: &'static str, value: f64) -> Self {
        SimulationError::NonFinite { context, value }
    }
}

impl From<crate::error::VolatilityError> for SimulationError {
    #[inline]
    fn from(err: crate::error::VolatilityError) -> Self {
        // Volatility is a pricing capability simulation depends on, so the
        // cause stays typed. Only the market wrapper it used to route
        // through was the problem, never the payload.
        SimulationError::Volatility(Box::new(err))
    }
}

/// Type alias for Results that may return a `SimulationError`.
///
/// This is a convenience type for functions that return simulation results.
pub type SimulationResult<T> = Result<T, SimulationError>;

#[cfg(test)]
mod tests_typed_causes {
    use super::*;
    use crate::error::VolatilityError;

    /// The conversion a simulation kernel uses when a volatility call fails
    /// keeps the pricing cause, so a caller can match on it instead of
    /// reading a message.
    #[test]
    fn test_a_volatility_failure_keeps_its_variant_through_the_conversion() {
        let cause = VolatilityError::NumericalFailure {
            reason: "constant_volatility: sqrt(variance) failed".to_string(),
        };

        let error = SimulationError::from(cause);

        match error {
            SimulationError::Volatility(inner) => match *inner {
                VolatilityError::NumericalFailure { reason } => {
                    assert!(reason.contains("constant_volatility"), "{reason}");
                }
                other => panic!("the volatility variant was not preserved: {other:?}"),
            },
            other => panic!("expected SimulationError::Volatility, got {other:?}"),
        }
    }

    /// A non-finite volatility keeps its structured payload rather than
    /// becoming an invalid-parameters string.
    #[test]
    fn test_a_non_finite_volatility_keeps_its_payload() {
        let cause = VolatilityError::NonFinite {
            context: "ou_process",
            value: f64::NAN,
        };

        match SimulationError::from(cause) {
            SimulationError::Volatility(inner) => match *inner {
                VolatilityError::NonFinite { context, value } => {
                    assert_eq!(context, "ou_process");
                    assert!(value.is_nan());
                }
                other => panic!("the payload was lost: {other:?}"),
            },
            other => panic!("expected SimulationError::Volatility, got {other:?}"),
        }
    }
}
