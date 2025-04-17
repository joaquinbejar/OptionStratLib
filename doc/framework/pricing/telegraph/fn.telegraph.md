::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)::[telegraph](index.html)
:::

# Function [telegraph]{.fn}Copy item path

[[Source](../../../src/optionstratlib/pricing/telegraph.rs.html#280-331){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn telegraph(
    option: &Options,
    no_steps: usize,
    lambda_up: Option<Decimal>,
    lambda_down: Option<Decimal>,
) -> Result<Decimal, Box<dyn Error>>
```

Expand description

::: docblock
Prices an option using the Telegraph process simulation method.

This function simulates the underlying asset price movement using a
Telegraph process, which oscillates between two states, affecting the
volatility of the price path. It provides a more sophisticated model
than standard geometric Brownian motion by capturing regime-switching
behavior in market volatility.

## [§](#arguments){.doc-anchor}Arguments

- `option` - Reference to the Options structure containing all option
  parameters
- `no_steps` - Number of time steps for the simulation
- `lambda_up` - Optional transition rate from down state (-1) to up
  state (+1)
- `lambda_down` - Optional transition rate from up state (+1) to down
  state (-1)

## [§](#returns){.doc-anchor}Returns

- `Result<Decimal, Box<dyn Error>>` - The simulated option price or an
  error

## [§](#details){.doc-anchor}Details

The function handles parameter estimation automatically if transition
rates are not provided. When missing, it simulates returns based on the
option's implied volatility to estimate appropriate telegraph
parameters.
:::
::::::
:::::::
