/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com
    Date: 24/12/24
******************************************************************************/

use crate::error::common::OperationErrorKind;
use crate::error::metrics::MetricsError;
use crate::error::{GreeksError, InterpolationError, OptionsError, PositionError};
use std::error::Error;
use std::fmt;

impl Error for CurveError {}

/// Represents different types of errors that can occur in the `curves` module.
///
/// This enum categorizes errors that may be encountered when working with curve-related
/// operations such as interpolation, construction, analysis, and other mathematical
/// operations on curves and points.
///
/// # Variants
///
/// ## `Point2DError`
/// Represents errors related to 2D point operations.
///
/// * `reason` - A static string explaining the specific point-related issue.
///
/// This variant is used for fundamental issues with points like invalid coordinates,
/// missing values, or formatting problems.
///
/// ## `OperationError`
/// Encapsulates general operational errors.
///
/// * `OperationErrorKind` - The specific kind of operation failure (see `OperationErrorKind` enum).
///
/// Used when an operation fails due to unsupported features or invalid parameters.
///
/// ## `StdError`
/// Wraps standard errors with additional context.
///
/// * `reason` - A dynamic string providing detailed error information.
///
/// Suitable for general error cases where specialized variants don't apply.
///
/// ## `InterpolationError`
/// Indicates issues during the curve interpolation process.
///
/// * `String` - A human-readable explanation of the interpolation failure.
///
/// Used when problems occur during data point interpolation or curve generation.
///
/// ## `ConstructionError`
/// Represents errors during the construction of curves or related structures.
///
/// * `String` - A description of the construction issue.
///
/// Applicable when curve initialization fails due to invalid inputs, unsupported
/// configurations, or missing required parameters.
///
/// ## `AnalysisError`
/// Captures errors related to curve analysis operations.
///
/// * `String` - A detailed explanation of the analysis failure.
///
/// Used for failures in analytical methods like curve fitting, differentiation,
/// or other mathematical operations on curves.
///
/// ## `MetricsError`
/// Represents errors when calculating or processing curve metrics.
///
/// * `String` - An explanation of the metrics-related issue.
///
/// Used when metric calculations fail due to invalid inputs or computational issues.
///
/// # Usage
///
/// This error type is designed to be used throughout the `curves` module wherever
/// operations might fail. It provides structured error information to help diagnose
/// and handle various failure scenarios.
///
/// # Implementation Notes
///
/// The error variants are designed to provide useful context for debugging and error handling.
/// Each variant includes specific information relevant to its error category.
///
/// # Examples
///
/// ```rust
/// // Example of creating a construction error
/// use optionstratlib::error::CurveError;
/// let error = CurveError::ConstructionError("Insufficient points to construct curve".to_string());
///
/// // Example of creating a point error
/// let point_error = CurveError::Point2DError { reason: "Point coordinates out of bounds" };
/// ```
#[derive(Debug)]
pub enum CurveError {
    /// Error related to 2D point operations
    Point2DError {
        /// Static description of the point-related issue
        reason: &'static str,
    },

    /// General operational error
    OperationError(
        /// The specific kind of operation failure
        OperationErrorKind,
    ),

    /// Standard error with additional context
    StdError {
        /// Detailed explanation of the error
        reason: String,
    },

    /// Error during curve interpolation
    InterpolationError(
        /// Description of the interpolation issue
        String,
    ),

    /// Error during curve or structure construction
    ConstructionError(
        /// Details about the construction failure
        String,
    ),

    /// Error during curve analysis operations
    AnalysisError(
        /// Explanation of the analysis issue
        String,
    ),

