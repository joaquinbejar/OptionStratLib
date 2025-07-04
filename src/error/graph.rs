use crate::error::{CurveError, SurfaceError};
use std::error::Error;
use std::fmt;

/// Represents errors that can occur during graph generation and rendering operations.
///
/// This error type encapsulates failures that may happen when creating, rendering,
/// or saving graphical representations of financial data such as option chains,
/// volatility surfaces, or strategy payoffs.
#[derive(Debug)]
pub enum GraphError {
    /// Represents errors that occur during the rendering process.
    /// Contains a descriptive message about what went wrong.
    Render(String),

    /// Represents I/O errors that occur when reading from or writing to files
    /// (e.g., when saving graphs to disk).
    Io(std::io::Error),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::Render(msg) => write!(f, "Render error: {msg}"),
            GraphError::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl Error for GraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GraphError::Render(_) => None,
            GraphError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for GraphError {
    fn from(e: std::io::Error) -> Self {
        GraphError::Io(e)
    }
}

impl From<Box<dyn Error>> for GraphError {
    fn from(err: Box<dyn Error>) -> Self {
        GraphError::Io(std::io::Error::other(err.to_string()))
    }
}

impl From<CurveError> for GraphError {
    fn from(err: CurveError) -> Self {
        GraphError::Render(err.to_string())
    }
}

impl From<SurfaceError> for GraphError {
    fn from(err: SurfaceError) -> Self {
        GraphError::Render(err.to_string())
    }
}
