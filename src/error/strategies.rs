/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com
    Date: 24/12/24
******************************************************************************/

//! # Strategy Error Module
//!
//! This module provides error handling for option trading strategies and their operations.
//! It defines error types for strategy calculations, validations, and operations,
//! with integration with the probability analysis system.
//!
//! ## Error Types
//!
//! ### Strategy Error (`StrategyError`)
//! The main error enum with four categories:
//! * `PriceError` - For price calculation failures
//! * `BreakEvenError` - For break-even point calculation errors
//! * `ProfitLossError` - For profit/loss calculation failures
//! * `OperationError` - For strategy operation errors
//!
//! ### Price Errors (`PriceErrorKind`)
//! Specific errors for price-related operations:
//! * Invalid underlying prices
//! * Invalid price ranges with start and end points
//!
//! ### Break-Even Errors (`BreakEvenErrorKind`)
//! Handles break-even point calculations:
//! * Calculation failures
//! * Missing break-even points
//!
//! ### Profit/Loss Errors (`ProfitLossErrorKind`)
//! Manages profit and loss calculations:
//! * Maximum profit calculation errors
//! * Maximum loss calculation errors
//! * Profit range calculation errors
//!
//! ## Integration with Probability Analysis
//!
//! Implements conversion from `StrategyError` to `ProbabilityError` for seamless
//! error handling between strategy and probability calculations.
//!
//! ## Usage Example
//!
//! ```rust
//!
//! use optionstratlib::error::strategies::{StrategyError, StrategyResult};
//!
//! fn validate_strategy_operation(operation: &str, strategy: &str) -> StrategyResult<()> {
//!     if !is_supported_operation(operation) {
//!         return Err(StrategyError::operation_not_supported(operation, strategy));
//!     }
//!     Ok(())
//! }
//!
//! fn is_supported_operation(p0: &str) -> bool  {
//!     false
//! }
//! ```
//!
//! ## Helper Methods
//!
//! The module provides convenient methods for creating common errors:
//! * `operation_not_supported` - Creates an error for unsupported operations
//! * `invalid_parameters` - Creates an error for invalid operation parameters
//!
//! ## Type Alias
//!
//! Provides `StrategyResult<T>` for convenient error handling in strategy operations.
use crate::error::{OptionsError, PositionError};
use crate::error::common::OperationErrorKind;
use std::error::Error;
use std::fmt;

impl Error for StrategyError {}
impl Error for PriceErrorKind {}
impl Error for BreakEvenErrorKind {}
impl Error for ProfitLossErrorKind {}

/// Represents the different types of errors that can occur in options trading strategies.
///
/// This enum acts as a comprehensive error type for the options strategy module,
/// grouping more specific error categories for better error handling. Each variant
/// corresponds to a different domain of potential failures within options strategy
/// operations.
///
/// # Variants
///
/// * `PriceError` - Errors related to pricing operations such as invalid prices or ranges
/// * `BreakEvenError` - Errors encountered when calculating strategy break-even points
/// * `ProfitLossError` - Errors related to profit/loss calculations including maximum values
/// * `OperationError` - General strategy operation errors including unsupported operations
/// * `StdError` - Standard errors with a descriptive reason
/// * `NotImplemented` - For features or operations that are not yet implemented
///
/// # Usage
///
/// This error type is designed to be returned from functions that perform operations
/// on options trading strategies, providing structured and detailed error information
/// to facilitate debugging and error handling.
#[derive(Debug)]
pub enum StrategyError {
    /// Errors related to price calculations
    PriceError(PriceErrorKind),

    /// Errors related to break-even points
    BreakEvenError(BreakEvenErrorKind),

    /// Errors related to profit/loss calculations
    ProfitLossError(ProfitLossErrorKind),

    /// Errors related to strategy operations
    OperationError(OperationErrorKind),

    /// Standard error with descriptive reason
    StdError {
        /// Detailed explanation of the standard error
        reason: String,
    },

    /// Indicates a feature or operation that has not been implemented yet
    NotImplemented,
}

