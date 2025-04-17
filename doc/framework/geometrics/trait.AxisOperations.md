::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [AxisOperations]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#21-116){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait AxisOperations<Point, Input>where
    Input: Hash + Eq + Clone + Ord,{
    type Error;

    // Required methods
    fn contains_point(&self, x: &Input) -> bool;
    fn get_index_values(&self) -> Vec<Input>;
    fn get_values(&self, x: Input) -> Vec<&Decimal>;
    fn get_closest_point(&self, x: &Input) -> Result<&Point, Self::Error>;
    fn get_point(&self, x: &Input) -> Option<&Point>;

    // Provided method
    fn merge_indexes(&self, axis: Vec<Input>) -> Vec<Input> { ... }
}
```

Expand description

::: docblock
Trait for handling axis-based operations on geometric structures.

This trait provides methods for efficient lookups and manipulations of
points based on their coordinate values. It is designed to work with
both 2D curves and 3D surfaces.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `Point` - The complete point type (Point2D for curves, Point3D for
  surfaces)
- `Input` - The input coordinate type (Decimal for curves, Point2D for
  surfaces)
:::

## Required Associated Types[§](#required-associated-types){.anchor} {#required-associated-types .section-header}

::::: methods
::: {#associatedtype.Error .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#26){.src
.rightside}

#### type [Error](#associatedtype.Error){.associatedtype} {#type-error .code-header}
:::

::: docblock
The type of error that can occur during point operations
:::
:::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::::::::: methods
::: {#tymethod.contains_point .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#35){.src
.rightside}

#### fn [contains_point](#tymethod.contains_point){.fn}(&self, x: [&Input](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-contains_pointself-x-input---bool .code-header}
:::

::: docblock
Checks if a coordinate value exists in the structure.

##### [§](#arguments){.doc-anchor}Arguments

- `x` - The coordinate value to check (x for curves, xy-point for
  surfaces)

##### [§](#returns){.doc-anchor}Returns

- `bool` - `true` if the coordinate exists, `false` otherwise
:::

::: {#tymethod.get_index_values .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#44){.src
.rightside}

#### fn [get_index_values](#tymethod.get_index_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-get_index_valuesself---vecinput .code-header}
:::

::: docblock
Returns a vector of references to all index values in the structure.

For curves, this returns x-coordinates. For surfaces, this returns
xy-coordinates.

##### [§](#returns-1){.doc-anchor}Returns

- `Vec<&Input>` - Vector of references to index values
:::

::: {#tymethod.get_values .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#56){.src
.rightside}

#### fn [get_values](#tymethod.get_values){.fn}(&self, x: Input) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\> {#fn-get_valuesself-x-input---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to dependent values for a given
coordinate.

For curves, returns y-values for a given x-coordinate. For surfaces,
returns z-values for a given xy-coordinate.

##### [§](#arguments-1){.doc-anchor}Arguments

- `x` - The coordinate value to lookup

##### [§](#returns-2){.doc-anchor}Returns

- `Vec<&Decimal>` - Vector of references to dependent values
:::

::: {#tymethod.get_closest_point .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#65){.src
.rightside}

#### fn [get_closest_point](#tymethod.get_closest_point){.fn}(&self, x: [&Input](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}, Self::[Error](trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-get_closest_pointself-x-input---resultpoint-selferror .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value.

##### [§](#arguments-2){.doc-anchor}Arguments

- `x` - The reference coordinate value

##### [§](#returns-3){.doc-anchor}Returns

- `Result<&Point, Self::Error>` - The closest point or an error if no
  points exist
:::

::: {#tymethod.get_point .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#74){.src
.rightside}

#### fn [get_point](#tymethod.get_point){.fn}(&self, x: [&Input](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}\> {#fn-get_pointself-x-input---optionpoint .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value.

##### [§](#arguments-3){.doc-anchor}Arguments

- `x` - The reference coordinate value

##### [§](#returns-4){.doc-anchor}Returns

- `Result<&Point, Self::Error>` - The closest point or an error if no
  points exist
:::
:::::::::::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.merge_indexes .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#85-115){.src
.rightside}

#### fn [merge_indexes](#method.merge_indexes){.fn}(&self, axis: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\>) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_indexesself-axis-vecinput---vecinput .code-header}
:::

::: docblock
Merges the index values from the current structure with an additional
set of indices. This combines self.get_index_values() with the provided
axis vector to create a single vector of unique indices.

##### [§](#arguments-4){.doc-anchor}Arguments

- `axis` - Additional index values to merge with current structure's
  indices

##### [§](#returns-5){.doc-anchor}Returns

- `Vec<&Input>` - Vector containing unique combined indices
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::::: {#implementors-list}
::: {#impl-AxisOperations%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1455-1494){.src
.rightside}[§](#impl-AxisOperations%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [AxisOperations](trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-axisoperationspoint2d-decimal-for-curve .code-header}
:::

:::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1456){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::
::::

::: {#impl-AxisOperations%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1287-1322){.src
.rightside}[§](#impl-AxisOperations%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [AxisOperations](trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-axisoperationspoint3d-point2d-for-surface .code-header}
:::

:::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1288){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::
::::
:::::::::
::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::
