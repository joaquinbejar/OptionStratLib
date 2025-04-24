:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [spos]{.macro}Copy item path

[[Source](../src/optionstratlib/model/positive.rs.html#83-87){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! spos {
    ($val:expr) => { ... };
}
```

Expand description

:::: docblock
Macro for creating an `Option<Positive>` value with simplified syntax.

This macro attempts to create a `Positive` value from the given
expression, unwraps the result, and wraps it in `Some()`. It will panic
if the value cannot be converted to a `Positive` (e.g., if the value is
negative or not representable).

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::spos;
let optional_positive = spos!(5.0); // Some(Positive(5.0))
```
:::

## [§](#panics){.doc-anchor}Panics

This macro will panic if the provided value cannot be converted to a
`Positive` value.
::::
:::::::
::::::::
