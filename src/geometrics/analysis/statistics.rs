/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::geometrics::{BasicMetrics, ShapeMetrics};

/// Contains comprehensive analysis results for a curve or dataset.
///
/// # Overview
/// The `AnalysisResult` structure encapsulates both basic statistical metrics and
/// shape-related characteristics of a dataset or curve. It serves as a complete analysis
/// output that can be used to understand both the statistical properties and the 
/// geometrical behavior of the data.
///
/// This structure is particularly useful for comprehensive data analysis in financial modeling,
/// statistical analysis, and geometric curve evaluation. It combines fundamental statistical
/// insights with shape-based properties to provide a holistic view of the data.
///
/// # Fields
///
/// - **statistics**: [`BasicMetrics`]
///   - Contains fundamental statistical metrics like mean, median, mode, and standard deviation.
///   - Provides insights into the central tendency and dispersion characteristics of the dataset.
///   - Used for understanding the overall statistical properties of the analyzed data.
///
/// - **shape_metrics**: [`ShapeMetrics`]
///   - Contains shape-related properties such as skewness, kurtosis, and critical points
///     (peaks, valleys, and inflection points).
///   - Provides insights into the geometrical behavior and distribution characteristics of the curve.
///   - Used for understanding the asymmetry, tailedness, and notable features in the curve's shape.
///
/// # Applications
/// The `AnalysisResult` structure is utilized in various domains requiring both statistical
/// and geometrical analysis:
///
/// - **Financial Analysis**:
///   - Evaluate price distributions and identify critical price levels.
///   - Detect patterns in market data through both statistical and shape-based observations.
///
/// - **Data Science**:
///   - Provide comprehensive analysis of datasets, combining statistical insights with
///     distributional shape characteristics.
///
/// - **Geometric Modeling**:
///   - Analyze curve properties for more accurate modeling and representation.
///   - Identify key points that define the geometric behavior of a curve.
///
#[derive(Debug)]
pub struct AnalysisResult {
    /// The fundamental statistical metrics of the dataset, including measures of
    /// central tendency (mean, median, mode) and dispersion (standard deviation).
    pub statistics: BasicMetrics,

    /// The shape-related metrics of the curve or dataset, including distribution
    /// characteristics (skewness, kurtosis) and critical geometric points
    /// (peaks, valleys, inflection points).
    pub shape_metrics: ShapeMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometrics::analysis::metrics::{BasicMetrics, ShapeMetrics};
    use rust_decimal_macros::dec;
    use crate::curves::Point2D;

    /// Creates a sample BasicMetrics for testing
    fn create_sample_basic_metrics() -> BasicMetrics {
        BasicMetrics {
            mean: dec!(10.5),
            median: dec!(10.0),
            mode: dec!(9.0),
            std_dev: dec!(1.2),
        }
    }

    /// Creates a sample ShapeMetrics for testing
    fn create_sample_shape_metrics() -> ShapeMetrics {
        ShapeMetrics {
            skewness: dec!(0.25),
            kurtosis: dec!(3.2),
            peaks: vec![Point2D::new(dec!(1.0), dec!(10.0)), Point2D::new(dec!(5.0), dec!(15.0))],
            valleys: vec![Point2D::new(dec!(3.0), dec!(5.0))],
            inflection_points: vec![Point2D::new(dec!(2.0), dec!(7.5)), Point2D::new(dec!(4.0), dec!(12.0))],
        }
    }

    #[test]
    fn test_analysis_result_creation() {
        // Create an instance of AnalysisResult
        let result = AnalysisResult {
            statistics: create_sample_basic_metrics(),
            shape_metrics: create_sample_shape_metrics(),
        };

        // Verify that it was created successfully
        assert!(result.statistics.mean == dec!(10.5));
        assert!(result.statistics.median == dec!(10.0));
        assert!(result.statistics.mode == dec!(9.0));
        assert!(result.statistics.std_dev == dec!(1.2));

        assert!(result.shape_metrics.skewness == dec!(0.25));
        assert!(result.shape_metrics.kurtosis == dec!(3.2));
    }

    #[test]
    fn test_analysis_result_field_access() {
        let result = AnalysisResult {
            statistics: create_sample_basic_metrics(),
            shape_metrics: create_sample_shape_metrics(),
        };

        // Test accessing statistics fields
        assert_eq!(result.statistics.mean, dec!(10.5));
        assert_eq!(result.statistics.median, dec!(10.0));
        assert_eq!(result.statistics.mode, dec!(9.0));
        assert_eq!(result.statistics.std_dev, dec!(1.2));

        // Test accessing shape_metrics fields
        assert_eq!(result.shape_metrics.skewness, dec!(0.25));
        assert_eq!(result.shape_metrics.kurtosis, dec!(3.2));
        assert_eq!(result.shape_metrics.peaks.len(), 2);
        assert_eq!(result.shape_metrics.valleys.len(), 1);
        assert_eq!(result.shape_metrics.inflection_points.len(), 2);
    }

    #[test]
    fn test_analysis_result_point_access() {
        let result = AnalysisResult {
            statistics: create_sample_basic_metrics(),
            shape_metrics: create_sample_shape_metrics(),
        };

        // Test specific point values in shape_metrics
        // Check first peak
        assert_eq!(result.shape_metrics.peaks[0].x, dec!(1.0));
        assert_eq!(result.shape_metrics.peaks[0].y, dec!(10.0));

        // Check second peak
        assert_eq!(result.shape_metrics.peaks[1].x, dec!(5.0));
        assert_eq!(result.shape_metrics.peaks[1].y, dec!(15.0));

        // Check valley
        assert_eq!(result.shape_metrics.valleys[0].x, dec!(3.0));
        assert_eq!(result.shape_metrics.valleys[0].y, dec!(5.0));

        // Check inflection points
        assert_eq!(result.shape_metrics.inflection_points[0].x, dec!(2.0));
        assert_eq!(result.shape_metrics.inflection_points[0].y, dec!(7.5));
        assert_eq!(result.shape_metrics.inflection_points[1].x, dec!(4.0));
        assert_eq!(result.shape_metrics.inflection_points[1].y, dec!(12.0));
    }

    #[test]
    fn test_analysis_result_with_empty_points() {
        // Create shape metrics with empty vectors
        let shape_metrics = ShapeMetrics {
            skewness: dec!(0.1),
            kurtosis: dec!(2.5),
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        };

        let result = AnalysisResult {
            statistics: create_sample_basic_metrics(),
            shape_metrics,
        };

        // Verify the empty vectors
        assert_eq!(result.shape_metrics.peaks.len(), 0);
        assert_eq!(result.shape_metrics.valleys.len(), 0);
        assert_eq!(result.shape_metrics.inflection_points.len(), 0);

        // Other fields should still be accessible
        assert_eq!(result.shape_metrics.skewness, dec!(0.1));
        assert_eq!(result.shape_metrics.kurtosis, dec!(2.5));
    }

    // If AnalysisResult implements Debug (which it likely should)
    #[test]
    fn test_analysis_result_debug() {
        let result = AnalysisResult {
            statistics: create_sample_basic_metrics(),
            shape_metrics: create_sample_shape_metrics(),
        };

        // This just ensures that debug formatting doesn't panic
        let debug_str = format!("{:?}", result);
        assert!(!debug_str.is_empty());
    }
}