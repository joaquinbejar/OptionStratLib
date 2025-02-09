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
/// ## Components
///
/// - **Basic Metrics (`basic`)**:
///   Includes fundamental statistical measures such as mean, median, mode, and standard deviation.
///   These measures provide a quick overview of the distribution of the observations within the curve.
///
/// - **Shape Metrics (`shape`)**:
///   Captures the structural characteristics of the curve, such as skewness, kurtosis,
///   the locations of peaks and valleys, and the points where the curve inflects.
///   Useful for understanding curve symmetry, tail behavior, and general shape nuances.
///
/// - **Range Metrics (`range`)**:
///   Describes the range of the data, including minimum and maximum observed points,
///   the extent between these points, and quartile-based statistical details such as
///   interquartile range. Particularly helpful when analyzing variability in data distribution.
///
/// - **Trend Metrics (`trend`)**:
///   Measures the directional tendencies of the curve over time. Includes the slope,
///   intercept, and statistical goodness-of-fit (R² value) as well as moving averages.
///   Ideal for identifying long-term trends and evaluating the predictive nature of the curve.
///
/// - **Risk Metrics (`risk`)**:
///   Quantifies curve risk using various financial metrics, such as volatility, value-at-risk (VaR),
///   expected shortfall, beta, and the Sharpe ratio. These metrics are often used to evaluate
///   the risk-return profile in financial contexts.
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
    // Basic statistics
    pub basic: BasicMetrics,
    // Shape characteristics
    pub shape: ShapeMetrics,
    // Range information
    pub range: RangeMetrics,
    // Trend analysis
    pub trend: TrendMetrics,
    // Risk metrics
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
/// ## Fields
///
/// - **mean**: [`Decimal`]
///   - The arithmetic average of the dataset. Represents the central value where the sum
///     of all data points is divided by the total number of points.
///   - Useful for understanding the overall trend or expected value of the dataset.
///
/// - **median**: [`Decimal`]
///   - The middle value in the dataset when sorted. If the dataset has an even number
///     of elements, the median is the average of the two middle elements.
///   - A robust measure of central tendency, particularly in datasets with outliers.
///
/// - **mode**: [`Decimal`]
///   - The most frequently occurring value in the dataset. If no value repeats, the
///     mode might not be well-defined (or may represent multi-modal distributions).
///   - Useful for identifying common or dominant values in a dataset.
///
/// - **std_dev**: [`Decimal`]
///   - The standard deviation of the dataset, which quantifies the amount of variation
///     or dispersion from the mean.
///   - A key measure of data spread, often used for assessing volatility or risk in
///     financial contexts.
///
/// ## Applications
/// The `BasicMetrics` structure is utilized in various domains requiring descriptive statistics:
///
/// - **Financial Analysis**:
///   - Evaluate price variations, returns, and risks.
/// - **Scientific Research**:
///   - Summarize observations and patterns in experimental data.
/// - **Data Science**:
///   - Understand distributions, clean data, and preprocess datasets for machine learning models.
///
/// ## Integration
/// The `BasicMetrics` structure is often used as part of larger metric aggregations, such as:
/// - [`crate::geometrics::Metrics`]: Combines `BasicMetrics` with other
///   metrics for a detailed analysis of curve behavior.
/// - [`AnalysisResult`]: Provides a high-level
///   result of statistical and shape analysis for curves.
///
/// ## Remarks
/// - The values are expressed as [`Decimal`] to maintain precision, important in financial
///   computations or datasets requiring high accuracy.
/// - The `BasicMetrics` structure is immutable and can be cloned/copied, making it efficient for use in
///   concurrent or parallel computations.
///
/// ## Example Workflow
/// Typically, `BasicMetrics` is computed from a dataset using statistical functions
/// and then integrated into a more comprehensive analysis pipeline.
///
/// ## Related Concepts
/// - [`ShapeMetrics`]: Captures shape-related properties.
/// - [`RiskMetrics`]: Measures risk characteristics.
/// - [`TrendMetrics`]: Represents time-based trends.
#[derive(Clone, Copy, Debug)]
pub struct BasicMetrics {
    pub mean: Decimal,
    pub median: Decimal,
    pub mode: Decimal,
    pub std_dev: Decimal,
}

