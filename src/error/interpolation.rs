/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 21/1/25
 ******************************************************************************/
use std::error::Error;
use crate::error::{CurvesError, PositionError, SurfaceError};

#[derive(Debug)]
pub enum InterpolationError {
    Linear(String),
    Bilinear(String),
    Cubic(String),
    Spline(String),
    StdError(String),
}

impl Error for InterpolationError {}

impl std::fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterpolationError::Linear(msg) => write!(f, "Linear interpolation error: {}", msg),
            InterpolationError::Bilinear(msg) => write!(f, "Bilinear interpolation error: {}", msg),
            InterpolationError::Cubic(msg) => write!(f, "Cubic interpolation error: {}", msg),
            InterpolationError::Spline(msg) => write!(f, "Spline interpolation error: {}", msg),
            InterpolationError::StdError(msg) => write!(f, "Standard error: {}", msg),
        }
    }
}


impl From<PositionError> for InterpolationError {
    fn from(err: PositionError) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

impl From<CurvesError> for InterpolationError {
    fn from(err: CurvesError) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

impl From<SurfaceError> for InterpolationError {
    fn from(err: SurfaceError) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}

impl From<Box<dyn Error>> for InterpolationError {
    fn from(err: Box<(dyn Error )>) -> Self {
        InterpolationError::StdError(err.to_string())
    }
}