::::::::::::::::: width-limiter
:::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [SplineInterpolation]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/interpolation/spline.rs.html#88-117){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait SplineInterpolation<Point, Input> {
    // Required method
    fn spline_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
```

Expand description

:::: docblock
A trait defining the functionality for spline-based interpolation over a
dataset.

## [§](#overview){.doc-anchor}Overview

The `SplineInterpolation` trait provides a framework for approximating
unknown values between known data points using spline interpolation
techniques. A spline is a piecewise polynomial function that ensures
smooth and continuous transitions across its entire range. This method
is commonly utilized in numerical analysis, computer graphics, and
scientific computations.

### [§](#use-cases){.doc-anchor}Use Cases

- Filling in missing values in datasets.
- Generating smooth curves to approximate trends in data.
- Scenarios requiring continuity and smoothness across multiple points.

### [§](#key-features){.doc-anchor}Key Features:

- Allows implementing custom spline interpolators.
- Provides error handling for boundary cases or dataset inconsistencies.
- Can be extended for different types of interpolation datasets.

## [§](#associated-types){.doc-anchor}Associated Types

- `Point`: Represents the output type, typically a point in 2D space
  (e.g., `Point2D`).
- `Input`: The type of the input x-coordinate for which interpolation is
  performed.
- `Error`: Represents possible errors encountered during interpolation
  (e.g., inadequate data, boundary conditions, or internal computation
  errors).

## [§](#required-method){.doc-anchor}Required Method

#### [§](#spline_interpolate){.doc-anchor}`spline_interpolate`

- **Purpose**: Computes the `y` value corresponding to a supplied `x`
  value using spline interpolation techniques.
- **Parameters**:
  - `x`: The x-coordinate (of type `Input`) for which interpolation is
    required.
- **Returns**:
  - `Ok(Point)`: Represents the interpolated output, typically
    containing both the x and calculated y coordinates.
  - `Err(Self::Error)`: Represents the failure scenario during the
    interpolation.

## [§](#error-handling){.doc-anchor}Error Handling

This trait defines an associated `Error` type to handle failures during
interpolation. Expected error cases include:

- Insufficient or invalid data points.
- Extrapolation requests (depending on implementation).
- Incorrect spline configurations or singularities in computation.

## [§](#example-usage){.doc-anchor}Example Usage

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::geometrics::SplineInterpolation;
use optionstratlib::error::InterpolationError;
use optionstratlib::curves::Point2D;
use rust_decimal::Decimal;///

use tracing::{error, info};

struct MySplineInterpolator {
    data_points: Vec<Point2D>,
}

impl SplineInterpolation<Point2D, Decimal> for MySplineInterpolator {
    fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        Ok(Point2D::new(x, x)) // Placeholder implementation
    }
}

// Demonstration of usage (not runnable code)
fn example_usage() {
    let interpolator = MySplineInterpolator {
        data_points: vec![],
    };

    match interpolator.spline_interpolate(Decimal::new(10, 1)) {
        Ok(point) => info!("Interpolated Point: {:?}", point),
        Err(err) => error!("Interpolation failed: {:?}", err),
    }
}
```
:::

## [§](#related-concepts){.doc-anchor}Related Concepts

- `LinearInterpolation`: Straight-line approximations between points.
- `CubicInterpolation`: For creating smooth curves with cubic
  polynomials.
- `BilinearInterpolation`: Interpolation techniques extended to two
  dimensions.

This trait is part of a broader framework supporting multiple
interpolation techniques, allowing developers to extend and choose
specific methods as per the dataset requirements.
::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

:::::: methods
::: {#tymethod.spline_interpolate .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/spline.rs.html#116){.src
.rightside}

#### fn [spline_interpolate](#tymethod.spline_interpolate){.fn}(&self, x: Input) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-spline_interpolateself-x-input---resultpoint-interpolationerror .code-header}
:::

:::: docblock
Interpolates a y-value for the provided x-coordinate using spline
interpolation.

- **Parameters:**

  - `x`: The x-coordinate value of type `Input` for which interpolation
    is required.

- **Returns:**

  - `Ok(Point)`: The interpolated point, typically containing both `x`
    and computed `y` values.
  - `Err(Self::Error)`: Represents an error during the interpolation
    process.

##### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use std::collections::BTreeSet;
use rust_decimal::Decimal;
use tracing::info;
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::SplineInterpolation;
let curve = Curve::new(BTreeSet::from_iter(vec![
           Point2D::new(Decimal::ZERO, Decimal::ZERO),
           Point2D::new(Decimal::ONE, Decimal::TWO),
       ]));
let result = curve.spline_interpolate(Decimal::from(2));

match result {
    Ok(point) => info!("Interpolated point: {:?}", point),
    Err(e) => info!("Interpolation failed: {:?}", e),
}
```
:::
::::
::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::: {#implementors-list}
:::: {#impl-SplineInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#771-953){.src
.rightside}[§](#impl-SplineInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [SplineInterpolation](trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-splineinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `SplineInterpolation` trait for the `Curve` struct,
providing functionality to perform cubic spline interpolation.
:::
::::

::: docblock
#### [§](#overview-1){.doc-anchor}Overview

This method calculates the interpolated `y` value for a given `x` value
by using cubic spline interpolation on the points in the `Curve`. The
method ensures a smooth transition between points by computing second
derivatives of the curve at each point, and uses those derivatives in
the spline interpolation formula.

#### [§](#parameters){.doc-anchor}Parameters

- `x`: The x-coordinate at which the curve should be interpolated. This
  value is passed as a `Decimal` for precise calculations.

#### [§](#returns){.doc-anchor}Returns

- On success, returns a `Point2D` instance representing the interpolated
  point.
- On error, returns a `CurvesError` indicating the reason for failure
  (e.g., insufficient points or an out-of-range x-coordinate).

#### [§](#errors){.doc-anchor}Errors

- Returns `CurvesError::InterpolationError` with an appropriate error
  message in the following cases:
  - If the curve contains fewer than three points, as spline
    interpolation requires at least three points.
  - If the given `x` value lies outside the range of x-coordinates
    spanned by the points in the curve.
  - If a valid segment for interpolation cannot be located.

#### [§](#details){.doc-anchor}Details

1.  **Validation**:
    - Ensures that there are at least three points in the curve for
      spline interpolation.
    - Validates that the provided `x` value is within the range of `x`
      values of the curve.
2.  **Exact Match**: If the `x` value matches the x-coordinate of an
    existing point, the corresponding `Point2D` is returned immediately.
3.  **Second Derivative Calculation**:
    - Uses a tridiagonal matrix to compute the second derivatives at
      each point. This step involves setting up the system of equations
      based on the boundary conditions (natural spline) and solving it
      using the Thomas algorithm.
4.  **Segment Identification**:
    - Determines the segment (interval between two consecutive points)
      in which the provided `x` value lies.
5.  **Interpolation**:
    - Computes the interpolated y-coordinate using the cubic spline
      formula, which is based on the second derivatives and the
      positions of the surrounding points.

#### [§](#implementation-notes){.doc-anchor}Implementation Notes

- This implementation uses `Decimal` from the `rust_decimal` crate to
  ensure high precision in calculations, making it suitable for
  scientific and financial applications.
- The Thomas algorithm is employed to solve the tridiagonal matrix
  system efficiently.
- The method assumes natural spline boundary conditions, where the
  second derivatives at the endpoints are set to zero, ensuring a smooth
  and continuous curve shape.

#### [§](#example-usage-1){.doc-anchor}Example Usage

Refer to the documentation for how to use the `SplineInterpolation`
trait, as examples are not provided inline with this implementation.

#### [§](#see-also){.doc-anchor}See Also

- [`SplineInterpolation`](trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"):
  The trait definition for spline interpolation.
- [`Point2D`](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents a point in 2D space.
- [`Curve`](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"):
  Represents a mathematical curve made up of points for interpolation.
- [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"):
  Enumerates possible errors during curve operations.
:::

::: {#impl-SplineInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#857-947){.src
.rightside}[§](#impl-SplineInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [SplineInterpolation](trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-splineinterpolationpoint3d-point2d-for-surface .code-header}
:::
:::::::
::::::::::::::::
:::::::::::::::::