/// Represents shape-related analysis metrics for a given curve.
///
/// # Overview
/// The `ShapeMetrics` structure is designed to encapsulate key shape-related
/// properties of a curve, as well as critical points that describe the curve's
/// geometrical behavior. This structure is typically used during mathematical
/// or statistical analysis of curves.
///
/// The main properties include:
/// - **Skewness**: A measure of the asymmetry of the probability distribution
///   of a real-valued random variable. This indicates the degree and direction
///   of asymmetry of the curve's shape.
/// - **Kurtosis**: A measure of the "tailedness" of the probability distribution.
///   High kurtosis implies the presence of heavy tails, whereas low kurtosis
///   indicates light tails.
/// - **Peaks, Valleys, and Inflection Points**: These are points along the curve
///   that highlight its geometrical features, such as high/low points and areas
///   where the curvature direction changes.
///
/// # Fields
/// - **skewness (`Decimal`)**:
///   Describes the asymmetry of the curve's distribution.
///   A positive value indicates a tail on the right, while a negative value
///   implies a tail on the left.
/// - **kurtosis (`Decimal`)**:
///   Indicates the tailedness of the curve's distribution. Higher values correspond
///   to more extreme values in the tails.
/// - **peaks (`Vec<Point2D>`)**:
///   Collection of `Point2D` instances indicating the global or local maxima (peaks) along the curve.
/// - **valleys (`Vec<Point2D>`)**:
///   Collection of `Point2D` instances representing the global or local minima (valleys) along the curve.
/// - **inflection_points (`Vec<Point2D>`)**:
///   Points where the curve changes its concavity, transitioning from concave up
///   to concave down (or vice versa).
///
/// # Applications
/// This structure is a core component of the `CurveAnalysisResult` and plays an
/// essential role in:
/// - Financial metrics analysis (e.g., identifying extreme price levels or trend reversals)
/// - Mathematical curve investigation to describe its distribution and critical behaviors
/// - Geometrical understanding of complex curve shapes during interpolation or model fitting
///
/// # Example Use in Statistical Analysis
/// The `ShapeMetrics` structure is typically instantiated and analyzed as part
/// of a comprehensive curve analysis process. Through this, important statistical
/// and geometrical insights can be derived, such as:
/// - Identifying skewed or symmetric curves
/// - Evaluating the significance of outliers through kurtosis
/// - Locating key turning points like peaks, valleys, or moments of inflection
///
/// # Traits Implemented
/// - `Debug`: Provides a human-readable representation of the structure, useful for debugging.
/// - `Clone`: Enables the structure to be cloned, creating a deep copy of its contents.
///
/// # Dependencies
/// The `Point2D` type is used to represent all positional points. This ensures that
/// points leverage the same high-precision `Decimal` type for accurate representation.
#[derive(Clone, Debug)]
pub struct ShapeMetrics {
    pub skewness: Decimal,
    pub kurtosis: Decimal,
    pub peaks: Vec<Point2D>,
    pub valleys: Vec<Point2D>,
    pub inflection_points: Vec<Point2D>,
}

