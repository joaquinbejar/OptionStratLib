:::::::::: width-limiter
::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [rho_d]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#1014-1044){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn rho_d(option: &Options) -> Result<Decimal, GreeksError>
```

Expand description

:::::: docblock
Computes the sensitivity of the option price to changes in the dividend
yield (Rho_d).

This function calculates how the price of an option changes with respect
to variations in the dividend yield of the underlying asset. This
metric, often referred to as "dividend rho", is essential for
understanding the impact of dividends on the option's value.

## [§](#parameters){.doc-anchor}Parameters

- `option: &Options` A reference to an `Options` struct containing the
  following relevant fields:
  - `underlying_price`: The current price of the underlying asset.
  - `strike_price`: The strike price of the option.
  - `risk_free_rate`: The risk-free interest rate.
  - `expiration_date`: The time to expiration in years (provides
    `get_years` method).
  - `implied_volatility`: The implied volatility of the option.
  - `dividend_yield`: The dividend yield of the underlying asset.
  - `quantity`: The quantity of the options.
  - `option_style`: The style of the option (`Call` or `Put`).

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The calculated dividend sensitivity (`Rho_d`) value for
  the options contract.
- `Err(GreeksError)`: Returns an error if any intermediate calculation
  fails (e.g., in `d1` or `big_n`).

## [§](#formula){.doc-anchor}Formula

The dividend sensitivity is calculated differently for Call and Put
options:

**Call Options:**

::: example-wrap
``` language-math
\rho_d^{\text{call}} = -T \cdot S \cdot e^{-qT} \cdot N(d1)
```
:::

**Put Options:**

::: example-wrap
``` language-math
\rho_d^{\text{put}} = T \cdot S \cdot e^{-qT} \cdot N(-d1)
```
:::

Where:

- ( T ): Time to expiration (in years).
- ( S ): Price of the underlying asset.
- ( q ): Dividend yield.
- ( N(d1) ): The cumulative distribution function (CDF) of the standard
  normal distribution evaluated at ( d1 ).
- ( d1 ): A parameter calculated using the Black-Scholes model.

## [§](#calculation-steps){.doc-anchor}Calculation Steps

1.  Compute ( d1 ) using the `d1` function.
2.  Evaluate the exponential factor ( e\^{-qT} ), which accounts for the
    dividend yield.
3.  Calculate ( N(d1) ) or ( N(-d1) ), depending on the option style.
4.  Use the appropriate formula for Call or Put options.
5.  Multiply the result by the option's quantity to adjust for position
    size.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::rho_d;
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

match rho_d(&option) {
    Ok(result) => info!("Dividend Rho (Rho_d): {}", result),
    Err(e) => error!("Error calculating Rho_d: {:?}", e),
}
```
:::

## [§](#notes){.doc-anchor}Notes

- **Call Options**: A higher dividend yield decreases the price of the
  call option, leading to a negative dividend sensitivity.
- **Put Options**: A higher dividend yield increases the price of the
  put option, leading to a positive dividend sensitivity.
- This calculation assumes that dividends are continuously compounded at
  the dividend yield rate.
- ( Rho_d ) is generally more significant for options with longer times
  to expiration.
::::::
:::::::::
::::::::::
