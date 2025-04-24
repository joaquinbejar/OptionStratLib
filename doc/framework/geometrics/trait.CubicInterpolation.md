::::::::::::::::: width-limiter
:::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [CubicInterpolation]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/interpolation/cubic.rs.html#96-108){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait CubicInterpolation<Point, Input> {
    // Required method
    fn cubic_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
```

Expand description

::::: docblock
A trait for performing cubic interpolation on a set of data points.

## [§](#overview){.doc-anchor}Overview

The `CubicInterpolation` trait provides a framework for implementing
cubic interpolation algorithms. This technique is used to calculate
smooth curves through a set of discrete data points, commonly via cubic
polynomials. The primary functionality is exposed via the
`cubic_interpolate` method.

Cubic interpolation offers smooth transitions between points and is
widely applied in fields such as:

- **Graphical applications**: Creating smooth animations or generating
  curves.
- **Signal processing**: Smoothing data or reconstructing missing data
  points.
- **Physics and simulations**: Modeling smooth transformations,
  trajectories, or motions.

When implementing this trait, it is recommended to carefully manage edge
cases, such as insufficient data points, invalid input values, or
floating-point precision issues.

## [§](#associated-types){.doc-anchor}Associated Types

- `Point`: Represents the type of output point (e.g., 2D or
  n-dimensional points).
- `Input`: The type of the input coordinate, typically a numeric type
  (e.g., `Decimal`).
- `Error`: The type used for error handling when interpolation fails.

## [§](#required-method){.doc-anchor}Required Method

### [§](#cubic_interpolate){.doc-anchor}`cubic_interpolate`

Performs cubic interpolation for a provided input value, calculating the
corresponding interpolated point.

#### [§](#parameters){.doc-anchor}Parameters

- `x`: The input value (of type `Input`) for which the interpolated
  point on the curve should be calculated.

#### [§](#returns){.doc-anchor}Returns

- `Ok(Point)`: The interpolated point successfully calculated for the
  given input.
- `Err(Error)`: An error that provides details about why the
  interpolation operation failed.

## [§](#notes-on-precision){.doc-anchor}Notes on Precision

The precision of the interpolation is directly influenced by the type of
the input (`Input`) and the internal calculations used. If high
numerical precision is required, using a type like `Decimal` (from the
`rust_decimal` crate) is highly recommended to avoid floating-point
inaccuracies.

## [§](#error-handling){.doc-anchor}Error Handling

Implementations of this trait should return meaningful errors of the
associated `Error` type when interpolation cannot be completed. Examples
include:

- **Insufficient data points**: If computation requires more data than
  is available.
- **Out-of-bounds input values**: When the input value `x` falls outside
  the defined interpolation range.
- **Invalid data**: Issues encountered in the provided data or curve
  structure.

## [§](#example-usage){.doc-anchor}Example Usage

Below is a general outline for implementing this trait:

::: example-wrap
``` language-text
1. Validate the data points necessary for interpolation.
2. Compute the required coefficients for a cubic polynomial.
3. Use the coefficients to evaluate the polynomial at the provided input `x`
   to obtain the interpolated point.
4. Handle edge cases and return appropriate errors if the operation fails.
```
:::

## [§](#see-also){.doc-anchor}See Also

- `Point`: Generic representation of a point in 2D or multi-dimensional
  space.
- `CurvesError`: A possible error type for representing issues during
  interpolation.
- `rust_decimal`: A crate for performing high-precision numerical
  calculations.

## [§](#usage-example){.doc-anchor}Usage Example

An implementation of cubic interpolation could apply this trait to a
data set as follows:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;///

use optionstratlib::curves::Point2D;
use optionstratlib::error::InterpolationError;


use optionstratlib::geometrics::CubicInterpolation;

struct MyCurve {
    data_points: Vec<Point2D>,
}

impl CubicInterpolation<Point2D, Decimal> for MyCurve {

    fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        // Validate the input and calculate the interpolated point.
        // Example logic here.
        Ok(Point2D::new(x, x)) // Placeholder implementation.
    }
}
```
:::
:::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.cubic_interpolate .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/cubic.rs.html#107){.src
.rightside}

#### fn [cubic_interpolate](#tymethod.cubic_interpolate){.fn}(&self, x: Input) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-cubic_interpolateself-x-input---resultpoint-interpolationerror .code-header}
:::

