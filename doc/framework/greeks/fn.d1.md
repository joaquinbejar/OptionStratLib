::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [d1]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/utils.rs.html#82-132){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn d1(
    underlying_price: Positive,
    strike_price: Positive,
    risk_free_rate: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError>
```

Expand description

::::: docblock
Calculates the `d1` parameter used in the Black-Scholes options pricing
model.

The `d1` value is an intermediary result used to determine option greeks
and prices. It is computed using the formula:

::: example-wrap
``` language-math
d1 = (ln(S / K) + (r + σ² / 2) * T) / (σ * sqrt(T))
```
:::

Where:

- `S`: Underlying price
- `K`: Strike price
- `r`: Risk-free rate
- `T`: Time to expiration (in years)
- `σ`: Implied volatility

## [§](#parameters){.doc-anchor}Parameters

- `underlying_price`: The current price of the underlying asset. Must be
  positive.
- `strike_price`: The strike price of the option. Must be greater than
  zero.
- `risk_free_rate`: The annual risk-free interest rate, expressed as a
  decimal.
- `expiration_date`: The time to expiration of the option, in years.
  Must be greater than zero.
- `implied_volatility`: The implied volatility of the option, expressed
  as a decimal. Must be greater than zero.

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The computed `d1` value.
- `Err(GreeksError)`: Returns an error if input validation fails.
  Possible errors include:
  - Invalid strike price (must be greater than zero).
  - Invalid implied volatility (must be greater than zero).
  - Invalid expiration time (must be greater than zero).

## [§](#errors){.doc-anchor}Errors

Returns a `GreeksError::InputError` in the following cases:

- **InvalidStrike**: Triggered when `strike_price` is zero or less.
- **InvalidVolatility**: Triggered when `implied_volatility` is zero.
- **InvalidTime**: Triggered when `expiration_date` is zero or less.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::d1;
use optionstratlib::{pos, Positive};

let underlying_price = pos!(100.0);
let strike_price = pos!(95.0);
let risk_free_rate = dec!(0.05);
let expiration_date = pos!(0.5); // 6 months
let implied_volatility = pos!(0.2);

match d1(
    underlying_price,
    strike_price,
    risk_free_rate,
    expiration_date,
    implied_volatility,
) {
    Ok(result) => info!("d1: {}", result),
    Err(e) => error!("Error: {:?}", e),
}
```
:::
:::::
::::::::
:::::::::
