#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-market
//!
//! Market data of OptionStratLib: option chains and option series, their
//! build parameters, parsing, and CSV, JSON and ZIP readers and writers. It
//! depends on `optionstratlib-core`, `optionstratlib-math` and
//! `optionstratlib-pricing`, on `optionstratlib-simulation` only under the
//! `synthetic` feature, and on no analytics, strategy, backtesting or
//! plotting code.
//!
//! - [`chains`]: [`chains::OptionChain`], [`chains::OptionData`], build
//!   parameters, strategy-leg lookups and OHLCV readers.
//! - [`series`]: [`series::OptionSeries`] across expirations and its build
//!   parameters.
//! - [`error`]: [`error::ChainError`] and [`error::OhlcvError`].
//!
//! Analyses over a chain (risk-neutral density, projections, metrics) are
//! analytics, in `optionstratlib-analytics`. The chain and series generators
//! driven by a random walk are here, behind `synthetic`.
//!
//! ## Example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-market`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/market); `make tree-example-direct-market`
//! asserts its resolved graph.
//!
//! ## Features
//!
//! - `io` (off by default): the filesystem entry points, which are
//!   `OptionChain::{save_to_csv, load_from_csv, save_to_json, load_from_json}`
//!   and the OHLCV ZIP reader in `chains::csv` with its `OhlcvError`. It adds
//!   `csv` and `zip`. Without it the crate is an in-memory chain and series
//!   library; serde (de)serialization works either way.
//! - `async` (off by default, implies `io`): `tokio`-backed `*_async`
//!   wrappers that run that blocking file I/O on `spawn_blocking`.
//! - `synthetic` (off by default): the simulation-backed generators
//!   `chains::generator_optionchain` and `series::generator_optionseries`,
//!   and `From<SimulationError> for ChainError`. It adds
//!   `optionstratlib-simulation`, the only edge from market data to the
//!   simulation engine (ADR-0003); a simulation failure reaches the caller as
//!   `ChainError::Generator`, whose source downcasts to `SimulationError`.
//! - `schema` (off by default): derives `utoipa::ToSchema` on the market
//!   types and enables `schema` in core, math and pricing.

/// Option chains: `OptionChain`, `OptionData`, build parameters, strategy-leg
/// lookups and OHLCV readers.
pub mod chains;

/// Option series across expirations: `OptionSeries` and its build
/// parameters.
pub mod series;

/// Errors raised by the market layer.
pub mod error;

/// Deterministic walkers shared by the generator tests.
#[cfg(test)]
#[cfg(feature = "synthetic")]
mod walk_test_support;

/// Version of the `optionstratlib-market` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
