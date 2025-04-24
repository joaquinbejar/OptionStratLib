:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[decimal](index.html)
:::

# Function [decimal_normal_sample]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/decimal.rs.html#308-312){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn decimal_normal_sample() -> Decimal
```

Expand description

:::: docblock
Generates a random positive value from a standard normal distribution.

This function samples from a normal distribution with mean 0.0 and
standard deviation 1.0, and returns the value as a `Positive` type.
Since the normal distribution can produce negative values, the function
uses the `pos!` macro to convert the sample to a `Positive` value, which
will handle the conversion according to the `Positive` type's
implementation.

## [§](#returns){.doc-anchor}Returns

A `Positive` value sampled from a standard normal distribution.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::model::decimal::decimal_normal_sample;
use optionstratlib::Positive;
let normal = decimal_normal_sample();
```
:::
::::
:::::::
::::::::
