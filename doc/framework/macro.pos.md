:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [pos]{.macro}Copy item path

[[Source](../src/optionstratlib/model/positive.rs.html#59-63){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! pos {
    ($val:expr) => { ... };
}
```

Expand description

:::: docblock
Macro for creating a new `Positive` value with simplified syntax.

This macro attempts to create a `Positive` value from the given
expression and unwraps the result. It will panic if the value cannot be
converted to a `Positive` (e.g., if the value is negative or not
representable).

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::pos;
let positive_value = pos!(5.0);
```
:::

## [§](#panics){.doc-anchor}Panics

This macro will panic if the provided value cannot be converted to a
`Positive` value.
::::
:::::::
::::::::
