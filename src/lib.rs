#![allow(unknown_lints)]
#![allow(clippy::literal_string_with_formatting_args)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code. Enforced crate-wide; individual modules that
// need a transitional escape hatch carry a scoped `#![allow(..)]` with a
// migration note (tracked as follow-ups to #341).
#![deny(clippy::indexing_slicing)]
// Unit and integration tests routinely index into `Vec`s they just pushed
// into, so the lint is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # OptionStratLib v0.22.0: Financial Options Library
//!
//! ## Table of Contents
//! 1. [Introduction](#introduction)
//! 2. [Features](#features)
//! 3. [Core Modules](#core-modules)
//! 4. [Workspace Crates](#workspace-crates)
//! 5. [Module Boundaries](#module-boundaries)
//! 6. [Trading Strategies](#trading-strategies)
//! 7. [Setup Instructions](#setup-instructions)
//! 8. [Library Usage](#library-usage)
//! 9. [Usage Examples](#usage-examples)
//! 10. [Testing](#testing)
//! 11. [Contribution and Contact](#contribution-and-contact)
//!
//! ## Introduction
//!
//! OptionStratLib is a comprehensive Rust library for options trading and strategy development across multiple asset classes.
//! This versatile toolkit enables traders, quants, and developers to model, analyze, and visualize options strategies with a
//! robust, type-safe approach. The library focuses on precision with decimal-based calculations, extensive test coverage,
//! and a modular architecture built on modern Rust 2024 edition.
//!
//! ## Features
//!
//! ### 1. **Pricing Models**
//! - **Black-Scholes Model**: European options pricing with full Greeks support
//! - **Binomial Tree Model**: American and European options with early exercise capability
//! - **Monte Carlo Simulations**: Complex pricing scenarios and path-dependent options
//! - **Telegraph Process Model**: Monte-Carlo pricing under two-state regime-switching volatility
//! - **American Options**: Barone-Adesi-Whaley approximation for early exercise
//! - **Exotic Options**: Complete support for 14 exotic option types (see below)
//!
//! ### 2. **Greeks Calculation**
//! - Complete Greeks suite: Delta, Gamma, Theta, Vega, Rho, Vanna, Vomma, Veta,
//!   Charm, Color
//! - Real-time sensitivity analysis
//! - Greeks visualization and risk profiling
//! - Custom Greeks implementations with adjustable parameters
//!
//! ### 3. **Volatility Models**
//! - Implied volatility calculation using Newton-Raphson method
//! - Volatility surface construction and interpolation
//! - Historical volatility estimation
//! - Advanced volatility modeling tools
//!
//! ### 4. **Option Chain Management**
//! - Complete option chain construction and analysis
//! - Strike price generation algorithms
//! - Chain data import/export (CSV/JSON formats)
//! - Advanced filtering and selection tools
//! - Option data grouping and organization
//!
//! ### 5. **Trading Strategies (25+ Strategies)**
//! - **Single Leg**: Long/Short Calls and Puts
//! - **Spreads**: Bull/Bear Call/Put Spreads
//! - **Butterflies**: Long/Short Butterfly Spreads (the long one is the textbook long call butterfly)
//! - **Ladders**: Bull Call Ladder
//! - **Complex**: Iron Condor, Iron Butterfly
//! - **Volatility**: Long/Short Straddles and Strangles
//! - **Income**: Covered Calls (with spot leg support), Poor Man's Covered Call
//! - **Protection**: Protective Puts, Collars
//! - **Custom**: Flexible custom strategy framework
//! - **Multi-Asset**: Strategies combining options with spot, futures, or perpetuals
//!
//! ### 6. **Risk Management & Analysis**
//! - Position tracking and management
//! - Break-even analysis with multiple break-even points
//! - Profit/Loss calculations at various price points
//! - Risk profiles and comprehensive visualizations
//! - Delta neutrality analysis and adjustment
//! - Probability analysis for strategy outcomes
//!
//! ### 7. **Backtesting Framework**
//! - Comprehensive backtesting engine
//! - Performance metrics calculation
//! - Strategy optimization tools
//! - Historical analysis capabilities
//!
//! ### 8. **Simulation Tools**
//! - Monte Carlo simulations for strategy testing
//! - Telegraph process implementation
//! - Random walk simulations
//! - Custom simulation frameworks
//! - Parametrized simulations with adjustable inputs
//!
//! ### 9. **Visualization & Plotting**
//! - Strategy payoff diagrams
//! - Greeks visualization
//! - 3D volatility surfaces
//! - Risk profiles and P&L charts
//! - Interactive charts (powered by `plotly.rs`)
//! - Binomial tree visualization
//! - Comprehensive plotting utilities
//!
//! ### 10. **Data Management**
//! - Efficient decimal-based calculations using `rust_decimal`
//! - CSV/JSON import/export functionality
//! - Time series data handling
//! - Price series management and manipulation
//! - Robust data validation and error handling
//!
//! ### 11. **Mathematical Tools**
//! - Curve interpolation techniques
//! - Surface construction and analysis
//! - Geometric operations for financial modeling
//! - Advanced mathematical utilities for options pricing
//!
//! ### 12. **Exotic Option Pricing**
//! Complete pricing support for all exotic option types:
//! - **Asian**: Arithmetic and geometric average price options
//! - **Barrier**: Up/Down, In/Out barrier options with rebates
//! - **Binary**: Cash-or-nothing and asset-or-nothing options
//! - **Lookback**: Fixed and floating strike lookback options
//! - **Compound**: Options on options
//! - **Chooser**: Options to choose call or put at future date
//! - **Cliquet**: Forward-starting options with local caps/floors
//! - **Rainbow**: Multi-asset best-of/worst-of options
//! - **Spread**: Kirk's approximation for price differentials
//! - **Quanto**: Currency-protected options
//! - **Exchange**: Margrabe's formula for asset exchange
//! - **Power**: Non-linear payoff options
//!
//! ## Quality & Discipline (0.16.x)
//!
//! The 0.16 line is a quality-hardening release. Every change below is
//! enforced crate-wide and documented in `CHANGELOG.md`:
//!
//! - **Checked `Decimal` arithmetic.** Every monetary-path kernel routes
//!   through `d_add` / `d_sub` / `d_mul` / `d_div` / `d_sum` /
//!   `d_sum_iter` in `model::decimal`. Overflow on any monetary
//!   expression surfaces `DecimalError::Overflow { operation, lhs, rhs }`
//!   tagged with a static call-site string; no silent wraparound.
//! - **Non-finite `f64` guards.** Every `f64 → Decimal` boundary inside
//!   pricing, Greeks, volatility, and simulation is wrapped with
//!   `finite_decimal(..)` and surfaces a domain-specific
//!   `NonFinite { context, value }` variant
//!   (`PricingError`, `GreeksError`, `VolatilityError`,
//!   `SimulationError`) instead of collapsing silently to
//!   `Decimal::ZERO`.
//! - **`NonZeroUsize` step counts.** `price_binomial`,
//!   `monte_carlo_option_pricing`, `telegraph` and related kernels take
//!   `std::num::NonZeroUsize` for `steps` / `simulations`; zero is
//!   structurally invalid at the type level. Use the `nz!(N)` macro
//!   at literal call sites.
//! - **`Positive` at every public boundary.** Monetary values,
//!   strikes, quantities, volatilities are `Positive` (newtype around
//!   `Decimal`). Strategy-level P&L goes through
//!   `Positive::new_decimal(..)` at every point where a signed
//!   `Decimal` would otherwise be clamped to `Positive`, so inverted
//!   strikes or out-of-range optimizer candidates return typed
//!   `StrategyError` rather than panicking.
//! - **Zero unchecked indexing in production code.**
//!   `#![deny(clippy::indexing_slicing)]` is enforced crate-wide with
//!   scoped, documented escapes per module. Tests stay permissive via
//!   `#![cfg_attr(test, allow(..))]`. Production paths use
//!   `.get(..).ok_or_else(..)` with typed errors.
//! - **Doc coverage floor.** `#![deny(missing_docs,
//!   rustdoc::broken_intra_doc_links)]`. Every `pub` item has a `///`
//!   summary; every `Result` returner documents its `# Errors`
//!   contract.
//! - **Structured tracing.** `#[tracing::instrument]` on the public
//!   hot paths: `pricing::black_scholes`,
//!   `pricing::monte_carlo_option_pricing`,
//!   `pricing::price_binomial`, `volatility::implied_volatility`,
//!   and the strategy optimizer entry points
//!   `get_best_ratio` / `get_best_area`. No `println!` / `eprintln!`
//!   / `dbg!` / `log::` anywhere in `src/`.
//! - **Compiler-attribute discipline.** `#[must_use]` on every pure
//!   function and builder, `#[inline]` on small hot-path helpers,
//!   `#[cold] #[inline(never)]` on every error constructor,
//!   `#[repr(u8)]` on small stable enums, canonical `#[derive]`
//!   ordering.
//! - **Deterministic simulation tests.** `utils::deterministic_rng(seed)`
//!   provides a canonical seeded `StdRng` for Monte-Carlo / simulation
//!   tests, so precision shifts in upstream arithmetic cannot flip
//!   assertions by luck.
//! - **Pricing-identity regression tests.** `crates/optionstratlib-pricing/tests/identities_test.rs`
//!   locks put-call parity on a grid, CRR binomial convergence to
//!   Black-Scholes, and the Greek sanity identities
//!   (`Γ_c = Γ_p`, `Vega_c = Vega_p`, `Δ_c − Δ_p ≈ e^{-qT}`).
//!
//! ### Arithmetic-Error Cascade
//!
//! ```mermaid
//! flowchart LR
//!     subgraph Kernels["Numeric kernels (model / pricing / greeks / volatility / simulation)"]
//!         DADD["d_add / d_sub / d_mul / d_div"]
//!         DSUM["d_sum / d_sum_iter"]
//!         FD["finite_decimal(f64)"]
//!     end
//!
//!     subgraph Errors["Typed errors (error/*)"]
//!         DOV["DecimalError::Overflow { operation, lhs, rhs }"]
//!         PNF["PricingError::NonFinite { context, value }"]
//!         GNF["GreeksError::NonFinite"]
//!         VNF["VolatilityError::NonFinite"]
//!         SNF["SimulationError::NonFinite"]
//!     end
//!
//!     DADD -- "checked_*" --> DOV
//!     DSUM -- "checked_*" --> DOV
//!     FD -- "NaN / ±∞ guard" --> PNF
//!     FD --> GNF
//!     FD --> VNF
//!     FD --> SNF
//!
//!     DOV -- "#[from]" --> PNF
//!     DOV -- "#[from]" --> GNF
//!     DOV -- "#[from]" --> VNF
//!     DOV -- "#[from]" --> SNF
//! ```
//!
//!
//! ## Core Modules
//!
//! The library is organized into the following key modules:
//!
//! ### **Model** (`model/`, defined by `optionstratlib-core`)
//! Core data structures and types for options trading:
//! - `option.rs`: the option contract (`Options`): terms, validation, payoff
//! - `position.rs`: positions with premium, fees and cost basis
//! - `expiration.rs`: flexible expiration date handling (Days/DateTime)
//! - `positive_ext.rs`: OptionStratLib extensions of `positive::Positive`
//! - `types.rs`: common enums (OptionType, Side, OptionStyle)
//! - `trade.rs`: trade records and their status
//! - `format.rs`: data formatting utilities
//! - **`leg/`**: Multi-instrument leg support for strategies
//!   - `traits.rs`: Common leg traits (`LegAble`, `Marginable`, `Fundable`, `Expirable`)
//!   - `spot.rs`: `SpotPosition` for underlying asset positions
//!   - `perpetual.rs`: `PerpetualPosition` for crypto perpetual swaps
//!   - `future.rs`: `FuturePosition` for exchange-traded futures
//!   - `leg_enum.rs`: `Leg` enum unifying all position types
//!
//! ### **Pricing Models** (`pricing/`)
//! Advanced pricing engines for options valuation:
//! - `black_scholes_model.rs`: European options pricing with Greeks
//! - `black_76.rs`: European options on futures/forwards (Black 1976)
//! - `garman_kohlhagen.rs`: European FX options (Garman-Kohlhagen 1983)
//! - `binomial_model.rs`: American/European options with early exercise
//! - `monte_carlo.rs`: Path-dependent and exotic options pricing
//! - `telegraph.rs`: Regime-switching volatility pricing
//! - `payoff.rs`: Payoff function implementations
//! - `american.rs`: Barone-Adesi-Whaley approximation
//! - **Exotic Options**:
//!   - `asian.rs`: Asian option pricing
//!   - `barrier.rs`: Barrier option pricing
//!   - `binary.rs`: Binary/Digital option pricing
//!   - `lookback.rs`: Lookback option pricing
//!   - `compound.rs`: Compound option pricing
//!   - `chooser.rs`: Chooser option pricing
//!   - `cliquet.rs`: Cliquet option pricing
//!   - `rainbow.rs`: Rainbow option pricing
//!   - `spread.rs`: Spread option pricing
//!   - `quanto.rs`: Quanto option pricing
//!   - `exchange.rs`: Exchange option pricing
//!   - `power.rs`: Power option pricing
//!
//! ### **Strategies** (`strategies/`)
//! Comprehensive trading strategy implementations:
//! - `base.rs`: Core traits (Strategable, BasicAble, Positionable, etc.)
//! - **Single Leg**: `long_call.rs`, `short_call.rs`, `long_put.rs`, `short_put.rs`
//! - **Spreads**: `bull_call_spread.rs`, `bear_call_spread.rs`, `bull_put_spread.rs`, `bear_put_spread.rs`
//! - **Ladders**: `bull_call_ladder.rs`
//! - **Butterflies**: `long_butterfly_spread.rs`, `short_butterfly_spread.rs`
//! - **Complex**: `iron_condor.rs`, `iron_butterfly.rs`
//! - **Volatility**: `long_straddle.rs`, `short_straddle.rs`, `long_strangle.rs`, `short_strangle.rs`
//! - **Income**: `covered_call.rs`, `poor_mans_covered_call.rs`
//! - **Protection**: `protective_put.rs`, `collar.rs`
//! - `custom.rs`: Flexible custom strategy framework
//! - `probabilities/`: Probability analysis for strategy outcomes
//! - `delta_neutral/`: Delta neutrality analysis and adjustment
//!
//! ### **Volatility** (`volatility/`)
//! Volatility modeling and analysis:
//! - `utils.rs`: Implied volatility calculation (Newton-Raphson method)
//! - `traits.rs`: Volatility model interfaces
//! - Advanced volatility surface construction
//!
//! ### **Greeks** (`greeks/`)
//! Complete Greeks calculation suite:
//! - Delta, Gamma, Theta, Vega, Rho, Vanna, Vomma, Veta, Charm, Color calculations
//! - Real-time sensitivity analysis
//! - Greeks-based risk management
//!
//! ### **Chains** (`chains/`)
//! Option chain management and analysis:
//! - `chain.rs`: Option chain construction and manipulation
//! - `utils.rs`: Chain analysis and filtering tools
//! - CSV/JSON import/export functionality
//! - Strike price generation algorithms
//!
//! ### **Backtesting** (`backtesting/`)
//! Strategy performance analysis:
//! - `metrics.rs`: Performance metrics calculation
//! - `results.rs`: Backtesting results management
//! - `types.rs`: Backtesting data structures
//!
//! ### **Simulation** (`simulation/`)
//! Monte Carlo and stochastic simulations:
//! - Random walk implementations
//! - Telegraph process modeling
//! - Custom simulation frameworks
//! - Parametrized simulation tools
//!
//! ### **Visualization** (`visualization/`)
//! Comprehensive plotting and charting (`optionstratlib-visualization`):
//! - `Graph`: one chart contract on every feature surface, with Plotly
//!   rendering behind `plotly` and PNG/SVG export behind `static_export`
//! - Strategy payoff diagrams
//! - Greeks visualization
//! - 3D volatility surfaces
//! - Risk profile charts
//!
//! ### **Metrics** (`metrics/`)
//! Performance, risk, and liquidity metrics analysis:
//! - **Price Metrics**: Volatility skew curves
//! - **Risk Metrics**:
//!   - Implied Volatility curves (by strike) and surfaces (strike vs time)
//!   - Risk Reversal curves (by strike)
//!   - Dollar Gamma curves (by strike)
//! - **Composite Metrics**:
//!   - Vanna-Volga Hedge surfaces (price vs volatility)
//!   - Delta-Gamma Profile curves (by strike) and surfaces (price vs time)
//!   - Smile Dynamics curves (by strike) and surfaces (strike vs time)
//! - **Liquidity Metrics**:
//!   - Bid-Ask Spread curves (by strike)
//!   - Volume Profile curves (by strike) and surfaces (strike vs time)
//!   - Open Interest Distribution curves (by strike)
//! - **Stress Metrics**:
//!   - Volatility Sensitivity curves (by strike) and surfaces (price vs volatility)
//!   - Time Decay Profile curves (by strike) and surfaces (price vs time)
//!   - Price Shock Impact curves (by strike) and surfaces (price vs volatility)
//! - **Temporal Metrics**:
//!   - Theta curves (by strike) and surfaces (price vs time)
//!   - Charm (Delta Decay) curves (by strike) and surfaces (price vs time)
//!   - Color (Gamma Decay) curves (by strike) and surfaces (price vs time)
//!
//! ### **Risk Management** (`risk/`)
//! Risk analysis and management tools:
//! - Position risk metrics
//! - Break-even analysis
//! - Risk profile generation
//!
//! ### **P&L** (`pnl/`)
//! Profit and loss calculation:
//! - Real-time P&L tracking
//! - Historical P&L analysis
//! - Performance attribution
//!
//! ### **Curves & Surfaces** (`curves/`, `surfaces/`)
//! Mathematical tools for financial modeling:
//! - Curve interpolation techniques
//! - Surface construction and analysis
//! - 3D visualization capabilities
//!
//! ### **Error Handling** (`error/`)
//! Robust error management:
//! - Comprehensive error types for each module
//! - Type-safe error propagation
//! - Detailed error reporting
//!
//!
//! ## Workspace Crates
//!
//! The library is being split into focused crates. Each one can be used on
//! its own, and this crate (the `optionstratlib` facade) re-exports them, so
//! the paths below are the same types whichever crate you import them from.
//!
//! | Crate | Facade feature | Facade paths | Contents |
//! | --- | --- | --- | --- |
//! | `optionstratlib-core` | always | `model`, `utils`, `constants`; the core errors in `error` (`DecimalError`, `DecimalResult`, `OptionsError`, `OptionsResult`, `PositionError`, `TradeError`, `OperationErrorKind` and the `error::position` module); `ExpirationDate`, `Options`, `OptionStyle`, `OptionType`, `RainbowType`, `Side` at the root; the `nz!`, `f2d!`, `f2du!`, `d2f!`, `d2fu!` and `assert_decimal_eq!` macros; `Positive`, `pos_or_panic!`, `spos!` and `assert_pos_relative_eq!` in `prelude` | domain model, foundational re-exports, checked `Decimal` helpers |
//! | `optionstratlib-math` | `math` | `curves`, `surfaces`, `geometrics`; the math errors in `error` (`CurveError`, `CurvesResult`, `SurfaceError`, `InterpolationError`, `MetricsError`) | generic curves, surfaces, interpolation |
//! | `optionstratlib-pricing` | `pricing` (implies `math`) | `pricing`, `greeks`, `volatility`; the pricing errors in `error` (`PricingError`, `PricingResult`, `GreeksError`, `GreeksResult`, `VolatilityError` and the `error::greeks` module) | pricing models, Greeks, implied and historical volatility |
//! | `optionstratlib-simulation` | `simulation` (implies `pricing`) | `simulation`; `SimulationError` and `SimulationResult` in `error` | random walks, stochastic processes, simulators, exit policies, generic path evaluation and statistics |
//! | `optionstratlib-market` | `market` (implies `pricing`) | `chains`, `series`; the market errors in `error` (`ChainError`, `OhlcvError` and the `error::chains` module) | option chains, option series, OHLCV candles; file I/O behind `io` |
//! | `optionstratlib-analytics` | `analytics` (implies `market`) | `analytics`, `pnl`, `risk`, `metrics`; the analytics errors in `error` (`ProbabilityError`, `ProbabilityResult`, `ProjectionError`, `TransactionError` and the `error::probability` module) | P&L, SPAN margin, price-probability kernels, risk-neutral densities, option-chain metrics and projections |
//! | `optionstratlib-strategies` | `strategies` (implies `analytics`) | `strategies`; `StrategyError`, `StrategyResult` and the `error::strategies` module in `error` | spreads, butterflies, condors, straddles, strangles, custom strategies, delta neutrality, strategy probability analysis |
//! | `optionstratlib-backtest` | `backtest` (implies `strategies` and `simulation`) | `backtesting`; `BacktestError` in `error` | strategy backtests over simulated paths: per-path evaluation, run statistics, reports and metrics |
//! | `optionstratlib-visualization` | `visualization` (implies `backtest`); `plotly` and `static_export` forward to its backend features | `visualization`; `GraphError` and the aggregate `Error` in `error`; the `impl_graph_for_payoff_strategy!` macro at the root | chart data, the `Graph` contract and its implementations, Plotly rendering and PNG/SVG export |
//!
//! Each facade path is an explicit module or item re-export (`pub use
//! optionstratlib_core::model;`, `pub use
//! optionstratlib_core::error::DecimalError`), never a glob over a
//! component's root, so
//! `optionstratlib::model::Options` *is* `optionstratlib_core::model::Options`.
//! Depend on a component directly when you need only that layer; its own
//! docs list its entry points, or enable only its facade feature:
//!
//! ```toml
//! [dependencies]
//! # pricing, greeks and volatility, without market data, I/O or charts
//! optionstratlib = { version = "0.22.0", default-features = false, features = ["pricing"] }
//! ```
//!
//! ## Canonical paths
//!
//! Every public item has one defining owner, a component crate, and one
//! canonical path: the shortest flat path its module exports. A type defined
//! in a submodule and re-exported by its parent is canonical at the parent,
//! `optionstratlib::pricing::black_scholes` rather than
//! `optionstratlib::pricing::black_scholes_model::black_scholes`, and
//! `optionstratlib::error::PricingError` for every error type. The defining
//! submodules stay public as documentation anchors (each carries the module
//! docs for its model or strategy), so both paths name the same item, but
//! examples, docs and new code use the flat one. The error kind modules
//! (`error::position`, `error::greeks`, `error::chains`,
//! `error::probability`, `error::strategies`) are canonical for their
//! `...Kind` enums, which are not flattened because their names collide
//! across crates. Paths kept only for 0.21 have been removed (#550).
//!
//! The [public concept ownership map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
//! (`docs/ownership.md`) lists, for every concept group, error and root
//! item, its defining crate and module, the direct import, the facade path
//! and the feature it needs. `make check-graph` checks it against the
//! public API, the identity tests and the manifests, so it cannot go stale.
//!
//! Runnable programs that depend on the component crates directly, one per
//! capability and with the smallest dependency set, are in
//! [`examples/direct`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct),
//! next to the facade equivalent of each.
//!
//! Every capability now lives in a component crate. The facade itself owns
//! only the `prelude` and the unified `error::Error`, which wraps every
//! component error including `GraphError` and therefore needs
//! `visualization`; the facade default enables all of it.
//!
//! ## Module Boundaries
//!
//! The modules form a directed acyclic graph of layers. Every module belongs
//! to exactly one layer and may reference only its own layer and the ones
//! below it, except that strategies and backtesting skip math, which they
//! reach only through pricing and analytics (ADR-0001 D9). This is the graph the 0.22 workspace split follows, so the
//! layers, not the module names, are what a future crate boundary cuts along
//! (ADR-0001 D9).
//!
//! ```mermaid
//! flowchart BT
//!     CORE["core — model, constants, utils"]
//!     MATH["math — geometrics, curves, surfaces"]
//!     PRICING["pricing — pricing, greeks, volatility"]
//!     SIM["simulation"]
//!     MARKET["market — chains, series"]
//!     ANALYTICS["analytics — analytics, pnl, risk, metrics"]
//!     STRAT["strategies"]
//!     BACKTEST["backtesting"]
//!     VIS["visualization"]
//!
//!     MATH --> CORE
//!     PRICING --> MATH
//!     SIM --> PRICING
//!     MARKET --> PRICING
//!     MARKET -. "synthetic (optional)" .-> SIM
//!     ANALYTICS --> MARKET
//!     STRAT --> ANALYTICS
//!     BACKTEST --> STRAT
//!     BACKTEST --> SIM
//!     VIS --> BACKTEST
//! ```
//!
//! Two rules make the graph checkable rather than aspirational:
//!
//! - **`src/error` is partitioned, not a layer of its own.** Each file under
//!   `src/error` belongs to the layer that raises it, so a lower layer
//!   returning a higher layer's error is a forbidden edge even though both
//!   types live in the same directory. `src/utils` is partitioned the same
//!   way, per file.
//! - **The one optional edge is `market -> simulation`, gated by
//!   `synthetic`.** Building with `default-features = false, features =
//!   ["market"]` leaves a market surface that names no simulation type,
//!   including through `ChainError`.
//!
//! ### Enforcement
//!
//! ```sh
//! make check-graph           # layer DAG, error/utils partition, synthetic gate, ownership map
//! make check-feature-trees   # the dependency graph of each market surface
//! ```
//!
//! Both run in CI on every pull request. `check-graph` resolves each
//! `crate::error::Name` reference to the file that defines it, follows
//! re-exports and aliases, and ignores `#[cfg(test)]` items, so a violation
//! cannot hide behind a bare import path or a test module. It also runs a
//! self-test proving the scanner catches what it must, and checks the
//! ownership map (`docs/ownership.md`) against the public-API snapshots,
//! `tests/unit/canonical_paths_test.rs` and the manifests.
//!
//! Known reverse edges whose removal is a breaking change wait in the
//! script's `DEFERRED` table, scoped to the exact files that carry them and
//! each naming the issue that removes it; the same module pair in any other
//! file is a fresh violation, and an entry whose edge has disappeared is
//! reported so the list gets pruned. `python3 scripts/check_module_boundaries.py
//! --inventory` prints the current table, empty since #658.
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
//! +vanna()
//! +vomma()
//! +veta()
//! +charm()
//! +color()
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
//! class Leg {
//! <<enumeration>>
//! Option(Position)
//! Spot(SpotPosition)
//! Future(FuturePosition)
//! Perpetual(PerpetualPosition)
//! +is_option()
//! +is_spot()
//! +is_linear()
//! +delta()
//! +pnl_at_price()
//! }
//!
//! class SpotPosition {
//! +symbol: String
//! +quantity: Positive
//! +cost_basis: Positive
//! +side: Side
//! +date: DateTime<Utc>
//! +open_fee: Positive
//! +close_fee: Positive
//! +pnl_at_price()
//! +delta()
//! +market_value()
//! +break_even_price()
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
//! Bermuda
//! Asian
//! Barrier
//! Binary
//! Lookback
//! Compound
//! Chooser
//! Cliquet
//! Rainbow
//! Spread
//! Quanto
//! Exchange
//! Power
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
//! Leg o-- Position : Option variant
//! Leg o-- SpotPosition : Spot variant
//! SpotPosition *-- Side : has
//! SpotPosition *-- Positive : uses
//! Options *-- OptionStyle : has
//! Options *-- OptionType : has
//! Options *-- Side : has
//! Options *-- ExpirationDate : has
//! Options *-- Positive : uses
//! ```
//!
//! ## Pricing Models Architecture
//!
//! ```mermaid
//! flowchart TB
//!     subgraph Standard["Standard Options"]
//!         EU[European]
//!         AM[American]
//!         BE[Bermuda]
//!     end
//!
//!     subgraph PathDependent["Path-Dependent"]
//!         AS[Asian]
//!         LB[Lookback]
//!         BA[Barrier]
//!         CL[Cliquet]
//!     end
//!
//!     subgraph MultiAsset["Multi-Asset"]
//!         RB[Rainbow]
//!         SP[Spread]
//!         EX[Exchange]
//!     end
//!
//!     subgraph Special["Special Payoffs"]
//!         BI[Binary]
//!         PW[Power]
//!         QU[Quanto]
//!         CO[Compound]
//!         CH[Chooser]
//!     end
//!
//!     subgraph Forward["Forward-Priced"]
//!         FUT[Future]
//!         FWD[Forward]
//!     end
//!
//!     subgraph FX["FX / Currency"]
//!         FX_S[FX Spot]
//!     end
//!
//!     BS[black_scholes] --> EU
//!     BS --> PathDependent
//!     BS --> MultiAsset
//!     BS --> Special
//!     B76[black_76] --> Forward
//!     GK[garman_kohlhagen] --> FX
//!     BAW[barone_adesi_whaley] --> AM
//!     BIN[binomial_model] --> AM
//!     BIN --> BE
//!     MC[monte_carlo] --> PathDependent
//! ```
//!
//! ## Strategy Traits System
//!
//! ```mermaid
//! classDiagram
//!     class Strategable {
//!         <<trait>>
//!         Master trait combining all capabilities
//!     }
//!
//!     class BasicAble {
//!         <<trait>>
//!         +get_underlying_price()
//!         +get_underlying_symbol()
//!         +get_expiration()
//!         +get_title()
//!     }
//!
//!     class Positionable {
//!         <<trait>>
//!         +get_positions()
//!         +add_position()
//!         +modify_position()
//!     }
//!
//!     class Strategies {
//!         <<trait>>
//!         +get_net_premium_received()
//!         +get_max_profit()
//!         +get_max_loss()
//!         +get_total_cost()
//!     }
//!
//!     class BreakEvenable {
//!         <<trait>>
//!         +get_break_even_points()
//!         +calculate_break_even()
//!     }
//!
//!     class Profit {
//!         <<trait>>
//!         +get_point_at_price()
//!         +calculate_profit_at()
//!     }
//!
//!     class Greeks {
//!         <<trait>>
//!         +delta()
//!         +gamma()
//!         +theta()
//!         +vega()
//!     }
//!
//!     class DeltaNeutrality {
//!         <<trait>>
//!         +get_delta()
//!         +suggest_delta_adjustments()
//!     }
//!
//!     Strategable --|> Strategies
//!     Strategable --|> StrategyConstructor
//!     Strategable --|> Profit
//!     Strategable --|> ProbabilityAnalysis
//!     Strategable --|> Greeks
//!     Strategable --|> DeltaNeutrality
//!     Strategable --|> PnLCalculator
//!     Strategies --|> Validable
//!     Strategies --|> Positionable
//!     Strategies --|> BreakEvenable
//!     Strategies --|> BasicAble
//! ```
//!
//! ## Metrics Framework
//!
//! ```mermaid
//! flowchart LR
//!     subgraph OptionChain
//!         OC[OptionChain]
//!     end
//!
//!     subgraph Curves["Curve Metrics"]
//!         IV_C[IV Curve]
//!         RR_C[Risk Reversal]
//!         DG_C[Dollar Gamma]
//!         TH_C[Theta Curve]
//!         VA_C[Vanna Curve]
//!         SK_C[Skew Curve]
//!     end
//!
//!     subgraph Surfaces["Surface Metrics"]
//!         IV_S[IV Surface]
//!         TH_S[Theta Surface]
//!         CH_S[Charm Surface]
//!         VS_S[Vol Sensitivity]
//!         TD_S[Time Decay]
//!     end
//!
//!     OC --> Curves
//!     OC --> Surfaces
//!     Curves --> |"2D Analysis"| Analysis[Risk Analysis]
//!     Surfaces --> |"3D Analysis"| Analysis
//! ```
//!
//! ## Observability
//!
//! Public hot paths are annotated with `#[tracing::instrument]`.
//! Enable a subscriber in the consumer crate (the library itself never
//! installs one) to surface structured spans:
//!
//! ```mermaid
//! flowchart LR
//!     APP[Consumer application] -- "installs" --> SUB["tracing_subscriber"]
//!
//!     subgraph Spans["Instrumented public fns"]
//!         BS["pricing::black_scholes\n(strike, style, side)"]
//!         MC["pricing::monte_carlo_option_pricing\n(steps, simulations, strike, style, side)"]
//!         BI["pricing::price_binomial\n(strike, asset, steps, style, side)"]
//!         IV["volatility::implied_volatility\n(market_price, strike, max_iterations)"]
//!         OPT["Optimizable::get_best_ratio/area\n(side, criteria)"]
//!     end
//!
//!     BS --> SUB
//!     MC --> SUB
//!     BI --> SUB
//!     IV --> SUB
//!     OPT --> SUB
//! ```
//!
//! ## Trading Strategies
//!
//! OptionStratLib provides 25+ comprehensive trading strategies organized by complexity and market outlook:
//!
//! ### **Single Leg Strategies**
//! Basic directional strategies for beginners:
//! - **Long Call**: Bullish strategy with unlimited upside potential
//! - **Short Call**: Bearish strategy collecting premium with limited profit
//! - **Long Put**: Bearish strategy with high profit potential
//! - **Short Put**: Bullish strategy collecting premium with assignment risk
//!
//! ### **Spread Strategies**
//! Defined risk strategies with limited profit/loss:
//! - **Bull Call Spread**: Moderately bullish with limited risk and reward
//! - **Bear Call Spread**: Moderately bearish credit spread
//! - **Bull Put Spread**: Moderately bullish credit spread
//! - **Bear Put Spread**: Moderately bearish debit spread
//! - **Bull Call Ladder**: A bull call spread with a second, higher short call; unlimited loss above the upper break-even
//!
//! ### **Butterfly Strategies**
//! Market neutral strategies profiting from low volatility:
//! - **Long Butterfly Spread**: Profits from price staying near middle strike
//! - **Short Butterfly Spread**: Profits from price moving away from middle strike
//!
//! ### **Complex Multi-Leg Strategies**
//! Advanced strategies for experienced traders:
//! - **Iron Condor**: Market neutral strategy with wide profit zone
//! - **Iron Butterfly**: Market neutral strategy with narrow profit zone
//!
//! ### **Volatility Strategies**
//! Strategies that profit from volatility changes:
//! - **Long Straddle**: Profits from high volatility in either direction
//! - **Short Straddle**: Profits from low volatility (range-bound market)
//! - **Long Strangle**: Similar to straddle but with different strikes
//! - **Short Strangle**: Credit strategy profiting from low volatility
//!
//! ### **Income Generation Strategies**
//! Strategies focused on generating regular income:
//! - **Covered Call**: Stock/spot ownership with call selling for income (now with full spot leg support)
//! - **Poor Man's Covered Call**: LEAPS-based covered call alternative
//!
//! ### **Protection Strategies**
//! Risk management and hedging strategies:
//! - **Protective Put**: Downside protection for stock positions
//! - **Collar**: Combination of covered call and protective put
//!
//! ### **Custom Strategy Framework**
//! - **Custom Strategy**: Flexible framework for creating any multi-leg strategy
//! - Supports unlimited number of legs
//! - Full integration with all analysis tools
//! - Complete trait implementation for consistency
//!
//! ### **Strategy Analysis Features**
//! All strategies include comprehensive analysis capabilities:
//! - **Profit/Loss Analysis**: P&L at any price point and time
//! - **Break-Even Points**: Multiple break-even calculations
//! - **Greeks Analysis**: Real-time risk metrics
//! - **Probability Analysis**: Success probability calculations
//! - **Delta Neutrality**: Delta-neutral position analysis
//! - **Visualization**: Interactive payoff diagrams and risk profiles
//! - **Optimization**: Find optimal strikes and expirations
//!
//! ### **Strategy Traits System**
//! All strategies implement a comprehensive trait system:
//!
//! - **Strategable**: Master trait combining all strategy capabilities
//! - **BasicAble**: Basic strategy information (symbol, price, etc.)
//! - **Positionable**: Position management and modification
//! - **Strategies**: Core strategy calculations (P&L, break-even, etc.)
//! - **Validable**: Strategy validation and error checking
//! - **BreakEvenable**: Break-even point calculations
//! - **Profit**: Profit/loss analysis at various price points
//! - **Greeks**: Greeks calculations for risk management
//! - **DeltaNeutrality**: Delta-neutral analysis and adjustments
//! - **ProbabilityAnalysis**: Outcome probability calculations
//! - **Graph**: Visualization and plotting capabilities; a visualization trait
//!   every concrete strategy implements, not a supertrait of `Strategable`
//!
//! ## Setup Instructions
//!
//! ### Prerequisites
//!
//! - Rust 1.85 or higher (Rust 2024 edition)
//! - Cargo package manager
//!
//! ### Installation
//!
//! Add OptionStratLib to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! optionstratlib = "0.22.0"
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
//! The library includes optional features for enhanced functionality:
//!
//! ```toml
//! [dependencies]
//! optionstratlib = { version = "0.22.0", features = ["plotly"] }
//! ```
//!
//! - `math`, `pricing`, `market`, `analytics`, `strategies` (default): the
//!   component crates of the same name and their facade paths (see
//!   [Workspace Crates](#workspace-crates)). Each implies the one below it,
//!   `pricing` alone resolves no market, I/O, async or visualization package,
//!   `analytics` alone resolves no strategy, simulation or plotting code, and
//!   `strategies` alone resolves no simulation, backtesting or plotting code
//! - `simulation` (default): the `optionstratlib-simulation` crate (random
//!   walks, stochastic processes, simulators, exit policies, path statistics)
//!   and its facade paths; implies `pricing` but not `market`, and resolves no
//!   option chain, strategy, backtesting or plotting code
//! - `backtest` (default): the `optionstratlib-backtest` crate (strategies
//!   evaluated over simulated paths, run statistics, reports and metrics) and
//!   its facade paths; implies `strategies` and `simulation`, and resolves no
//!   plotting code. Its progress is reported as `tracing` events
//! - `visualization` (default): the `optionstratlib-visualization` crate
//!   (chart data, the `Graph` contract and its implementations for options,
//!   positions, curves, surfaces, simulations and strategies), `GraphError`
//!   and the unified `error::Error`; implies `backtest` and resolves no
//!   Plotly package
//! - `plotly`: Enables interactive visualization using plotly.rs (implies
//!   `visualization`; forwards `optionstratlib-visualization/plotly`, which
//!   resolves no image-export or async package)
//! - `static_export`: PNG / SVG export via `plotly_static` (implies `plotly`
//!   and `async`; forwards `optionstratlib-visualization/static_export`)
//! - `io` (default): CSV, JSON and ZIP file I/O for chains and OHLCV candles
//!   (`OptionChain::save_to_csv` and friends, `read_ohlcv_from_zip`, `OhlcvError`);
//!   `default-features = false` drops it, and `csv` and `zip` with it
//! - `async`: asynchronous versions of that I/O (implies `market` and `io`; adds tokio
//!   through the market crate)
//! - `schema` (default): `utoipa::ToSchema` derives on the domain types of every
//!   enabled component (forwards `optionstratlib-core/schema` and, weakly,
//!   `schema` of each other component, so it never adds a component). Additive: it
//!   changes no type. Without it no build resolves `utoipa`
//! - `parallel`: reserved and empty in 0.22, so the name is not reused for another
//!   meaning. `rayon` is mandatory in the numeric crates; a sequential build is not offered
//! - `synthetic` (default): simulation-backed `OptionChain` and `OptionSeries` generators
//!   (`chains::generator_optionchain`, `series::generator_optionseries`, defined by
//!   `optionstratlib-market` behind its own `synthetic` feature), whose simulation
//!   failures arrive as `ChainError::Generator`; implies `market` and `simulation`.
//!   Leave it out (`default-features = false, features = ["market"]`) for a
//!   market surface that names no simulation type at all.
//!   `make check-graph` proves the gate holds and `make check-feature-trees` pins both
//!   dependency graphs
//!
//! #### Feature routing
//!
//! Every facade feature, with exactly what it switches on (ADR-0002 section
//! 2). `dep:` names the optional component dependency, `crate/feature` a
//! feature forwarded to a component, and "implies" another facade feature.
//! Features only add: none replaces a type, and a type has the same defining
//! crate whichever features are on.
//!
//! | Feature | Enables | Forwards | Implies | Default |
//! | --- | --- | --- | --- | --- |
//! | `math` | `dep:optionstratlib-math` | | | via `pricing` |
//! | `pricing` | `dep:optionstratlib-pricing` | | `math` | yes |
//! | `market` | `dep:optionstratlib-market` | | `pricing` | yes |
//! | `analytics` | `dep:optionstratlib-analytics` | | `market` | yes |
//! | `strategies` | `dep:optionstratlib-strategies` | | `analytics` | yes |
//! | `simulation` | `dep:optionstratlib-simulation` | | `pricing` | yes |
//! | `backtest` | `dep:optionstratlib-backtest` | | `strategies`, `simulation` | yes |
//! | `visualization` | `dep:optionstratlib-visualization` | | `backtest` | yes |
//! | `plotly` | | `optionstratlib-visualization/plotly` | `visualization` | no |
//! | `static_export` | | `optionstratlib-visualization/static_export` | `plotly`, `async` | no |
//! | `io` | | `optionstratlib-market/io` | `market` | yes |
//! | `async` | | `optionstratlib-market/async` | `market`, `io` | no |
//! | `synthetic` | | `optionstratlib-market/synthetic` | `market`, `simulation` | yes |
//! | `schema` | | `optionstratlib-core/schema`, `optionstratlib-{math,pricing,simulation,market,analytics,strategies,backtest}?/schema` | | yes |
//! | `parallel` | | | | no (reserved, empty) |
//!
//! The facade depends on `optionstratlib-core` always and declares no other
//! optional dependency: `async` adds `tokio` through the market crate, and
//! `plotly` and `static_export` add their packages through the visualization
//! crate. `make check-graph` fails when this routing drifts, when a lower
//! capability enables `visualization`, `plotly` or `static_export`, or when
//! any package other than the visualization crate declares Plotly, and
//! `make check-feature-trees` pins the resolved graph of the facade with no
//! capability, of each capability alone and of the default.
//!
//! **The 0.22 default** is every capability that 0.21 shipped without a
//! feature flag: `pricing`, `market`, `analytics`, `strategies`,
//! `simulation`, `backtest`, `visualization`, `synthetic`, `io` and `schema`, so a
//! plain `optionstratlib = "0.22.0"` keeps the whole library and resolves no
//! Plotly, image-export, async-runtime or HTTP package. The default is a
//! compatibility contract of the 0.22 line, not a statement that a consumer
//! needs all of it: opt out with `default-features = false` and name only the
//! capability you use, or depend on a component crate directly.
//!
//! **Combinations that do not exist.** The implications are the whole
//! dependency story, so some builds cannot be asked for:
//!
//! - There is no facade without `optionstratlib-core`.
//! - There is no chart-only build: `visualization` implies `backtest`, because
//!   the visualization crate renders strategies and simulations, so it brings
//!   every capability below it.
//! - There is no `plotly` or `static_export` without `visualization`, and no
//!   `static_export` without `async` (and so `io`, `csv`, `zip` and `tokio`);
//!   the implication is the 0.21 behaviour, kept for 0.22.
//! - There is no `synthetic` without `simulation`, no `async` without `io`,
//!   and no capability above `pricing` without `pricing` and `math`.
//! - `parallel` enables nothing: the parallel code paths use `rayon`
//!   unconditionally, so a sequential build is not offered.
//!
//! The matrix CI runs: no features, default, all features, each capability
//! alone (`math`, `schema`, `pricing`, `market`, `io`, `async`, `synthetic`,
//! `analytics`, `strategies`, `simulation`, `backtest`, `visualization`) and
//! the pairs `market,simulation`, `analytics,simulation`,
//! `strategies,simulation` and `market,synthetic`, plus `plotly` and
//! `static_export` as their own surfaces (`make lint`,
//! `make check-visualization`).
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
//! Run comprehensive test suite:
//!
//! ```bash
//! cargo test --all-features
//! ```
//!
//! Generate documentation:
//!
//! ```bash
//! cargo doc --open --all-features
//! ```
//!
//! Run benchmarks:
//!
//! ```bash
//! cargo bench
//! ```
//!
//! ## Library Usage
//!
//! ### Basic Option Creation and Pricing
//!
//! ```rust
//! # #[cfg(feature = "pricing")]
//! # mod example {
//! use optionstratlib::{Options, OptionStyle, OptionType, Side, ExpirationDate};
//! use positive::{pos_or_panic,Positive};
//! use rust_decimal_macros::dec;
//! use optionstratlib::greeks::Greeks;
//! use optionstratlib::pricing::OptionPricing;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a European call option
//!     let option = Options::new(
//!         OptionType::European,
//!         Side::Long,
//!         "AAPL".to_string(),
//!         pos_or_panic!(150.0),            // strike_price
//!         ExpirationDate::Days(pos_or_panic!(30.0)),
//!         pos_or_panic!(0.25),             // implied_volatility
//!         Positive::ONE,              // quantity
//!         pos_or_panic!(155.0),            // underlying_price
//!         dec!(0.05),             // risk_free_rate
//!         OptionStyle::Call,
//!         pos_or_panic!(0.02),             // dividend_yield
//!         None,                   // exotic_params
//!     );
//!
//!     // Calculate option price using Black-Scholes
//!     let price = option.calculate_price_black_scholes()?;
//!     tracing::info!("Option price: ${:.2}", price);
//!
//!     // Calculate Greeks for risk management. Every greek is the sensitivity
//!     // of the *position*: signed by its `Side` and scaled by its `quantity`,
//!     // so a short reports the negative of the equivalent long. `alpha` is the
//!     // one exception, being the ratio `gamma / theta`, which a short negates
//!     // in both terms.
//!     let delta = option.delta()?;
//!     let gamma = option.gamma()?;
//!     let theta = option.theta()?;
//!     let vega = option.vega()?;
//!     let vanna = option.vanna()?;
//!     let vomma = option.vomma()?;
//!     let veta = option.veta()?;
//!     let charm = option.charm()?;
//!     let color = option.color()?;
//!     tracing::info!("Greeks - Delta: {:.4}, Gamma: {:.4}, Theta: {:.4},
//!         Vega: {:.4}, Vanna: {:.4}, Vomma: {:.4}, Veta: {:.4}
//!         Charm: {:.4}, Color: {:.4}",
//!         delta, gamma, theta, vega, vanna, vomma, veta, charm, color);
//!     Ok(())
//! }
//! # pub fn run() -> Result<(), Box<dyn std::error::Error>> { main() }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     #[cfg(feature = "pricing")]
//! #     example::run()?;
//! #     Ok(())
//! # }
//! ```
//!
//! ### Working with Trading Strategies
//!
//! ```rust
//! # #[cfg(feature = "visualization")]
//! # mod example {
//! use positive::{Positive, pos_or_panic};
//! use optionstratlib::ExpirationDate;
//! use optionstratlib::strategies::Strategies;
//! use optionstratlib::strategies::bull_call_spread::BullCallSpread;
//! use optionstratlib::strategies::base::{BreakEvenable, BasicAble};
//! use optionstratlib::visualization::Graph;
//! use rust_decimal_macros::dec;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use optionstratlib::pricing::Profit;
//! let underlying_price = Positive::HUNDRED;
//!
//!     // Create a Bull Call Spread strategy
//!     let strategy = BullCallSpread::new(
//!         "AAPL".to_string(),
//!         underlying_price,
//!         pos_or_panic!(95.0),   // long_strike
//!         pos_or_panic!(105.0),  // short_strike  
//!         ExpirationDate::Days(pos_or_panic!(30.0)),
//!         pos_or_panic!(0.25),   // implied_volatility
//!         dec!(0.05),   // risk_free_rate
//!         pos_or_panic!(2.50),   // long_call_premium
//!         pos_or_panic!(2.50),   // long_call_open_fee
//!         pos_or_panic!(1.20),   // short_call_premium
//!         pos_or_panic!(1.20),   // short_call_close_fee
//!         Default::default(), Default::default(),
//!         Default::default(), Default::default()
//!     )?;
//!
//!     // Analyze the strategy
//!     tracing::info!("Strategy: {}", strategy.get_title());
//!     tracing::info!("Break-even points: {:?}", strategy.get_break_even_points()?);
//!     tracing::info!("Max profit: ${:.2}", strategy.get_max_profit().unwrap_or(Positive::ZERO));
//!     tracing::info!("Max loss: ${:.2}", strategy.get_max_loss().unwrap_or(Positive::ZERO));
//!     tracing::info!("Net premium: ${:.2}", strategy.get_net_premium_received()?);
//!
//!     // Calculate P&L at different price points
//!     let prices = vec![pos_or_panic!(90.0), pos_or_panic!(95.0), Positive::HUNDRED, pos_or_panic!(105.0), pos_or_panic!(110.0)];
//!     for price in prices {
//!         let pnl = strategy.get_point_at_price(&price)?;
//!         tracing::info!("P&L at ${}: ${:.2}", price, pnl.0);
//!     }
//!
//!     // Generate visualization
//!     #[cfg(feature = "plotly")]
//!     {
//!         strategy.write_html("Draws/Visualization/bull_call_spread.html".as_ref())?;
//!     }
//!
//!     Ok(())
//! }
//! # pub fn run() -> Result<(), Box<dyn std::error::Error>> { main() }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     #[cfg(feature = "visualization")]
//! #     example::run()?;
//! #     Ok(())
//! # }
//! ```
//!
//! ### Advanced Features: Volatility Analysis
//!
//! ```rust
//! # #[cfg(feature = "pricing")]
//! # mod example {
//! use optionstratlib::prelude::*;
//! use optionstratlib::volatility::implied_volatility;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an option for implied volatility calculation
//!     let mut option = Options::new(
//!         OptionType::European,
//!         Side::Long,
//!         "AAPL".to_string(),
//!         pos_or_panic!(105.0), // strike
//!         ExpirationDate::Days(pos_or_panic!(90.0)),
//!         pos_or_panic!(0.20), // initial IV guess
//!         Positive::ONE, // quantity
//!         Positive::HUNDRED, // underlying price
//!         dec!(0.05), // risk free rate
//!         OptionStyle::Call,
//!         pos_or_panic!(0.02), // dividend yield
//!         None,
//!     );
//!
//!     let market_price = pos_or_panic!(5.50);
//!     let iv = implied_volatility(market_price, &mut option, 100)?;
//!
//!     tracing::info!("Implied volatility: {:.2}%", iv.to_f64() * 100.0);
//!     Ok(())
//! }
//! # pub fn run() -> Result<(), Box<dyn std::error::Error>> { main() }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     #[cfg(feature = "pricing")]
//! #     example::run()?;
//! #     Ok(())
//! # }
//! ```
//!
//! ### Custom Strategy Creation
//!
//! ```rust
//! # #[cfg(feature = "strategies")]
//! # mod example {
//! use optionstratlib::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Define common parameters
//!     let underlying_symbol = "DAX".to_string();
//!     let underlying_price = pos_or_panic!(24000.0);
//!     let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
//!     let implied_volatility = pos_or_panic!(0.25);
//!     let risk_free_rate = dec!(0.05);
//!     let dividend_yield = pos_or_panic!(0.02);
//!     let fee = Positive::TWO;
//!
//!     // Create a long put option
//!     let long_put_option = Options::new(
//!         OptionType::European,
//!         Side::Long,
//!         underlying_symbol.clone(),
//!         pos_or_panic!(24070.0), // strike
//!         expiration.clone(),
//!         implied_volatility,
//!         Positive::ONE, // quantity
//!         underlying_price,
//!         risk_free_rate,
//!         OptionStyle::Put,
//!         dividend_yield,
//!         None,
//!     );
//!     let long_put = Position::new(
//!         long_put_option,
//!         pos_or_panic!(150.0), // premium
//!         Utc::now(),
//!         fee,
//!         fee,
//!         None,
//!         None,
//!     );
//!
//!     // Create a long call option
//!     let long_call_option = Options::new(
//!         OptionType::European,
//!         Side::Long,
//!         underlying_symbol.clone(),
//!         pos_or_panic!(24030.0), // strike
//!         expiration.clone(),
//!         implied_volatility,
//!         Positive::ONE, // quantity
//!         underlying_price,
//!         risk_free_rate,
//!         OptionStyle::Call,
//!         dividend_yield,
//!         None,
//!     );
//!     let long_call = Position::new(
//!         long_call_option,
//!         pos_or_panic!(120.0), // premium
//!         Utc::now(),
//!         fee,
//!         fee,
//!         None,
//!         None,
//!     );
//!
//!     // Create CustomStrategy with the positions
//!     let positions = vec![long_call, long_put];
//!     let strategy = CustomStrategy::new(
//!         "DAX Straddle Strategy".to_string(),
//!         underlying_symbol,
//!         "A DAX long straddle strategy".to_string(),
//!         underlying_price,
//!         positions,
//!         Positive::ONE,
//!         30,
//!         implied_volatility,
//!     )?;
//!
//!     tracing::info!("Strategy created: {}", strategy.get_title());
//!     Ok(())
//! }
//! # pub fn run() -> Result<(), Box<dyn std::error::Error>> { main() }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #     #[cfg(feature = "strategies")]
//! #     example::run()?;
//! #     Ok(())
//! # }
//! ```
//!
//! ## Testing
//!
//! OptionStratLib ships with a large, fully deterministic test suite
//! (3760 unit / integration tests + 205 doctests + property- and
//! identity-based regressions):
//!
//! ### **Running Tests**
//!
//! Run all tests:
//! ```bash
//! cargo test --all-features
//! ```
//!
//! Run tests for specific modules:
//! ```bash
//! cargo test strategies::bull_call_spread
//! cargo test pricing::black_scholes
//! cargo test volatility::utils
//! ```
//!
//! Run tests with output:
//! ```bash
//! cargo test -- --nocapture
//! ```
//!
//! ### **Test Categories**
//!
//! - **Unit Tests**: Individual function and method testing
//! - **Integration Tests**: Cross-module functionality under `tests/`
//! - **Strategy Tests**: Comprehensive strategy validation
//! - **Pricing Model Tests**: Accuracy and performance testing
//! - **Greeks Tests**: Mathematical precision validation
//! - **Visualization Tests**: Chart generation and export testing
//! - **Property-Based Tests**: Mathematical invariant testing with `proptest`
//!   (`crates/optionstratlib-pricing/tests/put_call_parity_test.rs`, `greeks_bounds_test.rs`)
//! - **Identity Regression Tests**: `crates/optionstratlib-pricing/tests/identities_test.rs`
//!   locks put-call parity, CRR → Black-Scholes convergence, and
//!   Greek sanity (`Γ_c = Γ_p`, `Vega_c = Vega_p`,
//!   `Δ_c − Δ_p ≈ e^{-qT}`).
//! - **Deterministic Monte-Carlo Tests**: Seeded via
//!   [`utils::deterministic_rng`] so arithmetic-precision shifts can't
//!   flip assertions.
//! - **Exotic Options Tests**: Complete coverage for all 14 exotic
//!   option types.
//!
//! ### **Benchmarking**
//!
//! Run performance benchmarks:
//! ```bash
//! cargo bench
//! ```
//!
//! Generate test coverage report:
//! ```bash
//! cargo tarpaulin --all-features --out Html
//! ```
//!
//! ## Examples
//!
//! Examples live in self-contained sub-crates under `examples/`, each
//! with its own `Cargo.toml`:
//!
//! - **`examples_strategies/`**: 25+ strategy demos
//! - **`examples_strategies_best/`**: Optimizer entry points
//!   (`get_best_area` / `get_best_ratio`) per strategy
//! - **`examples_strategies_delta/`**: Delta-neutrality workflows
//! - **`examples_chain/`**: Option chain construction, import/export,
//!   and async I/O
//! - **`examples_curves/`**: Greek curves (`charm`, `color`, `d1`, `d2`,
//!   `delta`, `gamma`, `rho`, `theta`, …) and vector curves
//! - **`examples_surfaces/`**: 3-D volatility surfaces
//! - **`examples_metrics/`**: Price / risk / liquidity / stress /
//!   temporal / composite metric curves and surfaces
//! - **`examples_volatility/`**: Implied-volatility solver walkthroughs
//! - **`examples_simulation/`**: Monte-Carlo random-walk demos for
//!   `LongCall`, `ShortPut`, position / strategy simulators, and
//!   random-walk-of-chain
//! - **`examples_exotics/`**: Exotic option pricing (barrier,
//!   cliquet, …)
//! - **`examples_visualization/`**: Interactive chart wiring
//!
//! Run any binary with the usual cargo invocation (from the repo
//! root, so relative data-fixture paths resolve correctly):
//!
//! ```bash
//! cargo run --manifest-path=examples/examples_strategies/Cargo.toml \
//!     --bin strategy_bull_call_spread
//! cargo run --manifest-path=examples/examples_simulation/Cargo.toml \
//!     --bin long_call_strategy_simulation --features plotly
//! cargo run --manifest-path=examples/examples_metrics/Cargo.toml \
//!     --bin implied_volatility_surface
//! ```
//!
//! Simulation-heavy demos (`*_strategy_simulation`, `position_simulator`,
//! `strategy_simulator`, `random_walk_chain`) use a demo-friendly
//! hourly grid so `cargo run` finishes in a few seconds in debug mode;
//! bump `n_steps` / `n_simulations` inside the binary if you want a
//! finer sample.
//!
//! ## Contribution and Contact
//!
//! ### **Contributing**
//!
//! Contributions are welcome! Please follow these guidelines:
//!
//! 1. **Fork** the repository
//! 2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
//! 3. **Commit** your changes: `git commit -m 'Add amazing feature'`
//! 4. **Push** to the branch: `git push origin feature/amazing-feature`
//! 5. **Open** a Pull Request
//!
//! ### **Development Setup**
//!
//! ```bash
//! git clone https://github.com/joaquinbejar/OptionStratLib.git
//! cd OptionStratLib
//! cargo build --all-features
//! cargo test --all-features
//! ```
//!
//! ### **Code Quality**
//!
//! - All code must pass `cargo clippy` without warnings
//! - Format code with `cargo fmt`
//! - Add tests for new functionality
//! - Update documentation for API changes
//! - Follow Rust 2024 edition best practices
//!
//!
//! ### **Support**
//!
//! - **Issues**: Report bugs and request features on GitHub
//! - **Discussions**: Join community discussions on GitHub Discussions
//! - **Documentation**: Comprehensive docs available at docs.rs
//!
//! ---
//!
//! **OptionStratLib v0.22.0** - Built with ❤️ in Rust for the financial community
//!

