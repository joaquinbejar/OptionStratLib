::::::::::::::::::: width-limiter
:::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [LinearInterpolation]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/interpolation/linear.rs.html#76-86){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait LinearInterpolation<Point, Input> {
    // Required method
    fn linear_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
```

Expand description

::: docblock
A trait that provides functionality for performing linear interpolation.

## [§](#overview){.doc-anchor}Overview

The `LinearInterpolation` trait defines the behavior for linearly
interpolating values along a curve or dataset. It allows implementors to
compute the interpolated point for a given input using linear methods.

Linear interpolation is a simple and widely-used technique in which new
data points can be estimated within the range of a discrete set of known
data points by connecting adjacent points with straight lines.

## [§](#associated-types){.doc-anchor}Associated Types

- **`Point`**: Represents the type of the interpolated data point (e.g.,
  coordinates, numerical values, or other types).
- **`Input`**: Represents the type of the input value used to calculate
  the interpolation (e.g., a single number such as an `f64` or a
  high-precision number like `Decimal`).
- **`Error`**: Defines the error type returned if interpolation cannot
  be performed due to invalid input or other constraints (e.g.,
  insufficient points for interpolation).

## [§](#required-method){.doc-anchor}Required Method

### [§](#linear_interpolate){.doc-anchor}`linear_interpolate`

This method calculates the interpolated value for a given input.

- **Input:**\
  Requires a value of type `Input` (e.g., a numerical value or
  coordinate) for which the corresponding interpolated `Point` should be
  calculated.

- **Returns:**

  - On success, returns a `Result` containing the interpolated value of
    type `Point`.
  - On failure, returns a `Result` containing an error of type
    `Self::Error` that describes the reason for failure (e.g., invalid
    input or insufficient data points).

## [§](#typical-usage){.doc-anchor}Typical Usage

This trait is abstract and should be implemented for data structures
representing curves or data sets where linear interpolation is required.
Examples include interpolating between points on a 2D curve, estimating
values in a time series, or refining graphical data.

## [§](#example-use-case-general){.doc-anchor}Example Use Case (General)

This trait can be used to:

- Estimate missing values in a dataset.
- Provide smooth transitions between adjacent points on a graph.
- Implement real-time interpolations for dynamic systems, such as
  animations or simulations.

## [§](#notes-on-implementation){.doc-anchor}Notes on Implementation

- Implementors of this trait must ensure that the interpolation logic
  respects the shape and constraints of the data being used. This
  involves:

  - Validating input values.
  - Handling boundary conditions (e.g., extrapolation or edge points).

- High precision or custom input/output types (e.g., `Decimal`) can be
  used when necessary to avoid issues related to floating-point errors
  in sensitive calculations.

## [§](#example-implementation-outline){.doc-anchor}Example Implementation Outline

1.  Access two known points adjacent to the input value.
2.  Compute the slope and intercept of the line between the two points.
3.  Evaluate the equation of the line at the given input to determine
    the interpolated point.

## [§](#errors){.doc-anchor}Errors

When implementing this trait, common errors that may be returned
include:

- Insufficient data points for interpolation.
- Out-of-bounds input values or indices.
- Invalid data point structures or internal state errors.

## [§](#see-also){.doc-anchor}See Also

- [`crate::geometrics::CubicInterpolation`](trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"):
  An alternative interpolation method using cubic polynomials.
- [`crate::geometrics::InterpolationType`](enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"):
  Enum representing supported interpolation methods in the library.

The `LinearInterpolation` trait is part of a modular design and is often
re-exported in higher-level library components for ease of use.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.linear_interpolate .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/linear.rs.html#85){.src
.rightside}

#### fn [linear_interpolate](#tymethod.linear_interpolate){.fn}(&self, x: Input) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-linear_interpolateself-x-input---resultpoint-interpolationerror .code-header}
:::

::: docblock
Performs linear interpolation for a given input value.

##### [§](#parameters){.doc-anchor}Parameters

- `x`: The input value of type `Input` for which the interpolated point
  should be calculated.

##### [§](#returns){.doc-anchor}Returns

- `Ok(Point)`: The calculated interpolated value of type `Point`.
- `Err(Self::Error)`: An error indicating the reason why interpolation
  failed.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::::::: {#implementors-list}
:::: {#impl-LinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#352-370){.src
.rightside}[§](#impl-LinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [LinearInterpolation](trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-linearinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `LinearInterpolation` trait for the `Curve` struct.
:::
::::

::::: docblock
This implementation provides linear interpolation functionality for a
given set of points on a curve. The interpolation computes the `y` value
corresponding to a given `x` value using the linear interpolation
formula. The method ensures that the input `x` is within the range of
the curve's defined points.

::: example-wrap
``` language-text
y = p1.y + (x - p1.x) * (p2.y - p1.y) / (p2.x - p1.x)
```
:::

#### [§](#parameters-1){.doc-anchor}Parameters

- `x`: A `Decimal` representing the `x`-coordinate for which the
  corresponding interpolated `y` value is to be computed.

#### [§](#returns-1){.doc-anchor}Returns

- `Ok(Point2D)`: A `Point2D` instance containing the input `x` value and
  the interpolated `y` value.
- `Err(CurvesError)`: Returns an error of type
  `CurvesError::InterpolationError` in any of the following cases:
  - The curve does not have enough points for interpolation.
  - The provided `x` value is outside the range of the curve's points.
  - Bracketing points for `x` cannot be found.

#### [§](#working-mechanism){.doc-anchor}Working Mechanism

1.  The method calls `find_bracket_points` (implemented in the
    `Interpolate` trait) to locate the index pair `(i, j)` of two points
    that bracket the `x` value.
2.  From the located points `p1` and `p2`, the method calculates the
    interpolated `y` value using the linear interpolation formula.
3.  Finally, a `Point2D` is created and returned with the provided `x`
    and the computed `y` value.

#### [§](#implementation-details){.doc-anchor}Implementation Details

- The function leverages `Decimal` arithmetic for high precision in
  calculations.
- It assumes that the provided points on the curve are sorted in
  ascending order based on their `x` values.

#### [§](#errors-1){.doc-anchor}Errors

This method returns a `CurvesError` in the following cases:

- **Insufficient Points**: When the curve has fewer than two points.
- **Out-of-Range `x`**: When the input `x` value lies outside the range
  of the defined points.
- **No Bracketing Points Found**: When the method fails to find two
  points that bracket the given `x`.

#### [§](#example-how-it-works-internally){.doc-anchor}Example (How it works internally)

Suppose the curve is defined by the following points:

- `p1 = (2.0, 4.0)`
- `p2 = (5.0, 10.0)`

Given `x = 3.0`, the method computes:

::: example-wrap
``` language-text
y = 4.0 + (3.0 - 2.0) * (10.0 - 4.0) / (5.0 - 2.0)
  = 4 + 1 * 6 / 3
  = 4 + 2
  = 6.0
