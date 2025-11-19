::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[positive](index.html)
:::

# Function [is_positive]{.fn} Copy item path

[[Source](../../../src/optionstratlib/model/positive.rs.html#170-172){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn is_positive<T: 'static>() -> bool
```

Expand description

::::: docblock
Determines if the given type parameter `T` is the `Positive` type.

## [§](#details){.doc-anchor}Details

The function `is_positive` is a utility method that checks if the
generic type parameter matches the type `Positive`. Internally, it uses
Rust's `TypeId` comparison from the `std::any` module for this check,
which is a reliable way to confirm whether two types are the same at
runtime.

The `Positive` type is part of the `crate::model::positive` module and
is a specialized struct used to represent non-negative decimal values.
It is implemented as follows:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;

#[derive(PartialEq, Clone, Copy)]
pub struct Positive(pub(crate) Decimal);
```
:::

This type enforces semantic clarity and helps avoid invalid numerical
operations, making it especially useful in financial modeling or domains
where valid numbers are constrained to non-negative values.

## [§](#usage){.doc-anchor}Usage

- Typically used in scenarios where type safety for numerical values
  like non-negativity is critical.
- Prevents misuse of data structures or operations by identifying
  whether a value is of the `Positive` type.

## [§](#implementation){.doc-anchor}Implementation

The function relies on the `TypeId` mechanism, part of Rust's type
reflection system, which allows runtime comparison of types. This is
efficient and avoids adding unnecessary overhead.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::model::positive::is_positive;
use optionstratlib::Positive;
assert!(is_positive::<Positive>());
assert!(!is_positive::<i32>());
```
:::

This example demonstrates that the function correctly identifies the
`Positive` type while distinguishing it from unrelated types like `i32`.

## [§](#note){.doc-anchor}Note

This function does **not** perform runtime checks on instances of
values, nor does it check type compatibility with conversions. It works
exclusively at the type level.

## [§](#see-also){.doc-anchor}See Also

- [`Positive`](struct.Positive.html "struct optionstratlib::model::positive::Positive")
  type, which encapsulates non-negative decimal values.
- `std::any::TypeId` for type reflection and runtime type
  identification.
:::::
::::::::
:::::::::
