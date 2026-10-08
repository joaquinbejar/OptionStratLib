//! Target crate (ADR-0001 D2): **analytics**. Owns `RNDError`.
//!
//! The risk-neutral density and the volatility skew are analytics computed
//! over an option chain, so their failures belong to analytics rather than
//! to the market crate that owns the chain (#829). A failure of the chain
//! itself (no ATM volatility, for instance) still travels as the market's
//! `ChainError`, carried unchanged.

use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::PositiveError;
use optionstratlib_market::error::ChainError;
use thiserror::Error;

/// Failure of [`crate::analytics::RNDAnalysis`] and of the
/// [`crate::analytics::RNDResult`] / [`crate::analytics::RNDStatistics`]
/// constructors.
#[derive(Debug, Error)]
pub enum RNDError {
    /// No strike produced a positive risk-neutral density.
    #[error("failed to calculate any valid risk-neutral density value")]
    EmptyDensities,

    /// No strike produced a point of the volatility skew.
    #[error("no valid data points available for volatility skew calculation")]
    EmptySkewData,

    /// A parameter of the calculation, or a property of the chain it reads,
    /// cannot be used.
    #[error("invalid RND parameter `{parameter}`: {reason}")]
    InvalidParameters {
        /// The parameter or chain property at fault.
        parameter: &'static str,
        /// Why it cannot be used.
        reason: String,
    },

    /// The option chain could not provide what the calculation needs.
    #[error(transparent)]
    Chain(Box<ChainError>),

    /// A density, a moment or the discount factor left the `Decimal` range.
    #[error(transparent)]
    Decimal(#[from] DecimalError),

    /// A variance or volatility has no representable `Positive` value.
    #[error(transparent)]
    Positive(#[from] PositiveError),
}

impl RNDError {
    /// Builds an [`RNDError::InvalidParameters`].
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn invalid_parameters(parameter: &'static str, reason: &str) -> Self {
        RNDError::InvalidParameters {
            parameter,
            reason: reason.to_string(),
        }
    }
}

impl From<ChainError> for RNDError {
    #[inline]
    fn from(error: ChainError) -> Self {
        RNDError::Chain(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rnd_error_messages_match_the_former_chain_variants() {
        assert_eq!(
            RNDError::EmptyDensities.to_string(),
            "failed to calculate any valid risk-neutral density value"
        );
        assert_eq!(
            RNDError::EmptySkewData.to_string(),
            "no valid data points available for volatility skew calculation"
        );
    }

    #[test]
    fn test_rnd_error_carries_a_chain_error_unchanged() {
        let chain = ChainError::invalid_parameters("strike", "missing");
        let message = chain.to_string();
        let error = RNDError::from(chain);
        assert!(matches!(error, RNDError::Chain(_)));
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn test_rnd_error_invalid_parameters_names_the_parameter() {
        let error = RNDError::invalid_parameters("derivative_tolerance", "must be positive");
        assert_eq!(
            error.to_string(),
            "invalid RND parameter `derivative_tolerance`: must be positive"
        );
    }
}
