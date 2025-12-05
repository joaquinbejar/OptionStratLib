::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[chains](index.html)
:::

# Function [generator_optionchain]{.fn} Copy item path

[[Source](../../src/optionstratlib/chains/generators.rs.html#70-172){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn generator_optionchain(
    walk_params: &WalkParams<Positive, OptionChain>,
) -> Vec<Step<Positive, OptionChain>>
```

Expand description

::: docblock
Generates a vector of `Step`s containing `Positive` x-values and
`OptionChain` y-values.

This function simulates a geometric Brownian motion walk for option
chains, generating a sequence of steps with updated option chains based
on the changing underlying price. It uses a fixed volatility of 0.20.

## [§](#arguments){.doc-anchor}Arguments

- `walk_params` - A reference to the `WalkParams` struct containing the
  walk parameters.

## [§](#returns){.doc-anchor}Returns

- `Vec<Step<Positive, OptionChain>>` - A vector of `Step`s representing
  the simulated walk.
:::
::::::
:::::::
