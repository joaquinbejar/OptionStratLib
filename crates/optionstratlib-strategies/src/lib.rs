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
//! ## Strategy families
//!
//! Every strategy belongs to exactly one family; the traits, construction,
//! optimisation, delta neutrality and probability analysis are shared by all
//! of them.
//!
//! | Family | Strategies |
//! | --- | --- |
//! | Single leg | `LongCall`, `LongPut`, `ShortCall`, `ShortPut` |
//! | Vertical spreads | `BullCallSpread`, `BullPutSpread`, `BearCallSpread`, `BearPutSpread` |
//! | Butterflies | `LongButterflySpread`, `ShortButterflySpread`, `CallButterfly`, `IronButterfly` |
//! | Condors | `IronCondor` |
//! | Straddles and strangles | `LongStraddle`, `ShortStraddle`, `LongStrangle`, `ShortStrangle` |
//! | Covered and protective | `CoveredCall`, `ProtectivePut`, `Collar`, `PoorMansCoveredCall` |
//! | Custom | `CustomStrategy` |
//! | Shared, always available | `base` and `shared` traits, `StrategyRequest` and `StrategyConstructor`, `combinations`, `delta_neutral`, `probabilities`, `utils` |
//!
//! There are no per-family features: the crate is one capability, as
//! ADR-0002 Decision 3 sets for 0.22, and #532 measured whether splitting it
//! would pay. It would not:
//!
//! - **Packages.** The crate adds no package to the graph of the layers below
//!   it: `cargo tree -p optionstratlib-strategies -e normal --prefix none`
//!   resolves 78 packages, `optionstratlib-analytics` 77, the difference being
//!   this crate. The one external crate only it imports, `itertools`, is
//!   already in that graph through `optionstratlib-math`, and only the
//!   shared optimiser (`combinations`) uses it.
//! - **Build time.** On rustc 1.99.0, Apple M5 Max, 18 cores, the whole crate
//!   costs 0.9 s to check, 1.7 s to build in debug and 3.0 s in release when
//!   its dependencies are already built (`CARGO_INCREMENTAL=0`, touching
//!   `src/lib.rs`, median of three), against 5.7 s for a clean check of
//!   analytics and everything below it. A family feature can save only a
//!   fraction of the crate's own time, so under a second per check.
//! - **Artifact.** The release `rlib` is 11.0 MB, about four times each lower
//!   crate's; the linker drops what a binary does not use, so a consumer of
//!   one family does not ship the others.
//! - **Cost of splitting.** `StrategyType` and `StrategyRequest` name every
//!   strategy, so gating a family would gate enum variants, which ADR-0002
//!   section 4 forbids; and seven optional families would add up to 2^7
//!   feature combinations for CI to build, lint and test.
//!
//! Reproduce with `make measure-strategies`; `tests/strategy_families.rs`
//! keeps the family table complete.
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
