#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-simulation
//!
//! The simulation engine of OptionStratLib: random walks, stochastic
//! processes, simulators, exit policies and the generic evaluation and
//! statistics of simulated paths. It depends on `optionstratlib-core` and
//! `optionstratlib-pricing`, and on no option chain, strategy, backtesting,
//! plotting, I/O or terminal-presentation code.
//!
//! - [`simulation`]: the stochastic processes ([`simulation::WalkType`]:
//!   Brownian, geometric Brownian, log-returns, mean-reverting, jump
//!   diffusion, GARCH, Heston, telegraph, custom and historical) and their
//!   default kernels ([`simulation::WalkTypeAble`]); walk parameters and
//!   steps ([`simulation::WalkParams`], [`simulation::steps::Step`],
//!   [`simulation::steps::Xstep`], [`simulation::steps::Ystep`]); the walk
//!   driver shared by every generator ([`simulation::walk_steps`],
//!   [`simulation::walk_steps_par`], [`simulation::generator_positive`]);
//!   [`simulation::randomwalk::RandomWalk`] and
//!   [`simulation::simulator::Simulator`]; the Ornstein-Uhlenbeck process
//!   ([`simulation::generate_ou_process`]); exit policies
//!   ([`simulation::ExitPolicy`], [`simulation::check_exit_policy`]); and the
//!   generic path evaluation ([`simulation::PathEvaluator`],
//!   [`simulation::PathOutcome`], [`simulation::PathStatistics`],
//!   [`simulation::evaluate_paths`]).
//! - [`error`]: [`error::SimulationError`] and [`error::SimulationResult`].
//!
//! Prices along a path are `Positive`, drifts and statistics `Decimal`;
//! `f64` stays inside the kernels, and a non-finite value at an
//! `f64` → `Decimal` boundary is reported as
//! [`error::SimulationError::NonFinite`].
//!
//! The evaluation of a strategy over simulated paths, its statistics report
//! and the charts of a walk stay with the backtesting and visualization
//! layers, which the `optionstratlib` facade provides; the chain and series
//! generators built on these walks are in `optionstratlib-market`, behind
//! its `synthetic` feature.
//!
//! ## Imports
//!
//! There is no prelude, as in the other component crates (ADR-0001 D7): the
//! [`simulation`] module root re-exports the capability, and the
//! `optionstratlib` facade prelude serves broad imports.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the simulation
//!   types and enables `schema` in core and pricing.

/// Random walks, stochastic processes, simulators, exit policies and the
/// generic evaluation of simulated paths.
pub mod simulation;

/// Errors raised by the simulation layer.
pub mod error;

/// Version of the `optionstratlib-simulation` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
