:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [annualized_volatility]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#317-322){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn annualized_volatility(
    volatility: Positive,
    timeframe: TimeFrame,
) -> Result<Positive, VolatilityError>
```

Expand description

:::: docblock
Annualizes a volatility value from a specific timeframe.

## [§](#arguments){.doc-anchor}Arguments

- `volatility` - The volatility value to annualize
- `timeframe` - The timeframe of the input volatility

## [§](#returns){.doc-anchor}Returns

The annualized volatility as Decimal

## [§](#formula){.doc-anchor}Formula

The annualization is performed using the square root of time rule:
annualized_vol = vol \* sqrt(periods_per_year)

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use positive::pos_or_panic;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::volatility::{annualized_volatility};
let daily_vol = pos!(0.01); // 1% daily volatility
let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day);
// annual_vol ≈ 0.1587 or about 15.87%
```
:::
::::
:::::::
::::::::
