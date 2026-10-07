//! Errors owned by the pricing layer.
//!
//! Each failure here is raised by a pricing model, a Greek or a volatility
//! solver. Errors of the layers below (`DecimalError`, `OptionsError`,
//! `CurveError`, …) stay with `optionstratlib-core` and `optionstratlib-math`,
//! and the variants below that wrap them carry those types unchanged.
//!
//! # Canonical paths (#550)
//!
//! Every error type is exported flat from this module, for example
//! `optionstratlib_pricing::error::PricingError`. The file module `pricing`
//! is private: it held nothing the flat paths do not, and was public only
//! because 0.21 exposed it. `GreeksResult` is flat too. `greeks` stays public
//! because its detail enums (`InputErrorKind`, …) are not flattened: kind
//! names collide across crates once the facade gathers every crate's errors
//! (see `greeks`'s module docs).
//!
//! ```rust
//! use optionstratlib_pricing::error::{GreeksError, GreeksResult, PricingError, PricingResult};
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_pricing::error::pricing::PricingError;
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_pricing::error::pricing::PricingResult;
//! ```

/// ### Greeks Errors (`GreeksError`)
/// Handles:
/// * Greeks calculations
/// * Mathematical validation
/// * Input parameter validation
/// * Numerical computations
pub mod greeks;

/// ### Pricing Errors (`PricingError`)
/// Handles:
/// * Pricing method failures (Black-Scholes, Binomial, etc.)
/// * Monte Carlo simulation errors
/// * Invalid pricing engine configurations
/// * Generic pricing-related errors
mod pricing;

/// ### Volatility Errors (`VolatilityError`)
/// Handles:
/// * Implied volatility calculation failures
/// * Historical volatility estimation issues
/// * Volatility model parameter validation
mod volatility;

pub use greeks::{GreeksError, GreeksResult};
pub use pricing::{PricingError, PricingResult};
pub use volatility::VolatilityError;
