/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341. The existing
// call sites are internal to this file and audited for invariant-bound
// indices (fixed-length buffers, just-pushed slices, etc.).
#![allow(clippy::indexing_slicing)]

//! Cliquet option pricing module.
//!
//! Cliquet options (also known as ratchet options) consist of a series of
//! forward-starting options. At each reset date, the strike is reset to the
//! current underlying price, locking in gains.
//!
//! # Payoff
//!
//! Total Payoff = Σ max(min(R_i, cap), floor)
//! where R_i = (S_t_i / S_t_{i-1}) - 1 is the return in period i.
//!
//! # Pricing
//!
//! Uses a series of forward-starting options. Each period's contribution is:
//! V_i = S_0 * e^{-q*t_{i-1}} * [ floor * e^{-r*Δt} + Call(S=1, K=1+floor, T=Δt) - Call(S=1, K=1+cap, T=Δt) ]
//! assuming S=1 at the start of each period effectively.

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_sqrt, d_sub, finite_decimal};
use crate::model::types::OptionType;
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Prices a Cliquet option using an analytical approach (sum of forward-starting options).
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Cliquet`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when `option` is not an
///   [`OptionType::Cliquet`] variant, when `reset_dates` contains a `NaN`, or
///   when the expiration cannot be converted to a year fraction.
/// - [`PricingError::NonFinite`] when a reset offset is not representable as
///   a `Decimal`.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the per-period discount factors, the
///   capped/floored strikes, the unit-call `d1` / `d2`, or the running total.
pub fn cliquet_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Cliquet { reset_dates } => {
            let resets: Vec<f64> = reset_dates.iter().map(Positive::to_f64).collect();
            price_cliquet(option, &resets)
        }
        _ => Err(PricingError::other(
            "cliquet_black_scholes requires OptionType::Cliquet",
        )),
    }
}

fn price_cliquet(option: &Options, reset_dates: &[f64]) -> Result<Decimal, PricingError> {
    // Retrieve caps/floors from exotic_params
    let (local_cap, local_floor) = if let Some(ref params) = option.exotic_params {
        (
            params.cliquet_local_cap.unwrap_or(dec!(0.1)), // Default 10% cap
            params.cliquet_local_floor.unwrap_or(dec!(0.0)), // Default 0% floor
        )
    } else {
        (dec!(0.1), dec!(0.0))
    };

    // Sort reset dates and ensure they are positive and before expiration.
    // Reject NaN reset dates explicitly (partial_cmp returns None for NaN).
    if reset_dates.iter().any(|d| d.is_nan()) {
        return Err(PricingError::method_error(
            "cliquet_black_scholes",
            "reset_dates contains NaN",
        ));
    }
    let mut dates = reset_dates.to_vec();
    dates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Total expiration in years
    let t_total = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    // Convert reset dates from days to years
    let t_total_f = t_total.to_f64();
    let mut reset_times_years = vec![0.0]; // Start at t=0
    for &d in &dates {
        let t = d / 365.0;
        if t > 0.0 && t < t_total_f {
            reset_times_years.push(t);
        }
    }
    reset_times_years.push(t_total_f);
    reset_times_years.dedup();

    let mut total_price = dec!(0.0);

    for window in reset_times_years.windows(2) {
        let [t_prev, t_curr] = window else { continue };
        let dt = t_curr - t_prev;

        let delta_price = price_period(option, *t_prev, dt, local_cap, local_floor)?;
        total_price = d_add(total_price, delta_price, "pricing::cliquet::total")?;
    }

    // Apply global caps/floors if present
    if let Some(ref params) = option.exotic_params {
        if let Some(g_cap) = params.cliquet_global_cap {
            total_price = total_price.min(g_cap);
        }
        if let Some(g_floor) = params.cliquet_global_floor {
            total_price = total_price.max(g_floor);
        }
    }

    Ok(apply_side(total_price, option))
}

/// Prices a single period of a cliquet option.
/// Payoff at t_curr = S_{t_prev} * max(min(S_{t_curr}/S_{t_prev} - 1, cap), floor)
fn price_period(
    option: &Options,
    t_start: f64,
    dt: f64,
    cap: Decimal,
    floor: Decimal,
) -> Result<Decimal, PricingError> {
    if dt <= 0.0 {
        return Ok(dec!(0.0));
    }

    let s0 = option.underlying_price.to_dec();
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility.to_dec();

    let dt_dec = finite_decimal(dt)
        .ok_or_else(|| PricingError::non_finite("pricing::cliquet::price_period::dt", dt))?;
    let t_start_dec = finite_decimal(t_start).ok_or_else(|| {
        PricingError::non_finite("pricing::cliquet::price_period::t_start", t_start)
    })?;

    // S_0 * e^(-q * t_start) is the present value of the expected S_{t_prev}
    let s_prev_pv = d_mul(
        s0,
        d_exp(
            d_mul(-q, t_start_dec, "pricing::cliquet::period::neg_q_t_start")?,
            "pricing::cliquet::period::dividend_discount",
        )?,
        "pricing::cliquet::period::s_prev_pv",
    )?;

    // The component at t_start: E[S_{t_prev}] = S_0 * e^{(r-q)*t_start}
    // But we need value at t=0 of S_{t_prev} * Payoff(Δt)
    // Value = e^{-r*t_start} * E[S_{t_prev} * e^{-r*Δt} * E[Payoff(Δt) | S_{t_prev}]]
    // = e^{-r*t_start} * E[S_{t_prev} * Value_at_t_prev(Payoff)]
    // = e^{-r*t_start} * E[S_{t_prev} * S_{t_prev} * [analytical_part]] ? No.

    // Correct logic:
    // Payoff at t_curr is S_{t_prev} * [ floor + max(R_i - floor, 0) - max(R_i - cap, 0) ]
    // = S_{t_prev} * [ floor + max(S_{t_curr}/S_{t_prev} - (1+floor), 0) - max(S_{t_curr}/S_{t_prev} - (1+cap), 0) ]

    // Value at t_prev:
    // V_{t_prev} = S_{t_prev} * [ floor * e^{-r*Δt} + Call(S=1, K=1+floor, Δt) - Call(S=1, K=1+cap, Δt) ]

    // Value at t=0:
    // V_0 = e^{-r*t_start} * E[V_{t_prev}]
    // = e^{-r*t_start} * [ floor * e^{-r*Δt} + Call(S=1, K=1+floor, Δt) - Call(S=1, K=1+cap, Δt) ] * E[S_{t_prev}]
    // Since E[S_{t_prev}] = S_0 * e^{(r-q)*t_start}
    // V_0 = S_0 * e^{-q*t_start} * [ floor * e^{-r*Δt} + Call(S=1, K=1+floor, Δt) - Call(S=1, K=1+cap, Δt) ]

    let call_f = call_price_on_unit(
        r,
        q,
        sigma,
        dt_dec,
        d_add(dec!(1.0), floor, "pricing::cliquet::period::floor_strike")?,
    )?;
    let call_c = call_price_on_unit(
        r,
        q,
        sigma,
        dt_dec,
        d_add(dec!(1.0), cap, "pricing::cliquet::period::cap_strike")?,
    )?;

    let floor_part = d_mul(
        floor,
        d_exp(
            d_mul(-r, dt_dec, "pricing::cliquet::period::neg_r_dt")?,
            "pricing::cliquet::period::discount",
        )?,
        "pricing::cliquet::period::floor_part",
    )?;

    let step = d_add(
        floor_part,
        call_f,
        "pricing::cliquet::period::floor_plus_call_f",
    )?;
    let period_val_at_t_prev = d_sub(step, call_c, "pricing::cliquet::period::value")?;

    d_mul(
        s_prev_pv,
        period_val_at_t_prev,
        "pricing::cliquet::period::price",
    )
    .map_err(PricingError::from)
}

/// Black-Scholes call price for S=1, K=k, T=t
fn call_price_on_unit(
    r: Decimal,
    q: Decimal,
    sigma: Decimal,
    t: Decimal,
    k: Decimal,
) -> Result<Decimal, PricingError> {
    let dividend_discount = d_exp(
        d_mul(-q, t, "pricing::cliquet::unit_call::neg_qt")?,
        "pricing::cliquet::unit_call::dividend_discount",
    )?;
    let discount = d_exp(
        d_mul(-r, t, "pricing::cliquet::unit_call::neg_rt")?,
        "pricing::cliquet::unit_call::discount",
    )?;

    if k <= dec!(0.0) {
        // If K <= 0, the call is always in the money.
        // Value = S*e^{-qT} - K*e^{-rT}
        let k_pv = d_mul(k, discount, "pricing::cliquet::unit_call::itm::k_pv")?;
        return d_sub(
            dividend_discount,
            k_pv,
            "pricing::cliquet::unit_call::itm::price",
        )
        .map_err(PricingError::from);
    }

    let b = d_sub(r, q, "pricing::cliquet::unit_call::carry")?;
    let sqrt_t = d_sqrt(t, "pricing::cliquet::unit_call::sqrt_t")?;
    let denominator = d_mul(sigma, sqrt_t, "pricing::cliquet::unit_call::denominator")?;

    // Zero volatility, zero maturity, or a `σ√t` that underflowed below the
    // representable scale: the unit forward is deterministic and the call is
    // worth its discounted intrinsic.
    if sigma == dec!(0.0) || t == dec!(0.0) || denominator.is_zero() {
        let forward = d_exp(
            d_mul(b, t, "pricing::cliquet::unit_call::zero_vol::carry_t")?,
            "pricing::cliquet::unit_call::zero_vol::forward",
        )?;
        let intrinsic = d_sub(
            forward,
            k,
            "pricing::cliquet::unit_call::zero_vol::intrinsic",
        )?
        .max(dec!(0.0));
        return d_mul(
            intrinsic,
            discount,
            "pricing::cliquet::unit_call::zero_vol::discounted",
        )
        .map_err(PricingError::from);
    }

    let inv_k = d_div(Decimal::ONE, k, "pricing::cliquet::unit_call::inv_k")?;
    if inv_k.is_zero() {
        // `1 / K` underflowed below the smallest representable `Decimal`, so
        // both normal arguments diverge to `-∞` and the call is worthless.
        return Ok(Decimal::ZERO);
    }
    let d1 = d_div(
        d_add(
            d_ln(inv_k, "pricing::cliquet::unit_call::log_inv_k")?,
            d_mul(
                d_add(
                    b,
                    d_div(
                        d_mul(sigma, sigma, "pricing::cliquet::unit_call::variance")?,
                        dec!(2.0),
                        "pricing::cliquet::unit_call::half_variance",
                    )?,
                    "pricing::cliquet::unit_call::drift_rate",
                )?,
                t,
                "pricing::cliquet::unit_call::drift",
            )?,
            "pricing::cliquet::unit_call::d1_numerator",
        )?,
        denominator,
        "pricing::cliquet::unit_call::d1",
    )?;
    let d2 = d_sub(d1, denominator, "pricing::cliquet::unit_call::d2")?;

    let n1 = big_n(d1).unwrap_or(dec!(0.0));
    let n2 = big_n(d2).unwrap_or(dec!(0.0));

    // `s_leg` is a unit-forward so its monetary boundary starts at `exp(-qt)`.
    let s_leg = d_mul(dividend_discount, n1, "pricing::cliquet::unit_call::s_leg")?;
    let discounted_k = d_mul(k, discount, "pricing::cliquet::unit_call::discounted_k")?;
    let k_leg = d_mul(discounted_k, n2, "pricing::cliquet::unit_call::k_leg")?;
    d_sub(s_leg, k_leg, "pricing::cliquet::unit_call::price").map_err(PricingError::from)
}

fn apply_side(price: Decimal, option: &Options) -> Decimal {
    match option.side {
        crate::model::types::Side::Long => price,
        crate::model::types::Side::Short => -price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::option::ExoticParams;
    use crate::model::types::{OptionStyle, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_cliquet_option() -> Options {
        Options::new(
            OptionType::Cliquet {
                reset_dates: vec![pos_or_panic!(90.0), pos_or_panic!(180.0)],
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(270.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            Some(ExoticParams {
                spot_prices: None,
                spot_min: None,
                spot_max: None,
                cliquet_local_cap: Some(dec!(0.05)),
                cliquet_local_floor: Some(dec!(0.0)),
                cliquet_global_cap: None,
                cliquet_global_floor: None,
                rainbow_second_asset_price: None,
                rainbow_second_asset_volatility: None,
                rainbow_second_asset_dividend: None,
                rainbow_correlation: None,
                spread_second_asset_volatility: None,
                spread_second_asset_dividend: None,
                spread_correlation: None,
                quanto_fx_volatility: None,
                quanto_fx_correlation: None,
                quanto_foreign_rate: None,
                exchange_second_asset_volatility: None,
                exchange_second_asset_dividend: None,
                exchange_correlation: None,
            }),
        )
    }

    #[test]
    fn test_cliquet_pricing() {
        let option = create_cliquet_option();
        let price = cliquet_black_scholes(&option).unwrap();
        assert!(price > dec!(0.0));
    }

    #[test]
    fn test_cliquet_zero_vol() {
        let mut option = create_cliquet_option();
        option.implied_volatility = Positive::ZERO;
        let price = cliquet_black_scholes(&option).unwrap();
        // With r=0.05, q=0, each period return is (exp(0.05*dt) - 1)
        // dt = 90/365 approx 0.246. exp(0.05*0.246) - 1 approx 0.0123.
        // 0.0123 is between 0 and 0.05 cap, so each period pays approx 0.0123 * S_prev_PV
        assert!(price > dec!(0.0));
    }

    #[test]
    fn test_cliquet_high_cap() {
        let mut option = create_cliquet_option();
        if let Some(ref mut params) = option.exotic_params {
            params.cliquet_local_cap = Some(dec!(1.0)); // Very high cap
        }
        let price_high = cliquet_black_scholes(&option).unwrap();

        if let Some(ref mut params) = option.exotic_params {
            params.cliquet_local_cap = Some(dec!(0.01)); // Very low cap
        }
        let price_low = cliquet_black_scholes(&option).unwrap();

        assert!(price_high > price_low);
    }
}
