::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [f2d]{.macro}Copy item path

[[Source](../src/optionstratlib/model/decimal.rs.html#387-391){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! f2d {
    ($val:expr) => { ... };
}
```

Expand description

::: docblock
Converts an f64 value to Decimal with error propagation.

This macro converts an f64 floating-point value to a Decimal type. It
propagates any errors that might occur during conversion using the `?`
operator.

## [§](#parameters){.doc-anchor}Parameters

- `$val` - An f64 value to be converted to Decimal
:::
::::::
:::::::
