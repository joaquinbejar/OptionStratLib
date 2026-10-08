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
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core` and `optionstratlib-pricing`.
//! - **Must not depend on** `optionstratlib-market`, `-analytics`, `-strategies`,
//!   `-backtest` and `-visualization`; `make check-graph` enforces the layering
//!   (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::simulation` and `SimulationError` /
//!   `SimulationResult` in `optionstratlib::error`, under the facade feature
//!   `simulation`. The facade paths are the same types as the paths here; the
//!   [ownership
//!   map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
//!   lists every one with its feature.
//!
//! Moving from 0.21? The [0.22 architecture and adoption
//! guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
//! maps each redesign to its replacement workflow; 0.22 keeps no
//! compatibility with 0.21.
//!
//! ## Minimal example
//!
//! ```rust
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_simulation::simulation::generate_ou_process;
//!
//! // An Ornstein-Uhlenbeck path of 50 steps mean-reverting to 100.
//! let path = generate_ou_process(
//!     pos_or_panic!(100.0),
//!     pos_or_panic!(100.0),
//!     pos_or_panic!(0.5),
//!     pos_or_panic!(0.2),
//!     pos_or_panic!(0.01),
//!     50,
//! )?;
//! assert_eq!(path.len(), 50);
//! # Ok::<(), optionstratlib_simulation::error::SimulationError>(())
//! ```
//!
//! ## Runnable example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-simulation`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/simulation); `make tree-example-direct-simulation`
//! asserts its resolved graph.
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

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
