//! Errors owned by the simulation layer.
//!
//! Each failure here is raised while generating a walk, validating its
//! parameters or stepping through it. Errors of the layers below
//! (`DecimalError`, `OptionsError`, `PricingError`, `VolatilityError`, …)
//! stay with core and pricing, and the variants below that wrap them carry
//! those types unchanged.

/// ### Simulation Errors (`SimulationError`)
/// Handles:
/// * Random walk generation failures
/// * Monte Carlo simulation errors
/// * Stochastic process parameter validation
/// * Step calculation issues
pub mod simulation;

pub use simulation::{SimulationError, SimulationResult};
