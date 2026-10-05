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
