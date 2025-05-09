use std::error::Error;
use std::fmt;
use crate::error::{CurveError, SurfaceError};

#[derive(Debug)]
pub enum GraphError {
    Render(String),
    Io(std::io::Error),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::Render(msg) => write!(f, "Render error: {}", msg),
            GraphError::Io(err) => write!(f, "IO error: {}", err),
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
        GraphError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
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
