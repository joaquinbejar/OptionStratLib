/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com
    Date: 24/12/24
******************************************************************************/

use crate::error::common::OperationErrorKind;
use crate::error::{InterpolationError, PositionError};
use std::error::Error;
use std::fmt;

impl Error for CurvesError {}

/// Represents different types of errors that can occur in the `curves` module.
///
/// This enum provides categorization of errors that may be encountered while using
/// the `curves` module for tasks such as interpolation, data analysis, or curve construction.
///
/// ## Variants
///
/// ### `Point2DError`
/// Indicates an issue related to a 2D point operation.
/// - Fields:
///   - `reason` (`&'static str`): A static string explaining the nature of the error.
///
/// Use this variant to represent fundamental issues related to points, such as missing
/// values or invalid coordinates.
///
///
/// ### `OperationError`
/// Represents a broader category of operational errors, encapsulated by the
/// [`OperationErrorKind`] enum.
/// - Fields:
///   - `OperationErrorKind`: Encapsulates the specific kind of operation failure.
///
///
/// ### `StdError`
/// Wraps a standard error with additional context.
/// - Fields:
///   - `reason` (`String`): A dynamic string explaining the error in detail.
///
/// Leverage this for general-purpose error handling where a static string or specialized
/// variant might not offer enough flexibility or context.
///
///
/// ### `InterpolationError`
/// Reflects issues encountered during a curve's interpolation process.
/// - Fields:
///   - `String`: A human-readable explanation of the problem.
///
/// This variant is commonly used for failures in interpolating data points or generating
/// smooth curves.
///
///
/// ### `ConstructionError`
/// Represents errors encountered during the construction of curves or related structures.
/// - Fields:
///   - `String`: A description of the problem that arose during construction.
///
/// Use this variant when there are issues initializing or creating curve objects, such
/// as missing inputs or unsupported formats.
///
///
/// ### `AnalysisError`
/// Captures errors related to the analysis of curves.
/// - Fields:
///   - `String`: A detailed explanation of the problem during analysis.
///
/// Frequently used for scenarios where analytical methods, such as curve fitting
/// or sampling, fail due to input errors or computational issues.
///
///
/// ## Integration
///
/// - This error type is closely tied to the `curves` module's functionality and is meant
///   to be used wherever curve-related operations (e.g., interpolation, construction, or analysis) might fail.
/// - It is part of a broader error handling system outlined in the `error` module.
///
///
/// ## Debugging
///
/// Errors of this type implement the `Debug` trait to aid in diagnosing issues during
/// development and testing processes. This ensures detailed debug output for better traceability.
///
#[derive(Debug)]
pub enum CurvesError {
    Point2DError { reason: &'static str },
    OperationError(OperationErrorKind),
    StdError { reason: String },
    ConstructionError(String),
    AnalysisError(String),
}

/// Provides helper methods for constructing specific variants of the `CurvesError` type.
///
/// These methods encapsulate common patterns of error creation, making it easier
/// to consistently generate errors with the necessary context.
///
///
/// ## Integration
/// - These methods simplify the process of creating meaningful error objects, improving readability
///   and maintainability of the code using the `CurvesError` type.
/// - The constructed errors leverage the [`OperationErrorKind`]
///   to ensure structured and detailed error categorization.
impl CurvesError {
    /// ### `operation_not_supported`
    /// Constructs a `CurvesError::OperationError` with an [`OperationErrorKind::NotSupported`] variant.
    /// - **Parameters:**
    ///   - `operation` (`&str`): The name of the operation that is not supported.
    ///   - `reason` (`&str`): A description of why the operation is not supported.
    /// - **Returns:**
    ///   - A `CurvesError` containing a `NotSupported` operation error.
    /// - **Use Cases:**
    ///   - Invoked when a requested operation is not compatible with the current context.
    ///   - For example, attempting an unsupported computation method on a specific curve type.
    ///
    pub fn operation_not_supported(operation: &str, reason: &str) -> Self {
        CurvesError::OperationError(OperationErrorKind::NotSupported {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }

    /// ### `invalid_parameters`
    /// Constructs a `CurvesError::OperationError` with an [`OperationErrorKind::InvalidParameters`] variant.
    /// - **Parameters:**
    ///   - `operation` (`&str`): The name of the operation that encountered invalid parameters.
    ///   - `reason` (`&str`): A description of why the parameters are invalid.
    /// - **Returns:**
    ///   - A `CurvesError` containing an `InvalidParameters` operation error.
    /// - **Use Cases:**
    ///   - Used when an operation fails due to issues with the provided input.
    ///   - For example, providing malformed or missing parameters for interpolation or curve construction.
    ///
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
            CurvesError::Point2DError { reason } => write!(f, "Error: {}", reason),
            CurvesError::ConstructionError(reason) => write!(f, "Construction error: {}", reason),
            CurvesError::AnalysisError(reason) => write!(f, "Analysis error: {}", reason),
        }
    }
}

pub type CurvesResult<T> = Result<T, CurvesError>;

/// Converts a `PositionError` into a `CurvesError` by mapping it to an
/// `OperationError` with the `InvalidParameters` variant.
///
/// This implementation ensures a smooth transition between error types
/// when a `PositionError` is encountered within a context that operates
/// on the `curves` module. The `InvalidParameters` variant is used to
/// provide detailed information about the failed operation and the reason
/// for its failure.
///
/// ## Details:
/// - The `operation` field is hardcoded as `"Position"` to indicate the
///   context of the error (i.e., relating to position management).
/// - The `reason` field is derived from the `to_string` representation of
///   the `PositionError`, ensuring a human-readable explanation.
///
/// ## Example Integration:
/// 1. If a `PositionError` is encountered during curve calculations, this
///    implementation converts it into a `CurvesError` for consistent error
///    handling within the `curves` module.
/// 2. The generated `CurvesError` provides detailed diagnostic information
///    about the reason for the failure, enabling effective debugging.
///
/// ## Implementation Notes:
/// - This conversion leverages the `OperationErrorKind::InvalidParameters`
///   variant to communicate that invalid parameters (or settings) were the
///   root cause of failure.
/// - Use this implementation to handle interoperability between error types
///   in modular design contexts.
///
/// ## Example Use Case:
/// This conversion is frequently used in scenarios where:
/// - A position-related error (e.g., from validation or limits) occurs during a
///   curve operation.
/// - Such errors need to be mapped into the `CurvesError` domain to maintain
///   consistent error handling across the library.
///
/// ## Debugging:
/// The resulting `CurvesError` will include contextual details, making it
/// straightforward to trace and debug the underlying issue.
impl From<PositionError> for CurvesError {
    fn from(err: PositionError) -> Self {
        CurvesError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Position".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<InterpolationError> for CurvesError {
    fn from(err: InterpolationError) -> Self {
        CurvesError::StdError {
            reason: err.to_string(),
        }
    }
}

/// Implements the `From` trait to enable seamless conversion from a boxed `dyn Error`
/// into a `CurvesError`. This is particularly useful for integrating standard error
/// handling mechanisms with the custom `CurvesError` type.
///
/// # Behavior
///
/// When constructing a `CurvesError` from a `Box<dyn Error>`, the `StdError` variant
/// is utilized. The `Box<dyn Error>` is unwrapped, and its string representation
/// (via `to_string`) is used to populate the `reason` field of the `StdError` variant.
///
/// # Parameters
///
/// - `err`: A boxed standard error (`Box<dyn Error>`). Represents the error to be
///   wrapped within a `CurvesError` variant.
///
/// # Returns
///
/// - `CurvesError::StdError`: The custom error type with a detailed `reason`
///   string derived from the provided error.
///
/// # Usage
///
/// This implementation is commonly employed when you need to bridge standard Rust
/// errors with the specific error handling system provided by the `curves` module.
/// It facilitates scenarios where standard error contexts need to be preserved
/// in a flexible, string-based `reason` for debugging or logging purposes.
///
/// # Example Scenario
///
/// Instead of handling standard errors separately, you can propagate them as `CurvesError`
/// within the larger error system of the `curves` module, ensuring consistent error
/// wrapping and management.
///
/// # Notes
///
/// - This implementation assumes that all input errors (`Box<dyn Error>`) are stringifiable
///   using the `to_string()` method.
/// - This conversion is particularly useful for libraries integrating generalized errors
///   (e.g., I/O errors, or third-party library errors) into a standardized error system.
///
/// # Module Context
///
/// This conversion is provided in the `crate::error::curves` module, which defines
/// the `CurvesError` enum encompassing multiple errors related to curve operations.
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
    use std::error::Error;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_curves_error_display() {
        let error = CurvesError::Point2DError {
            reason: "Invalid coordinates",
        };
        assert_eq!(error.to_string(), "Error: Invalid coordinates");

        let error = CurvesError::StdError {
            reason: "Standard error".to_string(),
        };
        assert_eq!(error.to_string(), "Error: Standard error");

        let error = CurvesError::operation_not_supported("calculate", "Strategy");
        assert_eq!(
            error.to_string(),
            "Operation error: Operation 'calculate' is not supported for strategy 'Strategy'"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_operation_not_supported() {
        let error = CurvesError::operation_not_supported("test_op", "TestStrat");
        match error {
            CurvesError::OperationError(OperationErrorKind::NotSupported {
                operation,
                reason: strategy_type,
            }) => {
                assert_eq!(operation, "test_op");
                assert_eq!(strategy_type, "TestStrat");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_parameters() {
        let error = CurvesError::invalid_parameters("test_op", "invalid input");
        match error {
            CurvesError::OperationError(OperationErrorKind::InvalidParameters {
                operation,
                reason,
            }) => {
                assert_eq!(operation, "test_op");
                assert_eq!(reason, "invalid input");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_trait_implementation() {
        let error = CurvesError::Point2DError {
            reason: "test error",
        };
        let error_ref: &dyn Error = &error;
        assert_eq!(error_ref.to_string(), "Error: test error");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_box_dyn_error() {
        let boxed_error: Box<dyn Error> =
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, "io error"));
        let curves_error = CurvesError::from(boxed_error);
        match curves_error {
            CurvesError::StdError { reason } => assert_eq!(reason, "io error"),
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_position_error() {
        let position_error = PositionError::unsupported_operation("TestStruct", "test_op");
        let curves_error = CurvesError::from(position_error);

        match curves_error {
            CurvesError::OperationError(OperationErrorKind::InvalidParameters {
                operation,
                reason,
            }) => {
                assert_eq!(operation, "Position");
                assert!(reason.contains("test_op"));
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_debug_implementation() {
        let error = CurvesError::Point2DError {
            reason: "test debug",
        };
        assert!(format!("{:?}", error).contains("test debug"));

        let error = CurvesError::StdError {
            reason: "test debug".to_string(),
        };
        assert!(format!("{:?}", error).contains("test debug"));
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_curves_error_construction_error() {
        let error =
            CurvesError::ConstructionError("Invalid curve construction parameters".to_string());
        assert_eq!(
            format!("{}", error),
            "Construction error: Invalid curve construction parameters"
        );
    }

    #[test]
    fn test_curves_error_analysis_error() {
        let error =
            CurvesError::AnalysisError("Analysis failed due to insufficient data".to_string());
        assert_eq!(
            format!("{}", error),
            "Analysis error: Analysis failed due to insufficient data"
        );
    }
}
