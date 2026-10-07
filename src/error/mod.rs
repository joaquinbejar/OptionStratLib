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
//! # #[cfg(feature = "market")]
//! # mod example {
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
//! # }
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
//! ## Canonical paths (0.22, #550)
//!
//! Every error type has one canonical facade path, flat in this module:
//! `optionstratlib::error::PricingError`, `optionstratlib::error::ChainError`,
//! the aggregate `optionstratlib::error::Error`, and so on. The defining
//! component crate exposes the same type at `optionstratlib_<crate>::error::<Type>`.
//!
//! The five component error modules that hold detail enums (the `...Kind`
//! types) are re-exported as modules, and the kinds are canonical there:
//! `error::position` (core), `error::greeks` (pricing), `error::chains`
//! (market), `error::probability` (analytics) and `error::strategies`
//! (strategies). They are not flattened into this module because their
//! names collide across crates: `StrategyErrorKind` is defined in both
//! `error::position` and `error::chains`, `PriceErrorKind` in both
//! `error::probability` and `error::strategies`.
//!
//! The 0.21 file modules that held nothing but a type already exported flat
//! here are gone: `error::decimal`, `error::trade`, `error::curves`,
//! `error::pricing`, `error::simulation`, and `error::unified` for
//! `error::Error`. Their types keep their flat paths. (The error codes on the
//! `compile_fail` examples below are checked by nightly rustdoc only; stable
//! checks that the import fails.)
//!
//! ```rust
//! use optionstratlib::error::{DecimalError, DecimalResult, TradeError};
//! # #[cfg(feature = "math")]
//! use optionstratlib::error::{CurveError, CurvesResult};
//! # #[cfg(feature = "pricing")]
//! use optionstratlib::error::{PricingError, PricingResult};
//! # #[cfg(feature = "simulation")]
//! use optionstratlib::error::{SimulationError, SimulationResult};
//! # #[cfg(feature = "visualization")]
//! use optionstratlib::error::Error;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::error::decimal::DecimalError;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::error::trade::TradeError;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::error::curves::CurveError;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::error::pricing::PricingError;
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::error::simulation::SimulationError;
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib::error::unified::Error;
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
//! | simulation | `simulation.rs`, now in `optionstratlib-simulation` |
//! | market | `chains.rs`, `csv.rs` (behind `io`), now in `optionstratlib-market` |
//! | analytics | `transaction.rs`, `probability.rs`, `projections.rs`, now in `optionstratlib-analytics` |
//! | strategies | `strategies.rs`, now in `optionstratlib-strategies` |
//! | backtest | `backtesting.rs`, now in `optionstratlib-backtest` |
//! | visualization | `graph.rs`, now in `optionstratlib-visualization` |
//! | facade | `unified.rs`, this file's re-exports |
//!
//! Variants that still reference a higher layer
//! (`CurveError::{MetricsError, Greeks, Graph}`, `SurfaceError::Greeks` and
//! its graph variants, `VolatilityError::Chain`) are removed in the
//! batch that follows the 0.22.0 version bump; removing a variant is a
//! breaking change the published-baseline semver gate rejects before then.

/// ### Unified Error Type
/// Top-level error type that encompasses all errors in the library.
/// Provides a single error type for unified error handling across modules.
/// It wraps `GraphError`, so it needs `visualization`.
#[cfg(feature = "visualization")]
mod unified;

/// Core errors (`optionstratlib-core`): decimal arithmetic, option
/// contracts, positions and trades.
pub use optionstratlib_core::error::{
    DecimalError, DecimalResult, OperationErrorKind, OptionsError, OptionsResult, PositionError,
    TradeError, position,
};

/// Math errors (`optionstratlib-math`): curves, surfaces, interpolation and
/// the metrics extracted from them.
#[cfg(feature = "math")]
pub use optionstratlib_math::error::{
    CurveError, CurvesResult, InterpolationError, MetricsError, SurfaceError,
};

/// Pricing errors (`optionstratlib-pricing`): pricing models, Greeks and
/// volatility solvers.
#[cfg(feature = "pricing")]
pub use optionstratlib_pricing::error::{
    GreeksError, GreeksResult, PricingError, PricingResult, VolatilityError, greeks,
};

/// Simulation errors (`optionstratlib-simulation`): random walk generation,
/// stochastic process parameters and step calculation.
#[cfg(feature = "simulation")]
pub use optionstratlib_simulation::error::{SimulationError, SimulationResult};

/// Market errors (`optionstratlib-market`): option chains, series and
/// OHLCV readers.
#[cfg(feature = "market")]
pub use optionstratlib_market::error::{ChainError, OhlcvError, chains};

/// Analytics errors (`optionstratlib-analytics`): probability kernels,
/// projections onto curves and surfaces, and transactions.
#[cfg(feature = "analytics")]
pub use optionstratlib_analytics::error::{
    ProbabilityError, ProbabilityResult, ProjectionError, TransactionError, probability,
};

/// Strategy errors (`optionstratlib-strategies`): building, validating,
/// optimising and evaluating a strategy.
#[cfg(feature = "strategies")]
pub use optionstratlib_strategies::error::{
    AdjustmentError, StrategyError, StrategyResult, strategies,
};

/// Backtest errors (`optionstratlib-backtest`): a strategy driven through a
/// simulation.
#[cfg(feature = "backtest")]
pub use optionstratlib_backtest::error::BacktestError;
/// Visualization errors (`optionstratlib-visualization`): a chart that could
/// not be built or written.
#[cfg(feature = "visualization")]
pub use optionstratlib_visualization::error::GraphError;
#[cfg(feature = "visualization")]
pub use unified::Error;
