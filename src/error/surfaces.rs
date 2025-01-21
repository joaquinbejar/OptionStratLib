/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use std::error::Error;
use std::fmt;
use crate::error::{OperationErrorKind, PositionError};

#[derive(Debug)]
pub enum SurfaceError {
    Point3DError { reason: &'static str },
    OperationError(OperationErrorKind),
    StdError { reason: String },
    ConstructionError(String),
    AnalysisError(String),
}

/// Provides helper methods for constructing specific variants of the `SurfaceError` type.
///
/// These methods encapsulate common patterns of error creation, making it easier
/// to consistently generate errors with the necessary context.
///
///
/// ## Integration
/// - These methods simplify the process of creating meaningful error objects, improving readability
///   and maintainability of the code using the `SurfaceError` type.
/// - The constructed errors leverage the [`OperationErrorKind`]
///   to ensure structured and detailed error categorization.
impl SurfaceError {
    /// ### `operation_not_supported`
    /// Constructs a `SurfaceError::OperationError` with an [`OperationErrorKind::NotSupported`] variant.
    /// - **Parameters:**
    ///   - `operation` (`&str`): The name of the operation that is not supported.
    ///   - `reason` (`&str`): A description of why the operation is not supported.
    /// - **Returns:**
    ///   - A `SurfaceError` containing a `NotSupported` operation error.
    /// - **Use Cases:**
    ///   - Invoked when a requested operation is not compatible with the current context.
    ///   - For example, attempting an unsupported computation method on a specific curve type.
    ///
    pub fn operation_not_supported(operation: &str, reason: &str) -> Self {
        SurfaceError::OperationError(OperationErrorKind::NotSupported {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }

    /// ### `invalid_parameters`
    /// Constructs a `SurfaceError::OperationError` with an [`OperationErrorKind::InvalidParameters`] variant.
    /// - **Parameters:**
    ///   - `operation` (`&str`): The name of the operation that encountered invalid parameters.
    ///   - `reason` (`&str`): A description of why the parameters are invalid.
    /// - **Returns:**
    ///   - A `SurfaceError` containing an `InvalidParameters` operation error.
    /// - **Use Cases:**
    ///   - Used when an operation fails due to issues with the provided input.
    ///   - For example, providing malformed or missing parameters for interpolation or curve construction.
    ///
    pub fn invalid_parameters(operation: &str, reason: &str) -> Self {
        SurfaceError::OperationError(OperationErrorKind::InvalidParameters {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SurfaceError::OperationError(err) => write!(f, "Operation error: {}", err),
            SurfaceError::StdError { reason } => write!(f, "Error: {}", reason),
            SurfaceError::Point3DError { reason } => write!(f, "Error: {}", reason),
            SurfaceError::ConstructionError(reason) => write!(f, "Construction error: {}", reason),
            SurfaceError::AnalysisError(reason) => write!(f, "Analysis error: {}", reason),
        }
    }
}

pub type CurvesResult<T> = Result<T, SurfaceError>;

/// Converts a `PositionError` into a `SurfaceError` by mapping it to an
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
///    implementation converts it into a `SurfaceError` for consistent error
///    handling within the `curves` module.
/// 2. The generated `SurfaceError` provides detailed diagnostic information
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
/// - Such errors need to be mapped into the `SurfaceError` domain to maintain
///   consistent error handling across the library.
///
/// ## Debugging:
/// The resulting `SurfaceError` will include contextual details, making it
/// straightforward to trace and debug the underlying issue.
impl From<PositionError> for SurfaceError {
    fn from(err: PositionError) -> Self {
        SurfaceError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Position".to_string(),
            reason: err.to_string(),
        })
    }
}

/// Implements the `From` trait to enable seamless conversion from a boxed `dyn Error`
/// into a `SurfaceError`. This is particularly useful for integrating standard error
/// handling mechanisms with the custom `SurfaceError` type.
///
/// # Behavior
///
/// When constructing a `SurfaceError` from a `Box<dyn Error>`, the `StdError` variant
/// is utilized. The `Box<dyn Error>` is unwrapped, and its string representation
/// (via `to_string`) is used to populate the `reason` field of the `StdError` variant.
///
/// # Parameters
///
/// - `err`: A boxed standard error (`Box<dyn Error>`). Represents the error to be
///   wrapped within a `SurfaceError` variant.
///
/// # Returns
///
/// - `SurfaceError::StdError`: The custom error type with a detailed `reason`
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
/// Instead of handling standard errors separately, you can propagate them as `SurfaceError`
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
/// This conversion is provided in the `curves` module, which defines
/// the `SurfaceError` enum encompassing multiple errors related to curve operations.
impl From<Box<dyn Error>> for SurfaceError {
    fn from(err: Box<dyn Error>) -> Self {
        SurfaceError::StdError {
            reason: err.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt;
    use crate::error::position::PositionValidationErrorKind;

    // Custom error type for testing From<Box<dyn Error>>
    #[derive(Debug)]
    struct TestError {
        message: String,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl Error for TestError {}

    #[test]
    fn test_operation_not_supported() {
        let error = SurfaceError::operation_not_supported(
            "test_operation",
            "Operation cannot be performed",
        );

        match error {
            SurfaceError::OperationError(OperationErrorKind::NotSupported { operation, reason }) => {
                assert_eq!(operation, "test_operation");
                assert_eq!(reason, "Operation cannot be performed");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_invalid_parameters() {
        let error = SurfaceError::invalid_parameters(
            "test_params",
            "Invalid input parameters",
        );

        match error {
            SurfaceError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }) => {
                assert_eq!(operation, "test_params");
                assert_eq!(reason, "Invalid input parameters");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_display_operation_error() {
        let error = SurfaceError::operation_not_supported(
            "test_operation",
            "Operation cannot be performed",
        );
        assert!(error.to_string().contains("Operation error"));
        assert!(error.to_string().contains("test_operation"));
        assert!(error.to_string().contains("Operation cannot be performed"));
    }

    #[test]
    fn test_display_std_error() {
        let error = SurfaceError::StdError {
            reason: "Standard error test".to_string(),
        };
        assert_eq!(error.to_string(), "Error: Standard error test");
    }

    #[test]
    fn test_display_point3d_error() {
        let error = SurfaceError::Point3DError {
            reason: "Point error test",
        };
        assert_eq!(error.to_string(), "Error: Point error test");
    }

    #[test]
    fn test_display_construction_error() {
        let error = SurfaceError::ConstructionError("Construction failed".to_string());
        assert_eq!(error.to_string(), "Construction error: Construction failed");
    }

    #[test]
    fn test_display_analysis_error() {
        let error = SurfaceError::AnalysisError("Analysis failed".to_string());
        assert_eq!(error.to_string(), "Analysis error: Analysis failed");
    }

    #[test]
    fn test_from_position_error() {
        let pos_error = PositionError::ValidationError(
            PositionValidationErrorKind::InvalidPosition {
                reason: "Test position error".to_string(),
            }
        );
        let surface_error = SurfaceError::from(pos_error);

        match surface_error {
            SurfaceError::OperationError(OperationErrorKind::InvalidParameters { operation, .. }) => {
                assert_eq!(operation, "Position");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_from_box_dyn_error() {
        let test_error = TestError {
            message: "Test box error".to_string(),
        };
        let boxed_error: Box<dyn Error> = Box::new(test_error);
        let surface_error = SurfaceError::from(boxed_error);

        match surface_error {
            SurfaceError::StdError { reason } => {
                assert_eq!(reason, "Test box error");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_curves_result_ok() {
        let result: CurvesResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_curves_result_err() {
        let result: CurvesResult<i32> = Err(SurfaceError::AnalysisError("Test error".to_string()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Analysis error: Test error");
    }
}