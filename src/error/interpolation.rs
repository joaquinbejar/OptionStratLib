/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
use crate::error::{CurveError, PositionError, SurfaceError};
use std::error::Error;

/// Represents errors that can occur during different interpolation operations.
///
/// This enum encapsulates various types of errors that might be encountered when
/// performing numerical interpolation across different methods. Each variant
/// corresponds to a specific interpolation technique and contains a descriptive
/// error message.
///
/// Interpolation errors typically arise from invalid input data, mathematical
/// constraints, or numerical stability issues specific to each interpolation method.
#[derive(Debug)]
pub enum InterpolationError {
    /// Errors that occur during linear interpolation.
    ///
    /// These may include insufficient data points, non-monotonic input values,
    /// or out-of-bounds interpolation attempts.
    Linear(String),

    /// Errors that occur during bilinear interpolation.
    ///
    /// These may include insufficient grid points, irregularly spaced grid,
    /// or extrapolation beyond the defined grid boundaries.
    Bilinear(String),

    /// Errors that occur during cubic interpolation.
    ///
    /// These may include insufficient data points for cubic polynomial fitting,
    /// oscillation issues, or numerical instability.
    Cubic(String),

    /// Errors that occur during spline interpolation.
    ///
    /// These may include insufficient data points for spline construction,
    /// boundary condition issues, or knot placement problems.
    Spline(String),

    /// Standard errors not specific to a particular interpolation method.
    ///
    /// These represent general errors that may occur during interpolation
    /// operations, such as memory allocation failures or system errors.
    StdError(String),
}

impl Error for InterpolationError {}

impl std::fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpolationError::Linear(msg) => write!(f, "Linear interpolation error: {msg}"),
            InterpolationError::Bilinear(msg) => write!(f, "Bilinear interpolation error: {msg}"),
            InterpolationError::Cubic(msg) => write!(f, "Cubic interpolation error: {msg}"),
            InterpolationError::Spline(msg) => write!(f, "Spline interpolation error: {msg}"),
            InterpolationError::StdError(msg) => write!(f, "Standard error: {msg}"),
        }
    }
}

impl From<PositionError> for InterpolationError {
    fn from(err: PositionError) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

impl From<CurveError> for InterpolationError {
    fn from(err: CurveError) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

impl From<SurfaceError> for InterpolationError {
    fn from(err: SurfaceError) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

impl From<Box<dyn Error>> for InterpolationError {
    fn from(err: Box<dyn Error>) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use std::error::Error as StdError;
    use std::fmt;

    // Mock errors for testing the From implementations
    #[derive(Debug)]
    struct MockError(String);

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl StdError for MockError {}

    // Mock implementations of the error types used in From implementations
    #[derive(Debug)]
    struct MockPositionError;

    impl fmt::Display for MockPositionError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Mock position error")
        }
    }

    impl StdError for MockPositionError {}

    #[derive(Debug)]
    struct MockCurveError;