```
:::

It will return `Point2D { x: 3.0, y: 6.0 }`.

#### [§](#see-also-1){.doc-anchor}See Also

- `find_bracket_points`: Finds two points that bracket a value.
- `Point2D`: Represents points in 2D space.
- `CurvesError`: Represents errors related to curve operations.
:::::

:::: {#impl-LinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#608-688){.src
.rightside}[§](#impl-LinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [LinearInterpolation](trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-linearinterpolationpoint3d-point2d-for-surface .code-header}

::: docblock
#### [§](#linear-interpolation-for-surfaces){.doc-anchor}Linear Interpolation for Surfaces

Implementation of the `LinearInterpolation` trait for `Surface`
structures, enabling interpolation from 2D points to 3D points using
barycentric coordinates.
:::
::::

::: docblock
##### [§](#overview-1){.doc-anchor}Overview

This implementation allows calculating the height (z-coordinate) of any
point within the surface's x-y range by using linear interpolation based
on the three nearest points in the surface. The method employs
barycentric coordinate interpolation with triangulation of the nearest
points.

##### [§](#algorithm){.doc-anchor}Algorithm

The interpolation process follows these steps:

1.  Validate that the input point is within the surface's range
2.  Check for degenerate cases (all points at same location)
3.  Check for exact matches with existing points
4.  Find the three nearest points to the query point
5.  Calculate barycentric coordinates for the triangle formed by these
    points
6.  Interpolate the z-value using the barycentric weights
:::
:::::::::::
::::::::::::::::::
:::::::::::::::::::
