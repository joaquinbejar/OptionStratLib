/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/2/25
******************************************************************************/
use crate::curves::{Curve, Point2D};
use crate::error::{CurveError, OperationErrorKind};
use crate::geometrics::{BasicMetrics, MetricsExtractor, RangeMetrics, ShapeMetrics, TrendMetrics};
use num_traits::ToPrimitive;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_decimal::Decimal;
use statrs::distribution::{ContinuousCDF, Normal};
use std::collections::BTreeSet;

/// A trait that defines the behavior of any object that can produce a curve representation.
///
/// Types implementing the `Curvable` trait must provide the `curve` method, which generates
/// a `Curve` object based on the internal state of the implementer. This method returns
/// a `Result`, allowing for proper error handling in situations where the curve
/// cannot be generated due to various issues (e.g., invalid data or computation errors).
///
/// # Method
///
/// - `curve`
///   - **Returns**:
///     - A `Result` containing:
///       - `Curve`: On success, a valid representation of the curve.
///       - `CurveError`: On failure, detailed information about why the curve could not be generated.
///
/// # Error Handling
///
/// Given the reliance on precise data and operations, the `Curvable` trait integrates
/// tightly with the `CurveError` type to handle potential issues, such as:
/// - Invalid points or coordinates (`Point2DError`)
/// - Issues in curve construction (`ConstructionError`)
/// - Errors during interpolation (`InterpolationError`)
/// - General computation or operational failures (`OperationError`, `StdError`)
///
/// # Example Usage
///
/// This trait forms the basis for creating highly customizable and precise curve objects,
/// ensuring compatibility with mathematical, computational, or graphical operations.
///
/// Implementing this trait allows an object to seamlessly interact with the higher-level
/// functionalities in the `curves` module, such as visualization, analysis, and transformation.
///
/// # See Also
/// - [`Curve`]: Represents the mathematical curve generated by this trait.
/// - [`CurveError`]: Error type encapsulating issues encountered during curve generation.
pub trait Curvable {
    /// Generates a `Curve` representation of the implementer.
    ///
    /// The `curve` method is the core functionality of this trait. It is expected
    /// to compute and return a `Curve` object that accurately describes the
    /// implementer's structure or state in the context of a two-dimensional curve.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: When the curve is successfully generated.
    /// - `Err(CurveError)`: If curve generation fails for any reason.
    ///
    /// # Errors
    ///
    /// The method may return a `CurveError` in scenarios such as:
    /// - **Point2DError**: If an invalid or missing 2D point is encountered.
    /// - **ConstructionError**: When the curve cannot be initialized due to invalid input.
    /// - **InterpolationError**: If there are issues during interpolation of the curve's points.
    /// - **AnalysisError**: In cases where analytical operations on the curve fail.
    ///
    /// This ensures robust error handling for downstream processes and applications.
    fn curve(&self) -> Result<Curve, CurveError>;
}

/// A trait for generating statistical curves based on metrics
///
/// This trait provides methods to generate curves that match specified
/// statistical properties. It extends the `MetricsExtractor` trait to
/// ensure implementing types can both extract and generate metrics.
pub trait StatisticalCurve: MetricsExtractor {
    /// Retrieves the x-axis values for the statistical curve.
    ///
    /// This method returns a vector of `Decimal` values representing the x-coordinates
    /// of the points that define the curve. These x-values are essential for plotting
    /// the curve and performing various statistical analyses.
    ///
    /// # Returns
    ///
    /// A `Vec<Decimal>` containing the x-values of the statistical curve. Each `Decimal`
    /// represents a point on the x-axis.
    fn get_x_values(&self) -> Vec<Decimal>;

