/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/10/26
******************************************************************************/

//! The implied-volatility solver shared by
//! [`OptionPricing::calculate_implied_volatility`](crate::pricing::OptionPricing::calculate_implied_volatility),
//! `volatility::implied_volatility` and `volatility::calculate_iv` (#859 P4).

use crate::error::VolatilityError;
use crate::kernels::calculate_d_values;
use crate::pricing::OptionPricing;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_mul, d_sqrt, d_sub, decimal_to_f64, f64_to_decimal,
};
use optionstratlib_core::model::{Options, Positive, Side};
use rust_decimal::Decimal;

/// Top of the bracket every implied-volatility solve searches: 500 % per
/// year, as the bisection of `OptionPricing::calculate_implied_volatility`
/// always used.
pub(crate) const IV_BRACKET_MAX: Positive = Positive::FIVE;

/// Relative size of a volatility step at which the solver stops (#859 P4).
/// A Newton step of relative size `δ` leaves an error of order `δ²`, and a
/// bisection step one below `δ`, so stopping at `1e-13` keeps the result
/// within `1e-12` of the root without spending steps on the rounding noise
/// of the prices.
const IV_STEP_TOLERANCE: Decimal = rust_decimal_macros::dec!(0.0000000000001);

/// Ratio `high / low` above which the solver splits its bracket at the
/// geometric mean instead of the arithmetic one.
const IV_GEOMETRIC_SPLIT: Decimal = Decimal::from_parts(4, 0, 0, false, 0);

/// What a solve over `[low, high]` found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum IvOutcome {
    /// The volatility whose price is the target.
    Found(Positive),
    /// The target lies at or below the price at the bottom of the bracket.
    BelowBracket,
    /// The target lies above the price at the top of the bracket.
    AboveBracket,
    /// The iteration cap was reached; the last volatility evaluated.
    NoConvergence(Positive),
}

/// Solves `residual(σ) = 0` for the implied volatility (#859 P4).
///
/// `evaluate(σ, with_slope)` returns the residual `price(σ) - target`, which
/// increases with `σ`, and, when `with_slope`, its derivative in `σ` (the
/// vega per unit of volatility); the two bracket ends only need the sign.
/// The search is Newton's method kept inside a bracket (`rtsafe`, Press et
/// al., *Numerical Recipes*, 9.4): every evaluation narrows `[low, high]` by
/// the sign of the residual, and a Newton step that leaves the bracket, or a
/// derivative that is not positive, falls back to bisection. It starts from
/// `guess` when it lies strictly inside the bracket and stops when a step
/// moves `σ` by at most `1e-13` relative.
///
/// The three public entry points, `volatility::implied_volatility`,
/// `volatility::calculate_iv` and `OptionPricing::calculate_implied_volatility`, each run this solve on
/// their own sign convention and map the outcome to their own errors.
///
/// # Errors
///
/// Propagates whatever `evaluate` or the checked arithmetic reports.
pub(crate) fn solve_implied_volatility<F>(
    low: Positive,
    high: Positive,
    guess: Positive,
    max_iterations: u32,
    mut evaluate: F,
) -> Result<IvOutcome, VolatilityError>
where
    F: FnMut(Positive, bool) -> Result<(Decimal, Decimal), VolatilityError>,
{
    let op = "volatility::iv_solve";
    // A target the floor already reaches is reported as below the bracket:
    // every volatility under the floor prices it too, so none is implied.
    let (residual_low, _) = evaluate(low, false)?;
    if residual_low >= Decimal::ZERO {
        return Ok(IvOutcome::BelowBracket);
    }
    let (residual_high, _) = evaluate(high, false)?;
    if residual_high.is_zero() {
        return Ok(IvOutcome::Found(high));
    }
    if residual_high < Decimal::ZERO {
        return Ok(IvOutcome::AboveBracket);
    }

    let (mut low, mut high) = (low.to_dec(), high.to_dec());
    // A bracket spanning more than a factor of four is split at its
    // geometric mean: the volatility scale is logarithmic, and from the
    // initial `[1e-16, 5]` the arithmetic midpoint lands far above an
    // out-of-the-money root.
    let midpoint = |low: Decimal, high: Decimal| -> Result<Decimal, VolatilityError> {
        if high > d_mul(low, IV_GEOMETRIC_SPLIT, op)?
            && let Ok(geometric) = d_sqrt(d_mul(low, high, op)?, op)
        {
            return Ok(geometric);
        }
        Ok(d_div(d_add(low, high, op)?, Decimal::TWO, op)?)
    };
    let mut sigma = if guess.to_dec() > low && guess.to_dec() < high {
        guess.to_dec()
    } else {
        midpoint(low, high)?
    };

    for _ in 0..max_iterations {
        let current = Positive::new_decimal(sigma)?;
        let (residual, slope) = evaluate(current, true)?;
        if residual.is_zero() {
            return Ok(IvOutcome::Found(current));
        }
        if residual < Decimal::ZERO {
            low = sigma;
        } else {
            high = sigma;
        }
        let newton = if slope > Decimal::ZERO {
            d_div(residual, slope, op)
                .and_then(|step| d_sub(sigma, step, op))
                .ok()
                .filter(|next| *next > low && *next < high)
        } else {
            None
        };
        let next = match newton {
            Some(next) => next,
            None => midpoint(low, high)?,
        };
        let moved = d_sub(next, sigma, op)?.abs();
        sigma = next;
        if moved <= d_mul(IV_STEP_TOLERANCE, sigma, op)? {
            return Ok(IvOutcome::Found(Positive::new_decimal(sigma)?));
        }
    }
    Ok(IvOutcome::NoConvergence(Positive::new_decimal(sigma)?))
}

