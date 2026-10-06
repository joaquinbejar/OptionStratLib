#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

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
