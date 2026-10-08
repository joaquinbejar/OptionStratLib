#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-analytics
//!
//! Strategy-neutral analytics of OptionStratLib: profit and loss, margin and
//! risk, price-probability kernels, risk-neutral densities, option-chain
//! metrics and the projections of option data onto curves and surfaces. It
//! depends on `optionstratlib-core`, `optionstratlib-math`,
//! `optionstratlib-pricing` and `optionstratlib-market`, and on no
//! strategy, simulation, backtesting or plotting code.
//!
//! - [`analytics`]: lognormal price-probability kernels
//!   ([`analytics::calculate_price_probability`]), profit ranges
//!   ([`analytics::ProfitLossRange`]), risk-neutral density and skew
//!   ([`analytics::RNDAnalysis`]) and the curve and surface projections
//!   ([`analytics::BasicCurves`], [`analytics::BasicSurfaces`],
//!   [`analytics::OptionChainProjections`]).
//! - [`pnl`]: [`pnl::PnL`], the [`pnl::PnLCalculator`] trait, transactions,
//!   delta adjustments and P&L metrics documents.
//! - [`risk`]: SPAN margin ([`risk::SPANMargin`]) and risk categories.
//! - [`metrics`]: price, risk, composite, temporal, stress and liquidity
//!   metrics over an `OptionChain`, as curves and surfaces.
//! - [`error`]: [`error::ProbabilityError`], [`error::ProjectionError`] and
//!   [`error::TransactionError`].
//!
//! The probability analysis of a concrete strategy (`ProbabilityAnalysis`)
//! builds on these kernels and lives with the strategies; nothing here names
//! a strategy type.
//!
//! ## Imports
//!
//! There is no prelude, as in the other component crates (ADR-0001 D7): the
//! module roots re-export each capability, and the `optionstratlib` facade
//! prelude serves broad imports. Bring an extension trait into scope
//! (`use optionstratlib_analytics::metrics::ImpliedVolatilityCurve;`) to
//! call its methods on an `OptionChain`.
//!
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core`, `optionstratlib-math`,
//!   `optionstratlib-pricing` and `optionstratlib-market`.
//! - **Must not depend on** `optionstratlib-simulation`, `-strategies`,
//!   `-backtest` and `-visualization`; `make check-graph` enforces the layering
//!   (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::{analytics, pnl, risk, metrics}` and the
//!   analytics errors in `optionstratlib::error`, under the facade feature
//!   `analytics`. The facade paths are the same types as the paths here; the
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
//! use optionstratlib_analytics::analytics::{VolatilityAdjustment, calculate_single_point_probability};
//! use optionstratlib_core::model::{ExpirationDate, Positive};
//! use optionstratlib_core::pos_or_panic;
//! use rust_decimal_macros::dec;
//!
//! // Probability of finishing below and above 105 in 30 days from 100.
//! let (below, above) = calculate_single_point_probability(
//!     &Positive::HUNDRED,
//!     &pos_or_panic!(105.0),
//!     VolatilityAdjustment {
//!         base_volatility: pos_or_panic!(0.2),
//!         std_dev_adjustment: Positive::ZERO,
//!     },
//!     None,
//!     &ExpirationDate::Days(pos_or_panic!(30.0)),
//!     Some(dec!(0.05)),
//! )?;
//! assert!((below.to_dec() + above.to_dec() - dec!(1)).abs() < dec!(1e-9));
//! # Ok::<(), optionstratlib_analytics::error::ProbabilityError>(())
//! ```
//!
//! ## Runnable example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-analytics`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/analytics); `make tree-example-direct-analytics`
//! asserts its resolved graph.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the analytics
//!   types and enables `schema` in core, math, pricing and market.

/// Strategy-neutral analytics: price-probability kernels, profit ranges,
/// risk-neutral densities and the projections of option data onto curves
/// and surfaces.
pub mod analytics;

/// Profit and loss: `PnL`, the `PnLCalculator` trait, transactions, delta
/// adjustments and P&L metrics documents.
pub mod pnl;

/// Margin and risk: SPAN margin and risk categories.
pub mod risk;

/// Metrics over an option chain: price, risk, composite, temporal, stress
/// and liquidity curves and surfaces.
pub mod metrics;

/// Errors raised by the analytics layer.
pub mod error;

/// Version of the `optionstratlib-analytics` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