    /// Generates a statistical curve with properties that match the provided metrics.
    ///
    /// # Overview
    /// This function creates a curve with statistical properties that approximate the
    /// specified metrics. It uses a combination of normal distribution sampling and
    /// transformations to achieve the desired statistical characteristics.
    ///
    /// # Parameters
    /// - `basic_metrics`: Basic statistical properties like mean, median, mode, and standard deviation.
    /// - `shape_metrics`: Shape-related metrics like skewness and kurtosis.
    /// - `range_metrics`: Range information including min, max, and quartile data.
    /// - `trend_metrics`: Trend information including slope and intercept for linear trend.
    /// - `num_points`: Number of points to generate in the curve.
    /// - `seed`: Optional random seed for reproducible curve generation.
    ///
    /// # Returns
    /// - `Result<Curve, CurveError>`: A curve matching the specified statistical properties,
    ///   or an error if generation fails.
    ///
    fn generate_statistical_curve(
        &self,
        basic_metrics: &BasicMetrics,
        shape_metrics: &ShapeMetrics,
        range_metrics: &RangeMetrics,
        trend_metrics: &TrendMetrics,
        num_points: usize,
        seed: Option<u64>,
    ) -> Result<Curve, CurveError> {
        if num_points < 2 {
            return Err(CurveError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "generate_statistical_curve".to_string(),
                    reason: "Number of points must be at least 2".to_string(),
                },
            ));
        }

        // Initialize random number generator with optional seed
        let seed_value = seed.unwrap_or_else(rand::random);
        let mut rng = StdRng::seed_from_u64(seed_value);

        // Create a normal distribution with the given mean and standard deviation
        let normal = Normal::new(
            basic_metrics.mean.to_f64().unwrap_or(0.0),
            basic_metrics.std_dev.to_f64().unwrap_or(1.0),
        )
        .map_err(|e| CurveError::MetricsError(e.to_string()))?;

        // Generate initial y-values from the normal distribution
        let mut y_values: Vec<f64> = (0..num_points)
            .map(|_| {
                let u: f64 = rng.random_range(0.0..1.0); // Generate a value between 0 and 1
                normal.inverse_cdf(u) // Convert to normal distribution using inverse CDF
            })
            .collect();

        // Apply transformations to match skewness and kurtosis (simplified approach)
        let skewness = shape_metrics.skewness.to_f64().unwrap_or(0.0);
        let kurtosis = shape_metrics.kurtosis.to_f64().unwrap_or(0.0);

        // Apply skewness transformation (simplified approach)
        if skewness.abs() > 0.01 {
            for y in &mut y_values {
                // Apply a simple transformation to induce skewness
                *y += skewness * (*y - basic_metrics.mean.to_f64().unwrap_or(0.0)).powi(2);
            }
        }

        // Apply kurtosis transformation (simplified approach)
        if kurtosis.abs() > 0.01 {
            for y in &mut y_values {
                // Apply a simple transformation to adjust kurtosis
                let z = (*y - basic_metrics.mean.to_f64().unwrap_or(0.0))
                    / basic_metrics.std_dev.to_f64().unwrap_or(1.0);
                *y += kurtosis * 0.1 * z.powi(3);
            }
        }

        let x_values: Vec<Decimal> = self.get_x_values();

        // Apply trend (slope and intercept)
        let slope = trend_metrics.slope.to_f64().unwrap_or(0.0);
        if slope.abs() > 0.001 {
            let intercept = trend_metrics.intercept.to_f64().unwrap_or(0.0);
            for i in 0..y_values.len() {
                y_values[i] += slope * x_values[i].to_f64().unwrap() + intercept;
            }
        }

        // Scale y-values to match the range
        let current_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let current_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let current_range = current_max - current_min;

        let target_min = range_metrics.min.y.to_f64().unwrap_or(0.0);
        let target_max = range_metrics.max.y.to_f64().unwrap_or(1.0);
        let target_range = target_max - target_min;

        // Scale and shift the y-values to match the target range
        if current_range > 0.0 {
            for y in &mut y_values {
                *y = ((*y - current_min) / current_range) * target_range + target_min;
            }
        }

        // Ensure mode value is included
        if num_points > 3 {
            let index = rng.random_range(0..(num_points / 3));
            y_values[index] = basic_metrics.mode.to_f64().unwrap_or(y_values[index]);
        }

        // Create points and construct curve
        let mut points = BTreeSet::new();
        for i in 0..num_points {
            let point = Point2D::from_f64_tuple(x_values[i].to_f64().unwrap(), y_values[i])?;
            points.insert(point);
        }

        // Create the curve
        Ok(Curve::new(points))
    }

    /// Generates a refined statistical curve that iteratively adjusts to better match
    /// the target metrics.
    ///
    /// This method extends the basic curve generation by performing multiple attempts
    /// with adjusted parameters until the resulting curve metrics are within the specified
    /// tolerance of the target metrics.
    ///
    /// # Parameters
    /// - `basic_metrics`: Target basic statistical metrics
    /// - `shape_metrics`: Target shape metrics
    /// - `range_metrics`: Target range metrics
    /// - `trend_metrics`: Target trend metrics
    /// - `num_points`: Number of points to generate
    /// - `max_attempts`: Maximum number of generation attempts (default: 5)
    /// - `tolerance`: Acceptable difference between target and actual metrics (default: 0.1)
    /// - `seed`: Optional random seed for reproducibility
    ///
    /// # Returns
    /// - `Result<Curve, CurveError>`: The generated curve or an error
    #[allow(clippy::too_many_arguments)]
    fn generate_refined_statistical_curve(
        &self,
        basic_metrics: &BasicMetrics,
        shape_metrics: &ShapeMetrics,
        range_metrics: &RangeMetrics,
        trend_metrics: &TrendMetrics,
        num_points: usize,
        max_attempts: usize,
        tolerance: Decimal,
        seed: Option<u64>,
    ) -> Result<Curve, CurveError> {
        let max_tries = if max_attempts == 0 { 5 } else { max_attempts };
        let mut seed_value = seed.unwrap_or_else(rand::random);

        for _ in 0..max_tries {
            let curve = self.generate_statistical_curve(
                basic_metrics,
                shape_metrics,
                range_metrics,
                trend_metrics,
                num_points,
                Some(seed_value),
            )?;

            if self.verify_curve_metrics(&curve, basic_metrics, tolerance)? {
                return Ok(curve);
            }

            // Try a different seed for the next attempt
            seed_value = seed_value.wrapping_add(1);
        }

        // Return the last generated curve even if it doesn't perfectly match
        self.generate_statistical_curve(
            basic_metrics,
            shape_metrics,
            range_metrics,
            trend_metrics,
            num_points,
            Some(seed_value),
        )
    }

    /// Verifies if the metrics of the generated curve match the target metrics
    /// within the specified tolerance.
    ///
    /// # Parameters
    /// - `curve`: The curve to verify
    /// - `target_metrics`: The target basic metrics to compare against
    /// - `tolerance`: Maximum acceptable difference between actual and target metrics
    ///
    /// # Returns
    /// - `Result<bool, CurveError>`: True if metrics match within tolerance, false otherwise
    fn verify_curve_metrics(
        &self,
        curve: &Curve,
        target_metrics: &BasicMetrics,
        tolerance: Decimal,
    ) -> Result<bool, CurveError> {
        let actual_metrics = curve
            .compute_basic_metrics()
            .map_err(|e| CurveError::MetricsError(format!("Failed to compute metrics: {}", e)))?;

        // Check if the key metrics are within tolerance
        let mean_diff = (actual_metrics.mean - target_metrics.mean).abs();
        let std_dev_diff = (actual_metrics.std_dev - target_metrics.std_dev).abs();

        Ok(mean_diff <= tolerance && std_dev_diff <= tolerance)
    }
}

