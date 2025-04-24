:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[others](index.html)
:::

# Function [approx_equal]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/others.rs.html#45-47){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn approx_equal(a: f64, b: f64) -> bool
```

Expand description

:::: docblock
Checks for approximate equality between two f64 values within a defined
tolerance.

This function compares two floating-point numbers and returns `true` if
the absolute difference between them is less than the predefined
`TOLERANCE` constant. It is useful for comparing floating-point values
that may be subject to small rounding errors.

## [§](#arguments){.doc-anchor}Arguments

- `a` - The first f64 value to compare.
- `b` - The second f64 value to compare.

## [§](#returns){.doc-anchor}Returns

`true` if the absolute difference between `a` and `b` is less than
`TOLERANCE`, `false` otherwise.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::utils::others::approx_equal;

let x = 1.0;
let y = 1.00000001;
assert!(approx_equal(x, y)); // Returns true

let x = 1.0;
let y = 1.1;
assert!(!approx_equal(x, y)); // Returns false
```
:::
::::
:::::::
::::::::
