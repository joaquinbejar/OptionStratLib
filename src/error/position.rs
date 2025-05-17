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

use crate::error::StrategyError;
use crate::model::types::{OptionStyle, Side};
use std::error::Error;
use std::fmt;

/// Represents errors that can occur when managing positions in strategies
///
/// This enum provides a top-level categorization of position-related errors,
/// grouping them by their source or nature. It helps with routing errors to
/// appropriate handlers and providing context-aware error messages.
///
/// # Variants
///
/// * `StrategyError` - Errors related to strategy operations such as configuration
///   issues or capacity limitations.
///
/// * `ValidationError` - Errors related to position validation including issues
///   with size, price, or compatibility with strategy requirements.
///
/// * `LimitError` - Errors related to position limits such as maximum number of
///   positions or maximum exposure thresholds.
///
/// # Usage
///
/// This error type is typically used in trading systems where positions need to
/// be validated, managed, and executed within the context of trading strategies.
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
///
/// This enum captures detailed error types related to strategy operations,
/// providing specific information about why a strategy operation failed.
/// These errors typically occur during strategy management operations
/// such as adding positions, updating configurations, or executing trades.
///
/// # Variants
///
/// * `UnsupportedOperation` - Indicates that the attempted operation is not
///   supported by the particular strategy type.
///
/// * `StrategyFull` - Indicates that the strategy has reached its maximum
///   capacity for positions.
///
/// * `InvalidConfiguration` - Indicates that the strategy configuration is
///   invalid or inconsistent.
///
/// # Usage
///
/// Used when validating and executing strategy operations to provide detailed
/// error information about why an operation could not be completed.
#[derive(Debug)]
pub enum StrategyErrorKind {
    /// Operation is not supported by this strategy
    ///
    /// Occurs when attempting to perform an operation that is not compatible
    /// with the current strategy type.
    UnsupportedOperation {
        /// The type of strategy that doesn't support the operation
        strategy_type: String,

        /// The name of the operation that was attempted
        operation: String,
    },

    /// Strategy has reached its maximum capacity
    ///
    /// Occurs when attempting to add more positions than a strategy can handle.
    StrategyFull {
        /// The type of strategy that reached its capacity
        strategy_type: String,

        /// The maximum number of positions the strategy can hold
        max_positions: usize,
    },

    /// Invalid strategy configuration
    ///
    /// Occurs when the strategy's configuration parameters are invalid or inconsistent.
    InvalidConfiguration(String),
}

/// Errors related to position validation
///
/// This enum represents errors that can occur during position validation checks.
/// These errors typically happen when attempting to create or modify a position
/// with invalid or incompatible parameters.
///
/// # Variants
///
/// * `InvalidSize` - The position size is invalid (e.g., zero, negative, or too large).
///
/// * `InvalidPrice` - The position price is invalid (e.g., negative or unrealistic).
///
/// * `IncompatibleSide` - The position side (long/short) is incompatible with the strategy.
///
/// * `IncompatibleStyle` - The option style is incompatible with the strategy.
///
/// * `InvalidPosition` - The position is invalid for other specific reasons.
///
/// * `StdError` - Standard error from external systems or libraries.
///
/// # Usage
///
/// Used when validating positions to ensure they meet all requirements before
/// being added to a strategy or executed in a trading system.
#[derive(Debug)]
pub enum PositionValidationErrorKind {
    /// Position size is invalid
    ///
    /// Occurs when the specified position size violates size constraints.
    InvalidSize {
        /// The invalid size value
        size: f64,

        /// Explanation of why the size is invalid
        reason: String,
    },

    /// Position price is invalid
    ///
    /// Occurs when the specified price violates price constraints or is unrealistic.
    InvalidPrice {
        /// The invalid price value
        price: f64,

        /// Explanation of why the price is invalid
        reason: String,
    },

    /// Position type is incompatible with strategy
    ///
    /// Occurs when the position side (long/short) conflicts with strategy requirements.
    IncompatibleSide {
        /// The incompatible position side
        position_side: Side,

        /// Explanation of why the side is incompatible
        reason: String,
    },

