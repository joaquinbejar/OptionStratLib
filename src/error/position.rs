/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/

//! # Position Error Module
//!
//! This module provides error handling for position-related operations in option trading strategies.
//! It defines error types to handle various scenarios related to position validation, strategy
//! operations, and position limits.
//!
//! ## Error Types
//!
//! ### Position Error (`PositionError`)
//! The main error type with three variants:
//! * `StrategyError` - For strategy operation failures
//! * `ValidationError` - For position validation failures
//! * `LimitError` - For position limit violations
//!
//! ### Strategy Errors (`StrategyErrorKind`)
//! Handles specific strategy-related errors:
//! * Unsupported operations
//! * Strategy capacity limits
//! * Invalid configurations
//!
//! ### Validation Errors (`PositionValidationErrorKind`)
//! Handles position validation failures:
//! * Invalid position sizes
//! * Invalid prices
//! * Incompatible sides (Long/Short)
//! * Incompatible styles (Call/Put)
//! * General position invalidity
//!
//! ### Limit Errors (`PositionLimitErrorKind`)
//! Handles position limit violations:
//! * Maximum positions reached
//! * Maximum exposure reached
//!
//! ## Usage Example
//!
//! ```rust
//! use optionstratlib::error::position::PositionError;
//!
//! fn validate_position_size(size: f64) -> Result<(), PositionError> {
//!     if size <= 0.0 {
//!         return Err(PositionError::invalid_position_size(
//!             size,
//!             "Position size must be positive"
//!         ));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Helper Methods
//!
//! The module provides several helper methods for creating common errors:
//! * `unsupported_operation` - Creates an error for unsupported strategy operations
//! * `strategy_full` - Creates an error when strategy capacity is reached
//! * `invalid_position_size` - Creates an error for invalid position sizes
//! * `invalid_position_type` - Creates an error for incompatible position sides
//! * `invalid_position_style` - Creates an error for incompatible option styles
//! * `invalid_position` - Creates a general position validation error
//!
//! All error types implement `std::error::Error` and `std::fmt::Display` for proper
//! error handling and formatting capabilities.

use crate::model::types::{OptionStyle, Side};
use std::error::Error;
use std::fmt;

/// Represents errors that can occur when managing positions in strategies
#[derive(Debug)]
pub enum PositionError {
    /// Errors related to strategy operations
    StrategyError(StrategyErrorKind),
    /// Errors related to position validation
    ValidationError(PositionValidationErrorKind),
    /// Errors related to position limits
    LimitError(PositionLimitErrorKind),
}

/// Specific errors that can occur in strategy operations
#[derive(Debug)]
pub enum StrategyErrorKind {
    /// Operation is not supported by this strategy
    UnsupportedOperation {
        strategy_type: String,
        operation: String,
    },
    /// Strategy has reached its maximum capacity
    StrategyFull {
        strategy_type: String,
        max_positions: usize,
    },
    /// Invalid strategy configuration
    InvalidConfiguration(String),
}

/// Errors related to position validation
#[derive(Debug)]
pub enum PositionValidationErrorKind {
    /// Position size is invalid
    InvalidSize {
        size: f64,
        reason: String,
    },
    /// Position price is invalid
    InvalidPrice {
        price: f64,
        reason: String,
    },
    /// Position type is incompatible with strategy
    IncompatibleSide {
        position_side: Side,
        reason: String,
    },
    IncompatibleStyle {
        style: OptionStyle,
        reason: String,
    },
    InvalidPosition {
        reason: String,
    },
    StdError {
        reason: String,
    },
}

/// Errors related to position limits
#[derive(Debug)]
pub enum PositionLimitErrorKind {
    /// Maximum number of positions reached
    MaxPositionsReached { current: usize, maximum: usize },
    /// Maximum exposure reached
    MaxExposureReached {
        current_exposure: f64,
        max_exposure: f64,
    },
}

impl fmt::Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionError::StrategyError(kind) => write!(f, "Strategy error: {}", kind),
            PositionError::ValidationError(kind) => {
                write!(f, "Position validation error: {}", kind)
            }
            PositionError::LimitError(kind) => write!(f, "Position limit error: {}", kind),
        }
    }
}

impl fmt::Display for StrategyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyErrorKind::UnsupportedOperation {
                strategy_type,
                operation,
            } => {
                write!(
                    f,
                    "Operation '{}' is not supported for strategy type '{}'",
                    operation, strategy_type
                )
            }
            StrategyErrorKind::StrategyFull {
                strategy_type,
                max_positions,
            } => {
                write!(
                    f,
                    "Strategy '{}' is full (maximum {} positions)",
                    strategy_type, max_positions
                )
            }
            StrategyErrorKind::InvalidConfiguration(msg) => {
                write!(f, "Invalid strategy configuration: {}", msg)
            }
        }
    }
}

impl fmt::Display for PositionValidationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionValidationErrorKind::InvalidSize { size, reason } => {
                write!(f, "Invalid position size {}: {}", size, reason)
            }
            PositionValidationErrorKind::InvalidPrice { price, reason } => {
                write!(f, "Invalid position price {}: {}", price, reason)
            }
            PositionValidationErrorKind::IncompatibleSide {
                position_side,
                reason: strategy_type,
            } => {
                write!(
                    f,
                    "Position type '{}' is incompatible with strategy '{}'",
                    position_side, strategy_type
                )
            }
            PositionValidationErrorKind::InvalidPosition { reason } => {
                write!(f, "Invalid position: {}", reason)
            }
            PositionValidationErrorKind::IncompatibleStyle { style, reason } => {
                write!(
                    f,
                    "Position style '{}' is incompatible with strategy: {}",
                    style, reason
                )
            }
            PositionValidationErrorKind::StdError { reason } => {
                write!(f, "Error: {}", reason)
            }
        }
    }
}

