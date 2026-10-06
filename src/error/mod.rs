/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/
//! # Error Module
//!
//! This module provides a comprehensive error handling system for options trading and financial calculations.
//! It defines specialized error types for different aspects of the library, including options trading,
//! pricing calculations, statistical analysis, and data management.
//!
//! ## Core Modules Overview
//!
//! ### Options and Pricing
//! * `OptionsError` - Core errors for option operations and validations
//! * `GreeksError` - Errors in Greeks calculations (delta, gamma, etc.)
//! * `VolatilityError` - Errors in volatility calculations including implied volatility
//!
//! ### Trading and Analysis
//! * `ChainError` - Option chain operations and data management
//! * `TradeError` - Trade position opening and status management
//! * `PositionError` - Position management and trading operations
//! * `StrategyError` - Trading strategy validation and execution
//! * `ProbabilityError` - Statistical analysis and probability calculations
//!
//! ### Mathematical and Data
//! * `CurveError` - Curve fitting and mathematical operations
//! * `DecimalError` - Decimal number handling and precision
//! * `InterpolationError` - Errors in data interpolation operations
//! * `MetricsError` - Performance and risk metrics calculation errors
//! * `SurfaceError` - Volatility and pricing surface construction errors
//!
//! ## Usage Example
//!
//! ```rust
//! use optionstratlib::error::{OptionsError, GreeksError, ChainError};
//!
//! // Options error handling
//! fn calculate_option_price() -> Result<f64, OptionsError> {
//!     // Implementation
//!     Ok(0.0)
//! }
//!
//! // Greeks calculation error handling
//! fn calculate_delta() -> Result<f64, GreeksError> {
//!     // Implementation
//!     Ok(0.0)
//! }
//!
//! // Chain operation error handling
//! fn process_option_chain() -> Result<(), ChainError> {
//!     // Implementation
//!     Ok(())
//! }
//! ```
//!
//! ## Error Design Principles
//!
//! * All error types implement standard traits (`Error`, `Display`, `Debug`)
//! * Structured error hierarchies for precise error handling
//! * Detailed error messages for debugging
//! * Clean error propagation through type conversions
//! * Context preservation in error chains
//!
//! ## Type Aliases
//!
//! * `OptionsResult<T>` - Specialized result type for options operations
//! * `DecimalResult<T>` - Specialized result type for decimal calculations
//!
//! ## Module Structure
//!
//! ```text
//! error/
//! ├── chains.rs       - Option chain errors
//! ├── common.rs       - Shared error types
//! ├── curves.rs       - Mathematical curve errors
//! ├── decimal.rs      - Decimal computation errors
//! ├── greeks.rs       - Greeks calculation errors
//! ├── interpolation.rs - Interpolation errors
//! ├── metrics.rs      - Performance metrics errors
//! ├── options.rs      - Core options errors
//! ├── position.rs     - Position management errors
//! ├── probability.rs  - Statistical analysis errors
//! ├── strategies.rs   - Trading strategy errors
//! ├── surfaces.rs     - Surface construction errors
//! ├── trade.rs        - Trade management errors
//! └── volatility.rs   - Volatility calculation errors
//! ```
//!
//! ## Ownership in the multi-crate workspace (ADR-0001 D6, roadmap M1-14)
//!
//! Every file in this directory has exactly one target crate, stated in its
//! own module docs. A file may wrap errors of lower layers with `#[from]`;
//! it never wraps a higher layer's error. Conversions whose source error is
//! owned by a higher layer live in that layer's file (for example
//! `impl From<StrategyError> for PositionError` sits in `strategies.rs`,
//! not in `position.rs`).
//!
//! | Target crate | Files |
//! | --- | --- |
//! | core | `common.rs`, `decimal.rs`, `options.rs`, `position.rs`, `trade.rs`, now in `optionstratlib-core` |
//! | math | `interpolation.rs`, `curves.rs`, `surfaces.rs`, `metrics.rs`, now in `optionstratlib-math` |
//! | pricing | `greeks.rs`, `volatility.rs`, `pricing.rs`, now in `optionstratlib-pricing` |
//! | simulation | `simulation.rs` |
//! | market | `chains.rs`, `csv.rs` (behind `io`), now in `optionstratlib-market` |
//! | analytics | `transaction.rs`, `probability.rs`, `projections.rs`, now in `optionstratlib-analytics` |
//! | strategies | `strategies.rs`, now in `optionstratlib-strategies` |
//! | visualization | `graph.rs` |
//! | facade | `unified.rs`, this file's re-exports |
//!
//! Variants that still reference a higher layer
//! (`CurveError::{MetricsError, Greeks, Graph}`, `SurfaceError::Greeks` and
//! its graph variants, `VolatilityError::Chain`,
//! `SimulationError::{Strategy, Chain, GraphError}`) are removed in the
//! batch that follows the 0.22.0 version bump; removing a variant is a
//! breaking change the published-baseline semver gate rejects before then.

/// Backtest-owned failures (a strategy driven through a simulation).
#[cfg(all(feature = "strategies", feature = "simulation"))]
mod backtesting;

#[cfg(all(feature = "strategies", feature = "simulation"))]
mod graph;

/// ### Simulation Errors (`SimulationError`)
/// Handles:
/// * Random walk generation failures
/// * Monte Carlo simulation errors
/// * Stochastic process parameter validation
/// * Step calculation issues
#[cfg(feature = "simulation")]
pub mod simulation;

/// ### Unified Error Type
/// Top-level error type that encompasses all errors in the library.
/// Provides a single error type for unified error handling across modules.
#[cfg(all(feature = "strategies", feature = "simulation"))]
pub mod unified;

/// Core errors (`optionstratlib-core`): decimal arithmetic, option
/// contracts, positions and trades.
pub use optionstratlib_core::error::{
    DecimalError, DecimalResult, OperationErrorKind, OptionsError, OptionsResult, PositionError,
    TradeError, decimal, position, trade,
};

/// Math errors (`optionstratlib-math`): curves, surfaces, interpolation and
/// the metrics extracted from them.
#[cfg(feature = "math")]
pub use optionstratlib_math::error::{
    CurveError, CurvesResult, InterpolationError, MetricsError, SurfaceError, curves,
};

/// Pricing errors (`optionstratlib-pricing`): pricing models, Greeks and
/// volatility solvers.
#[cfg(feature = "pricing")]
pub use optionstratlib_pricing::error::{
    GreeksError, PricingError, PricingResult, VolatilityError, greeks, pricing,
};

/// Market errors (`optionstratlib-market`): option chains, series and
/// OHLCV readers.
#[cfg(feature = "market")]
pub use optionstratlib_market::error::{ChainError, OhlcvError, chains};

/// Analytics errors (`optionstratlib-analytics`): probability kernels,
/// projections onto curves and surfaces, and transactions.
#[cfg(feature = "analytics")]
pub use optionstratlib_analytics::error::{
    ProbabilityError, ProjectionError, TransactionError, probability,
};

/// Strategy errors (`optionstratlib-strategies`): building, validating,
/// optimising and evaluating a strategy.
#[cfg(feature = "strategies")]
pub use optionstratlib_strategies::error::{StrategyError, strategies};

#[cfg(all(feature = "strategies", feature = "simulation"))]
pub use backtesting::BacktestError;
#[cfg(all(feature = "strategies", feature = "simulation"))]
pub use graph::GraphError;
#[cfg(feature = "simulation")]
pub use simulation::{SimulationError, SimulationResult};
#[cfg(all(feature = "strategies", feature = "simulation"))]
pub use unified::Error;
