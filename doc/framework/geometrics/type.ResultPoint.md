:::::::::::: width-limiter
::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Type Alias [ResultPoint]{.type}Copy item path

[[Source](../../src/optionstratlib/geometrics/construction/types.rs.html#15){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub type ResultPoint<Point> = Result<Point, Box<dyn Error>>;
```

Expand description

::: docblock
A result type for geometric point operations that may fail.

This type alias provides a consistent way to handle point generation
operations that could result in errors, encapsulating the resulting
point or an error.
:::

## Aliased Type[§](#aliased-type){.anchor} {#aliased-type .section-header}

``` {.rust .item-decl}
enum ResultPoint<Point> {
    Ok(Point),
    Err(Box<dyn Error>),
}
```

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

::::::: variants
::: {#variant.Ok .section .variant}
[§](#variant.Ok){.anchor}[1.0.0]{.since .rightside
title="Stable since Rust version 1.0.0"}

### Ok(Point) {#okpoint .code-header}
:::

::: docblock
Contains the success value
:::

::: {#variant.Err .section .variant}
[§](#variant.Err){.anchor}[1.0.0]{.since .rightside
title="Stable since Rust version 1.0.0"}

### Err([Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>) {#errboxdyn-error .code-header}
:::

::: docblock
Contains the error value
:::
:::::::
:::::::::::
::::::::::::
