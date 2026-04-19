use optionstratlib::error::{CurveError, MetricsError, SurfaceError};

#[test]
fn test_metrics_error_display() {
    // Test each variant of MetricsError for Display implementation
    let basic_err = MetricsError::BasicError("calculation failed".to_string());
    assert_eq!(format!("{basic_err}"), "Basic Error: calculation failed");

    let shape_err = MetricsError::ShapeError("incompatible dimensions".to_string());
    assert_eq!(
        format!("{shape_err}"),
        "Shape Error: incompatible dimensions"
    );

    let range_err = MetricsError::RangeError("value out of bounds".to_string());
    assert_eq!(format!("{range_err}"), "Range Error: value out of bounds");

    let trend_err = MetricsError::TrendError("trend analysis failed".to_string());
    assert_eq!(format!("{trend_err}"), "Trend Error: trend analysis failed");

    let risk_err = MetricsError::RiskError("risk calculation error".to_string());
    assert_eq!(format!("{risk_err}"), "Risk Error: risk calculation error");

    let curve_err = MetricsError::Curve(CurveError::Point2DError {
        reason: "curve fitting failed",
    });
    assert!(format!("{curve_err}").contains("curve fitting failed"));

    let surface_err = MetricsError::Surface(SurfaceError::Point3DError {
        reason: "surface modeling error",
    });
    assert!(format!("{surface_err}").contains("surface modeling error"));
}

#[test]
fn test_metrics_error_debug() {
    // Test Debug implementation for a few variants
    let basic_err = MetricsError::BasicError("test message".to_string());
    assert!(format!("{basic_err:?}").contains("BasicError"));
    assert!(format!("{basic_err:?}").contains("test message"));

    let range_err = MetricsError::RangeError("debug test".to_string());
    assert!(format!("{range_err:?}").contains("RangeError"));
    assert!(format!("{range_err:?}").contains("debug test"));
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
        MetricsError::Curve(_) => {
            // Conversion successful
        }
        _ => panic!("Expected Curve variant, got something else"),
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
        MetricsError::Surface(_) => {
            // Conversion successful
        }
        _ => panic!("Expected Surface variant, got something else"),
    }
}

#[test]
fn test_metrics_error_basic_error_display() {
    let error = MetricsError::BasicError("basic calc failed".to_string());
    assert_eq!(format!("{error}"), "Basic Error: basic calc failed");
}

#[test]
fn test_metrics_error_as_error() {
    // Create a MetricsError
    let error = MetricsError::BasicError("test error".to_string());

    // Verify it can be used as a Box<dyn Error>
    let boxed_error: Box<dyn std::error::Error> = Box::new(error);
    assert_eq!(boxed_error.to_string(), "Basic Error: test error");

    // Test another variant
    let error = MetricsError::RangeError("out of range".to_string());
    let boxed_error: Box<dyn std::error::Error> = Box::new(error);
    assert_eq!(boxed_error.to_string(), "Range Error: out of range");
}
