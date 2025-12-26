:::::::::: width-limiter
::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module risk Copy item path

[[Source](../../src/optionstratlib/risk/mod.rs.html#7-186){.src}
]{.sub-heading}
::::

Expand description

:::::: docblock
- `risk` - Risk assessment and management tools for options portfolios.

Tools for analyzing and quantifying risk in options positions and
portfolios, including Value at Risk (VaR), stress testing, scenario
analysis, and portfolio optimization algorithms.

## [§](#span-standard-portfolio-analysis-of-risk-module){.doc-anchor}SPAN (Standard Portfolio Analysis of Risk) Module

This module implements the SPAN® (Standard Portfolio Analysis of Risk)
methodology, a system developed by the Chicago Mercantile Exchange (CME)
for calculating margin requirements for derivatives portfolios.

### [§](#overview){.doc-anchor}Overview

SPAN calculates margin requirements by analyzing the potential losses a
portfolio might experience under various market scenarios. It considers:

- Price changes in the underlying asset
- Changes in volatility
- Extreme market moves
- Time decay effects
- Short option exposure

### [§](#core-components){.doc-anchor}Core Components

#### [§](#spanmargin-structure){.doc-anchor}SPANMargin Structure

::: example-wrap
``` {.rust .rust-example-rendered}
pub struct SPANMargin {
    scanning_range: f64,   // Overall market move range
    short_option_minimum: f64,   // Minimum charge for short options
    price_scan_range: f64,   // Range for price scenarios
    volatility_scan_range: f64,   // Range for volatility scenarios
}
```
:::

#### [§](#risk-scenarios){.doc-anchor}Risk Scenarios

The module evaluates positions under multiple scenarios combining:

- Price movements (up/down/unchanged)
- Volatility changes (up/down/unchanged)
- Time decay effects

### [§](#usage-examples){.doc-anchor}Usage Examples

#### [§](#basic-margin-calculation){.doc-anchor}Basic Margin Calculation

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use positive::Positive;
use optionstratlib::model::position::Position;
use positive::pos_or_panic;
use chrono::Utc;
use rust_decimal_macros::dec;
use optionstratlib::risk::SPANMargin;

// Create an option position
let option = Options::new(
    OptionType::European,
    Side::Short,
    "STOCK".to_string(),
    pos!(150.0),   // Strike price
    ExpirationDate::Days(pos!(30.0)),
    pos!(0.2),   // Volatility
    Positive::ONE,   // Quantity
    pos!(155.0),   // Current price
    dec!(0.05),   // Risk-free rate
    OptionStyle::Call,
    Positive::ZERO,   // Dividend yield
    None,   // Exotic parameters
);

let position = Position {
    option,
    premium: pos!(5.0),
    date: Utc::now(),
    open_fee: pos!(0.5),
    close_fee: pos!(0.5),
    epic: None,
    extra_fields: None,
};

// Create SPAN calculator
let span = SPANMargin::new(
    dec!(0.10),   // 10% short option minimum
    dec!(0.05),   // 5% price scan range
    dec!(0.10),   // 10% volatility scan range
);

// Calculate margin requirement
let margin = span.calculate_margin(&position);
```
:::

#### [§](#portfolio-analysis){.doc-anchor}Portfolio Analysis

::: example-wrap
``` {.rust .rust-example-rendered}
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::model::position::Position;
use positive::Positive;
use positive::pos_or_panic;
use optionstratlib::risk::SPANMargin;

let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(pos!(30.0)),
            implied_volatility: pos!(0.2),
            quantity: Positive::ONE,
            underlying_price: pos!(105.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: pos!(0.01),
            exotic_params: None,
        };
// Create multiple positions
let positions = vec![
    Position {
        option: option.clone(),
        premium: pos!(5.0),
        date: Utc::now(),
        open_fee: pos!(0.5),
        close_fee: pos!(0.5),
        epic: None,
        extra_fields: None,
    },
    Position {
        option,
        premium: pos!(3.0),
        date: Utc::now(),
        open_fee: pos!(0.5),
        close_fee: pos!(0.5),
        epic: None,
        extra_fields: None,
    },
];

let span = SPANMargin::new(dec!(0.10), dec!(0.05), dec!(0.10));

// Calculate margin for each position
let margins: Vec<Decimal> = positions.iter()
    .map(|pos| span.calculate_margin(pos))
    .collect();
```
:::

### [§](#implementation-details){.doc-anchor}Implementation Details

#### [§](#risk-array-calculation){.doc-anchor}Risk Array Calculation

The risk array is calculated by:

1.  Generating price scenarios
2.  Generating volatility scenarios
3.  Calculating potential loss in each scenario
4.  Taking the maximum loss as the base margin requirement

#### [§](#short-option-minimum){.doc-anchor}Short Option Minimum

Additional protection against short option positions:

- Applied when the position is short
- Based on the underlying price and quantity
- Acts as a floor for the margin requirement

### [§](#performance-considerations){.doc-anchor}Performance Considerations

- Time complexity: O(n \* m) where n is the number of price scenarios
  and m is the number of volatility scenarios
- Memory complexity: O(n \* m) for storing the risk array
- Calculation intensive due to multiple option pricing calculations per
  position

### [§](#notes){.doc-anchor}Notes

- All parameters should be provided as decimals (e.g., 0.15 for 15%)
- The module uses Black-Scholes pricing for scenario calculations
- Short option minimum is always enforced for short positions
- Results are conservative estimates of potential losses
::::::

## Structs[§](#structs){.anchor} {#structs .section-header}

[RiskMetricsSimulation](struct.RiskMetricsSimulation.html "struct optionstratlib::risk::RiskMetricsSimulation"){.struct}
:   Represents various risk metrics for the options strategy

[SPANMargin](struct.SPANMargin.html "struct optionstratlib::risk::SPANMargin"){.struct}
:   Represents parameters for calculating margin requirements using the
    Standard Portfolio Analysis of Risk (SPAN) methodology.

## Enums[§](#enums){.anchor} {#enums .section-header}

[RiskCategory](enum.RiskCategory.html "enum optionstratlib::risk::RiskCategory"){.enum}
:   Risk categories for options strategies
:::::::::
::::::::::
