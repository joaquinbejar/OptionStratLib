:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [HasX]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#171-177){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait HasX {
    // Required method
    fn get_x(&self) -> Decimal;
}
```

Expand description

::: docblock
A trait for types that provide access to an X-coordinate value.

This trait defines a standard interface for any type that contains or
can compute an X-coordinate represented as a `Decimal` value.
Implementing this trait allows objects to be used in contexts where
X-coordinate access is required, such as:

- Geometric calculations
- Plotting and visualization
- Interpolation algorithms
- Data point analysis

## [§](#required-method){.doc-anchor}Required Method

### [§](#get_x){.doc-anchor}`get_x`

Returns the X-coordinate value of the implementing type.

#### [§](#returns){.doc-anchor}Returns

- `Decimal`: The X-coordinate value, typically representing a position
  along the x-axis.

## [§](#implementation-notes){.doc-anchor}Implementation Notes

When implementing this trait, ensure that:

- The returned value accurately represents the X-coordinate in the
  appropriate scale and units
- The implementation handles any necessary internal conversions or
  calculations
- The method is computationally efficient if it will be called
  frequently

## [§](#usage-examples){.doc-anchor}Usage Examples

This trait can be implemented for various types such as:

- 2D or 3D points
- Data samples with timestamps or sequential positions
- Geometric shapes with a defined reference point

## [§](#see-also){.doc-anchor}See Also

- `Decimal`: The numeric type used to represent the X-coordinate value
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.get_x .section .method}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#176){.src
.rightside}

#### fn [get_x](#tymethod.get_x){.fn}(&self) -\> [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-get_xself---decimal .code-header}
:::

::: docblock
Returns the X-coordinate value of this object.

##### [§](#returns-1){.doc-anchor}Returns

- `Decimal`: The X-coordinate value as a `Decimal` type.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::: {#implementors-list}
::: {#impl-HasX-for-Point2D .section .impl}
[Source](../../src/optionstratlib/curves/types.rs.html#220-224){.src
.rightside}[§](#impl-HasX-for-Point2D){.anchor}

### impl [HasX](trait.HasX.html "trait optionstratlib::geometrics::HasX"){.trait} for [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#impl-hasx-for-point2d .code-header}
:::

::: {#impl-HasX-for-Decimal .section .impl}
[Source](../../src/optionstratlib/model/decimal.rs.html#314-318){.src
.rightside}[§](#impl-HasX-for-Decimal){.anchor}

### impl [HasX](trait.HasX.html "trait optionstratlib::geometrics::HasX"){.trait} for [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-hasx-for-decimal .code-header}
:::

::: {#impl-HasX-for-Point3D .section .impl}
[Source](../../src/optionstratlib/surfaces/types.rs.html#212-216){.src
.rightside}[§](#impl-HasX-for-Point3D){.anchor}

### impl [HasX](trait.HasX.html "trait optionstratlib::geometrics::HasX"){.trait} for [Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct} {#impl-hasx-for-point3d .code-header}
:::
::::::
:::::::::::::
::::::::::::::
