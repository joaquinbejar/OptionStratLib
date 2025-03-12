/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/1/25
******************************************************************************/
use crate::error::{CurveError, SurfaceError};
use std::error::Error;
use std::fmt;

impl Error for MetricsError {}

/// Error types specifically related to financial and statistical metrics calculations.
///
/// This enum represents various error conditions that can occur during metrics
/// processing, analysis, and validation within the library's metrics module.
/// Each variant provides context about the specific type of error encountered.
///
/// # Variants
///
/// * `BasicError` - General metrics calculation errors
/// * `ShapeError` - Errors related to data shape mismatches or incompatible dimensions
/// * `RangeError` - Errors when data falls outside of expected/valid ranges
/// * `TrendError` - Errors in trend analysis, regression, or pattern detection
/// * `RiskError` - Errors in risk metrics calculations (like VaR, Sharpe ratio, etc.)
/// * `CurveError` - Errors related to curve-fitting or curve-based calculations
/// * `SurfaceError` - Errors in surface modeling or multi-dimensional metrics
/// * `StdError` - Standard error conditions with additional context
///
/// # Examples
///
/// ```
/// use optionstratlib::error::MetricsError;
///
/// // Creating different error types
/// let basic_err = MetricsError::BasicError("Calculation failed".to_string());
/// let range_err = MetricsError::RangeError("Value outside expected bounds".to_string());
/// ```
#[derive(Debug)]
pub enum MetricsError {
    /// General errors in metrics calculations.
    BasicError(String),

    /// Errors related to data shape mismatches or dimensional incompatibility.
    /// This typically occurs when input data has an unexpected structure.
    ShapeError(String),

    /// Errors when data falls outside expected or valid ranges.
    /// This can indicate outliers or invalid input values.
    RangeError(String),

    /// Errors in trend analysis, regression, or pattern detection algorithms.
    TrendError(String),

    /// Errors specifically related to financial risk metrics calculations,
    /// such as Value at Risk (VaR), Conditional VaR, or Sharpe ratio.
    RiskError(String),

    /// Errors encountered during curve-fitting or curve-based calculations,
    /// such as yield curves or volatility curves.
    CurveError(String),

    /// Errors in surface modeling or multi-dimensional metrics,
    /// such as volatility surfaces or correlation matrices.
    SurfaceError(String),

    /// Standard error with additional contextual information.
    StdError {
        /// Detailed explanation of the error cause
        reason: String,
    },
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