/// Represents different types of errors that can occur during price-related operations.
///
/// This enum provides specific error variants for price calculations, validations,
/// and operations. Each variant contains detailed information about the error context
/// to facilitate debugging and error handling in pricing operations.
///
#[derive(Debug)]
pub enum PriceErrorKind {
    /// Error when underlying price is not available or invalid
    ///
    /// This error occurs when the price of the underlying asset cannot be determined
    /// or is considered invalid for calculations (e.g., negative values, NaN, etc.)
    ///
    /// # Fields
    ///
    InvalidUnderlyingPrice {
        /// * `reason` - A detailed explanation of why the price is considered invalid
        reason: String,
    },

    /// Error in price range calculation
    ///
    /// This error occurs when a price range specification is invalid, such as when
    /// the end price is less than the start price, or when price points are outside
    /// of valid bounds.
    ///
    InvalidPriceRange {
        /// * `start` - The beginning price of the attempted range
        start: f64,
        /// * `end` - The ending price of the attempted range
        end: f64,
        /// * `reason` - A detailed explanation of why the price range is invalid
        reason: String,
    },
}

/// Represents the type of errors that can occur during break-even point calculations.
///
/// Break-even points are critical price levels at which an options strategy neither generates
/// profit nor loss. This enum categorizes the various errors that might occur when attempting
/// to calculate these points, providing clear error handling paths for client code.
///
/// # Variants
///
/// * `CalculationError` - Indicates a failure in the mathematical calculation process
///   for break-even points, including the specific reason for the failure.
///
/// * `NoBreakEvenPoints` - Indicates that no break-even points exist for the given strategy.
///   This may occur with certain strategies that are always profitable or always unprofitable
///   across all possible price ranges.
///
/// # Examples
///
/// ```
/// use optionstratlib::error::strategies::BreakEvenErrorKind;
///
/// // When a calculation fails due to invalid input
/// let error = BreakEvenErrorKind::CalculationError {
///     reason: String::from("Division by zero when calculating ratio")
/// };
///
/// // When a strategy has no break-even points
/// let no_points_error = BreakEvenErrorKind::NoBreakEvenPoints;
/// ```
///
/// # Related Errors
///
/// This error type is typically used within the context of `StrategyError` to provide
/// detailed information about failures in strategy analysis, particularly when evaluating
/// profit/loss scenarios at different price points.
#[derive(Debug)]
pub enum BreakEvenErrorKind {
    /// Error that occurs when the break-even calculation process fails
    ///
    /// The `reason` field provides specific details about why the calculation failed,
    /// which can be useful for debugging or providing user feedback.
    CalculationError {
        /// Detailed explanation of what caused the calculation to fail
        reason: String,
    },

    /// Error indicating that no break-even points exist for the given strategy
    ///
    /// This typically occurs with strategies that maintain a consistent profit or loss
    /// profile regardless of the underlying asset's price.
    NoBreakEvenPoints,
}

/// Represents error types that can occur during profit and loss calculations.
///
/// This enum provides structured error information for various failure scenarios
/// that may occur when calculating profit, loss, and related metrics for options
/// strategies or financial instruments.
///
/// # Error Categories
///
/// The enum categorizes errors into three main types:
/// * Maximum profit calculation errors
/// * Maximum loss calculation errors
/// * Profit range and breakeven calculation errors
///
/// Each variant includes a detailed reason string to help diagnose the specific
/// cause of the error and facilitate troubleshooting.
#[derive(Debug)]
pub enum ProfitLossErrorKind {
    /// Error that occurs when calculating maximum profit.
    ///
    /// This might happen due to issues such as invalid input parameters,
    /// computational limitations, or logical inconsistencies in the profit model.
    MaxProfitError {
        /// Detailed explanation of why the maximum profit calculation failed
        reason: String,
    },

    /// Error that occurs when calculating maximum loss.
    ///
    /// This might happen due to issues such as invalid input parameters,
    /// computational limitations, or logical inconsistencies in the loss model.
    MaxLossError {
        /// Detailed explanation of why the maximum loss calculation failed
        reason: String,
    },

    /// Error that occurs during profit range calculation.
    ///
    /// This might happen when trying to determine profit/loss at different price
    /// points, breakeven points, or when analyzing the profit curve of a strategy.
    ProfitRangeError {
        /// Detailed explanation of why the profit range calculation failed
        reason: String,
    },
}

