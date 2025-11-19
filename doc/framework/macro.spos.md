:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [spos]{.macro} Copy item path

[[Source](../src/optionstratlib/model/positive.rs.html#107-109){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! spos {
    ($val:expr) => { ... };
}
```

Expand description

:::: docblock
A macro to create an optional `Positive` value from the given
expression.

This macro attempts to create a `Positive` value using the
`Positive::new` function, which returns `None` if the value is not a
positive number. The macro is useful for safely constructing optional
`Positive` values in a concise manner.

## [§](#parameters){.doc-anchor}Parameters

- `$val:expr`: An expression that evaluates to a value intended to be
  wrapped in a `Positive` type.

## [§](#returns){.doc-anchor}Returns

- `Option<Positive>`: Returns `Some(Positive)` if the given value is
  positive, otherwise returns `None`.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::Positive;
use optionstratlib::spos;

// Example with a positive value
let x = spos!(10.0);
assert_eq!(x.is_some(), true);

// Example with a non-positive value
let y = spos!(-5.0);
assert_eq!(y.is_none(), true);
```
:::

## [§](#notes){.doc-anchor}Notes

- Ensure that the type used with this macro implements the required
  constraints for constructing a `Positive` value (as defined in the
  `Positive::new` method).

## [§](#see-also){.doc-anchor}See Also

- [`Positive::new`](model/positive/struct.Positive.html#method.new "associated function optionstratlib::model::positive::Positive::new"):
  The method that performs the actual validation for determining whether
  a value can be wrapped in the `Positive` type.
::::
:::::::
::::::::
