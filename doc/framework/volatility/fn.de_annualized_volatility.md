:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [de_annualized_volatility]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#351-356){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn de_annualized_volatility(
    annual_volatility: Positive,
    timeframe: TimeFrame,
) -> Result<Positive, VolatilityError>
```

Expand description

:::: docblock
De-annualizes a volatility value to a specific timeframe.

## [§](#arguments){.doc-anchor}Arguments

- `annual_volatility` - The annualized volatility value
- `timeframe` - The target timeframe

## [§](#returns){.doc-anchor}Returns

The de-annualized volatility as Decimal

## [§](#formula){.doc-anchor}Formula

The de-annualization is performed using: timeframe_vol = annual_vol /
sqrt(periods_per_year)

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::pos;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::volatility::{de_annualized_volatility};
let annual_vol = pos!(0.20); // 20% annual volatility
let daily_vol = de_annualized_volatility(annual_vol, TimeFrame::Day);
// daily_vol ≈ 0.0126 or about 1.26%
```
:::
::::
:::::::
::::::::
