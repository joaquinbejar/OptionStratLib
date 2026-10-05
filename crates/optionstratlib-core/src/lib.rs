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
//! ## Foundational types
//!
//! Four standalone crates define the newtypes and enums every OptionStratLib
//! crate shares. Core depends on them and re-exports their types unchanged,
//! never wrapping or copying one, so a value reached through any path below
//! is the same type as the original and needs no conversion. Each crate
//! resolves to a single version across the workspace (`make check-graph`).
//!
//! | Type | Defining crate | Re-exported at |
//! | --- | --- | --- |
//! | `Positive`, `PositiveError`, `is_positive` | `positive` | `optionstratlib_core::model`, `optionstratlib::model`; `Positive` also in `optionstratlib::prelude` |
//! | `pos_or_panic!`, `spos!`, `assert_pos_relative_eq!` | `positive` | `optionstratlib_core`, `optionstratlib::prelude` |
//! | `ExpirationDate`, `ExpirationDateError` | `expiration_date` | `optionstratlib_core::model`, `optionstratlib::model`; `ExpirationDate` also at the `optionstratlib` root and in its prelude |
//! | `Side`, `OptionStyle`, `Action`, `UnderlyingAssetType` | `financial_types` | `optionstratlib_core::model::types`; `Side` and `OptionStyle` also in `optionstratlib_core::model` and at the `optionstratlib` root; `Side`, `OptionStyle` and `Action` in its prelude |
//! | `OptionType`, `OptionBasicType`, `AsianAveragingType`, `BarrierType`, `BinaryType`, `LookbackType`, `RainbowType` | `option_type` | `optionstratlib_core::model::types`; `OptionType` and `RainbowType` also in `optionstratlib_core::model` and at the `optionstratlib` root; `OptionType` in its prelude |
//! | `StrictlyPositive`, `PositiveResult`, the `positive::constants`; `ParseEnumError`; the day-count conventions (`DayCount`, `Actual360`, `Actual365Fixed`, `Thirty360US`) | `positive`; `financial_types`; `expiration_date` | not re-exported: name them through the defining crate |
//!
//! The facade's `optionstratlib::model` *is* `optionstratlib_core::model`, so
//! every core path above is also reachable with the `optionstratlib::model`
//! prefix. Enabling `schema` adds `ToSchema` impls to these types; it does not
//! change which type a path names.
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

pub use positive::{assert_pos_relative_eq, pos_or_panic, spos};

/// Version of the `optionstratlib-core` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
