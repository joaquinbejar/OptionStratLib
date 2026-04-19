use crate::error::{DecimalError, GreeksError, OptionsError, PositionError};
use expiration_date::error::ExpirationDateError;
use positive::PositiveError;
use rust_decimal::Decimal;
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

    /// Expiration-date conversion error surfaced during pricing.
    #[error(transparent)]
    ExpirationDate(#[from] ExpirationDateError),

    /// A delta adjustment was requested on a strategy that does not support it.
    #[error("delta adjustments are not applicable to single-leg {strategy} strategy")]
    DeltaAdjustmentNotApplicable {
        /// Name of the strategy, e.g. `"LongCall"`, `"ShortPut"`.
        strategy: &'static str,
    },

    /// A required intermediate value on a binomial lattice or pricing kernel
    /// was missing (typically a node that should have been populated by an
    /// earlier induction step).
    #[error("binomial pricing node `{node}` is missing")]
    BinomialNodeMissing {
        /// Identifier of the missing node, e.g. `"S_k"`, `"b"`.
        node: &'static str,
    },

    /// A square-root computation inside a pricing kernel failed (the operand
    /// was negative or not representable — e.g. a negative discriminant in a
    /// closed-form solver).
    #[error("pricing sqrt failed for value {value}")]
    SqrtFailure {
        /// The value for which the square root could not be computed.
        ///
        /// Stored as `Decimal` so negative operands are representable.
        value: Decimal,
    },

    /// Error from Positive operations.
    #[error(transparent)]
    Positive(#[from] PositiveError),

    /// Error for unsupported option types.
    #[error("Unsupported option type '{option_type}' for pricing method '{method}'")]
    UnsupportedOptionType {
        /// The option type that is not supported
        option_type: String,
        /// The pricing method that does not support this option type
        method: String,
    },

    /// A pricing kernel produced a non-finite `f64` value (`NaN` /
    /// `±∞`) at an `f64` → `Decimal` boundary.
    ///
    /// Emitted wherever a Black-Scholes style closed-form, a
    /// numerical integrator, or a Monte-Carlo payoff computes an
    /// intermediate `f64` that would otherwise be wrapped in
    /// `Decimal::from_f64(..)` and silently become `Decimal::ZERO`
    /// or saturate. `context` is a static call-site tag following the
    /// same convention as [`crate::error::DecimalError::Overflow`]
    /// (for example `"pricing::bs::call::d1"` or
    /// `"pricing::monte_carlo::payoff::cast"`), so the failing
    /// kernel is identifiable without a stack trace.
    #[error("pricing non-finite {context}: {value}")]
    NonFinite {
        /// Static tag identifying the kernel and step that produced
        /// the non-finite value.
        context: &'static str,
        /// The offending `f64` value (`NaN`, `+∞`, or `-∞`).
        value: f64,
    },
}

impl PricingError {
    /// Creates a new `MethodError` variant.
    ///
    /// # Arguments
    /// * `method` - Name of the pricing method that failed
    /// * `reason` - Detailed reason for the failure
    #[must_use]
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
    #[must_use]
    pub fn simulation_error(reason: &str) -> Self {
        PricingError::SimulationError {
            reason: reason.to_string(),
        }
    }

    /// Creates a new `InvalidEngine` variant.
    ///
    /// # Arguments
    /// * `reason` - Detailed reason for the invalid engine
    #[must_use]
    pub fn invalid_engine(reason: &str) -> Self {
        PricingError::InvalidEngine {
            reason: reason.to_string(),
        }
    }

    /// Creates a typed `MethodError` with `method = "pricing"` as a lightweight
    /// replacement for the former `OtherError` catch-all. Prefer
    /// [`PricingError::method_error`] with a specific method name when known.
    #[must_use]
    #[inline]
    pub fn other(reason: &str) -> Self {
        PricingError::MethodError {
            method: "pricing".to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `UnsupportedOptionType` variant.
    ///
    /// # Arguments
    /// * `option_type` - The option type that is not supported
    /// * `method` - The pricing method that does not support this option type
    #[must_use]
    pub fn unsupported_option_type(option_type: &str, method: &str) -> Self {
        PricingError::UnsupportedOptionType {
            option_type: option_type.to_string(),
            method: method.to_string(),
        }
    }

    /// Creates a [`PricingError::NonFinite`] from a static call-site
    /// tag and the offending `f64` value.
    ///
    /// Intended to be used at `f64` → `Decimal` boundaries inside
    /// pricing kernels, as a thin constructor paired with an
    /// `if !value.is_finite() { .. }` guard.
    #[must_use]
    #[inline]
    #[cold]
    pub fn non_finite(context: &'static str, value: f64) -> Self {
        PricingError::NonFinite { context, value }
    }
}

/// Type alias for Results that may return a `PricingError`.
///
/// This is a convenience type for functions that return pricing results.
pub type PricingResult<T> = Result<T, PricingError>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests_pricing_non_finite {
    use super::*;

    #[test]
    fn non_finite_constructor_nan() {
        let err = PricingError::non_finite("pricing::bs::d1", f64::NAN);
        match err {
            PricingError::NonFinite { context, value } => {
                assert_eq!(context, "pricing::bs::d1");
                assert!(value.is_nan());
            }
            _ => panic!("expected NonFinite"),
        }
    }

    #[test]
    fn non_finite_display_includes_context_and_value() {
        let err = PricingError::non_finite("pricing::mc::payoff", f64::INFINITY);
        let msg = err.to_string();
        assert!(msg.contains("pricing::mc::payoff"));
        assert!(msg.contains("inf"));
    }

    #[test]
    fn non_finite_neg_infinity() {
        let err = PricingError::non_finite("pricing::kernel", f64::NEG_INFINITY);
        match err {
            PricingError::NonFinite { value, .. } => {
                assert!(value.is_infinite() && value.is_sign_negative());
            }
            _ => panic!("expected NonFinite"),
        }
    }
}
