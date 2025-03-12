use crate::curves::Point2D;
use crate::error::CurveError;
use crate::geometrics::AnalysisResult;
use rust_decimal::Decimal;

/// Represents a comprehensive set of statistical and analytical metrics for curve data.
///
/// # Overview
/// The `CurveMetrics` structure aggregates various metrics that describe different
/// aspects of a curve in a unified form. It provides an encapsulated representation of
/// curve information that spans different categories, including basic statistical measures,
/// shape characteristics, range details, trend analysis, and risk evaluation.
///
/// ## Usage
/// This structure is particularly helpful in domains requiring holistic curve analysis,
/// such as:
///
/// - **Financial Analysis**:
///   Used to analyze return curves and assess the risk-return trade-offs for financial products or strategies.
/// - **Data Science**:
///   Provides comprehensive insights into a dataset's distribution, shape, and trends over time.
/// - **Scientific Research**:
///   Useful for analyzing phenomena modeled by curves in domains like physics, biology, or economics.
///
/// ## Field Descriptions
///
/// - **basic**: [`BasicMetrics`]
///   - Contains mean, median, mode, and standard deviation of the dataset.
/// - **shape**: [`ShapeMetrics`]
///   - Captures the shape-related characteristics such as skewness, kurtosis, and extrema (peaks/valleys).
/// - **range**: [`RangeMetrics`]
///   - Includes range, min/max points, and quartiles for the dataset.
/// - **trend**: [`TrendMetrics`]
///   - Represents the directional tendencies, moving averages, and regression characteristics.
/// - **risk**: [`RiskMetrics`]
///   - Evaluates financial risk metrics such as volatility and Sharpe ratio.
///
/// ## Example Workflow
/// The `CurveMetrics` structure is usually constructed by combining its fields using the individual
/// metric structures (`BasicMetrics`, `ShapeMetrics`, `RangeMetrics`, `TrendMetrics`, `RiskMetrics`).
/// It is often initialized as part of a larger curve analysis operation and may be transformed or queried for
/// generating insights.
///
/// ## Related Concepts
/// - [`BasicMetrics`]: Encodes fundamental statistics.
/// - [`ShapeMetrics`]: Provides characteristics associated with curve shape.
/// - [`RangeMetrics`]: Range and quartile information for a curve.
/// - [`TrendMetrics`]: Trendline and regression fit metrics.
/// - [`RiskMetrics`]: Quantifies financial risk.
///
/// ## Examples of Associated Tools
/// - Statistical Analysis: Plots, descriptive statistics, trend analysis.
/// - Visualizations: Understand curve behavior (e.g., peaks, valleys).
/// - Financial Metrics: Sharpe ratio, beta, and VaR for understanding portfolio risks.
///
/// ## Remarks
/// The `CurveMetrics` struct is designed to be reusable across various analytical contexts,
/// providing a versatile and standardized way to represent curve characteristics.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// - **Basic Metrics (`basic`)**:
    ///   Includes fundamental statistical measures such as mean, median, mode, and standard deviation.
    ///   These measures provide a quick overview of the distribution of the observations within the curve.
    pub basic: BasicMetrics,
    /// - **Shape Metrics (`shape`)**:
    ///   Captures the structural characteristics of the curve, such as skewness, kurtosis,
    ///   the locations of peaks and valleys, and the points where the curve inflects.
    ///   Useful for understanding curve symmetry, tail behavior, and general shape nuances.
    pub shape: ShapeMetrics,
    /// - **Range Metrics (`range`)**:
    ///   Describes the range of the data, including minimum and maximum observed points,
    ///   the extent between these points, and quartile-based statistical details such as
    ///   interquartile range. Particularly helpful when analyzing variability in data distribution.
    pub range: RangeMetrics,
    /// - **Trend Metrics (`trend`)**:
    ///   Measures the directional tendencies of the curve over time. Includes the slope,
    ///   intercept, and statistical goodness-of-fit (R² value) as well as moving averages.
    ///   Ideal for identifying long-term trends and evaluating the predictive nature of the curve.
    pub trend: TrendMetrics,
    /// - **Risk Metrics (`risk`)**:
    ///   Quantifies curve risk using various financial metrics, such as volatility, value-at-risk (VaR),
    ///   expected shortfall, beta, and the Sharpe ratio. These metrics are often used to evaluate
    ///   the risk-return profile in financial contexts.
    pub risk: RiskMetrics,
}