impl fmt::Display for StrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyError::PriceError(err) => write!(f, "Price error: {}", err),
            StrategyError::BreakEvenError(err) => write!(f, "Break-even error: {}", err),
            StrategyError::ProfitLossError(err) => write!(f, "Profit/Loss error: {}", err),
            StrategyError::OperationError(err) => write!(f, "Operation error: {}", err),
            StrategyError::StdError { reason } => write!(f, "Error: {}", reason),
            StrategyError::NotImplemented => write!(f, "Operation not implemented"),
        }
    }
}

impl fmt::Display for PriceErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceErrorKind::InvalidUnderlyingPrice { reason } => {
                write!(f, "Invalid underlying price: {}", reason)
            }
            PriceErrorKind::InvalidPriceRange { start, end, reason } => {
                write!(f, "Invalid price range [{}, {}]: {}", start, end, reason)
            }
        }
    }
}

impl fmt::Display for BreakEvenErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakEvenErrorKind::CalculationError { reason } => {
                write!(f, "Break-even calculation error: {}", reason)
            }
            BreakEvenErrorKind::NoBreakEvenPoints => {
                write!(f, "No break-even points found")
            }
        }
    }
}

impl fmt::Display for ProfitLossErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfitLossErrorKind::MaxProfitError { reason } => {
                write!(f, "Maximum profit calculation error: {}", reason)
            }
            ProfitLossErrorKind::MaxLossError { reason } => {
                write!(f, "Maximum loss calculation error: {}", reason)
            }
            ProfitLossErrorKind::ProfitRangeError { reason } => {
                write!(f, "Profit range calculation error: {}", reason)
            }
        }
    }
}

/// A specialized result type for strategy operations.
///
/// This type alias provides a convenient way to handle results from strategy-related
/// operations that might fail with a `StrategyError`. It follows the standard Rust
/// pattern of using `Result<T, E>` for operations that can fail.
///
/// This makes error handling more readable and concise throughout the strategy-related
/// code, compared to explicitly writing `Result<T, StrategyError>` everywhere.
pub type StrategyResult<T> = Result<T, StrategyError>;

// Implementation helpers
impl StrategyError {
    /// Creates a `StrategyError` for an unsupported operation on a specific strategy type.
    ///
    /// This helper method creates a structured error describing an operation that cannot be performed
    /// on a particular strategy type, encapsulating both the attempted operation name and the strategy
    /// type that doesn't support it.
    ///
    /// # Parameters
    /// * `operation` - The name of the operation that was attempted but is not supported
    /// * `strategy_type` - The type of strategy for which the operation is not supported
    ///
    /// # Returns
    /// A `StrategyError::OperationError` variant with the `NotSupported` kind
    ///
    /// # Example
    /// ```
    /// use optionstratlib::error::strategies::StrategyError;
    ///
    /// // Creating an error when trying to calculate butterfly spread adjustment on an iron condor
    /// let error = StrategyError::operation_not_supported("butterfly_adjustment", "IronCondor");
    /// ```
    pub fn operation_not_supported(operation: &str, strategy_type: &str) -> Self {
        StrategyError::OperationError(OperationErrorKind::NotSupported {
            operation: operation.to_string(),
            reason: strategy_type.to_string(),
        })
    }

