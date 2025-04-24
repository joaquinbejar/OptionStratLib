::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [gamma]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#492-519){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn gamma(option: &Options) -> Result<Decimal, GreeksError>
```

Expand description

::::: docblock
Computes the gamma of an option.

Gamma measures the rate of change of the option's delta with respect to
changes in the underlying asset's price. It is a second-order derivative
of the option price and provides insight into the sensitivity of delta
to movements in the underlying price.

## [§](#parameters){.doc-anchor}Parameters

- `option: &Options` A reference to an `Options` struct containing the
  following relevant parameters:
  - `underlying_price`: The current price of the underlying asset.
  - `strike_price`: The strike price of the option.
  - `risk_free_rate`: The risk-free interest rate.
  - `expiration_date`: The time to expiration in years.
  - `implied_volatility`: The implied volatility of the option.
  - `dividend_yield`: The dividend yield of the underlying asset.
  - `quantity`: The quantity of the options.

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The calculated gamma value.
- `Err(GreeksError)`: Returns an error if the computation of `d1` or the
  probability density function `n(d1)` fails.

## [§](#calculation){.doc-anchor}Calculation

Gamma is calculated using the formula:

::: example-wrap
``` language-math
\Gamma = \frac{e^{-qT} \cdot N'(d1)}{S \cdot \sigma \cdot \sqrt{T}}
```
:::

Where:

- (N'(d1)): The standard normal probability density function (PDF)
  evaluated at (d1).
- (S): The price of the underlying asset.
- (\\sigma): The implied volatility of the option.
- (T): The time to expiration in years.
- (q): The dividend yield of the underlying asset.

#### [§](#steps){.doc-anchor}Steps:

1.  Compute (d1) using the `d1` function.
2.  Evaluate (N'(d1)) using the `n` function.
3.  Apply the gamma formula, accounting for the effect of the dividend
    yield (e\^{-qT}).
4.  Multiply the result by the option's quantity.

## [§](#edge-cases){.doc-anchor}Edge Cases

- If the implied volatility ((\\sigma)) is zero, gamma is returned as
  `0`.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::greeks::gamma;
use optionstratlib::Options;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
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

match gamma(&option) {
    Ok(result) => info!("Gamma: {}", result),
    Err(e) => error!("Error calculating gamma: {:?}", e),
}
```
:::

## [§](#notes){.doc-anchor}Notes

- This function assumes that the dividend yield (q) and the time to
  expiration (T) are provided in consistent units.
- If the implied volatility or time to expiration is very small, the
  result may be close to 0, as gamma becomes negligible in those cases.
:::::
::::::::
:::::::::
