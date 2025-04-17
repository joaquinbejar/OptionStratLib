:::::::::::::::::::: width-limiter
::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [MergeAxisInterpolate]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#129-168){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait MergeAxisInterpolate<Point, Input>: AxisOperations<Point, Input>where
    Point: Clone,
    Input: Hash + Eq + Clone + Ord,{
    // Required method
    fn merge_axis_interpolate(
        &self,
        other: &Self,
        interpolation: InterpolationType,
    ) -> Result<(Self, Self), Self::Error>
       where Self: Sized;

    // Provided method
    fn merge_axis_index<'a>(&'a self, other: &'a Self) -> Vec<Input> { ... }
}
```

Expand description

::: docblock
Trait for merging and interpolating axes between compatible geometric
structures.

This trait extends `AxisOperations` by providing methods to merge index
values from two structures and interpolate points across the combined
axes. This functionality is essential for operations that require
aligning points from different structures onto a common set of
coordinates.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `Point` - The complete point type (typically Point2D or Point3D)
- `Input` - The input coordinate type (typically Decimal for 1D axes or
  Point2D for 2D axes)
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

:::::: methods
:::: {#tymethod.merge_axis_interpolate .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#161-167){.src
.rightside}

#### fn [merge_axis_interpolate](#tymethod.merge_axis_interpolate){.fn}( &self, other: &Self, interpolation: [InterpolationType](enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Self, Self), Self::[Error](trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-merge_axis_interpolate-self-other-self-interpolation-interpolationtype---resultself-self-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Interpolates both structures to align them on a common set of index
values.

This method ensures that both structures have points at exactly the same
coordinate positions by adding interpolated points where necessary.

##### [§](#arguments){.doc-anchor}Arguments

- `other` - Another structure implementing the same trait
- `interpolation` - The interpolation method to use when creating new
  points

##### [§](#returns){.doc-anchor}Returns

- `Result<(Self, Self), Self::Error>` - A tuple containing both
  structures with aligned coordinate points, or an error if
  interpolation fails
:::
::::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.merge_axis_index .section .method}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#144-147){.src
.rightside}

#### fn [merge_axis_index](#method.merge_axis_index){.fn}\<\'a\>(&\'a self, other: &\'a Self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_axis_indexaa-self-other-a-self---vecinput .code-header}
:::

::: docblock
Merges the index values from two structures into a single ordered
vector.

This method combines the index values from `self` and `other` to create
a common set of indices that can be used for interpolation or alignment.

##### [§](#arguments-1){.doc-anchor}Arguments

- `other` - Another structure implementing the same trait

##### [§](#returns-1){.doc-anchor}Returns

- `Vec<Input>` - Vector containing merged index values
:::
:::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.86.0/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::: {#implementors-list}
:::: {#impl-MergeAxisInterpolate%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1496-1534){.src
.rightside}[§](#impl-MergeAxisInterpolate%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [MergeAxisInterpolate](trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-mergeaxisinterpolatepoint2d-decimal-for-curve .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

:::: {#impl-MergeAxisInterpolate%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1324-1372){.src
.rightside}[§](#impl-MergeAxisInterpolate%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [MergeAxisInterpolate](trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-mergeaxisinterpolatepoint3d-point2d-for-surface .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::
:::::::
:::::::::::::::::::
::::::::::::::::::::
