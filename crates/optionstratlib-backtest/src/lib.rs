#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]
// Per rules/global_rules.md §Arithmetic, an operator that overflows, divides
// by zero or breaks the `Positive` invariant aborts the caller, so every
// operation on `Decimal`, `Positive` and the integers goes through its
// checked form (#788, #808). `clippy.toml` exempts the unary minus on
// `Decimal`, which cannot overflow. Casts that truncate, wrap or drop the
// sign are denied for the same reason. Unit tests are exempt, as above.
#![deny(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
#![cfg_attr(
    test,
    allow(
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )
)]

//! # optionstratlib-backtest
//!
//! Strategy backtests over simulated paths: a strategy is evaluated on every
//! path of a simulator ([`backtesting::Simulate`]), each path ends by an exit
//! policy, and the run is summarised as [`backtesting::SimulationStatsResult`]
//! with its report types and metrics. It depends on `optionstratlib-core`,
//! `optionstratlib-pricing`, `optionstratlib-simulation`,
//! `optionstratlib-analytics` and `optionstratlib-strategies`, and on no
//! option-chain I/O, plotting or async code.
//!
//! - [`backtesting`]: [`backtesting::Simulate`] and its single-leg
//!   implementations, the adapters from the simulation engine's generic
//!   `PathOutcome` / `PathStatistics` to the backtest results, run
//!   statistics, report types and performance metrics.
//! - [`error`]: the [`error::BacktestError`] type.
//!
//! The generic engine (random walks, exit policies, path evaluation and
//! statistics) is `optionstratlib-simulation`'s; this crate adapts it to
//! strategies and owns the reports. Charts stay with the visualization layer,
//! `optionstratlib-visualization`, which the `optionstratlib` facade
//! re-exports.
//!
//! ## Imports
//!
//! There is no prelude, as in the other component crates (ADR-0001 D7): the
//! module roots re-export each capability, and the `optionstratlib` facade
//! prelude serves broad imports.
//!
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core`, `optionstratlib-pricing`,
//!   `optionstratlib-simulation`, `optionstratlib-analytics` and
//!   `optionstratlib-strategies`.
//! - **Must not depend on** `optionstratlib-visualization`; `make check-graph`
//!   enforces the layering (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::backtesting` and `BacktestError` in
//!   `optionstratlib::error`, under the facade feature `backtest`. The facade
//!   paths are the same types as the paths here; the [ownership
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
//! use optionstratlib_backtest::backtesting::SimulationStatsResult;
//!
//! // A run's statistics start empty; `Simulate::simulate` fills them from a
//! // `Simulator`'s paths (see the runnable example below).
//! let stats = SimulationStatsResult::default();
//! assert_eq!(stats.total_simulations, 0);
//! ```
//!
//! ## Runnable example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-backtest`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/backtest); `make tree-example-direct-backtest`
//! asserts its resolved graph.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the backtest
//!   types and enables `schema` in core, pricing, simulation, analytics and
//!   strategies.

/// Strategy backtests over simulated paths: evaluation, statistics, reports
/// and metrics.
pub mod backtesting;

/// Errors raised by the backtest layer.
pub mod error;

/// Version of the `optionstratlib-backtest` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
