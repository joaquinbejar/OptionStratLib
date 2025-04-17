::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [positive_f64_to_f64]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#27-29){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn positive_f64_to_f64(vec: Vec<Positive>) -> Vec<f64>
```

Expand description

::: docblock
Converts a vector of `Positive` values to a vector of `f64` values.

This utility function transforms a collection of `Positive` type values
to standard floating-point values by applying the `to_f64()` method to
each element. The function consumes the input vector and returns a new
vector containing the converted values.

## [§](#parameters){.doc-anchor}Parameters

- `vec` - A vector of `Positive` values to be converted.

## [§](#returns){.doc-anchor}Returns

A vector of `f64` values corresponding to the input `Positive` values.
:::
::::::
:::::::