/// # OptionsStratLib: Financial Options Trading Library
///
/// A comprehensive library for options trading analytics, modeling, and strategy development.
/// Provides tools for pricing, risk assessment, strategy building, and performance analysis
/// of financial options across various market conditions.
///
/// ## Core Modules
extern crate core;

/// * `model` - Core data structures and models for options and derivatives.
///
/// Defines the fundamental data types and structures used throughout the library,
/// including option contract representations, positions, legs and trades.
/// Serves as the foundation for all other modules.
/// Defined by `optionstratlib-core` and re-exported here.
pub use optionstratlib_core::model;

/// * `analytics` - Strategy-neutral analytics: price-probability kernels and models.
///
/// Evaluates lognormal price distributions at expiry from prices, volatilities
/// and dates alone, and extracts risk-neutral densities and volatility skews
/// from an option chain (`RNDAnalysis`). The strategy layer builds its
/// probability analysis on top of these kernels; nothing here depends on a
/// concrete strategy.
///
/// Defined by `optionstratlib-analytics` and re-exported here (feature
/// `analytics`).
#[cfg(feature = "analytics")]
pub use optionstratlib_analytics::analytics;

/// * `backtesting` - Strategy backtests over simulated paths.
///
/// Evaluates a strategy on every path of a simulator, ends each path by an
/// exit policy and summarises the run: statistics, report types and
/// performance metrics.
///
/// Defined by `optionstratlib-backtest` and re-exported here (feature
/// `backtest`).
#[cfg(feature = "backtest")]
pub use optionstratlib_backtest::backtesting;

