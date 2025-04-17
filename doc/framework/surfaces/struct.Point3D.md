:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[surfaces](index.html)
:::

# Struct [Point3D]{.struct}Copy item path

[[Source](../../src/optionstratlib/surfaces/types.rs.html#46-53){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Point3D {
    pub x: Decimal,
    pub y: Decimal,
    pub z: Decimal,
}
```

Expand description

:::: docblock
Represents a point in three-dimensional space with `x`, `y` and `z`
coordinates.

## [§](#overview){.doc-anchor}Overview

The `Point3D` struct defines a point in a 3D Cartesian coordinate
system. All coordinates (`x`, `y`, and `z`) are stored as `Decimal`
values to provide high precision, making it suitable for applications
requiring accurate numerical calculations.

## [§](#fields-1){.doc-anchor}Fields {#fields-1}

- **x**: The x-coordinate of the point, represented as a `Decimal`
- **y**: The y-coordinate of the point, represented as a `Decimal`
- **z**: The z-coordinate of the point, represented as a `Decimal`

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::surfaces::Point3D;

// Create a new 3D point
let point = Point3D {
    x: dec!(1.5),
    y: dec!(2.0),
    z: dec!(-3.25),
};
```
:::

## [§](#usage){.doc-anchor}Usage

`Point3D` is primarily used within the surface module to represent
vertices of 3D surfaces and for various geometric calculations. The
high-precision `Decimal` type ensures accuracy in scientific and
engineering applications.
::::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.x){.anchor .field}`x: Decimal`]{#structfield.x
.structfield .section-header}

::: docblock
The x-coordinate in the Cartesian system
:::

