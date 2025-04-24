:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [assert_pos_relative_eq]{.macro}Copy item path

[[Source](../src/optionstratlib/utils/tests.rs.html#45-81){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! assert_pos_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => { ... };
}
```

Expand description

:::: docblock
Asserts that two `Positive` values are relatively equal within a given
epsilon.

This macro compares two `Positive` values and checks if their relative
difference is within the specified `epsilon`. It handles cases where one
or both values are zero, ensuring that the non-zero value is less than
or equal to epsilon.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::Positive;
use optionstratlib::{assert_pos_relative_eq, pos};

let a = pos!(1.0);
let b = pos!(1.0001);
let epsilon = pos!(0.001);
assert_pos_relative_eq!(a, b, epsilon); // Passes

let c = pos!(1.0);
let d = pos!(2.0);
let epsilon = pos!(0.001);
#[test]
#[should_panic]
assert_pos_relative_eq!(c, d, epsilon); // Panics

let e = pos!(0.0);
let f = pos!(0.0001);
let epsilon = pos!(0.001);
assert_pos_relative_eq!(e, f, epsilon); // Passes

let g = pos!(0.0);
let h = pos!(0.0011);
let epsilon = pos!(0.001);
#[test]
#[should_panic]
assert_pos_relative_eq!(g, h, epsilon); // Panics
```
:::

## [§](#panics){.doc-anchor}Panics

This macro panics if the relative difference between the two values is
greater than the specified `epsilon`, or if one value is zero and the
other is greater than epsilon. The panic message includes the values
being compared, the expected relative difference, and the actual
relative difference.
::::
:::::::
::::::::
