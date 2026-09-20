use optionstratlib::error::position::PositionValidationErrorKind;
use optionstratlib::error::{
    GraphError, GreeksError, InterpolationError, OptionsError, PositionError, SurfaceError,
};
use std::error::Error;

// MockGraphError struct removed as it was unused.
#[test]
fn test_from_interpolation_error() {
    // Create an interpolation error
    let interpolation_error = InterpolationError::Linear("interpolation error test".to_string());

    // Convert to SurfaceError
    let surface_error = SurfaceError::from(interpolation_error);

    // Verify the conversion was successful
    match surface_error {
        SurfaceError::AnalysisError(reason) => {
            assert!(reason.contains("interpolation error test"));
        }
        _ => panic!("Expected AnalysisError variant, got something else"),
    }
}

#[test]
fn test_from_options_error() {
    // Create an options error
    let options_error = OptionsError::validation_error("field", "options error test");

    // Convert to SurfaceError
    let surface_error = SurfaceError::from(options_error);

    // Verify the conversion was successful
    match surface_error {
        SurfaceError::Options(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected Options variant, got something else"),
    }
}

#[test]
fn test_surface_error_does_not_absorb_a_greeks_error() {
    // As for curves: the surface error carries the reason, not the pricing
    // error type (#511).
    let greeks_error = GreeksError::invalid_volatility(-1.0, "greeks error test");
    let surface_error = SurfaceError::AnalysisError(format!("delta: {greeks_error}"));

    match surface_error {
        SurfaceError::AnalysisError(reason) => {
            assert!(
                reason.contains("delta"),
                "the failing Greek is named: {reason}"
            );
            assert!(
                reason.contains("greeks error test"),
                "the original reason survives: {reason}"
            );
        }
        other => panic!("expected AnalysisError, got {other:?}"),
    }
}

#[test]
fn test_graph_error_keeps_its_own_direction() {
    // A rendering failure is visualization's, and `GraphError` wraps the
    // math error, never the other way round (#511).
    let surface_error = SurfaceError::AnalysisError("surface analysis".to_string());
    let graph_error = GraphError::from(surface_error);

    match graph_error {
        GraphError::Surface(_) => {}
        other => panic!("expected GraphError::Surface, got {other:?}"),
    }
}

#[test]
fn test_surface_error_debug() {
    // Test Debug implementation for each variant
    let point_error = SurfaceError::Point3DError {
        reason: "point debug test",
    };
    let debug_str = format!("{point_error:?}");
    assert!(debug_str.contains("Point3DError"));

    let op_error = SurfaceError::operation_not_supported("debug test", "not supported");
    let debug_str = format!("{op_error:?}");
    assert!(debug_str.contains("OperationError"));

    let render_error = SurfaceError::RenderError {
        backend: "plotters",
        reason: "std debug test".to_string(),
    };
    let debug_str = format!("{render_error:?}");
    assert!(debug_str.contains("RenderError"));

    let construction_error = SurfaceError::ConstructionError("construction debug test".to_string());
    let debug_str = format!("{construction_error:?}");
    assert!(debug_str.contains("ConstructionError"));

    let analysis_error = SurfaceError::AnalysisError("analysis debug test".to_string());
    let debug_str = format!("{analysis_error:?}");
    assert!(debug_str.contains("AnalysisError"));
}

#[test]
fn test_error_trait_implementation() {
    // Test that SurfaceError implements the Error trait
    let error = SurfaceError::AnalysisError("error trait test".to_string());

    // Verify it can be used as a dyn Error
    let _: Box<dyn Error> = Box::new(error);
}

#[test]
fn test_surface_error_render_error_variant() {
    let error = SurfaceError::RenderError {
        backend: "plotters",
        reason: "file not found".to_string(),
    };
    match error {
        SurfaceError::RenderError { backend, reason } => {
            assert_eq!(backend, "plotters");
            assert!(reason.contains("file not found"));
        }
        _ => panic!("Expected RenderError variant, got something else"),
    }
}

#[test]
fn test_from_position_error_with_details() {
    // Test different variants of PositionError
    let validation_error =
        PositionError::ValidationError(PositionValidationErrorKind::InvalidPosition {
            reason: "invalid position details".to_string(),
        });

    let surface_error = SurfaceError::from(validation_error);

    match surface_error {
        SurfaceError::Position(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected Position variant, got something else"),
    }
}
