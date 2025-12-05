:::::::::: width-limiter
::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module chains Copy item path

[[Source](../../src/optionstratlib/chains/mod.rs.html#1-232){.src}
]{.sub-heading}
::::

Expand description

:::::: docblock
- `chains` - Functionality for working with options chains and series
  data.

Tools for parsing, manipulating, and analyzing options chain data.
Includes methods to filter chains by expiration, strike price, and other
criteria, as well as utilities for chain visualization and analysis.

## [§](#chains-module){.doc-anchor}Chains Module

This module provides functionality for working with option chains and
their components. It includes tools for building, managing, and
manipulating option chains, as well as handling multiple-leg option
strategies.

### [§](#core-components){.doc-anchor}Core Components

- `chain` - Implements core option chain functionality (`OptionChain`
  and `OptionData` structures)
- `legs` - Provides strategy leg combinations through the `StrategyLegs`
  enum
- `utils` - Contains utility functions and parameter structures for
  chain operations

### [§](#main-features){.doc-anchor}Main Features

- Option chain construction and management
- Support for various option data formats
- Import/export capabilities (CSV, JSON)
- Multiple-leg strategy support
- Price calculation and volatility adjustments

### [§](#example-usage){.doc-anchor}Example Usage

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::{pos, spos, ExpirationDate, Positive};

let option_chain_params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.2),
            Decimal::ZERO,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(dec!(0.0)),
                spos!(0.05),
                Some("SP500".to_string()),
            ),
            pos!(0.2),
        );

let built_chain = OptionChain::build_chain(&option_chain_params);
assert_eq!(built_chain.symbol, "SP500");
assert_eq!(built_chain.underlying_price, Positive::new(100.0).unwrap());
```
:::

### [§](#strategy-legs-support){.doc-anchor}Strategy Legs Support

The module supports various option strategy combinations through the
`StrategyLegs` enum:

- Two-leg strategies (e.g., spreads)
- Four-leg strategies (e.g., iron condors)
- Six-leg strategies (e.g., butterfly variations)

### [§](#utility-functions){.doc-anchor}Utility Functions

The module provides various utility functions for:

- Strike price generation
- Volatility adjustment
- Price calculations
- Data parsing and formatting

### [§](#file-handling){.doc-anchor}File Handling

Supports both CSV and JSON formats for:

- Importing option chain data
- Exporting option chain data
- Maintaining consistent data formats

## [§](#risk-neutral-density-rnd-analysis-module){.doc-anchor}Risk Neutral Density (RND) Analysis Module

This module implements functionality to calculate and analyze the
Risk-Neutral Density (RND) from option chains. The RND represents the
market's implied probability distribution of future asset prices and is
a powerful tool for understanding market expectations.

### [§](#theory-and-background){.doc-anchor}Theory and Background

The Risk-Neutral Density (RND) is a probability distribution that
represents the market's view of possible future prices of an underlying
asset, derived from option prices. It is "risk-neutral" because it
incorporates both the market's expectations and risk preferences into a
single distribution.

Key aspects of RND:

- Extracted from option prices using the Breeden-Litzenberger formula
- Provides insights into market sentiment and expected volatility
- Used for pricing exotic derivatives and risk assessment

### [§](#statistical-moments-and-their-interpretation){.doc-anchor}Statistical Moments and Their Interpretation

The module calculates four key statistical moments:

1.  **Mean**: The expected future price of the underlying asset
2.  **Variance**: Measure of price dispersion, related to expected
    volatility
3.  **Skewness**: Indicates asymmetry in price expectations
    - Positive skew: Market expects upside potential
    - Negative skew: Market expects downside risks
4.  **Kurtosis**: Measures the likelihood of extreme events
    - High kurtosis: Market expects "fat tails" (more extreme moves)
    - Low kurtosis: Market expects more moderate price movements

### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::chains::{RNDParameters, RNDAnalysis};
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::{pos, ExpirationDate, Positive};

// Create parameters for RND calculation
let params = RNDParameters {
    risk_free_rate: dec!(0.05),
    interpolation_points: 100,
    derivative_tolerance: pos!(0.001),
};
let chain = OptionDataPriceParams::new(
    Some(Box::new(Positive::new(2000.0).unwrap())),
    Some(ExpirationDate::Days(pos!(10.0))),
    Some(dec!(0.01)),
    Some(Positive::ZERO),
    Some("Symbol".to_string()),
);

let option_chain_params = OptionChainBuildParams::new(
    "SP500".to_string(),
    Some(Positive::ONE),
    5,
    Some(Positive::ONE),
    dec!(-0.2),
    dec!(0.0001),
    Positive::new(0.02).unwrap(),
    2,
    chain,
    pos!(0.2),
);

let option_chain = OptionChain::build_chain(&option_chain_params);
// Calculate RND from option chain
let rnd_result = option_chain.calculate_rnd(&params).unwrap();

// Access statistical moments
info!("Expected price: {}", rnd_result.statistics.mean);
info!("Implied volatility: {}", rnd_result.statistics.volatility);
info!("Market bias: {}", rnd_result.statistics.skewness);
info!("Tail risk: {}", rnd_result.statistics.kurtosis);
```
:::

