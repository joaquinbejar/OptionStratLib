:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Trait [ToRound]{.trait} Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#363-378){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait ToRound {
    // Required methods
    fn round(&self) -> Decimal;
    fn round_to(&self, decimal_places: u32) -> Decimal;
}
```

Expand description

::: docblock
Trait for rounding operations on numeric types, specifically for
financial calculations.

This trait provides methods to round a number to the nearest integer and
to a specified number of decimal places, ensuring precision and accuracy
in financial computations.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.round .section .method}
[Source](../../../src/optionstratlib/model/utils.rs.html#367){.src
.rightside}

#### fn [round](#tymethod.round){.fn}(&self) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-roundself---decimal .code-header}
:::

::: docblock
Rounds the number to the nearest integer.

This method rounds the number to the nearest whole number, removing any
fractional part.
:::

::: {#tymethod.round_to .section .method}
[Source](../../../src/optionstratlib/model/utils.rs.html#377){.src
.rightside}

#### fn [round_to](#tymethod.round_to){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-round_toself-decimal_places-u32---decimal .code-header}
:::

::: docblock
Rounds the number to a specified number of decimal places.

This method rounds the number to the specified number of digits after
the decimal point, providing control over the precision of the rounded
value.

##### [§](#arguments){.doc-anchor}Arguments

- `decimal_places` - The number of decimal places to round to.
:::
:::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-ToRound-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#718-726){.src
.rightside}[§](#impl-ToRound-for-Positive){.anchor}

### impl [ToRound](trait.ToRound.html "trait optionstratlib::model::utils::ToRound"){.trait} for [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-toround-for-positive .code-header}
:::
::::
:::::::::::::
::::::::::::::
