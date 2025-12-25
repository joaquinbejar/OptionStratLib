::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::: {#main-content .section .content}
::: main-heading
# Crate optionstratlib Copy item path

[[Source](../src/optionstratlib/lib.rs.html#1-915){.src} ]{.sub-heading}
:::

Expand description

:::::::::::::::::::::: docblock
## [§](#optionstratlib-v0100-financial-options-library){.doc-anchor}OptionStratLib v0.10.3: Financial Options Library {#optionstratlib-v0100-financial-options-library}

### [§](#table-of-contents){.doc-anchor}Table of Contents

1.  [Introduction](#introduction)
2.  [Features](#features)
3.  [Core Modules](#core-modules)
4.  [Trading Strategies](#trading-strategies)
5.  [Setup Instructions](#setup-instructions)
6.  [Library Usage](#library-usage)
7.  [Usage Examples](#usage-examples)
8.  [Testing](#testing)
9.  [Contribution and Contact](#contribution-and-contact)

### [§](#introduction){.doc-anchor}Introduction

OptionStratLib is a comprehensive Rust library for options trading and
strategy development across multiple asset classes. This versatile
toolkit enables traders, quants, and developers to model, analyze, and
visualize options strategies with a robust, type-safe approach. The
library focuses on precision with decimal-based calculations, extensive
test coverage, and a modular architecture built on modern Rust 2024
edition.

### [§](#features){.doc-anchor}Features

#### [§](#1-pricing-models){.doc-anchor}1. **Pricing Models** {#1-pricing-models}

- **Black-Scholes Model**: European options pricing with full Greeks
  support
- **Binomial Tree Model**: American and European options with early
  exercise capability
- **Monte Carlo Simulations**: Complex pricing scenarios and
  path-dependent options
- **Telegraph Process Model**: Advanced stochastic modeling for
  jump-diffusion processes

#### [§](#2-greeks-calculation){.doc-anchor}2. **Greeks Calculation** {#2-greeks-calculation}

- Complete Greeks suite: Delta, Gamma, Theta, Vega, Rho
- Real-time sensitivity analysis
- Greeks visualization and risk profiling
- Custom Greeks implementations with adjustable parameters

#### [§](#3-volatility-models){.doc-anchor}3. **Volatility Models** {#3-volatility-models}

- Implied volatility calculation using Newton-Raphson method
- Volatility surface construction and interpolation
- Historical volatility estimation
- Advanced volatility modeling tools

#### [§](#4-option-chain-management){.doc-anchor}4. **Option Chain Management** {#4-option-chain-management}

- Complete option chain construction and analysis
- Strike price generation algorithms
- Chain data import/export (CSV/JSON formats)
- Advanced filtering and selection tools
- Option data grouping and organization

#### [§](#5-trading-strategies-25-strategies){.doc-anchor}5. **Trading Strategies (25+ Strategies)** {#5-trading-strategies-25-strategies}

- **Single Leg**: Long/Short Calls and Puts
- **Spreads**: Bull/Bear Call/Put Spreads
- **Butterflies**: Long/Short Butterfly Spreads, Call Butterfly
- **Complex**: Iron Condor, Iron Butterfly
- **Volatility**: Long/Short Straddles and Strangles
- **Income**: Covered Calls, Poor Man's Covered Call
- **Protection**: Protective Puts, Collars
- **Custom**: Flexible custom strategy framework

#### [§](#6-risk-management--analysis){.doc-anchor}6. **Risk Management & Analysis** {#6-risk-management--analysis}

- Position tracking and management
- Break-even analysis with multiple break-even points
- Profit/Loss calculations at various price points
- Risk profiles and comprehensive visualizations
- Delta neutrality analysis and adjustment
- Probability analysis for strategy outcomes

#### [§](#7-backtesting-framework){.doc-anchor}7. **Backtesting Framework** {#7-backtesting-framework}

- Comprehensive backtesting engine
- Performance metrics calculation
- Strategy optimization tools
- Historical analysis capabilities

#### [§](#8-simulation-tools){.doc-anchor}8. **Simulation Tools** {#8-simulation-tools}

- Monte Carlo simulations for strategy testing
- Telegraph process implementation
- Random walk simulations
- Custom simulation frameworks
- Parametrized simulations with adjustable inputs

#### [§](#9-visualization--plotting){.doc-anchor}9. **Visualization & Plotting** {#9-visualization--plotting}

- Strategy payoff diagrams
- Greeks visualization
- 3D volatility surfaces
- Risk profiles and P&L charts
- Interactive charts (powered by `plotly.rs`)
- Binomial tree visualization
- Comprehensive plotting utilities

#### [§](#10-data-management){.doc-anchor}10. **Data Management** {#10-data-management}

- Efficient decimal-based calculations using `rust_decimal`
- CSV/JSON import/export functionality
- Time series data handling
- Price series management and manipulation
- Robust data validation and error handling

#### [§](#11-mathematical-tools){.doc-anchor}11. **Mathematical Tools** {#11-mathematical-tools}

- Curve interpolation techniques
- Surface construction and analysis
- Geometric operations for financial modeling
- Advanced mathematical utilities for options pricing

### [§](#core-modules){.doc-anchor}Core Modules

The library is organized into the following key modules:

#### [§](#model-model){.doc-anchor}**Model** (`model/`)

Core data structures and types for options trading:

- `option.rs`: Complete option structures with pricing and Greeks
- `position.rs`: Position management and P&L tracking
- `expiration.rs`: Flexible expiration date handling (Days/DateTime)
- `positive.rs`: Type-safe positive number implementation
- `types.rs`: Common enums (OptionType, Side, OptionStyle)
- `trade.rs`: Trade execution and management
- `format.rs`: Data formatting utilities

#### [§](#pricing-models-pricing){.doc-anchor}**Pricing Models** (`pricing/`)

Advanced pricing engines for options valuation:

- `black_scholes_model.rs`: European options pricing with Greeks
- `binomial_model.rs`: American/European options with early exercise
- `monte_carlo.rs`: Path-dependent and exotic options pricing
- `telegraph.rs`: Jump-diffusion process modeling
- `payoff.rs`: Payoff function implementations

#### [§](#strategies-strategies){.doc-anchor}**Strategies** (`strategies/`)

Comprehensive trading strategy implementations:

- `base.rs`: Core traits (Strategable, BasicAble, Positionable, etc.)
- **Single Leg**: `long_call.rs`, `short_call.rs`, `long_put.rs`,
  `short_put.rs`
- **Spreads**: `bull_call_spread.rs`, `bear_call_spread.rs`,
  `bull_put_spread.rs`, `bear_put_spread.rs`
- **Butterflies**: `long_butterfly_spread.rs`,
  `short_butterfly_spread.rs`, `call_butterfly.rs`
- **Complex**: `iron_condor.rs`, `iron_butterfly.rs`
- **Volatility**: `long_straddle.rs`, `short_straddle.rs`,
  `long_strangle.rs`, `short_strangle.rs`
- **Income**: `covered_call.rs`, `poor_mans_covered_call.rs`
- **Protection**: `protective_put.rs`, `collar.rs`
- `custom.rs`: Flexible custom strategy framework
- `probabilities/`: Probability analysis for strategy outcomes
- `delta_neutral/`: Delta neutrality analysis and adjustment

#### [§](#volatility-volatility){.doc-anchor}**Volatility** (`volatility/`)

Volatility modeling and analysis:

- `utils.rs`: Implied volatility calculation (Newton-Raphson method)
- `traits.rs`: Volatility model interfaces
- Advanced volatility surface construction

#### [§](#greeks-greeks){.doc-anchor}**Greeks** (`greeks/`)

Complete Greeks calculation suite:

- Delta, Gamma, Theta, Vega, Rho calculations
- Real-time sensitivity analysis
- Greeks-based risk management

#### [§](#chains-chains){.doc-anchor}**Chains** (`chains/`)

Option chain management and analysis:

- `chain.rs`: Option chain construction and manipulation
- `utils.rs`: Chain analysis and filtering tools
- CSV/JSON import/export functionality
- Strike price generation algorithms

#### [§](#backtesting-backtesting){.doc-anchor}**Backtesting** (`backtesting/`)

Strategy performance analysis:

- `metrics.rs`: Performance metrics calculation
- `results.rs`: Backtesting results management
- `types.rs`: Backtesting data structures

#### [§](#simulation-simulation){.doc-anchor}**Simulation** (`simulation/`)

Monte Carlo and stochastic simulations:

- Random walk implementations
- Telegraph process modeling
- Custom simulation frameworks
- Parametrized simulation tools

#### [§](#visualization-visualization){.doc-anchor}**Visualization** (`visualization/`)

Comprehensive plotting and charting:

- `plotly.rs`: Interactive charts with Plotly integration
- Strategy payoff diagrams
- Greeks visualization
- 3D volatility surfaces
- Risk profile charts

#### [§](#risk-management-risk){.doc-anchor}**Risk Management** (`risk/`)

Risk analysis and management tools:

- Position risk metrics
- Break-even analysis
- Risk profile generation

#### [§](#pl-pnl){.doc-anchor}**P&L** (`pnl/`)

Profit and loss calculation:

- Real-time P&L tracking
- Historical P&L analysis
- Performance attribution

#### [§](#curves--surfaces-curves-surfaces){.doc-anchor}**Curves & Surfaces** (`curves/`, `surfaces/`) {#curves--surfaces-curves-surfaces}

Mathematical tools for financial modeling:

- Curve interpolation techniques
- Surface construction and analysis
- 3D visualization capabilities

#### [§](#error-handling-error){.doc-anchor}**Error Handling** (`error/`)

Robust error management:

- Comprehensive error types for each module
- Type-safe error propagation
- Detailed error reporting

### [§](#core-components){.doc-anchor}Core Components

::: example-wrap
``` language-mermaid
classDiagram
class Options {
+option_type: OptionType
+side: Side
+underlying_symbol: String
+strike_price: Positive
+expiration_date: ExpirationDate
+implied_volatility: Positive
+quantity: Positive
+underlying_price: Positive
+risk_free_rate: Decimal
+option_style: OptionStyle
+dividend_yield: Positive
+exotic_params: Option~ExoticParams~
+calculate_price_black_scholes()
+calculate_price_binomial()
+time_to_expiration()
+is_long()
+is_short()
+validate()
+to_plot()
+calculate_implied_volatility()
+delta()
+gamma()
+theta()
+vega()
+rho()
}

class Position {
+option: Options
+position_cost: Positive
+entry_date: DateTime<Utc>
+open_fee: Positive
+close_fee: Positive
+net_cost()
+net_premium_received()
+unrealized_pnl()
+pnl_at_expiration()
+validate()
}

class ExpirationDate {
+Days(Positive)
+Date(NaiveDate)
+get_years()
+get_date()
+get_date_string()
+from_string()
}

class Positive {
+value: Decimal
+ZERO: Positive
+ONE: Positive
+format_fixed_places()
+round_to_nice_number()
+is_positive()
}

class OptionStyle {
<<enumeration>>
Call
Put
}

class OptionType {
<<enumeration>>
European
American
}

class Side {
<<enumeration>>
Long
Short
}

class Graph {
<<interface>>
+graph_data()
+graph_config()
+to_plot()
+write_html()
+write_png()
+write_svg()
+write_jpeg()
}

class Greeks {
<<interface>>
+delta()
+gamma()
+theta()
+vega()
+rho()
+calculate_all_greeks()
}

Options --|> Greeks : implements
Options --|> Graph : implements
Position o-- Options : contains
Options *-- OptionStyle : has
Options *-- OptionType : has
Options *-- Side : has
Options *-- ExpirationDate : has
Options *-- Positive : uses
```
:::

### [§](#trading-strategies){.doc-anchor}Trading Strategies

OptionStratLib provides 25+ comprehensive trading strategies organized
by complexity and market outlook:

#### [§](#single-leg-strategies){.doc-anchor}**Single Leg Strategies**

Basic directional strategies for beginners:

- **Long Call**: Bullish strategy with unlimited upside potential
- **Short Call**: Bearish strategy collecting premium with limited
  profit
- **Long Put**: Bearish strategy with high profit potential
- **Short Put**: Bullish strategy collecting premium with assignment
  risk

#### [§](#spread-strategies){.doc-anchor}**Spread Strategies**

Defined risk strategies with limited profit/loss:

- **Bull Call Spread**: Moderately bullish with limited risk and reward
- **Bear Call Spread**: Moderately bearish credit spread
- **Bull Put Spread**: Moderately bullish credit spread
- **Bear Put Spread**: Moderately bearish debit spread

#### [§](#butterfly-strategies){.doc-anchor}**Butterfly Strategies**

Market neutral strategies profiting from low volatility:

- **Long Butterfly Spread**: Profits from price staying near middle
  strike
- **Short Butterfly Spread**: Profits from price moving away from middle
  strike
- **Call Butterfly**: Butterfly using only call options

#### [§](#complex-multi-leg-strategies){.doc-anchor}**Complex Multi-Leg Strategies**

Advanced strategies for experienced traders:

- **Iron Condor**: Market neutral strategy with wide profit zone
- **Iron Butterfly**: Market neutral strategy with narrow profit zone

#### [§](#volatility-strategies){.doc-anchor}**Volatility Strategies**

Strategies that profit from volatility changes:

- **Long Straddle**: Profits from high volatility in either direction
- **Short Straddle**: Profits from low volatility (range-bound market)
- **Long Strangle**: Similar to straddle but with different strikes
- **Short Strangle**: Credit strategy profiting from low volatility

#### [§](#income-generation-strategies){.doc-anchor}**Income Generation Strategies**

Strategies focused on generating regular income:

- **Covered Call**: Stock ownership with call selling for income
- **Poor Man's Covered Call**: LEAPS-based covered call alternative

#### [§](#protection-strategies){.doc-anchor}**Protection Strategies**

Risk management and hedging strategies:

- **Protective Put**: Downside protection for stock positions
- **Collar**: Combination of covered call and protective put

#### [§](#custom-strategy-framework){.doc-anchor}**Custom Strategy Framework**

- **Custom Strategy**: Flexible framework for creating any multi-leg
  strategy
- Supports unlimited number of legs
- Full integration with all analysis tools
- Complete trait implementation for consistency

#### [§](#strategy-analysis-features){.doc-anchor}**Strategy Analysis Features**

All strategies include comprehensive analysis capabilities:

- **Profit/Loss Analysis**: P&L at any price point and time
- **Break-Even Points**: Multiple break-even calculations
- **Greeks Analysis**: Real-time risk metrics
- **Probability Analysis**: Success probability calculations
- **Delta Neutrality**: Delta-neutral position analysis
- **Visualization**: Interactive payoff diagrams and risk profiles
- **Optimization**: Find optimal strikes and expirations

#### [§](#strategy-traits-system){.doc-anchor}**Strategy Traits System**

All strategies implement a comprehensive trait system:

- **Strategable**: Master trait combining all strategy capabilities
- **BasicAble**: Basic strategy information (symbol, price, etc.)
- **Positionable**: Position management and modification
- **Strategies**: Core strategy calculations (P&L, break-even, etc.)
- **Validable**: Strategy validation and error checking
- **BreakEvenable**: Break-even point calculations
- **Profit**: Profit/loss analysis at various price points
- **Greeks**: Greeks calculations for risk management
- **DeltaNeutrality**: Delta-neutral analysis and adjustments
- **ProbabilityAnalysis**: Outcome probability calculations
- **Graph**: Visualization and plotting capabilities

### [§](#setup-instructions){.doc-anchor}Setup Instructions

#### [§](#prerequisites){.doc-anchor}Prerequisites

- Rust 1.80 or higher (2024 edition)
- Cargo package manager

#### [§](#installation){.doc-anchor}Installation

Add OptionStratLib to your `Cargo.toml`:

::: example-wrap
``` language-toml
[dependencies]
optionstratlib = "0.6.1"
```
:::

Or use cargo to add it to your project:

::: example-wrap
``` language-bash
cargo add optionstratlib
```
:::

#### [§](#optional-features){.doc-anchor}Optional Features

The library includes optional features for enhanced functionality:

::: example-wrap
``` language-toml
[dependencies]
optionstratlib = { version = "0.6.1", features = ["plotly"] }
```
:::

- `plotly`: Enables interactive visualization using plotly.rs

#### [§](#building-from-source){.doc-anchor}Building from Source

Clone the repository and build using Cargo:

::: example-wrap
``` language-bash
git clone https://github.com/joaquinbejar/OptionStratLib.git
cd OptionStratLib
cargo build --release
```
:::

Run comprehensive test suite:

::: example-wrap
``` language-bash
cargo test --all-features
```
:::

Generate documentation:

::: example-wrap
``` language-bash
cargo doc --open --all-features
```
:::

Run benchmarks:

::: example-wrap
``` language-bash
cargo bench
```
:::

### [§](#library-usage){.doc-anchor}Library Usage

#### [§](#basic-option-creation-and-pricing){.doc-anchor}Basic Option Creation and Pricing

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{Options, OptionStyle, OptionType, Side, ExpirationDate};
use optionstratlib::pos_or_panic;
use rust_decimal_macros::dec;
use optionstratlib::greeks::Greeks;

// Create a European call option
let option = Options::new(
    OptionType::European,
    Side::Long,
    "AAPL".to_string(),
    pos!(150.0),            // strike_price
    ExpirationDate::Days(pos!(30.0)),
    pos!(0.25),             // implied_volatility
    pos!(1.0),              // quantity
    pos!(155.0),            // underlying_price
    dec!(0.05),             // risk_free_rate
    OptionStyle::Call,
    pos!(0.02),             // dividend_yield
    None,                   // exotic_params
);

// Calculate option price using Black-Scholes
let price = option.calculate_price_black_scholes().unwrap();
tracing::info!("Option price: ${:.2}", price);

// Calculate Greeks for risk management
let delta = option.delta().unwrap();
let gamma = option.gamma().unwrap();
let theta = option.theta().unwrap();
let vega = option.vega().unwrap();
tracing::info!("Greeks - Delta: {:.4}, Gamma: {:.4}, Theta: {:.4}, Vega: {:.4}",
         delta, gamma, theta, vega);
```
:::

#### [§](#working-with-trading-strategies){.doc-anchor}Working with Trading Strategies

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::{Positive, ExpirationDate, pos};
use optionstratlib::strategies::Strategies;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::strategies::base::{BreakEvenable, BasicAble};
use optionstratlib::visualization::Graph;
use rust_decimal_macros::dec;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    use optionstratlib::pricing::Profit;
let underlying_price = pos!(100.0);

    // Create a Bull Call Spread strategy
    let strategy = BullCallSpread::new(
        "AAPL".to_string(),
        underlying_price,
        pos!(95.0),   // long_strike
        pos!(105.0),  // short_strike  
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.25),   // implied_volatility
        dec!(0.05),   // risk_free_rate
        pos!(2.50),   // long_call_premium
        pos!(2.50),   // long_call_open_fee
        pos!(1.20),   // short_call_premium
        pos!(1.20),   // short_call_close_fee
        Default::default(), Default::default(),
        Default::default(), Default::default()
    );

    // Analyze the strategy
    tracing::info!("Strategy: {}", strategy.get_title());
    tracing::info!("Break-even points: {:?}", strategy.get_break_even_points()?);
    tracing::info!("Max profit: ${:.2}", strategy.get_max_profit().unwrap_or(Positive::ZERO));
    tracing::info!("Max loss: ${:.2}", strategy.get_max_loss().unwrap_or(Positive::ZERO));
    tracing::info!("Net premium: ${:.2}", strategy.get_net_premium_received()?);

    // Calculate P&L at different price points
    let prices = vec![pos!(90.0), pos!(95.0), pos!(100.0), pos!(105.0), pos!(110.0)];
    for price in prices {
        let pnl = strategy.get_point_at_price(&price)?;
        tracing::info!("P&L at ${}: ${:.2}", price, pnl.0);
    }

    // Generate visualization
    #[cfg(feature = "plotly")]
    {
        strategy.write_html("Draws/Visualization/bull_call_spread.html".as_ref())?;
    }

    Ok(())
}
```
:::

#### [§](#advanced-features-volatility-analysis){.doc-anchor}Advanced Features: Volatility Analysis

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an option for implied volatility calculation
    let mut option = Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        pos!(105.0), // strike
        ExpirationDate::Days(pos!(90.0)),
        pos!(0.20), // initial IV guess
        pos!(1.0), // quantity
        pos!(100.0), // underlying price
        dec!(0.05), // risk free rate
        OptionStyle::Call,
        pos!(0.02), // dividend yield
        None,
    );

    let market_price = pos!(5.50);
    let iv = implied_volatility(market_price, &mut option, 100)?;

    tracing::info!("Implied volatility: {:.2}%", iv.to_f64() * 100.0);
    Ok(())
}
```
:::

#### [§](#custom-strategy-creation){.doc-anchor}Custom Strategy Creation

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::prelude::*;

// Define common parameters
let underlying_symbol = "DAX".to_string();
let underlying_price = pos!(24000.0);
let expiration = ExpirationDate::Days(pos!(30.0));
let implied_volatility = pos!(0.25);
let risk_free_rate = dec!(0.05);
let dividend_yield = pos!(0.02);
let fee = pos!(2.0);

// Create a long put option
let long_put_option = Options::new(
    OptionType::European,
    Side::Long,
    underlying_symbol.clone(),
    pos!(24070.0), // strike
    expiration.clone(),
    implied_volatility,
    pos!(1.0), // quantity
    underlying_price,
    risk_free_rate,
    OptionStyle::Put,
    dividend_yield,
    None,
);
let long_put = Position::new(
    long_put_option,
    pos!(150.0), // premium
    Utc::now(),
    fee,
    fee,
    None,
    None,
);

// Create a long call option  
let long_call_option = Options::new(
    OptionType::European,
    Side::Long,
    underlying_symbol.clone(),
    pos!(24030.0), // strike
    expiration.clone(),
    implied_volatility,
    pos!(1.0), // quantity
    underlying_price,
    risk_free_rate,
    OptionStyle::Call,
    dividend_yield,
    None,
);
let long_call = Position::new(
    long_call_option,
    pos!(120.0), // premium
    Utc::now(),
    fee,
    fee,
    None,
    None,
);

// Create CustomStrategy with the positions
let positions = vec![long_call, long_put];
let strategy = CustomStrategy::new(
    "DAX Straddle Strategy".to_string(),
    underlying_symbol,
    "A DAX long straddle strategy".to_string(),
    underlying_price,
    positions,
    pos!(1.0),
    30,
    implied_volatility,
);

tracing::info!("Strategy created: {}", strategy.get_title());
```
:::

### [§](#testing){.doc-anchor}Testing

OptionStratLib includes a comprehensive test suite with over 1000 unit
and integration tests:

#### [§](#running-tests){.doc-anchor}**Running Tests**

Run all tests:

::: example-wrap
``` language-bash
cargo test --all-features
```
:::

Run tests for specific modules:

::: example-wrap
``` language-bash
cargo test strategies::bull_call_spread
cargo test pricing::black_scholes
cargo test volatility::utils
```
:::

Run tests with output:

::: example-wrap
``` language-bash
cargo test -- --nocapture
```
:::

#### [§](#test-categories){.doc-anchor}**Test Categories**

- **Unit Tests**: Individual function and method testing
- **Integration Tests**: Cross-module functionality testing
- **Strategy Tests**: Comprehensive strategy validation
- **Pricing Model Tests**: Accuracy and performance testing
- **Greeks Tests**: Mathematical precision validation
- **Visualization Tests**: Chart generation and export testing

#### [§](#benchmarking){.doc-anchor}**Benchmarking**

Run performance benchmarks:

::: example-wrap
``` language-bash
cargo bench
```
:::

Generate test coverage report:

::: example-wrap
``` language-bash
cargo tarpaulin --all-features --out Html
```
:::

### [§](#examples){.doc-anchor}Examples

The library includes extensive examples organized by functionality:

- **`examples/examples_strategies/`**: Complete strategy examples (25+
  strategies)
- **`examples/examples_chains/`**: Option chain analysis examples
- **`examples/examples_pricing/`**: Pricing model demonstrations
- **`examples/examples_visualization/`**: Interactive chart examples
- **`examples/examples_volatility/`**: Volatility analysis examples
- **`examples/examples_simulation/`**: Monte Carlo and simulation
  examples

Run examples:

::: example-wrap
``` language-bash
cargo run --example bull_call_spread --features plotly
cargo run --example black_scholes_pricing
cargo run --example volatility_surface
```
:::

### [§](#contribution-and-contact){.doc-anchor}Contribution and Contact

#### [§](#contributing){.doc-anchor}**Contributing**

Contributions are welcome! Please follow these guidelines:

1.  **Fork** the repository
2.  **Create** a feature branch:
    `git checkout -b feature/amazing-feature`
3.  **Commit** your changes: `git commit -m 'Add amazing feature'`
4.  **Push** to the branch: `git push origin feature/amazing-feature`
5.  **Open** a Pull Request

#### [§](#development-setup){.doc-anchor}**Development Setup**

::: example-wrap
``` language-bash
git clone https://github.com/joaquinbejar/OptionStratLib.git
cd OptionStratLib
cargo build --all-features
cargo test --all-features
```
:::

#### [§](#code-quality){.doc-anchor}**Code Quality**

- All code must pass `cargo clippy` without warnings
- Format code with `cargo fmt`
- Add tests for new functionality
- Update documentation for API changes
- Follow Rust 2024 edition best practices

#### [§](#support){.doc-anchor}**Support**

- **Issues**: Report bugs and request features on GitHub
- **Discussions**: Join community discussions on GitHub Discussions
- **Documentation**: Comprehensive docs available at docs.rs

------------------------------------------------------------------------

**OptionStratLib v0.7.0** - Built with ❤️ in Rust for the financial
community
::::::::::::::::::::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use model::`[`ExpirationDate`](model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}`;`

`pub use model::`[`Options`](model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}`;`

`pub use model::positive::`[`Positive`](model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`;`

`pub use model::types::`[`OptionStyle`](model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}`;`

`pub use model::types::`[`OptionType`](model/types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum}`;`

`pub use model::types::`[`Side`](model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[backtesting](backtesting/index.html "mod optionstratlib::backtesting"){.mod}
:   `backtesting` - Tools for historical performance evaluation of
    options strategies.

[chains](chains/index.html "mod optionstratlib::chains"){.mod}
:   `chains` - Functionality for working with options chains and series
    data.

[constants](constants/index.html "mod optionstratlib::constants"){.mod}
:   `constants` - Library-wide mathematical and financial constants.

[curves](curves/index.html "mod optionstratlib::curves"){.mod}
:   `curves` - Tools for yield curves, term structures, and other
    financial curves.

[error](error/index.html "mod optionstratlib::error"){.mod}
:   `error` - Error types and handling functionality for the library.

[geometrics](geometrics/index.html "mod optionstratlib::geometrics"){.mod}
:   `geometrics` - Mathematical utilities for geometric calculations
    relevant to options.

[greeks](greeks/index.html "mod optionstratlib::greeks"){.mod}
:   `greeks` - Calculation and management of option sensitivity metrics
    (Delta, Gamma, etc.).

[model](model/index.html "mod optionstratlib::model"){.mod}
:   `model` - Core data structures and models for options and
    derivatives.

[pnl](pnl/index.html "mod optionstratlib::pnl"){.mod}
:   `pnl` - Profit and loss analysis tools for options positions.

[prelude](prelude/index.html "mod optionstratlib::prelude"){.mod}
:   `prelude` - Convenient re-exports of commonly used types and traits.

[pricing](pricing/index.html "mod optionstratlib::pricing"){.mod}
:   `pricing` - Option pricing models including Black-Scholes and
    numerical methods.

[risk](risk/index.html "mod optionstratlib::risk"){.mod}
:   `risk` - Risk assessment and management tools for options
    portfolios.

[series](series/index.html "mod optionstratlib::series"){.mod}
:   `series` - Functionality for working with collections of option
    chains across expirations.

[simulation](simulation/index.html "mod optionstratlib::simulation"){.mod}
:   `simulation` - Simulation techniques for scenario analysis.

[strategies](strategies/index.html "mod optionstratlib::strategies"){.mod}
:   `strategies` - Pre-defined option strategy templates and building
    blocks.

[surfaces](surfaces/index.html "mod optionstratlib::surfaces"){.mod}
:   `surfaces` - Volatility surface and other 3D financial data
    modeling.

[utils](utils/index.html "mod optionstratlib::utils"){.mod}
:   `utils` - General utility functions for data manipulation and
    calculations.

[visualization](visualization/index.html "mod optionstratlib::visualization"){.mod}
:   `visualization` - Tools for plotting and visual representation of
    options data.

[volatility](volatility/index.html "mod optionstratlib::volatility"){.mod}
:   `volatility` - Volatility modeling, forecasting, and analysis
    utilities.

## Macros[§](#macros){.anchor} {#macros .section-header}

[assert_decimal_eq](macro.assert_decimal_eq.html "macro optionstratlib::assert_decimal_eq"){.macro}
:   Asserts that two Decimal values are approximately equal within a
    given epsilon

[assert_pos_relative_eq](macro.assert_pos_relative_eq.html "macro optionstratlib::assert_pos_relative_eq"){.macro}
:   Asserts that two `Positive` values are relatively equal within a
    given epsilon.

[d2f](macro.d2f.html "macro optionstratlib::d2f"){.macro}
:   Converts a Decimal value to f64 with error propagation.

[d2fu](macro.d2fu.html "macro optionstratlib::d2fu"){.macro}
:   Converts a Decimal value to f64 without error checking.

[f2d](macro.f2d.html "macro optionstratlib::f2d"){.macro}
:   Converts an f64 value to Decimal with error propagation.

[f2du](macro.f2du.html "macro optionstratlib::f2du"){.macro}
:   Converts an f64 value to Decimal without error checking.

[impl_graph_for_payoff_strategy](macro.impl_graph_for_payoff_strategy.html "macro optionstratlib::impl_graph_for_payoff_strategy"){.macro}
:   Macro `impl_graph_for_payoff_strategy` generates implementations of
    the `Graph` trait for one or more given types. This is specifically
    designed for types that represent payoff strategies, enabling them
    to produce graph data and configurations for financial
    visualizations, such as profit/loss graphs based on an underlying
    price range.

[pos](macro.pos.html "macro optionstratlib::pos_or_panic"){.macro}
:   Macro for creating a new `Positive` value with simplified syntax.

[spos](macro.spos.html "macro optionstratlib::spos"){.macro}
:   A macro to create an optional `Positive` value from the given
    expression.

[test_strategy_traits](macro.test_strategy_traits.html "macro optionstratlib::test_strategy_traits"){.macro}
:   Macro to test trait implementations for a specific strategy type.

## Constants[§](#constants){.anchor} {#constants .section-header}

[VERSION](constant.VERSION.html "constant optionstratlib::VERSION"){.constant}
:   Library version

## Functions[§](#functions){.anchor} {#functions .section-header}

[version](fn.version.html "fn optionstratlib::version"){.fn}
:   Returns the library version
::::::::::::::::::::::::
:::::::::::::::::::::::::