impl fmt::Display for PositionLimitErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionLimitErrorKind::MaxPositionsReached { current, maximum } => {
                write!(
                    f,
                    "Maximum number of positions reached ({}/{})",
                    current, maximum
                )
            }
            PositionLimitErrorKind::MaxExposureReached {
                current_exposure,
                max_exposure,
            } => {
                write!(
                    f,
                    "Maximum exposure reached (current: {}, max: {})",
                    current_exposure, max_exposure
                )
            }
        }
    }
}

impl Error for PositionError {}

// Helper methods for creating common errors
impl PositionError {
    pub fn unsupported_operation(strategy_type: &str, operation: &str) -> Self {
        PositionError::StrategyError(StrategyErrorKind::UnsupportedOperation {
            strategy_type: strategy_type.to_string(),
            operation: operation.to_string(),
        })
    }

    pub fn strategy_full(strategy_type: &str, max_positions: usize) -> Self {
        PositionError::StrategyError(StrategyErrorKind::StrategyFull {
            strategy_type: strategy_type.to_string(),
            max_positions,
        })
    }

    pub fn invalid_position_size(size: f64, reason: &str) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::InvalidSize {
            size,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_position_type(position_side: Side, reason: String) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::IncompatibleSide {
            position_side,
            reason,
        })
    }

    pub fn invalid_position_style(style: OptionStyle, reason: String) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::IncompatibleStyle {
            style,
            reason,
        })
    }

    pub fn invalid_position(reason: &str) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::InvalidPosition {
            reason: reason.to_string(),
        })
    }
}

impl From<Box<dyn Error>> for PositionError {
    fn from(err: Box<dyn Error>) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::StdError {
            reason: err.to_string(),
        })
    }
}

impl From<&str> for PositionError {
    fn from(err: &str) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::StdError {
            reason: err.to_string(),
        })
    }
}

impl From<String> for PositionError {
    fn from(err: String) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::StdError {
            reason: err.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::base::Positionable;

    struct DummyStrategy;
    impl Positionable for DummyStrategy {}

    #[test]
    fn test_unsupported_operation() {
        let strategy = DummyStrategy;
        let result = strategy.get_positions();
        assert!(matches!(
            result,
            Err(PositionError::StrategyError(
                StrategyErrorKind::UnsupportedOperation { .. }
            ))
        ));
    }

    #[test]
    fn test_error_messages() {
        let error = PositionError::unsupported_operation("TestStrategy", "add_position");
        assert!(error.to_string().contains("TestStrategy"));
        assert!(error.to_string().contains("add_position"));
    }

    #[test]
    fn test_invalid_position_size() {
        let error = PositionError::invalid_position_size(-1.0, "Size cannot be negative");
        assert!(matches!(
            error,
            PositionError::ValidationError(PositionValidationErrorKind::InvalidSize { .. })
        ));
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let error = PositionValidationErrorKind::InvalidSize {
            size: -1.0,
            reason: "Size must be positive".to_string(),
        };

        assert!(error.to_string().contains("-1"));
        assert!(error.to_string().contains("Size must be positive"));

        let error = PositionValidationErrorKind::IncompatibleSide {
            position_side: Side::Long,
            reason: "Strategy requires short positions".to_string(),
        };
        assert!(error.to_string().contains("Long"));
        assert!(error.to_string().contains("Strategy requires short"));
    }

    #[test]
    fn test_limit_error_display() {
        let error = PositionLimitErrorKind::MaxPositionsReached {
            current: 5,
            maximum: 4,
        };
        assert!(error.to_string().contains("5"));
        assert!(error.to_string().contains("4"));

        let error = PositionLimitErrorKind::MaxExposureReached {
            current_exposure: 1000.0,
            max_exposure: 500.0,
        };
        assert!(error.to_string().contains("1000"));
        assert!(error.to_string().contains("500"));
    }

    #[test]
    fn test_error_conversions() {
        // Test de str a PositionError
        let str_error: PositionError = "test error".into();
        assert!(matches!(
            str_error,
            PositionError::ValidationError(PositionValidationErrorKind::StdError { .. })
        ));

        // Test de String a PositionError
        let string_error: PositionError = "test error".to_string().into();
        assert!(matches!(
            string_error,
            PositionError::ValidationError(PositionValidationErrorKind::StdError { .. })
        ));

        // Test de Box<dyn Error> a PositionError
        let std_error: Box<dyn Error> = Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "dynamic error",
        ));
        let position_error = PositionError::from(std_error);
        assert!(matches!(
            position_error,
            PositionError::ValidationError(PositionValidationErrorKind::StdError { .. })
        ));
    }

    #[test]
    fn test_position_error_helper_methods() {
        let error = PositionError::invalid_position_size(-1.0, "Must be positive");
        assert!(matches!(
            error,
            PositionError::ValidationError(PositionValidationErrorKind::InvalidSize { .. })
        ));

        let error = PositionError::invalid_position_type(
            Side::Long,
            "Strategy requires short positions".to_string(),
        );
        assert!(matches!(
            error,
            PositionError::ValidationError(PositionValidationErrorKind::IncompatibleSide { .. })
        ));
    }

    #[test]
    fn test_strategy_error_helper_methods() {
        let error = PositionError::strategy_full("Iron Condor", 4);
        assert!(matches!(
            error,
            PositionError::StrategyError(StrategyErrorKind::StrategyFull { .. })
        ));

        let error = PositionError::unsupported_operation("Iron Condor", "add_leg");
        assert!(matches!(
            error,
            PositionError::StrategyError(StrategyErrorKind::UnsupportedOperation { .. })
        ));
    }
}
