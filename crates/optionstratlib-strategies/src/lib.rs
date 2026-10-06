#![allow(unknown_lints)]
#![allow(clippy::literal_string_with_formatting_args)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-strategies
//!
//! Option strategies of OptionStratLib: vertical spreads, butterflies,
//! condors, straddles, strangles, covered and protective structures, custom
//! multi-leg strategies, delta neutrality and the probability analysis of a
//! strategy. It depends on `optionstratlib-core`, `optionstratlib-pricing`,
//! `optionstratlib-market` and `optionstratlib-analytics`, and on no
//! simulation, backtesting or plotting code.
//!
//! - [`strategies`]: the concrete strategies ([`strategies::BullCallSpread`],
//!   [`strategies::IronCondor`], [`strategies::custom::CustomStrategy`], …),
//!   the strategy traits ([`strategies::Strategies`],
//!   [`strategies::Strategable`], [`strategies::base::Optimizable`],
//!   [`strategies::BasicAble`], [`strategies::Validable`]), construction from
//!   a request ([`strategies::StrategyConstructor`],
//!   [`strategies::StrategyRequest`]), delta neutrality
//!   ([`strategies::DeltaNeutrality`]) and the strategy probability analysis
//!   ([`strategies::probabilities::ProbabilityAnalysis`]).
//! - [`error`]: [`error::StrategyError`] and its kinds.
//!
//! Charts of a strategy (`Graph`) and the simulation of a strategy over a
//! price path stay with the visualization and backtesting layers; the
//! `optionstratlib` facade provides them.
//!
//! ## Imports
//!
//! There is no prelude, as in the other component crates (ADR-0001 D7): the
//! module roots re-export each capability, and the `optionstratlib` facade
//! prelude serves broad imports. The optimiser side filter is a market type,
//! `optionstratlib_market::chains::utils::FindOptimalSide`.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the strategy
//!   types and enables `schema` in core, pricing, market and analytics.

/// Option strategies, the strategy traits, delta neutrality and the
/// probability analysis of a strategy.
pub mod strategies;

/// Errors raised by the strategies layer.
pub mod error;

/// Version of the `optionstratlib-strategies` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