    /// Creates a `StrategyError` for invalid parameters provided to an operation.
    ///
    /// This helper method builds a structured error for cases when an operation fails due to
    /// invalid or insufficient parameters, providing context about both the operation and
    /// the specific validation issue.
    ///
    /// # Parameters
    /// * `operation` - The name of the operation that failed due to invalid parameters
    /// * `reason` - A descriptive explanation of why the parameters are invalid
    ///
    /// # Returns
    /// A `StrategyError::OperationError` variant with the `InvalidParameters` kind
    ///
    /// # Example
    /// ```
    /// use optionstratlib::error::strategies::StrategyError;
    ///
    /// // Creating an error when strike prices are invalid for a strategy
    /// let error = StrategyError::invalid_parameters(
    ///     "create_vertical_spread",
    ///     "Short strike must be higher than long strike for call spreads"
    /// );
    /// ```
    pub fn invalid_parameters(operation: &str, reason: &str) -> Self {
        StrategyError::OperationError(OperationErrorKind::InvalidParameters {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl From<PositionError> for StrategyError {
    fn from(err: PositionError) -> Self {
        StrategyError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Position".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<OptionsError> for StrategyError {
    fn from(err: OptionsError) -> Self {
        StrategyError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Options".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<Box<dyn Error>> for StrategyError {
    fn from(err: Box<dyn Error>) -> Self {
        StrategyError::StdError {
            reason: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests_from_str {
    use super::*;
    use crate::error::ProbabilityError;

    #[test]

    fn test_strategy_to_probability_error_conversion() {
        let strategy_error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        let probability_error = ProbabilityError::from(strategy_error);

        assert!(probability_error.to_string().contains("max_profit"));
        assert!(probability_error.to_string().contains("TestStrategy"));
    }

    #[test]

    fn test_profit_loss_error_conversion() {
        let strategy_error = StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
            reason: "Test error".to_string(),
        });
        let probability_error = ProbabilityError::from(strategy_error);

        assert!(probability_error.to_string().contains("Test error"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn test_strategy_error_creation() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        assert!(matches!(
            error,
            StrategyError::OperationError(OperationErrorKind::NotSupported { .. })
        ));
    }

    #[test]

    fn test_error_messages() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        let error_string = error.to_string();
        assert!(error_string.contains("max_profit"));
        assert!(error_string.contains("TestStrategy"));
    }
}

#[cfg(test)]
mod tests_display {
    use super::*;

    #[test]

    fn test_price_error_display() {
        let error = StrategyError::PriceError(PriceErrorKind::InvalidUnderlyingPrice {
            reason: "Price cannot be negative".to_string(),
        });
        assert!(error.to_string().contains("Price cannot be negative"));
    }

    #[test]

    fn test_break_even_error_display() {
        let error = StrategyError::BreakEvenError(BreakEvenErrorKind::CalculationError {
            reason: "Invalid input parameters".to_string(),
        });
        assert!(error.to_string().contains("Invalid input parameters"));
    }

    #[test]

    fn test_profit_loss_error_display() {
        let error = StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
            reason: "Cannot calculate maximum profit".to_string(),
        });
        assert!(
            error
                .to_string()
                .contains("Cannot calculate maximum profit")
        );
    }

    #[test]

    fn test_operation_error_display() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        assert!(error.to_string().contains("max_profit"));
        assert!(error.to_string().contains("TestStrategy"));
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_strategy_error_std_error() {
        let error = StrategyError::StdError {
            reason: "General failure".to_string(),
        };
        assert_eq!(format!("{}", error), "Error: General failure");
    }

    #[test]
    fn test_price_error_invalid_price_range() {
        let error = PriceErrorKind::InvalidPriceRange {
            start: 10.0,
            end: 50.0,
            reason: "Start price must be less than end price".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid price range [10, 50]: Start price must be less than end price"
        );
    }

    #[test]
    fn test_break_even_error_no_points() {
        let error = BreakEvenErrorKind::NoBreakEvenPoints;
        assert_eq!(format!("{}", error), "No break-even points found");
    }

    #[test]
    fn test_profit_loss_error_max_loss_error() {
        let error = ProfitLossErrorKind::MaxLossError {
            reason: "Loss exceeds margin requirements".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Maximum loss calculation error: Loss exceeds margin requirements"
        );
    }

    #[test]
    fn test_profit_loss_error_profit_range_error() {
        let error = ProfitLossErrorKind::ProfitRangeError {
            reason: "Profit calculation failed".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Profit range calculation error: Profit calculation failed"
        );
    }

    #[test]
    fn test_strategy_error_invalid_parameters_constructor() {
        let error = StrategyError::invalid_parameters("Open position", "Margin insufficient");
        assert_eq!(
            format!("{}", error),
            "Operation error: Invalid parameters for operation 'Open position': Margin insufficient"
        );
    }

    #[test]
    fn test_strategy_error_from_boxed_error() {
        let boxed_error: Box<dyn Error> = Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Underlying failure",
        ));
        let error: StrategyError = boxed_error.into();
        assert_eq!(format!("{}", error), "Error: Underlying failure");
    }
}
