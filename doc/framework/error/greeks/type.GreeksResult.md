:::::::::::: width-limiter
::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[greeks](index.html)
:::

# Type Alias [GreeksResult]{.type} Copy item path

[[Source](../../../src/optionstratlib/error/greeks.rs.html#344){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type GreeksResult<T> = Result<T, GreeksError>;
```

Expand description

::: docblock
Type alias for Results returned from Greek calculation functions.

This alias wraps the standard Rust `Result` type to provide a
specialized result type for Greek calculations, using `GreeksError` as
the error type.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `T` - The success value type that will be returned when operations
  succeed.

## [§](#related-types){.doc-anchor}Related Types

This type alias is part of the error handling system for Greek
calculations and works with the `GreeksError` enum which provides
detailed error information.

## [§](#usage-context){.doc-anchor}Usage Context

Typically used in functions that calculate option Greeks (delta, gamma,
theta, vega, rho) and other financial metrics where specialized error
handling for mathematical and input validation errors is needed.
:::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
pub enum GreeksResult<T> {
    Ok(T),
    Err(GreeksError),
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

### Err([GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) {#errgreekserror .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
:::::::::::
::::::::::::
