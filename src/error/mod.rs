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
//! └── volatility.rs   - Volatility calculation errors
//! ```

/// ### Chain Errors (`ChainError`)
/// Handles:
/// * Option data validation
/// * Chain construction
/// * File operations (CSV/JSON)
/// * Strategy validation
///
pub mod chains;

/// Provides a common set of error kinds used across various modules:
/// * Validation failures
/// * Mathematical errors
/// * Input/output errors
/// * Data consistency issues
mod common;

/// ### Curve Errors (`CurveError`) 
/// Handles:
/// * Yield curve construction
/// * Forward rate calculations
/// * Market data fitting issues
/// * Term structure consistency
pub mod curves;

/// ### Decimal Errors (`DecimalError`)
/// Handles:
/// * Decimal conversions
/// * Precision management
/// * Arithmetic operations
/// * Boundary validations
pub mod decimal;

/// ### Greeks Errors (`GreeksError`)
/// Handles:
/// * Greeks calculations
/// * Mathematical validation
/// * Input parameter validation
/// * Numerical computations
pub mod greeks;

/// ### Options Errors (`OptionsError`)
/// Core module handling:
/// * Option validation errors
/// * Pricing model failures
/// * Parameter boundary violations
/// * Contract specification issues
mod options;

/// ### Position Errors (`PositionError`)
/// Manages:
/// * Position validation
/// * Strategy operations
/// * Position limits
/// * Option style/side compatibility
pub mod position;

/// ### Probability Errors (`ProbabilityError`)
/// Manages:
/// * Statistical calculations
/// * Range analysis
/// * Probability distributions
/// * Market scenarios
pub mod probability;

/// ### Strategy Errors (`StrategyError`)
/// Covers:
/// * Price calculations
/// * Break-even analysis
/// * Profit/Loss calculations
/// * Operation validation
pub mod strategies;

/// ### Interpolation Errors (`InterpolationError`)
/// Manages:
/// * Data point validation
/// * Interpolation method errors
/// * Boundary conditions
/// * Mathematical approximation issues
mod interpolation;

/// ### Metrics Errors (`MetricsError`)
/// Handles:
/// * Performance calculation failures
/// * Risk metric validation
/// * Statistical measurement errors
/// * Benchmark comparison issues
mod metrics;

/// ### Surface Errors (`SurfaceError`)
/// Covers:
/// * Surface construction failures
/// * Volatility skew/smile errors
/// * Surface calibration issues
/// * Dimensional and data completeness errors
mod surfaces;

/// ### Volatility Errors (`VolatilityError`)
/// Handles:
/// * Implied volatility calculation failures
/// * Historical volatility estimation issues
/// * Volatility model parameter validation
/// * Market data consistency checks
mod volatility;


pub use chains::ChainError;
pub use common::OperationErrorKind;
pub use curves::CurveError;
pub use decimal::{DecimalError, DecimalResult};
pub use greeks::GreeksError;
pub use interpolation::InterpolationError;
pub use metrics::MetricsError;
pub use options::{OptionsError, OptionsResult};
pub use position::PositionError;
pub use probability::ProbabilityError;
pub use strategies::StrategyError;
pub use surfaces::SurfaceError;
pub use volatility::VolatilityError;