::: docblock
Interpolates a new point on the curve for a given `x` input value using
cubic interpolation.

##### [§](#parameters-1){.doc-anchor}Parameters

- `x`: The input value along the curve for which an interpolated point
  is calculated.

##### [§](#returns-1){.doc-anchor}Returns

- `Ok(Point)`: Represents the interpolated point on the curve.
- `Err(Self::Error)`: Describes why the interpolation failed (e.g.,
  invalid input).
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::: {#implementors-list}
:::: {#impl-CubicInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#583-699){.src
.rightside}[§](#impl-CubicInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [CubicInterpolation](trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-cubicinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `CubicInterpolation` trait for the `Curve` struct,
providing an algorithm for cubic interpolation utilizing a Catmull-Rom
spline.
:::
::::

::: docblock
#### [§](#method-cubic_interpolate){.doc-anchor}Method: `cubic_interpolate`

##### [§](#parameters-2){.doc-anchor}Parameters

- **`x`**: The x-value at which the interpolation is performed. This
  value must be within the range of x-values in the curve's defined
  points, and it is passed as a `Decimal` to allow for high-precision
  computation.

##### [§](#returns-2){.doc-anchor}Returns

- **`Ok(Point2D)`**: Returns a `Point2D` representing the interpolated x
  and y values.
- **`Err(CurvesError)`**: Returns an error if:
  - There are fewer than 4 points available for interpolation.
  - The x-value is outside the curve's range, or interpolation fails for
    any other reason.

##### [§](#behavior){.doc-anchor}Behavior

1.  **Point Validation**: Ensures at least four points exist for cubic
    interpolation, as this is a fundamental requirement for computing
    the Catmull-Rom spline.
2.  **Exact Match Check**: If the x-value matches an existing point in
    the curve, the method directly returns the corresponding `Point2D`
    without further computation.
3.  **Bracket Points**: Determines the bracketing points (4 points
    total) around the provided x-value. Depending on the position of the
    x-value in the curve, the method dynamically adjusts the selected
    points to ensure they form a proper bracket:
    - If near the start of the curve, uses the first four points.
    - If near the end, uses the last four points.
    - Else, selects points before and after x to define the bracket.
4.  **Parameter Calculation**: Computes a normalized parameter `t` that
    represents the relative position of the target x-value between `p1`
    and `p2`.
5.  **Catmull-Rom Spline**: Performs cubic interpolation using a
    Catmull-Rom spline, a widely used, smooth spline algorithm. The
    coefficients are calculated based on the relative x position and the
    y-values of the four surrounding points.
6.  **Interpolation**: Calculates the interpolated y-value using the
    cubic formula:

    ::: example-wrap
    ``` language-text
    y(t) = 0.5 * (
        2 * p1.y + (-p0.y + p2.y) * t
        + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t^2
        + (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * t^3
    )
    ```
    :::

    Here, `t` is the normalized x position, and `p0`, `p1`, `p2`, `p3`
    are the four bracketed points.

##### [§](#errors){.doc-anchor}Errors

- Returns an error of type `CurvesError::InterpolationError` if any
  issues are encountered, such as insufficient points or the inability
  to locate bracket points.

##### [§](#example){.doc-anchor}Example

This method is part of the `Curve` struct, which defines a set of points
and supports interpolation. It is often used in applications requiring
smooth manifolds or animations.

##### [§](#notes){.doc-anchor}Notes

- The computed y-value ensures smooth transitions and continuity between
  interpolated segments.
- Catmull-Rom splines are particularly effective for creating visually
  smooth transitions, making this method suitable for curves,
  animations, and numerical analysis.

#### [§](#see-also-1){.doc-anchor}See Also

- [`CubicInterpolation`](trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"):
  The trait defining this method.
- [`Point2D`](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents the points used for interpolation.
- [`find_bracket_points`](trait.Interpolate.html#method.find_bracket_points "method optionstratlib::geometrics::Interpolate::find_bracket_points"):
  Determines the bracketing points required for interpolation.
:::

::: {#impl-CubicInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#771-842){.src
.rightside}[§](#impl-CubicInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [CubicInterpolation](trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-cubicinterpolationpoint3d-point2d-for-surface .code-header}
:::
:::::::
::::::::::::::::
:::::::::::::::::
