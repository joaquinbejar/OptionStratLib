:::::::::: width-limiter
::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [theta]{.fn} Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#651-696){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn theta(option: &Options) -> Result<Decimal, GreeksError>
```

Expand description

:::::: docblock
Computes the Theta of an option.

Theta measures the sensitivity of the option's price to time decay,
indicating the rate at which the value of the option decreases as the
expiration date approaches. This is particularly important in options
trading, as Theta reflects the "time decay" of the option's extrinsic
value.

## [§](#parameters){.doc-anchor}Parameters

- `option: &Options` A reference to an `Options` struct containing the
  following relevant parameters:
  - `underlying_price`: The current price of the underlying asset.
  - `strike_price`: The strike price of the option.
  - `risk_free_rate`: The risk-free interest rate.
  - `expiration_date`: The time to expiration in years (provides
    `get_years` method).
  - `implied_volatility`: The implied volatility of the option.
  - `dividend_yield`: The dividend yield of the underlying asset.
  - `option_style`: The style of the option (Call or Put).
  - `quantity`: The quantity of the options.

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The calculated Theta value for the option.
- `Err(GreeksError)`: Returns an error if any intermediate calculation
  fails (e.g., in `d1`, `d2`, or `n`).

## [§](#formula){.doc-anchor}Formula

The Theta is calculated using the Black-Scholes model. The formula
differs for call and put options:

**Call Options:**

::: example-wrap
``` language-math
\Theta_{\text{call}} =
-\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
- r \cdot K \cdot e^{-rT} \cdot N(d2)
+ q \cdot S \cdot e^{-qT} \cdot N(d1)
```
:::

**Put Options:**

::: example-wrap
``` language-math
\Theta_{\text{put}} =
-\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
+ r \cdot K \cdot e^{-rT} \cdot N(-d2)
- q \cdot S \cdot e^{-qT} \cdot N(-d1)
```
:::

Where:

- ( S ): Underlying price
- ( \\sigma ): Implied volatility
- ( T ): Time to expiration (in years)
- ( r ): Risk-free rate
- ( q ): Dividend yield
- ( K ): Strike price
- ( N(d1) ): Cumulative distribution function (CDF) of the standard
  normal distribution at ( d1 ).
- ( n(d1) ): Probability density function (PDF) of the standard normal
  distribution at ( d1 ).

## [§](#calculation-steps){.doc-anchor}Calculation Steps

1.  Compute ( d1 ) and ( d2 ) using the `d1` and `d2` functions.
2.  Calculate the common term:

    ::: example-wrap
    ``` language-math
    \text{common\_term} = -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
    ```
    :::
3.  Apply the corresponding formula for Call or Put options, accounting
    for the effect of dividends (( e\^{-qT} )) and risk-free rate ((
    e\^{-rT} )).
4.  Multiply the resulting Theta by the quantity of options.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::theta;
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::pos_or_panic;
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

match theta(&option) {
    Ok(result) => info!("Theta: {}", result),
    Err(e) => error!("Error calculating Theta: {:?}", e),
}
```
:::

## [§](#notes){.doc-anchor}Notes

- A positive Theta means the option gains value as time passes (rare and
  usually for short positions).
- A negative Theta is typical for long positions, as the option loses
  extrinsic value over time.
- If the implied volatility is zero, Theta may be close to zero for
  far-out-of-the-money options.
::::::
:::::::::
::::::::::
