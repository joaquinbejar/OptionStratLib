::::::::::::::::: width-limiter
:::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [BiLinearInterpolation]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/interpolation/bilinear.rs.html#62-91){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait BiLinearInterpolation<Point, Input> {
    // Required method
    fn bilinear_interpolate(
        &self,
        x: Input,
    ) -> Result<Point, InterpolationError>;
}
```

Expand description

:::: docblock
A trait for bilinear interpolation on 2D data grids.

## [§](#purpose){.doc-anchor}Purpose

The `BiLinearInterpolation` trait is designed to perform bilinear
interpolation to estimate intermediate values within a grid of 2D
points. This method is often used in numerical computation tasks, such
as image processing, terrain modeling, and scientific data
visualization.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `Point`: The output type, typically used to represent the interpolated
  2D point.
- `Input`: The input type for the interpolation parameter, typically a
  scalar value.

## [§](#associated-type){.doc-anchor}Associated Type

- `Error`: Defines the type returned in case of a failure during
  interpolation.

## [§](#method){.doc-anchor}Method

- [`bilinear_interpolate`](#method.bilinear_interpolate): Computes the
  interpolated value at the given input, returning either the result or
  an error if the operation cannot proceed.

## [§](#errors){.doc-anchor}Errors

Any errors encountered during interpolation are encapsulated in the type
`Error`. This trait is expected to return meaningful errors in cases
like:

- Insufficient or invalid data for computation.
- Inputs that are out of bounds for the given dataset.
- Issues specific to the interpolation logic.

## [§](#example-usage){.doc-anchor}Example Usage

Below is an example demonstrating how an implementing struct might use
this trait:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use optionstratlib::curves::Point2D;
use optionstratlib::error::InterpolationError;
use optionstratlib::geometrics::BiLinearInterpolation;
struct GridInterpolator {
    // Implementation-specific fields like the grid or data.
}

impl BiLinearInterpolation<Point2D, Decimal> for GridInterpolator {

    fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        Ok(Point2D::new(x, x)) // Placeholder implementation
    }
}
```
:::

In this example:

- `GridInterpolator` implements the trait for bilinear interpolation.
- The `bilinear_interpolate` method calculates the interpolated
  `Point2D` for a given `x` value.

## [§](#related-types){.doc-anchor}Related Types

