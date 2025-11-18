use optionstratlib::error::position::PositionValidationErrorKind;
use optionstratlib::error::{
    GraphError, GreeksError, InterpolationError, OptionsError, PositionError, SurfaceError,
};
use std::error::Error;
use std::io;

// MockGraphError struct removed as it was unused.
#[test]
fn test_from_interpolation_error() {
    // Create an interpolation error
    let interpolation_error = InterpolationError::Linear("interpolation error test".to_string());

    // Convert to SurfaceError
    let surface_error = SurfaceError::from(interpolation_error);

    // Verify the conversion was successful
    match surface_error {
        SurfaceError::StdError { reason } => {
            assert!(reason.contains("interpolation error test"));
        }
        _ => panic!("Expected StdError variant, got something else"),
    }
}

#[test]
fn test_from_options_error() {
    // Create an options error
    let options_error = OptionsError::OtherError {
        reason: "options error test".to_string(),
    };

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
fn test_from_greeks_error() {
    // Create a Greeks error
    let greeks_error = GreeksError::StdError("greeks error test".to_string());

    // Convert to SurfaceError
    let surface_error = SurfaceError::from(greeks_error);

    // Verify the conversion was successful
    match surface_error {
        SurfaceError::Greeks(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected Greeks variant, got something else"),
    }
}

#[test]
fn test_from_graph_error() {
    // Create a graph error
    let graph_error = GraphError::Render("graph error test".to_string());

    // Convert to SurfaceError
    let surface_error = SurfaceError::from(graph_error);

    // Verify the conversion was successful
    match surface_error {
        SurfaceError::Graph(_) => {
            // The fact that we reached this arm means the conversion was successful
        }
        _ => panic!("Expected Graph variant, got something else"),
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

    let std_error = SurfaceError::StdError {
        reason: "std debug test".to_string(),
    };
    let debug_str = format!("{std_error:?}");
    assert!(debug_str.contains("StdError"));

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
fn test_from_io_error() {
    // Create an IO error
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let boxed_error: Box<dyn Error> = Box::new(io_error);

    // Convert to SurfaceError
    let surface_error = SurfaceError::from(boxed_error);

    // Verify the conversion was successful
    match surface_error {
        SurfaceError::StdError { reason } => {
            assert!(reason.contains("file not found"));
        }
        _ => panic!("Expected StdError variant, got something else"),
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