/// * `chains` - Functionality for working with options chains and series data.
///
/// Tools for parsing, manipulating, and analyzing options chain data. Includes
/// methods to filter chains by expiration, strike price, and other criteria,
/// as well as utilities for chain visualization and analysis.
///
/// Defined by `optionstratlib-market` and re-exported here (feature `market`).
#[cfg(feature = "market")]
pub use optionstratlib_market::chains;

/// * `constants` - Library-wide mathematical and financial constants.
///
/// Defines fundamental constants used throughout the library including mathematical
/// constants (π, epsilon values), market standards (trading days per year)
/// and time-unit conversions. Defined by `optionstratlib-core`; the pricing
/// solver defaults live in `pricing::constants` (feature `pricing`).
pub use optionstratlib_core::constants;

/// * `curves` - Generic two-dimensional curves on `Decimal` coordinates.
///
/// `Curve`, `Point2D` and the curve traits: construction, interpolation,
/// arithmetic and statistics. Defined by `optionstratlib-math`; option
/// projections onto a curve are `analytics::BasicCurves`.
#[cfg(feature = "math")]
pub use optionstratlib_math::curves;

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
///
/// Defined by `optionstratlib-math` and re-exported here (feature `math`).
#[cfg(feature = "math")]
pub use optionstratlib_math::geometrics;