/// Represents a set of metrics associated with analyzing and interpreting a curve.
///
/// This structure encapsulates multiple types of metrics, each responsible for
/// a specific aspect of curve analysis. These include basic statistical measures,
/// shape-related properties, range characteristics, trend analysis, and risk factors.
///
/// ## Fields
/// - `basic`: Basic statistical metrics such as mean, median, mode, and standard deviation.
/// - `shape`: Metrics that describe the shape of the curve, such as skewness, kurtosis,
///   and points of interest (peaks, valleys, and inflection points).
/// - `range`: Range-based metrics specifying properties like the minimum and maximum
///   values on the curve, quartiles, and interquartile range.
/// - `trend`: Metrics related to overall trends in the curve, such as slope,
///   intercept, R-squared value, and moving average data points.
/// - `risk`: Risk-related metrics, including volatility, value at risk (VaR),
///   expected shortfall, beta, and the Sharpe ratio.
///
/// ## Notes
/// This implementation ensures modularity by separating distinct aspects of curve
/// analysis into specific metric structures. These metrics can be used individually
/// or collectively for advanced data analysis or curve interpretation tasks.
///
/// ## Related Structures
/// - [`BasicMetrics`]: Provides basic statistical metrics about the dataset.
/// - [`ShapeMetrics`]: Describes the geometric properties of the curve.
/// - [`RangeMetrics`]: Assesses the range and quartile characteristics of the curve.
/// - [`TrendMetrics`]: Analyzes trends within the data to understand directional behavior.
/// - [`RiskMetrics`]: Highlights risk-based metrics for financial, statistical, or analytical use cases.
/// - [`AnalysisResult`]: The result type combining key metrics into a single analytic perspective.
/// - [`CurveError`]: Represents potential errors that may arise during curve analysis operations.
impl Metrics {
    /// ### `new`
    /// Constructs a new instance of `CurveMetrics` and initializes all relevant fields with the provided
    /// metric structures.
    ///
    /// #### Parameters:
    /// - `basic`: An instance of [`BasicMetrics`], holding essential statistical information.
    /// - `shape`: An instance of [`ShapeMetrics`], measuring the geometric properties of the curve.
    /// - `range`: An instance of [`RangeMetrics`], describing the range and distribution of the curve.
    /// - `trend`: An instance of [`TrendMetrics`], detailing trend-based analytical results.
    /// - `risk`: An instance of [`RiskMetrics`], specifying the risk characteristics related to the curve.
    ///
    /// #### Returns:
    /// - A new `CurveMetrics` instance containing the provided metrics.
    ///
    pub fn new(
        basic: BasicMetrics,
        shape: ShapeMetrics,
        range: RangeMetrics,
        trend: TrendMetrics,
        risk: RiskMetrics,
    ) -> Self {
        Self {
            basic,
            shape,
            range,
            trend,
            risk,
        }
    }

    /// ### `curve_analysis_result`
    /// Generates a high-level analysis result from the metrics encapsulated within the `CurveMetrics` instance.
    ///
    /// #### Returns:
    /// - `Ok(CurveAnalysisResult)`: A result that contains analyzed data in the form of a
    ///   [`AnalysisResult`] structure with basic statistics and shape metrics.
    /// - `Err(CurvesError)`: An error of type [`CurveError`] when analysis fails.
    ///
    /// The result provides the basic statistical measures (`BasicMetrics`) and
    /// shape metrics (`ShapeMetrics`) that were part of the `CurveMetrics` instance.
    ///
    pub fn analysis_result(&self) -> Result<AnalysisResult, CurveError> {
        Ok(AnalysisResult {
            statistics: self.basic,
            shape_metrics: self.shape.clone(),
        })
    }
}

