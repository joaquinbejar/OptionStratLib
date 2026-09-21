use optionstratlib::error::{
    CurveError, GraphError, GreeksError, InterpolationError, MetricsError, OperationErrorKind,
    OptionsError,
};

#[test]
fn test_curve_error_from_options_error() {
    // Create an OptionsError
    let options_error = OptionsError::validation_error("field", "options error test");

    // Convert to CurveError using From trait
    let curve_error: CurveError = options_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::Options(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected Options variant, got something else"),
    }
}

#[test]
fn test_curve_error_does_not_absorb_a_greeks_error() {
    // The math layer no longer carries a pricing payload (#511). Analytics,
    // which owns the projection, maps the Greek failure into the curve error
    // it reports, keeping the reason.
    let greeks_error = GreeksError::invalid_volatility(-1.0, "greeks error test");
    let curve_error = CurveError::MetricsError(format!("delta: {greeks_error}"));

    match curve_error {
        CurveError::MetricsError(reason) => {
            assert!(
                reason.contains("delta"),
                "the failing Greek is named: {reason}"
            );
            assert!(
                reason.contains("greeks error test"),
                "the original reason survives: {reason}"
            );
        }
        other => panic!("expected MetricsError, got {other:?}"),
    }
}

#[test]
fn test_curve_error_from_interpolation_error() {
    // Create an InterpolationError
    let interpolation_error = InterpolationError::Linear("interpolation error test".to_string());

    // Convert to CurveError using From trait
    let curve_error: CurveError = interpolation_error.into();

    // Verify the conversion was successful
    match curve_error {
        CurveError::InterpolationOp(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected InterpolationOp variant, got something else"),
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
fn test_graph_error_wraps_the_curve_error() {
    // The direction is visualization over math, never the reverse (#511).
    let curve_error = CurveError::MetricsError("curve metrics".to_string());
    let graph_error = GraphError::from(curve_error);

    match graph_error {
        GraphError::Curve(_) => {}
        other => panic!("expected GraphError::Curve, got {other:?}"),
    }
}

#[test]
fn test_curve_error_construction_error() {
    // Create a ConstructionError
    let error = CurveError::ConstructionError("construction error test".to_string());

    // Verify the Display implementation
    assert_eq!(
        format!("{error}"),
        "Construction error: construction error test"
    );

    // Verify the Debug implementation
    assert!(format!("{error:?}").contains("construction error test"));
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
        format!("{error}"),
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
        format!("{error}"),
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
    assert_eq!(format!("{error}"), "Error: invalid coordinates");

    // Verify the Debug implementation
    assert!(format!("{error:?}").contains("invalid coordinates"));
}

#[test]
fn test_curve_error_render_error() {
    let error = CurveError::RenderError {
        backend: "plotters",
        reason: "io error test".to_string(),
    };
    match error {
        CurveError::RenderError { backend, reason } => {
            assert_eq!(backend, "plotters");
            assert!(reason.contains("io error test"));
        }
        _ => panic!("Expected RenderError variant, got something else"),
    }
}