/// * `greeks` - Calculation and management of option sensitivity metrics (Delta, Gamma, etc.).
///
/// Comprehensive implementation of options Greeks (sensitivity measures) including
/// Delta, Gamma, Theta, Vega, Rho, Vanna, Vomma, Veta, Charm and Color. Includes analytical
/// formulas, numerical approximations, and visualization tools for risk analysis.
///
/// Defined by `optionstratlib-pricing` and re-exported here (feature `pricing`).
#[cfg(feature = "pricing")]
pub use optionstratlib_pricing::greeks;

/// * `metrics` - Performance and risk metrics analysis for options.
///
/// Comprehensive tools for performance and risk analysis including:
/// - **Price Metrics**: Volatility skew analysis
/// - **Risk Metrics**: Implied volatility curves/surfaces, risk reversal curves, dollar gamma curves
///
/// ## Risk Metrics
///
/// The risk metrics module provides key tools for assessing market risk, sentiment, and exposure:
///
/// | Metric | Curve (by strike) | Surface (strike vs time) |
/// |--------|-------------------|--------------------------|
/// | Implied Volatility | `iv_curve()` | `iv_surface(days)` |
/// | Risk Reversal | `risk_reversal_curve()` | - |
/// | Dollar Gamma | `dollar_gamma_curve(style)` | - |
///
/// ### Implied Volatility
/// Shows how IV varies across strikes and time horizons. Essential for understanding
/// market expectations and pricing options.
///
/// ### Risk Reversal
/// Measures the difference between call and put implied volatilities, indicating
/// market sentiment (bullish vs bearish bias).
///
/// ### Dollar Gamma
/// Gamma exposure in monetary terms: `Dollar Gamma = Gamma × Spot² × 0.01`
/// Shows how much delta changes for a 1% move in the underlying.
///
/// Defined by `optionstratlib-analytics` and re-exported here (feature
/// `analytics`).
#[cfg(feature = "analytics")]
pub use optionstratlib_analytics::metrics;

