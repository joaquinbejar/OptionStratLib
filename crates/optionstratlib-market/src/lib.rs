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
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core`, `optionstratlib-math` and
//!   `optionstratlib-pricing`; `optionstratlib-simulation` only under
//!   `synthetic`.
//! - **Must not depend on** `optionstratlib-analytics`, `-strategies`,
//!   `-backtest` and `-visualization`, and `optionstratlib-simulation` without
//!   `synthetic`; `make check-graph` enforces the layering (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::{chains, series}` and the market errors
//!   in `optionstratlib::error`, under the facade feature `market`; the facade's
//!   `io`, `async` and `synthetic` forward to the features below. The facade
//!   paths are the same types as the paths here; the [ownership
//!   map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
//!   lists every one with its feature.
//!
//! <!-- #553: link the 0.21 to 0.22 migration guide here -->
//!
//! ## Minimal example
//!
//! ```rust
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_market::chains::OptionChain;
//!
//! let chain = OptionChain::new("XYZ", pos_or_panic!(100.0), "2030-01-18".to_string(), None, None);
//! assert_eq!(chain.symbol, "XYZ");
//! ```
//!
//! ## Runnable example
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

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
