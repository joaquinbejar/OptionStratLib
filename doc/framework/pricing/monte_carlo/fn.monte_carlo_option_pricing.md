::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)::[monte_carlo](index.html)
:::

# Function [monte_carlo_option_pricing]{.fn} Copy item path

[[Source](../../../src/optionstratlib/pricing/monte_carlo.rs.html#34-60){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn monte_carlo_option_pricing(
    option: &Options,
    steps: usize,
    simulations: usize,
) -> Result<Decimal, PricingError>
```

Expand description

::: docblock
This function performs Monte Carlo simulation to price an option.

## [§](#arguments){.doc-anchor}Arguments

- `option` - An `Options` struct containing the option's parameters,
  such as underlying price, strike price, risk-free rate, implied
  volatility, and expiration date.
- `steps` - An integer indicating the number of time steps in the
  simulation.
- `simulations` - An integer indicating the number of Monte Carlo
  simulations to run.

## [§](#returns){.doc-anchor}Returns

- A floating-point number representing the estimated price of the
  option.

## [§](#description){.doc-anchor}Description

The function follows the below steps:

1.  Calculate the time increment `dt` based on the number of steps.
2.  Initialize a sum variable `payoff_sum` to accumulate the payoffs
    from each simulation.
3.  Loop through the number of simulations:
    - For each simulation, initialize the stock price `st` to the
      underlying price.
    - Loop through the number of steps:
      - Calculate a Wiener process increment `w`.
      - Update the stock price `st` using the discrete approximation of
        the geometric Brownian motion model.
    - Calculate the payoff of the option for this simulation (for a call
      option, this is `max(st - strike_price, 0)`).
    - Add the payoff to the `payoff_sum`.
4.  Return the average payoff discounted to its present value.
:::
::::::
:::::::