    /// Option style is incompatible with strategy
    ///
    /// Occurs when the option style (American/European) conflicts with strategy requirements.
    IncompatibleStyle {
        /// The incompatible option style
        style: OptionStyle,

        /// Explanation of why the style is incompatible
        reason: String,
    },

    /// Position is invalid for other reasons
    ///
    /// A general error for positions that fail validation for reasons not covered by other variants.
    InvalidPosition {
        /// Explanation of why the position is invalid
        reason: String,
    },

    /// Standard error from external systems
    ///
    /// Wraps standard errors from external libraries or systems.
    StdError {
        /// Description of the standard error
        reason: String,
    },
}

/// Represents errors related to position limits in trading operations.
///
/// This enum captures different types of limit violations that can occur when
/// managing trading positions, such as exceeding the maximum number of allowed
/// positions or reaching the maximum exposure threshold for a portfolio.
///
/// # Variants
///
/// * `MaxPositionsReached` - Error indicating that the maximum number of allowed
///   positions has been reached, preventing the creation of additional positions.
///
/// * `MaxExposureReached` - Error indicating that the maximum allowed financial
///   exposure has been reached, preventing positions that would increase exposure.
///
/// # Usage
///
/// This error type is typically used in position management systems, risk control
/// mechanisms, and trading platforms to enforce safety limits and prevent excessive
/// risk taking.
///
#[derive(Debug)]
pub enum PositionLimitErrorKind {
    /// Error indicating the maximum number of positions has been reached
    ///
    /// This variant is triggered when attempting to create a new position would
    /// exceed the configured maximum number of positions allowed in a portfolio
    /// or trading account.
    MaxPositionsReached {
        /// The current number of positions in the portfolio
        current: usize,
        /// The maximum number of positions allowed
        maximum: usize,
    },

    /// Error indicating the maximum allowed exposure has been reached
    ///
    /// This variant is triggered when a new position or modification would cause
    /// the total portfolio exposure to exceed the configured maximum risk threshold.
    /// Exposure is typically measured in monetary terms based on position value or risk.
    MaxExposureReached {
        /// The current financial exposure level of the portfolio
        current_exposure: f64,
        /// The maximum allowed exposure level
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

/// Factory methods for creating position-related errors
///
/// This implementation provides a set of convenience factory methods for creating
/// different types of position errors. These methods create properly structured
/// error instances with clear, descriptive information about what went wrong.
///
/// # Methods
///
/// These factory methods simplify error creation throughout the codebase and ensure
/// that errors have consistent formatting and information.
impl PositionError {
    /// Creates an error for operations not supported by a specific strategy type
    ///
    /// # Parameters
    ///
    /// * `strategy_type` - The name or identifier of the strategy that doesn't support the operation
    /// * `operation` - The name of the unsupported operation that was attempted
    ///
    /// # Returns
    ///
    /// A `PositionError::StrategyError` variant with UnsupportedOperation details
    pub fn unsupported_operation(strategy_type: &str, operation: &str) -> Self {
        PositionError::StrategyError(StrategyErrorKind::UnsupportedOperation {
            strategy_type: strategy_type.to_string(),
            operation: operation.to_string(),
        })
    }

    /// Creates an error when a strategy has reached its maximum position capacity
    ///
    /// # Parameters
    ///
    /// * `strategy_type` - The name or identifier of the strategy that is at capacity
    /// * `max_positions` - The maximum number of positions the strategy can hold
    ///
    /// # Returns
    ///
    /// A `PositionError::StrategyError` variant with StrategyFull details
    pub fn strategy_full(strategy_type: &str, max_positions: usize) -> Self {
        PositionError::StrategyError(StrategyErrorKind::StrategyFull {
            strategy_type: strategy_type.to_string(),
            max_positions,
        })
    }

    /// Creates an error for invalid position size values
    ///
    /// # Parameters
    ///
    /// * `size` - The invalid position size value
    /// * `reason` - A description of why the size is invalid
    ///
    /// # Returns
    ///
    /// A `PositionError::ValidationError` variant with InvalidSize details
    pub fn invalid_position_size(size: f64, reason: &str) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::InvalidSize {
            size,
            reason: reason.to_string(),
        })
    }

