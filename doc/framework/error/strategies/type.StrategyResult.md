:::::::::::: width-limiter
::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[strategies](index.html)
:::

# Type Alias [StrategyResult]{.type} Copy item path

[[Source](../../../src/optionstratlib/error/strategies.rs.html#277){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type StrategyResult<T> = Result<T, StrategyError>;
```

Expand description

::: docblock
A specialized result type for strategy operations.

This type alias provides a convenient way to handle results from
strategy-related operations that might fail with a `StrategyError`. It
follows the standard Rust pattern of using `Result<T, E>` for operations
that can fail.

This makes error handling more readable and concise throughout the
strategy-related code, compared to explicitly writing
`Result<T, StrategyError>` everywhere.
:::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
pub enum StrategyResult<T> {
    Ok(T),
    Err(StrategyError),
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

### Err([StrategyError](enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}) {#errstrategyerror .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
:::::::::::
::::::::::::
