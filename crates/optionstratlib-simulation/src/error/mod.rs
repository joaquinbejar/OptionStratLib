//! Errors owned by the simulation layer.
//!
//! Each failure here is raised while generating a walk, validating its
//! parameters or stepping through it. Errors of the layers below
//! (`DecimalError`, `OptionsError`, `PricingError`, `VolatilityError`, …)
//! stay with core and pricing, and the variants below that wrap them carry
//! those types unchanged.
//!
//! # Canonical paths (#550)
//!
//! Every error type is exported flat from this module, for example
//! `optionstratlib_simulation::error::SimulationError`. The file module
//! `simulation` is private: it held nothing the flat paths do not, and was
//! public only because 0.21 exposed it.
//!
//! ```rust
//! use optionstratlib_simulation::error::{SimulationError, SimulationResult};
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_simulation::error::simulation::SimulationError;
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_simulation::error::simulation::SimulationResult;
//! ```

/// ### Simulation Errors (`SimulationError`)
/// Handles:
/// * Random walk generation failures
/// * Monte Carlo simulation errors
/// * Stochastic process parameter validation
/// * Step calculation issues
mod simulation;

pub use simulation::{SimulationError, SimulationResult};