### [§](#market-insights-from-rnd){.doc-anchor}Market Insights from RND

The RND provides several valuable insights:

1.  **Price Expectations**

    - Mean indicates the market's expected future price
    - Variance shows uncertainty around this expectation

2.  **Market Sentiment**

    - Skewness reveals directional bias
    - Kurtosis indicates expected market stability

3.  **Risk Assessment**

    - Shape of distribution helps quantify various risks
    - Particularly useful for stress testing and VaR calculations

4.  **Volatility Structure**

    - Implied volatility skew analysis
    - Term structure of market expectations

### [§](#mathematical-foundation){.doc-anchor}Mathematical Foundation

The RND is calculated using the Breeden-Litzenberger formula:

::: example-wrap
``` language-text
q(K) = e^(rT) * (∂²C/∂K²)
```
:::

Where:

- q(K) is the RND value at strike K
- r is the risk-free rate
- T is time to expiration
- C is the call option price
- ∂²C/∂K² is the second derivative with respect to strike

### [§](#implementation-details){.doc-anchor}Implementation Details

The module implements:

- Numerical approximation of derivatives
- Statistical moment calculations
- Error handling for numerical stability
- Volatility skew analysis

The implementation focuses on numerical stability and accurate moment
calculations, particularly for extreme market conditions.
::::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use chain::`[`OptionChain`](chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}`;`

`pub use utils::`[`OptionChainBuildParams`](utils/struct.OptionChainBuildParams.html "struct optionstratlib::chains::utils::OptionChainBuildParams"){.struct}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[chain](chain/index.html "mod optionstratlib::chains::chain"){.mod}
:   `chain` - Public module for handling option chains and related
    functionalities

[utils](utils/index.html "mod optionstratlib::chains::utils"){.mod}
:   `utils` - Public module containing utility functions and helpers for
    financial calculations

## Structs[§](#structs){.anchor} {#structs .section-header}

[DeltasInStrike](struct.DeltasInStrike.html "struct optionstratlib::chains::DeltasInStrike"){.struct}
:   Represents option delta values for all four basic option positions
    at a specific strike price.

[OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}
:   Struct representing a row in an option chain with detailed pricing
    and analytics data.

[OptionsInStrike](struct.OptionsInStrike.html "struct optionstratlib::chains::OptionsInStrike"){.struct}
:   Represents a collection of option positions at the same strike
    price.

[RNDParameters](struct.RNDParameters.html "struct optionstratlib::chains::RNDParameters"){.struct}
:   Parameters for Risk-Neutral Density calculation

[RNDResult](struct.RNDResult.html "struct optionstratlib::chains::RNDResult"){.struct}
:   Results of Risk-Neutral Density calculation

## Enums[§](#enums){.anchor} {#enums .section-header}

[StrategyLegs](enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}
:   Represents the various configurations of option strategy legs with
    different complexities.

## Traits[§](#traits){.anchor} {#traits .section-header}

[RNDAnalysis](trait.RNDAnalysis.html "trait optionstratlib::chains::RNDAnalysis"){.trait}
:   Trait defining Risk-Neutral Density analysis capabilities

## Functions[§](#functions){.anchor} {#functions .section-header}

[generator_optionchain](fn.generator_optionchain.html "fn optionstratlib::chains::generator_optionchain"){.fn}
:   Generates a vector of `Step`s containing `Positive` x-values and
    `OptionChain` y-values.

[generator_positive](fn.generator_positive.html "fn optionstratlib::chains::generator_positive"){.fn}
:   Generates a vector of `Step`s containing `Positive` x-values and
    `Positive` y-values.
:::::::::
::::::::::
