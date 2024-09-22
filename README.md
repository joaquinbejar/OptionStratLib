
<div style="text-align: center;">
  <img src="doc/images/logo.png" alt="OptionStratLib" style="width: 100%; height: 200px;">
</div>

[![Dual License](https://img.shields.io/badge/license-MIT%20and%20Apache%202.0-blue)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/optionstratlib.svg)](https://crates.io/crates/optionstratlib)
[![Downloads](https://img.shields.io/crates/d/optionstratlib.svg)](https://crates.io/crates/optionstratlib)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/stargazers)

[![Build Status](https://img.shields.io/github/workflow/status/joaquinbejar/OptionStratLib/CI)](https://github.com/joaquinbejar/OptionStratLib/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/OptionStratLib)](https://codecov.io/gh/joaquinbejar/OptionStratLib)
[![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/OptionStratLib)](https://libraries.io/github/joaquinbejar/OptionStratLib)

# OptionStratLib v0.1.1: Financial Options Library

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

1. **Valuation Models**: Implements Black-Scholes and binomial models for option pricing.
2. **Greeks Calculation**: Calculates delta, gamma, theta, vega, and rho for sensitivity analysis.
3. **Option Types**: Supports European and American options, both calls and puts.
4. **Risk Analysis**: Includes VaR (Value at Risk) calculations and other risk metrics. **TODO!**
5. **Simulations**: Allows Monte Carlo simulations for scenario analysis.
6. **Exotic Options**: Supports some types of exotic options such as Asian and lookback options. **TODO!**
7. **Strategy Development**: Provides tools for creating and backtesting option trading strategies.
8. **Performance Visualization**: Generates payoff diagrams and risk profiles for visual analysis.
9. **Multi-Asset Support**: **TODO!**
10. **Risk Management**: **TODO!**
11. **Backtesting**: **TODO!**
12. **Performance Metrics**: Generates performance metrics for strategy evaluation and Positions.


## Project Structure

The project is structured as follows:

1. **Options Module** (`option.rs`): This module contains the core implementation of option-related structures and methods. It includes the `Options` struct that represents an option contract, along with methods for pricing, calculating Greeks, and managing the profit and loss (P&L) of the option.

2. **Pricing Models** (`pricing/`):
    - `binomial_model.rs`: Implements the binomial model for pricing options.
    - `black_scholes_model.rs`: Implements the Black-Scholes model for pricing options.
    - `monte_carlo.rs`: Provides tools for running Monte Carlo simulations to price options.
    - `payoff.rs`: Defines the payoff functions for different option types.
    - `telegraph.rs`: Implements the Telegraphic method for pricing options.
    - `utils.rs`: Utility functions related to option pricing.

3. **Greeks Calculation** (`greeks/`):
    - `equations.rs`: Contains the mathematical equations for calculating various Greeks (delta, gamma, theta, vega, rho).
    - `utils.rs`: Utility functions for working with Greeks.

4. **Profit and Loss (P&L)** (`pnl/`):
    - `utils.rs`: Implements functions for calculating the P&L of option positions.

5. **Risk Management** (`risk/`):
    - This module will contain implementations of risk metrics and management strategies, such as Value-at-Risk (VaR), Expected Shortfall, and risk-based portfolio optimization.

6. **Strategies** (`strategies/`):
    - `base.rs`: Defines the base traits and structures for option trading strategies.
    - `bear_put_spread.rs`, `bull_call_spread.rs`, `butterfly_spread.rs`, `collar.rs`, `covered_call.rs`, `iron_condor.rs`, `protective_put.rs`, `straddle.rs`, `strangle.rs`: Implementations of various option trading strategies.
    - `utils.rs`: Utility functions for working with option trading strategies.

7. **Curves and Surfaces** (`curves/`, `surfaces/`):
    - These modules will contain functionality for constructing, analyzing, and visualizing yield curves, volatility surfaces, and other financial curves and surfaces.

8. **Visualization** (`visualization/`):
    - `binomial_tree.rs`: Visualization of binomial option pricing trees.
    - `strategy.rs`: Visualization of option trading strategies.
    - `utils.rs`: Utility functions for creating visualizations.

9. **Volatility** (`volatility/`):
    - `utils.rs`: Utility functions for working with volatility.

10. **Backtesting** (`backtesting/`):
    - This module will contain the necessary functionality for performing backtesting of trading strategies.

11. **Utility Modules**:
    - `constants.rs`: Defines common constants used throughout the project.
    - `model/`: Contains structs and types used to represent various financial concepts, such as options, positions, and formats.


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
use optionstratlib::model::option::Options;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use optionstratlib::greeks::equations::Greeks;

fn create_sample_option() -> Options {
   Options::new(
      OptionType::European,
      Side::Short,
      "AAPL".to_string(),
      100.0,
      ExpirationDate::Days(30.0),
      0.2,
      1,
      105.0,
      0.05,
      OptionStyle::Call,
      0.0,
      None,
   )
}
fn main() -> Result<(), Box<dyn Error>> {
   let option = create_sample_option();
   info!("Title: {}", option.title());
   info!("Greeks: {:?}", option.greeks());

   // Define a range of prices for the graph
   let price_range: Vec<f64> = (50..150).map(|x| x as f64).collect();

   // Generate the intrinsic value graph
   option.graph(&price_range,
                "Draws/Options/intrinsic_value_chart.png",
                25,
                (1400, 933),
                (10, 30),
                10
   )?;

   Ok(())
}
```

```rust
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
   let strategy = BullCallSpread::new(
      "GOLD".to_string(),
      2505.8,
      2460.0,
      2515.0,
      ExpirationDate::Days(30.0),
      0.2,
      0.05,
      0.0,
      1,
      27.26,
      5.33,
      0.58,
      0.58,
      0.55,
      0.55,
   );
   let price_range: Vec<f64> = (2400..2600).map(|x| x as f64).collect();
   info!("Title: {}", strategy.title());
   info!("Break Even {}", strategy.break_even());
   info!("Net Premium Received: {}", strategy.net_premium_received());
   info!("Max Profit: {}", strategy.max_profit());
   info!("Max Loss: {}", strategy.max_loss());
   info!("Total Cost: {}", strategy.total_cost());

   // Generate the intrinsic value graph
   strategy.graph(
      &price_range,
      "Draws/Strategy/bull_call_spread_value_chart.png",
      20,
      (1400, 933),
      (10, 30),
      15
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
