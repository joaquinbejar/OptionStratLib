/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/

//! # Error Module
//!
//! This module provides a comprehensive error handling system for the library's various components.
//! It includes specialized error types for different aspects of options trading and analysis.
//!
//! ## Key Components
//!
//! * `chains` - Error types for option chain operations and validations
//! * `position` - Error handling for position management and strategy operations
//! * `probability` - Error types for probability analysis and calculations
//! * `strategies` - Error handling for trading strategy operations
//!
//! ## Error Categories
//!
//! ### Chain Errors (`ChainError`)
//! Handles errors related to:
//! * Option data validation
//! * Chain building operations
//! * File operations (CSV/JSON)
//! * Strategy validations
//!
//! ### Position Errors (`PositionError`)
//! Manages errors for:
//! * Strategy operations
//! * Position validation
//! * Position limits
//! * Option style and side compatibility
//!
//! ### Probability Errors (`ProbabilityError`)
//! Covers errors in:
//! * Probability calculations
//! * Profit/Loss range analysis
//! * Expiration date handling
//! * Price parameter validation
//!
//! ### Strategy Errors (`StrategyError`)
//! Handles errors for:
//! * Price calculations
//! * Break-even points
//! * Profit/Loss calculations
//! * Strategy operations
//!
//! ## Usage Example
//!
//! ```rust
//! use optionstratlib::error::chains::ChainError;
//! use optionstratlib::error::position::PositionError;
//!
//! // Chain error handling
//! fn process_chain() -> Result<(), ChainError> {
//!     // Implementation
//!     Ok(())
//! }
//!
//! // Position error handling
//! fn validate_position() -> Result<(), PositionError> {
//!     // Implementation
//!     Ok(())
//! }
//! ```
//!
//! Each error type implements standard traits like `Error`, `Display`, and conversion traits
//! for seamless error handling and propagation throughout the library.

pub mod chains;
mod common;
pub mod curves;
pub mod decimal;
pub mod greeks;
mod options;
pub mod position;
pub mod probability;
pub mod strategies;

pub use chains::ChainError;
pub use common::OperationErrorKind;
pub use curves::CurvesError;
pub use decimal::{DecimalError, DecimalResult};
pub use greeks::GreeksError;
pub use options::{OptionsError, OptionsResult};
pub use position::PositionError;
pub use probability::ProbabilityError;
pub use strategies::StrategyError;
