//! Errors owned by the strategies layer.
//!
//! Each failure here is raised while building, validating, optimising or
//! evaluating a strategy. Errors of the layers below (`PositionError`,
//! `PricingError`, `GreeksError`, `ProbabilityError`, …) stay with core,
//! pricing and analytics, and the variants below that wrap them carry those
//! types unchanged.

/// ### Strategy Errors (`StrategyError`)
/// Covers:
/// * Price calculations
/// * Break-even analysis
/// * Profit/Loss calculations
/// * Operation validation
pub mod strategies;

pub use strategies::{StrategyError, StrategyResult};
