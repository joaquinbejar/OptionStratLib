use crate::error::{CurveError, SurfaceError};
use thiserror::Error;

/// Represents errors that can occur during graph generation and rendering operations.
///
/// This error type encapsulates failures that may happen when creating, rendering,
/// or saving graphical representations of financial data such as option chains,
/// volatility surfaces, or strategy payoffs.
#[derive(Error, Debug)]
pub enum GraphError {
    /// Represents errors that occur during the rendering process.
    /// Contains a descriptive message about what went wrong.
    #[error("Render error: {0}")]
    Render(String),

    /// Represents I/O errors that occur when reading from or writing to files
    /// (e.g., when saving graphs to disk).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from curve operations.
    #[error(transparent)]
    Curve(CurveError),

    /// Error from surface operations.
    #[error(transparent)]
    Surface(SurfaceError),
}

impl From<CurveError> for GraphError {
    fn from(err: CurveError) -> Self {
        GraphError::Curve(err)
    }
}

impl From<SurfaceError> for GraphError {
    fn from(err: SurfaceError) -> Self {
        GraphError::Surface(err)
    }
}

impl From<Box<dyn std::error::Error>> for GraphError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        GraphError::Render(err.to_string())
    }
}