    /// Creates an error for incompatible position side/direction
    ///
    /// # Parameters
    ///
    /// * `position_side` - The position side (Long or Short) that is incompatible
    /// * `reason` - A description of why the position side is incompatible
    ///
    /// # Returns
    ///
    /// A `PositionError::ValidationError` variant with IncompatibleSide details
    pub fn invalid_position_type(position_side: Side, reason: String) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::IncompatibleSide {
            position_side,
            reason,
        })
    }

    /// Creates an error for incompatible option style
    ///
    /// # Parameters
    ///
    /// * `style` - The option style (Call or Put) that is incompatible
    /// * `reason` - A description of why the option style is incompatible
    ///
    /// # Returns
    ///
    /// A `PositionError::ValidationError` variant with IncompatibleStyle details
    pub fn invalid_position_style(style: OptionStyle, reason: String) -> Self {
        PositionError::ValidationError(PositionValidationErrorKind::IncompatibleStyle {
            style,
            reason,
        })
    }

    /// Creates a generic invalid position error
    ///
    /// # Parameters
    ///
    /// * `reason` - A description of why the position is invalid
    ///
    /// # Returns
    ///
    /// A `PositionError::ValidationError` variant with InvalidPosition details
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

// Implement conversion from StrategyError to PositionError
impl From<StrategyError> for PositionError {
    fn from(error: StrategyError) -> Self {
        PositionError::StrategyError(StrategyErrorKind::UnsupportedOperation {
            operation: "".to_string(),
            strategy_type: error.to_string(),
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
        let std_error: Box<dyn Error> = Box::new(std::io::Error::other("dynamic error"));
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

    #[test]
    fn test_position_error_validation_error() {
        let error = PositionError::ValidationError(PositionValidationErrorKind::InvalidSize {
            size: -1.0,
            reason: "Size must be positive".to_string(),
        });
        assert_eq!(
            format!("{}", error),
            "Position validation error: Invalid position size -1: Size must be positive"
        );
    }

    #[test]
    fn test_position_error_limit_error() {
        let error = PositionError::LimitError(PositionLimitErrorKind::MaxPositionsReached {
            current: 10,
            maximum: 5,
        });
        assert_eq!(
            format!("{}", error),
            "Position limit error: Maximum number of positions reached (10/5)"
        );
    }

    #[test]
    fn test_strategy_error_strategy_full() {
        let error = StrategyErrorKind::StrategyFull {
            strategy_type: "Iron Condor".to_string(),
            max_positions: 10,
        };
        assert_eq!(
            format!("{}", error),
            "Strategy 'Iron Condor' is full (maximum 10 positions)"
        );
    }

    #[test]
    fn test_strategy_error_invalid_configuration() {
        let error = StrategyErrorKind::InvalidConfiguration("Invalid risk parameters".to_string());
        assert_eq!(
            format!("{}", error),
            "Invalid strategy configuration: Invalid risk parameters"
        );
    }

    #[test]
    fn test_position_validation_error_invalid_price() {
        let error = PositionValidationErrorKind::InvalidPrice {
            price: 105.5,
            reason: "Outside allowable range".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid position price 105.5: Outside allowable range"
        );
    }

    #[test]
    fn test_position_validation_error_invalid_position() {
        let error = PositionValidationErrorKind::InvalidPosition {
            reason: "Position size exceeds margin".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid position: Position size exceeds margin"
        );
    }

    #[test]
    fn test_position_validation_error_incompatible_style() {
        let error = PositionValidationErrorKind::IncompatibleStyle {
            style: OptionStyle::Call,
            reason: "Unsupported for Call options".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Position style 'Call' is incompatible with strategy: Unsupported for Call options"
        );
    }

    #[test]
    fn test_position_validation_error_std_error() {
        let error = PositionValidationErrorKind::StdError {
            reason: "Unexpected null value".to_string(),
        };
        assert_eq!(format!("{}", error), "Error: Unexpected null value");
    }
}