/// * `pnl` - Profit and loss analysis tools for options positions.
///
/// [`pnl::PnL`], the [`pnl::PnLCalculator`] trait, transactions, delta
/// adjustments and P&L metrics documents for options and positions.
///
/// Defined by `optionstratlib-analytics` and re-exported here (feature
/// `analytics`).
#[cfg(feature = "analytics")]
pub use optionstratlib_analytics::pnl;

/// * `pricing` - Option pricing models including Black-Scholes and numerical methods.
///
/// Implementations of various option pricing models including Black-Scholes-Merton,
/// binomial trees, Monte Carlo simulation, and finite difference methods. Supports
/// European, American, and exotic options.
///
/// Defined by `optionstratlib-pricing` and re-exported here (feature `pricing`).
#[cfg(feature = "pricing")]
pub use optionstratlib_pricing::pricing;

/// * `risk` - Risk assessment and management tools for options portfolios.
///
/// SPAN margin ([`risk::SPANMargin`]), the VaR, CVaR and Sharpe figures of a
/// simulated position (`RiskMetricsSimulation`) and risk categories.
///
/// Defined by `optionstratlib-analytics` and re-exported here (feature
/// `analytics`).
#[cfg(feature = "analytics")]
pub use optionstratlib_analytics::risk;

/// * `simulation` - Simulation techniques for scenario analysis.
///
/// Framework for Monte Carlo and other simulation methods to model potential
/// market scenarios. Includes path generation algorithms, exit policies and
/// the generic evaluation and statistics of simulated paths.
///
/// Defined by `optionstratlib-simulation` and re-exported here (feature
/// `simulation`).
#[cfg(feature = "simulation")]
pub use optionstratlib_simulation::simulation;

