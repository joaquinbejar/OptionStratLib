::::::::::::::::::: width-limiter
:::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[curves](index.html)
:::

# Trait [StatisticalCurve]{.trait} Copy item path

[[Source](../../src/optionstratlib/curves/traits.rs.html#81-314){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait StatisticalCurve: MetricsExtractor {
    // Required method
    fn get_x_values(&self) -> Vec<Decimal>;

    // Provided methods
    fn generate_statistical_curve(
        &self,
        basic_metrics: &BasicMetrics,
        shape_metrics: &ShapeMetrics,
        range_metrics: &RangeMetrics,
        trend_metrics: &TrendMetrics,
        num_points: usize,
        seed: Option<u64>,
    ) -> Result<Curve, CurveError> { ... }
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
    ) -> Result<Curve, CurveError> { ... }
    fn verify_curve_metrics(
        &self,
        curve: &Curve,
        target_metrics: &BasicMetrics,
        tolerance: Decimal,
    ) -> Result<bool, CurveError> { ... }
}
```

Expand description

::: docblock
A trait for generating statistical curves based on metrics

This trait provides methods to generate curves that match specified
statistical properties. It extends the `MetricsExtractor` trait to
ensure implementing types can both extract and generate metrics.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.get_x_values .section .method}
[Source](../../src/optionstratlib/curves/traits.rs.html#92){.src
.rightside}

#### fn [get_x_values](#tymethod.get_x_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_x_valuesself---vecdecimal .code-header}
:::

::: docblock
Retrieves the x-axis values for the statistical curve.

This method returns a vector of `Decimal` values representing the
x-coordinates of the points that define the curve. These x-values are
essential for plotting the curve and performing various statistical
analyses.

##### [§](#returns){.doc-anchor}Returns

A `Vec<Decimal>` containing the x-values of the statistical curve. Each
`Decimal` represents a point on the x-axis.
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::: methods
::: {#method.generate_statistical_curve .section .method}
[Source](../../src/optionstratlib/curves/traits.rs.html#113-223){.src
.rightside}

#### fn [generate_statistical_curve](#method.generate_statistical_curve){.fn}( &self, basic_metrics: &[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, shape_metrics: &[ShapeMetrics](../geometrics/struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, range_metrics: &[RangeMetrics](../geometrics/struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, trend_metrics: &[TrendMetrics](../geometrics/struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, num_points: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, seed: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-generate_statistical_curve-self-basic_metrics-basicmetrics-shape_metrics-shapemetrics-range_metrics-rangemetrics-trend_metrics-trendmetrics-num_points-usize-seed-optionu64---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a statistical curve with properties that match the provided
metrics.

##### [§](#overview){.doc-anchor}Overview

This function creates a curve with statistical properties that
approximate the specified metrics. It uses a combination of normal
distribution sampling and transformations to achieve the desired
statistical characteristics.

##### [§](#parameters){.doc-anchor}Parameters

- `basic_metrics`: Basic statistical properties like mean, median, mode,
  and standard deviation.
- `shape_metrics`: Shape-related metrics like skewness and kurtosis.
- `range_metrics`: Range information including min, max, and quartile
  data.
- `trend_metrics`: Trend information including slope and intercept for
  linear trend.
- `num_points`: Number of points to generate in the curve.
- `seed`: Optional random seed for reproducible curve generation.

##### [§](#returns-1){.doc-anchor}Returns

- `Result<Curve, CurveError>`: A curve matching the specified
  statistical properties, or an error if generation fails.
:::

::: {#method.generate_refined_statistical_curve .section .method}
[Source](../../src/optionstratlib/curves/traits.rs.html#245-286){.src
.rightside}

#### fn [generate_refined_statistical_curve](#method.generate_refined_statistical_curve){.fn}( &self, basic_metrics: &[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, shape_metrics: &[ShapeMetrics](../geometrics/struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, range_metrics: &[RangeMetrics](../geometrics/struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, trend_metrics: &[TrendMetrics](../geometrics/struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, num_points: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, max_attempts: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, tolerance: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, seed: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-generate_refined_statistical_curve-self-basic_metrics-basicmetrics-shape_metrics-shapemetrics-range_metrics-rangemetrics-trend_metrics-trendmetrics-num_points-usize-max_attempts-usize-tolerance-decimal-seed-optionu64---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a refined statistical curve that iteratively adjusts to better
match the target metrics.

This method extends the basic curve generation by performing multiple
attempts with adjusted parameters until the resulting curve metrics are
within the specified tolerance of the target metrics.

##### [§](#parameters-1){.doc-anchor}Parameters

- `basic_metrics`: Target basic statistical metrics
- `shape_metrics`: Target shape metrics
- `range_metrics`: Target range metrics
- `trend_metrics`: Target trend metrics
- `num_points`: Number of points to generate
- `max_attempts`: Maximum number of generation attempts (default: 5)
- `tolerance`: Acceptable difference between target and actual metrics
  (default: 0.1)
- `seed`: Optional random seed for reproducibility

##### [§](#returns-2){.doc-anchor}Returns

- `Result<Curve, CurveError>`: The generated curve or an error
:::

::: {#method.verify_curve_metrics .section .method}
[Source](../../src/optionstratlib/curves/traits.rs.html#298-313){.src
.rightside}

#### fn [verify_curve_metrics](#method.verify_curve_metrics){.fn}( &self, curve: &[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, target_metrics: &[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, tolerance: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-verify_curve_metrics-self-curve-curve-target_metrics-basicmetrics-tolerance-decimal---resultbool-curveerror .code-header}
:::

::: docblock
Verifies if the metrics of the generated curve match the target metrics
within the specified tolerance.

##### [§](#parameters-2){.doc-anchor}Parameters

- `curve`: The curve to verify
- `target_metrics`: The target basic metrics to compare against
- `tolerance`: Maximum acceptable difference between actual and target
  metrics

##### [§](#returns-3){.doc-anchor}Returns

- `Result<bool, CurveError>`: True if metrics match within tolerance,
  false otherwise
:::
:::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-StatisticalCurve-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#955-959){.src
.rightside}[§](#impl-StatisticalCurve-for-Curve){.anchor}

### impl [StatisticalCurve](trait.StatisticalCurve.html "trait optionstratlib::curves::StatisticalCurve"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-statisticalcurve-for-curve .code-header}
:::
::::
::::::::::::::::::
:::::::::::::::::::
