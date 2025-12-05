:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[curves](index.html)
:::

# Struct [Point2D]{.struct} Copy item path

[[Source](../../src/optionstratlib/curves/types.rs.html#52-60){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Point2D {
    pub x: Decimal,
    pub y: Decimal,
}
```

Expand description

:::: docblock
Represents a point in two-dimensional space with `x` and `y`
coordinates.

## [§](#overview){.doc-anchor}Overview

The `Point2D` struct is used to define a point in a 2D Cartesian
coordinate system. Both coordinates are stored as `Decimal` values to
provide high precision, making it suitable for applications requiring
accurate numerical calculations.

## [§](#usage){.doc-anchor}Usage

This structure serves as a fundamental data type in various geometric
operations:

- Defining positions in curve plotting and interpolation
- Representing intersections between curves
- Serving as input/output for mathematical transformations
- Supporting coordinate-based algorithms in the library

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::curves::Point2D;

// Create a point at coordinates (3.5, -2.25)
let point = Point2D {
    x: dec!(3.5),
    y: dec!(-2.25)
};
```
:::

## [§](#derivable-traits){.doc-anchor}Derivable Traits

- `Debug`: Enables formatted debugging output
- `Clone` and `Copy`: Allow efficient duplication of point values
- `Serialize` and `Deserialize`: Support for serialization frameworks

This struct is primarily used in conjunction with the `Curve` and
`Curvable` types to represent mathematical curves and perform geometric
operations.
::::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.x){.anchor
.field}`x: `[`Decimal`](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}]{#structfield.x
.structfield .section-header}

::: docblock
The x-coordinate in the Cartesian plane, represented as a high-precision
`Decimal` value to ensure accuracy in mathematical operations.
:::

[[§](#structfield.y){.anchor
.field}`y: `[`Decimal`](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}]{#structfield.y
.structfield .section-header}

