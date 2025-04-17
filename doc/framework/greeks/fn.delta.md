:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [delta]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#320-406){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn delta(option: &Options) -> Result<Decimal, GreeksError>
```

Expand description

:::: docblock
Calculates the delta of an option.

The delta measures the sensitivity of an option's price to changes in
the price of the underlying asset. It is calculated differently for call
and put options. For options with zero implied volatility, the delta is
determined based on whether the option is in-the-money or
out-of-the-money.

## [§](#parameters){.doc-anchor}Parameters

- `option: &Options` A reference to an `Options` struct containing all
  the relevant parameters for the calculation:
  - `underlying_price`: The current price of the underlying asset.
  - `strike_price`: The strike price of the option.
  - `risk_free_rate`: The annualized risk-free interest rate.
  - `expiration_date`: The time to expiration of the option, in years.
  - `implied_volatility`: The implied volatility of the option.
  - `dividend_yield`: The dividend yield of the underlying asset.
  - `quantity`: The quantity of the options.
  - `option_style`: The style of the option (Call or Put).

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The calculated delta value.
- `Err(GreeksError)`: Returns an error if any intermediate calculations
  fail.

## [§](#calculation-details){.doc-anchor}Calculation Details

- If `implied_volatility == 0`, the delta is determined based on whether
  the option is in-the-money or out-of-the-money:

  - Call Option:
    - In-the-money: Delta = `sign`
    - Out-of-the-money: Delta = 0
  - Put Option:
    - In-the-money: Delta = `-sign`
    - Out-of-the-money: Delta = 0

- For options with non-zero implied volatility, the delta is calculated
  as:

  - Call Option: \[ \\Delta\_{\\text{call}} = \\text{sign} \\cdot N(d1)
    \\cdot e\^{-qT} \]
  - Put Option: \[ \\Delta\_{\\text{put}} = \\text{sign} \\cdot
    (N(d1) - 1) \\cdot e\^{-qT} \] Where:
    - (N(d1)): The cumulative distribution function (CDF) of the
      standard normal distribution evaluated at (d1).
    - (q): The dividend yield.
    - (T): Time to expiration.

- The delta is adjusted by multiplying it by the option quantity.

## [§](#errors){.doc-anchor}Errors

- `GreeksError`: If the calculation of (d1) or the standard normal CDF
  (`big_n`) fails.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{error, info};
use optionstratlib::constants::ZERO;
use optionstratlib::greeks::delta;
use optionstratlib::Options;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::{pos, Positive};
let option = Options {
    option_type: OptionType::European,side:
    Side::Long,underlying_price:
    pos!(100.0),
    strike_price: pos!(95.0),
    risk_free_rate: dec!(0.05),
    expiration_date: ExpirationDate::Days(pos!(30.0)),
    implied_volatility: pos!(0.2),
    dividend_yield: Positive::ZERO,
    quantity: pos!(1.0),
    option_style: OptionStyle::Call,
    underlying_symbol: "AAPL".to_string(),
    exotic_params: None,
};

match delta(&option) {
    Ok(result) => info!("Delta: {}", result),
    Err(e) => error!("Error calculating delta: {:?}", e),
}
```
:::
::::
:::::::
::::::::
