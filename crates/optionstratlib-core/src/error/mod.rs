//! Errors owned by the core layer.
//!
//! Each file here belongs to `optionstratlib-core` because the failure it
//! describes is raised by a core type: a decimal operation, an option
//! contract, a position or a trade. Errors raised by higher layers (pricing,
//! market, analytics, strategies, …) live with those layers, so core never
//! names an error it cannot raise.
//!
//! # Canonical paths (#550)
//!
//! Every error type is exported flat from this module, for example
//! `optionstratlib_core::error::DecimalError`. The file modules `decimal`
//! and `trade` are private: they held nothing the flat paths do not, and
//! were public only because 0.21 exposed them. `position` stays public
//! because its detail enums (`PositionValidationErrorKind`, …) are not
//! flattened: kind names collide across crates once the facade gathers every
//! crate's errors (see `position`'s module docs).
//!
//! ```rust
//! use optionstratlib_core::error::{DecimalError, DecimalResult, TradeError};
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_core::error::decimal::DecimalError;
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_core::error::decimal::DecimalResult;
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_core::error::trade::TradeError;
//! ```

/// ### Common error kinds (`OperationErrorKind`)
/// Provides a common set of error kinds used across various modules:
/// * Validation failures
/// * Mathematical errors
/// * Input/output errors
/// * Data consistency issues
mod common;

/// ### Decimal Errors (`DecimalError`)
/// Handles:
/// * Decimal conversions
/// * Precision management
/// * Arithmetic operations
/// * Boundary validations
mod decimal;

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

/// ### Trade Errors (`TradeError`)
/// Covers:
/// * Trade execution issues
/// * Trade parameter validation
mod trade;

pub use common::OperationErrorKind;
pub use decimal::{DecimalError, DecimalResult};
pub use options::{OptionsError, OptionsResult};
pub use position::PositionError;
pub use trade::TradeError;
