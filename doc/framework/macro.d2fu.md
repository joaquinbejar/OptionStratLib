:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [d2fu]{.macro}Copy item path

[[Source](../src/optionstratlib/model/decimal.rs.html#336-340){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! d2fu {
    ($val:expr) => { ... };
}
```

Expand description

:::: docblock
Converts a Decimal value to f64 without error checking.

This macro converts a Decimal type to an f64 floating-point value. It's
an "unchecked" version that doesn't handle potential conversion errors.

## [§](#parameters){.doc-anchor}Parameters

- `$val` - A Decimal value to be converted to f64

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::d2fu;
let decimal_value = dec!(10.5);
let float_value = d2fu!(decimal_value);
```
:::
::::
:::::::
::::::::
