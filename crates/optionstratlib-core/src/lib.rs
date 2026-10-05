#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-core
//!
//! The core domain model of OptionStratLib: the types every other
//! OptionStratLib crate builds on. It holds no pricing model, market data,
//! strategy, backtesting or presentation code, and depends on no other
//! OptionStratLib crate.
//!
//! - [`model`]: [`model::Options`], [`model::Position`], [`model::leg::Leg`]
//!   and its spot, future and perpetual legs, [`model::Trade`], payoff
//!   contracts at expiry and checked `Decimal` helpers.
//! - [`error`]: the errors those types raise.
//! - [`utils`]: time frames and date helpers, the deterministic RNG,
//!   numeric helpers and [`utils::Len`].
//! - [`constants`]: library-wide numeric constants, market conventions and time units.
//!
//! Monetary values are [`rust_decimal::Decimal`] or [`positive::Positive`],
//! and arithmetic on them is checked.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the core types
//!   and enables the same derive on `positive`, `expiration_date`,
//!   `financial_types` and `option_type`.

/// Domain types: options, positions, legs, trades, payoffs and checked
/// `Decimal` helpers.
pub mod model;

/// Errors raised by the core domain types.
pub mod error;

/// Time frames, date helpers, the deterministic RNG and numeric helpers.
pub mod utils;

/// Library-wide numeric constants, market conventions and time units.
pub mod constants;

/// Version of the `optionstratlib-core` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
