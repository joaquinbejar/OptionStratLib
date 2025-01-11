use crate::curves::analysis::metrics::{
    BasicMetrics, CurveMetrics, RangeMetrics, RiskMetrics, ShapeMetrics, TrendMetrics,
};
use crate::error::CurvesError;

/// A trait for extracting comprehensive metrics from a curve.
///
/// # Overview
/// The `CurveMetricsExtractor` trait provides methods to compute various statistical,
/// analytical, and risk-related metrics for a given curve. It allows for a systematic
/// and extensible approach to curve analysis across different curve types and contexts.
///
/// # Methods
///
/// ## Metric Computation Methods
/// - `compute_basic_metrics`: Calculates fundamental statistical measures.
/// - `compute_shape_metrics`: Analyzes curve shape characteristics.
/// - `compute_range_metrics`: Determines range and distribution properties.
/// - `compute_trend_metrics`: Evaluates directional and regression-related metrics.
/// - `compute_risk_metrics`: Quantifies financial and statistical risk indicators.
///
/// ## Comprehensive Metrics
/// - `compute_curve_metrics`: Computes all metrics and combines them into a `CurveMetrics` struct.
///
/// # Usage
/// Implement this trait for specific curve types or analysis strategies to provide
/// custom metric computation logic tailored to different domains or requirements.
///
pub trait CurveMetricsExtractor {
    /// Computes basic statistical metrics for the curve.
    ///
    /// # Returns
    /// - `Ok(BasicMetrics)`: Struct containing mean, median, mode, and standard deviation.
    /// - `Err(CurvesError)`: If metrics computation fails.
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, CurvesError>;

    /// Computes shape-related metrics for the curve.
    ///
    /// # Returns
    /// - `Ok(ShapeMetrics)`: Struct containing skewness, kurtosis, peaks, valleys, and inflection points.
    /// - `Err(CurvesError)`: If metrics computation fails.
    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, CurvesError>;

    /// Computes range-related metrics for the curve.
    ///
    /// # Returns
    /// - `Ok(RangeMetrics)`: Struct containing min/max points, range, quartiles, and interquartile range.
    /// - `Err(CurvesError)`: If metrics computation fails.
    fn compute_range_metrics(&self) -> Result<RangeMetrics, CurvesError>;

    /// Computes trend-related metrics for the curve.
    ///
    /// # Returns
    /// - `Ok(TrendMetrics)`: Struct containing slope, intercept, R-squared, and moving average.
    /// - `Err(CurvesError)`: If metrics computation fails.
    fn compute_trend_metrics(&self) -> Result<TrendMetrics, CurvesError>;

    /// Computes risk-related metrics for the curve.
    ///
    /// # Returns
    /// - `Ok(RiskMetrics)`: Struct containing volatility, VaR, expected shortfall, beta, and Sharpe ratio.
    /// - `Err(CurvesError)`: If metrics computation fails.
    fn compute_risk_metrics(&self) -> Result<RiskMetrics, CurvesError>;

    /// Computes and aggregates all curve metrics into a comprehensive `CurveMetrics` struct.
    ///
    /// # Returns
    /// - `Ok(CurveMetrics)`: A struct containing all computed metrics.
    /// - `Err(CurvesError)`: If any metrics computation fails.
    fn compute_curve_metrics(&self) -> Result<CurveMetrics, CurvesError> {
        let basic = self.compute_basic_metrics()?;
        let shape = self.compute_shape_metrics()?;
        let range = self.compute_range_metrics()?;
        let trend = self.compute_trend_metrics()?;
        let risk = self.compute_risk_metrics()?;

        Ok(CurveMetrics::new(basic, shape, range, trend, risk))
    }
}
