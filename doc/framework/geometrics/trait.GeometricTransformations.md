::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [GeometricTransformations]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#12-96){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait GeometricTransformations<Point> {
    type Error;

    // Required methods
    fn translate(&self, deltas: Vec<&Decimal>) -> Result<Self, Self::Error>
       where Self: Sized;
    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error>
       where Self: Sized;
    fn intersect_with(&self, other: &Self) -> Result<Vec<Point>, Self::Error>;
    fn derivative_at(&self, point: &Point) -> Result<Vec<Decimal>, Self::Error>;
    fn extrema(&self) -> Result<(Point, Point), Self::Error>;
    fn measure_under(
        &self,
        base_value: &Decimal,
    ) -> Result<Decimal, Self::Error>;
}
```

Expand description

::: docblock
## [§](#geometric-transformations){.doc-anchor}Geometric Transformations

A trait that defines common geometric transformations and operations for
geometric objects in any number of dimensions. This trait provides
methods for manipulating objects in space and analyzing their
properties.

### [§](#type-parameters){.doc-anchor}Type Parameters

- `Point` - The point type used to represent positions in the geometric
  space.
:::

## Required Associated Types[§](#required-associated-types){.anchor} {#required-associated-types .section-header}

::::: methods
::: {#associatedtype.Error .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#14){.src
.rightside}

#### type [Error](#associatedtype.Error){.associatedtype} {#type-error .code-header}
:::

::: docblock
The error type that can be returned by geometric operations.
:::
:::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::::::::::::: methods
:::: {#tymethod.translate .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#28-30){.src
.rightside}

#### fn [translate](#tymethod.translate){.fn}(&self, deltas: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-translateself-deltas-vecdecimal---resultself-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Translates the geometric object by specified amounts along each
dimension.

##### [§](#arguments){.doc-anchor}Arguments

- `deltas` - A vector of decimal values representing the translation
  distance along each dimension. The length of this vector should match
  the dimensionality of the geometric object.

##### [§](#returns){.doc-anchor}Returns

A new instance of the geometric object after translation, or an error if
the transformation could not be applied.
:::

:::: {#tymethod.scale .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#44-46){.src
.rightside}

#### fn [scale](#tymethod.scale){.fn}(&self, factors: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-scaleself-factors-vecdecimal---resultself-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Scales the geometric object by specified factors along each dimension.

##### [§](#arguments-1){.doc-anchor}Arguments

- `factors` - A vector of decimal values representing the scaling factor
  for each dimension. The length of this vector should match the
  dimensionality of the geometric object.

##### [§](#returns-1){.doc-anchor}Returns

A new instance of the geometric object after scaling, or an error if the
transformation could not be applied.
:::

::: {#tymethod.intersect_with .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#58){.src
.rightside}

#### fn [intersect_with](#tymethod.intersect_with){.fn}(&self, other: &Self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Point\>, Self::[Error](trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-intersect_withself-other-self---resultvecpoint-selferror .code-header}
:::

::: docblock
Finds all intersection points between this geometric object and another.

##### [§](#arguments-2){.doc-anchor}Arguments

- `other` - The other geometric object to find intersections with.

##### [§](#returns-2){.doc-anchor}Returns

A vector of intersection points, or an error if the intersections could
not be determined.
:::

::: {#tymethod.derivative_at .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#73){.src
.rightside}

#### fn [derivative_at](#tymethod.derivative_at){.fn}(&self, point: [&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\>, Self::[Error](trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-derivative_atself-point-point---resultvecdecimal-selferror .code-header}
:::

::: docblock
Calculates the derivative at a specific point on the geometric object.

For curves, this represents the tangent. For surfaces, this can
represent partial derivatives.

##### [§](#arguments-3){.doc-anchor}Arguments

- `point` - The point at which to calculate the derivative.

##### [§](#returns-3){.doc-anchor}Returns

A vector containing the derivative values along each dimension, or an
error if the derivative could not be calculated.
:::

::: {#tymethod.extrema .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#81){.src
.rightside}

#### fn [extrema](#tymethod.extrema){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[(Point, Point)](https://doc.rust-lang.org/1.86.0/std/primitive.tuple.html){.primitive}, Self::[Error](trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-extremaself---resultpoint-point-selferror .code-header}
:::

::: docblock
Finds the extrema (minimum and maximum points) of the geometric object.

##### [§](#returns-4){.doc-anchor}Returns

A tuple containing the minimum and maximum points of the geometric
object, or an error if the extrema could not be determined.
:::

::: {#tymethod.measure_under .section .method}
[Source](../../src/optionstratlib/geometrics/operations/transformations.rs.html#95){.src
.rightside}

#### fn [measure_under](#tymethod.measure_under){.fn}(&self, base_value: &Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, Self::[Error](trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-measure_underself-base_value-decimal---resultdecimal-selferror .code-header}
:::

::: docblock
Calculates the area or volume under the geometric object relative to a
base value.

For curves, this calculates the area. For higher-dimensional objects,
this calculates volume.

##### [§](#arguments-4){.doc-anchor}Arguments

- `base_value` - The reference value to measure from.

##### [§](#returns-5){.doc-anchor}Returns

The calculated area or volume, or an error if the calculation failed.
:::
:::::::::::::::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.86.0/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::::: {#implementors-list}
::: {#impl-GeometricTransformations%3CPoint2D%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1536-1645){.src
.rightside}[§](#impl-GeometricTransformations%3CPoint2D%3E-for-Curve){.anchor}

### impl [GeometricTransformations](trait.GeometricTransformations.html "trait optionstratlib::geometrics::GeometricTransformations"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-geometrictransformationspoint2d-for-curve .code-header}
:::

:::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1537){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::
::::

::: {#impl-GeometricTransformations%3CPoint3D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1374-1600){.src
.rightside}[§](#impl-GeometricTransformations%3CPoint3D%3E-for-Surface){.anchor}

### impl [GeometricTransformations](trait.GeometricTransformations.html "trait optionstratlib::geometrics::GeometricTransformations"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-geometrictransformationspoint3d-for-surface .code-header}
:::

:::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1375){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::
::::
:::::::::
::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::
