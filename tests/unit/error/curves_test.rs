use optionstratlib::error::{
    CurveError, GraphError, GreeksError, InterpolationError, MetricsError, OperationErrorKind,
    OptionsError,
};
use std::error::Error;

#[test]
fn test_curve_error_from_options_error() {
    // Create an OptionsError
    let options_error = OptionsError::OtherError {
        reason: "options error test".to_string(),
    };

    // Convert to CurveError using From trait
    let curve_error: CurveError = options_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::OperationError(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected OperationError variant, got something else"),
    }
}

#[test]
fn test_curve_error_from_greeks_error() {
    // Create a GreeksError
    let greeks_error = GreeksError::StdError("greeks error test".to_string());

    // Convert to CurveError using From trait
    let curve_error: CurveError = greeks_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::OperationError(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected OperationError variant, got something else"),
    }
}

#[test]
fn test_curve_error_from_interpolation_error() {
    // Create an InterpolationError
    let interpolation_error = InterpolationError::StdError("interpolation error test".to_string());

    // Convert to CurveError using From trait
    let curve_error: CurveError = interpolation_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::StdError { reason } => {
            assert!(reason.contains("interpolation error test"));
        }
        _ => panic!("Expected StdError variant, got something else"),
    }
}

#[test]
fn test_curve_error_from_metrics_error() {
    // Create a MetricsError
    let metrics_error = MetricsError::BasicError("metrics error test".to_string());

    // Convert to CurveError using From trait
    let curve_error: CurveError = metrics_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::MetricsError(msg) => {
            assert!(msg.contains("metrics error test"));
        }
        _ => panic!("Expected MetricsError variant, got something else"),
    }
}

#[test]
fn test_curve_error_from_graph_error() {
    // Create a GraphError
    let graph_error = GraphError::Render("graph error test".to_string());

    // Convert to CurveError using From trait
    let curve_error: CurveError = graph_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::StdError { reason } => {
            assert!(reason.contains("graph error test"));
        }
        _ => panic!("Expected StdError variant, got something else"),
    }
}

#[test]
fn test_curve_error_construction_error() {
    // Create a ConstructionError
    let error = CurveError::ConstructionError("construction error test".to_string());

    // Verify the Display implementation
    assert_eq!(
        format!("{}", error),
        "Construction error: construction error test"
    );

    // Verify the Debug implementation
    assert!(format!("{:?}", error).contains("construction error test"));
}

#[test]
fn test_curve_error_operation_not_supported() {
    // Test the operation_not_supported constructor method
    let operation = "calculate";
    let reason = "TestStrategy";
    let error = CurveError::operation_not_supported(operation, reason);

    // Verify the error is constructed correctly
    match error {
        CurveError::OperationError(OperationErrorKind::NotSupported {
            operation: ref op,
            reason: ref r,
        }) => {
            assert_eq!(op, operation);
            assert_eq!(r, reason);
        }
        _ => panic!("Expected OperationError with NotSupported variant"),
    }

    // Verify the Display implementation
    assert_eq!(
        format!("{}", error),
        "Operation error: Operation 'calculate' is not supported for strategy 'TestStrategy'"
    );
}

#[test]
fn test_curve_error_invalid_parameters() {
    // Test the invalid_parameters constructor method
    let operation = "interpolate";
    let reason = "insufficient data points";
    let error = CurveError::invalid_parameters(operation, reason);

    // Verify the error is constructed correctly
    match error {
        CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation: ref op,
            reason: ref r,
        }) => {
            assert_eq!(op, operation);
            assert_eq!(r, reason);
        }
        _ => panic!("Expected OperationError with InvalidParameters variant"),
    }

    // Verify the Display implementation
    assert_eq!(
        format!("{}", error),
        "Operation error: Invalid parameters for operation 'interpolate': insufficient data points"
    );
}

#[test]
fn test_curve_error_point2d_error() {
    // Create a Point2DError
    let error = CurveError::Point2DError {
        reason: "invalid coordinates",
    };

    // Verify the Display implementation
    assert_eq!(format!("{}", error), "Error: invalid coordinates");

    // Verify the Debug implementation
    assert!(format!("{:?}", error).contains("invalid coordinates"));
}

#[test]
fn test_curve_error_from_box_dyn_error() {
    // Create a standard error and box it
    let std_error = std::io::Error::other("io error test");
    let boxed_error: Box<dyn Error> = Box::new(std_error);

    // Convert to CurveError
    let curve_error = CurveError::from(boxed_error);

    // Verify the conversion was successful
    match curve_error {
        CurveError::StdError { reason } => {
            assert!(reason.contains("io error test"));
        }
        _ => panic!("Expected StdError variant, got something else"),
    }
}
