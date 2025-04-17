::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [d2]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/utils.rs.html#197-227){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn d2(
    underlying_price: Positive,
    strike_price: Positive,
    risk_free_rate: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError>
```

Expand description

::::: docblock
Calculates the `d2` parameter used in the Black-Scholes options pricing
model.

The `d2` value is an intermediary result derived from the `d1` value and
is used to determine option greeks and prices. It is computed using the
formula:

::: example-wrap
``` language-math
d2 = d1 - σ * sqrt(T)
```
:::

Where:

- `d1`: The `d1` value calculated using the `d1` function.
- `σ`: Implied volatility.
- `T`: Time to expiration (in years).

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

- `Ok(Decimal)`: The computed `d2` value.
- `Err(GreeksError)`: Returns an error if input validation fails or if
  the `d1` computation fails.

## [§](#errors){.doc-anchor}Errors

Returns a `GreeksError::InputError` in the following cases:

- **InvalidVolatility**: Triggered when `implied_volatility` is zero.
- **InvalidTime**: Triggered when `expiration_date` is zero.

## [§](#notes){.doc-anchor}Notes

This function depends on the `d1` function to compute the `d1` value.
Any errors from the `d1` function will propagate to this function.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::d2;
use optionstratlib::{pos, Positive};
let underlying_price = Positive::new(100.0).unwrap();
let strike_price = Positive::new(95.0).unwrap();
let risk_free_rate = dec!(0.05);
let expiration_date = pos!(0.5); // 6 months
let implied_volatility = pos!(0.2);

match d2(
    underlying_price,
    strike_price,
    risk_free_rate,
    expiration_date,
    implied_volatility,
) {
    Ok(result) => info!("d2: {}", result),
    Err(e) => error!("Error: {:?}", e),
}
```
:::
:::::
::::::::
:::::::::
