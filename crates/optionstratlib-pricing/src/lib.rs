#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

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
//! Prices, premia and Greeks cross the public boundary as
//! `rust_decimal::Decimal` or `Positive`; `f64` stays inside the numerical
//! kernels.
//!
//! ## Imports
//!
//! There is no prelude, for the same reason as in core and math (#518): the
//! module roots re-export each capability, and the `optionstratlib` facade
//! prelude serves broad imports.
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

/// Version of the `optionstratlib-pricing` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
