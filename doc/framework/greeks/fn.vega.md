::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [vega]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#755-780){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn vega(option: &Options) -> Result<Decimal, GreeksError>
```

Expand description

::::: docblock
Computes the vega of an option.

Vega measures the sensitivity of the option's price to changes in the
implied volatility of the underlying asset. It quantifies the expected
change in the option's price for a 1% change in the implied volatility.
Vega is particularly important for understanding how an option's value
is affected by market conditions that alter volatility.

## [§](#parameters){.doc-anchor}Parameters

- `option: &Options` A reference to an `Options` struct containing the
  necessary parameters:
  - `underlying_price`: The current price of the underlying asset.
  - `strike_price`: The strike price of the option.
  - `risk_free_rate`: The annualized risk-free interest rate.
  - `expiration_date`: The time to expiration in years (provides
    `get_years` method).
  - `implied_volatility`: The implied volatility of the option.
  - `dividend_yield`: The dividend yield of the underlying asset.
  - `quantity`: The quantity of the options.
  - `option_style`: The style of the option (e.g., European).

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The computed vega value of the option.
- `Err(GreeksError)`: Returns an error if any intermediate calculation
  fails (e.g., in `d1` or `big_n`).

## [§](#formula){.doc-anchor}Formula

Vega is computed using the Black-Scholes model formula:

::: example-wrap
``` language-math
\text{Vega} = S \cdot e^{-qT} \cdot n(d1) \cdot \sqrt{T}
```
:::

Where:

- ( S ): The price of the underlying asset.
- ( q ): The dividend yield of the underlying asset.
- ( T ): Time to expiration in years.
- ( n(d1) ): The probability density function (PDF) of the standard
  normal distribution at ( d1 ).
- ( d1 ): A parameter calculated using the Black-Scholes model.

## [§](#calculation-steps){.doc-anchor}Calculation Steps

1.  Compute ( d1 ) using the `d1` function.
2.  Calculate the exponential factor ( e\^{-qT} ), which accounts for
    the effect of dividends.
3.  Evaluate ( n(d1) ), the PDF of the standard normal distribution at (
    d1 ).
4.  Multiply the underlying price, the exponential factor, ( n(d1) ),
    and the square root of time to expiration.
5.  Multiply the result by the quantity of options to adjust for
    position size.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::vega;
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

match vega(&option) {
    Ok(result) => info!("Vega: {}", result),
    Err(e) => error!("Error calculating Vega: {:?}", e),
}
```
:::

## [§](#notes){.doc-anchor}Notes

- Vega is usually highest for at-the-money options and decreases as the
  option moves deeper in-the-money or out-of-the-money.
- For shorter time to expiration, Vega is smaller as the sensitivity to
  volatility diminishes.
- A positive Vega indicates that an increase in implied volatility will
  increase the option's value.
:::::
::::::::
:::::::::
