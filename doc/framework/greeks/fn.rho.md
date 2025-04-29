:::::::::: width-limiter
::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [rho]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#879-918){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn rho(option: &Options) -> Result<Decimal, GreeksError>
```

Expand description

:::::: docblock
Computes the rho of an options contract.

Rho measures the sensitivity of the option's price to changes in the
risk-free interest rate. It quantifies the expected change in the
option's price for a 1% change in the risk-free rate. This metric is
useful for understanding how interest rate fluctuations affect the value
of options contracts.

## [§](#parameters){.doc-anchor}Parameters

- `option: &Options` A reference to an `Options` struct containing the
  following fields:
  - `underlying_price`: The current price of the underlying asset.
  - `strike_price`: The strike price of the option.
  - `risk_free_rate`: The annualized risk-free interest rate.
  - `expiration_date`: The time to expiration in years (provides
    `get_years` method).
  - `implied_volatility`: The implied volatility of the option.
  - `option_style`: The style of the option (`Call` or `Put`).
  - `quantity`: The quantity of the options.

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The computed rho value for the options contract.
- `Err(GreeksError)`: Returns an error if any intermediate calculation
  fails (e.g., in `d2` or `big_n`).

## [§](#formula){.doc-anchor}Formula

The rho is calculated differently for Call and Put options, as follows:

**Call Options:**

::: example-wrap
``` language-math
\rho_{\text{call}} = K \cdot T \cdot e^{-rT} \cdot N(d2)
```
:::

**Put Options:**

::: example-wrap
``` language-math
\rho_{\text{put}} = -K \cdot T \cdot e^{-rT} \cdot N(-d2)
```
:::

Where:

- ( K ): The strike price of the option.
- ( T ): The time to expiration (in years).
- ( r ): The risk-free interest rate.
- ( N(d2) ): The cumulative distribution function (CDF) of the standard
  normal distribution evaluated at ( d2 ).
- ( e\^{-rT} ): The discount factor for the risk-free rate.

## [§](#calculation-steps){.doc-anchor}Calculation Steps

1.  Compute ( d2 ) using the `d2` function.
2.  Calculate the discount factor ( e\^{-rT} ).
3.  Evaluate ( N(d2) ) or ( N(-d2) ), depending on the option style.
4.  Multiply the strike price, time to expiration, discount factor, and
    ( N(d2) ) or ( N(-d2) ).
5.  Multiply the result by the option's quantity.

## [§](#edge-cases){.doc-anchor}Edge Cases

- If the discount factor (( e\^{-rT} )) is zero, the rho is returned as
  zero.
- If ( N(d2) ) or ( N(-d2) ) is zero, the rho is returned as zero.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::rho;
use optionstratlib::Options;
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::pos;

let option = Options {
    option_type: OptionType::European,
    side: Side::Long,
    underlying_price: pos!(100.0),
    strike_price: pos!(95.0),
    risk_free_rate: dec!(0.05),
    expiration_date: ExpirationDate::Days(pos!(30.0)),
    implied_volatility: pos!(0.2),
    dividend_yield: pos!(0.01),
    quantity: pos!(1.0),
    option_style: OptionStyle::Call,
    underlying_symbol: "".to_string(),
    exotic_params: None,
};

match rho(&option) {
    Ok(result) => info!("Rho: {}", result),
    Err(e) => error!("Error calculating rho: {:?}", e),
}
```
:::

## [§](#notes){.doc-anchor}Notes

- Rho is typically higher for options with longer time to expiration, as
  they are more sensitive to changes in the risk-free rate.
- Call options have positive rho values, as an increase in interest
  rates increases their value.
- Put options have negative rho values, as an increase in interest rates
  decreases their value.
::::::
:::::::::
::::::::::
