//! Errors of the delta-neutral adjustment optimiser.
//!
//! Target crate (ADR-0001 D6): **strategies**. Owns `AdjustmentError`, raised
//! by `AdjustmentOptimizer` while building an `AdjustmentPlan`. It moved here
//! from `strategies::delta_neutral::adjustment` so every concrete error of the
//! crate lives in `optionstratlib_strategies::error` (#556).

use optionstratlib_core::error::DecimalError;
use optionstratlib_pricing::error::GreeksError;
use thiserror::Error;

/// Error types specific to adjustment operations.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AdjustmentError {
    /// No viable adjustment plan could be found
    #[error("No viable adjustment plan found")]
    NoViablePlan,
    /// Cost constraint exceeded
    #[error("Adjustment cost exceeds maximum")]
    CostExceeded,
    /// No positions to adjust
    #[error("No positions to adjust")]
    NoPositions,
    /// Invalid leg index
    #[error("Invalid leg index: {0}")]
    InvalidLegIndex(usize),
    /// Greeks calculation failed
    #[error("Greeks calculation error: {0}")]
    GreeksError(String),
    /// Configuration constraint violated
    #[error("Configuration violation: {0}")]
    ConfigurationViolation(String),
}

impl From<DecimalError> for AdjustmentError {
    #[cold]
    #[inline(never)]
    fn from(error: DecimalError) -> Self {
        AdjustmentError::GreeksError(error.to_string())
    }
}

impl From<GreeksError> for AdjustmentError {
    #[cold]
    #[inline(never)]
    fn from(err: GreeksError) -> Self {
        AdjustmentError::GreeksError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The messages are the ones the hand-written `Display` produced before
    /// the move.
    #[test]
    fn test_adjustment_error_display_unchanged() {
        assert_eq!(
            AdjustmentError::NoViablePlan.to_string(),
            "No viable adjustment plan found"
        );
        assert_eq!(
            AdjustmentError::CostExceeded.to_string(),
            "Adjustment cost exceeds maximum"
        );
        assert_eq!(
            AdjustmentError::NoPositions.to_string(),
            "No positions to adjust"
        );
        assert_eq!(
            AdjustmentError::InvalidLegIndex(3).to_string(),
            "Invalid leg index: 3"
        );
        assert_eq!(
            AdjustmentError::GreeksError("x".to_string()).to_string(),
            "Greeks calculation error: x"
        );
        assert_eq!(
            AdjustmentError::ConfigurationViolation("y".to_string()).to_string(),
            "Configuration violation: y"
        );
    }
}
