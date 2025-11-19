:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Struct [Metrics]{.struct} Copy item path

[[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#63-88){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Metrics {
    pub basic: BasicMetrics,
    pub shape: ShapeMetrics,
    pub range: RangeMetrics,
    pub trend: TrendMetrics,
    pub risk: RiskMetrics,
}
```

Expand description

::: docblock
Represents a comprehensive set of statistical and analytical metrics for
curve data.

## [§](#overview){.doc-anchor}Overview

The `CurveMetrics` structure aggregates various metrics that describe
different aspects of a curve in a unified form. It provides an
encapsulated representation of curve information that spans different
categories, including basic statistical measures, shape characteristics,
range details, trend analysis, and risk evaluation.

### [§](#usage){.doc-anchor}Usage

This structure is particularly helpful in domains requiring holistic
curve analysis, such as:

- **Financial Analysis**: Used to analyze return curves and assess the
  risk-return trade-offs for financial products or strategies.
- **Data Science**: Provides comprehensive insights into a dataset's
  distribution, shape, and trends over time.
- **Scientific Research**: Useful for analyzing phenomena modeled by
  curves in domains like physics, biology, or economics.

### [§](#field-descriptions){.doc-anchor}Field Descriptions

- **basic**:
  [`BasicMetrics`](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics")
  - Contains mean, median, mode, and standard deviation of the dataset.
- **shape**:
  [`ShapeMetrics`](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics")
  - Captures the shape-related characteristics such as skewness,
    kurtosis, and extrema (peaks/valleys).
- **range**:
  [`RangeMetrics`](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics")
  - Includes range, min/max points, and quartiles for the dataset.
- **trend**:
  [`TrendMetrics`](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics")
  - Represents the directional tendencies, moving averages, and
    regression characteristics.
- **risk**:
  [`RiskMetrics`](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics")
  - Evaluates financial risk metrics such as volatility and Sharpe
    ratio.

### [§](#example-workflow){.doc-anchor}Example Workflow

The `CurveMetrics` structure is usually constructed by combining its
fields using the individual metric structures (`BasicMetrics`,
`ShapeMetrics`, `RangeMetrics`, `TrendMetrics`, `RiskMetrics`). It is
often initialized as part of a larger curve analysis operation and may
be transformed or queried for generating insights.

### [§](#related-concepts){.doc-anchor}Related Concepts

- [`BasicMetrics`](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"):
  Encodes fundamental statistics.
- [`ShapeMetrics`](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"):
  Provides characteristics associated with curve shape.
- [`RangeMetrics`](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"):
  Range and quartile information for a curve.
- [`TrendMetrics`](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"):
  Trendline and regression fit metrics.
- [`RiskMetrics`](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"):
  Quantifies financial risk.

### [§](#examples-of-associated-tools){.doc-anchor}Examples of Associated Tools

- Statistical Analysis: Plots, descriptive statistics, trend analysis.
- Visualizations: Understand curve behavior (e.g., peaks, valleys).
- Financial Metrics: Sharpe ratio, beta, and VaR for understanding
  portfolio risks.

### [§](#remarks){.doc-anchor}Remarks

The `CurveMetrics` struct is designed to be reusable across various
analytical contexts, providing a versatile and standardized way to
represent curve characteristics.
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.basic){.anchor
.field}`basic: `[`BasicMetrics`](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}]{#structfield.basic
.structfield .section-header}

::: docblock
- **Basic Metrics (`basic`)**: Includes fundamental statistical measures
  such as mean, median, mode, and standard deviation. These measures
  provide a quick overview of the distribution of the observations
  within the curve.
:::

[[§](#structfield.shape){.anchor
.field}`shape: `[`ShapeMetrics`](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}]{#structfield.shape
.structfield .section-header}

::: docblock
- **Shape Metrics (`shape`)**: Captures the structural characteristics
  of the curve, such as skewness, kurtosis, the locations of peaks and
  valleys, and the points where the curve inflects. Useful for
  understanding curve symmetry, tail behavior, and general shape
  nuances.
:::

[[§](#structfield.range){.anchor
.field}`range: `[`RangeMetrics`](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}]{#structfield.range
.structfield .section-header}

::: docblock
- **Range Metrics (`range`)**: Describes the range of the data,
  including minimum and maximum observed points, the extent between
  these points, and quartile-based statistical details such as
  interquartile range. Particularly helpful when analyzing variability
  in data distribution.
:::

[[§](#structfield.trend){.anchor
.field}`trend: `[`TrendMetrics`](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}]{#structfield.trend
.structfield .section-header}

::: docblock
- **Trend Metrics (`trend`)**: Measures the directional tendencies of
  the curve over time. Includes the slope, intercept, and statistical
  goodness-of-fit (R² value) as well as moving averages. Ideal for
  identifying long-term trends and evaluating the predictive nature of
  the curve.
:::

[[§](#structfield.risk){.anchor
.field}`risk: `[`RiskMetrics`](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"){.struct}]{#structfield.risk
.structfield .section-header}

::: docblock
- **Risk Metrics (`risk`)**: Quantifies curve risk using various
  financial metrics, such as volatility, value-at-risk (VaR), expected
  shortfall, beta, and the Sharpe ratio. These metrics are often used to
  evaluate the risk-return profile in financial contexts.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::: {#implementations-list}
:::: {#impl-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#120-168){.src
.rightside}[§](#impl-Metrics){.anchor}

### impl [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-metrics .code-header}

::: docblock
Represents a set of metrics associated with analyzing and interpreting a
curve.
:::
::::

::: docblock
This structure encapsulates multiple types of metrics, each responsible
for a specific aspect of curve analysis. These include basic statistical
measures, shape-related properties, range characteristics, trend
analysis, and risk factors.

##### [§](#fields-1){.doc-anchor}Fields

- `basic`: Basic statistical metrics such as mean, median, mode, and
  standard deviation.
- `shape`: Metrics that describe the shape of the curve, such as
  skewness, kurtosis, and points of interest (peaks, valleys, and
  inflection points).
- `range`: Range-based metrics specifying properties like the minimum
  and maximum values on the curve, quartiles, and interquartile range.
- `trend`: Metrics related to overall trends in the curve, such as
  slope, intercept, R-squared value, and moving average data points.
- `risk`: Risk-related metrics, including volatility, value at risk
  (VaR), expected shortfall, beta, and the Sharpe ratio.

##### [§](#notes){.doc-anchor}Notes

This implementation ensures modularity by separating distinct aspects of
curve analysis into specific metric structures. These metrics can be
used individually or collectively for advanced data analysis or curve
interpretation tasks.

##### [§](#related-structures){.doc-anchor}Related Structures

- [`BasicMetrics`](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"):
  Provides basic statistical metrics about the dataset.
- [`ShapeMetrics`](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"):
  Describes the geometric properties of the curve.
- [`RangeMetrics`](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"):
  Assesses the range and quartile characteristics of the curve.
- [`TrendMetrics`](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"):
  Analyzes trends within the data to understand directional behavior.
- [`RiskMetrics`](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"):
  Highlights risk-based metrics for financial, statistical, or
  analytical use cases.
- [`AnalysisResult`](struct.AnalysisResult.html "struct optionstratlib::geometrics::AnalysisResult"):
  The result type combining key metrics into a single analytic
  perspective.
- [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"):
  Represents potential errors that may arise during curve analysis
  operations.
:::

::::::: impl-items
::: {#method.new .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#135-149){.src
.rightside}

#### pub fn [new](#method.new){.fn}( basic: [BasicMetrics](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, shape: [ShapeMetrics](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, range: [RangeMetrics](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, trend: [TrendMetrics](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, risk: [RiskMetrics](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"){.struct}, ) -\> Self {#pub-fn-new-basic-basicmetrics-shape-shapemetrics-range-rangemetrics-trend-trendmetrics-risk-riskmetrics---self .code-header}
:::

::: docblock
###### [§](#new){.doc-anchor}`new`

Constructs a new instance of `CurveMetrics` and initializes all relevant
fields with the provided metric structures.

###### [§](#parameters){.doc-anchor}Parameters:

- `basic`: An instance of
  [`BasicMetrics`](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"),
  holding essential statistical information.
- `shape`: An instance of
  [`ShapeMetrics`](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"),
  measuring the geometric properties of the curve.
- `range`: An instance of
  [`RangeMetrics`](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"),
  describing the range and distribution of the curve.
- `trend`: An instance of
  [`TrendMetrics`](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"),
  detailing trend-based analytical results.
- `risk`: An instance of
  [`RiskMetrics`](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"),
  specifying the risk characteristics related to the curve.

###### [§](#returns){.doc-anchor}Returns:

- A new `CurveMetrics` instance containing the provided metrics.
:::

::: {#method.analysis_result .section .method}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#162-167){.src
.rightside}

#### pub fn [analysis_result](#method.analysis_result){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[AnalysisResult](struct.AnalysisResult.html "struct optionstratlib::geometrics::AnalysisResult"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-analysis_resultself---resultanalysisresult-curveerror .code-header}
:::

::: docblock
###### [§](#curve_analysis_result){.doc-anchor}`curve_analysis_result`

Generates a high-level analysis result from the metrics encapsulated
within the `CurveMetrics` instance.

###### [§](#returns-1){.doc-anchor}Returns:

- `Ok(CurveAnalysisResult)`: A result that contains analyzed data in the
  form of a
  [`AnalysisResult`](struct.AnalysisResult.html "struct optionstratlib::geometrics::AnalysisResult")
  structure with basic statistics and shape metrics.
- `Err(CurvesError)`: An error of type
  [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError")
  when analysis fails.

The result provides the basic statistical measures (`BasicMetrics`) and
shape metrics (`ShapeMetrics`) that were part of the `CurveMetrics`
instance.
:::
:::::::
:::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#impl-Clone-for-Metrics){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-clone-for-metrics .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#fn-cloneself---metrics .code-header}
:::

::: docblock
Returns a duplicate of the value. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#245-247){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-ComposeSchema-for-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#impl-ComposeSchema-for-Metrics){.anchor}

### impl ComposeSchema for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-composeschema-for-metrics .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#impl-Debug-for-Metrics){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-debug-for-metrics .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Display-for-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#impl-Display-for-Metrics){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-display-for-metrics .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Serialize-for-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#impl-Serialize-for-Metrics){.anchor}

### impl [Serialize](../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-serialize-for-metrics .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](../../serde_core/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](../../serde_core/ser/trait.Serializer.html#associatedtype.Ok "type serde_core::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](../../serde_core/ser/trait.Serializer.html#associatedtype.Error "type serde_core::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](../../serde_core/ser/trait.Serializer.html "trait serde_core::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](../../serde_core/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-ToSchema-for-Metrics .section .impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#impl-ToSchema-for-Metrics){.anchor}

### impl [ToSchema](../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-toschema-for-metrics .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/metrics.rs.html#62){.src
.rightside}[§](#method.schemas){.anchor}

#### fn [schemas](../../utoipa/trait.ToSchema.html#method.schemas){.fn}(schemas: &mut [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>)\>) {#fn-schemasschemas-mut-vecstring-reforschema .code-header}
:::

::: docblock
Implement reference
[`utoipa::openapi::schema::Schema`](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema")s
for this type. [Read
more](../../utoipa/trait.ToSchema.html#method.schemas)
:::
:::::::
:::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Metrics .section .impl}
[§](#impl-Freeze-for-Metrics){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-freeze-for-metrics .code-header}
:::

::: {#impl-RefUnwindSafe-for-Metrics .section .impl}
[§](#impl-RefUnwindSafe-for-Metrics){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-refunwindsafe-for-metrics .code-header}
:::

::: {#impl-Send-for-Metrics .section .impl}
[§](#impl-Send-for-Metrics){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-send-for-metrics .code-header}
:::

::: {#impl-Sync-for-Metrics .section .impl}
[§](#impl-Sync-for-Metrics){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-sync-for-metrics .code-header}
:::

::: {#impl-Unpin-for-Metrics .section .impl}
[§](#impl-Unpin-for-Metrics){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-unpin-for-metrics .code-header}
:::

::: {#impl-UnwindSafe-for-Metrics .section .impl}
[§](#impl-UnwindSafe-for-Metrics){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct} {#impl-unwindsafe-for-metrics .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.91.1/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#212){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#214){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#221){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#222){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#515){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#517){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dest: [\*mut](https://doc.rust-lang.org/1.91.1/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.91.1/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dest-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dest`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#785){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[Source](../../src/tracing/instrument.rs.html#325){.src
.rightside}[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> [Instrument](../../tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument"){.trait} for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#86){.src
.rightside}[§](#method.instrument){.anchor}

#### fn [instrument](../../tracing/instrument/trait.Instrument.html#method.instrument){.fn}(self, span: [Span](../../tracing/span/struct.Span.html "struct tracing::span::Span"){.struct}) -\> [Instrumented](../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided
[`Span`](../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../tracing/instrument/trait.Instrument.html#method.instrument)
:::

::: {#method.in_current_span .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#128){.src
.rightside}[§](#method.in_current_span){.anchor}

#### fn [in_current_span](../../tracing/instrument/trait.Instrument.html#method.in_current_span){.fn}(self) -\> [Instrumented](../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the
[current](../../tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current")
[`Span`](../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../tracing/instrument/trait.Instrument.html#method.in_current_span)
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#767-769){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#777){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](../../src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](../../either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](../../src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](../../either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> [Either](../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](../../src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](../../either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

:::: {#impl-PartialSchema-for-T .section .impl}
[Source](../../src/utoipa/lib.rs.html#1375){.src
.rightside}[§](#impl-PartialSchema-for-T){.anchor}

### impl\<T\> [PartialSchema](../../utoipa/trait.PartialSchema.html "trait utoipa::PartialSchema"){.trait} for T {#implt-partialschema-for-t .code-header}

::: where
where T: ComposeSchema +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.schema .section .method .trait-impl}
[Source](../../src/utoipa/lib.rs.html#1376){.src
.rightside}[§](#method.schema){.anchor}

#### fn [schema](../../utoipa/trait.PartialSchema.html#tymethod.schema){.fn}() -\> [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-schema---reforschema .code-header}
:::

::: docblock
Return ref or schema of implementing type that can then be used to
construct combined schemas.
:::
:::::

::: {#impl-Pointable-for-T .section .impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#194){.src
.rightside}[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> [Pointable](../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait} for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#195){.src
.rightside}[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN){.constant}: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#197){.src
.rightside}[§](#associatedtype.Init){.anchor}

#### type [Init](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init){.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#199){.src
.rightside}[§](#method.init){.anchor}

#### unsafe fn [init](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init){.fn}(init: \<T as [Pointable](../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait}\>::[Init](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init"){.associatedtype}) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)
:::

::: {#method.deref .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#203){.src
.rightside}[§](#method.deref){.anchor}

#### unsafe fn [deref](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)
:::

::: {#method.deref_mut .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#207){.src
.rightside}[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)
:::

::: {#method.drop .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#211){.src
.rightside}[§](#method.drop){.anchor}

#### unsafe fn [drop](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop){.fn}(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](../../src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](../../typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](../../typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[Source](../../src/simba/scalar/subset.rs.html#90){.src
.rightside}[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> [SupersetOf](../../simba/scalar/subset/trait.SupersetOf.html "trait simba::scalar::subset::SupersetOf"){.trait}\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS:
[SubsetOf](../../simba/scalar/subset/trait.SubsetOf.html "trait simba::scalar::subset::SubsetOf"){.trait}\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#92){.src
.rightside}[§](#method.to_subset){.anchor}

#### fn [to_subset](../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. [Read
more](../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset)
:::

::: {#method.is_in_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#97){.src
.rightside}[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.is_in_subset){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#102){.src
.rightside}[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.to_subset_unchecked){.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#107){.src
.rightside}[§](#method.from_subset){.anchor}

#### fn [from_subset](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.from_subset){.fn}(element: [&SS](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#85-87){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#89){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#90){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#94){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2796){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2798){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#827-829){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#831){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.91.1/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#834){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#811-813){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#815){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#818){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[Source](../../src/ppv_lite86/types.rs.html#221-223){.src
.rightside}[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> [VZip](../../ppv_lite86/types/trait.VZip.html "trait ppv_lite86::types::VZip"){.trait}\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V:
[MultiLane](../../ppv_lite86/types/trait.MultiLane.html "trait ppv_lite86::types::MultiLane"){.trait}\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[Source](../../src/ppv_lite86/types.rs.html#226){.src
.rightside}[§](#method.vzip){.anchor}

#### fn [vzip](../../ppv_lite86/types/trait.VZip.html#tymethod.vzip){.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[Source](../../src/tracing/instrument.rs.html#393){.src
.rightside}[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> [WithSubscriber](../../tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber"){.trait} for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#176-178){.src
.rightside}[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber](../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber){.fn}\<S\>(self, subscriber: S) -\> [WithDispatch](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Dispatch](../../tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch"){.struct}\>,
:::
::::

::: docblock
Attaches the provided
[`Subscriber`](../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#228){.src
.rightside}[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber](../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber){.fn}(self) -\> [WithDispatch](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](../../tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher")
[`Subscriber`](../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)
:::
::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
