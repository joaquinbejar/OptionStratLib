::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [Arithmetic]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/operations/traits.rs.html#67-76){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Arithmetic<Input> {
    type Error;

    // Required methods
    fn merge(
        geometries: &[&Input],
        operation: MergeOperation,
    ) -> Result<Input, Self::Error>;
    fn merge_with(
        &self,
        other: &Input,
        operation: MergeOperation,
    ) -> Result<Input, Self::Error>;
}
```

Expand description

:::: docblock
A trait that provides arithmetic operations for working with geometries,
enabling merging and combining of curve data based on different
operations.

This trait allows performing mathematical operations between geometric
objects, particularly curves, to create new derived geometries. It
supports various operations like addition, subtraction, multiplication,
and more through the `MergeOperation` enum.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `Input`: The type of geometric object that can be merged and operated
  upon.

## [§](#associated-types){.doc-anchor}Associated Types

- `Error`: Represents the type of error that may occur during curve
  operations.

## [§](#required-methods-1){.doc-anchor}Required Methods {#required-methods-1}

### [§](#merge){.doc-anchor}`merge`

Combines multiple geometries into a single curve based on a specified
`MergeOperation`.

#### [§](#parameters){.doc-anchor}Parameters

- `geometries`: A slice of references to the input geometries that need
  to be merged.
- `operation`: The operation used to combine the geometries (e.g.,
  addition, subtraction, etc.).

#### [§](#returns){.doc-anchor}Returns

- `Result<Input, Self::Error>`: Returns the resulting merged curve if
  successful, or an error of type `Self::Error` if the merge process
  fails.

### [§](#merge_with){.doc-anchor}`merge_with`

Merges the current curve with another curve based on a specified
`MergeOperation`.

#### [§](#parameters-1){.doc-anchor}Parameters

- `other`: A reference to another curve to be merged.
- `operation`: The operation used to combine the geometries (e.g.,
  addition, subtraction, etc.).

#### [§](#returns-1){.doc-anchor}Returns

- `Result<Input, Self::Error>`: Returns the resulting merged curve if
  successful, or an error of type `Self::Error` if the merge process
  fails.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use optionstratlib::geometrics::{Arithmetic, GeometricObject, MergeOperation};
use optionstratlib::curves::{Curve, Point2D};

let curve1 = Curve::from_vector(vec![
    Point2D::new(Decimal::ZERO, Decimal::ZERO),
    Point2D::new(Decimal::ONE, Decimal::ONE),
]);
let curve2 = Curve::from_vector(vec![
    Point2D::new(Decimal::ZERO, Decimal::ONE),
    Point2D::new(Decimal::ONE, Decimal::TWO),
]);

// Merge two curves by adding their values
let result_curve = Curve::merge(&[&curve1, &curve2], MergeOperation::Add);
```
:::

## [§](#notes){.doc-anchor}Notes

- This trait is designed to be implemented for specific curve types
  which define how the merging will occur. The associated error type
  should capture and communicate any issues encountered during
  operations.
- The implementation may need to handle cases where curves have
  different domains or sampling points.
::::

## Required Associated Types[§](#required-associated-types){.anchor} {#required-associated-types .section-header}

::::: methods
::: {#associatedtype.Error .section .method}
[Source](../../src/optionstratlib/geometrics/operations/traits.rs.html#69){.src
.rightside}

#### type [Error](#associatedtype.Error){.associatedtype} {#type-error .code-header}
:::

::: docblock
The error type returned when merging operations fail
:::
:::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.merge .section .method}
[Source](../../src/optionstratlib/geometrics/operations/traits.rs.html#72){.src
.rightside}

#### fn [merge](#tymethod.merge){.fn}( geometries: &\[[&Input](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\], operation: [MergeOperation](enum.MergeOperation.html "enum optionstratlib::geometrics::MergeOperation"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Input, Self::[Error](trait.Arithmetic.html#associatedtype.Error "type optionstratlib::geometrics::Arithmetic::Error"){.associatedtype}\> {#fn-merge-geometries-input-operation-mergeoperation---resultinput-selferror .code-header}
:::

::: docblock
Combines multiple geometries into one using the specified merge
operation.
:::

::: {#tymethod.merge_with .section .method}
[Source](../../src/optionstratlib/geometrics/operations/traits.rs.html#75){.src
.rightside}

#### fn [merge_with](#tymethod.merge_with){.fn}( &self, other: [&Input](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}, operation: [MergeOperation](enum.MergeOperation.html "enum optionstratlib::geometrics::MergeOperation"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Input, Self::[Error](trait.Arithmetic.html#associatedtype.Error "type optionstratlib::geometrics::Arithmetic::Error"){.associatedtype}\> {#fn-merge_with-self-other-input-operation-mergeoperation---resultinput-selferror .code-header}
:::

::: docblock
Merges the current curve with another curve using the specified merge
operation.
:::
:::::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.91.1/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::: {#implementors-list}
:::: {#impl-Arithmetic%3CCurve%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1268-1467){.src
.rightside}[§](#impl-Arithmetic%3CCurve%3E-for-Curve){.anchor}

### impl [Arithmetic](trait.Arithmetic.html "trait optionstratlib::geometrics::Arithmetic"){.trait}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-arithmeticcurve-for-curve .code-header}

::: docblock
Implements the `CurveArithmetic` trait for the `Curve` type, providing
functionality for merging multiple curves using a specified mathematical
operation and performing arithmetic operations between two curves.
:::
::::

:::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1269){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::
::::

::: {#impl-Arithmetic%3CSurface%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1179-1298){.src
.rightside}[§](#impl-Arithmetic%3CSurface%3E-for-Surface){.anchor}

### impl [Arithmetic](trait.Arithmetic.html "trait optionstratlib::geometrics::Arithmetic"){.trait}\<[Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-arithmeticsurface-for-surface .code-header}
:::

:::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#1180){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::
::::
::::::::::
::::::::::::::::::::::::
:::::::::::::::::::::::::