- [`Point2D`](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  A struct representing a 2D point with `x` and `y` coordinates.
- [`CurvesError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"):
  A recommended error type for detailed error categorization.

## [§](#see-also){.doc-anchor}See Also

- [`crate::geometrics::interpolation::InterpolationType`](enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"):
  A module defining different types of interpolation methods.
- [`crate::geometrics::interpolation::LinearInterpolation`](trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"):
  A simpler interpolation method for one-dimensional data.
::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

:::::: methods
::: {#tymethod.bilinear_interpolate .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/bilinear.rs.html#90){.src
.rightside}

#### fn [bilinear_interpolate](#tymethod.bilinear_interpolate){.fn}(&self, x: Input) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-bilinear_interpolateself-x-input---resultpoint-interpolationerror .code-header}
:::

:::: docblock
Performs bilinear interpolation to compute a value for the given input.

##### [§](#parameters){.doc-anchor}Parameters

- `x`: The input value (e.g., an `x` coordinate in 2D space) for which
  the interpolation is performed.

##### [§](#returns){.doc-anchor}Returns

- `Ok(Point)`: The interpolated point (e.g., a `Point2D`) representing
  the computed values.
- `Err(Self::Error)`: An error indicating why the interpolation could
  not be performed.

##### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use std::collections::BTreeSet;
use rust_decimal::Decimal;
use tracing::info;
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::BiLinearInterpolation;
let curve = Curve::new(BTreeSet::from_iter(vec![
           Point2D::new(Decimal::ZERO, Decimal::ZERO),
           Point2D::new(Decimal::ONE, Decimal::TWO),
       ]));
let result = curve.bilinear_interpolate(Decimal::from(2));

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
:::: {#impl-BiLinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#460-536){.src
.rightside}[§](#impl-BiLinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [BiLinearInterpolation](trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-bilinearinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implementation of the `BiLinearInterpolation` trait for the `Curve`
struct.
:::
::::

::: docblock
This function performs bilinear interpolation, which is used to estimate
the value of a function at a given point `x` within a grid defined by at
least 4 points. Bilinear interpolation combines two linear
interpolations: one along the x-axis and another along the y-axis,
within the bounds defined by the four surrounding points.

#### [§](#parameters-1){.doc-anchor}Parameters

- **`x`**: The x-coordinate value for which the interpolation will be
  performed. Must fall within the range of the x-coordinates of the
  curve's points.

#### [§](#returns-1){.doc-anchor}Returns

- **`Ok(Point2D)`**: A `Point2D` instance representing the interpolated
  point at the given x-coordinate, with both x and y provided as
  `Decimal` values.
- **`Err(CurvesError)`**: An error if the interpolation cannot be
  performed due to one of the following reasons:
  - There are fewer than 4 points in the curve.
  - The x-coordinate does not fall within a valid range of points.
  - The bracketing points for the given x-coordinate cannot be
    determined.

#### [§](#function-details){.doc-anchor}Function Details

1.  **Input Validation**:

    - Ensures the curve has at least 4 points, as required for bilinear
      interpolation.
    - Returns an error if the condition is not met.

2.  **Exact Match Check**:

    - If the x-coordinate matches exactly with one of the points in the
      curve, the corresponding `Point2D` is returned directly.

3.  **Bracket Point Search**:

    - Determines the bracketing points (`i` and `j`) for the given
      x-coordinate using the `find_bracket_points` method.

4.  **Grid Point Selection**:

    - Extracts four points from the curve:
      - `p11`: Bottom-left point.
      - `p12`: Bottom-right point.
      - `p21`: Top-left point.
      - `p22`: Top-right point.

5.  **x-Normalization**:

    - Computes a normalized x value (`dx` in the range `[0,1]`), used to
      perform interpolation along the x-axis within the defined grid.

6.  **Linear Interpolation**:

    - First performs interpolation along the x-axis for the bottom and
      top edges of the grid, resulting in partial y-values `bottom` and
      `top`.
    - Then, interpolates along the y-axis between the bottom and top
      edge values, resulting in the final interpolated y-coordinate.

7.  **Output**:

    - Returns the interpolated `Point2D` with the input x-coordinate and
      the computed y-coordinate.

#### [§](#errors-1){.doc-anchor}Errors

- **Insufficient Points**: If the curve contains fewer than 4 points, a
  `CurvesError` with a relevant message is returned.
- **Out-of-Range x**: If the x-coordinate cannot be bracketed by points
  in the curve, a `CurvesError` is returned with an appropriate message.

#### [§](#related-traits){.doc-anchor}Related Traits

- [`BiLinearInterpolation`](trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"):
  The trait defining this method.
- [`Interpolate`](trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"):
  Ensures compatibility of the curve with multiple interpolation
  methods.

#### [§](#see-also-1){.doc-anchor}See Also

- [`Curve`](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"):
  The overarching structure that represents the curve.
- [`Point2D`](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The data type used to represent individual points on the curve.
- [`find_bracket_points`](trait.Interpolate.html#method.find_bracket_points "method optionstratlib::geometrics::Interpolate::find_bracket_points"):
  A helper method used to locate the two points that bracket the given
  x-coordinate.
:::

::: {#impl-BiLinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#703-782){.src
.rightside}[§](#impl-BiLinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [BiLinearInterpolation](trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-bilinearinterpolationpoint3d-point2d-for-surface .code-header}
:::
:::::::
::::::::::::::::
:::::::::::::::::
