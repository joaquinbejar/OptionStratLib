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
// checked form (#788). `clippy.toml` exempts the unary minus on `Decimal`,
// which cannot overflow. Casts that truncate, wrap or drop the sign are
// denied for the same reason. Unit tests are exempt, as above.
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

//! # optionstratlib-pricing
//!
//! Option pricing models, Greeks and volatility of OptionStratLib, on the
//! core domain types. It depends only on `optionstratlib-core`,
//! `optionstratlib-math` and general numeric crates: no option chain,
//! simulation engine, strategy, plotting, I/O or async runtime.
//!
//! - [`pricing`]: Black-Scholes, Black-76, Garman-Kohlhagen, binomial,
//!   Monte Carlo over supplied paths, telegraph, American and exotic
//!   kernels, the [`pricing::OptionPricing`] extension trait for
//!   `Options`, the unified [`pricing::price_option_with`] entry point, profit
//!   contracts and solver defaults.
//! - [`greeks`]: first and higher-order Greeks, the [`greeks::Greeks`]
//!   trait, model-specific Greeks and the Greeks of the leg types.
//! - [`volatility`]: implied-volatility solvers and volatility models and
//!   traits.
//! - [`error`]: [`error::PricingError`], [`error::GreeksError`] and
//!   [`error::VolatilityError`].
//!
//! ## Numerical boundary
//!
//! Prices, premia, strikes, rates and Greeks cross the public boundary as
//! `rust_decimal::Decimal` or `Positive`. `f64` stays inside the numerical
//! kernels (the normal distribution, lattice and finite-difference steps).
//! The only public `f64`s are error diagnostics, which `make
//! check-float-boundary` enforces (#522). Most kernels bring floats back
//! through `optionstratlib_core::model::decimal::finite_decimal`, so a
//! non-finite intermediate becomes a typed `NonFinite` error; several exotic
//! pricers still substitute zero for a failed step instead, which #639
//! tracks. The kernels perform no filesystem, network or stdout I/O and
//! install no logging subscriber.
//!
//! For fixed inputs, closed-form and lattice pricers are deterministic; an
//! `ExpirationDate::DateTime` expiry is measured from the current clock, so
//! use `ExpirationDate::Days` for reproducible results. No public function
//! draws from the thread-local RNG implicitly (#638): Monte Carlo pricing
//! (`pricing::monte_carlo_option_pricing`), the telegraph pricer
//! (`pricing::telegraph`, `OptionPricing::calculate_price_telegraph`),
//! `pricing::TelegraphProcess`, `pricing::simulate_returns` and
//! `volatility::simulate_heston_volatility` take the generator from the
//! caller. Pass a seeded one such as
//! `optionstratlib_core::utils::deterministic_rng(seed)` for reproducible
//! results, or `&mut rand::rng()` for fresh draws on every call.
//!
//! ## Imports
//!
//! There is no prelude, for the same reason as in core and math (#518): the
//! module roots re-export each capability, and the `optionstratlib` facade
//! prelude serves broad imports.
//!
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core` and `optionstratlib-math`.
//! - **Must not depend on** `optionstratlib-simulation`, `-market`, `-analytics`,
//!   `-strategies`, `-backtest` and `-visualization`; `make check-graph` enforces
//!   the layering (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::{pricing, greeks, volatility}` and the
//!   pricing errors in `optionstratlib::error`, under the facade feature
//!   `pricing`. The facade paths are the same types as the paths here; the
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
//! use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_pricing::pricing::black_scholes;
//! use rust_decimal_macros::dec;
//!
//! // Hull's worked example: S = 42, K = 40, r = 10%, sigma = 20%, T = 0.5.
//! let call = Options::new(
//!     OptionType::European,
//!     Side::Long,
//!     "XYZ".to_string(),
//!     pos_or_panic!(40.0),
//!     ExpirationDate::Days(pos_or_panic!(182.5)),
//!     pos_or_panic!(0.2),
//!     Positive::ONE,
//!     pos_or_panic!(42.0),
//!     dec!(0.10),
//!     OptionStyle::Call,
//!     Positive::ZERO,
//!     None,
//! );
//! let price = black_scholes(&call)?;
//! assert!((price - dec!(4.76)).abs() < dec!(0.01));
//! # Ok::<(), optionstratlib_pricing::error::PricingError>(())
//! ```
//!
//! ## Runnable example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-pricing`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/pricing); `make tree-example-direct-pricing`
//! asserts its resolved graph.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the pricing
//!   types and enables `optionstratlib-core/schema` and
//!   `optionstratlib-math/schema`.

/// Pricing models, the `OptionPricing` trait, profit contracts and solver
/// defaults.
pub mod pricing;

/// Greeks: equations, the `Greeks` trait, model-specific and numerical
/// Greeks, and the Greeks of the leg types.
pub mod greeks;

/// Implied-volatility solvers, volatility models and volatility traits.
pub mod volatility;

/// Errors raised by the pricing layer.
pub mod error;

// Formulas shared by `pricing` and `greeks` (normal distribution, d1/d2,
// discounting). Private: it is the neutral bottom of the crate's internal
// graph and depends on neither module (#523).
mod kernels;

/// Version of the `optionstratlib-pricing` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
