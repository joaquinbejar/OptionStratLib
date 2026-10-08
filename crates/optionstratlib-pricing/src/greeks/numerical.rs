/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! Numerical Greeks implementation using finite differences.
//!
//! This module provides a fallback for calculating option Greeks when analytical
//! solutions are complex or unavailable (e.g., for exotic options like Barriers).

use crate::error::greeks::GreeksError;
use crate::pricing::{ClosedFormEngine, price_option_with};
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_mul, d_sub};
use optionstratlib_core::model::{ExpirationDate, Options};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const H: Decimal = dec!(0.01);

/// The time bump of [`numerical_theta`], in calendar days: theta is quoted
/// per day, so a one-day step differences in the quoted unit (#796).
const THETA_BUMP_DAYS: Decimal = dec!(1);

/// `2 × THETA_BUMP_DAYS`, the span of the central time difference.
const THETA_SPAN_DAYS: Decimal = dec!(2);

/// `2 × H`, the span of the central differences: the literal carries the
/// mantissa and scale `dec!(2.0) * H` produced, so no multiplication runs.
const TWO_H: Decimal = dec!(0.020);

/// `x + H` for a bumped input; an input within `H` of `Decimal::MAX`
/// aborted the raw addition (#788).
fn bump_up(x: Decimal, op: &'static str) -> Result<Decimal, GreeksError> {
    Ok(d_add(x, H, op)?)
}

/// `x - H` for a bumped input.
fn bump_down(x: Decimal, op: &'static str) -> Result<Decimal, GreeksError> {
    Ok(d_sub(x, H, op)?)
}

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
    opt_plus.underlying_price = Positive::new_decimal(
        bump_up(
            option.underlying_price.to_dec(),
            "greeks::numerical::spot_up",
        )?
        .abs(),
    )?;

    let mut opt_minus = option.clone();
    opt_minus.underlying_price = Positive::new_decimal(
        bump_down(
            option.underlying_price.to_dec(),
            "greeks::numerical::spot_down",
        )?
        .abs(),
    )?;

    let p_plus = price_option_with(&opt_plus, &ClosedFormEngine::ClosedFormBS)?;
    let p_minus = price_option_with(&opt_minus, &ClosedFormEngine::ClosedFormBS)?;

    let diff = d_sub(
        p_plus.to_dec(),
        p_minus.to_dec(),
        "greeks::numerical::delta::diff",
    )?;
    Ok(d_div(diff, TWO_H, "greeks::numerical::delta::scaled")?)
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
    opt_plus.underlying_price = Positive::new_decimal(
        bump_up(
            option.underlying_price.to_dec(),
            "greeks::numerical::spot_up",
        )?
        .abs(),
    )?;

    let mut opt_minus = option.clone();
    opt_minus.underlying_price = Positive::new_decimal(
        bump_down(
            option.underlying_price.to_dec(),
            "greeks::numerical::spot_down",
        )?
        .abs(),
    )?;

    let p_plus = price_option_with(&opt_plus, &ClosedFormEngine::ClosedFormBS)?;
    let p_minus = price_option_with(&opt_minus, &ClosedFormEngine::ClosedFormBS)?;
    let p = price_option_with(option, &ClosedFormEngine::ClosedFormBS)?;

    // Central-second-difference numerator:
    //   p_plus - 2*p + p_minus.
    // Build `2*p` via `d_mul` so an overflow on the doubled price does
    // not silently saturate before the checked `d_sub` / `d_add`.
    let two_p = d_mul(dec!(2.0), p.to_dec(), "greeks::numerical::gamma::two_p")?;
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
    opt_plus.implied_volatility = Positive::new_decimal(
        bump_up(
            option.implied_volatility.to_dec(),
            "greeks::numerical::vol_up",
        )?
        .abs(),
    )?;

    let mut opt_minus = option.clone();
    opt_minus.implied_volatility = Positive::new_decimal(
        bump_down(
            option.implied_volatility.to_dec(),
            "greeks::numerical::vol_down",
        )?
        .abs(),
    )?;

    let p_plus = price_option_with(&opt_plus, &ClosedFormEngine::ClosedFormBS)?;
    let p_minus = price_option_with(&opt_minus, &ClosedFormEngine::ClosedFormBS)?;

    let diff = d_sub(
        p_plus.to_dec(),
        p_minus.to_dec(),
        "greeks::numerical::vega::diff",
    )?;
    Ok(d_div(diff, TWO_H, "greeks::numerical::vega::scaled")?)
}