/// Represents a collection of fundamental statistical metrics.
///
/// # Overview
/// The `BasicMetrics` structure encapsulates the core descriptive statistics
/// for a dataset, providing a quick summary of its central tendency, spread,
/// and distribution properties. These metrics are widely used in statistical analysis,
/// curve analysis, and various financial or scientific computations.
///
/// # Fields
///
/// * `mean` - The arithmetic average of the dataset. Represents the central value where the sum
///   of all data points is divided by the total number of points.
///   Useful for understanding the overall trend or expected value of the dataset.
///
/// * `median` - The middle value in the dataset when sorted. If the dataset has an even number
///   of elements, the median is the average of the two middle elements.
///   A robust measure of central tendency, particularly in datasets with outliers.
///
/// * `mode` - The most frequently occurring value in the dataset. If no value repeats, the
///   mode might not be well-defined (or may represent multi-modal distributions).
///   Useful for identifying common or dominant values in a dataset.
///
/// * `std_dev` - The standard deviation of the dataset, which quantifies the amount of variation
///   or dispersion from the mean.
///   A key measure of data spread, often used for assessing volatility or risk in
///   financial contexts.
///
/// # Applications
/// The `BasicMetrics` structure is utilized in various domains requiring descriptive statistics:
///
/// - **Financial Analysis**:
///   - Evaluate price variations, returns, and risks.
///
/// - **Scientific Research**:
///   - Summarize observations and patterns in experimental data.
///
/// - **Data Science**:
///   - Understand distributions, clean data, and preprocess datasets for machine learning models.
///
/// # Integration
/// The `BasicMetrics` structure is often used as part of larger metric aggregations, such as:
/// - [`Metrics`]: Combines `BasicMetrics` with other
///   metrics for a detailed analysis of curve behavior.
/// - [`AnalysisResult`]: Provides a high-level
///   result of statistical and shape analysis for curves.
///
/// # Remarks
/// - The values are expressed as [`Decimal`] to maintain precision, important in financial
///   computations or datasets requiring high accuracy.
/// - The `BasicMetrics` structure is immutable and can be cloned/copied, making it efficient for use in
///   concurrent or parallel computations.
///
/// # Example Workflow
/// Typically, `BasicMetrics` is computed from a dataset using statistical functions
/// and then integrated into a more comprehensive analysis pipeline.
///
/// # Related Concepts
/// - [`ShapeMetrics`]: Captures shape-related properties like skewness and kurtosis.
/// - [`RiskMetrics`]: Measures risk characteristics of financial instruments.
/// - [`TrendMetrics`]: Represents time-based trends in data series.
#[derive(Clone, Copy, Debug)]
pub struct BasicMetrics {
    /// The arithmetic mean (average) of the dataset.
    /// Calculated by summing all values and dividing by the count of values.
    /// Provides a measure of central tendency that is affected by all values,
    /// including outliers.
    pub mean: Decimal,

    /// The median value of the dataset.
    /// Represents the middle value when all data points are arranged in ascending order.
    /// For datasets with even count, it's the average of the two middle values.
    /// Less sensitive to outliers than the mean.
    pub median: Decimal,

    /// The most frequently occurring value in the dataset.
    /// If multiple values occur with equal frequency, this typically represents
    /// the first encountered mode or a calculated central mode.
    /// For multi-modal distributions, this is a simplification.
    pub mode: Decimal,

    /// The standard deviation of the dataset.
    /// Measures the amount of variation or dispersion from the mean.
    /// A low standard deviation indicates values tend to be close to the mean,
    /// while a high standard deviation indicates values are spread over a wider range.
    pub std_dev: Decimal,
}

