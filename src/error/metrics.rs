/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/1/25
******************************************************************************/
use crate::error::{CurveError, SurfaceError};
use thiserror::Error;

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
/// * `Curve` - Errors related to curve-fitting or curve-based calculations
/// * `Surface` - Errors in surface modeling or multi-dimensional metrics
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
#[derive(Error, Debug)]
pub enum MetricsError {
    /// General errors in metrics calculations.
    #[error("Basic Error: {0}")]
    BasicError(String),

    /// Errors related to data shape mismatches or dimensional incompatibility.
    /// This typically occurs when input data has an unexpected structure.
    #[error("Shape Error: {0}")]
    ShapeError(String),

    /// Errors when data falls outside expected or valid ranges.
    /// This can indicate outliers or invalid input values.
    #[error("Range Error: {0}")]
    RangeError(String),

    /// Errors in trend analysis, regression, or pattern detection algorithms.
    #[error("Trend Error: {0}")]
    TrendError(String),

    /// Errors specifically related to financial risk metrics calculations,
    /// such as Value at Risk (VaR), Conditional VaR, or Sharpe ratio.
    #[error("Risk Error: {0}")]
    RiskError(String),

    /// Errors encountered during curve-fitting or curve-based calculations,
    /// such as yield curves or volatility curves.
    #[error(transparent)]
    Curve(#[from] CurveError),

    /// Errors in surface modeling or multi-dimensional metrics,
    /// such as volatility surfaces or correlation matrices.
    #[error(transparent)]
    Surface(#[from] SurfaceError),

    /// Standard error with additional contextual information.
    #[error("Standard Error: {reason}")]
    StdError {
        /// Detailed explanation of the error cause
        reason: String,
    },
}

impl From<Box<dyn std::error::Error>> for MetricsError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        MetricsError::StdError {
            reason: err.to_string(),
        }
    }
}
