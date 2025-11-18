/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/1/25
******************************************************************************/
use thiserror::Error;

/// Represents the types of errors that can occur during operations related to trading strategies or other processes.
///
/// This enum categorizes errors for better debugging and handling in a structured way.
///
/// # Variants
///
/// * `NotSupported` - Indicates that the requested operation is not supported for a given strategy type
///   or context. Used when there's an incompatibility between an operation and its execution context.
///
/// * `InvalidParameters` - Reflects that the operation was provided with invalid or insufficient parameters
///   that prevent successful execution. Used when input validation fails.
///
/// # Examples
///
/// ```
/// use optionstratlib::error::OperationErrorKind;
///
/// // Creating a NotSupported error
/// let error = OperationErrorKind::NotSupported {
///     operation: "calculate_delta".to_string(),
///     reason: "Not applicable for binary options".to_string()
/// };
///
/// // Creating an InvalidParameters error
/// let error = OperationErrorKind::InvalidParameters {
///     operation: "calculate_premium".to_string(),
///     reason: "Missing volatility input".to_string()
/// };
/// ```
///
/// # Use Cases
///
/// - Used in strategy implementations to signal operation compatibility issues
/// - Used in validation layers to report problems with input parameters
/// - Helps create clear error messages that guide users toward resolution
///
/// # Related Types
///
/// This error type is typically used alongside other specialized error enums in the
/// error module, such as `StrategyError`, `OptionsError`, and others.
#[derive(Error, Debug)]
pub enum OperationErrorKind {
    /// Operation not supported for this strategy or context
    ///
    /// Used when an operation cannot be performed due to incompatibility with
    /// the target strategy type or other contextual constraints.
    #[error("Operation '{operation}' is not supported for strategy '{reason}'")]
    NotSupported {
        /// The name of the operation that was attempted
        operation: String,
        /// A detailed explanation of why the operation is not supported
        reason: String,
    },

    /// Invalid parameters for operation
    ///
    /// Used when the input parameters for an operation are invalid, insufficient,
    /// or incompatible with the requirements of the operation.
    #[error("Invalid parameters for operation '{operation}': {reason}")]
    InvalidParameters {
        /// The name of the operation that failed
        operation: String,
        /// A detailed explanation of why the parameters are invalid
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_error_not_supported_display() {
        let error = OperationErrorKind::NotSupported {
            operation: "calculate_profit".to_string(),
            reason: "IronCondor".to_string(),
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
            reason: "IronCondor".to_string(),
        };

        assert_eq!(
            format!("{error:?}"),
            "NotSupported { operation: \"calculate_profit\", reason: \"IronCondor\" }"
        );
    }

    #[test]
    fn test_operation_error_as_error() {
        let error = OperationErrorKind::InvalidParameters {
            operation: "validate_strikes".to_string(),
            reason: "Strike prices must be positive".to_string(),
        };

        let error_ref: &dyn std::error::Error = &error;
        assert_eq!(
            error_ref.to_string(),
            "Invalid parameters for operation 'validate_strikes': Strike prices must be positive"
        );
    }

    #[test]
    fn test_operation_error_kinds_distinct() {
        let error1 = OperationErrorKind::NotSupported {
            operation: "op".to_string(),
            reason: "strat".to_string(),
        };

        let error2 = OperationErrorKind::InvalidParameters {
            operation: "op".to_string(),
            reason: "err".to_string(),
        };

        assert_ne!(format!("{error1:?}"), format!("{:?}", error2));
    }

    #[test]
    fn test_operation_error_empty_strings() {
        let error = OperationErrorKind::NotSupported {
            operation: "".to_string(),
            reason: "".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Operation '' is not supported for strategy ''"
        );
    }
}