/// Represents shape-related analysis metrics for a given curve.
///
/// # Overview
/// The `ShapeMetrics` structure encapsulates shape-related properties and critical
/// points describing a curve's geometrical behavior. It's primarily used for 
/// mathematical and statistical analysis of curves representing data distributions
/// or mathematical functions.
///
/// # Fields
/// * `skewness` - A measure of distribution asymmetry. Positive values indicate right
///   tailing while negative values indicate left tailing.
///
/// * `kurtosis` - A measure of the "tailedness" or presence of outliers in the distribution.
///   Higher values indicate heavier tails and more outliers compared to a normal distribution.
///
/// * `peaks` - Collection of points representing local or global maxima along the curve,
///   stored as `Point2D` coordinates.
///
/// * `valleys` - Collection of points representing local or global minima along the curve,
///   stored as `Point2D` coordinates.
///
/// * `inflection_points` - Points where the curve changes concavity (transitioning from
///   concave up to concave down or vice versa), stored as `Point2D` coordinates.
///
/// # Applications
/// This structure is commonly used in:
/// - Financial analysis to identify trend reversals or extreme price levels
/// - Statistical distribution analysis to characterize data shape properties
/// - Mathematical modeling to identify critical points in curve functions
/// - Visualization and interpretation of complex curve behaviors
///
/// # Relationship with Other Structures
/// `ShapeMetrics` is typically part of the larger `AnalysisResult` structure
/// that provides comprehensive curve analysis. It uses `Point2D` to represent
/// all positional data with high-precision `Decimal` values.
#[derive(Clone, Debug)]
pub struct ShapeMetrics {
    /// Describes the asymmetry of the curve's distribution.
    /// A positive value indicates a right tail (right-skewed distribution),
    /// while a negative value indicates a left tail (left-skewed distribution).
    /// A value close to zero suggests symmetry.
    pub skewness: Decimal,

    /// Indicates the "tailedness" or presence of outliers in the curve's distribution.
    /// Higher values correspond to heavier tails and more outliers compared to
    /// a normal distribution. Standard normal distribution has kurtosis of 3.0.
    /// Often expressed as excess kurtosis (kurtosis - 3).
    pub kurtosis: Decimal,

    /// Collection of `Point2D` instances representing local or global maxima (peaks)
    /// along the curve. These points have higher y-values than their immediate
    /// neighboring points.
    pub peaks: Vec<Point2D>,

    /// Collection of `Point2D` instances representing local or global minima (valleys)
    /// along the curve. These points have lower y-values than their immediate
    /// neighboring points.
    pub valleys: Vec<Point2D>,

    /// Points where the curve changes concavity, transitioning from concave up to
    /// concave down (or vice versa). At these points, the second derivative equals zero
    /// while changing sign.
    pub inflection_points: Vec<Point2D>,
}

/// Represents statistical and range-related metrics for a dataset.
///
/// `RangeMetrics` provides key statistical measures related to data ranges and distributions.
/// This structure is particularly useful when analyzing numerical datasets from financial curves,
/// scientific measurements, or computational geometry applications.
///
/// # Fields
///
/// * `min` - The minimum point in the dataset, containing both x and y coordinates as the smallest values.
///   This defines the lower boundary of the data range.
///
/// * `max` - The maximum point in the dataset, containing both x and y coordinates as the largest values.
///   This defines the upper boundary of the data range.
///
/// * `range` - The numerical difference between the maximum and minimum values.
///   Provides a simple measure of data dispersion or spread.
///
/// * `quartiles` - A tuple containing first quartile (Q1), median (Q2), and third quartile (Q3).
///   These values divide the dataset into four equal parts, providing insight into data distribution.
///
/// * `interquartile_range` - The difference between the third and first quartiles (Q3 - Q1).
///   A robust statistical measure of dispersion that is less sensitive to outliers than the full range.
///
/// # Usage
///
/// `RangeMetrics` is typically calculated as part of a larger statistical analysis workflow.
/// It provides essential information for box plots, outlier detection, and distribution analysis.
///
/// This structure is part of the curves analysis module that provides comprehensive
/// statistical and financial analysis tools for mathematical curves.
#[derive(Clone, Copy, Debug)]
pub struct RangeMetrics {
    /// The minimum point in the dataset, containing the smallest x and y coordinates observed.
    pub min: Point2D,

    /// The maximum point in the dataset, containing the largest x and y coordinates observed.
    pub max: Point2D,

    /// The numerical difference between the maximum and minimum values, 
    /// representing the total span of the data.
    pub range: Decimal,

    /// A tuple of three values representing the first quartile (Q1), 
    /// median (Q2), and third quartile (Q3) of the dataset.
    pub quartiles: (Decimal, Decimal, Decimal),