/// * `strategies` - Pre-defined option strategy templates and building blocks.
///
/// Library of common option strategies (spreads, straddles, condors, etc.) with
/// implementation helpers, parameter optimization, and analysis tools. Supports
/// strategy composition and customization.
///
/// Defined by `optionstratlib-strategies` and re-exported here (feature
/// `strategies`).
#[cfg(feature = "strategies")]
pub use optionstratlib_strategies::strategies;

/// * `surfaces` - Generic three-dimensional surfaces on `Decimal` coordinates.
///
/// `Surface`, `Point3D` and the surface traits: construction, interpolation,
/// arithmetic and metric extraction. Defined by `optionstratlib-math`
/// (feature `math`); option
/// projections onto a surface are `analytics::BasicSurfaces`.
#[cfg(feature = "math")]
pub use optionstratlib_math::surfaces;

/// * `utils` - General utility functions for data manipulation and calculations.
///
/// Collection of helper functions and utilities used across the library for
/// data manipulation, mathematical operations, date handling, and other
/// common tasks in financial calculations.
///
/// Defined by `optionstratlib-core` and re-exported here.
pub use optionstratlib_core::utils;

/// * `visualization` - Tools for plotting and visual representation of options data.
///
/// Chart data, the `Graph` contract and its implementations for options,
/// positions, curves, surfaces, simulations and strategies, with Plotly
/// rendering behind `plotly` and PNG/SVG export behind `static_export`.
///
/// Defined by `optionstratlib-visualization` and re-exported here (feature
/// `visualization`).
#[cfg(feature = "visualization")]
pub use optionstratlib_visualization::visualization;

