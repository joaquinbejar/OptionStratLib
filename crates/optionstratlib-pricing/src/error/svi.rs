/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/11/26
******************************************************************************/

//! Target crate (ADR-0001 D6): **pricing**. Owns `SviError`.

use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::{Positive, PositiveError};
use rust_decimal::Decimal;
use thiserror::Error;

/// Reasons why an SVI slice cannot be built, evaluated or calibrated.
///
/// Raised by [`SviParameters::try_new`](crate::volatility::SviParameters::try_new),
/// [`SviSlice`](crate::volatility::SviSlice) and
/// [`calibrate_svi`](crate::volatility::calibrate_svi). A failed calibration
/// always comes back as one of these, and there is no flat fallback: a flat
/// result can only be returned when the quotes are flat within
/// [`SviCalibrationConfig::max_relative_rmse`](crate::volatility::SviCalibrationConfig::max_relative_rmse).
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SviError {
    /// A raw SVI parameter is outside its domain.
    #[error("invalid svi parameter {parameter} = {value}: {reason}")]
    InvalidParameter {
        /// The rejected parameter: `b`, `rho` or `sigma`.
        parameter: &'static str,
        /// The rejected value.
        value: Decimal,
        /// The condition the value breaks.
        reason: &'static str,
    },

    /// The parameters give a negative minimum total variance, so
    /// `a + b * sigma * sqrt(1 - rho^2) >= 0` does not hold.
    #[error("svi minimum total variance {minimum} is negative")]
    NegativeMinimumVariance {
        /// The value of `a + b * sigma * sqrt(1 - rho^2)`.
        minimum: Decimal,
    },

    /// The forward is zero, so no strike has a log moneyness.
    #[error("svi forward must be positive")]
    ZeroForward,

    /// The time to expiry is zero, so total variance has no implied
    /// volatility.
    #[error("svi time to expiry must be positive")]
    ZeroTimeToExpiry,

    /// The strike is zero, so it has no log moneyness.
    #[error("svi strike must be positive, got {strike}")]
    ZeroStrike {
        /// The rejected strike.
        strike: Positive,
    },

    /// Evaluating the slice gave a negative total variance larger than the
    /// rounding of that evaluation can explain. Valid parameters are not
    /// expected to reach this, at their minimum or elsewhere: it guards the
    /// arithmetic rather than a known input.
    #[error("svi total variance {total_variance} at log moneyness {log_moneyness} is negative")]
    NegativeTotalVariance {
        /// The log moneyness the slice was evaluated at.
        log_moneyness: Decimal,
        /// The value computed there.
        total_variance: Decimal,
    },

    /// A field of the calibration configuration is outside its domain.
    #[error("invalid svi calibration config {parameter} = {value}: {reason}")]
    InvalidConfig {
        /// The rejected field: `tolerance`.
        parameter: &'static str,
        /// The rejected value.
        value: Decimal,
        /// The condition the value breaks.
        reason: &'static str,
    },

    /// An observation cannot enter a calibration.
    #[error("invalid svi observation at index {index}: {reason}")]
    InvalidObservation {
        /// The position of the observation in the input slice.
        index: usize,
        /// The condition the observation breaks.
        reason: &'static str,
    },

    /// Too few observations with a positive weight to fix five parameters.
    #[error("svi calibration needs at least {required} weighted observations, got {got}")]
    TooFewObservations {
        /// The observations with a positive weight (or, if there are too few
        /// to look further, the observations supplied).
        got: usize,
        /// The minimum: five, one per raw SVI parameter.
        required: usize,
    },

    /// The weights of the observations sum to zero.
    #[error("svi observation weights must have a positive sum")]
    NonPositiveTotalWeight,

    /// Repeated strikes leave too few distinct log moneyness values, which
    /// makes the calibration singular.
    #[error("svi calibration needs {required} distinct strikes, got {distinct}")]
    InsufficientDistinctStrikes {
        /// The distinct strikes among the weighted observations.
        distinct: usize,
        /// The minimum: five, one per raw SVI parameter.
        required: usize,
    },

    /// No start of the search converged within the iteration limit.
    #[error(
        "svi calibration did not converge: {iterations} iterations, limit {limit} for each start"
    )]
    NoConvergence {
        /// The iterations run, summed over every start.
        iterations: u64,
        /// The configured limit for each start.
        limit: u64,
    },

    /// The optimum the kernel found is not a valid set of raw SVI parameters
    /// once rounded to `Decimal`.
    #[error("svi calibration found no feasible parameters: {reason}")]
    NoFeasibleFit {
        /// The validation failure of the rounded parameters.
        reason: String,
    },

    /// The best fit misses the observations by more than the configured limit,
    /// relative to their level. The fit is rejected rather than returned.
    ///
    /// The limit is the only judgement of a fit: a fit within it is not
    /// otherwise judged good, and the caller tightens or loosens it.
    #[error(
        "svi fit relative rmse {relative_rmse} exceeds the limit {limit} (rmse {rmse} in implied volatility)"
    )]
    RmseAboveLimit {
        /// The weighted root mean square error of the best fit, in implied
        /// volatility, over the weighted mean observed implied volatility of
        /// the points with a positive weight.
        relative_rmse: Positive,
        /// The configured limit on that ratio.
        limit: Positive,
        /// The weighted root mean square error of the best fit, in implied
        /// volatility.
        rmse: Positive,
    },

    /// A step of the `f64` kernel produced a non-finite value.
    #[error("svi non-finite {context}: {value}")]
    NonFinite {
        /// Static tag identifying the kernel step.
        context: &'static str,
        /// The offending `f64` value (`NaN`, `+inf`, or `-inf`).
        value: f64,
    },

    /// A checked `Decimal` operation or conversion failed.
    #[error(transparent)]
    Decimal(#[from] DecimalError),

    /// A value could not be represented as a `Positive`.
    #[error(transparent)]
    Positive(#[from] PositiveError),
}

impl SviError {
    /// Creates a [`SviError::NonFinite`] from a static call-site tag and the
    /// offending `f64` value.
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn non_finite(context: &'static str, value: f64) -> Self {
        SviError::NonFinite { context, value }
    }
}

#[cfg(test)]
mod tests_svi_error {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_svi_error_rmse_above_limit_names_all_three_values() {
        let error = SviError::RmseAboveLimit {
            relative_rmse: Positive::new_decimal(dec!(0.15)).unwrap(),
            limit: Positive::new_decimal(dec!(0.03)).unwrap(),
            rmse: Positive::new_decimal(dec!(0.0173)).unwrap(),
        };
        let message = error.to_string();
        assert!(message.contains("0.15"), "{message}");
        assert!(message.contains("0.03"), "{message}");
        assert!(message.contains("0.0173"), "{message}");
    }

    #[test]
    fn test_svi_error_invalid_config_names_the_field_and_value() {
        let message = SviError::InvalidConfig {
            parameter: "tolerance",
            value: dec!(1),
            reason: "must not exceed 0.001",
        }
        .to_string();
        assert!(message.contains("tolerance = 1"), "{message}");
    }

    #[test]
    fn test_svi_error_non_finite_keeps_context_and_value() {
        let error = SviError::non_finite("svi::kernel::objective", f64::INFINITY);
        assert!(matches!(
            error,
            SviError::NonFinite {
                context: "svi::kernel::objective",
                value
            } if value == f64::INFINITY
        ));
    }

    #[test]
    fn test_svi_error_decimal_converts_with_from() {
        let error: SviError = DecimalError::arithmetic_error("svi", "boom").into();
        assert!(matches!(error, SviError::Decimal(_)));
    }

    #[test]
    fn test_svi_error_no_convergence_names_iterations_and_limit() {
        let message = SviError::NoConvergence {
            iterations: 40,
            limit: 2,
        }
        .to_string();
        assert!(message.contains("40"), "{message}");
        assert!(message.contains("limit 2"), "{message}");
    }
}
