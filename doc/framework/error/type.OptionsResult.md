::::::::::::: width-limiter
:::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[error](index.html)
:::

# Type Alias [OptionsResult]{.type}Copy item path

[[Source](../../src/optionstratlib/error/options.rs.html#242){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type OptionsResult<T> = Result<T, OptionsError>;
```

Expand description

:::: docblock
A specialized result type for operations related to Options calculations
and processing.

This type alias simplifies error handling for functions that can fail
with various options-specific errors. It uses the
[`OptionsError`](enum.OptionsError.html) enum to provide structured
error information about validation failures, pricing issues, Greeks
calculations, time-related problems, and other option-specific errors.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `T` - The success type that will be returned if the operation
  succeeds.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::{OptionsResult, OptionsError};

fn calculate_call_price(strike: f64, spot: f64) -> OptionsResult<f64> {
    if strike <= 0.0 {
        return Err(OptionsError::ValidationError {
            field: "strike".to_string(),
            reason: "Strike price must be positive".to_string()
        });
    }
     
    // Calculation logic would go here
    let price = 0.0; // Placeholder
    Ok(price)
}
```
:::

## [§](#usage-context){.doc-anchor}Usage Context

This result type is commonly used throughout the library for:

- Option pricing calculations
- Parameter validation
- Greeks calculations
- Expiration and time value calculations
- Option payoff analysis
::::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
enum OptionsResult<T> {
    Ok(T),
    Err(OptionsError),
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

### Err([OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) {#erroptionserror .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
::::::::::::
:::::::::::::
