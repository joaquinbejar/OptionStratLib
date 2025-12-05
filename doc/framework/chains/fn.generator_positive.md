::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[chains](index.html)
:::

# Function [generator_positive]{.fn} Copy item path

[[Source](../../src/optionstratlib/chains/generators.rs.html#187-227){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn generator_positive(
    walk_params: &WalkParams<Positive, Positive>,
) -> Vec<Step<Positive, Positive>>
```

Expand description

::: docblock
Generates a vector of `Step`s containing `Positive` x-values and
`Positive` y-values.

This function simulates a geometric Brownian motion walk for positive
values, generating a sequence of steps with updated positive values.

## [§](#arguments){.doc-anchor}Arguments

- `walk_params` - A reference to the `WalkParams` struct containing the
  walk parameters.

## [§](#returns){.doc-anchor}Returns

- `Vec<Step<Positive, Positive>>` - A vector of `Step`s representing the
  simulated walk.
:::
::::::
:::::::