/// Represents statistical and range-related metrics for a dataset.
///
/// `RangeMetrics` is primarily designed to provide key statistical measures
/// related to data ranges. This structure is particularly useful when analyzing
/// numerical datasets, such as those derived from financial curves, scientific data,
/// or computational geometry. Key information such as the minimum and maximum points,
/// interquartile ranges, and quartiles are stored.
///
/// # Fields
///
/// - **min**: A `Point2D` structure representing the minimum point in the range.
///   This defines the smallest x and y coordinates observed in the data.
///   
/// - **max**: A `Point2D` structure representing the maximum point in the range.
///   This captures the largest x and y coordinates observed in the data.
///
/// - **range**: A `Decimal` value representing the difference between the maximum and
///   minimum values in the dataset. This is a key measure of variability.
///
/// - **quartiles**: A tuple `(Decimal, Decimal, Decimal)` representing the first (Q1),
///   second (Q2, or median), and third (Q3) quartiles of the dataset. These provide
///   insight into the distribution of the data within the range.
///
/// - **interquartile_range**: The interquartile range (IQR) is a `Decimal` value
///   that represents the spread between the first (Q1) and third (Q3) quartiles.
///   This measure is a robust indicator of variability, as it excludes potential
///   outliers.
///
/// # Overview
///
/// The `RangeMetrics` struct focuses on summarizing key metrics of a dataset.
/// It's especially suited for analyzing data distributions, identifying outliers
/// through IQR, and pinpointing extreme values via its `min` and `max` fields.
/// This structure ensures precision by utilizing `Decimal` for all numerical
/// values, making it a great fit for applications requiring high numerical accuracy.
///
/// Derived traits such as `Clone`, `Copy`, and `Debug` make the structure both
/// versatile and convenient to use, whether duplicating values or debugging
/// intermediate results.
#[derive(Clone, Copy, Debug)]
pub struct RangeMetrics {
    pub min: Point2D,
    pub max: Point2D,
    pub range: Decimal,
    pub quartiles: (Decimal, Decimal, Decimal),
    pub interquartile_range: Decimal,
}