    /// The difference between the third and first quartiles (Q3 - Q1),
    /// providing a measure of statistical dispersion that ignores extremes.
    pub interquartile_range: Decimal,
}

/// Represents key metrics for analyzing trends within a dataset or curve.
///
/// # Overview
/// This structure stores comprehensive metrics that describe the linear relationship
/// and trend behavior in a dataset. It provides statistical values derived from
/// linear regression analysis along with smoothed data points for visualization
/// and further analysis.
///
/// # Fields
/// - **slope**: The rate of change in the dependent variable (y-axis) with respect
///   to the independent variable (x-axis), calculated from linear regression.
/// - **intercept**: The y-coordinate where the regression line intersects the y-axis,
///   representing the value when x = 0.
/// - **r_squared**: The coefficient of determination that measures how well the
///   data fits the regression model, with values closer to 1 indicating a better fit.
/// - **moving_average**: A series of smoothed data points derived from the original
///   dataset to reduce noise and highlight the underlying trend pattern.
///
/// # Purpose
/// `TrendMetrics` is particularly useful for:
/// - Analyzing time-series data in financial markets
/// - Evaluating performance trends over time
/// - Quantifying the strength and direction of relationships between variables
/// - Providing a basis for predictive modeling and forecasting
///
/// This structure is a key component of the curve analysis module and works
/// alongside other metric types to provide comprehensive statistical analysis.
#[derive(Clone, Debug)]
pub struct TrendMetrics {
    /// The slope coefficient of the linear regression line, indicating the
    /// rate and direction of change in the dataset.
    pub slope: Decimal,

    /// The y-intercept of the regression line, showing the baseline value
    /// when the independent variable equals zero.
    pub intercept: Decimal,

    /// The coefficient of determination (R²), measuring how well the regression
    /// line fits the actual data points. Values range from 0 to 1, where 1
    /// represents a perfect fit.
    pub r_squared: Decimal,

    /// A collection of points representing the smoothed version of the original
    /// data, calculated using moving average techniques to reduce noise and
    /// highlight the underlying trend.
    pub moving_average: Vec<Point2D>,
}

/// Represents a collection of key financial risk metrics used in risk analysis
/// and performance evaluation.
///
/// # Overview
/// The `RiskMetrics` structure encapsulates a set of quantitative measures
/// that assess the risk and performance characteristics of a financial portfolio,
/// asset, or investment strategy. These metrics provide insights into the
/// variability, potential losses, and overall return of the analyzed entity.
///
/// # Applications
/// The `RiskMetrics` structure is commonly utilized in:
///
/// - **Portfolio Management**: To assess and balance risk-return trade-offs
/// - **Risk Management**: To estimate potential losses and evaluate market exposures
/// - **Performance Analysis**: To compare against benchmarks and alternative strategies
/// - **Financial Modeling**: To perform stress testing and scenario analysis
///
/// # Related Concepts
/// - [`Metrics`]: Aggregates risk metrics alongside other statistical measures
/// - **Financial Risk Indicators**: Additional measures like maximum drawdown,
///   Treynor ratio, and Sortino ratio
///
/// # Note
/// For accuracy, ensure the data used to compute these metrics represents a
/// sufficiently long time horizon and is free from anomalies.
#[derive(Clone, Copy, Debug)]
pub struct RiskMetrics {
    /// Measures the degree of variation in returns over time.
    /// Represents the standard deviation of returns and is a widely used
    /// indicator of market risk.
    pub volatility: Decimal,

    /// Quantifies the maximum expected loss over a defined time horizon
    /// with a given confidence level (typically 95% or 99%).
    /// Essential for understanding tail-risk and evaluating potential losses.
    pub value_at_risk: Decimal,

    /// Represents the average loss in worst-case scenarios beyond the VaR threshold.
    /// Provides a more comprehensive assessment of tail risk than VaR alone.
    pub expected_shortfall: Decimal,

    /// Measures the sensitivity of an asset's returns to market returns.
    /// A beta greater than 1 indicates higher volatility compared to the market,
    /// while a beta less than 1 indicates lower volatility.
    pub beta: Decimal,

