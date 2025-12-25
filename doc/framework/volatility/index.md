:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module volatility Copy item path

[[Source](../../src/optionstratlib/volatility/mod.rs.html#7-178){.src}
]{.sub-heading}
::::

Expand description

:::::::::: docblock
- `volatility` - Volatility modeling, forecasting, and analysis
  utilities.

Comprehensive tools for volatility analysis including historical
volatility calculation, implied volatility determination, volatility
forecasting models (GARCH, EWMA), and volatility skew/smile analysis.

## [§](#volatility-module){.doc-anchor}Volatility Module

This module provides comprehensive volatility calculation and modeling
tools for financial applications, including historical, implied, and
stochastic volatility models.

### [§](#core-features){.doc-anchor}Core Features

#### [§](#volatility-calculation-methods){.doc-anchor}Volatility Calculation Methods

- Constant Volatility
- Historical Volatility (Moving Window)
- EWMA (Exponentially Weighted Moving Average)
- GARCH(1,1)
- Heston Stochastic Volatility
- Implied Volatility
- Uncertain Volatility Bounds
- Volatility Surface Interpolation

### [§](#usage-examples){.doc-anchor}Usage Examples

#### [§](#basic-volatility-calculations){.doc-anchor}Basic Volatility Calculations

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::Positive;
use optionstratlib::volatility::constant_volatility;

let returns = [dec!(0.02), dec!(0.02), dec!(0.02), dec!(0.02)];
let vol = constant_volatility(&returns);
```
:::

#### [§](#implied-volatility-calculation){.doc-anchor}Implied Volatility Calculation

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::volatility::implied_volatility;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;

let mut option = Options::new(
    OptionType::European,
    Side::Long,
    "STOCK".to_string(),
    pos!(100.0),   // Strike price
    ExpirationDate::Days(pos!(30.0)),
    pos!(0.2),   // Initial volatility guess
    Positive::ONE,   // Quantity
    pos!(100.0),   // Current price
    dec!(0.05),   // Risk-free rate
    OptionStyle::Call,
    Positive::ZERO,   // Dividend yield
    None,   // Exotic parameters
);

let market_price = pos!(30.0);
let iv = implied_volatility(market_price, &mut option, 100);
```
:::

#### [§](#historical-volatility-with-moving-window){.doc-anchor}Historical Volatility with Moving Window

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::volatility::historical_volatility;

let returns = [dec!(0.02), dec!(0.02), dec!(-0.02), dec!(0.02)];
let window_size = 3;
let hist_vol = historical_volatility(&returns, window_size);
```
:::

#### [§](#ewma-volatility){.doc-anchor}EWMA Volatility

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::volatility::ewma_volatility;

let returns = vec![dec!(0.01), dec!(-0.02), dec!(0.015), dec!(-0.01)];
let lambda = dec!(0.94); // Standard decay factor for daily data
let ewma_vol = ewma_volatility(&returns, lambda);
```
:::

### [§](#mathematical-models){.doc-anchor}Mathematical Models

#### [§](#garch11){.doc-anchor}GARCH(1,1)

The GARCH(1,1) model is implemented as: σ²(t) = ω + α \* r²(t-1) + β \*
σ²(t-1)

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::volatility::garch_volatility;

let returns = vec![dec!(0.01), dec!(-0.02), dec!(0.015)];
let omega = dec!(0.1);  // Long-term variance weight
let alpha = dec!(0.2);  // Recent shock weight
let beta = dec!(0.7);   // Previous variance weight
let garch_vol = garch_volatility(&returns, omega, alpha, beta);
```
:::

#### [§](#heston-stochastic-volatility){.doc-anchor}Heston Stochastic Volatility

Implements the Heston model: dv(t) = κ(θ - v(t))dt + ξ√v(t)dW(t)

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use optionstratlib::assert_decimal_eq;
use optionstratlib::volatility::simulate_heston_volatility;

let kappa = dec!(2.0);      // Mean reversion speed
let theta = dec!(0.04);     // Long-term variance
let xi = dec!(0.3);         // Volatility of volatility
let v0 = dec!(0.04);        // Initial variance
let dt = Decimal::ONE / dec!(252.0);   // Daily time step
let steps = 252;      // Number of steps

let heston_vol = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);
```
:::

### [§](#time-frame-handling){.doc-anchor}Time Frame Handling

The module includes utilities for converting between different time
frames:

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::pos_or_panic;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::volatility::{annualized_volatility, de_annualized_volatility};

let daily_vol = pos!(0.01);
let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day).unwrap();
let daily_vol_again = de_annualized_volatility(annual_vol, TimeFrame::Day);
```
:::

### [§](#performance-considerations){.doc-anchor}Performance Considerations

- Implied volatility calculation: O(n) where n is max_iterations
- Historical volatility: O(n\*w) where n is returns length and w is
  window size
- EWMA: O(n) where n is returns length
- GARCH: O(n) where n is returns length
- Heston simulation: O(n) where n is number of steps

### [§](#implementation-notes){.doc-anchor}Implementation Notes

- All volatility calculations ensure non-negative results
- Implied volatility uses Newton-Raphson method with bounds
- Surface interpolation uses bilinear interpolation
- Time scaling follows the square root of time rule
- Numerical stability is ensured through bounds checking

### [§](#references){.doc-anchor}References

The implementations are based on standard financial mathematics
literature:

- Black-Scholes-Merton option pricing model
- RiskMetrics™ Technical Document for EWMA
- Heston (1993) stochastic volatility model
- GARCH by Bollerslev (1986)
::::::::::

## Traits[§](#traits){.anchor} {#traits .section-header}

[AtmIvProvider](trait.AtmIvProvider.html "trait optionstratlib::volatility::AtmIvProvider"){.trait}
:   Trait for providing at-the-money implied volatility.

[VolatilitySmile](trait.VolatilitySmile.html "trait optionstratlib::volatility::VolatilitySmile"){.trait}
:   A trait defining a volatility smile representation.

## Functions[§](#functions){.anchor} {#functions .section-header}

[adjust_volatility](fn.adjust_volatility.html "fn optionstratlib::volatility::adjust_volatility"){.fn}
:   Adjusts volatility between different timeframes using the square
    root of time rule

[annualized_volatility](fn.annualized_volatility.html "fn optionstratlib::volatility::annualized_volatility"){.fn}
:   Annualizes a volatility value from a specific timeframe.

[calculate_iv](fn.calculate_iv.html "fn optionstratlib::volatility::calculate_iv"){.fn}
:   Calculates the implied volatility (IV) of an option given its
    parameters.

[constant_volatility](fn.constant_volatility.html "fn optionstratlib::volatility::constant_volatility"){.fn}
:   Calculates the constant volatility from a series of returns.

[de_annualized_volatility](fn.de_annualized_volatility.html "fn optionstratlib::volatility::de_annualized_volatility"){.fn}
:   De-annualizes a volatility value to a specific timeframe.

[ewma_volatility](fn.ewma_volatility.html "fn optionstratlib::volatility::ewma_volatility"){.fn}
:   Calculates EWMA (Exponentially Weighted Moving Average) volatility.

[garch_volatility](fn.garch_volatility.html "fn optionstratlib::volatility::garch_volatility"){.fn}
:   Calculates GARCH(1,1) volatility (simplified).

[generate_ou_process](fn.generate_ou_process.html "fn optionstratlib::volatility::generate_ou_process"){.fn}
:   Generates a mean-reverting Ornstein-Uhlenbeck process time series

[historical_volatility](fn.historical_volatility.html "fn optionstratlib::volatility::historical_volatility"){.fn}
:   Calculates historical volatility using a moving window approach.

[implied_volatility](fn.implied_volatility.html "fn optionstratlib::volatility::implied_volatility"){.fn}
:   Calculates the implied volatility of an option given its market
    price.

[simulate_heston_volatility](fn.simulate_heston_volatility.html "fn optionstratlib::volatility::simulate_heston_volatility"){.fn}
:   Simulates stochastic volatility using the Heston model (simplified).

[uncertain_volatility_bounds](fn.uncertain_volatility_bounds.html "fn optionstratlib::volatility::uncertain_volatility_bounds"){.fn}
:   Calculates bounds for uncertain volatility.

[volatility_for_dt](fn.volatility_for_dt.html "fn optionstratlib::volatility::volatility_for_dt"){.fn}
:   Adjusts annualized volatility for use in random walk simulations
    with a specific dt.
:::::::::::::
::::::::::::::
