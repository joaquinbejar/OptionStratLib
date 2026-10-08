#![allow(unknown_lints)]
#![allow(clippy::literal_string_with_formatting_args)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]
// Per rules/global_rules.md §Arithmetic, an operator that overflows, divides
// by zero or breaks the `Positive` invariant aborts the caller, so every
// operation on `Decimal`, `Positive` and the integers goes through its
// checked form (#788, #808). `clippy.toml` exempts the unary minus on
// `Decimal`, which cannot overflow. Casts that truncate, wrap or drop the
// sign are denied for the same reason. Unit tests are exempt, as above.
#![deny(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
#![cfg_attr(
    test,
    allow(
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )
)]
// An error or a default built eagerly in `ok_or` / `unwrap_or` is paid on
// the success path too (#857), so it is built in a closure.
#![deny(clippy::or_fun_call)]

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
//! price path stay with the visualization and backtesting layers
//! (`optionstratlib-visualization`, `optionstratlib-backtest`); the
//! `optionstratlib` facade re-exports them.
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
//! Every strategy belongs to exactly one family. The family traits in
//! `shared` belong to their family (`SpreadStrategy` to vertical spreads,
//! `ButterflyStrategy` to butterflies, `CondorStrategy` to condors,
//! `StraddleStrategy` and `StrangleStrategy` to straddles and strangles);
//! everything else is shared by all families.
//!
//! | Family | Strategies |
//! | --- | --- |
//! | Single leg | `LongCall`, `LongPut`, `ShortCall`, `ShortPut` |
//! | Vertical spreads | `BullCallSpread`, `BullPutSpread`, `BearCallSpread`, `BearPutSpread` |
//! | Ladders | `BullCallLadder` |
//! | Butterflies | `LongButterflySpread`, `ShortButterflySpread`, `IronButterfly` |
//! | Condors | `IronCondor` |
//! | Straddles and strangles | `LongStraddle`, `ShortStraddle`, `LongStrangle`, `ShortStrangle` |
//! | Covered and protective | `CoveredCall`, `ProtectivePut`, `Collar`, `PoorMansCoveredCall` |
//! | Custom | `CustomStrategy` |
//! | Shared, always available | `base`, `default`, `model_impls`, `utils`, `build` (`StrategyRequest`, `StrategyConstructor`), `combinations`, `delta_neutral`, `probabilities`, and `error` |
//!
//! There are no per-family features: the crate is one capability, as
//! ADR-0002 Decision 3 sets for 0.22. #532 measured whether splitting it
//! would pay, and it would not.
//!
//! - **Packages.** The crate adds no package to the graph of the layers
//!   below it: `cargo tree -p optionstratlib-strategies -e normal` resolves
//!   78 packages and `optionstratlib-analytics` 77, the difference being this
//!   crate, with default and with all features. The two external crates only
//!   it imports, `itertools` and `rayon`, are used only by the shared
//!   optimiser (`combinations`) and are already in that graph through math
//!   and pricing.
//! - **Build time.** Because no family brings a package of its own, any
//!   grouping still builds everything below analytics plus the shared code,
//!   so the crate's own build time bounds what every candidate grouping can
//!   save. With its dependencies built (`CARGO_INCREMENTAL=0`, touching
//!   `src/lib.rs`), the whole crate takes a median 0.93 s to check, 1.86 s to
//!   build in debug and 3.12 s in release, against 5.85 s for a clean check of
//!   analytics and everything below it: at most about 14% of a clean check,
//!   less than a second per check. Measured on commit 21d71751 with rustc
//!   1.99.0, Apple M5 Max, 18 cores, three runs each. The crate-own time is a
//!   metric added on top of the M0-01 method (doc/BASELINE.md), which times
//!   the whole graph.
//! - **Artifact.** The release `rlib` is 11.0 MB, about four times each lower
//!   crate's; the linker drops what a binary does not use, so a consumer of
//!   one family does not ship the others.
//! - **Cost of splitting.** `StrategyType` and `StrategyRequest` name every
//!   strategy. Gating a family would either gate enum variants, which
//!   ADR-0002 section 4 forbids, or leave variants that `StrategyRequest`
//!   builds only when a feature is on, a failure that depends on features at
//!   run time. Family features would also have to be forwarded by the facade
//!   and by `optionstratlib-visualization`, whose per-strategy `Graph` impls
//!   name every strategy, and each would add a CI cell (alone, with `schema`,
//!   and in the facade).
//!
//! The adopted state adds no CI cell and has no unsupported feature
//! combination: `schema` is the crate's only feature, and none, default and
//! all features are built and tested. Reproduce the numbers with
//! `make measure-strategies`; `tests/strategy_families.rs` keeps the family
//! assignment of `StrategyType` exhaustive.
//!
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core`, `optionstratlib-pricing`,
//!   `optionstratlib-market` and `optionstratlib-analytics`.
//! - **Must not depend on** `optionstratlib-simulation`, `-backtest` and
//!   `-visualization`; `make check-graph` enforces the layering (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::strategies` and `StrategyError` /
//!   `StrategyResult` / `AdjustmentError` in `optionstratlib::error`, under the
//!   facade feature `strategies`. The facade paths are the same types as the
//!   paths here; the [ownership
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
//! use optionstratlib_core::model::{ExpirationDate, Positive};
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_strategies::strategies::base::BreakEvenable;
//! use optionstratlib_strategies::strategies::{BullCallSpread, Validable};
//! use rust_decimal_macros::dec;
//!
//! // Long the 95 call for 6.50, short the 105 call for 1.50: a debit of 5.
//! let spread = BullCallSpread::new(
//!     "XYZ".to_string(),
//!     Positive::HUNDRED,
//!     pos_or_panic!(95.0),
//!     pos_or_panic!(105.0),
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.2),
//!     dec!(0.05),
//!     Positive::ZERO,
//!     Positive::ONE,
//!     pos_or_panic!(6.5),
//!     pos_or_panic!(1.5),
//!     Positive::ZERO,
//!     Positive::ZERO,
//!     Positive::ZERO,
//!     Positive::ZERO,
//! )?;
//! assert!(spread.validate());
//! assert_eq!(spread.get_break_even_points()?, &vec![pos_or_panic!(100.0)]);
//! # Ok::<(), optionstratlib_strategies::error::StrategyError>(())
//! ```
//!
//! ## Runnable example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-strategies`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/strategies); `make tree-example-direct-strategies`
//! asserts its resolved graph.
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

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
