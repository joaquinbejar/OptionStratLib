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
use crate::pricing::unified::{Priceable, PricingEngine};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const H: Decimal = dec!(0.01);

/// Calculates delta numerically using finite differences.
///
/// Delta measures the rate of change of the option price with respect to
/// changes in the underlying asset's price.
pub fn numerical_delta(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() + H).abs())?;

    let mut opt_minus = option.clone();
    opt_minus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() - H).abs())?;

    let p_plus = opt_plus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;
    let p_minus = opt_minus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;

    Ok((p_plus.to_dec() - p_minus.to_dec()) / (dec!(2.0) * H))
}

/// Calculates gamma numerically using finite differences.
///
/// Gamma measures the rate of change of delta with respect to changes in the
/// underlying asset's price.
pub fn numerical_gamma(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() + H).abs())?;

    let mut opt_minus = option.clone();
    opt_minus.underlying_price =
        Positive::new_decimal((option.underlying_price.to_dec() - H).abs())?;

    let p_plus = opt_plus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;
    let p_minus = opt_minus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;
    let p = option
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;

    Ok((p_plus.to_dec() - dec!(2.0) * p.to_dec() + p_minus.to_dec()) / (H * H))
}

/// Calculates vega numerically using finite differences.
///
/// Vega measures the sensitivity of the option price to changes in the
/// underlying asset's volatility.
pub fn numerical_vega(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.implied_volatility =
        Positive::new_decimal((option.implied_volatility.to_dec() + H).abs())?;

    let mut opt_minus = option.clone();
    opt_minus.implied_volatility =
        Positive::new_decimal((option.implied_volatility.to_dec() - H).abs())?;

    let p_plus = opt_plus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;
    let p_minus = opt_minus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;

    Ok((p_plus.to_dec() - p_minus.to_dec()) / (dec!(2.0) * H))
}

/// Calculates theta numerically using finite differences.
///
/// Theta measures the rate of decay of the option's value over time.
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
pub fn numerical_rho(option: &Options) -> Result<Decimal, GreeksError> {
    let mut opt_plus = option.clone();
    opt_plus.risk_free_rate += H;

    let mut opt_minus = option.clone();
    opt_minus.risk_free_rate -= H;

    let p_plus = opt_plus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;
    let p_minus = opt_minus
        .price(&PricingEngine::ClosedFormBS)
        .map_err(|e| GreeksError::StdError(e.to_string()))?;

    Ok((p_plus.to_dec() - p_minus.to_dec()) / (dec!(2.0) * H))
}
