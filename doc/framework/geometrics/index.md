::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module geometricsCopy item path

[[Source](../../src/optionstratlib/geometrics/mod.rs.html#6-31){.src}
]{.sub-heading}
::::

Expand description

::: docblock
- `geometrics` - Mathematical utilities for geometric calculations
  relevant to options.

Provides specialized geometric functions and algorithms for options
pricing and modeling, including path-dependent calculations and spatial
transformations for volatility surfaces.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[AnalysisResult](struct.AnalysisResult.html "struct optionstratlib::geometrics::AnalysisResult"){.struct}
:   Contains comprehensive analysis results for a curve or dataset.

[BasicMetrics](struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}
:   Represents a collection of fundamental statistical metrics.

[Metrics](struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct}
:   Represents a comprehensive set of statistical and analytical metrics
    for curve data.

[PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}
:   A builder for creating and configuring data visualizations.

[PlotOptions](struct.PlotOptions.html "struct optionstratlib::geometrics::PlotOptions"){.struct}
:   Plot configuration options for data visualization.

[RangeMetrics](struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}
:   Represents statistical and range-related metrics for a dataset.

[RiskMetrics](struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"){.struct}
:   Represents a collection of key financial risk metrics used in risk
    analysis and performance evaluation.

[ShapeMetrics](struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}
:   Represents shape-related analysis metrics for a given curve.

[TrendMetrics](struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}
:   Represents key metrics for analyzing trends within a dataset or
    curve.

## Enums[§](#enums){.anchor} {#enums .section-header}

[ConstructionMethod](enum.ConstructionMethod.html "enum optionstratlib::geometrics::ConstructionMethod"){.enum}
:   Defines methods for constructing geometric objects.

[ConstructionParams](enum.ConstructionParams.html "enum optionstratlib::geometrics::ConstructionParams"){.enum}
:   Parameters for constructing geometric objects in different
    dimensions.

[InterpolationType](enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}
:   Represents the different types of interpolation methods supported in
    the library.

[MergeOperation](enum.MergeOperation.html "enum optionstratlib::geometrics::MergeOperation"){.enum}
:   Represents mathematical or aggregation operations that can be
    applied during a merge or combination process of geometric objects
    or curves.

## Traits[§](#traits){.anchor} {#traits .section-header}

[Arithmetic](trait.Arithmetic.html "trait optionstratlib::geometrics::Arithmetic"){.trait}
:   A trait that provides arithmetic operations for working with
    geometries, enabling merging and combining of curve data based on
    different operations.

[AxisOperations](trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}
:   Trait for handling axis-based operations on geometric structures.

[BiLinearInterpolation](trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}
:   A trait for bilinear interpolation on 2D data grids.

[CubicInterpolation](trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}
:   A trait for performing cubic interpolation on a set of data points.

[GeometricObject](trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}
:   Defines a geometric object constructed from a set of points.
    Provides methods for creating, accessing, and manipulating these
    objects.

[GeometricTransformations](trait.GeometricTransformations.html "trait optionstratlib::geometrics::GeometricTransformations"){.trait}
:   Geometric Transformations

[HasX](trait.HasX.html "trait optionstratlib::geometrics::HasX"){.trait}
:   A trait for types that provide access to an X-coordinate value.

[Interpolate](trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}
:   A trait for performing various types of interpolation on a set of 2D
    points.

[LinearInterpolation](trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}
:   A trait that provides functionality for performing linear
    interpolation.

[MergeAxisInterpolate](trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}
:   Trait for merging and interpolating axes between compatible
    geometric structures.

[MetricsExtractor](trait.MetricsExtractor.html "trait optionstratlib::geometrics::MetricsExtractor"){.trait}
:   A trait for extracting comprehensive metrics from a curve.

[PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}
:   Extension methods for the plot building process.

[Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait}
:   Trait for defining objects that can be visualized as plots.

[SplineInterpolation](trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}
:   A trait defining the functionality for spline-based interpolation
    over a dataset.

## Type Aliases[§](#types){.anchor} {#types .section-header}

[ResultPoint](type.ResultPoint.html "type optionstratlib::geometrics::ResultPoint"){.type}
:   A result type for geometric point operations that may fail.
::::::
:::::::
