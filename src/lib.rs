#![allow(unknown_lints)]
#![allow(clippy::literal_string_with_formatting_args)]

//! # OptionStratLib v0.5.0: Financial Options Library
//!
//! ## Table of Contents
//! 1. [Introduction](#introduction)
//! 2. [Features](#features)
//! 3. [Project Structure](#project-structure)
//! 4. [Core Components](#core-components)
//! 5. [Strategies Classification](#strategies-classification)
//! 6. [Setup Instructions](#setup-instructions)
//! 7. [Library Usage](#library-usage)
//! 8. [Usage Examples](#usage-examples)
//! 9. [Testing](#testing)
//! 10. [Contribution and Contact](#contribution-and-contact)
//!
//! ## Introduction
//!
//! OptionStratLib is a comprehensive Rust library for options trading and strategy development across multiple asset classes. This versatile toolkit enables traders, quants, and developers to model, analyze, and visualize options strategies with a robust, type-safe approach. The library focuses on precision with decimal-based calculations, extensive test coverage, and a modular architecture.
//!
//! ## Features
//!
//! 1. **Valuation Models**:
//! - Black-Scholes model for European options pricing
//! - Binomial tree model for American and European options
//! - Monte Carlo simulations for complex pricing scenarios
//! - Telegraph process model for advanced stochastic modeling
//!
//! 2. **Greeks Calculation**:
//! - Delta, gamma, theta, vega, and rho calculations
//! - Custom Greeks implementation with adjustable parameters
//! - Greeks visualization for risk analysis
//! - Sensitivity analysis tools
//!
//! 3. **Option Types**:
//! - European and American options
//! - Calls and puts with customizable parameters
//! - Support for exotic options (Asian, Barrier, Binary, etc.)
//! - Comprehensive validation and error handling
//!
//! 4. **Volatility Models**:
//! - Constant volatility implementation
//! - EWMA (Exponentially Weighted Moving Average)
//! - GARCH implementation for volatility forecasting
//! - Heston stochastic volatility model
//! - Volatility surface interpolation techniques
//!
//! 5. **Option Chain Management**:
//! - Chain construction and analysis tools
//! - Strike price generation algorithms
//! - Chain data import/export (CSV/JSON)
//! - Filtering and selection tools
//!
//! 6. **Trading Strategies**:
//! - Bull Call/Put Spreads
//! - Bear Call/Put Spreads
//! - Butterfly Spreads (Long/Short)
//! - Iron Condor/Butterfly
//! - Straddles and Strangles
//! - Covered Calls and Protective Puts
//! - Strategy optimization framework
//! - Custom strategy development tools
//!
//! 7. **Risk Management**:
//! - SPAN margin calculation
//! - Position tracking and management
//! - Break-even analysis
//! - Profit/Loss calculations at various price points
//! - Risk profiles and visualizations
//!
//! 8. **Simulation Tools**:
//! - Random Walk simulation
//! - Telegraph process implementation
//! - Monte Carlo methods for scenario analysis
//! - Custom simulation frameworks for strategy testing
//! - Parametrized simulations with adjustable inputs
//!
//! 9. **Visualization**:
//! - Strategy payoff diagrams
//! - Greeks visualization
//! - Binomial trees
//! - Risk profiles
//! - Interactive charts (powered by `plotly.rs`)
//! - Robust generation of various plot types (scatter, surface) for different data representations
//! - Comprehensive test coverage for visualization components, ensuring reliability
//!
//! 10. **Data Management**:
//! - CSV/JSON import/export functionality
//! - Option chain data handling and processing
//! - Historical data analysis tools
//! - Price series management and manipulation
//! - Efficient decimal-based storage
//!
//! 11. **Geometry and Curve Tools**:
//! - Curve interpolation techniques
//! - Surface construction and analysis
//! - 3D visualization of option surfaces
//! - Custom geometric operations
//!
//! 12. **Backtesting** (In Development):
//! - Historical strategy performance evaluation
//! - Parameter optimization
//! - Performance metrics calculation
//!
//! 13. **Performance Metrics** (In Development):
//! - Sharpe ratio, Sortino ratio
//! - Maximum drawdown analysis
//! - Win/loss ratio calculations
//! - Return distributions
//!
//! ## Project Structure
//!
//! The project is organized into the following key modules:
//!
//! 1. **Model** (`model/`):
//! - `option.rs`: Core option structures and methods
//! - `position.rs`: Position management
//! - `expiration.rs`: Expiration date handling
//! - `positive.rs`: Non-negative number type implementation
//! - `types.rs`: Common enums and types
//!
//! 2. **Pricing Models** (`pricing/`):
//! - `binomial_model.rs`: Binomial tree implementation
//! - `black_scholes_model.rs`: Black-Scholes pricing
//! - `monte_carlo.rs`: Monte Carlo simulations
//! - `telegraph.rs`: Telegraph process model
//! - `payoff.rs`: Payoff function implementations
//!
//! 3. **Greeks** (`greeks/`):
//! - Base trait definition for Greeks calculations
//! - Implementation for different option models
//! - Sensitivity analysis tools
//!
//! 4. **Volatility** (`volatility/`):
//! - `constant.rs`: Constant volatility model
//! - `ewma.rs`: EWMA implementation
//! - `garch.rs`: GARCH model
//! - `heston.rs`: Heston model
//! - `surface.rs`: Volatility surface handling
//! - `traits.rs`: Common interfaces for volatility models
//!
//! 5. **Strategies** (`strategies/`):
//! - `base.rs`: Strategy base traits and interfaces
//! - Individual strategy implementations:
//! - `bear_put_spread.rs`, `bull_call_spread.rs`, etc.
//! - `build/`: Strategy construction tools
//! - `custom.rs`: Custom strategy framework
//! - `probabilities/`: Probability analysis for strategies
//!
//! 6. **Risk Management** (`risk/`):
//! - `span.rs`: SPAN margin calculation
//! - `margin.rs`: Margin requirements computation
//! - `position.rs`: Position risk metrics
//!
//! 7. **Simulation** (`simulation/`):
//! - `random_walk.rs`: Random walk implementations
//! - `telegraph.rs`: Telegraph process modeling
//! - `monte_carlo.rs`: Monte Carlo simulation tools
//! - `steps/`: Step generation for simulations
//! - `params.rs`: Simulation parameters
//!
//! 8. **Visualization** (`visualization/`):
//! - `plotly.rs`: Plotly integration for interactive charts
//! - Graph trait implementations
//! - Utility functions for visualization
//!
//! 9. **Geometrics** (`geometrics/`):
//! - `construction/`: Tools for building geometric structures
//! - `interpolation/`: Interpolation techniques
//! - `analysis/`: Analysis tools for curves and surfaces
//! - `visualization/`: Visualization of geometric structures
//!
//! 10. **Error Handling** (`error/`):
//! - Comprehensive error types for each module
//! - Error propagation and handling utilities
//!
//! ## Core Components
//!
//! ```mermaid
//! classDiagram
//! class Options {
//! +option_type: OptionType
//! +side: Side
//! +underlying_symbol: String
//! +strike_price: Positive
//! +expiration_date: ExpirationDate
//! +implied_volatility: Positive
//! +quantity: Positive
//! +underlying_price: Positive
//! +risk_free_rate: Decimal
//! +option_style: OptionStyle
//! +dividend_yield: Positive
//! +exotic_params: Option~ExoticParams~
//! +calculate_price_black_scholes()
//! +calculate_price_binomial()
//! +time_to_expiration()
//! +is_long()
//! +is_short()
//! +validate()
//! +to_plot()
//! +calculate_implied_volatility()
//! +delta()
//! +gamma()
//! +theta()
//! +vega()
//! +rho()
//! }
//!
//! class Position {
//! +option: Options
//! +position_cost: Positive
//! +entry_date: DateTime<Utc>
//! +open_fee: Positive
//! +close_fee: Positive
//! +net_cost()
//! +net_premium_received()
//! +unrealized_pnl()
//! +pnl_at_expiration()
//! +validate()
//! }
//!
//! class ExpirationDate {
//! +Days(Positive)
//! +Date(NaiveDate)
//! +get_years()
//! +get_date()
//! +get_date_string()
//! +from_string()
//! }
//!
//! class Positive {
//! +value: Decimal
//! +ZERO: Positive
//! +ONE: Positive
//! +format_fixed_places()
//! +round_to_nice_number()
//! +is_positive()
//! }
//!
//! class OptionStyle {
//! <<enumeration>>
//! Call
//! Put
//! }
//!
//! class OptionType {
//! <<enumeration>>
//! European
//! American
//! }
//!
//! class Side {
//! <<enumeration>>
//! Long
//! Short
//! }
//!
//! class Graph {
//! <<interface>>
//! +graph_data()
//! +graph_config()
//! +to_plot()
//! +write_html()
//! +write_png()
//! +write_svg()
//! +write_jpeg()
//! }
//!
//! class Greeks {
//! <<interface>>
//! +delta()
//! +gamma()
//! +theta()
//! +vega()
//! +rho()
//! +calculate_all_greeks()
//! }
//!
//! Options --|> Greeks : implements
//! Options --|> Graph : implements
//! Position o-- Options : contains
//! Options *-- OptionStyle : has
//! Options *-- OptionType : has
//! Options *-- Side : has
//! Options *-- ExpirationDate : has
//! Options *-- Positive : uses
//! ```
//!
//! ## Strategies and Methods
//!
//! ```mermaid
//! classDiagram
//! class Strategy {
//! +name: String
//! +strategy_type: StrategyType
//! +description: String
//! +legs: Vec~Position~
//! +new()
//! +validate()
//! +add_leg()
//! }
//!
//! class Strategies {
//! <<interface>>
//! +get_volume()
//! +get_net_premium_received()
//! +get_max_profit()
//! +get_max_loss()
//! +get_profit_ratio()
//! +get_profit_area()
//! +get_break_even_points()
//! +get_fees()
//! +get_title()
//! +get_profit_loss_at_price()
//! +get_positions()
//! +get_options()
//! }
//!
//! class BullCallSpread {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +long_call: Position
//! +short_call: Position
//! +new()
//! +validate()
//! }
//!
//! class BearPutSpread {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +long_put: Position
//! +short_put: Position
//! +new()
//! +validate()
//! }
//!
//! class LongCall {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +long_call: Position
//! +new()
//! +validate()
//! }
//!
//! class LongPut {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +long_put: Position
//! +new()
//! +validate()
//! }
//!
//! class ShortCall {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +short_call: Position
//! +new()
//! +validate()
//! }
//!
//! class ShortPut {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +short_put: Position
//! +new()
//! +validate()
//! }
//!
//! class IronCondor {
//! +underlying_symbol: String
//! +underlying_price: Positive
//! +long_put: Position
//! +short_put: Position
//! +short_call: Position
//! +long_call: Position
//! +new()
//! +validate()
//! }
//!
//! class BasicAble {
//! <<interface>>
//! +get_underlying_symbol()
//! +get_underlying_price()
//! +one_option()
//! }
//!
//! class BreakEvenable {
//! <<interface>>
//! +get_break_even_points()
//! +get_profit_loss_zones()
//! }
//!
//! class Validable {
//! <<interface>>
//! +validate()
//! }
//!
//! class Optimizable {
//! <<interface>>
//! +get_best_ratio()
//! +get_best_profit()
//! +get_best_loss()
//! +get_best_premium()
//! +find_optimal()
//! }
//!
//! Strategy ..|> Strategies : implements
//! BullCallSpread ..|> Strategies : implements
//! BearPutSpread ..|> Strategies : implements
//! LongCall ..|> Strategies : implements
//! LongPut ..|> Strategies : implements
//! ShortCall ..|> Strategies : implements
//! ShortPut ..|> Strategies : implements
//! IronCondor ..|> Strategies : implements
//!
//! Strategies ..> BasicAble : requires
//! Strategies ..> BreakEvenable : requires
//! Strategies ..> Validable : requires
//! Optimizable ..> Validable : requires
//! Optimizable ..> Strategies : requires
//! ```
//!
//! ## Strategies Classification
//!
//! ```mermaid
//! classDiagram
//! class StrategyType {
//! <<enumeration>>
//! Custom
//! BullCallSpread
//! BearPutSpread
//! LongCall
//! LongPut
//! ShortCall
//! ShortPut
//! CallButterfly
//! LongButterflySpread
//! ShortButterflySpread
//! IronButterfly
//! IronCondor
//! Straddle
//! LongStraddle
//! ShortStraddle
//! Strangle
//! LongStrangle
//! ShortStrangle
//! CoveredCall
//! ProtectivePut
//! PoorMansCoveredCall
//! Collar
//! }
//!
//! class DirectionalBias {
//! <<enumeration>>
//! Bullish
//! Bearish
//! Neutral
//! }
//!
//! class VolatilityOutlook {
//! <<enumeration>>
//! High
//! Low
//! Neutral
//! }
//!
//! class ComplexityLevel {
//! <<enumeration>>
//! Basic
//! Intermediate
//! Advanced
//! }
//!
//! class RiskProfile {
//! <<enumeration>>
//! DefinedRisk
//! UndefinedRisk
//! LimitedProfit
//! UnlimitedProfit
//! }
//!
//! DirectionalBias <-- StrategyType : categorized by
//! VolatilityOutlook <-- StrategyType : categorized by
//! ComplexityLevel <-- StrategyType : categorized by
//! RiskProfile <-- StrategyType : categorized by
//!
//! class BullishStrategies {
//! BullCallSpread
//! LongCall
//! CoveredCall
//! PoorMansCoveredCall
//! }
//!
//! class BearishStrategies {
//! BearPutSpread
//! LongPut
//! ShortCall
//! }
//!
//! class NeutralStrategies {
//! IronCondor
//! IronButterfly
//! ButterflySpread
//! Straddle
//! Strangle
//! }
//!
//! class HighVolatilityStrategies {
//! LongStraddle
//! LongStrangle
//! LongCall
//! LongPut
//! }
//!
//! class LowVolatilityStrategies {
//! ShortStraddle
//! ShortStrangle
//! IronCondor
//! ButterflySpread
//! CoveredCall
//! }
//!
//! DirectionalBias <.. BullishStrategies : implements
//! DirectionalBias <.. BearishStrategies : implements
//! DirectionalBias <.. NeutralStrategies : implements
//! VolatilityOutlook <.. HighVolatilityStrategies : implements
//! VolatilityOutlook <.. LowVolatilityStrategies : implements
//! ```
//!
//! ## Setup Instructions
//!
//! ### Prerequisites
//!
//! - Rust 1.65 or higher
//! - Cargo
//!
//! ### Installation
//!
//! Add OptionStratLib to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! optionstratlib = "0.5.0"
//! ```
//!
//! Or use cargo to add it to your project:
//!
//! ```bash
//! cargo add optionstratlib
//! ```
//!
//! ### Optional Features
//!
//! The library includes several optional features that can be enabled:
//!
//! ```toml
//! [dependencies]
//! optionstratlib = { version = "0.5.0", features = ["plotly", "kaleido", "full"] }
//! ```
//!
//! - `plotly`: Enables visualization using plotly.rs
//! - `kaleido`: Enables saving static images (requires plotly)
//! - `full`: Enables all features
//!
//! ### Building from Source
//!
//! Clone the repository and build using Cargo:
//!
//! ```bash
//! git clone https://github.com/joaquinbejar/OptionStratLib.git
//! cd OptionStratLib
//! cargo build --release
//! ```
//!
//! Run tests:
//!
//! ```bash
//! cargo test --all-features
//! ```
//!
//! Generate documentation:
//!
//! ```bash
//! cargo doc --open
//! ```
//!
//! ## Library Usage
//!
//! ### Basic Usage
//!
//! ```rust
//! use optionstratlib::{Options, OptionStyle, OptionType, Side, ExpirationDate};
//! use optionstratlib::pos;
//! use rust_decimal_macros::dec;
//! use optionstratlib::greeks::Greeks;
//!
//! // Create a basic European call option
//! let option = Options::new(
//! OptionType::European,
//! Side::Long,
//! "AAPL".to_string(),
//! pos!(100.0),            // strike_price
//! ExpirationDate::Days(pos!(30.0)),
//! pos!(0.2),              // implied_volatility
//! pos!(1.0),              // quantity
//! pos!(105.0),            // underlying_price
//! dec!(0.05),             // risk_free_rate
//! OptionStyle::Call,
//! pos!(0.02),             // dividend_yield
//! None,                   // exotic_params
//! );
//!
//! // Calculate option price using Black-Scholes
//! let price = option.calculate_price_black_scholes().unwrap();
//! println!("Option price: {}", price);
//!
//! // Calculate Greeks
//! let delta = option.delta().unwrap();
//! let gamma = option.gamma().unwrap();
//! let theta = option.theta().unwrap();
//! println!("Delta: {}, Gamma: {}, Theta: {}", delta, gamma, theta);
//! ```
//!
//! ### Working with Strategies
//!
//! ```rust
//! use optionstratlib::{Positive, ExpirationDate, pos};
//! use optionstratlib::strategies::Strategies;
//! use optionstratlib::strategies::bull_call_spread::BullCallSpread;
//! use optionstratlib::visualization::Graph;
//! use rust_decimal_macros::dec;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     use optionstratlib::strategies::base::BreakEvenable;
//!     use optionstratlib::strategies::BasicAble;
//!     let underlying_price = pos!(5781.88);
//!
//!     // Create a Bull Call Spread strategy
//!     let strategy = BullCallSpread::new("SP500".to_string(), underlying_price, pos!(5750.0), pos!(5820.0), ExpirationDate::Days(pos!(2.0)), pos!(0.18), dec!(0.01), pos!(0.78), pos!(0.78), pos!(0.73), pos!(0.73), Default::default(), Default::default(), Default::default(), Default::default());
//!
//!     // Get information about the strategy
//!     println!("Title: {}", strategy.get_title());
//!     println!("Break Even Points: {:?}", strategy.get_break_even_points()?);
//!     println!("Net Premium Received: ${:.2}", strategy.get_net_premium_received()?);
//!     println!("Max Profit: ${:.2}", strategy.get_max_profit().unwrap_or(Positive::ZERO));
//!     println!("Max Loss: ${:0.2}", strategy.get_max_loss().unwrap_or(Positive::ZERO));
//!     println!("Total Fees: ${:.2}", strategy.get_fees()?);
//!     println!("Profit Area: {:.2}%", strategy.get_profit_area()?);
//!     println!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);
//!
//!     // Generate visualization and save to HTML file
//!     #[cfg(feature = "kaleido")]
//!     {
//!         let file_path = "Draws/Strategy/bull_call_spread_profit_loss_chart.html".as_ref();
//!         strategy.write_html(file_path)?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Visualization Example
//!
//! ```rust
//! use optionstratlib::visualization::{Graph, GraphData, Series2D, TraceMode, GraphConfig};
//! use optionstratlib::error::GraphError;
//! use std::path::PathBuf;
//! use rust_decimal_macros::dec;
//!
//! struct SimpleChart {
//!     series: Series2D
//! }
//!
//! impl Graph for SimpleChart {
//!     fn graph_data(&self) -> GraphData {
//!         GraphData::Series(self.series.clone())
//!     }
//!
//!     fn graph_config(&self) -> GraphConfig {
//!         use optionstratlib::visualization::{ColorScheme, LineStyle};
//! GraphConfig {
//!             title: "Interactive Chart Example".into(),
//!             width: 800,
//!             height: 600,
//!             x_label: Some("X Axis".into()),
//!             y_label: Some("Y Axis".into()),
//!             z_label: None,
//!             line_style: LineStyle::Solid,
//!             color_scheme: ColorScheme::Viridis,
//!             legend: Some(vec!["My Data".to_string()]),
//!             show_legend: true,
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), GraphError> {
//!     let series = Series2D {
//!         x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
//!         y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
//!         name: "Series 1".to_string(),
//!         mode: TraceMode::Lines,
//!         line_color: Some("#1f77b4".to_string()),
//!         line_width: Some(2.0),
//!     };
//!
//!     #[cfg(feature = "kaleido")]
//!     {
//!         let chart = SimpleChart { series };
//!         let filename: PathBuf = PathBuf::from("interactive_chart.html");
//!         chart.write_html(&filename)?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Testing
//!
//! To run the test suite:
//!
//! ```bash
//! make test
//! ```
//!
//! For running tests with specific features:
//!
//! ```bash
//! cargo test --features "plotly kaleido"
//! ```
//!
//! To generate a test coverage report:
//!
//! ```bash
//! make coverage
//! ```
//!

/// # OptionsStratLib: Financial Options Trading Library
///
/// A comprehensive library for options trading analytics, modeling, and strategy development.
/// Provides tools for pricing, risk assessment, strategy building, and performance analysis
/// of financial options across various market conditions.
///
/// ## Core Modules
extern crate core;

/// * `backtesting` - Tools for historical performance evaluation of options strategies.
///
/// Provides framework and utilities to simulate and analyze how option strategies
/// would have performed using historical market data. Supports various performance
/// metrics, drawdown analysis, and strategy comparison.
pub mod backtesting;

/// * `chains` - Functionality for working with options chains and series data.
///
/// Tools for parsing, manipulating, and analyzing options chain data. Includes
/// methods to filter chains by expiration, strike price, and other criteria,
/// as well as utilities for chain visualization and analysis.
pub mod chains;

/// * `constants` - Library-wide mathematical and financial constants.
///
/// Defines fundamental constants used throughout the library including mathematical
/// constants (π, epsilon values), market standards (trading days per year),
/// calculation parameters, and visualization color schemes.
pub mod constants;

/// * `curves` - Tools for yield curves, term structures, and other financial curves.
///
/// Implementations of various interest rate curves, forward curves, and term structures
/// used in options pricing and risk management. Includes interpolation methods and
/// curve fitting algorithms.
pub mod curves;

/// * `error` - Error types and handling functionality for the library.
///
/// Defines the error hierarchy used throughout the library, providing detailed
/// error types for different categories of failures including validation errors,
/// calculation errors, and input/output errors.
pub mod error;

/// * `geometrics` - Mathematical utilities for geometric calculations relevant to options.
///
/// Provides specialized geometric functions and algorithms for options pricing and modeling,
/// including path-dependent calculations and spatial transformations for volatility surfaces.
pub mod geometrics;

/// * `greeks` - Calculation and management of option sensitivity metrics (Delta, Gamma, etc.).
///
/// Comprehensive implementation of options Greeks (sensitivity measures) including
/// Delta, Gamma, Theta, Vega, and Rho. Includes analytical formulas, numerical
/// approximations, and visualization tools for risk analysis.
pub mod greeks;

/// * `model` - Core data structures and models for options and derivatives.
///
/// Defines the fundamental data types and structures used throughout the library,
/// including option contract representations, position tracking, and market data models.
/// Serves as the foundation for all other modules.
pub mod model;

/// * `pnl` - Profit and loss analysis tools for options positions.
///
/// Utilities for calculating, projecting, and visualizing profit and loss (P&L) profiles
/// for individual options and complex strategies. Includes time-based P&L evolution and
/// scenario analysis.
pub mod pnl;

/// * `pricing` - Option pricing models including Black-Scholes and numerical methods.
///
/// Implementations of various option pricing models including Black-Scholes-Merton,
/// binomial trees, Monte Carlo simulation, and finite difference methods. Supports
/// European, American, and exotic options.
pub mod pricing;

/// * `risk` - Risk assessment and management tools for options portfolios.
///
/// Tools for analyzing and quantifying risk in options positions and portfolios,
/// including Value at Risk (VaR), stress testing, scenario analysis, and
/// portfolio optimization algorithms.
pub mod risk;

/// * `simulation` - Simulation techniques for scenario analysis.
///
/// Framework for Monte Carlo and other simulation methods to model potential
/// market scenarios and their impact on options strategies. Includes path generation
/// algorithms and statistical analysis of simulation results.
pub mod simulation;

/// * `strategies` - Pre-defined option strategy templates and building blocks.
///
/// Library of common option strategies (spreads, straddles, condors, etc.) with
/// implementation helpers, parameter optimization, and analysis tools. Supports
/// strategy composition and customization.
pub mod strategies;

/// * `surfaces` - Volatility surface and other 3D financial data modeling.
///
/// Tools for constructing, manipulating, and analyzing volatility surfaces and
/// other three-dimensional financial data structures. Includes interpolation methods,
/// fitting algorithms, and visualization utilities.
pub mod surfaces;

/// * `utils` - General utility functions for data manipulation and calculations.
///
/// Collection of helper functions and utilities used across the library for
/// data manipulation, mathematical operations, date handling, and other
/// common tasks in financial calculations.
pub mod utils;

/// * `visualization` - Tools for plotting and visual representation of options data.
///
/// Graphics and visualization utilities for creating charts, graphs, and interactive
/// plots of options data, strategies, and analytics. Supports various plot types
/// optimized for different aspects of options analysis.
pub mod visualization;

/// * `volatility` - Volatility modeling, forecasting, and analysis utilities.
///
/// Comprehensive tools for volatility analysis including historical volatility calculation,
/// implied volatility determination, volatility forecasting models (GARCH, EWMA), and
/// volatility skew/smile analysis.
pub mod volatility;

/// * `series` - Functionality for working with collections of option chains across expirations.
///
/// Provides tools to manage, filter, and analyze multiple option chains grouped by expiration dates.
/// Includes utilities for constructing series data, navigating expirations, and performing
/// cross-expiration analysis and visualization.
pub mod series;

pub use model::ExpirationDate;
pub use model::Options;
pub use model::positive::Positive;
pub use model::types::{OptionStyle, OptionType, Side};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the library version
pub fn version() -> &'static str {
    VERSION
}
