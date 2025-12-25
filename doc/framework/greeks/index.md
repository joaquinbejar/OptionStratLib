:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module greeks Copy item path

[[Source](../../src/optionstratlib/greeks/mod.rs.html#7-94){.src}
]{.sub-heading}
::::

Expand description

:::: docblock
- `greeks` - Calculation and management of option sensitivity metrics
  (Delta, Gamma, etc.).

Comprehensive implementation of options Greeks (sensitivity measures)
including Delta, Gamma, Theta, Vega, and Rho. Includes analytical
formulas, numerical approximations, and visualization tools for risk
analysis.

## [§](#greeks-module){.doc-anchor}Greeks Module

This module provides functionality for calculating option Greeks and
related metrics used in options trading and risk management.

### [§](#core-components){.doc-anchor}Core Components

- `equations` - Implementation of Greek calculations (delta, gamma,
  theta, vega, rho)
- `utils` - Utility functions for Greek calculations and related math

### [§](#greeks-provided){.doc-anchor}Greeks Provided

The module calculates the following Greeks:

- Delta (Δ) - Measures the rate of change in option value with respect
  to the underlying price
- Gamma (Γ) - Measures the rate of change in delta with respect to the
  underlying price
- Theta (Θ) - Measures the rate of change in option value with respect
  to time
- Vega (V) - Measures the rate of change in option value with respect to
  volatility
- Rho (ρ) - Measures the rate of change in option value with respect to
  the risk-free rate
- Rho_d - Measures sensitivity to dividend yield changes

### [§](#utilities-included){.doc-anchor}Utilities Included

The utilities module provides essential mathematical functions for Greek
calculations:

- d1/d2 calculations for Black-Scholes model
- Normal distribution functions (PDF, CDF)
- Mathematical helper functions

### [§](#example-usage){.doc-anchor}Example Usage

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::greeks::{delta, gamma, rho, theta, vega};
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::pos_or_panic;
use optionstratlib::Positive;

// Create a sample option
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

// Calculate Greeks
let delta_value = delta(&option);
let gamma_value = gamma(&option);
let theta_value = theta(&option);
let vega_value = vega(&option);
let rho_value = rho(&option);
```
:::

### [§](#mathematical-background){.doc-anchor}Mathematical Background

The Greeks are calculated using the Black-Scholes model and its
derivatives. Each Greek represents a different dimension of risk:

- Delta: First-order price sensitivity
- Gamma: Second-order price sensitivity
- Theta: Time decay
- Vega: Volatility sensitivity
- Rho: Interest rate sensitivity

### [§](#additional-features){.doc-anchor}Additional Features

- Support for both European and American options
- Handling of zero volatility cases
- Adjustments for dividends
- Special case handling for extreme values
::::

## Structs[§](#structs){.anchor} {#structs .section-header}

[Greek](struct.Greek.html "struct optionstratlib::greeks::Greek"){.struct}
:   Represents a complete set of option Greeks, which measure the
    sensitivity of an option's price to various market factors.

[GreeksSnapshot](struct.GreeksSnapshot.html "struct optionstratlib::greeks::GreeksSnapshot"){.struct}
:   A struct representing a snapshot of the Greeks, financial measures
    used to assess risk and sensitivity of derivative instruments such
    as options.

## Traits[§](#traits){.anchor} {#traits .section-header}

[Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait}
:   Trait that provides option Greeks calculation functionality for
    financial instruments.

## Functions[§](#functions){.anchor} {#functions .section-header}

[big_n](fn.big_n.html "fn optionstratlib::greeks::big_n"){.fn}
:   Computes the cumulative distribution function (CDF) of the standard
    normal distribution for a given input `x`.

[calculate_delta_neutral_sizes](fn.calculate_delta_neutral_sizes.html "fn optionstratlib::greeks::calculate_delta_neutral_sizes"){.fn}
:   Calculates the optimal position sizes for two positions to achieve
    delta neutrality while maintaining a specified total position size.

[d1](fn.d1.html "fn optionstratlib::greeks::d1"){.fn}
:   Calculates the `d1` parameter used in the Black-Scholes options
    pricing model.

[d2](fn.d2.html "fn optionstratlib::greeks::d2"){.fn}
:   Calculates the `d2` parameter used in the Black-Scholes options
    pricing model.

[delta](fn.delta.html "fn optionstratlib::greeks::delta"){.fn}
:   Calculates the delta of an option.

[gamma](fn.gamma.html "fn optionstratlib::greeks::gamma"){.fn}
:   Computes the gamma of an option.

[n](fn.n.html "fn optionstratlib::greeks::n"){.fn}
:   Computes the probability density function (PDF) of the standard
    normal distribution for a given input `x`.

[rho](fn.rho.html "fn optionstratlib::greeks::rho"){.fn}
:   Computes the rho of an options contract.

[rho_d](fn.rho_d.html "fn optionstratlib::greeks::rho_d"){.fn}
:   Computes the sensitivity of the option price to changes in the
    dividend yield (Rho_d).

[theta](fn.theta.html "fn optionstratlib::greeks::theta"){.fn}
:   Computes the Theta of an option.

[vega](fn.vega.html "fn optionstratlib::greeks::vega"){.fn}
:   Computes the vega of an option.
:::::::
::::::::