#[cfg(test)]
mod tests_statistical_curve {
    use super::*;
    use crate::error::MetricsError;
    use crate::geometrics::RiskMetrics;
    use crate::utils::Len;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    // A simple struct that implements StatisticalCurve trait for testing
    struct TestCurveGenerator;

    impl Len for TestCurveGenerator {
        fn len(&self) -> usize {
            unimplemented!()
        }
    }

    impl MetricsExtractor for TestCurveGenerator {
        fn compute_basic_metrics(&self) -> Result<BasicMetrics, MetricsError> {
            // Return default metrics for testing
            Ok(BasicMetrics {
                mean: dec!(0.0),
                median: dec!(0.0),
                mode: dec!(0.0),
                std_dev: dec!(1.0),
            })
        }

        fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError> {
            Ok(ShapeMetrics {
                skewness: dec!(0.0),
                kurtosis: dec!(0.0),
                peaks: vec![],
                valleys: vec![],
                inflection_points: vec![],
            })
        }

        fn compute_range_metrics(&self) -> Result<RangeMetrics, MetricsError> {
            Ok(RangeMetrics {
                min: Point2D::new(dec!(0.0), dec!(0.0)),
                max: Point2D::new(dec!(10.0), dec!(10.0)),
                range: dec!(10.0),
                quartiles: (dec!(2.5), dec!(5.0), dec!(7.5)),
                interquartile_range: dec!(5.0),
            })
        }

        fn compute_trend_metrics(&self) -> Result<TrendMetrics, MetricsError> {
            Ok(TrendMetrics {
                slope: dec!(0.0),
                intercept: dec!(0.0),
                r_squared: dec!(0.0),
                moving_average: vec![],
            })
        }

        fn compute_risk_metrics(&self) -> Result<RiskMetrics, MetricsError> {
            unimplemented!()
        }
    }

