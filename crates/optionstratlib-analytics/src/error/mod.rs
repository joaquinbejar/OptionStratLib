//! Errors owned by the analytics layer.
//!
//! Each failure here is raised while computing a probability, projecting
//! option data onto a curve or a surface, or recording a transaction. Errors
//! of the layers below (`DecimalError`, `CurveError`, `GreeksError`,
//! `ChainError`, …) stay with core, math, pricing and market, and the
//! variants below that wrap them carry those types unchanged.

/// ### Probability Errors (`ProbabilityError`)
/// Manages:
/// * Statistical calculations
/// * Range analysis
/// * Probability distributions
/// * Market scenarios
pub mod probability;

/// Analytics-owned projection failures (curves and surfaces from option data).
mod projections;

/// Failures of the operations on a `Transaction`.
mod transaction;

pub use probability::{ProbabilityError, ProbabilityResult};
pub use projections::ProjectionError;
pub use transaction::TransactionError;
