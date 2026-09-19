/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/1/25
******************************************************************************/
//! Target crate (ADR-0001 D6, roadmap M1-14): **analytics**. Owns `MetricsError`.

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
}

// Conversions whose SOURCE error is owned by this layer and whose target
// sits in a lower layer. They live here (ADR-0001 D6, M1-14) so that the
// lower layer's error file never names a higher one.

/// Converts a `PositionError` into a `CurvesError` by mapping it to an
/// `OperationError` with the `InvalidParameters` variant.
///
/// This implementation ensures a smooth transition between error types
/// when a `PositionError` is encountered within a context that operates
/// on the `curves` module. The `InvalidParameters` variant is used to
/// provide detailed information about the failed operation and the reason
/// for its failure.
///
/// ## Details:
/// - The `operation` field is hardcoded as `"Position"` to indicate the
///   context of the error (i.e., relating to position management).
/// - The `reason` field is derived from the `to_string` representation of
///   the `PositionError`, ensuring a human-readable explanation.
///
/// ## Example Integration:
/// 1. If a `PositionError` is encountered during curve calculations, this
///    implementation converts it into a `CurvesError` for consistent error
///    handling within the `curves` module.
/// 2. The generated `CurvesError` provides detailed diagnostic information
///    about the reason for the failure, enabling effective debugging.
///
/// ## Implementation Notes:
/// - This conversion leverages the `OperationErrorKind::InvalidParameters`
///   variant to communicate that invalid parameters (or settings) were the
///   root cause of failure.
/// - Use this implementation to handle interoperability between error types
///   in modular design contexts.
///
/// ## Example Use Case:
/// This conversion is frequently used in scenarios where:
/// - A position-related error (e.g., from validation or limits) occurs during a
///   curve operation.
/// - Such errors need to be mapped into the `CurvesError` domain to maintain
///   consistent error handling across the library.
///
/// ## Debugging:
/// The resulting `CurvesError` will include contextual details, making it
/// straightforward to trace and debug the underlying issue.
impl From<MetricsError> for CurveError {
    fn from(err: MetricsError) -> Self {
        CurveError::MetricsError(err.to_string())
    }
}
