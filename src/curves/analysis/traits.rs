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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curves::Point2D;
    use crate::error::OperationErrorKind;
    use num_traits::FromPrimitive;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    // Mock struct to test trait implementation
    struct MockCurve;

    // Helper function to create test points
    fn create_test_point(x: f64, y: f64) -> Point2D {
        Point2D::new(Decimal::from_f64(x).unwrap(), Decimal::from_f64(y).unwrap())
    }

    // Implementation of CurveMetricsExtractor for our mock struct
    impl CurveMetricsExtractor for MockCurve {
        fn compute_basic_metrics(&self) -> Result<BasicMetrics, CurvesError> {
            Ok(BasicMetrics {
                mean: dec!(10.5),
                median: dec!(10.0),
                mode: dec!(9.0),
                std_dev: dec!(1.2),
            })
        }

        fn compute_shape_metrics(&self) -> Result<ShapeMetrics, CurvesError> {
            Ok(ShapeMetrics {
                skewness: dec!(0.5),
                kurtosis: dec!(3.0),
                peaks: vec![create_test_point(1.0, 10.0)],
                valleys: vec![create_test_point(2.0, 5.0)],
                inflection_points: vec![create_test_point(1.5, 7.5)],
            })
        }

        fn compute_range_metrics(&self) -> Result<RangeMetrics, CurvesError> {
            Ok(RangeMetrics {
                min: create_test_point(0.0, 5.0),
                max: create_test_point(10.0, 15.0),
                range: dec!(10.0),
                quartiles: (dec!(7.0), dec!(10.0), dec!(13.0)),
                interquartile_range: dec!(6.0),
            })
        }

        fn compute_trend_metrics(&self) -> Result<TrendMetrics, CurvesError> {
            Ok(TrendMetrics {
                slope: dec!(1.5),
                intercept: dec!(2.0),
                r_squared: dec!(0.95),
                moving_average: vec![create_test_point(1.0, 3.5), create_test_point(2.0, 5.0)],
            })
        }

        fn compute_risk_metrics(&self) -> Result<RiskMetrics, CurvesError> {
            Ok(RiskMetrics {
                volatility: dec!(0.15),
                value_at_risk: dec!(0.05),
                expected_shortfall: dec!(0.07),
                beta: dec!(1.2),
                sharpe_ratio: dec!(2.5),
            })
        }
    }

    // Mock struct for testing error cases
    struct ErrorMockCurve;

    impl CurveMetricsExtractor for ErrorMockCurve {
        fn compute_basic_metrics(&self) -> Result<BasicMetrics, CurvesError> {
            Err(CurvesError::OperationError(
                OperationErrorKind::InvalidParameters {
                    reason: "Basic metrics computation failed".to_string(),
                    operation: "compute_basic_metrics".to_string(),
                },
            ))
        }

        fn compute_shape_metrics(&self) -> Result<ShapeMetrics, CurvesError> {
            Err(CurvesError::OperationError(
                OperationErrorKind::InvalidParameters {
                    reason: "Shape metrics computation failed".to_string(),
                    operation: "compute_shape_metrics".to_string(),
                },
            ))
        }

        fn compute_range_metrics(&self) -> Result<RangeMetrics, CurvesError> {
            Err(CurvesError::OperationError(
                OperationErrorKind::InvalidParameters {
                    reason: "Range metrics computation failed".to_string(),
                    operation: "compute_range_metrics".to_string(),
                },
            ))
        }

        fn compute_trend_metrics(&self) -> Result<TrendMetrics, CurvesError> {
            Err(CurvesError::OperationError(
                OperationErrorKind::InvalidParameters {
                    reason: "Trend metrics computation failed".to_string(),
                    operation: "compute_trend_metrics".to_string(),
                },
            ))
        }

        fn compute_risk_metrics(&self) -> Result<RiskMetrics, CurvesError> {
            Err(CurvesError::OperationError(
                OperationErrorKind::InvalidParameters {
                    reason: "Risk metrics computation failed".to_string(),
                    operation: "compute_risk_metrics".to_string(),
                },
            ))
        }
    }

    mod test_successful_computations {
        use super::*;

        #[test]
        fn test_compute_basic_metrics() {
            let curve = MockCurve;
            let result = curve.compute_basic_metrics();
            assert!(result.is_ok());
            let metrics = result.unwrap();
            assert_eq!(metrics.mean, dec!(10.5));
            assert_eq!(metrics.median, dec!(10.0));
            assert_eq!(metrics.mode, dec!(9.0));
            assert_eq!(metrics.std_dev, dec!(1.2));
        }

        #[test]
        fn test_compute_shape_metrics() {
            let curve = MockCurve;
            let result = curve.compute_shape_metrics();
            assert!(result.is_ok());
            let metrics = result.unwrap();
            assert_eq!(metrics.skewness, dec!(0.5));
            assert_eq!(metrics.kurtosis, dec!(3.0));
            assert_eq!(metrics.peaks.len(), 1);
            assert_eq!(metrics.valleys.len(), 1);
            assert_eq!(metrics.inflection_points.len(), 1);
        }

        #[test]
        fn test_compute_range_metrics() {
            let curve = MockCurve;
            let result = curve.compute_range_metrics();
            assert!(result.is_ok());
            let metrics = result.unwrap();
            assert_eq!(metrics.range, dec!(10.0));
            assert_eq!(metrics.quartiles.0, dec!(7.0));
            assert_eq!(metrics.interquartile_range, dec!(6.0));
        }

        #[test]
        fn test_compute_trend_metrics() {
            let curve = MockCurve;
            let result = curve.compute_trend_metrics();
            assert!(result.is_ok());
            let metrics = result.unwrap();
            assert_eq!(metrics.slope, dec!(1.5));
            assert_eq!(metrics.intercept, dec!(2.0));
            assert_eq!(metrics.r_squared, dec!(0.95));
            assert_eq!(metrics.moving_average.len(), 2);
        }

        #[test]
        fn test_compute_risk_metrics() {
            let curve = MockCurve;
            let result = curve.compute_risk_metrics();
            assert!(result.is_ok());
            let metrics = result.unwrap();
            assert_eq!(metrics.volatility, dec!(0.15));
            assert_eq!(metrics.value_at_risk, dec!(0.05));
            assert_eq!(metrics.expected_shortfall, dec!(0.07));
            assert_eq!(metrics.beta, dec!(1.2));
            assert_eq!(metrics.sharpe_ratio, dec!(2.5));
        }

        #[test]
        fn test_compute_curve_metrics() {
            let curve = MockCurve;
            let result = curve.compute_curve_metrics();
            assert!(result.is_ok());

            let metrics = result.unwrap();
            assert_eq!(metrics.basic.mean, dec!(10.5));
            assert_eq!(metrics.shape.skewness, dec!(0.5));
            assert_eq!(metrics.range.range, dec!(10.0));
            assert_eq!(metrics.trend.slope, dec!(1.5));
            assert_eq!(metrics.risk.volatility, dec!(0.15));
        }
    }

    mod test_error_cases {
        use super::*;

        #[test]
        fn test_basic_metrics_error() {
            let curve = ErrorMockCurve;
            let result = curve.compute_basic_metrics();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Operation error: Invalid parameters for operation 'compute_basic_metrics': Basic metrics computation failed"
            );
        }

        #[test]
        fn test_shape_metrics_error() {
            let curve = ErrorMockCurve;
            let result = curve.compute_shape_metrics();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Operation error: Invalid parameters for operation 'compute_shape_metrics': Shape metrics computation failed"
            );
        }

        #[test]
        fn test_range_metrics_error() {
            let curve = ErrorMockCurve;
            let result = curve.compute_range_metrics();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Operation error: Invalid parameters for operation 'compute_range_metrics': Range metrics computation failed"
            );
        }

        #[test]
        fn test_trend_metrics_error() {
            let curve = ErrorMockCurve;
            let result = curve.compute_trend_metrics();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Operation error: Invalid parameters for operation 'compute_trend_metrics': Trend metrics computation failed"
            );
        }

        #[test]
        fn test_risk_metrics_error() {
            let curve = ErrorMockCurve;
            let result = curve.compute_risk_metrics();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Operation error: Invalid parameters for operation 'compute_risk_metrics': Risk metrics computation failed"
            );
        }

        #[test]
        fn test_curve_metrics_error_propagation() {
            let curve = ErrorMockCurve;
            let result = curve.compute_curve_metrics();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Operation error: Invalid parameters for operation 'compute_basic_metrics': Basic metrics computation failed"
            );
        }
    }
}