/// * `volatility` - Volatility modeling, forecasting, and analysis utilities.
///
/// Comprehensive tools for volatility analysis including historical volatility calculation,
/// implied volatility determination, volatility forecasting models (GARCH, EWMA), and
/// volatility skew/smile analysis.
///
/// Defined by `optionstratlib-pricing` and re-exported here (feature `pricing`).
#[cfg(feature = "pricing")]
pub use optionstratlib_pricing::volatility;

/// * `series` - Functionality for working with collections of option chains across expirations.
///
/// Provides tools to manage, filter, and analyze multiple option chains grouped by expiration dates.
/// Includes utilities for constructing series data, navigating expirations, and performing
/// cross-expiration analysis and visualization.
///
/// Defined by `optionstratlib-market` and re-exported here (feature `market`).
#[cfg(feature = "market")]
pub use optionstratlib_market::series;

/// Domain vocabulary, extension traits and the entry type of each capability;
/// the module docs give the selection rule and the feature of each item.
pub mod prelude;

pub use optionstratlib_core::{assert_decimal_eq, d2f, d2fu, f2d, f2du, nz};
/// `impl Graph` for a payoff strategy, defined by `optionstratlib-visualization`
/// (feature `visualization`).
#[cfg(feature = "visualization")]
pub use optionstratlib_visualization::impl_graph_for_payoff_strategy;

pub use model::ExpirationDate;
pub use model::Options;
pub use model::types::{OptionStyle, OptionType, RainbowType, Side};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the library version
#[must_use]
pub fn version() -> &'static str {
    VERSION
}
