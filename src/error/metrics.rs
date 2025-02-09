/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/1/25
******************************************************************************/
use crate::error::{CurveError, SurfaceError};
use std::error::Error;
use std::fmt;

impl Error for MetricsError {}

#[derive(Debug)]
pub enum MetricsError {
    BasicError(String),
    ShapeError(String),
    RangeError(String),
    TrendError(String),
    RiskError(String),
    CurveError(String),
    SurfaceError(String),
    StdError { reason: String },
}

impl fmt::Display for MetricsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricsError::BasicError(msg) => write!(f, "Basic Error: {}", msg),
            MetricsError::ShapeError(msg) => write!(f, "Shape Error: {}", msg),
            MetricsError::RangeError(msg) => write!(f, "Range Error: {}", msg),
            MetricsError::TrendError(msg) => write!(f, "Trend Error: {}", msg),
            MetricsError::RiskError(msg) => write!(f, "Risk Error: {}", msg),
            MetricsError::CurveError(msg) => write!(f, "Curve Error: {}", msg),
            MetricsError::SurfaceError(msg) => write!(f, "Surface Error: {}", msg),
            MetricsError::StdError { reason } => write!(f, "Standard Error: {}", reason),
        }
    }
}

impl From<CurveError> for MetricsError {
    fn from(err: CurveError) -> Self {
        MetricsError::CurveError(err.to_string())
    }
}

impl From<SurfaceError> for MetricsError {
    fn from(err: SurfaceError) -> Self {
        MetricsError::SurfaceError(err.to_string())
    }
}

impl From<Box<dyn Error>> for MetricsError {
    fn from(err: Box<dyn Error>) -> Self {
        MetricsError::StdError {
            reason: err.to_string(),
        }
    }
}