/// Calculates theta numerically using finite differences in time.
///
/// Theta measures the rate of decay of the option's value over time. Like
/// the other numerical Greeks it is a central difference on one input, here
/// the time to expiry bumped by one calendar day either way:
///
/// ```text
/// theta = (P(T - 1 day) - P(T + 1 day)) / 2
/// ```
///
/// Calendar time running forward shortens `T`, hence the order of the
/// terms. The step is one day, so the value is per day, the unit of the
/// closed-form [`crate::greeks::theta`]; its truncation error is
/// `O(h^2)` with `h = 1/365` year, and grows toward expiry: at the money
/// with `sigma = 0.25` it is about `7e-6` per day at 30 days and `2.5e-4`
/// at 7 days, so the one-day step is coarse in the last week. Within one
/// day of expiry `P(T - 1 day)` does not exist and the difference is the
/// one-sided `P(T) - P(T + 1 day)`, the mean theta over the next day, with
/// an `O(h)` error; at expiry (`T = 0`) the result is `0`, as for the
/// closed form. The bumped expiries are
/// `ExpirationDate::Days`, so an absolute-date expiry is read as its days
/// to expiry at the time of the call.
///
/// Like the other numerical Greeks, the value is that of one long unit
/// contract: the evaluator prices the absolute value, with no `Side`,
/// quantity or contract size.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be resolved, [`GreeksError::CalculationError`] when a bumped
/// expiry or the difference leaves the `Decimal` range, and propagates any
/// `PricingError` returned by the unified-pricing evaluator on the
/// perturbed option clones (wrapped as [`GreeksError::Pricing`]).
pub fn numerical_theta(option: &Options) -> Result<Decimal, GreeksError> {
    let days = option.expiration_date.get_days()?.to_dec();
    if days.is_zero() {
        return Ok(Decimal::ZERO);
    }

    let longer = with_days_to_expiry(
        option,
        d_add(days, THETA_BUMP_DAYS, "greeks::numerical::theta::days_up")?,
    )?;
    let p_longer = price_option_with(&longer, &ClosedFormEngine::ClosedFormBS)?;

    if days > THETA_BUMP_DAYS {
        let shorter = with_days_to_expiry(
            option,
            d_sub(days, THETA_BUMP_DAYS, "greeks::numerical::theta::days_down")?,
        )?;
        let p_shorter = price_option_with(&shorter, &ClosedFormEngine::ClosedFormBS)?;
        let diff = d_sub(
            p_shorter.to_dec(),
            p_longer.to_dec(),
            "greeks::numerical::theta::diff",
        )?;
        Ok(d_div(
            diff,
            THETA_SPAN_DAYS,
            "greeks::numerical::theta::scaled",
        )?)
    } else {
        let p = price_option_with(option, &ClosedFormEngine::ClosedFormBS)?;
        Ok(d_sub(
            p.to_dec(),
            p_longer.to_dec(),
            "greeks::numerical::theta::forward_diff",
        )?)
    }
}

/// A clone of `option` expiring in `days` days.
fn with_days_to_expiry(option: &Options, days: Decimal) -> Result<Options, GreeksError> {
    let mut bumped = option.clone();
    bumped.expiration_date = ExpirationDate::Days(Positive::new_decimal(days)?);
    Ok(bumped)
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
    opt_plus.risk_free_rate = bump_up(option.risk_free_rate, "greeks::numerical::rate_up")?;

    let mut opt_minus = option.clone();
    opt_minus.risk_free_rate = bump_down(option.risk_free_rate, "greeks::numerical::rate_down")?;

    let p_plus = price_option_with(&opt_plus, &ClosedFormEngine::ClosedFormBS)?;
    let p_minus = price_option_with(&opt_minus, &ClosedFormEngine::ClosedFormBS)?;

    let diff = d_sub(
        p_plus.to_dec(),
        p_minus.to_dec(),
        "greeks::numerical::rho::diff",
    )?;
    Ok(d_div(diff, TWO_H, "greeks::numerical::rho::scaled")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::model::types::{OptionStyle, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest;

    #[test]
    fn test_two_h_is_the_product_it_replaced() {
        assert_eq!(TWO_H.to_string(), (dec!(2.0) * H).to_string());
    }

    // The bumps are checked additions; `Decimal::MAX + H` rounds back to
    // `Decimal::MAX` rather than overflowing, so the extremes come back
    // as a value or an error, never as an abort.
    #[test]
    fn test_bumps_at_the_ends_of_the_range_return() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.underlying_price = Positive::MAX;
        let _ = numerical_delta(&option);
        let _ = numerical_gamma(&option);

        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.implied_volatility = Positive::MAX;
        let _ = numerical_vega(&option);

        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        for rate in [Decimal::MAX, Decimal::MIN] {
            option.risk_free_rate = rate;
            let _ = numerical_rho(&option);
        }

        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.expiration_date = ExpirationDate::Days(Positive::MAX);
        let _ = numerical_theta(&option);
    }

    #[test]
    fn test_bump_matches_the_operator() {
        let x = dec!(100.5);
        assert_eq!(bump_up(x, "test").ok(), Some(x + H));
        assert_eq!(bump_down(x, "test").ok(), Some(x - H));
    }
}
