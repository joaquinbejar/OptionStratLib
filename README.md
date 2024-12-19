# optionstratlib


 <div style="text-align: center;">
 <img src="https://raw.githubusercontent.com/joaquinbejar/OptionStratLib/refs/heads/main/doc/images/logo.png" alt="OptionStratLib" style="width: 100%; height: 200px;">
 </div>

 [![Dual License](https://img.shields.io/badge/license-MIT%20and%20Apache%202.0-blue)](./LICENSE)
 [![Crates.io](https://img.shields.io/crates/v/optionstratlib.svg)](https://crates.io/crates/optionstratlib)
 [![Downloads](https://img.shields.io/crates/d/optionstratlib.svg)](https://crates.io/crates/optionstratlib)
 [![Stars](https://img.shields.io/github/stars/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/stargazers)
 [![Issues](https://img.shields.io/github/issues/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/issues)
 [![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/pulls)


 [![Build Status](https://img.shields.io/github/workflow/status/joaquinbejar/OptionStratLib/CI)](https://github.com/joaquinbejar/OptionStratLib/actions)
 [![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/OptionStratLib)](https://codecov.io/gh/joaquinbejar/OptionStratLib)
 [![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/OptionStratLib)](https://libraries.io/github/joaquinbejar/OptionStratLib)
 [![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/optionstratlib)

 # OptionStratLib v0.2.6: Financial Options Library

 ## Table of Contents
 1. [Introduction](#introduction)
 2. [Features](#features)
 3. [Project Structure](#project-structure)
 4. [Setup Instructions](#setup-instructions)
 5. [Library Usage](#library-usage)
 6. [Usage Examples](#usage-examples)
 7. [Testing](#testing)
 8. [Contribution and Contact](#contribution-and-contact)

 ## Introduction

 OptionStratLib is a comprehensive Rust library for options trading and strategy development across multiple asset classes. This versatile toolkit enables traders, quants, and developers to:

 ## Features

 1. **Valuation Models**:
 - Black-Scholes model
 - Binomial model
 - Monte Carlo simulations
 - Telegraph process model

 2. **Greeks Calculation**:
 - Delta, gamma, theta, vega, and rho
 - Custom Greeks implementation
 - Greeks visualization

 3. **Option Types**:
 - European and American options
 - Calls and puts
 - Support for exotic options (Asian, Barrier, etc.)

 4. **Volatility Models**:
 - Constant volatility
 - EWMA (Exponentially Weighted Moving Average)
 - GARCH implementation
 - Heston stochastic volatility
 - Volatility surface interpolation

 5. **Option Chain Management**:
 - Chain construction and analysis
 - Strike price generation
 - Chain data import/export (CSV/JSON)

 6. **Trading Strategies**:
 - Bull Call Spread
 - Bear Put Spread
 - Call Butterfly
 - Strategy optimization
 - Custom strategy development

 7. **Risk Management**:
 - SPAN margin calculation
 - Position tracking
 - Break-even analysis
 - Profit/Loss calculations

 8. **Simulation Tools**:
 - Random Walk simulation
 - Telegraph process
 - Monte Carlo methods
 - Custom simulation frameworks

 9. **Visualization**:
 - Strategy payoff diagrams
 - Greeks visualization
 - Binomial trees
 - Risk profiles
 - Interactive charts

 10. **Data Management**:
 - CSV/JSON import/export
 - Option chain data handling
 - Historical data analysis
 - Price series management

 11. **Backtesting**: **TODO!**
 12. **Performance Metrics**: **TODO!**


 ## Project Structure

 The project is organized into the following key modules:

 1. **Core Options** (`options/`):
 - `option.rs`: Core option structures and methods
 - `position.rs`: Position management
 - `chain.rs`: Option chain handling

 2. **Pricing Models** (`pricing/`):
 - `binomial_model.rs`: Binomial tree implementation
 - `black_scholes_model.rs`: Black-Scholes pricing
 - `monte_carlo.rs`: Monte Carlo simulations
 - `telegraph.rs`: Telegraph process model

 3. **Greeks** (`greeks/`):
 - `equations.rs`: Greeks calculations
 - `utils.rs`: Greek utilities

 4. **Volatility** (`volatility/`):
 - `constant.rs`: Constant volatility model
 - `ewma.rs`: EWMA implementation
 - `garch.rs`: GARCH model
 - `heston.rs`: Heston model
 - `surface.rs`: Volatility surface handling

 5. **Strategies** (`strategies/`):
 - `base.rs`: Strategy base traits
 - `bear_put_spread.rs`:
 - `bull_call_spread.rs`:
 - `butterfly_spread.rs`:
 - `call_butterfly.rs`:
 - `collar.rs`:
 - `covered_call.rs`:
 - `custom.rs`: Custom strategy framework
 - `iron_condor.rs`:
 - `poor_mans_covered_call.rs`:
 - `protective_put.rs`:
 - `straddle.rs`:
 - `strangle.rs`:
 - `utils.rs`: Strategy utilities

 6. **Risk Management** (`risk/`):
 - `span.rs`: SPAN margin calculation
 - `margin.rs`: Margin requirements
 - `position.rs`: Position risk metrics

 7. **Simulation** (`simulation/`):
 - `random_walk.rs`
 - `telegraph.rs`
 - `monte_carlo.rs`

 8. **Visualization** (`visualization/`):
 - `binomial_tree.rs`
 - `strategy.rs`
 - `utils.rs`

 9. **Data Management** (`data/`):
 - `chain.rs`: Chain data structures
 - `import.rs`: Data import utilities
 - `export.rs`: Data export utilities

 ## Relationships

 ### Base Structure
 ```mermaid
 classDiagram
 class Options {
 +option_type: OptionType
 +side: Side
 +underlying_symbol: String
 +strike_price: PositiveF64
 +expiration_date: ExpirationDate
 +implied_volatility: f64
 +quantity: PositiveF64
 +underlying_price: PositiveF64
 +risk_free_rate: f64
 +option_style: OptionStyle
 +dividend_yield: f64
 +exotic_params: Option~ExoticParams~
 +calculate_price_black_scholes()
 +calculate_price_binomial()
 +time_to_expiration()
 +is_long()
 +is_short()
 +validate()
 }

 class OptionType {
 <<enumeration>>
 European
 American
 Bermuda
 Asian
 Barrier
 Binary
 Lookback
 Compound
 Chooser
 Cliquet
 Rainbow
 Spread
 Quanto
 Exchange
 Power
 }

 class Side {
 <<enumeration>>
 Long
 Short
 }

 class OptionStyle {
 <<enumeration>>
 Call
 Put
 }

 class Position {
 +option: Options
 +premium: f64
 +date: DateTime
 +open_fee: f64
 +close_fee: f64
 +total_cost()
 +unrealized_pnl()
 +days_held()
 +days_to_expiration()
 +is_long()
 +is_short()
 +break_even()
 }

 class Strategies {
 <<interface>>
 +add_leg()
 +get_legs()
 +break_even()
 +max_profit()
 +max_loss()
 +total_cost()
 +net_premium_received()
 +fees()
 }

 class BullCallSpread {
 +long_call: Position
 +short_call: Position
 +break_even()
 +max_profit()
 +max_loss()
 }

 class CallButterfly {
 +long_call_itm: Position
 +short_call: Position
 +long_call_otm: Position
 +break_even()
 +max_profit()
 +max_loss()
 }

 class Greeks {
 <<interface>>
 +delta()
 +gamma()
 +theta()
 +vega()
 +rho()
 +rho_d()
 }

 class Profit {
 <<interface>>
 +calculate_profit_at()
 }

 class Graph {
 <<interface>>
 +graph()
 +title()
 +get_values()
 }

 class PnLCalculator {
 <<interface>>
 +calculate_pnl()
 +calculate_pnl_at_expiration()
 }

 Options --|> Greeks
 Options --|> Profit
 Options --|> Graph
 Position *-- Options
 Position --|> Greeks
 Position --|> Profit
 Position --|> Graph
 Position --|> PnLCalculator
 BullCallSpread --|> Strategies
 BullCallSpread --|> Profit
 BullCallSpread --|> Graph
 CallButterfly --|> Strategies
 CallButterfly --|> Profit
 CallButterfly --|> Graph
 Options o-- OptionType
 Options o-- Side
 Options o-- OptionStyle
 ```

 ### Strategy Structure
 ```mermaid
 classDiagram
 class Options {
 +option_type: OptionType
 +side: Side
 +strike_price: PositiveF64
 +expiration_date: ExpirationDate
 +implied_volatility: f64
 +calculate_price_black_scholes()
 +calculate_price_binomial()
 +calculate_delta()
 +payoff()
 }

 class Position {
 +option: Options
 +premium: f64
 +date: DateTime
 +open_fee: f64
 +close_fee: f64
 +total_cost()
 +unrealized_pnl()
 +days_held()
 }

 class OptionChain {
 +symbol: String
 +underlying_price: PositiveF64
 +options: BTreeSet<OptionData>
 +build_chain()
 +add_option()
 +save_to_csv()
 +load_from_csv()
 }

 class Strategy {
 <<Interface>>
 +add_leg()
 +get_legs()
 +break_even()
 +max_profit()
 +max_loss()
 +total_cost()
 }

 class BullCallSpread {
 +long_call: Position
 +short_call: Position
 +break_even_points: Vec<f64>
 +calculate_profit_at()
 }

 class CallButterfly {
 +long_call_itm: Position
 +long_call_otm: Position
 +short_call: Position
 +break_even_points: Vec<f64>
 }

 class Graph {
 <<Interface>>
 +title()
 +get_values()
 +get_vertical_lines()
 +get_points()
 }

 class Profit {
 <<Interface>>
 +calculate_profit_at()
 }

 class Greeks {
 <<Interface>>
 +delta()
 +gamma()
 +theta()
 +vega()
 }

 Position o-- Options
 Strategy <|.. BullCallSpread
 Strategy <|.. CallButterfly
 Graph <|.. Options
 Graph <|.. Position
 Graph <|.. Strategy
 Profit <|.. Options
 Profit <|.. Position
 Profit <|.. Strategy
 Greeks <|.. Options
 OptionChain o-- Options
 BullCallSpread o-- Position
 CallButterfly o-- Position
 ```

 ## Strategies Classifications

 ```mermaid
 ---
 config:
 layout: fixed
 ---
 flowchart TD
 start["Options Strategies"] --> bullish["Bullish"] & bearish["Bearish"] & neutral["Neutral"]
 bullish --> bull_high["High Volatility"] & bull_low["Low Volatility"]
 bull_high --> bull_high_lim["Limited Risk"] & bull_high_unlim["Unlimited Risk"]
 bull_low --> bull_low_lim["Limited Risk"] & bull_low_unlim["Unlimited Risk"]
 bull_high_lim --> bull_high_lim_opt["Options Only"] & bull_high_lim_stock["With Underlying"]
 bull_high_unlim --> bull_high_unlim_opt["Options Only"] & bull_high_unlim_stock["With Underlying"]
 bull_low_lim --> bull_low_lim_opt["Options Only"] & bull_low_lim_stock["With Underlying"]
 bull_low_unlim --> bull_low_unlim_opt["Options Only"] & bull_low_unlim_stock["With Underlying"]
 bull_high_lim_opt --> bull_call(("Bull Call Spread"))
 bull_high_lim_stock --> protective_put(("Protective Put"))
 bull_high_unlim_opt --> long_call(("Long Call"))
 bull_high_unlim_stock --> pmcc@{ label: "Poor Man's Covered Call" }
 bull_low_lim_opt --> bull_put(("Bull Put Spread"))
 bull_low_lim_stock --> collar(("Collar"))
 bull_low_unlim_opt --> naked_put(("Naked Put"))
 bull_low_unlim_stock --> covered_call(("Covered Call"))
 bearish --> bear_high["High Volatility"] & bear_low["Low Volatility"]
 bear_high --> bear_high_lim["Limited Risk"] & bear_high_unlim["Unlimited Risk"]
 bear_low --> bear_low_lim["Limited Risk"] & bear_low_unlim["Unlimited Risk"]
 bear_high_lim --> bear_high_lim_opt["Options Only"] & bear_high_lim_stock["With Underlying"]
 bear_high_unlim --> bear_high_unlim_opt["Options Only"] & bear_high_unlim_stock["With Underlying"]
 bear_low_lim --> bear_low_lim_opt["Options Only"] & bear_low_lim_stock["With Underlying"]
 bear_low_unlim --> bear_low_unlim_opt["Options Only"] & bear_low_unlim_stock["With Underlying"]
 bear_high_lim_opt --> bear_put(("Bear Put Spread"))
 bear_high_lim_stock --> synthetic_put(("Synthetic Put"))
 bear_high_unlim_opt --> long_put(("Long Put"))
 bear_high_unlim_stock --> covered_put(("Covered Put"))
 bear_low_lim_opt --> bear_call(("Bear Call Spread"))
 bear_low_lim_stock --> reverse_collar(("Reverse Collar"))
 bear_low_unlim_opt --> naked_call(("Naked Call"))
 bear_low_unlim_stock --> protective_call(("Protective Call"))
 neutral --> neut_high["High Volatility"] & neut_low["Low Volatility"]
 neut_high --> neut_high_lim["Limited Risk"] & neut_high_unlim["Unlimited Risk"]
 neut_low --> neut_low_lim["Limited Risk"] & neut_low_unlim["Unlimited Risk"]
 neut_high_lim --> neut_high_lim_opt["Options Only"] & neut_high_lim_stock["With Underlying"]
 neut_high_unlim --> neut_high_unlim_opt["Options Only"] & neut_high_unlim_stock["With Underlying"]
 neut_low_lim --> neut_low_lim_opt["Options Only"] & neut_low_lim_stock["With Underlying"]
 neut_low_unlim --> neut_low_unlim_opt["Options Only"] & neut_low_unlim_stock["With Underlying"]
 neut_high_lim_opt --> call_butterfly(("Call Butterfly")) & butterfly_spread(("Butterfly Spread")) & long_straddle(("Long Straddle")) & long_strangle(("Long Strangle"))
 neut_high_unlim_opt --> short_straddle(("Short Straddle")) & short_strangle(("Short Strangle"))
 neut_low_lim_opt --> iron_butterfly(("Iron Butterfly")) & iron_condor(("Iron Condor"))
 neut_low_unlim_opt --> calendar_spread(("Calendar Spread")) & box_spread(("Box Spread"))
 neut_high_lim_stock --> conversion(("Conversion"))
 neut_high_unlim_stock --> reversal(("Reversal"))
 neut_low_lim_stock --> married_combo(("Married Combo"))
 neut_low_unlim_stock --> ratio_spread(("Ratio Spread"))
 pmcc@{ shape: circle}

 ```

 ## Setup Instructions

 1. Clone the repository:
 ```shell
 git clone https://github.com/joaquinbejar/OptionStratLib.git
 cd OptionStratLib
 ```

 2. Build the project:
 ```shell
 make build
 ```

 3. Run tests:
 ```shell
 make test
 ```

 4. Format the code:
 ```shell
 make fmt
 ```

 5. Run linting:
 ```shell
 make lint
 ```

 6. Clean the project:
 ```shell
 make clean
 ```

 7. Run the project:
 ```shell
 make run
 ```

 8. Fix issues:
 ```shell
 make fix
 ```

 9. Run pre-push checks:
 ```shell
 make pre-push
 ```

 10. Generate documentation:
 ```shell
 make doc
 ```

 11. Publish the package:
 ```shell
 make publish
 ```

 12. Generate coverage report:
 ```shell
 make coverage
 ```



 ## Library Usage

 To use the library in your project, add the following to your `Cargo.toml`:

 ```toml
 [dependencies]
 optionstratlib = { git = "https://github.com/joaquinbejar/OptionStratLib.git" }
 ```

 ## Usage Examples

 Here are some examples of how to use the library for option pricing and analysis:

 ```rust
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::option::Options;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::pos;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn create_sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        pos!(100.0),
        ExpirationDate::Days(30.0),
        0.2,
        pos!(1.0),
        pos!(105.0),
        0.05,
        OptionStyle::Call,
        0.0,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let option = create_sample_option();
    info!("Title: {}", option.title());
    info!("Greeks: {:?}", option.greeks());

    // Define a range of prices for the graph
    let price_range: Vec<PositiveF64> = (50..150)
        .map(|x| PositiveF64::new(x as f64).unwrap())
        .collect();

    // Generate the intrinsic value graph
    option.graph(
        &price_range,
        "Draws/Options/intrinsic_value_chart.png",
        25,
        (1400, 933),
    )?;

    Ok(())
}
 ```

```rust
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, PZERO};
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(5781.88);

    let strategy = BullCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5820.0),     // short_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(2.0), // long quantity
        85.04,     // premium_long
        29.85,     // premium_short
        0.78,      // open_fee_long
        0.78,      // open_fee_long
        0.73,      // close_fee_long
        0.73,      // close_fee_short
    );

    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit().unwrap_or(PZERO));
    info!("Max Loss: ${:0.2}", strategy.max_loss().unwrap_or(PZERO));
    info!("Total Fees: ${:.2}", strategy.fees());
    info!("Profit Area: {:.2}%", strategy.profit_area());
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio());

    // Generate the profit/loss graph
    strategy.graph(
        &price_range,
        "Draws/Strategy/bull_call_spread_profit_loss_chart.png",
        20,
        (1400, 933),
    )?;

    Ok(())
}
```

 ## Testing

 To run unit tests:
 ```shell
 make test
 ```

 To run tests with coverage:
 ```shell
 make coverage
 ```

 ## Contribution and Contact

 We welcome contributions to this project! If you would like to contribute, please follow these steps:

 1. Fork the repository.
 2. Create a new branch for your feature or bug fix.
 3. Make your changes and ensure that the project still builds and all tests pass.
 4. Commit your changes and push your branch to your forked repository.
 5. Submit a pull request to the main repository.

 If you have any questions, issues, or would like to provide feedback, please feel free to contact the project maintainer:

 **Joaquín Béjar García**
 - Email: jb@taunais.com
 - GitHub: [joaquinbejar](https://github.com/joaquinbejar)

 We appreciate your interest and look forward to your contributions!


License: MIT