    /// Indicates the risk-adjusted return by comparing the portfolio's excess return
    /// (return above the risk-free rate) to its volatility.
    /// Helps evaluate whether returns justify the associated risk profile.
    pub sharpe_ratio: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    use rust_decimal_macros::dec;

    // Helper function to create a Point2D for testing
    fn create_test_point(x: f64, y: f64) -> Point2D {
        Point2D::new(Decimal::from_f64(x).unwrap(), Decimal::from_f64(y).unwrap())
    }

    mod test_basic_metrics {
        use super::*;

        #[test]
        fn test_basic_metrics_creation() {
            let metrics = BasicMetrics {
                mean: dec!(10.5),
                median: dec!(10.0),
                mode: dec!(9.0),
                std_dev: dec!(1.2),
            };

            assert_eq!(metrics.mean, dec!(10.5));
            assert_eq!(metrics.median, dec!(10.0));
            assert_eq!(metrics.mode, dec!(9.0));
            assert_eq!(metrics.std_dev, dec!(1.2));
        }

        #[test]
        fn test_basic_metrics_clone() {
            let metrics = BasicMetrics {
                mean: dec!(10.5),
                median: dec!(10.0),
                mode: dec!(9.0),
                std_dev: dec!(1.2),
            };

            let cloned_metrics = metrics;
            assert_eq!(metrics.mean, cloned_metrics.mean);
            assert_eq!(metrics.median, cloned_metrics.median);
            assert_eq!(metrics.mode, cloned_metrics.mode);
            assert_eq!(metrics.std_dev, cloned_metrics.std_dev);
        }
    }

    mod test_shape_metrics {
        use super::*;

        #[test]
        fn test_shape_metrics_creation() {
            let metrics = ShapeMetrics {
                skewness: dec!(0.5),
                kurtosis: dec!(3.0),
                peaks: vec![create_test_point(1.0, 10.0)],
                valleys: vec![create_test_point(2.0, 5.0)],
                inflection_points: vec![create_test_point(1.5, 7.5)],
            };

            assert_eq!(metrics.skewness, dec!(0.5));
            assert_eq!(metrics.kurtosis, dec!(3.0));
            assert_eq!(metrics.peaks.len(), 1);
            assert_eq!(metrics.valleys.len(), 1);
            assert_eq!(metrics.inflection_points.len(), 1);
        }

        #[test]
        fn test_shape_metrics_clone() {
            let metrics = ShapeMetrics {
                skewness: dec!(0.5),
                kurtosis: dec!(3.0),
                peaks: vec![create_test_point(1.0, 10.0)],
                valleys: vec![create_test_point(2.0, 5.0)],
                inflection_points: vec![create_test_point(1.5, 7.5)],
            };

            let cloned_metrics = metrics.clone();
            assert_eq!(metrics.skewness, cloned_metrics.skewness);
            assert_eq!(metrics.kurtosis, cloned_metrics.kurtosis);
            assert_eq!(metrics.peaks.len(), cloned_metrics.peaks.len());
            assert_eq!(metrics.valleys.len(), cloned_metrics.valleys.len());
            assert_eq!(
                metrics.inflection_points.len(),
                cloned_metrics.inflection_points.len()
            );
        }
    }

    mod test_range_metrics {
        use super::*;

        #[test]
        fn test_range_metrics_creation() {
            let metrics = RangeMetrics {
                min: create_test_point(0.0, 5.0),
                max: create_test_point(10.0, 15.0),
                range: dec!(10.0),
                quartiles: (dec!(7.0), dec!(10.0), dec!(13.0)),
                interquartile_range: dec!(6.0),
            };

            assert_eq!(metrics.min.x, dec!(0.0));
            assert_eq!(metrics.max.x, dec!(10.0));
            assert_eq!(metrics.range, dec!(10.0));
            assert_eq!(metrics.quartiles.0, dec!(7.0));
            assert_eq!(metrics.interquartile_range, dec!(6.0));
        }

