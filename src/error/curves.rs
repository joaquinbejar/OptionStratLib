/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com
    Date: 24/12/24
******************************************************************************/


use crate::error::PositionError;
use std::error::Error;
use std::fmt;
use crate::error::common::OperationErrorKind;

impl Error for CurvesError {}

/// Represents errors related to the management and calculation of trading curves.
///
/// This enum provides detailed errors for various situations encountered in
/// the `curves` module, including price calculations, break-even errors, and
/// general operation failures. Each variant is designed to encapsulate specific
/// information related to its context, aiding in debugging and error recovery.
///
/// ## Variants
///
/// ### `Point2D`
/// Represents errors that arise during calculations or operations involving
/// 2D points (e.g., plotting, data interpolation).
/// - Fields:
///   - `reason` (`&'static str`): A static string describing the reason for
///     the error.
///
/// Use this variant to signal issues like missing data points, invalid input,
/// or calculation failures specific to two-dimensional curve data.
///
/// ### `OperationError`
/// Reflects errors related to trading strategy operations, such as unsupported
/// operations or invalid parameters. It encapsulates the `OperationErrorKind`
/// type, which provides more granular categorization:
/// - `NotSupported`: Indicates the operation is not supported for a specific
///   strategy type.
/// - `InvalidParameters`: Represents issues with input parameters for the
///   operation.
///
/// This variant is helpful for handling errors that arise during complex
/// strategy-related workflows, ensuring clear and structured error reporting.
///
/// ### `StdError`
/// Captures general-purpose errors that do not fall under the other specialized
/// categories.
/// - Fields:
///   - `reason` (`String`): A string describing the error, meant for dynamic
///     error messages or data.
///
/// Use this variant for errors that might originate outside of the library or
/// represent unexpected conditions.
///
/// ## Implementation Notes
/// - Implements the `Debug` trait for easy error inspection during development.
/// - Each variant is designed to correspond with specific aspects of the `curves`
///   module functionality, ensuring precise categorization and improved maintainability.
///
/// ## Examples
/// - `Point2D(reason: "Invalid point value")`: An example of a failure when
///   trying to perform a calculation involving invalid data.
/// - `OperationError(OperationErrorKind::NotSupported { operation: "SomeOp", strategy_type: "SomeStrategy" })`: Indicates an unsupported operation for a trading strategy.
/// - `StdError { reason: "File not found" }`: Represents a general error unrelated to specific logic.
///
/// ## Integration
/// This enum can integrate seamlessly into a broader library structure, leveraging
/// the categorized error handling paradigm:
/// - Commonly used with higher-level modules like `strategies` or `chains`.
/// - Compatible with other custom errors for a unified error-handling approach.
///
/// ## Compatibility
/// - Errors in this enum may interact closely with the `OperationErrorKind` type,
///   specifically for strategy-related operations.
/// - Designed to provide sufficient information for debugging and guiding the user
///   towards resolving the issue.
#[derive(Debug)]
pub enum CurvesError {
    /// Errors related to price calculations
    Point2D{
        reason: &'static str,
    },
    /// Errors related to break-even points

    /// Errors related to strategy operations
    OperationError(OperationErrorKind),

    StdError {
        reason: String,
    },
}