    impl fmt::Display for MockCurveError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Mock curve error")
        }
    }

    impl StdError for MockCurveError {}

    #[derive(Debug)]
    struct MockSurfaceError;

    impl fmt::Display for MockSurfaceError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Mock surface error")
        }
    }

    impl StdError for MockSurfaceError {}

    impl From<MockSurfaceError> for SurfaceError {
        fn from(err: MockSurfaceError) -> Self {
            SurfaceError::AnalysisError(err.to_string())
        }
    }

    // Mock implementation of the From trait for testing
    impl From<MockCurveError> for CurveError {
        fn from(err: MockCurveError) -> Self {
            // The actual implementation doesn't matter for this test
            CurveError::AnalysisError(err.to_string())
        }
    }

    // Mock implementation of the From trait for testing
    impl From<MockPositionError> for PositionError {
        fn from(err: MockPositionError) -> Self {
            // The actual implementation doesn't matter for this test
            PositionError::ValidationError(PositionValidationErrorKind::InvalidPosition {
                reason: err.to_string(),
            })
        }
    }

    #[test]
    fn test_create_interpolation_errors() {
        // Test creating instances of each variant
        let linear_err = InterpolationError::Linear("insufficient data points".to_string());
        let bilinear_err = InterpolationError::Bilinear("out of grid boundary".to_string());
        let cubic_err = InterpolationError::Cubic("numerical instability".to_string());
        let spline_err = InterpolationError::Spline("invalid knot placement".to_string());
        let std_err = InterpolationError::StdError("general error".to_string());

        // Verify the variants are created correctly
        match linear_err {
            InterpolationError::Linear(msg) => assert_eq!(msg, "insufficient data points"),
            _ => panic!("Expected Linear variant"),
        }

        match bilinear_err {
            InterpolationError::Bilinear(msg) => assert_eq!(msg, "out of grid boundary"),
            _ => panic!("Expected Bilinear variant"),
        }

        match cubic_err {
            InterpolationError::Cubic(msg) => assert_eq!(msg, "numerical instability"),
            _ => panic!("Expected Cubic variant"),
        }

        match spline_err {
            InterpolationError::Spline(msg) => assert_eq!(msg, "invalid knot placement"),
            _ => panic!("Expected Spline variant"),
        }

        match std_err {
            InterpolationError::StdError(msg) => assert_eq!(msg, "general error"),
            _ => panic!("Expected StdError variant"),
        }
    }

    #[test]
    fn test_display_implementation() {
        // Test that the Display implementation formats messages correctly
        let linear_err = InterpolationError::Linear("test error".to_string());
        let bilinear_err = InterpolationError::Bilinear("test error".to_string());
        let cubic_err = InterpolationError::Cubic("test error".to_string());
        let spline_err = InterpolationError::Spline("test error".to_string());
        let std_err = InterpolationError::StdError("test error".to_string());

        assert_eq!(
            format!("{linear_err}"),
            "Linear interpolation error: test error"
        );
        assert_eq!(
            format!("{bilinear_err}"),
            "Bilinear interpolation error: test error"
        );
        assert_eq!(
            format!("{cubic_err}"),
            "Cubic interpolation error: test error"
        );
        assert_eq!(
            format!("{spline_err}"),
            "Spline interpolation error: test error"
        );
        assert_eq!(format!("{std_err}"), "Standard error: test error");
    }

    #[test]
    fn test_from_box_dyn_error() {
        // Create a Box<dyn Error> and convert it to an InterpolationError
        let mock_error = Box::new(MockError("boxed error".to_string())) as Box<dyn StdError>;

        let interpolation_err = InterpolationError::from(mock_error);

        match interpolation_err {
            InterpolationError::StdError(msg) => assert!(msg.contains("boxed error")),
            _ => panic!("Expected StdError variant"),
        }
    }

    #[test]
    fn test_conversion_from_position_error() {
        let position_err = MockPositionError;
        let position_err = PositionError::from(position_err);
        let interpolation_err = InterpolationError::from(position_err);

        match interpolation_err {
            InterpolationError::StdError(msg) => {
                assert!(
                    msg.contains("position"),
                    "Error message should contain 'position'"
                );
            }
            _ => panic!("Expected StdError variant"),
        }
    }

    #[test]
    fn test_conversion_from_curve_error() {
        let curve_err = MockCurveError;
        let curve_err = CurveError::from(curve_err);
        let interpolation_err = InterpolationError::from(curve_err);
        match interpolation_err {
            InterpolationError::StdError(msg) => {
                assert!(
                    msg.contains("curve"),
                    "Error message should contain reference to 'curve'"
                );
            }
            _ => panic!("Expected StdError variant"),
        }
    }

    #[test]
    fn test_conversion_from_surface_error() {
        let surface_err = MockSurfaceError;
        let surface_err = SurfaceError::from(surface_err);
        let interpolation_err = InterpolationError::from(surface_err);

        match interpolation_err {
            InterpolationError::StdError(msg) => {
                assert!(
                    msg.contains("surface"),
                    "Error message should contain reference to 'surface'"
                );
            }
            _ => panic!("Expected StdError variant"),
        }
    }

    #[test]
    fn test_debug_implementation() {
        let err = InterpolationError::Linear("test debug".to_string());
        let debug_str = format!("{err:?}");
        assert!(
            debug_str.contains("Linear"),
            "Debug representation should include the variant"
        );
        assert!(
            debug_str.contains("test debug"),
            "Debug representation should include the error message"
        );
    }
}
