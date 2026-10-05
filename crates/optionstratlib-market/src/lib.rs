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
//! - `async` (off by default): `tokio`-backed `*_async` readers and writers
//!   that run the blocking file I/O on `spawn_blocking`.
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

/// Version of the `optionstratlib-market` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
