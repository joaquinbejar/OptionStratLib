:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [f2du]{.macro}Copy item path

[[Source](../src/optionstratlib/model/decimal.rs.html#372-376){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! f2du {
    ($val:expr) => { ... };
}
```

Expand description

:::: docblock
Converts an f64 value to Decimal without error checking.

This macro converts an f64 floating-point value to a Decimal type. It's
an "unchecked" version that doesn't handle potential conversion errors.

## [§](#parameters){.doc-anchor}Parameters

- `$val` - An f64 value to be converted to Decimal

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::f2du;
let float_value = 10.5;
let decimal_value = f2du!(float_value);
```
:::
::::
:::::::
::::::::