        #[test]
        fn test_range_metrics_clone() {
            let metrics = RangeMetrics {
                min: create_test_point(0.0, 5.0),
                max: create_test_point(10.0, 15.0),
                range: dec!(10.0),
                quartiles: (dec!(7.0), dec!(10.0), dec!(13.0)),
                interquartile_range: dec!(6.0),
            };

            let cloned_metrics = metrics;
            assert_eq!(metrics.min.x, cloned_metrics.min.x);
            assert_eq!(metrics.max.x, cloned_metrics.max.x);
            assert_eq!(metrics.range, cloned_metrics.range);
            assert_eq!(metrics.quartiles.0, cloned_metrics.quartiles.0);
            assert_eq!(
                metrics.interquartile_range,
                cloned_metrics.interquartile_range
            );
        }
    }

    mod test_trend_metrics {
        use super::*;

        #[test]
        fn test_trend_metrics_creation() {
            let metrics = TrendMetrics {
                slope: dec!(1.5),
                intercept: dec!(2.0),
                r_squared: dec!(0.95),
                moving_average: vec![create_test_point(1.0, 3.5), create_test_point(2.0, 5.0)],
            };

            assert_eq!(metrics.slope, dec!(1.5));
            assert_eq!(metrics.intercept, dec!(2.0));
            assert_eq!(metrics.r_squared, dec!(0.95));
            assert_eq!(metrics.moving_average.len(), 2);
        }

        #[test]
        fn test_trend_metrics_clone() {
            let metrics = TrendMetrics {
                slope: dec!(1.5),
                intercept: dec!(2.0),
                r_squared: dec!(0.95),
                moving_average: vec![create_test_point(1.0, 3.5), create_test_point(2.0, 5.0)],
            };

            let cloned_metrics = metrics.clone();
            assert_eq!(metrics.slope, cloned_metrics.slope);
            assert_eq!(metrics.intercept, cloned_metrics.intercept);
            assert_eq!(metrics.r_squared, cloned_metrics.r_squared);
            assert_eq!(
                metrics.moving_average.len(),
                cloned_metrics.moving_average.len()
            );
        }
    }

    mod test_risk_metrics {
        use super::*;

        #[test]
        fn test_risk_metrics_creation() {
            let metrics = RiskMetrics {
                volatility: dec!(0.15),
                value_at_risk: dec!(0.05),
                expected_shortfall: dec!(0.07),
                beta: dec!(1.2),
                sharpe_ratio: dec!(2.5),
            };

            assert_eq!(metrics.volatility, dec!(0.15));
            assert_eq!(metrics.value_at_risk, dec!(0.05));
            assert_eq!(metrics.expected_shortfall, dec!(0.07));
            assert_eq!(metrics.beta, dec!(1.2));
            assert_eq!(metrics.sharpe_ratio, dec!(2.5));
        }

        #[test]
        fn test_risk_metrics_clone() {
            let metrics = RiskMetrics {
                volatility: dec!(0.15),
                value_at_risk: dec!(0.05),
                expected_shortfall: dec!(0.07),
                beta: dec!(1.2),
                sharpe_ratio: dec!(2.5),
            };

            let cloned_metrics = metrics;
            assert_eq!(metrics.volatility, cloned_metrics.volatility);
            assert_eq!(metrics.value_at_risk, cloned_metrics.value_at_risk);
            assert_eq!(
                metrics.expected_shortfall,
                cloned_metrics.expected_shortfall
            );
            assert_eq!(metrics.beta, cloned_metrics.beta);
            assert_eq!(metrics.sharpe_ratio, cloned_metrics.sharpe_ratio);
        }
    }

    mod test_curve_metrics {
        use super::*;

