:::::::::::: width-limiter
::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[curves](index.html)
:::

# Type Alias [CurvesResult]{.type}Copy item path

[[Source](../../../src/optionstratlib/error/curves.rs.html#226){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type CurvesResult<T> = Result<T, CurveError>;
```

Expand description

::: docblock
Type alias representing the result of operations related to curve
calculations.

This type alias provides a standardized result type for functions that
perform operations with mathematical curves, including interpolation,
construction, analysis, and other curve-related operations.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `T` - The success value type returned when operations complete
  successfully.

## [§](#return-value){.doc-anchor}Return Value

Returns either:

- `Ok(T)` - The operation completed successfully with a value of type
  `T`.
- `Err(CurveError)` - The operation failed, with a
  [`CurveError`](enum.CurveError.html "enum optionstratlib::error::curves::CurveError")
  describing the specific failure.

## [§](#usage){.doc-anchor}Usage

This result type is used throughout the curves module to provide
consistent error handling for curve operations. It allows functions to
return detailed error information using the
[`CurveError`](enum.CurveError.html "enum optionstratlib::error::curves::CurveError")
enum when operations fail, while returning the expected value when
successful.
:::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
enum CurvesResult<T> {
    Ok(T),
    Err(CurveError),
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

### Err([CurveError](enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}) {#errcurveerror .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
:::::::::::
::::::::::::
