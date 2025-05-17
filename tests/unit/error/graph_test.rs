use optionstratlib::error::{CurveError, GraphError, SurfaceError};
use std::error::Error;
use std::io;

#[test]
fn test_graph_error_display() {
    // Test Render variant
    let render_error = GraphError::Render("Test render error".to_string());
    assert_eq!(
        format!("{}", render_error),
        "Render error: Test render error"
    );

    // Test Io variant
    let io_error = GraphError::Io(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    assert!(format!("{}", io_error).contains("IO error"));
    assert!(format!("{}", io_error).contains("File not found"));
}

#[test]
fn test_graph_error_source() {
    // Test Render variant (no source)
    let render_error = GraphError::Render("Test render error".to_string());
    assert!(render_error.source().is_none());

    // Test Io variant (has source)
    let io_error = GraphError::Io(io::Error::new(io::ErrorKind::NotFound, "File not found"));
    assert!(io_error.source().is_some());
}

#[test]
fn test_graph_error_from_io_error() {
    // Create an io::Error
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");

    // Convert to GraphError using From trait
    let graph_error: GraphError = io_error.into();

    // Verify it's an Io variant
    match graph_error {
        GraphError::Io(_) => assert!(true),
        _ => panic!("Expected GraphError::Io variant"),
    }
}

#[test]
fn test_graph_error_from_box_dyn_error() {
    // Create a boxed error
    let boxed_error: Box<dyn Error> =
        Box::new(io::Error::other("Generic error"));

    // Convert to GraphError using From trait
    let graph_error: GraphError = boxed_error.into();

    // Verify it's an Io variant
    match graph_error {
        GraphError::Io(_) => assert!(true),
        _ => panic!("Expected GraphError::Io variant"),
    }
}

#[test]
fn test_graph_error_from_curve_error() {
    // Create a CurveError
    let curve_error = CurveError::StdError {
        reason: "Invalid curve data".to_string(),
    };

    // Convert to GraphError using From trait
    let graph_error: GraphError = curve_error.into();

    // Verify it's a Render variant with the correct message
    match graph_error {
        GraphError::Render(msg) => {
            assert!(msg.contains("Invalid curve data"));
        }
        _ => panic!("Expected GraphError::Render variant"),
    }
}

#[test]
fn test_graph_error_from_surface_error() {
    // Create a SurfaceError
    let surface_error = SurfaceError::StdError {
        reason: "Invalid surface data".to_string(),
    };

    // Convert to GraphError using From trait
    let graph_error: GraphError = surface_error.into();

    // Verify it's a Render variant with the correct message
    match graph_error {
        GraphError::Render(msg) => {
            assert!(msg.contains("Invalid surface data"));
        }
        _ => panic!("Expected GraphError::Render variant"),
    }
}
