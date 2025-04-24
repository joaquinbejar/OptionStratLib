::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [d2f]{.macro}Copy item path

[[Source](../src/optionstratlib/model/decimal.rs.html#351-355){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! d2f {
    ($val:expr) => { ... };
}
```

Expand description

::: docblock
Converts a Decimal value to f64 with error propagation.

This macro converts a Decimal type to an f64 floating-point value. It
propagates any errors that might occur during conversion using the `?`
operator.

## [§](#parameters){.doc-anchor}Parameters

- `$val` - A Decimal value to be converted to f64
:::
::::::
:::::::
