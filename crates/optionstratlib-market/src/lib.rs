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
//! `optionstratlib-pricing`, and on no analytics, strategy, simulation,
//! backtesting or plotting code.
//!
//! - [`chains`]: [`chains::OptionChain`], [`chains::OptionData`], build
//!   parameters, strategy-leg lookups and OHLCV readers.
//! - [`series`]: [`series::OptionSeries`] across expirations and its build
//!   parameters.
//! - [`error`]: [`error::ChainError`] and [`error::OhlcvError`].
//!
//! Analyses over a chain (risk-neutral density, projections, metrics) are
//! analytics, and the chain and series generators driven by a random walk
//! need the simulation engine; the `optionstratlib` facade provides both
//! (the generators under its `synthetic` feature) until the simulation layer
//! is its own crate.
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
