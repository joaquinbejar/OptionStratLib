::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)::[black_scholes_model](index.html)
:::

# Function [black_scholes]{.fn} Copy item path

[[Source](../../../src/optionstratlib/pricing/black_scholes_model.rs.html#38-57){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn black_scholes(option: &Options) -> Result<Decimal, PricingError>
```

Expand description

::: docblock
Computes the price of an option using the Black-Scholes model.

## [§](#arguments){.doc-anchor}Arguments

- `option`: An `Options` struct containing all the relevant parameters
  for the option (e.g., strike price, underlying asset price, etc.).
- `time_to_expiry`: An optional `f64` representing the time until the
  option's expiration in years.

## [§](#returns){.doc-anchor}Returns

A `f64` representing the calculated price of the option.

## [§](#description){.doc-anchor}Description

This function leverages the Black-Scholes model to determine the price
of either a call option or a put option. It first calculates the `d1`
and `d2` parameters required for the model and then matches the
`option_style` to use the appropriate pricing formula for call or put
options.

The function calls helper functions:

- `calculate_d1_d2_and_time()`: Computes the necessary `d1`, `d2`, and
  expiry time.
- `calculate_call_option_price()`: Computes the price of a call option.
- `calculate_put_option_price()`: Computes the price of a put option.

The function returns the computed price based on the type of option
provided.
:::
::::::
:::::::
