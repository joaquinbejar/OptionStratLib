:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [adjust_volatility]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#376-406){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn adjust_volatility(
    volatility: Positive,
    from_frame: TimeFrame,
    to_frame: TimeFrame,
) -> Result<Positive, VolatilityError>
```

Expand description

:::: docblock
Adjusts volatility between different timeframes using the square root of
time rule

## [§](#arguments){.doc-anchor}Arguments

- `volatility` - The volatility to adjust
- `from_frame` - The original timeframe of the volatility
- `to_frame` - The target timeframe for the volatility

## [§](#returns){.doc-anchor}Returns

The adjusted volatility for the target timeframe

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::pos_or_panic;
use optionstratlib::utils::TimeFrame;
use optionstratlib::volatility::adjust_volatility;
let daily_vol = pos!(0.2); // 20% daily volatility
let minute_vol = adjust_volatility(daily_vol, TimeFrame::Day, TimeFrame::Minute).unwrap();
```
:::
::::
:::::::
::::::::