/// Represents key metrics for analyzing trends within a dataset or curve.
///
/// # Overview
/// The `TrendMetrics` struct is used to store various metrics related to the
/// trend or behavior of a dataset or curve. It includes statistical values
/// necessary to describe the linear relationship and additional smoothed data
/// for further analysis.
///
/// # Fields
/// - **slope**: A `Decimal` representing the slope of the linear regression line.
///     - Indicates the rate of change in the dependent variable (y-axis) with respect
///       to the independent variable (x-axis).
/// - **intercept**: A `Decimal` representing the y-intercept of the regression line.
///     - Represents the value of the dependent variable when the independent variable
///       is zero.
/// - **r_squared**: A `Decimal` value known as the coefficient of determination.
///     - Measures the goodness-of-fit of the regression line to the data. A value
///       closer to 1 indicates a stronger fit.
/// - **moving_average**: A `Vec<Point2D>` representing the smoothed version of the data points using a moving
///   average technique.
///     - Each `Point2D` in the vector contains an x and y coordinate for the smoothed dataset.
///
/// # Purpose
/// The `TrendMetrics` struct serves a variety of use cases, including but not limited to:
/// - Evaluating financial trends (e.g., stock price movements over time).
/// - Assessing relationships in scientific or engineering datasets.
/// - Fitting data to linear models for predictive analysis.
///
/// # Usage
/// Typically, this structure is the result of a trend analysis operation, where:
/// - The slope and intercept are computed using linear regression.
/// - The `r_squared` value is derived to assess accuracy and reliability.
/// - The moving average is generated as part of smoothing operations to reduce
///   noise and identify the true trend.
///
/// # Key Characteristics
/// - **High Precision**: All numerical fields are of type `Decimal` to ensure accuracy and precision.
/// - **Debugging**: Implements the `Debug` trait to facilitate easy inspection of values during development or testing.
/// - **Clonable**: Implements `Clone` to allow duplication of an instance, useful for preserving metrics
///   snapshots at different stages of data analysis.
///
/// # Example
/// The `TrendMetrics` is commonly used in conjunction with data manipulation and statistical
/// libraries for performing tasks such as regression analysis, smoothing, and statistical modeling.
///
/// - The `slope` and `intercept` values can be used directly to create the equation of a trend line:
///   `y = slope * x + intercept`.
/// - The `r_squared` value provides insights into the reliability of the linear model.
/// - The moving average offers a simplified view of the data points, useful for visualizations and
///   identifying key trends without noise.
///
/// This struct is most useful in applications that require detailed statistical computations and
/// visualization of linear trends and smoothed data.
#[derive(Clone, Debug)]
pub struct TrendMetrics {
    pub slope: Decimal,
    pub intercept: Decimal,
    pub r_squared: Decimal,
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
/// ## Components
///
/// - **Volatility (`volatility`)**:
///   Measures the degree of variation in the returns of an asset or portfolio over time.
///   It is a widely used indicator of market risk, reflecting the extent to which returns
///   deviate from their average.
///   
/// - **Value at Risk (VaR) (`value_at_risk`)**:
///   Quantifies the maximum expected loss of an investment over a defined time horizon,
///   with a given confidence level. It is essential for understanding tail-risk and
///   evaluating potential losses in extreme conditions.
///
/// - **Expected Shortfall (ES) (`expected_shortfall`)**:
///   Represents the average loss in the worst-case scenarios (beyond the VaR threshold).
///   It provides a more comprehensive assessment of tail risk than VaR alone.
///
/// - **Beta (`beta`)**:
///   Measures the sensitivity of an asset's returns to the market returns.
///   A beta greater than 1 indicates higher volatility compared to the market,
///   while a beta less than 1 indicates lower volatility.
///   Useful for determining the systemic risk in relation to the broader market.
///
/// - **Sharpe Ratio (`sharpe_ratio`)**:
///   Indicates the risk-adjusted return by comparing the portfolio's excess return
///   (return above the risk-free rate) to its volatility. It helps to evaluate
///   whether the returns justify the associated risk profile.
///
/// ## Applications
/// The `RiskMetrics` structure is commonly utilized in:
///
/// - **Portfolio Management**:
///   To assess and balance risk-return trade-offs in portfolio construction and optimization.
/// - **Risk Management**:
///   To estimate potential losses and evaluate exposure to adverse market conditions.
/// - **Performance Analysis**:
///   To compare the investment's performance against benchmarks and alternative strategies.
/// - **Financial Modeling**:
///   To perform stress testing, scenario analysis, and forecasting using historical data.
///
/// ## Field Descriptions
///
/// - **volatility**: [`Decimal`]
///   - Indicates the standard deviation of returns, quantifying the asset's return variability.
/// - **value_at_risk**: [`Decimal`]
///   - Specifies the maximum expected loss at a specified confidence level (e.g., 95% or 99%).
/// - **expected_shortfall**: [`Decimal`]
///   - Represents the average loss in the worst scenarios beyond the VaR threshold.
/// - **beta**: [`Decimal`]
///   - Describes the systemic risk by measuring market sensitivity.
/// - **sharpe_ratio**: [`Decimal`]
///   - Indicates the risk-adjusted performance of the investment.
///
/// ## Remarks
/// The `RiskMetrics` structure is a critical component in financial analysis and
/// often serves as an input to higher-level structures like [`crate::geometrics::Metrics`]. By
/// combining these risk metrics with other statistical measures, it is possible to
/// gain a comprehensive understanding of the risk-return characteristics of an asset
/// or portfolio.
///
/// ## Related Concepts
/// - [`Metrics`]: Aggregates risk
///   alongside other statistical measures (e.g., trend, range, and shape metrics).
/// - **Financial Risk Indicators**:
///   Includes measures like maximum drawdown, Treynor ratio, and Sortino ratio
///   (typically used alongside `RiskMetrics` for advanced analysis).
///
/// ## Note
/// For accuracy and meaningful interpretation, ensure that the data used to compute
/// these metrics represents a sufficiently long-time horizon and is free from anomalies.
#[derive(Clone, Copy, Debug)]
pub struct RiskMetrics {
    pub volatility: Decimal,
    pub value_at_risk: Decimal,
    pub expected_shortfall: Decimal,
    pub beta: Decimal,
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
