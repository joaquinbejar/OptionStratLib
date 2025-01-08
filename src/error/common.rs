/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/1/25
******************************************************************************/
use std::error::Error;
use std::fmt;

impl Error for OperationErrorKind {}

/// Represents the types of errors that can occur during operations related to trading strategies or other processes.
///
/// This enum categorizes errors for better debugging and handling in a structured way.
///
/// ## Variants:
///
/// ### `NotSupported`
/// Indicates that the requested operation is not supported for a given strategy type.
/// - Fields:
///   - `operation` (`String`): The name of the operation that was attempted.
///   - `strategy_type` (`String`): The type of strategy for which the operation is not supported.
///
/// Use this variant to signify an incompatibility between a requested operation and 
/// the context (e.g., trying to calculate an invalid metric for the strategy type).
///
/// ### `InvalidParameters`
/// Reflects that the operation was provided with invalid or insufficient parameters.
/// - Fields:
///   - `operation` (`String`): The name of the operation that failed.
///   - `reason` (`String`): A human-readable explanation of why the parameters are invalid.
///
/// Leverage this variant when a specific operation fails due to malformed input or missing 
/// configuration.
///
/// ## Example Use Cases:
/// - `NotSupported`: Attempting to calculate delta for a non-standard options strategy.
/// - `InvalidParameters`: Invalid expiration date or pricing information provided during 
///   probability or profit calculations.
///
/// ## Implementation Notes:
/// - This enum is useful for libraries or systems implementing and managing trading strategies where clear error reporting is critical.
/// - It should be used consistently across the library wherever operations might fail due to strategy-specific issues or input validation errors.
///
/// ## Integration:
/// - Can be combined with other custom and standard error types to construct a comprehensive
///   error-handling flow within the library.
/// - Variants should provide enough information to guide library users or developers toward correcting the issue (e.g., suggesting valid operations or explaining the parameter issue).
///
/// ## Debugging:
/// Implements the `Debug` trait for detailed error information during development and testing.
///
/// ## Related Modules:
/// - Strategy operations often interact with this error type to handle unsupported cases dynamically or to validate input correctness.
/// - Closely related to `StrategyError` or other error categories from the `strategies` module.
#[derive(Debug)]
pub enum OperationErrorKind {
    /// Operation not supported for this strategy
    NotSupported {
        operation: String,
        strategy_type: String,
    },
    /// Invalid parameters for operation
    InvalidParameters { operation: String, reason: String },
}

impl fmt::Display for OperationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationErrorKind::NotSupported {
                operation,
                strategy_type,
            } => {
                write!(
                    f,
                    "Operation '{}' is not supported for strategy '{}'",
                    operation, strategy_type
                )
            }
            OperationErrorKind::InvalidParameters { operation, reason } => {
                write!(
                    f,
                    "Invalid parameters for operation '{}': {}",
                    operation, reason
                )
            }
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