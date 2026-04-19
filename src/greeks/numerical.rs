/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! Numerical Greeks implementation using finite differences.
//!
//! This module provides a fallback for calculating option Greeks when analytical
//! solutions are complex or unavailable (e.g., for exotic options like Barriers).

use crate::Options;
use crate::error::greeks::GreeksError;
use crate::model::decimal::{d_add, d_div, d_mul, d_sub};
use crate::pricing::unified::{Priceable, PricingEngine};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const H: Decimal = dec!(0.01);

/// Calculates delta numerically using finite differences.
///
/// Delta measures the rate of change of the option price with respect to
/// changes in the underlying asset's price.
///
/// # Errors
///
/// Propagates any `PricingError` returned by the unified-pricing
/// evaluator on the perturbed option clones, wrapped as
/// [`GreeksError::Pricing`]; typically
/// `PricingError::ExpirationDate` or `PricingError::MethodError` on
/// numerical failure.
pub fn numerical_delta(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() + H).abs())?;

    let mut opt_minus = option.clone();
    opt_minus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() - H).abs())?;

    let p_plus = opt_plus.price(&PricingEngine::ClosedFormBS)?;
    let p_minus = opt_minus.price(&PricingEngine::ClosedFormBS)?;

    let diff = d_sub(
        p_plus.to_dec(),
        p_minus.to_dec(),
        "greeks::numerical::delta::diff",
    )?;
    Ok(d_div(
        diff,
        dec!(2.0) * H,
        "greeks::numerical::delta::scaled",
    )?)
}

/// Calculates gamma numerically using finite differences.
///
/// Gamma measures the rate of change of delta with respect to changes in the
/// underlying asset's price.
///
/// # Errors
///
/// Propagates any `PricingError` returned by the unified-pricing
/// evaluator on the three perturbed option clones, wrapped as
/// [`GreeksError::Pricing`].
pub fn numerical_gamma(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() + H).abs())?;

    let mut opt_minus = option.clone();
    opt_minus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() - H).abs())?;

    let p_plus = opt_plus.price(&PricingEngine::ClosedFormBS)?;
    let p_minus = opt_minus.price(&PricingEngine::ClosedFormBS)?;
    let p = option.price(&PricingEngine::ClosedFormBS)?;

    // Central-second-difference numerator:
    //   p_plus - 2*p + p_minus.
    // Build `2*p` via `d_mul` so an overflow on the doubled price does
    // not silently saturate before the checked `d_sub` / `d_add`.
    let two_p = d_mul(
        dec!(2.0),
        p.to_dec(),
        "greeks::numerical::gamma::two_p",
    )?;
    let step = d_sub(p_plus.to_dec(), two_p, "greeks::numerical::gamma::step")?;
    let numer = d_add(step, p_minus.to_dec(), "greeks::numerical::gamma::numer")?;
    let h_squared = d_mul(H, H, "greeks::numerical::gamma::h_squared")?;
    Ok(d_div(numer, h_squared, "greeks::numerical::gamma::scaled")?)
}

/// Calculates vega numerically using finite differences.
///
/// Vega measures the sensitivity of the option price to changes in the
/// underlying asset's volatility.
///
/// # Errors
///
/// Propagates any `PricingError` returned by the unified-pricing
/// evaluator on the perturbed option clones, wrapped as
/// [`GreeksError::Pricing`].
pub fn numerical_vega(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.implied_volatility =
        Positive::new_decimal((option.implied_volatility.to_dec() + H).abs())?;

    let mut opt_minus = option.clone();
    opt_minus.implied_volatility =
        Positive::new_decimal((option.implied_volatility.to_dec() - H).abs())?;

    let p_plus = opt_plus.price(&PricingEngine::ClosedFormBS)?;
    let p_minus = opt_minus.price(&PricingEngine::ClosedFormBS)?;

    let diff = d_sub(
        p_plus.to_dec(),
        p_minus.to_dec(),
        "greeks::numerical::vega::diff",
    )?;
    Ok(d_div(
        diff,
        dec!(2.0) * H,
        "greeks::numerical::vega::scaled",
    )?)
}

/// Calculates theta numerically using finite differences.
///
/// Theta measures the rate of decay of the option's value over time.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be resolved, and propagates any `PricingError` returned by
/// the unified-pricing evaluator on the perturbed option clones
/// (wrapped as [`GreeksError::Pricing`]).
pub fn numerical_theta(option: &Options) -> Result<Decimal, GreeksError> {
    let t = option.expiration_date.get_years()?;
    if t < H {
        return Ok(Decimal::ZERO);
    }

    let _opt_plus = option.clone();
    // ExpirationDate doesn't have a direct setter for years, but we can use Days for now if we assume 365 days/year
    // Actually, we can't easily mutate ExpirationDate to subtract a small delta in years.
    // Let's Skip numerical theta for now or implement it carefully.
    // For now, return error or 0.
    Err(GreeksError::CalculationError(crate::error::greeks::CalculationErrorKind::ThetaError { reason: "Numerical theta not yet implemented for exotics due to ExpirationDate mutation complexity".to_string() }))
}

/// Calculates rho numerically using finite differences.
///
/// Rho measures the sensitivity of the option price to changes in the
/// risk-free interest rate.
///
/// # Errors
///
/// Propagates any `PricingError` returned by the unified-pricing
/// evaluator on the perturbed option clones, wrapped as
/// [`GreeksError::Pricing`].
pub fn numerical_rho(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.risk_free_rate += H;

    let mut opt_minus = option.clone();
    opt_minus.risk_free_rate -= H;

    let p_plus = opt_plus.price(&PricingEngine::ClosedFormBS)?;
    let p_minus = opt_minus.price(&PricingEngine::ClosedFormBS)?;

    let diff = d_sub(
        p_plus.to_dec(),
        p_minus.to_dec(),
        "greeks::numerical::rho::diff",
    )?;
    Ok(d_div(
        diff,
        dec!(2.0) * H,
        "greeks::numerical::rho::scaled",
    )?)
}