::: docblock
The y-coordinate in the Cartesian plane, represented as a high-precision
`Decimal` value to ensure accuracy in mathematical operations.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::: {#implementations-list}
::: {#impl-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#98-212){.src
.rightside}[§](#impl-Point2D){.anchor}

### impl [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-point2d .code-header}
:::

::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../src/optionstratlib/curves/types.rs.html#111-116){.src
.rightside}

#### pub fn [new](#method.new){.fn}\<T: [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, U: [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>\>(x: T, y: U) -\> Self {#pub-fn-newt-intodecimal-u-intodecimalx-t-y-u---self .code-header}
:::

::: docblock
Creates a new instance of `Point2D` using the specified `x` and `y`
coordinates.

##### [§](#parameters){.doc-anchor}Parameters

- `x`: The x-coordinate of the point, which implements `Into<Decimal>`.
- `y`: The y-coordinate of the point, which implements `Into<Decimal>`.

##### [§](#returns){.doc-anchor}Returns

A `Point2D` instance with the provided `x` and `y` coordinates,
converted into `Decimal`.

##### [§](#usage-1){.doc-anchor}Usage

This function is used when creating a `Point2D` object from any type
that can be converted into `Decimal`, allowing flexibility in input
types (e.g., `f64`, `i32`, etc.).
:::

::: {#method.to_tuple .section .method}
[Source](../../src/optionstratlib/curves/types.rs.html#133-149){.src
.rightside}

#### pub fn [to_tuple](#method.to_tuple){.fn}\<T: [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> + \'static, U: [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> + \'static\>( &self, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[(T, U)](https://doc.rust-lang.org/1.91.1/std/primitive.tuple.html){.primitive}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-to_tuplet-fromdecimal-static-u-fromdecimal-static-self---resultt-u-curveerror .code-header}
:::

::: docblock
Converts the `Point2D` instance into a tuple `(T, U)`.

##### [§](#parameters-1){.doc-anchor}Parameters

- `T`: The type for the x-coordinate, which must implement
  `From<Decimal>` and have a 'static lifetime.
- `U`: The type for the y-coordinate, which must implement
  `From<Decimal>` and have a 'static lifetime.

##### [§](#returns-1){.doc-anchor}Returns

- `Ok`: A tuple `(T, U)` containing the converted `x` and `y` values.
- `Err`: A `CurvesError` if conversion constraints are violated:
  - `x` must be positive if `T` is the `Positive` type.
  - `y` must be positive if `U` is the `Positive` type.

##### [§](#errors){.doc-anchor}Errors

This function returns an error if the positivity constraints are
violated or if conversions fail due to invalid type requirements.
:::

::: {#method.from_tuple .section .method}
[Source](../../src/optionstratlib/curves/types.rs.html#163-165){.src
.rightside}

#### pub fn [from_tuple](#method.from_tuple){.fn}\<T: [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, U: [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>\>( x: T, y: U, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-from_tuplet-intodecimal-u-intodecimal-x-t-y-u---resultself-curveerror .code-header}
:::

::: docblock
Creates a new `Point2D` instance from a tuple containing `x` and `y`
values.

##### [§](#parameters-2){.doc-anchor}Parameters

- `x`: The x-coordinate, which implements `Into<Decimal>`.
- `y`: The y-coordinate, which implements `Into<Decimal>`.

##### [§](#returns-2){.doc-anchor}Returns

- `Ok`: A new `Point2D` instance with the given `x` and `y` coordinates.
- `Err`: A `CurvesError` if coordinate creation fails.

##### [§](#usage-2){.doc-anchor}Usage

This function allows constructing a `Point2D` directly from a tuple
representation.
:::

::: {#method.to_f64_tuple .section .method}
[Source](../../src/optionstratlib/curves/types.rs.html#176-186){.src
.rightside}

#### pub fn [to_f64_tuple](#method.to_f64_tuple){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}), [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-to_f64_tupleself---resultf64-f64-curveerror .code-header}
:::

::: docblock
Converts the `Point2D` instance into a tuple of `(f64, f64)`.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok`: A tuple `(f64, f64)` containing the `x` and `y` values.
- `Err`: A `CurvesError` if either `x` or `y` cannot be converted from
  `Decimal` to `f64` (e.g., out-of-range value).

##### [§](#errors-1){.doc-anchor}Errors

Returns a `CurvesError::Point2DError` with a reason explaining the
failure.
:::

::: {#method.from_f64_tuple .section .method}
[Source](../../src/optionstratlib/curves/types.rs.html#202-211){.src
.rightside}

#### pub fn [from_f64_tuple](#method.from_f64_tuple){.fn}(x: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}, y: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-from_f64_tuplex-f64-y-f64---resultself-curveerror .code-header}
:::

::: docblock
Creates a new `Point2D` instance from a tuple of `(f64, f64)` values.

##### [§](#parameters-3){.doc-anchor}Parameters

- `x`: The x-coordinate of the point as a `f64`.
- `y`: The y-coordinate of the point as a `f64`.

##### [§](#returns-4){.doc-anchor}Returns

- `Ok`: A new `Point2D` instance if both `x` and `y` values can be
  successfully converted from `f64` to `Decimal`.
- `Err`: A `CurvesError` if the conversion fails (e.g., invalid
  precision).

##### [§](#errors-2){.doc-anchor}Errors

Returns a `CurvesError::Point2DError` with a reason if either `x` or `y`
could not be converted from `f64`.
:::
:::::::::::::
:::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-AxisOperations%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1469-1508){.src
.rightside}[§](#impl-AxisOperations%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [AxisOperations](../geometrics/trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-axisoperationspoint2d-decimal-for-curve .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1470){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](../geometrics/trait.AxisOperations.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::

::: docblock
The type of error that can occur during point operations
:::

::: {#method.contains_point .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1472-1474){.src
.rightside}[§](#method.contains_point){.anchor}

#### fn [contains_point](../geometrics/trait.AxisOperations.html#tymethod.contains_point){.fn}(&self, x: &[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-contains_pointself-x-decimal---bool .code-header}
:::

::: docblock
Checks if a coordinate value exists in the structure. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.contains_point)
:::

::: {#method.get_index_values .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1476-1478){.src
.rightside}[§](#method.get_index_values){.anchor}

#### fn [get_index_values](../geometrics/trait.AxisOperations.html#tymethod.get_index_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_index_valuesself---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to all index values in the structure.
[Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_index_values)
:::

::: {#method.get_values .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1480-1486){.src
.rightside}[§](#method.get_values){.anchor}

#### fn [get_values](../geometrics/trait.AxisOperations.html#tymethod.get_values){.fn}(&self, x: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_valuesself-x-decimal---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to dependent values for a given
coordinate. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_values)
:::

::: {#method.get_closest_point .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1488-1499){.src
.rightside}[§](#method.get_closest_point){.anchor}

#### fn [get_closest_point](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point){.fn}(&self, x: &[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-get_closest_pointself-x-decimal---resultpoint2d-selferror .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point)
:::

::: {#method.get_point .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1501-1507){.src
.rightside}[§](#method.get_point){.anchor}

#### fn [get_point](../geometrics/trait.AxisOperations.html#tymethod.get_point){.fn}(&self, x: &[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#fn-get_pointself-x-decimal---optionpoint2d .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_point)
:::

::: {#method.merge_indexes .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#85-115){.src
.rightside}[§](#method.merge_indexes){.anchor}

#### fn [merge_indexes](../geometrics/trait.AxisOperations.html#method.merge_indexes){.fn}(&self, axis: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\>) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_indexesself-axis-vecinput---vecinput .code-header}
:::

::: docblock
Merges the index values from the current structure with an additional
set of indices. This combines self.get_index_values() with the provided
axis vector to create a single vector of unique indices. [Read
more](../geometrics/trait.AxisOperations.html#method.merge_indexes)
:::
:::::::::::::::::

::: {#impl-AxisOperations%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1300-1335){.src
.rightside}[§](#impl-AxisOperations%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [AxisOperations](../geometrics/trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-axisoperationspoint3d-point2d-for-surface .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-4 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1301){.src
.rightside}[§](#associatedtype.Error-4){.anchor}

#### type [Error](../geometrics/trait.AxisOperations.html#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::

::: docblock
The type of error that can occur during point operations
:::

::: {#method.contains_point-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1303-1305){.src
.rightside}[§](#method.contains_point-1){.anchor}

#### fn [contains_point](../geometrics/trait.AxisOperations.html#tymethod.contains_point){.fn}(&self, x: &[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-contains_pointself-x-point2d---bool .code-header}
:::

::: docblock
Checks if a coordinate value exists in the structure. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.contains_point)
:::

::: {#method.get_index_values-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1307-1309){.src
.rightside}[§](#method.get_index_values-1){.anchor}

#### fn [get_index_values](../geometrics/trait.AxisOperations.html#tymethod.get_index_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#fn-get_index_valuesself---vecpoint2d .code-header}
:::

::: docblock
Returns a vector of references to all index values in the structure.
[Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_index_values)
:::

::: {#method.get_values-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1311-1317){.src
.rightside}[§](#method.get_values-1){.anchor}

#### fn [get_values](../geometrics/trait.AxisOperations.html#tymethod.get_values){.fn}(&self, x: [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_valuesself-x-point2d---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to dependent values for a given
coordinate. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_values)
:::

::: {#method.get_closest_point-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1319-1330){.src
.rightside}[§](#method.get_closest_point-1){.anchor}

#### fn [get_closest_point](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point){.fn}(&self, x: &[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-get_closest_pointself-x-point2d---resultpoint3d-selferror .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point)
:::

::: {#method.get_point-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1332-1334){.src
.rightside}[§](#method.get_point-1){.anchor}

#### fn [get_point](../geometrics/trait.AxisOperations.html#tymethod.get_point){.fn}(&self, x: &[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> {#fn-get_pointself-x-point2d---optionpoint3d .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_point)
:::

::: {#method.merge_indexes-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#85-115){.src
.rightside}[§](#method.merge_indexes-1){.anchor}

#### fn [merge_indexes](../geometrics/trait.AxisOperations.html#method.merge_indexes){.fn}(&self, axis: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\>) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_indexesself-axis-vecinput---vecinput-1 .code-header}
:::

::: docblock
Merges the index values from the current structure with an additional
set of indices. This combines self.get_index_values() with the provided
axis vector to create a single vector of unique indices. [Read
more](../geometrics/trait.AxisOperations.html#method.merge_indexes)
:::
:::::::::::::::::

:::: {#impl-BiLinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#460-536){.src
.rightside}[§](#impl-BiLinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [BiLinearInterpolation](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-bilinearinterpolationpoint2d-decimal-for-curve .code-header}

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

#### [§](#parameters-6){.doc-anchor}Parameters {#parameters-6}

- **`x`**: The x-coordinate value for which the interpolation will be
  performed. Must fall within the range of the x-coordinates of the
  curve's points.

#### [§](#returns-7){.doc-anchor}Returns {#returns-7}

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

#### [§](#errors-5){.doc-anchor}Errors {#errors-5}

- **Insufficient Points**: If the curve contains fewer than 4 points, a
  `CurvesError` with a relevant message is returned.
- **Out-of-Range x**: If the x-coordinate cannot be bracketed by points
  in the curve, a `CurvesError` is returned with an appropriate message.

#### [§](#related-traits){.doc-anchor}Related Traits

- [`BiLinearInterpolation`](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"):
  The trait defining this method.
- [`Interpolate`](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"):
  Ensures compatibility of the curve with multiple interpolation
  methods.

#### [§](#see-also-2){.doc-anchor}See Also {#see-also-2}

- [`Curve`](struct.Curve.html "struct optionstratlib::curves::Curve"):
  The overarching structure that represents the curve.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The data type used to represent individual points on the curve.
- [`find_bracket_points`](../geometrics/trait.Interpolate.html#method.find_bracket_points "method optionstratlib::geometrics::Interpolate::find_bracket_points"):
  A helper method used to locate the two points that bracket the given
  x-coordinate.
:::

::::: impl-items
::: {#method.bilinear_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#500-535){.src
.rightside}[§](#method.bilinear_interpolate){.anchor}

#### fn [bilinear_interpolate](../geometrics/trait.BiLinearInterpolation.html#tymethod.bilinear_interpolate){.fn}( &self, x: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-bilinear_interpolate-self-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

::: docblock
Performs bilinear interpolation to find the value of the curve at a
given `x` coordinate.

##### [§](#parameters-5){.doc-anchor}Parameters

- `x`: The x-coordinate at which the interpolation is to be performed.
  This should be a `Decimal` value within the range of the curve's known
  points.

##### [§](#returns-6){.doc-anchor}Returns

- On success, returns a `Point2D` instance representing the interpolated
  point at the given `x` value.
- On failure, returns a `CurvesError`:
  - `CurvesError::InterpolationError`: If there are fewer than four
    points available for interpolation or if the required conditions for
    interpolation are not met.

##### [§](#function-description){.doc-anchor}Function Description

- The function retrieves the set of points defining the curve using
  `self.get_points()`.
- If fewer than four points exist, the function immediately fails with
  an `InterpolationError`.
- If the exact `x` value is found in the point set, its corresponding
  `Point2D` is returned directly.
- Otherwise, it determines the bracketing points (two pairs of points
  forming a square grid) necessary for bilinear interpolation using
  `self.find_bracket_points()`.
- From the bracketing points, it computes:
  - `dx`: A normalized value representing the relative position of `x`
    between its bracketing x-coordinates in the `[`0,1`]` interval.
  - `bottom`: The interpolated y-value along the bottom edge of the
    grid.
  - `top`: The interpolated y-value along the top edge of the grid.
  - `y`: The final interpolated value along the y-dimension from
    `bottom` to `top`.
- Returns the final interpolated point as `Point2D(x, y)`.

##### [§](#errors-4){.doc-anchor}Errors

- Returns an error if the curve has fewer than four points, as bilinear
  interpolation requires at least four.
- Returns an error from `self.find_bracket_points()` if `x` cannot be
  bracketed.

##### [§](#notes){.doc-anchor}Notes

- The input `x` should be within the bounds of the curve for
  interpolation to succeed, as specified by the bracketing function.
- This function assumes that the points provided by `get_points` are
  sorted by ascending x-coordinate.

##### [§](#example-use-case){.doc-anchor}Example Use Case

This method is useful for calculating intermediate values on a 2D grid
when exact measurements are unavailable. Bilinear interpolation is
particularly applicable for approximating smoother values in a tabular
dataset or a regularly sampled grid.
:::
:::::

::: {#impl-BiLinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#703-782){.src
.rightside}[§](#impl-BiLinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [BiLinearInterpolation](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-bilinearinterpolationpoint3d-point2d-for-surface .code-header}
:::

::::: impl-items
::: {#method.bilinear_interpolate-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#704-781){.src
.rightside}[§](#method.bilinear_interpolate-1){.anchor}

#### fn [bilinear_interpolate](../geometrics/trait.BiLinearInterpolation.html#tymethod.bilinear_interpolate){.fn}( &self, xy: [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-bilinear_interpolate-self-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
Performs bilinear interpolation to compute a value for the given input.
[Read
more](../geometrics/trait.BiLinearInterpolation.html#tymethod.bilinear_interpolate)
:::
:::::

::: {#impl-Clone-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-Clone-for-Point2D){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-clone-for-point2d .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#fn-cloneself---point2d .code-header}
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

::: {#impl-ComposeSchema-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-ComposeSchema-for-Point2D){.anchor}

### impl ComposeSchema for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-composeschema-for-point2d .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

:::: {#impl-CubicInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#597-713){.src
.rightside}[§](#impl-CubicInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [CubicInterpolation](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-cubicinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `CubicInterpolation` trait for the `Curve` struct,
providing an algorithm for cubic interpolation utilizing a Catmull-Rom
spline.
:::
::::

::: docblock
#### [§](#method-cubic_interpolate){.doc-anchor}Method: `cubic_interpolate`

##### [§](#parameters-8){.doc-anchor}Parameters {#parameters-8}

- **`x`**: The x-value at which the interpolation is performed. This
  value must be within the range of x-values in the curve's defined
  points, and it is passed as a `Decimal` to allow for high-precision
  computation.

##### [§](#returns-9){.doc-anchor}Returns {#returns-9}

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

##### [§](#errors-6){.doc-anchor}Errors {#errors-6}

- Returns an error of type `CurvesError::InterpolationError` if any
  issues are encountered, such as insufficient points or the inability
  to locate bracket points.

##### [§](#example-1){.doc-anchor}Example {#example-1}

This method is part of the `Curve` struct, which defines a set of points
and supports interpolation. It is often used in applications requiring
smooth manifolds or animations.

##### [§](#notes-1){.doc-anchor}Notes

- The computed y-value ensures smooth transitions and continuity between
  interpolated segments.
- Catmull-Rom splines are particularly effective for creating visually
  smooth transitions, making this method suitable for curves,
  animations, and numerical analysis.

#### [§](#see-also-3){.doc-anchor}See Also {#see-also-3}

- [`CubicInterpolation`](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"):
  The trait defining this method.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents the points used for interpolation.
- [`find_bracket_points`](../geometrics/trait.Interpolate.html#method.find_bracket_points "method optionstratlib::geometrics::Interpolate::find_bracket_points"):
  Determines the bracketing points required for interpolation.
:::

::::: impl-items
::: {#method.cubic_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#665-712){.src
.rightside}[§](#method.cubic_interpolate){.anchor}

#### fn [cubic_interpolate](../geometrics/trait.CubicInterpolation.html#tymethod.cubic_interpolate){.fn}(&self, x: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-cubic_interpolateself-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

::: docblock
Performs cubic interpolation on a set of points to estimate the
y-coordinate for a given x value using a Catmull-Rom spline.

##### [§](#parameters-7){.doc-anchor}Parameters

- `x`: The x-coordinate for which the interpolation is performed. This
  value should lie within the range of the points on the curve.

##### [§](#returns-8){.doc-anchor}Returns

- `Ok(Point2D)`: A `Point2D` instance representing the interpolated
  position `(x, y)`, where `y` is estimated using cubic interpolation.
- `Err(CurvesError)`: An error indicating issues with the interpolation
  process, such as insufficient points or an out-of-range x value.

##### [§](#requirements){.doc-anchor}Requirements

- The number of points in the curve must be at least 4, as cubic
  interpolation requires four points for accurate calculations.
- The specified `x` value should be inside the range defined by the
  curve's points.
- If the specified x matches an existing point on the curve, the
  interpolated result directly returns that exact point.

##### [§](#functionality){.doc-anchor}Functionality

This method performs cubic interpolation using the general properties of
the Catmull-Rom spline, which is well-suited for smooth curve fitting.
It operates as follows:

1.  **Exact Point Check**: If the x value matches an existing point, the
    method returns that point without further processing.

2.  **Bracketing Points Selection**:

    - Searches for two points that bracket the given x value (using
      `find_bracket_points` from the `Interpolate` trait). The method
      ensures that there are always enough points before and after the
      target x value to perform cubic interpolation.

3.  **Point Selection for Interpolation**:

    - Depending on the position of the target x value, four points
      (`p0, p1, p2, p3`) are selected:
      - When `x` is near the start of the points, select the first four.
      - When `x` is near the end, select the last four.
      - Otherwise, select the two points just before and after the x
        value and include an additional adjacent point on either side.

4.  **Parameter Calculation**:

    - The `t` parameter is derived, representing the normalized position
      of x between `p1` and `p2`.

5.  **Cubic Interpolation**:

    - The interpolated y-coordinate is computed using the Catmull-Rom
      spline formula, leveraging the `t`-value and the y-coordinates of
      the four selected points.

##### [§](#error-handling){.doc-anchor}Error Handling

This method returns an error in the following circumstances:

- If fewer than 4 points are available, it returns a
  `CurvesError::InterpolationError` with a corresponding message.
- If the bracketing points cannot be identified (e.g., when `x` is
  outside the range of points), the appropriate interpolation error is
  propagated.

##### [§](#example){.doc-anchor}Example {#example}

- Interpolating smoothly along a curve defined by a set of points,
  avoiding sharp transitions between segments.

- Provides a high degree of precision due to the use of the `Decimal`
  type for `x` and `y` calculations.
:::
:::::

::: {#impl-CubicInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#784-855){.src
.rightside}[§](#impl-CubicInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [CubicInterpolation](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-cubicinterpolationpoint3d-point2d-for-surface .code-header}
:::

::::: impl-items
::: {#method.cubic_interpolate-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#785-854){.src
.rightside}[§](#method.cubic_interpolate-1){.anchor}

#### fn [cubic_interpolate](../geometrics/trait.CubicInterpolation.html#tymethod.cubic_interpolate){.fn}(&self, xy: [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-cubic_interpolateself-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
Interpolates a new point on the curve for a given `x` input value using
cubic interpolation. [Read
more](../geometrics/trait.CubicInterpolation.html#tymethod.cubic_interpolate)
:::
:::::

::: {#impl-Debug-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-Debug-for-Point2D){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-debug-for-point2d .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Point2D){.anchor}

### impl\<\'de\> [Deserialize](../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#implde-deserializede-for-point2d .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](../../serde_core/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](../../serde_core/de/trait.Deserializer.html#associatedtype.Error "type serde_core::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](../../serde_core/de/trait.Deserializer.html "trait serde_core::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](../../serde_core/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#62-66){.src
.rightside}[§](#impl-Display-for-Point2D){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-display-for-point2d .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#63-65){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-From%3C%26Point2D%3E-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#214-218){.src
.rightside}[§](#impl-From%3C%26Point2D%3E-for-Point2D){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-frompoint2d-for-point2d .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#215-217){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(point: &[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> Self {#fn-frompoint-point2d---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-GeometricObject%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#148-207){.src
.rightside}[§](#impl-GeometricObject%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [GeometricObject](../geometrics/trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-geometricobjectpoint2d-decimal-for-curve .code-header}
:::

:::::::::::::::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#149){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](../geometrics/trait.GeometricObject.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-1 .code-header}
:::

::: docblock
Type alias for any errors that might occur during the construction of
the geometric object.
:::

::: {#method.get_points .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#151-153){.src
.rightside}[§](#method.get_points){.anchor}

#### fn [get_points](../geometrics/trait.GeometricObject.html#tymethod.get_points){.fn}(&self) -\> [BTreeSet](https://doc.rust-lang.org/1.91.1/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#fn-get_pointsself---btreesetpoint2d .code-header}
:::

::: docblock
Returns a `BTreeSet` containing references to the points that constitute
the geometric object. The `BTreeSet` ensures that the points are ordered
and unique.
:::

:::: {#method.from_vector .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#155-163){.src
.rightside}[§](#method.from_vector){.anchor}

#### fn [from_vector](../geometrics/trait.GeometricObject.html#tymethod.from_vector){.fn}\<T\>(points: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<T\>) -\> Self {#fn-from_vectortpoints-vect---self .code-header}

::: where
where T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::: docblock
Creates a new geometric object from a `Vec` of points. [Read
more](../geometrics/trait.GeometricObject.html#tymethod.from_vector)
:::

:::: {#method.construct .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#165-206){.src
.rightside}[§](#method.construct){.anchor}

#### fn [construct](../geometrics/trait.GeometricObject.html#tymethod.construct){.fn}\<T\>(method: T) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricObject.html#associatedtype.Error "type optionstratlib::geometrics::GeometricObject::Error"){.associatedtype}\> {#fn-constructtmethod-t---resultself-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[ConstructionMethod](../geometrics/enum.ConstructionMethod.html "enum optionstratlib::geometrics::ConstructionMethod"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct},
[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>\>,
:::
::::

::: docblock
Constructs a geometric object using a specific construction method.
[Read more](../geometrics/trait.GeometricObject.html#tymethod.construct)
:::

::: {#method.vector .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#22-24){.src
.rightside}[§](#method.vector){.anchor}

#### fn [vector](../geometrics/trait.GeometricObject.html#method.vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-vectorself---vecpoint .code-header}
:::

::: docblock
Returns a `Vec` containing references to the points that constitute the
geometric object. This method simply converts the `BTreeSet` from
`get_points` into a `Vec`.
:::

::: {#method.to_vector .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#50-52){.src
.rightside}[§](#method.to_vector){.anchor}

#### fn [to_vector](../geometrics/trait.GeometricObject.html#method.to_vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-to_vectorself---vecpoint .code-header}
:::

::: docblock
Returns the points of the geometric object as a `Vec` of references.
Equivalent to calling the `vector()` method.
:::

:::: {#method.calculate_range .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#57-64){.src
.rightside}[§](#method.calculate_range){.anchor}

#### fn [calculate_range](../geometrics/trait.GeometricObject.html#method.calculate_range){.fn}\<I\>(iter: I) -\> ([Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) {#fn-calculate_rangeiiter-i---decimal-decimal .code-header}

::: where
where I:
[Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item
=
[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>,
:::
::::

::: docblock
Calculates the minimum and maximum decimal values from an iterator of
decimals. [Read
more](../geometrics/trait.GeometricObject.html#method.calculate_range)
:::
::::::::::::::::::::

:::: {#impl-GeometricObject%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#335-535){.src
.rightside}[§](#impl-GeometricObject%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [GeometricObject](../geometrics/trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-geometricobjectpoint3d-point2d-for-surface .code-header}

::: docblock
Implementation of the `GeometricObject` trait for the `Surface` struct.
:::
::::

::: docblock
This implementation provides functionality to create and manipulate 3D
surfaces using points in three-dimensional space. It supports
construction from explicit point collections or through parametric
functions.

#### [§](#type-parameters-2){.doc-anchor}Type Parameters {#type-parameters-2}

- Uses `Point3D` as the points that form the surface
- Uses `Point2D` as the parametric input for surface generation

#### [§](#methods){.doc-anchor}Methods

- `get_points()`: Retrieves all points in the surface
- `from_vector()`: Constructs a surface from a vector of points
- `construct()`: Creates a surface using different construction methods

#### [§](#error-handling-1){.doc-anchor}Error Handling

Uses `SurfaceError` for various error conditions, including:

- Empty point collections
- Invalid construction parameters
- Errors during parametric function evaluation
:::

:::::::::::::::::::::::: impl-items
::: {#method.get_points-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#366-368){.src
.rightside}[§](#method.get_points-1){.anchor}

#### fn [get_points](../geometrics/trait.GeometricObject.html#tymethod.get_points){.fn}(&self) -\> [BTreeSet](https://doc.rust-lang.org/1.91.1/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}\<&[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> {#fn-get_pointsself---btreesetpoint3d .code-header}
:::

:::: docblock
Returns a borrowed reference to all points in the surface as an ordered
set

##### [§](#returns-12){.doc-anchor}Returns {#returns-12}

- `BTreeSet<&Point3D>` - A sorted set containing references to all
  points that define the surface, maintaining the natural ordering of
  points

##### [§](#example-2){.doc-anchor}Example

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

:::: {#method.from_vector-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#409-421){.src
.rightside}[§](#method.from_vector-1){.anchor}

#### fn [from_vector](../geometrics/trait.GeometricObject.html#tymethod.from_vector){.fn}\<T\>(points: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<T\>) -\> Self {#fn-from_vectortpoints-vect---self-1 .code-header}

::: where
where T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}\> +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::: docblock
Creates a new Surface from a vector of points that can be converted into
Point3D objects.

This method constructs a Surface by converting each point in the input
vector to a Point3D and collecting them into an ordered set. It also
calculates the x and y coordinate ranges of the points to define the
surface's boundaries.

##### [§](#type-parameters){.doc-anchor}Type Parameters {#type-parameters}

- `T`: A type that can be converted into Point3D via the Into trait and
  can be cloned.

##### [§](#parameters-11){.doc-anchor}Parameters {#parameters-11}

- `points`: A vector of objects that can be converted to Point3D.

##### [§](#returns-13){.doc-anchor}Returns {#returns-13}

A new Surface instance containing the converted points and their
coordinate ranges.

##### [§](#example-3){.doc-anchor}Example

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

:::: {#method.construct-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#481-534){.src
.rightside}[§](#method.construct-1){.anchor}

#### fn [construct](../geometrics/trait.GeometricObject.html#tymethod.construct){.fn}\<T\>(method: T) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricObject.html#associatedtype.Error "type optionstratlib::geometrics::GeometricObject::Error"){.associatedtype}\> {#fn-constructtmethod-t---resultself-selferror-1 .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[ConstructionMethod](../geometrics/enum.ConstructionMethod.html "enum optionstratlib::geometrics::ConstructionMethod"){.enum}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct},
[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\>\>,
:::
::::

::::: docblock
Constructs a Surface from a given construction method.

This function creates a Surface object from either a set of 3D points or
a parametric function.

##### [§](#parameters-12){.doc-anchor}Parameters {#parameters-12}

- `method` - A construction method that can be converted into a
  `ConstructionMethod<Point3D, Point2D>`

##### [§](#type-parameters-1){.doc-anchor}Type Parameters {#type-parameters-1}

- `T` - Type that can be converted into a
  `ConstructionMethod<Point3D, Point2D>`

##### [§](#returns-14){.doc-anchor}Returns {#returns-14}

- `Result<Self, Self::Error>` - Either a successfully constructed
  Surface or an error

##### [§](#errors-9){.doc-anchor}Errors {#errors-9}

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

::: {#associatedtype.Error-3 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#336){.src
.rightside}[§](#associatedtype.Error-3){.anchor}

#### type [Error](../geometrics/trait.GeometricObject.html#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror-1 .code-header}
:::

::: docblock
Type alias for any errors that might occur during the construction of
the geometric object.
:::

::: {#method.vector-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#22-24){.src
.rightside}[§](#method.vector-1){.anchor}

#### fn [vector](../geometrics/trait.GeometricObject.html#method.vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-vectorself---vecpoint-1 .code-header}
:::

::: docblock
Returns a `Vec` containing references to the points that constitute the
geometric object. This method simply converts the `BTreeSet` from
`get_points` into a `Vec`.
:::

::: {#method.to_vector-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#50-52){.src
.rightside}[§](#method.to_vector-1){.anchor}

#### fn [to_vector](../geometrics/trait.GeometricObject.html#method.to_vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-to_vectorself---vecpoint-1 .code-header}
:::

::: docblock
Returns the points of the geometric object as a `Vec` of references.
Equivalent to calling the `vector()` method.
:::

:::: {#method.calculate_range-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#57-64){.src
.rightside}[§](#method.calculate_range-1){.anchor}

#### fn [calculate_range](../geometrics/trait.GeometricObject.html#method.calculate_range){.fn}\<I\>(iter: I) -\> ([Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) {#fn-calculate_rangeiiter-i---decimal-decimal-1 .code-header}

::: where
where I:
[Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item
=
[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>,
:::
::::

::: docblock
Calculates the minimum and maximum decimal values from an iterator of
decimals. [Read
more](../geometrics/trait.GeometricObject.html#method.calculate_range)
:::
::::::::::::::::::::::::

::: {#impl-GeometricTransformations%3CPoint2D%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1550-1659){.src
.rightside}[§](#impl-GeometricTransformations%3CPoint2D%3E-for-Curve){.anchor}

### impl [GeometricTransformations](../geometrics/trait.GeometricTransformations.html "trait optionstratlib::geometrics::GeometricTransformations"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-geometrictransformationspoint2d-for-curve .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1551){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-2 .code-header}
:::

::: docblock
The error type that can be returned by geometric operations.
:::

::: {#method.translate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1553-1568){.src
.rightside}[§](#method.translate){.anchor}

#### fn [translate](../geometrics/trait.GeometricTransformations.html#tymethod.translate){.fn}(&self, deltas: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-translateself-deltas-vecdecimal---resultself-selferror .code-header}
:::

::: docblock
Translates the geometric object by specified amounts along each
dimension. [Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.translate)
:::

::: {#method.scale .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1570-1585){.src
.rightside}[§](#method.scale){.anchor}

#### fn [scale](../geometrics/trait.GeometricTransformations.html#tymethod.scale){.fn}(&self, factors: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-scaleself-factors-vecdecimal---resultself-selferror .code-header}
:::

::: docblock
Scales the geometric object by specified factors along each dimension.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.scale)
:::

::: {#method.intersect_with .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1587-1603){.src
.rightside}[§](#method.intersect_with){.anchor}

#### fn [intersect_with](../geometrics/trait.GeometricTransformations.html#tymethod.intersect_with){.fn}(&self, other: &Self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\>, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-intersect_withself-other-self---resultvecpoint2d-selferror .code-header}
:::

::: docblock
Finds all intersection points between this geometric object and another.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.intersect_with)
:::

::: {#method.derivative_at .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1605-1615){.src
.rightside}[§](#method.derivative_at){.anchor}

#### fn [derivative_at](../geometrics/trait.GeometricTransformations.html#tymethod.derivative_at){.fn}(&self, point: &[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-derivative_atself-point-point2d---resultvecdecimal-selferror .code-header}
:::

::: docblock
Calculates the derivative at a specific point on the geometric object.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.derivative_at)
:::

::: {#method.extrema .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1617-1640){.src
.rightside}[§](#method.extrema){.anchor}

#### fn [extrema](../geometrics/trait.GeometricTransformations.html#tymethod.extrema){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}), Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-extremaself---resultpoint2d-point2d-selferror .code-header}
:::

::: docblock
Finds the extrema (minimum and maximum points) of the geometric object.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.extrema)
:::

::: {#method.measure_under .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1642-1658){.src
.rightside}[§](#method.measure_under){.anchor}

#### fn [measure_under](../geometrics/trait.GeometricTransformations.html#tymethod.measure_under){.fn}(&self, base_value: &[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-measure_underself-base_value-decimal---resultdecimal-selferror .code-header}
:::

::: docblock
Calculates the area or volume under the geometric object relative to a
base value. [Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.measure_under)
:::
:::::::::::::::::

::: {#impl-HasX-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#220-224){.src
.rightside}[§](#impl-HasX-for-Point2D){.anchor}

### impl [HasX](../geometrics/trait.HasX.html "trait optionstratlib::geometrics::HasX"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-hasx-for-point2d .code-header}
:::

::::: impl-items
::: {#method.get_x .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#221-223){.src
.rightside}[§](#method.get_x){.anchor}

#### fn [get_x](../geometrics/trait.HasX.html#tymethod.get_x){.fn}(&self) -\> [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-get_xself---decimal .code-header}
:::

::: docblock
Returns the X-coordinate value of this object. [Read
more](../geometrics/trait.HasX.html#tymethod.get_x)
:::
:::::

::: {#impl-Hash-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#76-81){.src
.rightside}[§](#impl-Hash-for-Point2D){.anchor}

### impl [Hash](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html "trait core::hash::Hash"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-hash-for-point2d .code-header}
:::

:::::::: impl-items
::: {#method.hash .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#77-80){.src
.rightside}[§](#method.hash){.anchor}

#### fn [hash](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#tymethod.hash){.fn}\<H: [Hasher](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher"){.trait}\>(&self, state: [&mut H](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-hashh-hasherself-state-mut-h .code-header}
:::

::: docblock
Feeds this value into the given
[`Hasher`](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher").
[Read
more](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#tymethod.hash)
:::

:::: {#method.hash_slice .section .method .trait-impl}
[[1.3.0]{.since title="Stable since Rust version 1.3.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/hash/mod.rs.html#235-237){.src}]{.rightside}[§](#method.hash_slice){.anchor}

#### fn [hash_slice](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#method.hash_slice){.fn}\<H\>(data: &\[Self\], state: [&mut H](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-hash_slicehdata-self-state-mut-h .code-header}

::: where
where H:
[Hasher](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher"){.trait},
Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Feeds a slice of this type into the given
[`Hasher`](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher").
[Read
more](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#method.hash_slice)
:::
::::::::

:::: {#impl-Interpolate%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#301){.src
.rightside}[§](#impl-Interpolate%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [Interpolate](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-interpolatepoint2d-decimal-for-curve .code-header}

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

- [`LinearInterpolation`](../geometrics/trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation")
- [`BiLinearInterpolation`](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation")
- [`CubicInterpolation`](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation")
- [`SplineInterpolation`](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation")

These underlying traits implement specific interpolation algorithms,
enabling `Curve` to support a robust set of interpolation options
through the associated methods. Depending on the use case and provided
parameters (e.g., interpolation type and target x-coordinate), the
appropriate algorithm is invoked.

#### [§](#see-also){.doc-anchor}See Also {#see-also}

- [`Curve`](struct.Curve.html "struct optionstratlib::curves::Curve"):
  The underlying mathematical structure being interpolated.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The fundamental data type for the curve's points.
- [`Interpolate`](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"):
  The trait defining interpolation operations.
:::

::::::: impl-items
::: {#method.interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#80-91){.src
.rightside}[§](#method.interpolate){.anchor}

#### fn [interpolate](../geometrics/trait.Interpolate.html#method.interpolate){.fn}( &self, x: Input, interpolation_type: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-interpolate-self-x-input-interpolation_type-interpolationtype---resultpoint-interpolationerror .code-header}
:::

::: docblock
Interpolates a value at the given x-coordinate using the specified
interpolation method. [Read
more](../geometrics/trait.Interpolate.html#method.interpolate)
:::

::: {#method.find_bracket_points .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#110-132){.src
.rightside}[§](#method.find_bracket_points){.anchor}

#### fn [find_bracket_points](../geometrics/trait.Interpolate.html#method.find_bracket_points){.fn}( &self, x: Input, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}), [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-find_bracket_points-self-x-input---resultusize-usize-interpolationerror .code-header}
:::

::: docblock
Finds the indices of points that bracket the given x-coordinate. [Read
more](../geometrics/trait.Interpolate.html#method.find_bracket_points)
:::
:::::::

:::: {#impl-Interpolate%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#598){.src
.rightside}[§](#impl-Interpolate%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [Interpolate](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-interpolatepoint3d-point2d-for-surface .code-header}

::: docblock
Implementation of the `Interpolate` trait for the `Surface` type,
enabling interpolation from 3D surface points to 2D points.
:::
::::

:::: docblock
#### [§](#overview-2){.doc-anchor}Overview {#overview-2}

This implementation allows a `Surface` object to perform various types
of interpolation (linear, bilinear, cubic, and spline) by projecting 3D
points from the surface to 2D points.

#### [§](#functionality-1){.doc-anchor}Functionality

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

#### [§](#related-traits-1){.doc-anchor}Related Traits

This implementation relies on the surface also implementing:

- `LinearInterpolation<Point3D, Point2D>`
- `BiLinearInterpolation<Point3D, Point2D>`
- `CubicInterpolation<Point3D, Point2D>`
- `SplineInterpolation<Point3D, Point2D>`
- `GeometricObject<Point3D, Point2D>`
::::

::::::: impl-items
::: {#method.interpolate-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#80-91){.src
.rightside}[§](#method.interpolate-1){.anchor}

#### fn [interpolate](../geometrics/trait.Interpolate.html#method.interpolate){.fn}( &self, x: Input, interpolation_type: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-interpolate-self-x-input-interpolation_type-interpolationtype---resultpoint-interpolationerror-1 .code-header}
:::

::: docblock
Interpolates a value at the given x-coordinate using the specified
interpolation method. [Read
more](../geometrics/trait.Interpolate.html#method.interpolate)
:::

::: {#method.find_bracket_points-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#110-132){.src
.rightside}[§](#method.find_bracket_points-1){.anchor}

#### fn [find_bracket_points](../geometrics/trait.Interpolate.html#method.find_bracket_points){.fn}( &self, x: Input, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}), [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-find_bracket_points-self-x-input---resultusize-usize-interpolationerror-1 .code-header}
:::

::: docblock
Finds the indices of points that bracket the given x-coordinate. [Read
more](../geometrics/trait.Interpolate.html#method.find_bracket_points)
:::
:::::::

:::: {#impl-LinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#366-384){.src
.rightside}[§](#impl-LinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [LinearInterpolation](../geometrics/trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-linearinterpolationpoint2d-decimal-for-curve .code-header}

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

#### [§](#parameters-4){.doc-anchor}Parameters {#parameters-4}

- `x`: A `Decimal` representing the `x`-coordinate for which the
  corresponding interpolated `y` value is to be computed.

#### [§](#returns-5){.doc-anchor}Returns {#returns-5}

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

#### [§](#errors-3){.doc-anchor}Errors {#errors-3}

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

#### [§](#see-also-1){.doc-anchor}See Also {#see-also-1}

- `find_bracket_points`: Finds two points that bracket a value.
- `Point2D`: Represents points in 2D space.
- `CurvesError`: Represents errors related to curve operations.
:::::

::::: impl-items
::: {#method.linear_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#373-383){.src
.rightside}[§](#method.linear_interpolate){.anchor}

#### fn [linear_interpolate](../geometrics/trait.LinearInterpolation.html#tymethod.linear_interpolate){.fn}(&self, x: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-linear_interpolateself-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

::: docblock
##### [§](#method){.doc-anchor}Method

###### [§](#linear_interpolate){.doc-anchor}`linear_interpolate`

Performs linear interpolation for a given `x` value by finding two
consecutive points on the curve (`p1` and `p2`) that bracket the
provided `x`. The `y` value is then calculated using the linear
interpolation formula:
:::
:::::

:::: {#impl-LinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#621-701){.src
.rightside}[§](#impl-LinearInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [LinearInterpolation](../geometrics/trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-linearinterpolationpoint3d-point2d-for-surface .code-header}

::: docblock
#### [§](#linear-interpolation-for-surfaces){.doc-anchor}Linear Interpolation for Surfaces

Implementation of the `LinearInterpolation` trait for `Surface`
structures, enabling interpolation from 2D points to 3D points using
barycentric coordinates.
:::
::::

::: docblock
##### [§](#overview-3){.doc-anchor}Overview {#overview-3}

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
::: {#method.linear_interpolate-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#636-700){.src
.rightside}[§](#method.linear_interpolate-1){.anchor}

#### fn [linear_interpolate](../geometrics/trait.LinearInterpolation.html#tymethod.linear_interpolate){.fn}(&self, xy: [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-linear_interpolateself-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
###### [§](#parameters-13){.doc-anchor}Parameters {#parameters-13}

- `xy` - A `Point2D` representing the x and y coordinates where
  interpolation is needed

###### [§](#returns-15){.doc-anchor}Returns {#returns-15}

- `Result<Point3D, InterpolationError>` - The interpolated 3D point if
  successful, or an appropriate error if interpolation cannot be
  performed

###### [§](#errors-10){.doc-anchor}Errors {#errors-10}

Returns `InterpolationError::Linear` in the following cases:

- When the surface contains only coincident points forming a degenerate
  triangle
- When the query point is outside the surface's x-y range
:::
:::::

:::: {#impl-MergeAxisInterpolate%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1510-1548){.src
.rightside}[§](#impl-MergeAxisInterpolate%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [MergeAxisInterpolate](../geometrics/trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-mergeaxisinterpolatepoint2d-decimal-for-curve .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::::: impl-items
::: {#method.merge_axis_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1514-1547){.src
.rightside}[§](#method.merge_axis_interpolate){.anchor}

#### fn [merge_axis_interpolate](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate){.fn}( &self, other: &Self, interpolation: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Self, Self), Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-merge_axis_interpolate-self-other-self-interpolation-interpolationtype---resultself-self-selferror .code-header}
:::

::: docblock
Interpolates both structures to align them on a common set of index
values. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate)
:::

::: {#method.merge_axis_index .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#144-147){.src
.rightside}[§](#method.merge_axis_index){.anchor}

#### fn [merge_axis_index](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index){.fn}\<\'a\>(&\'a self, other: &\'a Self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_axis_indexaa-self-other-a-self---vecinput .code-header}
:::

::: docblock
Merges the index values from two structures into a single ordered
vector. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index)
:::
:::::::

:::: {#impl-MergeAxisInterpolate%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1337-1385){.src
.rightside}[§](#impl-MergeAxisInterpolate%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [MergeAxisInterpolate](../geometrics/trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-mergeaxisinterpolatepoint3d-point2d-for-surface .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::::: impl-items
::: {#method.merge_axis_interpolate-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1341-1384){.src
.rightside}[§](#method.merge_axis_interpolate-1){.anchor}

#### fn [merge_axis_interpolate](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate){.fn}( &self, other: &Self, interpolation: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Self, Self), Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-merge_axis_interpolate-self-other-self-interpolation-interpolationtype---resultself-self-selferror-1 .code-header}
:::

::: docblock
Interpolates both structures to align them on a common set of index
values. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate)
:::

::: {#method.merge_axis_index-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#144-147){.src
.rightside}[§](#method.merge_axis_index-1){.anchor}

#### fn [merge_axis_index](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index){.fn}\<\'a\>(&\'a self, other: &\'a Self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_axis_indexaa-self-other-a-self---vecinput-1 .code-header}
:::

::: docblock
Merges the index values from two structures into a single ordered
vector. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index)
:::
:::::::

::: {#impl-Ord-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#89-96){.src
.rightside}[§](#impl-Ord-for-Point2D){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-ord-for-point2d .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#90-95){.src
.rightside}[§](#method.cmp){.anchor}

#### fn [cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#tymethod.cmp){.fn}(&self, other: &Self) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-cmpself-other-self---ordering .code-header}
:::

::: docblock
This method returns an
[`Ordering`](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering")
between `self` and `other`. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#tymethod.cmp)
:::

:::: {#method.max .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1023-1025){.src}]{.rightside}[§](#method.max){.anchor}

#### fn [max](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.max){.fn}(self, other: Self) -\> Self {#fn-maxself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the maximum of two values. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.max)
:::

:::: {#method.min .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1062-1064){.src}]{.rightside}[§](#method.min){.anchor}

#### fn [min](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.min){.fn}(self, other: Self) -\> Self {#fn-minself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the minimum of two values. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.min)
:::

:::: {#method.clamp .section .method .trait-impl}
[[1.50.0]{.since title="Stable since Rust version 1.50.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1088-1090){.src}]{.rightside}[§](#method.clamp){.anchor}

#### fn [clamp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.clamp){.fn}(self, min: Self, max: Self) -\> Self {#fn-clampself-min-self-max-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Restrict a value to a certain interval. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.clamp)
:::
::::::::::::::

::: {#impl-PartialEq-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#68-72){.src
.rightside}[§](#impl-PartialEq-for-Point2D){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-partialeq-for-point2d .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#69-71){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-self---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialOrd-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#83-87){.src
.rightside}[§](#impl-PartialOrd-for-Point2D){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-partialord-for-point2d .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#84-86){.src
.rightside}[§](#method.partial_cmp){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-self---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-Serialize-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-Serialize-for-Point2D){.anchor}

### impl [Serialize](../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-serialize-for-point2d .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
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

:::: {#impl-SplineInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#771-953){.src
.rightside}[§](#impl-SplineInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [SplineInterpolation](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-splineinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `SplineInterpolation` trait for the `Curve` struct,
providing functionality to perform cubic spline interpolation.
:::
::::

::: docblock
#### [§](#overview-1){.doc-anchor}Overview {#overview-1}

This method calculates the interpolated `y` value for a given `x` value
by using cubic spline interpolation on the points in the `Curve`. The
method ensures a smooth transition between points by computing second
derivatives of the curve at each point, and uses those derivatives in
the spline interpolation formula.

#### [§](#parameters-10){.doc-anchor}Parameters {#parameters-10}

- `x`: The x-coordinate at which the curve should be interpolated. This
  value is passed as a `Decimal` for precise calculations.

#### [§](#returns-11){.doc-anchor}Returns {#returns-11}

- On success, returns a `Point2D` instance representing the interpolated
  point.
- On error, returns a `CurvesError` indicating the reason for failure
  (e.g., insufficient points or an out-of-range x-coordinate).

#### [§](#errors-8){.doc-anchor}Errors {#errors-8}

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

#### [§](#example-usage){.doc-anchor}Example Usage

Refer to the documentation for how to use the `SplineInterpolation`
trait, as examples are not provided inline with this implementation.

#### [§](#see-also-4){.doc-anchor}See Also

- [`SplineInterpolation`](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"):
  The trait definition for spline interpolation.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents a point in 2D space.
- [`Curve`](struct.Curve.html "struct optionstratlib::curves::Curve"):
  Represents a mathematical curve made up of points for interpolation.
- [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"):
  Enumerates possible errors during curve operations.
:::

:::::: impl-items
::: {#method.spline_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#867-952){.src
.rightside}[§](#method.spline_interpolate){.anchor}

#### fn [spline_interpolate](../geometrics/trait.SplineInterpolation.html#tymethod.spline_interpolate){.fn}(&self, x: [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-spline_interpolateself-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

:::: docblock
Performs cubic spline interpolation for a given x-coordinate and returns
the interpolated `Point2D` value. This function computes the second
derivatives of the curve points, solves a tridiagonal system to derive
the interpolation parameters, and evaluates the spline function for the
provided `x` value.

##### [§](#parameters-9){.doc-anchor}Parameters {#parameters-9}

- `x`:
  - The x-coordinate at which the interpolation is to be performed.
  - Must be of type `Decimal`.

##### [§](#returns-10){.doc-anchor}Returns {#returns-10}

- `Ok(Point2D)`:

  - The `Point2D` instance representing the interpolated point at the
    given `x` value.
  - The interpolated `y` value is calculated based on the cubic spline
    interpolation algorithm.

- `Err(CurvesError)`:

  - Returned when an error occurs during the interpolation process, such
    as:
    - Insufficient points provided (less than 3 points).
    - The given `x` is outside the valid range of the points.
    - Unable to determine the correct segment for interpolation.

##### [§](#errors-7){.doc-anchor}Errors {#errors-7}

- `CurvesError::InterpolationError`:
  - Occurs under the following conditions:
    - **"Need at least three points for spline interpolation"**:
      Requires at least 3 points to perform cubic spline interpolation.
    - **"x is outside the range of points"**: The provided `x` value
      lies outside the domain of the curve points.
    - **"Could not find valid segment for interpolation"**: Spline
      interpolation fails due to an invalid segment determination.

##### [§](#pre-conditions){.doc-anchor}Pre-conditions

- The curve must contain at least three points for cubic spline
  interpolation.
- The `x` value must fall within the range of the curve's x-coordinates.

##### [§](#implementation-details-1){.doc-anchor}Implementation Details

- **Inputs**:
  - Uses the `get_points` method of the curve to retrieve the list of
    `Point2D` instances that define the interpolation curve.
  - Computes the second derivatives (`m`) for each point using the
    Thomas algorithm to solve a tridiagonal system.
- **Boundary Conditions**:
  - Natural spline boundary conditions are used, with the second
    derivatives on the boundary set to zero.
- **Interpolation**:
  - Determines the segment `[x_i, x_{i+1}]` to which the input `x`
    belongs.
  - Uses the cubic spline equation to calculate the interpolated `y`
    value.

##### [§](#mathematical-formulation){.doc-anchor}Mathematical Formulation

Let `x_i`, `x_{i+1}`, `y_i`, `y_{i+1}` refer to the points of the
segment where `x` lies. The cubic spline function at `x` is computed as
follows:

::: example-wrap
``` language-text
S(x) = m_i * (x_{i+1} - x)^3 / (6 * h)
     + m_{i+1} * (x - x_i)^3 / (6 * h)
     + (y_i / h - h * m_i / 6) * (x_{i+1} - x)
     + (y_{i+1} / h - h * m_{i+1} / 6) * (x - x_i)
```
:::

Where:

- `m_i`, `m_{i+1}` are the second derivatives at `x_i` and `x_{i+1}`.
- `h = x_{i+1} - x_i` is the distance between the two points.
- `(x_{i+1} - x)` and `(x - x_i)` are the relative distances within the
  segment.

##### [§](#example-usages-non-code){.doc-anchor}Example Usages (Non-code)

This method is typically used for high-precision curve fitting or
graphical rendering where smooth transitions between points are
essential. Common applications include:

- Signal processing.
- Data interpolation for missing values.
- Smooth graphical representations of mathematical functions.

##### [§](#related-types){.doc-anchor}Related Types

- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents a 2D point and is used as input/output for this function.
- [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError")
  Represents any error encountered during interpolation.

##### [§](#performance){.doc-anchor}Performance

- The function operates with `O(n)` complexity, where `n` is the number
  of points. The tridiagonal system is solved efficiently using the
  Thomas algorithm.

##### [§](#notes-2){.doc-anchor}Notes

- Natural spline interpolation may introduce minor deviations beyond the
  range of existing data points due to its boundary conditions. For
  strictly constrained results, consider alternative interpolation
  methods, such as linear or cubic Hermite interpolation.
::::
::::::

::: {#impl-SplineInterpolation%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#857-947){.src
.rightside}[§](#impl-SplineInterpolation%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [SplineInterpolation](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-splineinterpolationpoint3d-point2d-for-surface .code-header}
:::

::::: impl-items
::: {#method.spline_interpolate-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#858-946){.src
.rightside}[§](#method.spline_interpolate-1){.anchor}

#### fn [spline_interpolate](../geometrics/trait.SplineInterpolation.html#tymethod.spline_interpolate){.fn}(&self, xy: [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-spline_interpolateself-xy-point2d---resultpoint3d-interpolationerror .code-header}
:::

::: docblock
Interpolates a y-value for the provided x-coordinate using spline
interpolation. [Read
more](../geometrics/trait.SplineInterpolation.html#tymethod.spline_interpolate)
:::
:::::

::: {#impl-ToSchema-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-ToSchema-for-Point2D){.anchor}

### impl [ToSchema](../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-toschema-for-point2d .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
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

::: {#impl-Copy-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#51){.src
.rightside}[§](#impl-Copy-for-Point2D){.anchor}

### impl [Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-copy-for-point2d .code-header}
:::

::: {#impl-Eq-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#74){.src
.rightside}[§](#impl-Eq-for-Point2D){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-eq-for-point2d .code-header}
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Point2D .section .impl}
[§](#impl-Freeze-for-Point2D){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-freeze-for-point2d .code-header}
:::

::: {#impl-RefUnwindSafe-for-Point2D .section .impl}
[§](#impl-RefUnwindSafe-for-Point2D){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-refunwindsafe-for-point2d .code-header}
:::

::: {#impl-Send-for-Point2D .section .impl}
[§](#impl-Send-for-Point2D){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-send-for-point2d .code-header}
:::

::: {#impl-Sync-for-Point2D .section .impl}
[§](#impl-Sync-for-Point2D){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-sync-for-point2d .code-header}
:::

::: {#impl-Unpin-for-Point2D .section .impl}
[§](#impl-Unpin-for-Point2D){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-unpin-for-point2d .code-header}
:::

::: {#impl-UnwindSafe-for-Point2D .section .impl}
[§](#impl-UnwindSafe-for-Point2D){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-unwindsafe-for-point2d .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-Comparable%3CK%3E-for-Q .section .impl}
[Source](../../src/equivalent/lib.rs.html#104-107){.src
.rightside}[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Comparable](../../equivalent/trait.Comparable.html "trait equivalent::Comparable"){.trait}\<K\> for Q {#implq-k-comparablek-for-q .code-header}

::: where
where Q:
[Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.compare .section .method .trait-impl}
[Source](../../src/equivalent/lib.rs.html#110){.src
.rightside}[§](#method.compare){.anchor}

#### fn [compare](../../equivalent/trait.Comparable.html#tymethod.compare){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[Source](../../src/hashbrown/lib.rs.html#167-170){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Equivalent](../../hashbrown/trait.Equivalent.html "trait hashbrown::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent .section .method .trait-impl}
[Source](../../src/hashbrown/lib.rs.html#172){.src
.rightside}[§](#method.equivalent){.anchor}

#### fn [equivalent](../../hashbrown/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. [Read
more](../../hashbrown/trait.Equivalent.html#tymethod.equivalent)
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[Source](../../src/equivalent/lib.rs.html#82-85){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> [Equivalent](../../equivalent/trait.Equivalent.html "trait equivalent::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent-1 .section .method .trait-impl}
[Source](../../src/equivalent/lib.rs.html#88){.src
.rightside}[§](#method.equivalent-1){.anchor}

#### fn [equivalent](../../equivalent/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
:::

::: docblock
Compare self to `key` and return `true` if they are equal.
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#785){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from-1){.anchor}

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
::: {#associatedtype.Error-6 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#831){.src
.rightside}[§](#associatedtype.Error-6){.anchor}

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
::: {#associatedtype.Error-5 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#815){.src
.rightside}[§](#associatedtype.Error-5){.anchor}

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

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](../../src/serde_core/de/mod.rs.html#633){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](../../serde_core/de/trait.DeserializeOwned.html "trait serde_core::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](../../src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](../../nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
