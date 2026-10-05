//! Errors owned by the core layer.
//!
//! Each file here belongs to `optionstratlib-core` because the failure it
//! describes is raised by a core type: a decimal operation, an option
//! contract, a position or a trade. Errors raised by higher layers (pricing,
//! market, analytics, strategies, …) live with those layers, so core never
//! names an error it cannot raise.

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
pub mod decimal;

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
pub mod trade;

pub use common::OperationErrorKind;
pub use decimal::{DecimalError, DecimalResult};
pub use options::{OptionsError, OptionsResult};
pub use position::PositionError;
pub use trade::TradeError;
