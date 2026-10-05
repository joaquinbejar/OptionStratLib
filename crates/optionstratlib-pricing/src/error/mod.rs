//! Errors owned by the pricing layer.
//!
//! Each failure here is raised by a pricing model, a Greek or a volatility
//! solver. Errors of the layers below (`DecimalError`, `OptionsError`,
//! `CurveError`, …) stay with `optionstratlib-core` and `optionstratlib-math`,
//! and the variants below that wrap them carry those types unchanged.

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
pub mod pricing;

/// ### Volatility Errors (`VolatilityError`)
/// Handles:
/// * Implied volatility calculation failures
/// * Historical volatility estimation issues
/// * Volatility model parameter validation
mod volatility;

pub use greeks::GreeksError;
pub use pricing::{PricingError, PricingResult};
pub use volatility::VolatilityError;