    /// Error when calculating or processing curve metrics
    MetricsError(
        /// Description of the metrics-related issue
        String,
    ),
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
impl CurveError {
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
        CurveError::OperationError(OperationErrorKind::NotSupported {
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
        CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl fmt::Display for CurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurveError::OperationError(err) => write!(f, "Operation error: {}", err),
            CurveError::StdError { reason } => write!(f, "Error: {}", reason),
            CurveError::Point2DError { reason } => write!(f, "Error: {}", reason),
            CurveError::ConstructionError(reason) => write!(f, "Construction error: {}", reason),
            CurveError::AnalysisError(reason) => write!(f, "Analysis error: {}", reason),
            CurveError::MetricsError(reason) => write!(f, "Metrics error: {}", reason),
            CurveError::InterpolationError(reason) => write!(f, "Interpolation error: {}", reason),
        }
    }
}

/// Type alias representing the result of operations related to curve calculations.
///
/// This type alias provides a standardized result type for functions that perform operations
/// with mathematical curves, including interpolation, construction, analysis, and other
/// curve-related operations.
///
/// # Type Parameters
///
/// * `T` - The success value type returned when operations complete successfully.
///
/// # Return Value
///
/// Returns either:
/// * `Ok(T)` - The operation completed successfully with a value of type `T`.
/// * `Err(CurveError)` - The operation failed, with a [`CurveError`] describing the specific failure.
///
/// # Usage
///
/// This result type is used throughout the curves module to provide consistent error handling
/// for curve operations. It allows functions to return detailed error information using the
/// [`CurveError`] enum when operations fail, while returning the expected value when successful.
///
pub type CurvesResult<T> = Result<T, CurveError>;

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
impl From<PositionError> for CurveError {
    fn from(err: PositionError) -> Self {
        CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Position".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<OptionsError> for CurveError {
    fn from(err: OptionsError) -> Self {
        CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Option".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<GreeksError> for CurveError {
    fn from(err: GreeksError) -> Self {
        CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Greeks".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<InterpolationError> for CurveError {
    fn from(err: InterpolationError) -> Self {
        CurveError::StdError {
            reason: err.to_string(),
        }
    }
}

impl From<MetricsError> for CurveError {
    fn from(err: MetricsError) -> Self {
        CurveError::MetricsError(err.to_string())
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
impl From<Box<dyn Error>> for CurveError {
    fn from(err: Box<dyn Error>) -> Self {
        CurveError::StdError {
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
        let error = CurveError::Point2DError {
            reason: "Invalid coordinates",
        };
        assert_eq!(error.to_string(), "Error: Invalid coordinates");

        let error = CurveError::StdError {
            reason: "Standard error".to_string(),
        };
        assert_eq!(error.to_string(), "Error: Standard error");

        let error = CurveError::operation_not_supported("calculate", "Strategy");
        assert_eq!(
            error.to_string(),
            "Operation error: Operation 'calculate' is not supported for strategy 'Strategy'"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_operation_not_supported() {
        let error = CurveError::operation_not_supported("test_op", "TestStrat");
        match error {
            CurveError::OperationError(OperationErrorKind::NotSupported {
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
        let error = CurveError::invalid_parameters("test_op", "invalid input");
        match error {
            CurveError::OperationError(OperationErrorKind::InvalidParameters {
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
        let error = CurveError::Point2DError {
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
        let curves_error = CurveError::from(boxed_error);
        match curves_error {
            CurveError::StdError { reason } => assert_eq!(reason, "io error"),
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_position_error() {
        let position_error = PositionError::unsupported_operation("TestStruct", "test_op");
        let curves_error = CurveError::from(position_error);

        match curves_error {
            CurveError::OperationError(OperationErrorKind::InvalidParameters {
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
        let error = CurveError::Point2DError {
            reason: "test debug",
        };
        assert!(format!("{:?}", error).contains("test debug"));

        let error = CurveError::StdError {
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
            CurveError::ConstructionError("Invalid curve construction parameters".to_string());
        assert_eq!(
            format!("{}", error),
            "Construction error: Invalid curve construction parameters"
        );
    }

    #[test]
    fn test_curves_error_analysis_error() {
        let error =
            CurveError::AnalysisError("Analysis failed due to insufficient data".to_string());
        assert_eq!(
            format!("{}", error),
            "Analysis error: Analysis failed due to insufficient data"
        );
    }
}