[[§](#structfield.y){.anchor .field}`y: Decimal`]{#structfield.y
.structfield .section-header}

::: docblock
The y-coordinate in the Cartesian system
:::

[[§](#structfield.z){.anchor .field}`z: Decimal`]{#structfield.z
.structfield .section-header}

::: docblock
The z-coordinate in the Cartesian system
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::: {#implementations-list}
::: {#impl-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#87-203){.src
.rightside}[§](#impl-Point3D){.anchor}

### impl [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-point3d .code-header}
:::

:::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../src/optionstratlib/surfaces/types.rs.html#94-100){.src
.rightside}

#### pub fn [new](#method.new){.fn}\<T: [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Decimal\>, U: [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Decimal\>, V: [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Decimal\>\>( x: T, y: U, z: V, ) -\> Self {#pub-fn-newt-intodecimal-u-intodecimal-v-intodecimal-x-t-y-u-z-v---self .code-header}
:::

::: docblock
Creates a new instance of `Point3D` using the specified coordinates.

##### [§](#parameters){.doc-anchor}Parameters

- `x`: The x-coordinate, which implements `Into<Decimal>`
- `y`: The y-coordinate, which implements `Into<Decimal>`
- `z`: The z-coordinate, which implements `Into<Decimal>`
:::

::: {#method.to_tuple .section .method}
[Source](../../src/optionstratlib/surfaces/types.rs.html#108-134){.src
.rightside}

#### pub fn [to_tuple](#method.to_tuple){.fn}\<T: [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<Decimal\> + \'static, U: [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<Decimal\> + \'static, V: [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<Decimal\> + \'static\>( &self, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[(T, U, V)](https://doc.rust-lang.org/1.86.0/std/primitive.tuple.html){.primitive}, [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#pub-fn-to_tuplet-fromdecimal-static-u-fromdecimal-static-v-fromdecimal-static-self---resultt-u-v-surfaceerror .code-header}
:::

::: docblock
Converts the Point3D to a tuple of three values.

##### [§](#type-parameters){.doc-anchor}Type Parameters

- `T`: Type for x-coordinate
- `U`: Type for y-coordinate
- `V`: Type for z-coordinate
:::

::: {#method.from_tuple .section .method}
[Source](../../src/optionstratlib/surfaces/types.rs.html#137-143){.src
.rightside}

#### pub fn [from_tuple](#method.from_tuple){.fn}\<T: [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Decimal\>, U: [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Decimal\>, V: [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Decimal\>\>( x: T, y: U, z: V, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#pub-fn-from_tuplet-intodecimal-u-intodecimal-v-intodecimal-x-t-y-u-z-v---resultself-surfaceerror .code-header}
:::

::: docblock
Creates a Point3D from a tuple of three values.
:::

::: {#method.to_f64_tuple .section .method}
[Source](../../src/optionstratlib/surfaces/types.rs.html#146-157){.src
.rightside}

#### pub fn [to_f64_tuple](#method.to_f64_tuple){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}), [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#pub-fn-to_f64_tupleself---resultf64-f64-f64-surfaceerror .code-header}
:::

::: docblock
Converts the Point3D to a tuple of f64 values.
:::

::: {#method.from_f64_tuple .section .method}
[Source](../../src/optionstratlib/surfaces/types.rs.html#160-171){.src
.rightside}

#### pub fn [from_f64_tuple](#method.from_f64_tuple){.fn}(x: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, y: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, z: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#pub-fn-from_f64_tuplex-f64-y-f64-z-f64---resultself-surfaceerror .code-header}
:::

::: docblock
Creates a Point3D from a tuple of f64 values.
:::

::: {#method.point2d .section .method}
[Source](../../src/optionstratlib/surfaces/types.rs.html#200-202){.src
.rightside}

#### pub fn [point2d](#method.point2d){.fn}(&self) -\> [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#pub-fn-point2dself---boxpoint2d .code-header}
:::

:::: docblock
Converts this `Point3D` instance to a `Point2D` by projecting onto the
XY plane.

##### [§](#overview-1){.doc-anchor}Overview

This method creates a new `Point2D` instance using only the `x` and `y`
coordinates of the current `Point3D` object, effectively projecting the
3D point onto the XY plane. The `z` coordinate is discarded in this
operation.

##### [§](#returns){.doc-anchor}Returns

A heap-allocated `Point2D` instance (`Box<Point2D>`) containing the `x`
and `y` coordinates from this `Point3D` object.

##### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::surfaces::Point3D;

let point3d = Point3D { x: dec!(1.5), y: dec!(2.0), z: dec!(3.5) };
let point2d = point3d.point2d();
assert_eq!(point2d.x, dec!(1.5));
assert_eq!(point2d.y, dec!(2.0));
```
:::

##### [§](#use-cases){.doc-anchor}Use Cases

This method is useful when:

- Working with visualization that requires 2D projections of 3D points
- Analyzing specific planes within a 3D model
- Converting between coordinate systems from 3D to 2D
::::
::::::::::::::::
::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-AxisOperations%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1287-1322){.src
.rightside}[§](#impl-AxisOperations%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [AxisOperations](../geometrics/trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-axisoperationspoint3d-point2d-for-surface .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1288){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](../geometrics/trait.AxisOperations.html#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::

::: docblock
The type of error that can occur during point operations
:::

::: {#method.contains_point .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1290-1292){.src
.rightside}[§](#method.contains_point){.anchor}

#### fn [contains_point](../geometrics/trait.AxisOperations.html#tymethod.contains_point){.fn}(&self, x: &[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-contains_pointself-x-point2d---bool .code-header}
:::

::: docblock
Checks if a coordinate value exists in the structure. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.contains_point)
:::

::: {#method.get_index_values .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1294-1296){.src
.rightside}[§](#method.get_index_values){.anchor}

#### fn [get_index_values](../geometrics/trait.AxisOperations.html#tymethod.get_index_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#fn-get_index_valuesself---vecpoint2d .code-header}
:::

::: docblock
Returns a vector of references to all index values in the structure.
[Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_index_values)
:::

::: {#method.get_values .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1298-1304){.src
.rightside}[§](#method.get_values){.anchor}

#### fn [get_values](../geometrics/trait.AxisOperations.html#tymethod.get_values){.fn}(&self, x: [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\> {#fn-get_valuesself-x-point2d---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to dependent values for a given
coordinate. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_values)
:::

::: {#method.get_closest_point .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1306-1317){.src
.rightside}[§](#method.get_closest_point){.anchor}

#### fn [get_closest_point](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point){.fn}(&self, x: &[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-get_closest_pointself-x-point2d---resultpoint3d-selferror .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point)
:::

::: {#method.get_point .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1319-1321){.src
.rightside}[§](#method.get_point){.anchor}

#### fn [get_point](../geometrics/trait.AxisOperations.html#tymethod.get_point){.fn}(&self, x: &[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> {#fn-get_pointself-x-point2d---optionpoint3d .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_point)
:::

::: {#method.merge_indexes .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#85-115){.src
.rightside}[§](#method.merge_indexes){.anchor}

#### fn [merge_indexes](../geometrics/trait.AxisOperations.html#method.merge_indexes){.fn}(&self, axis: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\>) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_indexesself-axis-vecinput---vecinput .code-header}
:::

::: docblock
Merges the index values from the current structure with an additional
set of indices. This combines self.get_index_values() with the provided
axis vector to create a single vector of unique indices. [Read
more](../geometrics/trait.AxisOperations.html#method.merge_indexes)
:::
:::::::::::::::::

::: {#impl-BiLinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#690-769){.src
.rightside}[§](#impl-BiLinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [BiLinearInterpolation](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-bilinearinterpolationpoint3d-point2d-for-surface .code-header}
:::

::::: impl-items
::: {#method.bilinear_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#691-768){.src
.rightside}[§](#method.bilinear_interpolate){.anchor}

#### fn [bilinear_interpolate](../geometrics/trait.BiLinearInterpolation.html#tymethod.bilinear_interpolate){.fn}( &self, xy: [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-bilinear_interpolate-self-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
Performs bilinear interpolation to compute a value for the given input.
[Read
more](../geometrics/trait.BiLinearInterpolation.html#tymethod.bilinear_interpolate)
:::
:::::

::: {#impl-Clone-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#impl-Clone-for-Point3D){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-clone-for-point3d .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#fn-cloneself---point3d .code-header}
:::

::: docblock
Returns a copy of the value. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#174){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-CubicInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#771-842){.src
.rightside}[§](#impl-CubicInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [CubicInterpolation](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-cubicinterpolationpoint3d-point2d-for-surface .code-header}
:::

::::: impl-items
::: {#method.cubic_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#772-841){.src
.rightside}[§](#method.cubic_interpolate){.anchor}

#### fn [cubic_interpolate](../geometrics/trait.CubicInterpolation.html#tymethod.cubic_interpolate){.fn}(&self, xy: [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-cubic_interpolateself-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
Interpolates a new point on the curve for a given `x` input value using
cubic interpolation. [Read
more](../geometrics/trait.CubicInterpolation.html#tymethod.cubic_interpolate)
:::
:::::

::: {#impl-Debug-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#impl-Debug-for-Point3D){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-debug-for-point3d .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Point3D){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#implde-deserializede-for-point3d .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#55-59){.src
.rightside}[§](#impl-Display-for-Point3D){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-display-for-point3d .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#56-58){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-From%3C%26Point3D%3E-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#205-209){.src
.rightside}[§](#impl-From%3C%26Point3D%3E-for-Point3D){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-frompoint3d-for-point3d .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#206-208){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(point: &[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}) -\> Self {#fn-frompoint-point3d---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

:::: {#impl-GeometricObject%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#322-522){.src
.rightside}[§](#impl-GeometricObject%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [GeometricObject](../geometrics/trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-geometricobjectpoint3d-point2d-for-surface .code-header}

::: docblock
Implementation of the `GeometricObject` trait for the `Surface` struct.
:::
::::

::: docblock
This implementation provides functionality to create and manipulate 3D
surfaces using points in three-dimensional space. It supports
construction from explicit point collections or through parametric
functions.

#### [§](#type-parameters-3){.doc-anchor}Type Parameters {#type-parameters-3}

- Uses `Point3D` as the points that form the surface
- Uses `Point2D` as the parametric input for surface generation

#### [§](#methods){.doc-anchor}Methods

- `get_points()`: Retrieves all points in the surface
- `from_vector()`: Constructs a surface from a vector of points
- `construct()`: Creates a surface using different construction methods

#### [§](#error-handling){.doc-anchor}Error Handling

Uses `SurfaceError` for various error conditions, including:

- Empty point collections
- Invalid construction parameters
- Errors during parametric function evaluation
:::

:::::::::::::::::::::::: impl-items
::: {#method.get_points .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#353-355){.src
.rightside}[§](#method.get_points){.anchor}

#### fn [get_points](../geometrics/trait.GeometricObject.html#tymethod.get_points){.fn}(&self) -\> [BTreeSet](https://doc.rust-lang.org/1.86.0/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}\<&[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> {#fn-get_pointsself---btreesetpoint3d .code-header}
:::

:::: docblock
Returns a borrowed reference to all points in the surface as an ordered
set

##### [§](#returns-1){.doc-anchor}Returns

- `BTreeSet<&Point3D>` - A sorted set containing references to all
  points that define the surface, maintaining the natural ordering of
  points

##### [§](#example-1){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::surfaces::{Surface, Point3D};
use std::collections::BTreeSet;
use rust_decimal_macros::dec;
use optionstratlib::geometrics::GeometricObject;

// Create a surface with some points
let mut surface = Surface {
    points: BTreeSet::new(),
    x_range: (dec!(0), dec!(10)),
    y_range: (dec!(0), dec!(10)),
};

// Add points to the surface
surface.points.insert(Point3D { x: dec!(1.0), y: dec!(2.0), z: dec!(3.0) });
surface.points.insert(Point3D { x: dec!(4.0), y: dec!(5.0), z: dec!(6.0) });

// Get references to all points in the surface
let points = surface.get_points();
assert_eq!(points.len(), 2);
```
:::
::::

:::: {#method.from_vector .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#396-408){.src
.rightside}[§](#method.from_vector){.anchor}

#### fn [from_vector](../geometrics/trait.GeometricObject.html#tymethod.from_vector){.fn}\<T\>(points: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<T\>) -\> Self {#fn-from_vectortpoints-vect---self .code-header}

::: where
where T:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::: docblock
Creates a new Surface from a vector of points that can be converted into
Point3D objects.

This method constructs a Surface by converting each point in the input
vector to a Point3D and collecting them into an ordered set. It also
calculates the x and y coordinate ranges of the points to define the
surface's boundaries.

##### [§](#type-parameters-1){.doc-anchor}Type Parameters {#type-parameters-1}

- `T`: A type that can be converted into Point3D via the Into trait and
  can be cloned.

##### [§](#parameters-1){.doc-anchor}Parameters

- `points`: A vector of objects that can be converted to Point3D.

##### [§](#returns-2){.doc-anchor}Returns

A new Surface instance containing the converted points and their
coordinate ranges.

##### [§](#example-2){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::surfaces::{Surface, Point3D};
use optionstratlib::geometrics::GeometricObject;
use rust_decimal_macros::dec;

// Create points data
let points = vec![
    Point3D { x: dec!(1.0), y: dec!(2.0), z: dec!(3.0) },
    Point3D { x: dec!(4.0), y: dec!(5.0), z: dec!(6.0) }
];

// Create a surface from the points
let surface = Surface::from_vector(points);

// The surface will contain both points and have x_range and y_range calculated automatically
assert_eq!(surface.points.len(), 2);
assert_eq!(surface.x_range, (dec!(1.0), dec!(4.0)));
assert_eq!(surface.y_range, (dec!(2.0), dec!(5.0)));
```
:::
::::

:::: {#method.construct .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#468-521){.src
.rightside}[§](#method.construct){.anchor}

#### fn [construct](../geometrics/trait.GeometricObject.html#tymethod.construct){.fn}\<T\>(method: T) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricObject.html#associatedtype.Error "type optionstratlib::geometrics::GeometricObject::Error"){.associatedtype}\> {#fn-constructtmethod-t---resultself-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
T:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[ConstructionMethod](../geometrics/enum.ConstructionMethod.html "enum optionstratlib::geometrics::ConstructionMethod"){.enum}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct},
[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\>\>,
:::
::::

::::: docblock
Constructs a Surface from a given construction method.

This function creates a Surface object from either a set of 3D points or
a parametric function.

##### [§](#parameters-2){.doc-anchor}Parameters

- `method` - A construction method that can be converted into a
  `ConstructionMethod<Point3D, Point2D>`

##### [§](#type-parameters-2){.doc-anchor}Type Parameters {#type-parameters-2}

- `T` - Type that can be converted into a
  `ConstructionMethod<Point3D, Point2D>`

##### [§](#returns-3){.doc-anchor}Returns

- `Result<Self, Self::Error>` - Either a successfully constructed
  Surface or an error

##### [§](#errors){.doc-anchor}Errors

- `SurfaceError::Point3DError` - If an empty points array is provided
- `SurfaceError::ConstructionError` - If invalid parameters are provided
  or the parametric function fails

##### [§](#examples-1){.doc-anchor}Examples

###### [§](#creating-from-existing-points){.doc-anchor}Creating from existing points

::: example-wrap
``` {.rust .rust-example-rendered}
use std::collections::BTreeSet;
use optionstratlib::geometrics::{ConstructionMethod, GeometricObject};
use optionstratlib::surfaces::{Point3D, Surface};
let points = BTreeSet::from_iter(vec![
    Point3D::new(0, 0, 0),
    Point3D::new(1, 0, 1),
    Point3D::new(0, 1, 1),
]);
let surface = Surface::construct(ConstructionMethod::FromData { points }).unwrap();
```
:::

###### [§](#creating-from-a-parametric-function){.doc-anchor}Creating from a parametric function

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::curves::Point2D;
use optionstratlib::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject, ResultPoint};
use optionstratlib::surfaces::{Point3D, Surface};
let params = ConstructionParams::D3 {
    x_start: dec!(-1.0),
    x_end: dec!(1.0),
    y_start: dec!(-1.0),
    y_end: dec!(1.0),
    x_steps: 20,
    y_steps: 20,
};

// Parametric function defining a paraboloid: z = x² + y²
let f = Box::new(|p: Point2D| -> ResultPoint<Point3D> {
    Ok(Point3D {
        x: p.x,
        y: p.y,
        z: p.x * p.x + p.y * p.y,
    })
});

let surface = Surface::construct(ConstructionMethod::Parametric { f, params }).unwrap();
```
:::
:::::

::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#323){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](../geometrics/trait.GeometricObject.html#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror-1 .code-header}
:::

::: docblock
Type alias for any errors that might occur during the construction of
the geometric object.
:::

::: {#method.vector .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#22-24){.src
.rightside}[§](#method.vector){.anchor}

#### fn [vector](../geometrics/trait.GeometricObject.html#method.vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}\> {#fn-vectorself---vecpoint .code-header}
:::

::: docblock
Returns a `Vec` containing references to the points that constitute the
geometric object. This method simply converts the `BTreeSet` from
`get_points` into a `Vec`.
:::

::: {#method.to_vector .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#50-52){.src
.rightside}[§](#method.to_vector){.anchor}

#### fn [to_vector](../geometrics/trait.GeometricObject.html#method.to_vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}\> {#fn-to_vectorself---vecpoint .code-header}
:::

::: docblock
Returns the points of the geometric object as a `Vec` of references.
Equivalent to calling the `vector()` method.
:::

:::: {#method.calculate_range .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#57-64){.src
.rightside}[§](#method.calculate_range){.anchor}

#### fn [calculate_range](../geometrics/trait.GeometricObject.html#method.calculate_range){.fn}\<I\>(iter: I) -\> (Decimal, Decimal) {#fn-calculate_rangeiiter-i---decimal-decimal .code-header}

::: where
where I:
[Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item
= Decimal\>,
:::
::::

::: docblock
Calculates the minimum and maximum decimal values from an iterator of
decimals. [Read
more](../geometrics/trait.GeometricObject.html#method.calculate_range)
:::
::::::::::::::::::::::::

::: {#impl-GeometricTransformations%3CPoint3D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1374-1600){.src
.rightside}[§](#impl-GeometricTransformations%3CPoint3D%3E-for-Surface){.anchor}

### impl [GeometricTransformations](../geometrics/trait.GeometricTransformations.html "trait optionstratlib::geometrics::GeometricTransformations"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-geometrictransformationspoint3d-for-surface .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1375){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror-2 .code-header}
:::

::: docblock
The error type that can be returned by geometric operations.
:::

::: {#method.translate .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1377-1398){.src
.rightside}[§](#method.translate){.anchor}

#### fn [translate](../geometrics/trait.GeometricTransformations.html#tymethod.translate){.fn}(&self, deltas: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-translateself-deltas-vecdecimal---resultself-selferror .code-header}
:::

::: docblock
Translates the geometric object by specified amounts along each
dimension. [Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.translate)
:::

::: {#method.scale .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1400-1421){.src
.rightside}[§](#method.scale){.anchor}

#### fn [scale](../geometrics/trait.GeometricTransformations.html#tymethod.scale){.fn}(&self, factors: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-scaleself-factors-vecdecimal---resultself-selferror .code-header}
:::

::: docblock
Scales the geometric object by specified factors along each dimension.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.scale)
:::

::: {#method.intersect_with .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1423-1439){.src
.rightside}[§](#method.intersect_with){.anchor}

#### fn [intersect_with](../geometrics/trait.GeometricTransformations.html#tymethod.intersect_with){.fn}(&self, other: &Self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\>, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-intersect_withself-other-self---resultvecpoint3d-selferror .code-header}
:::

::: docblock
Finds all intersection points between this geometric object and another.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.intersect_with)
:::

::: {#method.derivative_at .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1441-1545){.src
.rightside}[§](#method.derivative_at){.anchor}

#### fn [derivative_at](../geometrics/trait.GeometricTransformations.html#tymethod.derivative_at){.fn}(&self, point: &[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\>, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-derivative_atself-point-point3d---resultvecdecimal-selferror .code-header}
:::

::: docblock
Calculates the derivative at a specific point on the geometric object.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.derivative_at)
:::

::: {#method.extrema .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1547-1570){.src
.rightside}[§](#method.extrema){.anchor}

#### fn [extrema](../geometrics/trait.GeometricTransformations.html#tymethod.extrema){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}), Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-extremaself---resultpoint3d-point3d-selferror .code-header}
:::

::: docblock
Finds the extrema (minimum and maximum points) of the geometric object.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.extrema)
:::

::: {#method.measure_under .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1572-1599){.src
.rightside}[§](#method.measure_under){.anchor}

#### fn [measure_under](../geometrics/trait.GeometricTransformations.html#tymethod.measure_under){.fn}(&self, base_value: &Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-measure_underself-base_value-decimal---resultdecimal-selferror .code-header}
:::

::: docblock
Calculates the area or volume under the geometric object relative to a
base value. [Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.measure_under)
:::
:::::::::::::::::

::: {#impl-HasX-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#211-215){.src
.rightside}[§](#impl-HasX-for-Point3D){.anchor}

### impl [HasX](../geometrics/trait.HasX.html "trait optionstratlib::geometrics::HasX"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-hasx-for-point3d .code-header}
:::

::::: impl-items
::: {#method.get_x .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#212-214){.src
.rightside}[§](#method.get_x){.anchor}

#### fn [get_x](../geometrics/trait.HasX.html#tymethod.get_x){.fn}(&self) -\> Decimal {#fn-get_xself---decimal .code-header}
:::

::: docblock
Returns the X-coordinate value of this object. [Read
more](../geometrics/trait.HasX.html#tymethod.get_x)
:::
:::::

:::: {#impl-Interpolate%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#585){.src
.rightside}[§](#impl-Interpolate%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [Interpolate](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-interpolatepoint3d-point2d-for-surface .code-header}

::: docblock
Implementation of the `Interpolate` trait for the `Surface` type,
enabling interpolation from 3D surface points to 2D points.
:::
::::

:::: docblock
#### [§](#overview-2){.doc-anchor}Overview

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

::::::: impl-items
::: {#method.interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#80-91){.src
.rightside}[§](#method.interpolate){.anchor}

#### fn [interpolate](../geometrics/trait.Interpolate.html#method.interpolate){.fn}( &self, x: Input, interpolation_type: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-interpolate-self-x-input-interpolation_type-interpolationtype---resultpoint-interpolationerror .code-header}
:::

::: docblock
Interpolates a value at the given x-coordinate using the specified
interpolation method. [Read
more](../geometrics/trait.Interpolate.html#method.interpolate)
:::

::: {#method.find_bracket_points .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#110-132){.src
.rightside}[§](#method.find_bracket_points){.anchor}

#### fn [find_bracket_points](../geometrics/trait.Interpolate.html#method.find_bracket_points){.fn}( &self, x: Input, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}), [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-find_bracket_points-self-x-input---resultusize-usize-interpolationerror .code-header}
:::

::: docblock
Finds the indices of points that bracket the given x-coordinate. [Read
more](../geometrics/trait.Interpolate.html#method.find_bracket_points)
:::
:::::::

:::: {#impl-LinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#608-688){.src
.rightside}[§](#impl-LinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [LinearInterpolation](../geometrics/trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-linearinterpolationpoint3d-point2d-for-surface .code-header}

::: docblock
#### [§](#linear-interpolation-for-surfaces){.doc-anchor}Linear Interpolation for Surfaces

Implementation of the `LinearInterpolation` trait for `Surface`
structures, enabling interpolation from 2D points to 3D points using
barycentric coordinates.
:::
::::

::: docblock
##### [§](#overview-3){.doc-anchor}Overview

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

::::: impl-items
::: {#method.linear_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#623-687){.src
.rightside}[§](#method.linear_interpolate){.anchor}

#### fn [linear_interpolate](../geometrics/trait.LinearInterpolation.html#tymethod.linear_interpolate){.fn}(&self, xy: [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-linear_interpolateself-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
###### [§](#parameters-3){.doc-anchor}Parameters

- `xy` - A `Point2D` representing the x and y coordinates where
  interpolation is needed

###### [§](#returns-4){.doc-anchor}Returns

- `Result<Point3D, InterpolationError>` - The interpolated 3D point if
  successful, or an appropriate error if interpolation cannot be
  performed

###### [§](#errors-1){.doc-anchor}Errors

Returns `InterpolationError::Linear` in the following cases:

- When the surface contains only coincident points forming a degenerate
  triangle
- When the query point is outside the surface's x-y range
:::
:::::

:::: {#impl-MergeAxisInterpolate%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1324-1372){.src
.rightside}[§](#impl-MergeAxisInterpolate%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [MergeAxisInterpolate](../geometrics/trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-mergeaxisinterpolatepoint3d-point2d-for-surface .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::::: impl-items
::: {#method.merge_axis_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1328-1371){.src
.rightside}[§](#method.merge_axis_interpolate){.anchor}

#### fn [merge_axis_interpolate](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate){.fn}( &self, other: &Self, interpolation: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Self, Self), Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-merge_axis_interpolate-self-other-self-interpolation-interpolationtype---resultself-self-selferror .code-header}
:::

::: docblock
Interpolates both structures to align them on a common set of index
values. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate)
:::

::: {#method.merge_axis_index .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#144-147){.src
.rightside}[§](#method.merge_axis_index){.anchor}

#### fn [merge_axis_index](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index){.fn}\<\'a\>(&\'a self, other: &\'a Self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_axis_indexaa-self-other-a-self---vecinput .code-header}
:::

::: docblock
Merges the index values from two structures into a single ordered
vector. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index)
:::
:::::::

::: {#impl-Ord-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#75-85){.src
.rightside}[§](#impl-Ord-for-Point3D){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-ord-for-point3d .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#76-84){.src
.rightside}[§](#method.cmp){.anchor}

#### fn [cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#tymethod.cmp){.fn}(&self, other: &Self) -\> [Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-cmpself-other-self---ordering .code-header}
:::

::: docblock
This method returns an
[`Ordering`](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering")
between `self` and `other`. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#tymethod.cmp)
:::

:::: {#method.max .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#998-1000){.src}]{.rightside}[§](#method.max){.anchor}

#### fn [max](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.max){.fn}(self, other: Self) -\> Self {#fn-maxself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the maximum of two values. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.max)
:::

:::: {#method.min .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1037-1039){.src}]{.rightside}[§](#method.min){.anchor}

#### fn [min](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.min){.fn}(self, other: Self) -\> Self {#fn-minself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the minimum of two values. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.min)
:::

:::: {#method.clamp .section .method .trait-impl}
[[1.50.0]{.since title="Stable since Rust version 1.50.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1063-1065){.src}]{.rightside}[§](#method.clamp){.anchor}

#### fn [clamp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.clamp){.fn}(self, min: Self, max: Self) -\> Self {#fn-clampself-min-self-max-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Restrict a value to a certain interval. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.clamp)
:::
::::::::::::::

::: {#impl-PartialEq-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#61-65){.src
.rightside}[§](#impl-PartialEq-for-Point3D){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-partialeq-for-point3d .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#62-64){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-self---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialOrd-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#69-73){.src
.rightside}[§](#impl-PartialOrd-for-Point3D){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-partialord-for-point3d .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#70-72){.src
.rightside}[§](#method.partial_cmp){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-self---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-Serialize-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#impl-Serialize-for-Point3D){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-serialize-for-point3d .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-SplineInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#844-934){.src
.rightside}[§](#impl-SplineInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [SplineInterpolation](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-splineinterpolationpoint3d-point2d-for-surface .code-header}
:::

::::: impl-items
::: {#method.spline_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#845-933){.src
.rightside}[§](#method.spline_interpolate){.anchor}

#### fn [spline_interpolate](../geometrics/trait.SplineInterpolation.html#tymethod.spline_interpolate){.fn}(&self, xy: [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-spline_interpolateself-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
Interpolates a y-value for the provided x-coordinate using spline
interpolation. [Read
more](../geometrics/trait.SplineInterpolation.html#tymethod.spline_interpolate)
:::
:::::

::: {#impl-Copy-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#45){.src
.rightside}[§](#impl-Copy-for-Point3D){.anchor}

### impl [Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-copy-for-point3d .code-header}
:::

::: {#impl-Eq-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#67){.src
.rightside}[§](#impl-Eq-for-Point3D){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-eq-for-point3d .code-header}
:::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Point3D .section .impl}
[§](#impl-Freeze-for-Point3D){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-freeze-for-point3d .code-header}
:::

::: {#impl-RefUnwindSafe-for-Point3D .section .impl}
[§](#impl-RefUnwindSafe-for-Point3D){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-refunwindsafe-for-point3d .code-header}
:::

::: {#impl-Send-for-Point3D .section .impl}
[§](#impl-Send-for-Point3D){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-send-for-point3d .code-header}
:::

::: {#impl-Sync-for-Point3D .section .impl}
[§](#impl-Sync-for-Point3D){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-sync-for-point3d .code-header}
:::

::: {#impl-Unpin-for-Point3D .section .impl}
[§](#impl-Unpin-for-Point3D){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-unpin-for-point3d .code-header}
:::

::: {#impl-UnwindSafe-for-Point3D .section .impl}
[§](#impl-UnwindSafe-for-Point3D){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-unwindsafe-for-point3d .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.86.0/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#209){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#211){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#217){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#218){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#273){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#275){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dst: [\*mut](https://doc.rust-lang.org/1.86.0/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.86.0/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dst-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dst`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

:::: {#impl-Comparable%3CK%3E-for-Q .section .impl}
[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> Comparable\<K\> for Q {#implq-k-comparablek-for-q .code-header}

::: where
where Q:
[Ord](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.compare .section .method .trait-impl}
[§](#method.compare){.anchor}

#### fn [compare]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> Equivalent\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent .section .method .trait-impl}
[§](#method.equivalent){.anchor}

#### fn [equivalent]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. Read more
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> Equivalent\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent-1 .section .method .trait-impl}
[§](#method.equivalent-1){.anchor}

#### fn [equivalent]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
:::

::: docblock
Compare self to `key` and return `true` if they are equal.
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> Instrument for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[§](#method.instrument){.anchor}

#### fn [instrument]{.fn}(self, span: Span) -\> Instrumented\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided \[`Span`\], returning an
`Instrumented` wrapper. Read more
:::

::: {#method.in_current_span .section .method .trait-impl}
[§](#method.in_current_span){.anchor}

#### fn [in_current_span]{.fn}(self) -\> Instrumented\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the [current](super::Span::current())
[`Span`](crate::Span), returning an `Instrumented` wrapper. Read more
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#750-752){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#760){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](https://docs.rs/either/1/either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> Pointable for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN]{.constant}: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[§](#associatedtype.Init){.anchor}

#### type [Init]{.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[§](#method.init){.anchor}

#### unsafe fn [init]{.fn}(init: \<T as Pointable\>::Init) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. Read more
:::

::: {#method.deref .section .method .trait-impl}
[§](#method.deref){.anchor}

#### unsafe fn [deref]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. Read more
:::

::: {#method.deref_mut .section .method .trait-impl}
[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. Read more
:::

::: {#method.drop .section .method .trait-impl}
[§](#method.drop){.anchor}

#### unsafe fn [drop]{.fn}(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. Read more
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> SupersetOf\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS: SubsetOf\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[§](#method.to_subset){.anchor}

#### fn [to_subset]{.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. Read more
:::

::: {#method.is_in_subset .section .method .trait-impl}
[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset]{.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked]{.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[§](#method.from_subset){.anchor}

#### fn [from_subset]{.fn}(element: [&SS](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#82-84){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#86){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#87){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#91){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2758){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2760){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#807-809){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-4 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#811){.src
.rightside}[§](#associatedtype.Error-4){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.86.0/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#814){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#792-794){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-3 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#796){.src
.rightside}[§](#associatedtype.Error-3){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#799){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> VZip\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V: MultiLane\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[§](#method.vzip){.anchor}

#### fn [vzip]{.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> WithSubscriber for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber]{.fn}\<S\>(self, subscriber: S) -\> WithDispatch\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Dispatch\>,
:::
::::

::: docblock
Attaches the provided [`Subscriber`](super::Subscriber) to this type,
returning a \[`WithDispatch`\] wrapper. Read more
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber]{.fn}(self) -\> WithDispatch\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](crate::dispatcher#setting-the-default-subscriber)
[`Subscriber`](super::Subscriber) to this type, returning a
\[`WithDispatch`\] wrapper. Read more
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](https://docs.rs/serde/1.0.219/src/serde/de/mod.rs.html#614){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](https://docs.rs/serde/1.0.219/serde/de/trait.DeserializeOwned.html "trait serde::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](https://docs.rs/nalgebra/0.25.0/src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](https://docs.rs/nalgebra/0.25.0/nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
