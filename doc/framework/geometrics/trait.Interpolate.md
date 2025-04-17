::::::::::::::::::::: width-limiter
:::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [Interpolate]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#58-133){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Interpolate<Point, Input>:
    LinearInterpolation<Point, Input>
    + BiLinearInterpolation<Point, Input>
    + CubicInterpolation<Point, Input>
    + SplineInterpolation<Point, Input>
    + GeometricObject<Point, Input>where
    Input: HasX,
    Point: HasX + Clone,{
    // Provided methods
    fn interpolate(
        &self,
        x: Input,
        interpolation_type: InterpolationType,
    ) -> Result<Point, InterpolationError> { ... }
    fn find_bracket_points(
        &self,
        x: Input,
    ) -> Result<(usize, usize), InterpolationError> { ... }
}
```

Expand description

::: docblock
A trait for performing various types of interpolation on a set of 2D
points.

## [§](#overview){.doc-anchor}Overview

The `Interpolate` trait unifies methods for interpolating values in 2D
Cartesian space. Implementers of this trait must support linear,
bilinear, cubic, and spline interpolation methods.

This trait is designed for use with numerical and graphical applications
requiring high-precision interpolation of data points. It provides
functionality to retrieve the points, interpolate a value, and find
bracketing points for a given x-coordinate.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `Point`: The point type that contains coordinates, must implement
  `HasX` and `Clone`
- `Input`: The input type for interpolation queries, must implement
  `HasX`

## [§](#methods){.doc-anchor}Methods

### [§](#interpolate){.doc-anchor}`interpolate`

Interpolates a value for a given x-coordinate using a specified
interpolation method.

#### [§](#parameters){.doc-anchor}Parameters

- `x`: The x-coordinate at which to interpolate
- `interpolation_type`: The type of interpolation algorithm to use

#### [§](#returns){.doc-anchor}Returns

- `Result<Point, InterpolationError>`: The interpolated point or an
  error

### [§](#find_bracket_points){.doc-anchor}`find_bracket_points`

Identifies the pair of points that bracket the target x-coordinate for
interpolation.

#### [§](#parameters-1){.doc-anchor}Parameters

- `x`: The x-coordinate to bracket

#### [§](#returns-1){.doc-anchor}Returns

- `Result<(usize, usize), InterpolationError>`: The indices of the
  bracketing points or an error

## [§](#error-handling){.doc-anchor}Error Handling

Methods in this trait return `InterpolationError` to represent various
issues during interpolation:

- Insufficient points for interpolation (need at least two points)
- X-coordinate outside the valid range of data points
- Failure to find bracketing points
- Specific errors from the various interpolation algorithms

## [§](#requirements){.doc-anchor}Requirements

Implementers must also implement the following traits:

- `LinearInterpolation`
- `BiLinearInterpolation`
- `CubicInterpolation`
- `SplineInterpolation`
- `GeometricObject`
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::: methods
::: {#method.interpolate .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#80-91){.src
.rightside}

#### fn [interpolate](#method.interpolate){.fn}( &self, x: Input, interpolation_type: [InterpolationType](enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-interpolate-self-x-input-interpolation_type-interpolationtype---resultpoint-interpolationerror .code-header}
:::

::: docblock
Interpolates a value at the given x-coordinate using the specified
interpolation method.

This method acts as a facade over the individual interpolation
algorithms, delegating to the appropriate method based on the requested
interpolation type.

##### [§](#parameters-2){.doc-anchor}Parameters

- `x`: The x-coordinate at which to interpolate
- `interpolation_type`: The interpolation algorithm to use

##### [§](#returns-2){.doc-anchor}Returns

- `Ok(Point)`: The successfully interpolated point
- `Err(InterpolationError)`: If interpolation fails for any reason
:::

::: {#method.find_bracket_points .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#110-132){.src
.rightside}

#### fn [find_bracket_points](#method.find_bracket_points){.fn}( &self, x: Input, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}), [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-find_bracket_points-self-x-input---resultusize-usize-interpolationerror .code-header}
:::

::: docblock
Finds the indices of points that bracket the given x-coordinate.

This utility method identifies the pair of consecutive points in the
dataset where the first point's x-coordinate is less than or equal to
the target x, and the second point's x-coordinate is greater than or
equal to the target x.

##### [§](#parameters-3){.doc-anchor}Parameters

- `x`: The x-coordinate for which to find bracketing points

##### [§](#returns-3){.doc-anchor}Returns

- `Ok((usize, usize))`: The indices of the two bracketing points
- `Err(InterpolationError)`: If bracketing points cannot be found

##### [§](#errors){.doc-anchor}Errors

- If there are fewer than 2 points in the dataset
- If the requested x-coordinate is outside the range of available points
- If bracketing points cannot be determined for any other reason
:::
:::::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.86.0/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::: {#implementors-list}
:::: {#impl-Interpolate%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#287){.src
.rightside}[§](#impl-Interpolate%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [Interpolate](trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-interpolatepoint2d-decimal-for-curve .code-header}

::: docblock
Implementation of the `Interpolate` trait for the `Curve` struct.
:::
::::

::: docblock
This implementation integrates the `get_points` method for the `Curve`
structure, providing access to its internal points. The `Interpolate`
trait ensures compatibility with various interpolation methods such as
Linear, BiLinear, Cubic, and Spline interpolations. By implementing this
trait, `Curve` gains the ability to perform interpolation operations and
access bracketing points.

#### [§](#traits-involved){.doc-anchor}Traits Involved

The `Interpolate` trait is an aggregation of multiple
interpolation-related traits:

- [`LinearInterpolation`](trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation")
- [`BiLinearInterpolation`](trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation")
- [`CubicInterpolation`](trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation")
- [`SplineInterpolation`](trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation")

These underlying traits implement specific interpolation algorithms,
enabling `Curve` to support a robust set of interpolation options
through the associated methods. Depending on the use case and provided
parameters (e.g., interpolation type and target x-coordinate), the
appropriate algorithm is invoked.

#### [§](#see-also){.doc-anchor}See Also

- [`Curve`](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"):
  The underlying mathematical structure being interpolated.
- [`Point2D`](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The fundamental data type for the curve's points.
- [`Interpolate`](trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"):
  The trait defining interpolation operations.
:::

:::: {#impl-Interpolate%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#585){.src
.rightside}[§](#impl-Interpolate%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [Interpolate](trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-interpolatepoint3d-point2d-for-surface .code-header}

::: docblock
Implementation of the `Interpolate` trait for the `Surface` type,
enabling interpolation from 3D surface points to 2D points.
:::
::::

:::: docblock
#### [§](#overview-1){.doc-anchor}Overview

This implementation allows a `Surface` object to perform various types
of interpolation (linear, bilinear, cubic, and spline) by projecting 3D
points from the surface to 2D points.

#### [§](#functionality){.doc-anchor}Functionality

By implementing the `Interpolate` trait, `Surface` gains the following
capabilities:

- Interpolating between 3D surface points to produce 2D projections
- Finding bracket points for interpolation operations
- Supporting multiple interpolation algorithms through the trait's
  methods

#### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::surfaces::{Surface, Point3D};
use optionstratlib::curves::Point2D;
use optionstratlib::geometrics::{Interpolate, InterpolationType};

let surface = Surface::new(Default::default());

// Interpolate a 2D point at a specific position using linear interpolation
let input_point = Point2D { x: dec!(1.5), y: dec!(2.0) };
let result = surface.interpolate(input_point, InterpolationType::Linear);
```
:::

#### [§](#related-traits){.doc-anchor}Related Traits

This implementation relies on the surface also implementing:

- `LinearInterpolation<Point3D, Point2D>`
- `BiLinearInterpolation<Point3D, Point2D>`
- `CubicInterpolation<Point3D, Point2D>`
- `SplineInterpolation<Point3D, Point2D>`
- `GeometricObject<Point3D, Point2D>`
::::
::::::::::
::::::::::::::::::::
:::::::::::::::::::::
