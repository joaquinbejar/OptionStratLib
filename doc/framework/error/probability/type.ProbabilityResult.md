:::::::::::: width-limiter
::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[probability](index.html)
:::

# Type Alias [ProbabilityResult]{.type}Copy item path

[[Source](../../../src/optionstratlib/error/probability.rs.html#446){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type ProbabilityResult<T> = Result<T, ProbabilityError>;
```

Expand description

::: docblock
Convenient type alias for Results with ProbabilityError
:::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
enum ProbabilityResult<T> {
    Ok(T),
    Err(ProbabilityError),
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

### Err([ProbabilityError](enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}) {#errprobabilityerror .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
:::::::::::
::::::::::::
