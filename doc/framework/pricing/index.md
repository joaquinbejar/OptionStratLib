::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module pricing Copy item path

[[Source](../../src/optionstratlib/pricing/mod.rs.html#1-255){.src}
]{.sub-heading}
::::

Expand description

::::: docblock
- `pricing` - Option pricing models including Black-Scholes and
  numerical methods.

Implementations of various option pricing models including
Black-Scholes-Merton, binomial trees, Monte Carlo simulation, and finite
difference methods. Supports European, American, and exotic options.

## [§](#options-pricing-module){.doc-anchor}Options Pricing Module

This module provides implementations for various financial models and
utilities to calculate and simulate option pricing. The module includes
support for several well-known mathematical models such as the Binomial
Tree Model, Black-Scholes Model, Monte Carlo Simulations, and Telegraph
Process.

### [§](#core-models){.doc-anchor}Core Models

#### [§](#binomial-model-binomial_model){.doc-anchor}Binomial Model (`binomial_model`)

Contains the implementation of the Binomial Tree Model for option
pricing. This model supports both European and American style options
and allows for customization of steps and parameters like volatility,
interest rates, and time increments.

#### [§](#black-scholes-model-black_scholes_model){.doc-anchor}Black-Scholes Model (`black_scholes_model`)

Implements the Black-Scholes option pricing model, a widely used formula
for pricing European-style options. This module provides tools to
calculate option prices and associated Greek values.

#### [§](#monte-carlo-simulations-monte_carlo){.doc-anchor}Monte Carlo Simulations (`monte_carlo`)

Provides Monte Carlo simulation capabilities for option pricing. This
module supports simulation of stock price paths and uses statistical
methods to estimate option values under various stochastic processes.

#### [§](#telegraph-process-telegraph){.doc-anchor}Telegraph Process (`telegraph`)

Implements the Telegraph process, a two-state stochastic process for
modeling price movements. Key features include:

- State transitions between +1 and -1 based on transition rates
- Parameter estimation from historical data
- Support for asymmetric transition rates
- Applications in regime-switching scenarios

The Telegraph Process is particularly useful for:

- Modeling regime changes in volatility
- Capturing market sentiment switches
- Simulating discrete state transitions

### [§](#supporting-modules){.doc-anchor}Supporting Modules

#### [§](#payoff-calculations-payoff){.doc-anchor}Payoff Calculations (`payoff`)

Defines payoff structures and calculations for:

- Standard options (calls and puts)
- Exotic options
- Custom payoff functions

#### [§](#utility-functions-utils){.doc-anchor}Utility Functions (`utils`)

Provides essential mathematical and financial utilities:

- Probability calculations
- Discount factor computations
- Statistical functions
- Parameter estimation tools

#### [§](#constants-constants){.doc-anchor}Constants (`constants`)

Defines model parameters and limits used across the pricing
implementations:

- Numerical bounds
- Default values
- Calculation constraints

### [§](#usage-examples){.doc-anchor}Usage Examples

#### [§](#using-the-telegraph-process){.doc-anchor}Using the Telegraph Process

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::pricing::telegraph::{TelegraphProcess, telegraph};
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::Positive;
use optionstratlib::pos;

// Create a Telegraph Process with transition rates
let process = TelegraphProcess::new(dec!(0.5), dec!(0.3));

// Price an option using the Telegraph Process
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
let price = telegraph(&option, 1000, Some(dec!(0.5)), Some(dec!(0.3)));
```
:::

#### [§](#combined-model-analysis){.doc-anchor}Combined Model Analysis

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
use optionstratlib::Positive;
use optionstratlib::pos;
use optionstratlib::pricing::{
    black_scholes_model::black_scholes,
    monte_carlo::monte_carlo_option_pricing,
    telegraph::telegraph
};
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
// Compare prices across different models
let bs_price = black_scholes(&option);
let mc_price = monte_carlo_option_pricing(&option, 2, 2);
let tp_price = telegraph(&option, 1000, Some(dec!(0.5)), Some(dec!(0.3)));
```
:::

### [§](#implementation-notes){.doc-anchor}Implementation Notes

- All models support standard market conventions for option pricing
- Parameter validation and bounds checking are implemented
- Error handling follows Rust's Result pattern
- Performance optimizations are included for numerical calculations

### [§](#model-selection-guidelines){.doc-anchor}Model Selection Guidelines

Choose the appropriate model based on your needs:

- Black-Scholes: Quick pricing of European options
- Binomial: American options and early exercise
- Monte Carlo: Complex path-dependent options
- Telegraph: Regime-switching and discrete state transitions

### [§](#performance-considerations){.doc-anchor}Performance Considerations

- Telegraph Process: O(n) complexity where n is the number of steps
- Monte Carlo: O(m\*n) where m is the number of simulations
- Binomial: O(n²) where n is the number of steps
- Black-Scholes: O(1) constant time calculation

For high-frequency calculations, consider using the Black-Scholes model
when applicable, as it provides the fastest computation times.
:::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use binomial_model::`[`BinomialPricingParams`](binomial_model/struct.BinomialPricingParams.html "struct optionstratlib::pricing::binomial_model::BinomialPricingParams"){.struct}`;`

`pub use binomial_model::`[`generate_binomial_tree`](binomial_model/fn.generate_binomial_tree.html "fn optionstratlib::pricing::binomial_model::generate_binomial_tree"){.fn}`;`

`pub use binomial_model::`[`price_binomial`](binomial_model/fn.price_binomial.html "fn optionstratlib::pricing::binomial_model::price_binomial"){.fn}`;`

`pub use black_scholes_model::`[`BlackScholes`](black_scholes_model/trait.BlackScholes.html "trait optionstratlib::pricing::black_scholes_model::BlackScholes"){.trait}`;`

`pub use black_scholes_model::`[`black_scholes`](black_scholes_model/fn.black_scholes.html "fn optionstratlib::pricing::black_scholes_model::black_scholes"){.fn}`;`

`pub use monte_carlo::`[`monte_carlo_option_pricing`](monte_carlo/fn.monte_carlo_option_pricing.html "fn optionstratlib::pricing::monte_carlo::monte_carlo_option_pricing"){.fn}`;`

`pub use telegraph::`[`TelegraphProcess`](telegraph/struct.TelegraphProcess.html "struct optionstratlib::pricing::telegraph::TelegraphProcess"){.struct}`;`

`pub use telegraph::`[`telegraph`](telegraph/fn.telegraph.html "fn optionstratlib::pricing::telegraph::telegraph"){.fn}`;`

`pub use unified::`[`Priceable`](unified/trait.Priceable.html "trait optionstratlib::pricing::unified::Priceable"){.trait}`;`

`pub use unified::`[`PricingEngine`](unified/enum.PricingEngine.html "enum optionstratlib::pricing::unified::PricingEngine"){.enum}`;`

`pub use unified::`[`price_option`](unified/fn.price_option.html "fn optionstratlib::pricing::unified::price_option"){.fn}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[binomial_model](binomial_model/index.html "mod optionstratlib::pricing::binomial_model"){.mod}
:   Binomial tree model implementation for option pricing.

[black_scholes_model](black_scholes_model/index.html "mod optionstratlib::pricing::black_scholes_model"){.mod}
:   Black-Scholes model for option pricing and analysis.

[monte_carlo](monte_carlo/index.html "mod optionstratlib::pricing::monte_carlo"){.mod}
:   Monte Carlo simulation methods for financial modeling.

[telegraph](telegraph/index.html "mod optionstratlib::pricing::telegraph"){.mod}
:   Telegraph process model for asset price movement.

[unified](unified/index.html "mod optionstratlib::pricing::unified"){.mod}
:   Unified pricing system for options.

## Structs[§](#structs){.anchor} {#structs .section-header}

[PayoffInfo](struct.PayoffInfo.html "struct optionstratlib::pricing::PayoffInfo"){.struct}
:   `PayoffInfo` is a struct that holds information about an option's
    payoff calculation parameters.

## Traits[§](#traits){.anchor} {#traits .section-header}

[Payoff](trait.Payoff.html "trait optionstratlib::pricing::Payoff"){.trait}
:   Defines a contract for calculating the payoff value of an option.

[Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait}
:   Defines the profit calculation behavior for financial instruments.

## Functions[§](#functions){.anchor} {#functions .section-header}

[probability_keep_under_strike](fn.probability_keep_under_strike.html "fn optionstratlib::pricing::probability_keep_under_strike"){.fn}
:   Calculates the probability that the option will remain under the
    strike price.

[simulate_returns](fn.simulate_returns.html "fn optionstratlib::pricing::simulate_returns"){.fn}
:   Simulates stock returns based on a normal distribution using pure
    decimal arithmetic.
::::::::
:::::::::
