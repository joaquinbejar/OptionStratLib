use optionstratlib::error::{CurveError, MetricsError, SurfaceError};
use std::error::Error;

#[test]
fn test_metrics_error_display() {
    // Test each variant of MetricsError for Display implementation
    let basic_err = MetricsError::BasicError("calculation failed".to_string());
    assert_eq!(format!("{}", basic_err), "Basic Error: calculation failed");

    let shape_err = MetricsError::ShapeError("incompatible dimensions".to_string());
    assert_eq!(
        format!("{}", shape_err),
        "Shape Error: incompatible dimensions"
    );

    let range_err = MetricsError::RangeError("value out of bounds".to_string());
    assert_eq!(format!("{}", range_err), "Range Error: value out of bounds");

    let trend_err = MetricsError::TrendError("trend analysis failed".to_string());
    assert_eq!(
        format!("{}", trend_err),
        "Trend Error: trend analysis failed"
    );

    let risk_err = MetricsError::RiskError("risk calculation error".to_string());
    assert_eq!(
        format!("{}", risk_err),
        "Risk Error: risk calculation error"
    );

    let curve_err = MetricsError::CurveError("curve fitting failed".to_string());
    assert_eq!(
        format!("{}", curve_err),
        "Curve Error: curve fitting failed"
    );

    let surface_err = MetricsError::SurfaceError("surface modeling error".to_string());
    assert_eq!(
        format!("{}", surface_err),
        "Surface Error: surface modeling error"
    );

    let std_err = MetricsError::StdError {
        reason: "standard error occurred".to_string(),
    };
    assert_eq!(
        format!("{}", std_err),
        "Standard Error: standard error occurred"
    );
}

#[test]
fn test_metrics_error_debug() {
    // Test Debug implementation for a few variants
    let basic_err = MetricsError::BasicError("test message".to_string());
    assert!(format!("{:?}", basic_err).contains("BasicError"));
    assert!(format!("{:?}", basic_err).contains("test message"));

    let std_err = MetricsError::StdError {
        reason: "debug test".to_string(),
    };
    assert!(format!("{:?}", std_err).contains("StdError"));
    assert!(format!("{:?}", std_err).contains("debug test"));
}

#[test]
fn test_metrics_error_from_curve_error() {
    // Create a CurveError
    let curve_err = CurveError::Point2DError {
        reason: "invalid point",
    };

    // Convert to MetricsError using From trait
    let metrics_err: MetricsError = curve_err.into();

    // Verify the conversion was successful
    match metrics_err {
        MetricsError::CurveError(msg) => {
            assert!(msg.contains("invalid point"));
        }
        _ => panic!("Expected CurveError variant, got something else"),
    }
}

#[test]
fn test_metrics_error_from_surface_error() {
    // Create a SurfaceError
    let surface_err = SurfaceError::Point3DError {
        reason: "invalid surface point",
    };

    // Convert to MetricsError using From trait
    let metrics_err: MetricsError = surface_err.into();

    // Verify the conversion was successful
    match metrics_err {
        MetricsError::SurfaceError(msg) => {
            assert!(msg.contains("invalid surface point"));
        }
        _ => panic!("Expected SurfaceError variant, got something else"),
    }
}

#[test]
fn test_metrics_error_from_box_dyn_error() {
    // Create a simple error that implements Error trait
    let std_err = std::io::Error::other("IO error occurred");
    let boxed_err: Box<dyn Error> = Box::new(std_err);

    // Convert to MetricsError using From trait
    let metrics_err: MetricsError = boxed_err.into();

    // Verify the conversion was successful
    match metrics_err {
        MetricsError::StdError { reason } => {
            assert!(reason.contains("IO error occurred"));
        }
        _ => panic!("Expected StdError variant, got something else"),
    }
}

#[test]
fn test_metrics_error_as_error() {
    // Create a MetricsError
    let error = MetricsError::BasicError("test error".to_string());

    // Verify it can be used as a Box<dyn Error>
    let boxed_error: Box<dyn Error> = Box::new(error);
    assert_eq!(boxed_error.to_string(), "Basic Error: test error");

    // Test another variant
    let error = MetricsError::StdError {
        reason: "standard error test".to_string(),
    };
    let boxed_error: Box<dyn Error> = Box::new(error);
    assert_eq!(
        boxed_error.to_string(),
        "Standard Error: standard error test"
    );
}
