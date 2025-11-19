:::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [MetricsExtractor]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#30-112){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait MetricsExtractor: Len {
    // Required methods
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, MetricsError>;
    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError>;
    fn compute_range_metrics(&self) -> Result<RangeMetrics, MetricsError>;
    fn compute_trend_metrics(&self) -> Result<TrendMetrics, MetricsError>;
    fn compute_risk_metrics(&self) -> Result<RiskMetrics, MetricsError>;

    // Provided methods
    fn compute_curve_metrics(&self) -> Result<Metrics, MetricsError> { ... }
    fn compute_surface_metrics(&self) -> Result<Metrics, MetricsError> { ... }
}
```

Expand description

::: docblock
A trait for extracting comprehensive metrics from a curve.

## [§](#overview){.doc-anchor}Overview

The `CurveMetricsExtractor` trait provides methods to compute various
statistical, analytical, and risk-related metrics for a given curve. It
allows for a systematic and extensible approach to curve analysis across
different curve types and contexts.

## [§](#methods){.doc-anchor}Methods

### [§](#metric-computation-methods){.doc-anchor}Metric Computation Methods

- `compute_basic_metrics`: Calculates fundamental statistical measures.
- `compute_shape_metrics`: Analyzes curve shape characteristics.
- `compute_range_metrics`: Determines range and distribution properties.
- `compute_trend_metrics`: Evaluates directional and regression-related
  metrics.
- `compute_risk_metrics`: Quantifies financial and statistical risk
  indicators.

### [§](#comprehensive-metrics){.doc-anchor}Comprehensive Metrics

- `compute_curve_metrics`: Computes all metrics and combines them into a
  `CurveMetrics` struct.

## [§](#usage){.doc-anchor}Usage

Implement this trait for specific curve types or analysis strategies to
provide custom metric computation logic tailored to different domains or
requirements.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::::::::: methods
::: {#tymethod.compute_basic_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#36){.src
.rightside}

#### fn [compute_basic_metrics](#tymethod.compute_basic_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[BasicMetrics](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_basic_metricsself---resultbasicmetrics-metricserror .code-header}
:::

::: docblock
Computes basic statistical metrics for the curve.

##### [§](#returns){.doc-anchor}Returns

- `Ok(BasicMetrics)`: Struct containing mean, median, mode, and standard
  deviation.
- `Err(CurvesError)`: If metrics computation fails.
:::

::: {#tymethod.compute_shape_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#43){.src
.rightside}

#### fn [compute_shape_metrics](#tymethod.compute_shape_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[ShapeMetrics](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_shape_metricsself---resultshapemetrics-metricserror .code-header}
:::

::: docblock
Computes shape-related metrics for the curve.

##### [§](#returns-1){.doc-anchor}Returns

- `Ok(ShapeMetrics)`: Struct containing skewness, kurtosis, peaks,
  valleys, and inflection points.
- `Err(CurvesError)`: If metrics computation fails.
:::

::: {#tymethod.compute_range_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#50){.src
.rightside}

#### fn [compute_range_metrics](#tymethod.compute_range_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RangeMetrics](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_range_metricsself---resultrangemetrics-metricserror .code-header}
:::

::: docblock
Computes range-related metrics for the curve.

##### [§](#returns-2){.doc-anchor}Returns

- `Ok(RangeMetrics)`: Struct containing min/max points, range,
  quartiles, and interquartile range.
- `Err(CurvesError)`: If metrics computation fails.
:::

::: {#tymethod.compute_trend_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#57){.src
.rightside}

#### fn [compute_trend_metrics](#tymethod.compute_trend_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[TrendMetrics](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_trend_metricsself---resulttrendmetrics-metricserror .code-header}
:::

::: docblock
Computes trend-related metrics for the curve.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok(TrendMetrics)`: Struct containing slope, intercept, R-squared, and
  moving average.
- `Err(CurvesError)`: If metrics computation fails.
:::

::: {#tymethod.compute_risk_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#64){.src
.rightside}

#### fn [compute_risk_metrics](#tymethod.compute_risk_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RiskMetrics](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_risk_metricsself---resultriskmetrics-metricserror .code-header}
:::

::: docblock
Computes risk-related metrics for the curve.

##### [§](#returns-4){.doc-anchor}Returns

- `Ok(RiskMetrics)`: Struct containing volatility, VaR, expected
  shortfall, beta, and Sharpe ratio.
- `Err(CurvesError)`: If metrics computation fails.
:::
:::::::::::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::: methods
::: {#method.compute_curve_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#71-79){.src
.rightside}

#### fn [compute_curve_metrics](#method.compute_curve_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_curve_metricsself---resultmetrics-metricserror .code-header}
:::

::: docblock
Computes and aggregates all curve metrics into a comprehensive
`CurveMetrics` struct.

##### [§](#returns-5){.doc-anchor}Returns

- `Ok(CurveMetrics)`: A struct containing all computed metrics.
- `Err(CurvesError)`: If any metrics computation fails.
:::

::: {#method.compute_surface_metrics .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#104-111){.src
.rightside}

#### fn [compute_surface_metrics](#method.compute_surface_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_surface_metricsself---resultmetrics-metricserror .code-header}
:::

::: docblock
Computes comprehensive metrics for a surface representation.

This method aggregates multiple specialized metric calculations into a
single cohesive result. It collects basic statistical properties, shape
characteristics, range information, trend analysis, and risk assessments
to provide a complete analytical view of the surface.

##### [§](#returns-6){.doc-anchor}Returns

- `Ok(Metrics)`: A composite structure containing all computed metrics
  categories:
  - Basic metrics (statistical fundamentals)
  - Shape metrics (geometric properties)
  - Range metrics (distribution characteristics)
  - Trend metrics (directional patterns)
  - Risk metrics (uncertainty measurements)
- `Err(MetricsError)`: If any of the individual metric computations
  fails

##### [§](#errors){.doc-anchor}Errors

This method can return various error types depending on which
computation fails:

- `MetricsError::BasicError` - If basic metrics computation fails
- `MetricsError::ShapeError` - If shape metrics computation fails
- `MetricsError::RangeError` - If range metrics computation fails
- `MetricsError::TrendError` - If trend metrics computation fails
- `MetricsError::RiskError` - If risk metrics computation fails
:::
:::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::: {#implementors-list}
:::: {#impl-MetricsExtractor-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#970-1263){.src
.rightside}[§](#impl-MetricsExtractor-for-Curve){.anchor}

### impl [MetricsExtractor](trait.MetricsExtractor.html "trait optionstratlib::geometrics::MetricsExtractor"){.trait} for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-metricsextractor-for-curve .code-header}

::: docblock
A default implementation for the `Curve` type using a provided default
strategy.
:::
::::

::: docblock
This implementation provides a basic approach to computing curve metrics
by using interpolation and statistical methods available in the standard
curve analysis library.

#### [§](#note){.doc-anchor}Note

This is a minimal implementation that may need to be customized or
enhanced based on specific requirements or domain-specific analysis
needs.
:::

::: {#impl-MetricsExtractor-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#959-1177){.src
.rightside}[§](#impl-MetricsExtractor-for-Surface){.anchor}

### impl [MetricsExtractor](trait.MetricsExtractor.html "trait optionstratlib::geometrics::MetricsExtractor"){.trait} for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-metricsextractor-for-surface .code-header}
:::
:::::::
:::::::::::::::::::::::::::
::::::::::::::::::::::::::::