/// The Brenner and Subrahmanyam (1988) at-the-money estimate,
/// `σ ≈ sqrt(2π / T) · price / S`, the solver's first guess. Any value it
/// cannot form returns `None`, and the solver starts from the bracket's
/// midpoint instead.
pub(crate) fn iv_initial_guess(option: &Options, price: Decimal) -> Option<Positive> {
    let years = decimal_to_f64(option.time_to_expiration().ok()?.to_dec()).ok()?;
    let spot = decimal_to_f64(option.underlying_price.to_dec()).ok()?;
    let price = decimal_to_f64(price.abs()).ok()?;
    if years <= 0.0 || spot <= 0.0 {
        return None;
    }
    let guess = (2.0 * std::f64::consts::PI / years).sqrt() * price / spot; // scan-banned: allow -- f64 `sqrt` of a positive finite ratio, it does not abort; a non-finite guess is rejected by `Positive::new` below
    Positive::new(guess).ok()
}

/// The residual and its slope for one long unit of `option` at volatility
/// `sigma`: `price(σ) - target` and, when `with_slope`, the vega per unit
/// of volatility.
pub(crate) fn long_residual(
    option: &Options,
    target: Decimal,
    sigma: Positive,
    with_slope: bool,
) -> Result<(Decimal, Decimal), VolatilityError> {
    // One long unit: the residual and the slope are per unit, whatever the
    // side, quantity and contract size of `option`.
    let mut candidate = option.clone();
    candidate.side = Side::Long;
    candidate.quantity = Positive::ONE;
    candidate.contract_size = Positive::ONE;
    candidate.implied_volatility = sigma;
    let price = candidate.calculate_price_black_scholes()?;
    let residual = d_sub(price, target, "pricing::iv_residual")?;
    if !with_slope {
        return Ok((residual, Decimal::ZERO));
    }
    Ok((residual, vega_slope(&candidate)))
}

/// The Black–Scholes vega per unit of volatility, `S e^(-qT) φ(d1) √T`,
/// taken in `f64`: it only steers the Newton step, whose result the bracket
/// and the `Decimal` residual check, so its rounding does not reach the
/// answer. Anything it cannot form is `0`, which makes the solver bisect.
/// For a non-European option it is the European vega, an approximation
/// that the bracket keeps safe.
fn vega_slope(option: &Options) -> Decimal {
    let slope = || -> Option<Decimal> {
        let (d1, _) = calculate_d_values(option).ok()?;
        let d1 = decimal_to_f64(d1).ok()?;
        let spot = decimal_to_f64(option.underlying_price.to_dec()).ok()?;
        let yield_rate = decimal_to_f64(option.dividend_yield.to_dec()).ok()?;
        let years = decimal_to_f64(option.time_to_expiration().ok()?.to_dec()).ok()?;
        let vega = spot * (-yield_rate * years).exp() * (-0.5 * d1 * d1).exp() * years.sqrt() // scan-banned: allow -- f64 `exp` and `sqrt` saturate or give NaN instead of aborting, and a non-finite vega is rejected by `f64_to_decimal` below
            / (2.0 * std::f64::consts::PI).sqrt(); // scan-banned: allow -- f64 `sqrt` of a positive constant
        f64_to_decimal(vega).ok()
    };
    slope()
        .filter(|v| *v > Decimal::ZERO)
        .unwrap_or(Decimal::ZERO)
}
