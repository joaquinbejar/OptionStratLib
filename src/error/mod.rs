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
//! ## Core Error Types
//!
//! ### Options and Pricing
//! * `OptionsError` - Core errors for option operations and validations
//! * `GreeksError` - Errors in Greeks calculations (delta, gamma, etc.)
//! * `ImpliedVolatilityError` - Errors in implied volatility calculations
//!
//! ### Trading and Analysis
//! * `ChainError` - Option chain operations and data management
//! * `PositionError` - Position management and trading operations
//! * `StrategyError` - Trading strategy validation and execution
//! * `ProbabilityError` - Statistical analysis and probability calculations
//!
//! ### Mathematical and Data
//! * `CurvesError` - Curve fitting and mathematical operations
//! * `DecimalError` - Decimal number handling and precision
//!
//! ## Error Categories By Module
//!
//! ### Chain Errors (`ChainError`)
//! Handles:
//! * Option data validation
//! * Chain construction
//! * File operations (CSV/JSON)
//! * Strategy validation
//!
//! ### Position Errors (`PositionError`)
//! Manages:
//! * Position validation
//! * Strategy operations
//! * Position limits
//! * Option style/side compatibility
//!
//! ### Greeks Errors (`GreeksError`)
//! Handles:
//! * Greeks calculations
//! * Mathematical validation
//! * Input parameter validation
//! * Numerical computations
//!
//! ### Strategy Errors (`StrategyError`)
//! Covers:
//! * Price calculations
//! * Break-even analysis
//! * Profit/Loss calculations
//! * Operation validation
//!
//! ### Probability Errors (`ProbabilityError`)
//! Manages:
//! * Statistical calculations
//! * Range analysis
//! * Probability distributions
//! * Market scenarios
//!
//! ### Decimal Errors (`DecimalError`)
//! Handles:
//! * Decimal conversions
//! * Precision management
//! * Arithmetic operations
//! * Boundary validations
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
//! ## Implementation Details
//!
//! * Each error type has specialized constructors for common cases
//! * Error types support standard error trait implementations
//! * Conversion traits implemented for seamless error handling
//! * Thread-safe error types (`Send` + `Sync`)
//! * Comprehensive string formatting for logging
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
//! ├── options.rs      - Core options errors
//! ├── position.rs     - Position management errors
//! ├── probability.rs  - Statistical analysis errors
//! ├── strategies.rs   - Trading strategy errors
//! └── volatility.rs   - Volatility calculation errors
//! ```

pub mod chains;
mod common;
pub mod curves;
pub mod decimal;
pub mod greeks;
mod options;
pub mod position;
pub mod probability;
pub mod strategies;

mod volatility;
mod surfaces;

pub use chains::ChainError;
pub use common::OperationErrorKind;
pub use curves::CurvesError;
pub use decimal::{DecimalError, DecimalResult};
pub use greeks::GreeksError;
pub use options::{OptionsError, OptionsResult};
pub use position::PositionError;
pub use probability::ProbabilityError;
pub use strategies::StrategyError;
pub use volatility::ImpliedVolatilityError;
pub use surfaces::SurfaceError;