        fn create_test_curve_metrics() -> Metrics {
            Metrics {
                basic: BasicMetrics {
                    mean: dec!(10.5),
                    median: dec!(10.0),
                    mode: dec!(9.0),
                    std_dev: dec!(1.2),
                },
                shape: ShapeMetrics {
                    skewness: dec!(0.5),
                    kurtosis: dec!(3.0),
                    peaks: vec![create_test_point(1.0, 10.0)],
                    valleys: vec![create_test_point(2.0, 5.0)],
                    inflection_points: vec![create_test_point(1.5, 7.5)],
                },
                range: RangeMetrics {
                    min: create_test_point(0.0, 5.0),
                    max: create_test_point(10.0, 15.0),
                    range: dec!(10.0),
                    quartiles: (dec!(7.0), dec!(10.0), dec!(13.0)),
                    interquartile_range: dec!(6.0),
                },
                trend: TrendMetrics {
                    slope: dec!(1.5),
                    intercept: dec!(2.0),
                    r_squared: dec!(0.95),
                    moving_average: vec![create_test_point(1.0, 3.5), create_test_point(2.0, 5.0)],
                },
                risk: RiskMetrics {
                    volatility: dec!(0.15),
                    value_at_risk: dec!(0.05),
                    expected_shortfall: dec!(0.07),
                    beta: dec!(1.2),
                    sharpe_ratio: dec!(2.5),
                },
            }
        }

        #[test]
        fn test_curve_metrics_creation() {
            let metrics = create_test_curve_metrics();

            assert_eq!(metrics.basic.mean, dec!(10.5));
            assert_eq!(metrics.shape.skewness, dec!(0.5));
            assert_eq!(metrics.range.range, dec!(10.0));
            assert_eq!(metrics.trend.slope, dec!(1.5));
            assert_eq!(metrics.risk.volatility, dec!(0.15));
        }

        #[test]
        fn test_curve_metrics_clone() {
            let metrics = create_test_curve_metrics();
            let cloned_metrics = metrics.clone();

            assert_eq!(metrics.basic.mean, cloned_metrics.basic.mean);
            assert_eq!(metrics.shape.skewness, cloned_metrics.shape.skewness);
            assert_eq!(metrics.range.range, cloned_metrics.range.range);
            assert_eq!(metrics.trend.slope, cloned_metrics.trend.slope);
            assert_eq!(metrics.risk.volatility, cloned_metrics.risk.volatility);
        }

        #[test]
        fn test_curve_analysis_result() {
            let metrics = create_test_curve_metrics();
            let result = metrics.analysis_result();

            assert!(result.is_ok());
            let analysis = result.unwrap();

            assert_eq!(analysis.statistics.mean, metrics.basic.mean);
            assert_eq!(analysis.shape_metrics.skewness, metrics.shape.skewness);
        }

        #[test]
        fn test_curve_metrics_new() {
            let basic = BasicMetrics {
                mean: dec!(10.5),
                median: dec!(10.0),
                mode: dec!(9.0),
                std_dev: dec!(1.2),
            };

            let shape = ShapeMetrics {
                skewness: dec!(0.5),
                kurtosis: dec!(3.0),
                peaks: vec![create_test_point(1.0, 10.0)],
                valleys: vec![create_test_point(2.0, 5.0)],
                inflection_points: vec![create_test_point(1.5, 7.5)],
            };

            let range = RangeMetrics {
                min: create_test_point(0.0, 5.0),
                max: create_test_point(10.0, 15.0),
                range: dec!(10.0),
                quartiles: (dec!(7.0), dec!(10.0), dec!(13.0)),
                interquartile_range: dec!(6.0),
            };

            let trend = TrendMetrics {
                slope: dec!(1.5),
                intercept: dec!(2.0),
                r_squared: dec!(0.95),
                moving_average: vec![create_test_point(1.0, 3.5), create_test_point(2.0, 5.0)],
            };

            let risk = RiskMetrics {
                volatility: dec!(0.15),
                value_at_risk: dec!(0.05),
                expected_shortfall: dec!(0.07),
                beta: dec!(1.2),
                sharpe_ratio: dec!(2.5),
            };

            let metrics = Metrics::new(basic, shape, range, trend, risk);

            assert_eq!(metrics.basic.mean, dec!(10.5));
            assert_eq!(metrics.shape.skewness, dec!(0.5));
            assert_eq!(metrics.range.range, dec!(10.0));
            assert_eq!(metrics.trend.slope, dec!(1.5));
            assert_eq!(metrics.risk.volatility, dec!(0.15));
        }
    }
}