/// Implementation of utility methods for constructing specific `CurvesError`
/// variants related to trading strategy operations.
///
/// These methods provide convenient ways to generate errors of the 
/// `OperationErrorKind` type, encapsulated within the `CurvesError::OperationError` variant.
///
/// ## Methods
///
/// ### `operation_not_supported`
/// Creates a `CurvesError` representing an unsupported operation error.
/// - Parameters:
///   - `operation` (`&str`): The name of the operation that is not supported.
///   - `strategy_type` (`&str`): The strategy type for which the operation is not supported.
/// - Returns:
///   - `CurvesError` containing an `OperationErrorKind::NotSupported` instance.
///
/// Use this method to indicate that the requested operation cannot be performed
/// for the specified trading strategy.
///
/// ### `invalid_parameters`
/// Constructs a `CurvesError` representing an invalid parameters error.
/// - Parameters:
///   - `operation` (`&str`): The name of the operation that failed.
///   - `reason` (`&str`): A description of why the parameters are invalid.
/// - Returns:
///   - `CurvesError` containing an `OperationErrorKind::InvalidParameters` instance.
///
/// This method is useful for scenarios where an operation failure occurs due to
/// malformed or invalid input parameters.
///
/// ## Example Integration
/// These methods are designed to provide a standardized way of creating errors
/// for operation failures in the context of trading curves or strategy management.
/// They are closely tied to the `OperationErrorKind` enum, which categorizes 
/// strategy-related errors in a structured manner.
///
/// ## Debugging and Error Handling
/// Utilizing these methods ensures that the error reporting is consistent and 
/// provides detailed information for developers or library users to resolve 
/// issues effectively.
///
/// ## Related Types
/// - `CurvesError`: The parent enum which encapsulates `OperationErrorKind`.
/// - `OperationErrorKind`: The enum representing granular types of errors for
///   strategy operations.
impl CurvesError {
    pub fn operation_not_supported(operation: &str, strategy_type: &str) -> Self {
        CurvesError::OperationError(OperationErrorKind::NotSupported {
            operation: operation.to_string(),
            strategy_type: strategy_type.to_string(),
        })
    }

    pub fn invalid_parameters(operation: &str, reason: &str) -> Self {
        CurvesError::OperationError(OperationErrorKind::InvalidParameters {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl fmt::Display for CurvesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurvesError::OperationError(err) => write!(f, "Operation error: {}", err),
            CurvesError::StdError { reason } => write!(f, "Error: {}", reason),
            CurvesError::Point2D { reason } => write!(f, "Error: {}", reason),
        }
    }
}

pub type CurvesResult<T> = Result<T, CurvesError>;

impl From<PositionError> for CurvesError {
    fn from(err: PositionError) -> Self {
        CurvesError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Position".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<Box<dyn Error>> for CurvesError {
    fn from(err: Box<dyn Error>) -> Self {
        CurvesError::StdError {
            reason: err.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_error_not_supported_display() {
        let error = OperationErrorKind::NotSupported {
            operation: "calculate_profit".to_string(),
            strategy_type: "IronCondor".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Operation 'calculate_profit' is not supported for strategy 'IronCondor'"
        );
    }

    #[test]
    fn test_operation_error_invalid_parameters_display() {
        let error = OperationErrorKind::InvalidParameters {
            operation: "validate_strikes".to_string(),
            reason: "Strike prices must be positive".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Invalid parameters for operation 'validate_strikes': Strike prices must be positive"
        );
    }

    #[test]
    fn test_operation_error_debug() {
        let error = OperationErrorKind::NotSupported {
            operation: "calculate_profit".to_string(),
            strategy_type: "IronCondor".to_string(),
        };

        assert_eq!(
            format!("{:?}", error),
            "NotSupported { operation: \"calculate_profit\", strategy_type: \"IronCondor\" }"
        );
    }

    #[test]
    fn test_operation_error_as_error() {
        let error = OperationErrorKind::InvalidParameters {
            operation: "validate_strikes".to_string(),
            reason: "Strike prices must be positive".to_string(),
        };

        let error_ref: &dyn Error = &error;
        assert_eq!(
            error_ref.to_string(),
            "Invalid parameters for operation 'validate_strikes': Strike prices must be positive"
        );
    }

    #[test]
    fn test_operation_error_kinds_distinct() {
        let error1 = OperationErrorKind::NotSupported {
            operation: "op".to_string(),
            strategy_type: "strat".to_string(),
        };

        let error2 = OperationErrorKind::InvalidParameters {
            operation: "op".to_string(),
            reason: "err".to_string(),
        };

        assert_ne!(format!("{:?}", error1), format!("{:?}", error2));
    }

    #[test]
    fn test_operation_error_empty_strings() {
        let error = OperationErrorKind::NotSupported {
            operation: "".to_string(),
            strategy_type: "".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Operation '' is not supported for strategy ''"
        );
    }
}