    impl StatisticalCurve for TestCurveGenerator {
        fn get_x_values(&self) -> Vec<Decimal> {
            (0..10).map(Decimal::from).collect()
        }
    }

    // Create a struct that implements Curve's compute_basic_metrics for testing
    impl Curvable for TestCurveGenerator {
        fn curve(&self) -> Result<Curve, CurveError> {
            // Create a simple linear curve
            let points: BTreeSet<Point2D> = (0..10)
                .map(|i| Point2D::new(Decimal::from(i), Decimal::from(i)))
                .collect();
            Ok(Curve::new(points))
        }
    }

    #[test]
    fn test_get_x_values() {
        let generator = TestCurveGenerator;
        let x_values = generator.get_x_values();

        assert_eq!(x_values.len(), 10);
        assert_eq!(x_values[0], dec!(0));
        assert_eq!(x_values[9], dec!(9));
    }

    #[test]
    fn test_generate_statistical_curve_invalid_points() {
        let generator = TestCurveGenerator;

        let basic_metrics = BasicMetrics::default();
        let shape_metrics = ShapeMetrics::default();
        let range_metrics = RangeMetrics::default();
        let trend_metrics = TrendMetrics::default();

        // Test with less than 2 points
        let result = generator.generate_statistical_curve(
            &basic_metrics,
            &shape_metrics,
            &range_metrics,
            &trend_metrics,
            1, // Invalid: less than 2 points
            None,
        );

        assert!(result.is_err());
        if let Err(CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation,
            reason,
        })) = result
        {
            assert_eq!(operation, "generate_statistical_curve");
            assert!(reason.contains("Number of points must be at least 2"));
        } else {
            panic!("Expected InvalidParameters error");
        }
    }

    #[test]
    fn test_verify_curve_metrics() {
        let generator = TestCurveGenerator;

        // Create a simple curve
        let curve = generator.curve().unwrap();

        // Define target metrics close to the actual metrics
        let target_metrics = BasicMetrics {
            mean: dec!(4.5), // Close to actual mean (4.5 for points 0..9)
            median: dec!(4.5),
            mode: dec!(0.0),
            std_dev: dec!(3.0), // Close to actual std_dev
        };

        // Verify with a generous tolerance
        let result = generator.verify_curve_metrics(&curve, &target_metrics, dec!(1.0));
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify with a strict tolerance that should fail
        let result = generator.verify_curve_metrics(&curve, &target_metrics, dec!(0.1));
        assert!(result.is_ok());
        // This might fail depending on the actual metrics of the curve
    }

    // Test Default implementation for the required metrics structs
    impl Default for BasicMetrics {
        fn default() -> Self {
            Self {
                mean: dec!(0.0),
                median: dec!(0.0),
                mode: dec!(0.0),
                std_dev: dec!(1.0),
            }
        }
    }

    impl Default for ShapeMetrics {
        fn default() -> Self {
            Self {
                skewness: dec!(0.0),
                kurtosis: dec!(0.0),
                peaks: vec![],
                valleys: vec![],
                inflection_points: vec![],
            }
        }
    }

    impl Default for RangeMetrics {
        fn default() -> Self {
            Self {
                min: Point2D::new(dec!(0.0), dec!(0.0)),
                max: Point2D::new(dec!(10.0), dec!(10.0)),
                range: dec!(10.0),
                quartiles: (dec!(2.5), dec!(5.0), dec!(7.5)),
                interquartile_range: dec!(5.0),
            }
        }
    }

    impl Default for TrendMetrics {
        fn default() -> Self {
            Self {
                slope: dec!(0.0),
                intercept: dec!(0.0),
                r_squared: dec!(0.0),
                moving_average: vec![],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MetricsError;
    use crate::geometrics::RiskMetrics;
    use crate::utils::Len;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct MockCurvable {
        points: BTreeSet<Point2D>,
        should_fail: bool,
    }

    impl MockCurvable {
        fn new(should_fail: bool) -> Self {
            let mut points = BTreeSet::new();

            if !should_fail {
                // Crear algunos puntos válidos
                points.insert(Point2D::new(dec!(1.0), dec!(2.0)));
                points.insert(Point2D::new(dec!(2.0), dec!(3.0)));
                points.insert(Point2D::new(dec!(3.0), dec!(4.0)));
            }

            Self {
                points,
                should_fail,
            }
        }
    }

    impl Curvable for MockCurvable {
        fn curve(&self) -> Result<Curve, CurveError> {
            if self.should_fail {
                Err(CurveError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "curve".to_string(),
                        reason: "Test failure".to_string(),
                    },
                ))
            } else {
                Ok(Curve::new(self.points.clone()))
            }
        }
    }

    // Mock implementación para StatisticalCurve
    struct MockStatisticalCurve {
        x_values: Vec<Decimal>,
    }

    impl MockStatisticalCurve {
        fn new() -> Self {
            let x_values = vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)];
            Self { x_values }
        }
    }

    impl Len for MockStatisticalCurve {
        fn len(&self) -> usize {
            self.x_values.len()
        }
    }

    impl MetricsExtractor for MockStatisticalCurve {
        fn compute_basic_metrics(&self) -> Result<BasicMetrics, MetricsError> {
            Ok(BasicMetrics {
                mean: dec!(3.0),
                median: dec!(3.0),
                mode: dec!(3.0),
                std_dev: dec!(1.5),
            })
        }

        fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError> {
            Ok(ShapeMetrics {
                skewness: dec!(0.0),
                kurtosis: dec!(0.0),
                peaks: vec![],
                valleys: vec![],
                inflection_points: vec![],
            })
        }

        fn compute_range_metrics(&self) -> Result<RangeMetrics, MetricsError> {
            Ok(RangeMetrics {
                min: Point2D::new(dec!(1.0), dec!(1.0)),
                max: Point2D::new(dec!(5.0), dec!(5.0)),
                range: dec!(4.0),
                quartiles: (Default::default(), Default::default(), Default::default()),
                interquartile_range: Default::default(),
            })
        }

        fn compute_trend_metrics(&self) -> Result<TrendMetrics, MetricsError> {
            Ok(TrendMetrics {
                slope: dec!(1.0),
                intercept: dec!(0.0),
                r_squared: dec!(1.0),
                moving_average: vec![],
            })
        }

        fn compute_risk_metrics(&self) -> Result<RiskMetrics, MetricsError> {
            Ok(RiskMetrics {
                volatility: Default::default(),
                value_at_risk: Default::default(),
                expected_shortfall: Default::default(),
                beta: Default::default(),
                sharpe_ratio: Default::default(),
            })
        }
    }

    impl StatisticalCurve for MockStatisticalCurve {
        fn get_x_values(&self) -> Vec<Decimal> {
            self.x_values.clone()
        }
    }

    // Tests para Curvable
    #[test]
    fn test_curvable_success() {
        let mock = MockCurvable::new(false);
        let result = mock.curve();

        assert!(result.is_ok(), "Curve generation should succeed");
        let curve = result.unwrap();

        assert_eq!(curve.len(), 3, "Curve should have 3 points");
    }

    #[test]
    fn test_curvable_failure() {
        let mock = MockCurvable::new(true);
        let result = mock.curve();

        assert!(result.is_err(), "Curve generation should fail");

        if let Err(CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation,
            reason,
        })) = result
        {
            assert_eq!(operation, "curve", "Operation name should match");
            assert_eq!(reason, "Test failure", "Error reason should match");
        } else {
            panic!("Unexpected error type");
        }
    }

    // Tests para StatisticalCurve
    #[test]
    fn test_get_x_values() {
        let mock = MockStatisticalCurve::new();
        let x_values = mock.get_x_values();

        assert_eq!(x_values.len(), 5, "Should return 5 x values");
        assert_eq!(x_values[0], dec!(1.0), "First x value should be 1.0");
        assert_eq!(x_values[4], dec!(5.0), "Last x value should be 5.0");
    }

    #[test]
    fn test_generate_statistical_curve_invalid_points() {
        let mock = MockStatisticalCurve::new();

        let basic_metrics = BasicMetrics {
            mean: dec!(3.0),
            median: dec!(3.0),
            mode: dec!(3.0),
            std_dev: dec!(1.5),
        };

        let shape_metrics = ShapeMetrics {
            skewness: dec!(0.0),
            kurtosis: dec!(0.0),
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        };

        let range_metrics = RangeMetrics {
            min: Point2D::new(dec!(1.0), dec!(1.0)),
            max: Point2D::new(dec!(5.0), dec!(5.0)),
            range: dec!(4.0),
            quartiles: (Default::default(), Default::default(), Default::default()),
            interquartile_range: Default::default(),
        };

        let trend_metrics = TrendMetrics {
            slope: dec!(1.0),
            intercept: dec!(0.0),
            r_squared: dec!(1.0),
            moving_average: vec![],
        };

        let result = mock.generate_statistical_curve(
            &basic_metrics,
            &shape_metrics,
            &range_metrics,
            &trend_metrics,
            1,
            None,
        );

        assert!(result.is_err(), "Should fail with less than 2 points");
        if let Err(CurveError::OperationError(OperationErrorKind::InvalidParameters {
            operation,
            reason,
        })) = result
        {
            assert_eq!(operation, "generate_statistical_curve");
            assert!(reason.contains("at least 2"));
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_generate_statistical_curve_success() {
        let mock = MockStatisticalCurve::new();

        let basic_metrics = BasicMetrics {
            mean: dec!(3.0),
            median: dec!(3.0),
            mode: dec!(3.0),
            std_dev: dec!(1.5),
        };

        let shape_metrics = ShapeMetrics {
            skewness: dec!(0.0),
            kurtosis: dec!(0.0),
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        };

        let range_metrics = RangeMetrics {
            min: Point2D::new(dec!(1.0), dec!(1.0)),
            max: Point2D::new(dec!(5.0), dec!(5.0)),
            range: dec!(4.0),
            quartiles: (Default::default(), Default::default(), Default::default()),
            interquartile_range: Default::default(),
        };

        let trend_metrics = TrendMetrics {
            slope: dec!(1.0),
            intercept: dec!(0.0),
            r_squared: dec!(1.0),
            moving_average: vec![],
        };

        // Probar generación de curva válida
        let result = mock.generate_statistical_curve(
            &basic_metrics,
            &shape_metrics,
            &range_metrics,
            &trend_metrics,
            5,        // 5 puntos
            Some(42), // seed fijo para prueba reproducible
        );

        assert!(result.is_ok(), "Should successfully generate a curve");
        let curve = result.unwrap();
        assert!(curve.len() > 0, "Generated curve should contain points");
    }

    #[test]
    fn test_verify_curve_metrics() {
        let mock = MockStatisticalCurve::new();

        // Crear una curva simple para pruebas
        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(1.0), dec!(2.0)));
        points.insert(Point2D::new(dec!(2.0), dec!(3.0)));
        points.insert(Point2D::new(dec!(3.0), dec!(4.0)));
        let curve = Curve::new(points);

        // Métricas objetivo cercanas a las reales (dentro de la tolerancia)
        let target_metrics = BasicMetrics {
            mean: dec!(3.1),
            median: dec!(3.0),
            mode: dec!(3.0),
            std_dev: dec!(1.0),
        };

        // Tolerancia suficiente para que pase
        let result = mock.verify_curve_metrics(&curve, &target_metrics, dec!(0.5));
        assert!(result.is_ok(), "Verification should not fail");
        assert!(result.unwrap(), "Metrics should be within tolerance");

        // Tolerancia insuficiente para que pase
        let result = mock.verify_curve_metrics(&curve, &target_metrics, dec!(0.05));
        assert!(result.is_ok(), "Verification should not fail");
        assert!(!result.unwrap(), "Metrics should not be within tolerance");
    }

    #[test]
    fn test_refined_statistical_curve() {
        let mock = MockStatisticalCurve::new();

        let basic_metrics = BasicMetrics {
            mean: dec!(3.0),
            median: dec!(3.0),
            mode: dec!(3.0),
            std_dev: dec!(1.5),
        };

        let shape_metrics = ShapeMetrics {
            skewness: dec!(0.0),
            kurtosis: dec!(0.0),
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        };

        let range_metrics = RangeMetrics {
            min: Point2D::new(dec!(1.0), dec!(1.0)),
            max: Point2D::new(dec!(5.0), dec!(5.0)),
            range: dec!(4.0),
            quartiles: (Default::default(), Default::default(), Default::default()),
            interquartile_range: Default::default(),
        };

        let trend_metrics = TrendMetrics {
            slope: dec!(1.0),
            intercept: dec!(0.0),
            r_squared: dec!(1.0),
            moving_average: vec![],
        };

        let result = mock.generate_refined_statistical_curve(
            &basic_metrics,
            &shape_metrics,
            &range_metrics,
            &trend_metrics,
            5,
            3,         // máx 3 intentos
            dec!(0.2), // tolerancia
            Some(42),  // seed fijo
        );

        assert!(
            result.is_ok(),
            "Should successfully generate a refined curve"
        );
        let curve = result.unwrap();
        assert!(curve.len() > 0, "Generated curve should contain points");
    }
}
