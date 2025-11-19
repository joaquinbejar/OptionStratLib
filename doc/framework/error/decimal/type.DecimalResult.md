::::::::::::: width-limiter
:::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[decimal](index.html)
:::

# Type Alias [DecimalResult]{.type} Copy item path

[[Source](../../../src/optionstratlib/error/decimal.rs.html#169){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type DecimalResult<T> = Result<T, DecimalError>;
```

Expand description

:::: docblock
A specialized `Result` type for decimal calculation operations.

This type alias provides a convenient shorthand for operations that can
result in a `DecimalError`. It helps improve code readability and
reduces boilerplate when working with decimal calculations throughout
the library.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `T` - The successful result type of the operation

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::{DecimalError, DecimalResult};

fn divide(a: f64, b: f64) -> DecimalResult<f64> {
    if b == 0.0 {
        Err(DecimalError::ArithmeticError {
            operation: "division".to_string(),
            reason: "division by zero".to_string(),
        })
    } else {
        Ok(a / b)
    }
}
```
:::

## [§](#usage-context){.doc-anchor}Usage Context

This type is primarily used in the financial calculations and decimal
handling components of the library, where precise decimal operations are
critical and error handling needs to be consistent and well-structured.

## [§](#related-types){.doc-anchor}Related Types

- `DecimalError` - The error type representing various decimal operation
  failures
::::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
pub enum DecimalResult<T> {
    Ok(T),
    Err(DecimalError),
}
```

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

::::::: variants
::: {#variant.Ok .section .variant}
[§](#variant.Ok){.anchor}[1.0.0]{.since .rightside
title="Stable since Rust version 1.0.0"}

### Ok(T) {#okt .code-header}
:::

::: docblock
Contains the success value
:::

::: {#variant.Err .section .variant}
[§](#variant.Err){.anchor}[1.0.0]{.since .rightside
title="Stable since Rust version 1.0.0"}

### Err([DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}) {#errdecimalerror .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
::::::::::::
:::::::::::::
