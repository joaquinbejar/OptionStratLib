/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! Asian option pricing module.
//!
//! Asian options are path-dependent options where the payoff depends on the
//! average price of the underlying asset over a specified period. This module
//! implements pricing for both geometric and arithmetic averaging.
//!
//! # Averaging Types
//!
//! - **Geometric Average**: Uses geometric mean of prices. Has a closed-form
//!   Black-Scholes solution with adjusted volatility and drift.
//! - **Arithmetic Average**: Uses arithmetic mean of prices. No closed-form
//!   solution exists; uses Turnbull-Wakeman approximation.
//!
//! # Formula Sources
//!
//! - Kemna & Vorst (1990) for geometric average Asian options
//! - Turnbull & Wakeman (1991) for arithmetic average approximation

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::decimal::{d_add, d_div, d_exp, d_ln, d_mul, d_powd, d_sqrt, d_sub};
use crate::model::types::{AsianAveragingType, OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Terms kept in the small-argument series for the scaled second moment.
///
/// Six carry `g(x, z)` through `(x, z)^5` and truncate at `2 h_6 / 8!`, a
/// relative `3.5e-4 u^6` at `u = max(|x|, |z|)`.
const M2_SERIES_TERMS: u32 = 6;

/// `max(|bT|, |cT|)` below which the scaled second moment is summed as a
/// series rather than evaluated in closed form.
///
/// Measured, not chosen. Both forms were run against 50-digit `mpmath`
/// quadrature on `E[A²] = (2 S² / T²) ∫₀^T e^{bu} ∫₀^u e^{at} dt du` at
/// `S = 100`, `σ = 20%`, `b = 4%`, twelve maturities per decade. Relative
/// error of the second moment:
///
/// | `u` | 6-term series | divided difference |
/// |---|---|---|
/// | `3.1e-3` | `7.0e-20` | `2.8e-24` |
/// | `2.1e-3` | `7.0e-21` | `5.4e-23` |
/// | `1.5e-3` | `7.0e-22` | `2.5e-22` |
/// | `9.9e-4` | `7.0e-23` | `6.2e-22` |
/// | `8.2e-4` | `2.2e-23` | `6.9e-22` |
///
/// The series falls as `u^6`; the closed form rises as `u^-2`, the `1 / y`
/// amplification of the rounding `e^w` leaves inside `φ(z) - φ(x)`, which
/// reaches a relative `4e-8` by `u = 1.2e-10`. They cross at `u ≈ 1.2e-3` at
/// a relative `3e-22`, and this is that crossing to the decade.
const M2_SERIES_THRESHOLD: Decimal = dec!(1e-3);

/// `|aT|` below which the divided difference collapses onto the derivative
/// at the midpoint.
///
/// Measured the same way, sweeping `a` through zero at `b = a - σ²`,
/// `σ = 20%`, `T = 1` — the `b = -σ²` boundary contract itself:
///
/// | `|y|` | midpoint `2 φ'(m)` | divided difference |
/// |---|---|---|
/// | `1e-6` | `2.1e-14` | `6.4e-16` |
/// | `1e-7` | `2.1e-16` | `6.4e-17` |
/// | `1e-8` | `2.1e-18` | `7.0e-18` |
/// | `1e-9` | `2.1e-20` | `4.5e-18` |
/// | `1e-10` | `2.1e-22` | `1.6e-17` |
///
/// The midpoint truncation falls as `y² / 48`. The divided difference falls
/// with `y` too — the `e^w` error at `x` and at `z` nearly cancels once the
/// two arguments coincide — but only until the `φ(z) - φ(x)` rounding floor
/// turns it round near `1e-9`. They cross just under `3e-8` at a relative
/// `2e-17`; this is that crossing to the decade.
const M2_MIDPOINT_THRESHOLD: Decimal = dec!(1e-8);

/// Prices an Asian option using the appropriate method based on averaging type.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Asian`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when the option type is not
///   `OptionType::Asian`, when the averaging type is an unsupported
///   `#[non_exhaustive]` variant, or when the expiration cannot be converted
///   to a year fraction.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range: the discount factor, the forward, either
///   Turnbull-Wakeman moment, the moment-matched variance, or the final
///   Black legs. The three removable singularities of the Turnbull-Wakeman
///   second moment, `b = 0`, `b = -σ²` and `b = -σ²/2`, are evaluated at
///   their limits and never raise.
/// - [`PricingError::Positive`] when the `σ / √3` geometric adjustment is not
///   representable as a `Positive`.
pub fn asian_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Asian { averaging_type } => match averaging_type {
            AsianAveragingType::Geometric => geometric_asian_price(option),
            AsianAveragingType::Arithmetic => arithmetic_asian_price(option),
            // `AsianAveragingType` is `#[non_exhaustive]`.
            _ => Err(PricingError::other(
                "asian_black_scholes: unsupported AsianAveragingType",
            )),
        },
        _ => Err(PricingError::other(
            "asian_black_scholes requires OptionType::Asian",
        )),
    }
}

/// Prices a geometric average Asian option using closed-form Black-Scholes.
///
/// Uses the Kemna-Vorst (1990) closed-form solution. The geometric average
/// of a lognormal process is also lognormal, allowing for an analytical solution.
///
/// # Adjustments
///
/// For geometric averaging:
/// - Adjusted volatility: `σ_adj = σ / √3`
/// - Adjusted cost-of-carry: `b_adj = (r - q - σ²/6) / 2`
fn geometric_asian_price(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        return intrinsic_value(option);
    }

    if sigma == Positive::ZERO {
        // Deterministic path, but the payoff is still on an *average*. The
        // geometric mean of `S e^{b t}` over the window is
        //
        //     exp((1 / T) ∫₀^T (ln S + b t) dt) = S e^{b T / 2},
        //
        // the forward carried for half the window, not the terminal forward
        // `S e^{b T}`. That is also the `σ → 0` value of the Kemna-Vorst legs
        // below, whose adjusted carry `b_adj = (b - σ² / 6) / 2` tends to
        // `b / 2` while `σ_adj = σ / √3` tends to zero. The two agree at
        // `b = 0`, where both collapse to `S`, and differ for every other
        // carry.
        let t_dec = t.to_dec();
        let discount = d_exp(
            d_mul(-r, t_dec, "pricing::asian::geometric::det::neg_rt")?,
            "pricing::asian::geometric::det::discount",
        )?;
        let carry = d_sub(r, q, "pricing::asian::geometric::det::carry")?;
        let average = d_mul(
            s.to_dec(),
            d_exp(
                d_div(
                    d_mul(carry, t_dec, "pricing::asian::geometric::det::carry_t")?,
                    dec!(2),
                    "pricing::asian::geometric::det::half_carry_t",
                )?,
                "pricing::asian::geometric::det::growth",
            )?,
            "pricing::asian::geometric::det::average",
        )?;
        return deterministic_price(average, k.to_dec(), discount, option);
    }

    // Geometric average adjustments (Kemna-Vorst)
    let sigma_sq = d_mul(
        sigma.to_dec(),
        sigma.to_dec(),
        "pricing::asian::geometric::sigma_sq",
    )?;
    let sqrt_three = Positive::new(3.0_f64.sqrt())
        .map_err(|e| PricingError::method_error("geometric_asian_price", &e.to_string()))?;
    let sigma_adj = Positive::new_decimal(d_div(
        sigma.to_dec(),
        sqrt_three.to_dec(),
        "pricing::asian::geometric::sigma_adj",
    )?)?;
    let b_adj = d_div(
        d_sub(
            d_sub(r, q, "pricing::asian::geometric::carry")?,
            d_div(
                sigma_sq,
                dec!(6),
                "pricing::asian::geometric::variance_drag",
            )?,
            "pricing::asian::geometric::b_numerator",
        )?,
        dec!(2),
        "pricing::asian::geometric::b_adj",
    )?;

    // Calculate d1 and d2 with adjusted parameters
    let d1_val = d1(s, k, b_adj, t, sigma_adj)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;
    let d2_val = d2(s, k, b_adj, t, sigma_adj)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let t_dec = t.to_dec();
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::asian::geometric::neg_rt")?,
        "pricing::asian::geometric::discount",
    )?;
    // e^((b_adj - r) T): the geometric-average carry replaces the spot drift.
    let carry_discount = d_exp(
        d_mul(
            d_sub(b_adj, r, "pricing::asian::geometric::carry_spread")?,
            t_dec,
            "pricing::asian::geometric::carry_spread_t",
        )?,
        "pricing::asian::geometric::carry_discount",
    )?;
    let s_leg = d_mul(
        s.to_dec(),
        carry_discount,
        "pricing::asian::geometric::spot_leg",
    )?;
    let k_leg = d_mul(
        k.to_dec(),
        discount,
        "pricing::asian::geometric::strike_leg",
    )?;

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            d_sub(
                d_mul(s_leg, n_d1, "pricing::asian::geometric::call::spot")?,
                d_mul(k_leg, n_d2, "pricing::asian::geometric::call::strike")?,
                "pricing::asian::geometric::call",
            )?
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            d_sub(
                d_mul(k_leg, n_neg_d2, "pricing::asian::geometric::put::strike")?,
                d_mul(s_leg, n_neg_d1, "pricing::asian::geometric::put::spot")?,
                "pricing::asian::geometric::put",
            )?
        }
    };

    Ok(apply_side(price, option))
}

/// Prices an arithmetic average Asian option using Turnbull-Wakeman approximation.
///
/// The arithmetic average of a lognormal process is not lognormal, so no
/// closed-form solution exists. This implementation uses the Turnbull-Wakeman
/// (1991) approximation which matches the first two moments of the arithmetic
/// average to a lognormal distribution.
fn arithmetic_asian_price(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        return intrinsic_value(option);
    }

    let t_dec = t.to_dec();
    let discount = d_exp(
        d_mul(-r, t_dec, "pricing::asian::arithmetic::neg_rt")?,
        "pricing::asian::arithmetic::discount",
    )?;

    if sigma == Positive::ZERO {
        // Same correction as in the geometric kernel: a deterministic path is
        // still averaged. The arithmetic mean of `S e^{b t}` over the window
        // is `M1`, which carries no σ at all, so it is both the `σ → 0` value
        // of the moment matching below and the answer here. The terminal
        // forward `S e^{b T}` overstates it for every `b > 0`.
        let carry = d_sub(r, q, "pricing::asian::arithmetic::det::carry")?;
        let average = arithmetic_average_forward(s.to_dec(), carry, t_dec)?;
        return deterministic_price(average, k.to_dec(), discount, option);
    }

    // Turnbull-Wakeman approximation
    let b = d_sub(r, q, "pricing::asian::arithmetic::carry")?; // cost of carry
    let sigma_dec = sigma.to_dec();
    let sigma_sq = d_mul(sigma_dec, sigma_dec, "pricing::asian::arithmetic::sigma_sq")?;
    let s_dec = s.to_dec();
    let s_sq = d_powd(s_dec, Decimal::TWO, "pricing::asian::arithmetic::s_sq")?;
    let two_b_plus_var = d_add(
        d_mul(dec!(2), b, "pricing::asian::arithmetic::two_b")?,
        sigma_sq,
        "pricing::asian::arithmetic::two_b_plus_var",
    )?;
    let b_plus_var = d_add(b, sigma_sq, "pricing::asian::arithmetic::b_plus_var")?;

    // First moment of the arithmetic average (M1), including its own
    // removable singularity at `b = 0`.
    let m1 = arithmetic_average_forward(s_dec, b, t_dec)?;

    // Second moment of arithmetic average (M2).
    //
    // Writing `a = b + σ²` and `c = 2b + σ²`, the Turnbull-Wakeman second
    // moment is the double integral of `E[S_t S_u] = S² e^{b(t+u)+σ² min(t,u)}`
    // over the averaging window,
    //
    //     M2 = (2 S² / T²) ∫₀^T e^{b u} ∫₀^u e^{a t} dt du
    //        = (2 S² / (a T²)) [ (e^{c T} - 1) / c - (e^{b T} - 1) / b ].
    //
    // Both brackets are windows of `e^{w t}`, so the whole expression is
    // carried by the single function
    //
    //     φ(w) = (e^w - 1) / w = ∫₀^1 e^{w s} ds,   φ(0) = 1,
    //
    // read at the three dimensionless rates `x = b T`, `z = c T` and
    // `y = a T`, which satisfy `y = z - x`. Collecting on them,
    //
    //     M2 = S² g(x, z),   g(x, z) = 2 (φ(z) - φ(x)) / (z - x),
    //
    // twice the first divided difference of `φ`. That is the natural object
    // here: it is symmetric, entire in both arguments, and each of the three
    // apparent poles of the closed form is an ordinary point of it. `b = 0`
    // is `x = 0` and `c = 0` is `z = 0`, both covered by `φ(0) = 1`; `a = 0`
    // is `z = x`, where a divided difference is the derivative `2 φ'(x)`. No
    // branch has to reconstruct a limit the expression does not already hold,
    // and — the point of this shape — nothing is recovered from a difference
    // of two terms larger than the answer.
    let x = d_mul(b, t_dec, "pricing::asian::arithmetic::m2::outer_rate")?;
    let z = d_mul(
        two_b_plus_var,
        t_dec,
        "pricing::asian::arithmetic::m2::total_rate",
    )?;
    let y = d_mul(
        b_plus_var,
        t_dec,
        "pricing::asian::arithmetic::m2::inner_rate",
    )?;
    let m2 = d_mul(
        s_sq,
        scaled_second_moment(x, z, y)?,
        "pricing::asian::arithmetic::m2",
    )?;

    // Forward price of the average
    let f_adj = m1;

    // A non-positive first moment leaves the moment matching undefined: the
    // average is known to be `f_adj`, so the option is worth its discounted
    // intrinsic (the σ → 0 limit of the Black formula below).
    if f_adj <= Decimal::ZERO {
        return deterministic_price(f_adj, k.to_dec(), discount, option);
    }

    // Adjusted volatility from moment matching
    let m1_sq = d_powd(m1, Decimal::TWO, "pricing::asian::arithmetic::m1_sq")?;
    let moment_ratio = d_div(m2, m1_sq, "pricing::asian::arithmetic::moment_ratio")?;
    let variance = if moment_ratio > Decimal::ZERO {
        d_div(
            d_ln(moment_ratio, "pricing::asian::arithmetic::log_moment_ratio")?,
            t_dec,
            "pricing::asian::arithmetic::variance",
        )?
    } else {
        // `M2 / M1²` collapsed below the representable scale: the matched
        // lognormal has no spread left, which the `sqrt` fallback below maps
        // back onto the input volatility.
        Decimal::ZERO
    };
    let sigma_adj = variance.sqrt().unwrap_or(sigma_dec);

    // Use Black-Scholes with adjusted parameters
    let sqrt_t = d_sqrt(t_dec, "pricing::asian::arithmetic::sqrt_t")?;
    let denominator = d_mul(sigma_adj, sqrt_t, "pricing::asian::arithmetic::denominator")?;
    if denominator.is_zero() {
        // Zero matched volatility (or zero maturity in the limit): the average
        // is deterministic and the price is the discounted intrinsic.
        return deterministic_price(f_adj, k.to_dec(), discount, option);
    }

    let moneyness = d_div(f_adj, k.to_dec(), "pricing::asian::arithmetic::moneyness")?;
    let log_moneyness = if moneyness.is_zero() {
        // `F / K` rounded below the smallest representable `Decimal`, so the
        // logarithm diverges; `Decimal::MIN` is the representable stand-in and
        // drives `big_n` to the same 0 / 1 saturation the limit produces.
        Decimal::MIN
    } else {
        d_ln(moneyness, "pricing::asian::arithmetic::log_moneyness")?
    };
    let half_variance_t = d_div(
        d_mul(
            d_mul(
                sigma_adj,
                sigma_adj,
                "pricing::asian::arithmetic::sigma_adj_sq",
            )?,
            t_dec,
            "pricing::asian::arithmetic::variance_t",
        )?,
        dec!(2),
        "pricing::asian::arithmetic::half_variance_t",
    )?;
    let d1_val = d_div(
        d_add(
            log_moneyness,
            half_variance_t,
            "pricing::asian::arithmetic::d1_numerator",
        )?,
        denominator,
        "pricing::asian::arithmetic::d1",
    )?;
    let d2_val = d_sub(d1_val, denominator, "pricing::asian::arithmetic::d2")?;

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            d_mul(
                discount,
                d_sub(
                    d_mul(f_adj, n_d1, "pricing::asian::arithmetic::call::forward")?,
                    d_mul(k.to_dec(), n_d2, "pricing::asian::arithmetic::call::strike")?,
                    "pricing::asian::arithmetic::call::intrinsic",
                )?,
                "pricing::asian::arithmetic::call",
            )?
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            d_mul(
                discount,
                d_sub(
                    d_mul(
                        k.to_dec(),
                        n_neg_d2,
                        "pricing::asian::arithmetic::put::strike",
                    )?,
                    d_mul(f_adj, n_neg_d1, "pricing::asian::arithmetic::put::forward")?,
                    "pricing::asian::arithmetic::put::intrinsic",
                )?,
                "pricing::asian::arithmetic::put",
            )?
        }
    };

    Ok(apply_side(price, option))
}

/// Average of `e^{w s}` over the unit interval, `φ(w) = (e^w - 1) / w`.
///
/// The building block of both Turnbull-Wakeman moments: `M1 = S φ(bT)` and
/// `M2 = 2 S² (φ(cT) - φ(bT)) / (aT)`. The `w = 0` singularity is removable
/// and the limit is `∫₀^1 ds = 1`, which the series below returns exactly
/// rather than dividing by the rate that vanished.
///
/// Small `w` is summed rather than evaluated, and at the *same*
/// [`M2_SERIES_THRESHOLD`] the second moment uses. Sharing the constant is
/// the point, not a convenience: the two moments have to change regime
/// together. When they did not, `M1` collapsed to exactly `S` below a cutoff
/// of its own while `M2` went on evaluating the real carry, and the price
/// jumped twofold across that cutoff — `S = K = 100`, `σ = 1e-5`, `T = 1`
/// priced at `4.6066e-4` for `b = 9.99999e-11` and at `2.3033e-4` for
/// `b = 1e-10`, for a `1e-16` change in the rate.
///
/// The closed form loses the leading digits of `e^w` to the `1 /`
/// amplification exactly as it does inside `φ(z) - φ(x)`: `e^w` rounds at
/// `Decimal`'s scale, so `(e^w - 1) / w` carries a relative error of about
/// `u / |w|` — `1e-18` at `w = 1e-10`, but `1e-8` by `w = 1e-20`, past the
/// `1e-9` bound the issue sets. The series has no such subtraction and
/// truncates at `w⁶ / 7!`, a relative `2e-4 w⁶`, which is `2e-22` at the
/// threshold.
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when `e^w` leaves the representable
/// `Decimal` range.
fn growth_average(w: Decimal) -> Result<Decimal, PricingError> {
    if w.abs() < M2_SERIES_THRESHOLD {
        return growth_average_series(w);
    }
    Ok(d_div(
        d_sub(
            d_exp(w, "pricing::asian::growth_average::exp")?,
            Decimal::ONE,
            "pricing::asian::growth_average::numerator",
        )?,
        w,
        "pricing::asian::growth_average",
    )?)
}

/// `φ(w)` summed as `Σ_{n ≥ 0} wⁿ / (n+1)! = 1 + w/2 + w²/6 + …`, which
/// carries the leading `1` explicitly instead of recovering it from
/// `e^w - 1`.
///
/// `w = 0` gives exactly `1` here, every term after the first vanishing, so
/// the removable singularity needs no guard of its own.
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when a power, a factorial or the running
/// sum leaves the representable `Decimal` range.
fn growth_average_series(w: Decimal) -> Result<Decimal, PricingError> {
    // The `n = 0` term is `w⁰ / 1! = 1`, the `1` the correction is added to.
    let mut w_power = Decimal::ONE;
    let mut factorial = Decimal::ONE;
    let mut correction = Decimal::ZERO;
    for n in 1..M2_SERIES_TERMS {
        w_power = d_mul(w_power, w, "pricing::asian::phi::series::power")?;
        factorial = d_mul(
            factorial,
            Decimal::from(n + 1),
            "pricing::asian::phi::series::factorial",
        )?;
        correction = d_add(
            correction,
            d_div(w_power, factorial, "pricing::asian::phi::series::term")?,
            "pricing::asian::phi::series::correction",
        )?;
    }
    Ok(d_add(
        Decimal::ONE,
        correction,
        "pricing::asian::phi::series",
    )?)
}

/// The Turnbull-Wakeman second moment of the arithmetic average, divided by
/// `S²`.
///
/// `g(x, z) = 2 (φ(z) - φ(x)) / (z - x)` at `x = bT`, `z = (2b + σ²) T` and
/// `y = (b + σ²) T`, which equals `z - x` and is passed separately because
/// the product is better conditioned than the difference. `g` is symmetric,
/// entire, and equals `1 + (x + z) / 3 + …` at the origin, so the moment
/// ratio `M2 / M1²` it feeds tends to `1` and the matched variance to `0` as
/// the window closes.
///
/// Three regimes, each chosen where its error is the smaller of the two
/// available (see [`M2_SERIES_THRESHOLD`] and [`M2_MIDPOINT_THRESHOLD`] for
/// the measurements):
///
/// - `max(|x|, |z|) < M2_SERIES_THRESHOLD` — the power series. Both `φ` are
///   within `u / 2` of `1` there, so the closed form would recover `g / 2`
///   from a difference of two numbers a decade or more larger than it, and
///   lose the leading `e^w` digits to the `1 /` amplification.
/// - `|y| < M2_MIDPOINT_THRESHOLD` — the derivative at the midpoint,
///   `2 φ'(m)`, `m = (x + z) / 2`. A divided difference over a vanishing
///   interval is a derivative; the midpoint makes the neglected term
///   `φ'''(m) (y / 2)² / 6` rather than the first-order `φ''(x) y / 2`.
/// - otherwise the divided difference itself.
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when any exponential or product leaves
/// the representable `Decimal` range.
fn scaled_second_moment(x: Decimal, z: Decimal, y: Decimal) -> Result<Decimal, PricingError> {
    if x.abs().max(z.abs()) < M2_SERIES_THRESHOLD {
        return second_moment_series(x, z);
    }

    if y.abs() < M2_MIDPOINT_THRESHOLD {
        // `z = x` to within the threshold, so
        //
        //     g = 2 φ'(m) = 2 [m e^m - e^m + 1] / m²,   m = (x + z) / 2.
        //
        // `m` cannot vanish here: `m = 0` means `z = -x`, hence
        // `|y| = |z - x| = 2 max(|x|, |z|)`, which the branch above already
        // took when that maximum was small and which fails this test when it
        // was not.
        let m = d_div(
            d_add(x, z, "pricing::asian::m2::midpoint::sum")?,
            dec!(2),
            "pricing::asian::m2::midpoint",
        )?;
        debug_assert!(!m.is_zero(), "midpoint branch reached with x = -z");
        let exp_m = d_exp(m, "pricing::asian::m2::midpoint::exp")?;
        return Ok(d_div(
            d_mul(
                dec!(2),
                d_add(
                    d_mul(
                        exp_m,
                        d_sub(m, Decimal::ONE, "pricing::asian::m2::midpoint::m_less_one")?,
                        "pricing::asian::m2::midpoint::exp_term",
                    )?,
                    Decimal::ONE,
                    "pricing::asian::m2::midpoint::bracket",
                )?,
                "pricing::asian::m2::midpoint::numerator",
            )?,
            d_mul(m, m, "pricing::asian::m2::midpoint::m_sq")?,
            "pricing::asian::m2::midpoint::derivative",
        )?);
    }

    Ok(d_div(
        d_mul(
            dec!(2),
            d_sub(
                growth_average(z)?,
                growth_average(x)?,
                "pricing::asian::m2::divided_difference::numerator",
            )?,
            "pricing::asian::m2::divided_difference::scaled",
        )?,
        y,
        "pricing::asian::m2::divided_difference",
    )?)
}

/// `g(x, z)` summed term by term instead of evaluated in closed form.
///
/// `φ(w) = Σ_{n ≥ 0} w^n / (n + 1)!` is entire, so its divided difference is
/// too, and the difference of powers telescopes:
///
/// ```text
/// g(x, z) = 2 Σ_{n ≥ 1} (z^n - x^n) / ((z - x) (n + 1)!)
///         = 2 Σ_{k ≥ 0} h_k(x, z) / (k + 2)!
///         = 1 + (x + z) / 3 + (x² + x z + z²) / 12 + …
/// ```
///
/// with `h_k(x, z) = Σ_{j=0}^{k} x^j z^{k-j}` the complete homogeneous
/// symmetric polynomial, accumulated by `h_k = x h_{k-1} + z^k`. Nothing
/// cancels: the leading `1` is exact and every correction is added to it at
/// its own scale, so the result carries the digits the divided difference
/// spends on `φ(z) - φ(x)`. At `x = 0` this is the driftless series
/// `1 + z / 3 + z² / 12 + …` of the `b = 0` limit.
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when a power or a factorial leaves the
/// representable `Decimal` range, which the [`M2_SERIES_THRESHOLD`] bound on
/// `x` and `z` rules out for every admissible contract.
fn second_moment_series(x: Decimal, z: Decimal) -> Result<Decimal, PricingError> {
    // `h_0 = 1`, `z^0 = 1`, `(0 + 2)! = 2`, so the `k = 0` term is exactly 1.
    let mut h = Decimal::ONE;
    let mut z_power = Decimal::ONE;
    let mut factorial = dec!(2);
    let mut correction = Decimal::ZERO;
    for k in 1..M2_SERIES_TERMS {
        z_power = d_mul(z_power, z, "pricing::asian::m2::series::z_power")?;
        h = d_add(
            d_mul(x, h, "pricing::asian::m2::series::shift")?,
            z_power,
            "pricing::asian::m2::series::homogeneous",
        )?;
        factorial = d_mul(
            factorial,
            Decimal::from(k + 2),
            "pricing::asian::m2::series::factorial",
        )?;
        correction = d_add(
            correction,
            d_div(
                d_mul(dec!(2), h, "pricing::asian::m2::series::twice")?,
                factorial,
                "pricing::asian::m2::series::term",
            )?,
            "pricing::asian::m2::series::correction",
        )?;
    }
    Ok(d_add(
        Decimal::ONE,
        correction,
        "pricing::asian::m2::series",
    )?)
}

/// Arithmetic average of the deterministic carry path, `S (e^{bT} - 1) / (bT)`.
///
/// This is the Turnbull-Wakeman first moment `M1`. It carries no volatility:
/// the mean of `S e^{b t}` over `[0, T]` is the same whether or not the path
/// fluctuates around it, so the expression doubles as the `σ → 0` value of
/// the arithmetic average.
///
/// The `b → 0` singularity is removable and the limit is `S` itself:
/// `(e^{bT} - 1) / (bT) = 1 + bT/2 + (bT)²/6 + … → 1`. The same `1e-10`
/// threshold as the second moment selects it, so the two moments switch to
/// their limits together.
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when `e^{bT}` or the product with `S`
/// leaves the representable `Decimal` range.
fn arithmetic_average_forward(s: Decimal, b: Decimal, t: Decimal) -> Result<Decimal, PricingError> {
    // No cutoff of its own. A `|b| < 1e-10` shortcut to exactly `S` used to
    // live here, which put this moment in a different limit regime from the
    // second one and made the price discontinuous across the boundary.
    // `growth_average` handles a vanishing `bT` by summing its series, and
    // returns exactly `1` at zero.
    let bt = d_mul(b, t, "pricing::asian::average_forward::bt")?;
    Ok(d_mul(
        s,
        growth_average(bt)?,
        "pricing::asian::average_forward",
    )?)
}

/// Discounted intrinsic on a known average, i.e. the `σ → 0` limit of the
/// Black formula used by both Asian kernels.
fn deterministic_price(
    forward: Decimal,
    strike: Decimal,
    discount: Decimal,
    option: &Options,
) -> Result<Decimal, PricingError> {
    let intrinsic = match option.option_style {
        OptionStyle::Call => {
            d_sub(forward, strike, "pricing::asian::deterministic::call")?.max(Decimal::ZERO)
        }
        OptionStyle::Put => {
            d_sub(strike, forward, "pricing::asian::deterministic::put")?.max(Decimal::ZERO)
        }
    };
    let price = d_mul(intrinsic, discount, "pricing::asian::deterministic::price")?;
    Ok(apply_side(price, option))
}

/// Calculates intrinsic value at expiration.
fn intrinsic_value(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let value = match option.option_style {
        OptionStyle::Call => d_sub(s, k, "pricing::asian::intrinsic::call")?.max(Decimal::ZERO),
        OptionStyle::Put => d_sub(k, s, "pricing::asian::intrinsic::put")?.max(Decimal::ZERO),
    };
    Ok(apply_side(value, option))
}

/// Applies the side (long/short) multiplier to the price.
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
    use crate::assert_decimal_eq;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_asian_option(style: OptionStyle, averaging_type: AsianAveragingType) -> Options {
        Options::new(
            OptionType::Asian { averaging_type },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,                          // strike
            ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 years
            pos_or_panic!(0.25),                        // volatility
            Positive::ONE,                              // quantity
            Positive::HUNDRED,                          // underlying
            dec!(0.05),                                 // risk-free rate
            style,
            Positive::ZERO, // dividend yield
            None,
        )
    }

    /// `S = K = 100`, `T = 1`, `σ = 20%`, `r = 0`, so the Turnbull-Wakeman
    /// carry is `b = -q` and the dividend yield alone selects the boundary:
    /// `q = 4%` puts `b` on `-σ²`, `q = 2%` puts it on `-σ² / 2`.
    fn create_turnbull_wakeman_boundary_option(dividend_yield: Decimal) -> Options {
        Options::new(
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(365.0)),
            Positive::new_decimal(dec!(0.2)).unwrap(),
            Positive::ONE,
            Positive::HUNDRED,
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::new_decimal(dividend_yield).unwrap(),
            None,
        )
    }

    /// `b = -σ²` zeroes the `b + σ²` factor of the Turnbull-Wakeman second
    /// moment. The singularity is removable, so an ordinary contract must
    /// price rather than raise.
    #[test]
    fn test_arithmetic_asian_carry_at_negative_variance_prices() {
        let option = create_turnbull_wakeman_boundary_option(dec!(0.04));
        let price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(price, dec!(3.6245129), dec!(1e-6));

        let mut geometric = option.clone();
        geometric.option_type = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let geometric_price = asian_black_scholes(&geometric).unwrap();
        assert!(
            geometric_price < price,
            "geometric {} should sit below arithmetic {}",
            geometric_price,
            price
        );
    }

    /// `b = -σ² / 2` zeroes the `2b + σ²` factor of the Turnbull-Wakeman
    /// second moment. Removable for the same reason.
    #[test]
    fn test_arithmetic_asian_carry_at_half_negative_variance_prices() {
        let option = create_turnbull_wakeman_boundary_option(dec!(0.02));
        let price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(price, dec!(4.0977699), dec!(1e-6));

        let mut geometric = option.clone();
        geometric.option_type = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let geometric_price = asian_black_scholes(&geometric).unwrap();
        assert!(
            geometric_price < price,
            "geometric {} should sit below arithmetic {}",
            geometric_price,
            price
        );
    }

    /// The limit branch has to agree with the general formula evaluated a hair
    /// off the boundary, otherwise it is merely a value and not the limit.
    ///
    /// The tolerance is `1e-7`. A `1e-9` shift in `q` moves the price by about
    /// `2.3e-8` through the genuine `dP/dq` sensitivity, and the general
    /// branch still carries roughly seventeen significant digits at
    /// `|b + σ²| = 1e-9` despite the cancellation between its two terms, so
    /// `1e-7` clears the real sensitivity with room to spare while a wrong
    /// limiting expression would miss by order one.
    #[test]
    fn test_arithmetic_asian_negative_variance_boundary_is_continuous() {
        let at = asian_black_scholes(&create_turnbull_wakeman_boundary_option(dec!(0.04))).unwrap();
        let below =
            asian_black_scholes(&create_turnbull_wakeman_boundary_option(dec!(0.039999999)))
                .unwrap();
        let above =
            asian_black_scholes(&create_turnbull_wakeman_boundary_option(dec!(0.040000001)))
                .unwrap();

        assert_decimal_eq!(at, below, dec!(1e-7));
        assert_decimal_eq!(at, above, dec!(1e-7));
        assert!(
            above < at && at < below,
            "the price must stay monotone in q across the boundary: {} {} {}",
            below,
            at,
            above
        );
    }

    /// Same continuity check at the second removable singularity, same
    /// tolerance and same reasoning.
    #[test]
    fn test_arithmetic_asian_half_negative_variance_boundary_is_continuous() {
        let at = asian_black_scholes(&create_turnbull_wakeman_boundary_option(dec!(0.02))).unwrap();
        let below =
            asian_black_scholes(&create_turnbull_wakeman_boundary_option(dec!(0.019999999)))
                .unwrap();
        let above =
            asian_black_scholes(&create_turnbull_wakeman_boundary_option(dec!(0.020000001)))
                .unwrap();

        assert_decimal_eq!(at, below, dec!(1e-7));
        assert_decimal_eq!(at, above, dec!(1e-7));
        assert!(
            above < at && at < below,
            "the price must stay monotone in q across the boundary: {} {} {}",
            below,
            at,
            above
        );
    }

    /// `S = K = 100`, `T = 1`, `σ = 20%`, `q = 4%`, so the Turnbull-Wakeman
    /// carry is `b = r - 4%` and the risk-free rate alone selects the
    /// boundary: `r = 4%` puts `b` on zero. That is a fully-carried
    /// underlying, i.e. a forward-priced contract, not an edge case.
    fn create_zero_carry_option(risk_free_rate: Decimal) -> Options {
        Options::new(
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(365.0)),
            Positive::new_decimal(dec!(0.2)).unwrap(),
            Positive::ONE,
            Positive::HUNDRED,
            risk_free_rate,
            OptionStyle::Call,
            Positive::new_decimal(dec!(0.04)).unwrap(),
            None,
        )
    }

    /// `S = K = 100`, `T = 1`, `r = 5%`, `q = 0`: a carried underlying on a
    /// deterministic path, whose payoff is still struck against an *average*.
    fn create_deterministic_option(
        averaging_type: AsianAveragingType,
        implied_volatility: Positive,
    ) -> Options {
        Options::new(
            OptionType::Asian { averaging_type },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(365.0)),
            implied_volatility,
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        )
    }

    fn low_vol_option(risk_free_rate: Decimal) -> Options {
        Options::new(
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(365.0)),
            Positive::new_decimal(dec!(1e-5)).unwrap(),
            Positive::ONE,
            Positive::HUNDRED,
            risk_free_rate,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        )
    }

    /// The two moments have to change limit regime at the same place. `M1`
    /// used to collapse to exactly `S` below `|b| = 1e-10` while `M2` went on
    /// evaluating the real carry, so the price jumped by a factor of two
    /// across a cutoff only one of them had: `4.6066e-4` against `2.3033e-4`
    /// for a `1e-16` change in the rate, on a contract that is otherwise
    /// ordinary. Sharing `M2_SERIES_THRESHOLD` is what removes the seam.
    #[test]
    fn test_arithmetic_asian_low_volatility_carry_boundary_is_continuous() {
        let price_at = |rate: Decimal| asian_black_scholes(&low_vol_option(rate)).unwrap();

        let below = price_at(dec!(9.99999e-11));
        let at = price_at(dec!(1e-10));
        let above = price_at(dec!(1.00001e-10));

        // Neighbours a relative 1e-5 apart in the rate, on either side of the
        // cutoff that used to be there. The gaps are 2.5e-15 and 2.5e-14
        // absolute, a relative 1.1e-11 and 1.1e-10 on a price of 2.3e-4,
        // against the factor of two the seam used to produce.
        assert_decimal_eq!(below, at, dec!(1e-13));
        assert_decimal_eq!(above, at, dec!(1e-13));
        assert!(
            below < at && at < above,
            "the price must stay monotone in the carry across the old cutoff: {below} {at} {above}"
        );

        // And it is the carry that moves it: three decades up, the price has
        // to have moved by more than that gap.
        assert!(
            price_at(dec!(1e-7)) > at + dec!(1e-13),
            "a real carry must price above the vanishing one"
        );
    }

    /// `b = 0` zeroes the outer rate of the Turnbull-Wakeman second moment.
    /// Removable like the other two, and the limit is emphatically not
    /// `S² e^{σ² T}`: quadrature on the defining double integral gives
    /// `M2 = 1.0134677405 S²`, which prices at `4.4308753050`.
    #[test]
    fn test_arithmetic_asian_zero_carry_prices_the_average() {
        let option = create_zero_carry_option(dec!(0.04));
        let price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(price, dec!(4.4308753050), dec!(1e-6));

        // The matched volatility has to land just above the geometric-average
        // volatility `σ / √3`, so the two prices must sit close together. The
        // terminal second moment would have matched the spot `σ = 20%`
        // instead and priced this contract at `7.6532`, more than `3.3` above
        // the geometric leg.
        let mut geometric = option.clone();
        geometric.option_type = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let geometric_price = asian_black_scholes(&geometric).unwrap();
        assert_decimal_eq!(geometric_price, dec!(4.2581167661), dec!(1e-6));
        assert!(
            geometric_price < price && price < geometric_price + dec!(0.25),
            "arithmetic {} must sit just above geometric {}",
            price,
            geometric_price
        );
    }

    /// The limit branch has to agree with the general formula evaluated a hair
    /// off the boundary, otherwise it is merely a value and not the limit.
    ///
    /// The tolerance is `1e-7`. A `1e-9` shift in `r` moves the price by about
    /// `2.1e-8` through the genuine `dP/dr ≈ 21` sensitivity, and the general
    /// branch holds its precision at `|b| = 1e-9` despite the cancellation
    /// between its two terms — the two neighbours below straddle the limit and
    /// are only `4.3e-8` apart from each other. `1e-7` therefore clears the
    /// real sensitivity while the terminal-moment stand-in would miss by `3.2`.
    #[test]
    fn test_arithmetic_asian_zero_carry_boundary_is_continuous() {
        let at = asian_black_scholes(&create_zero_carry_option(dec!(0.04))).unwrap();
        let below = asian_black_scholes(&create_zero_carry_option(dec!(0.039999999))).unwrap();
        let above = asian_black_scholes(&create_zero_carry_option(dec!(0.040000001))).unwrap();

        assert_decimal_eq!(at, below, dec!(1e-7));
        assert_decimal_eq!(at, above, dec!(1e-7));
        assert!(
            below < at && at < above,
            "the price must stay monotone in r across the boundary: {} {} {}",
            below,
            at,
            above
        );
    }

    /// A volatility under `1e-14` squares to zero at `Decimal`'s scale, which
    /// would make the vanishing-carry branch divide by the variance it just
    /// lost. The nested `σ² → 0` limit is `M2 = S²`, matching `M1²`, so the
    /// matched variance is zero and the contract prices as the discounted
    /// intrinsic on the average — `b = 0`, so that average is `S` itself.
    #[test]
    fn test_arithmetic_asian_zero_carry_underflowed_volatility_prices() {
        let mut option = create_zero_carry_option(dec!(0.04));
        option.underlying_price = pos_or_panic!(110.0);
        option.implied_volatility = Positive::new_decimal(dec!(1e-15)).unwrap();

        let price = asian_black_scholes(&option).unwrap();

        // (110 - 100) e^{-0.04}.
        assert_decimal_eq!(price, dec!(9.6078943915), dec!(1e-9));
    }

    /// Below `x = σ² T = 1e-6` the vanishing-carry branch sums the series
    /// instead of evaluating `e^x - 1 - x`, whose leading `x² / 2` would sink
    /// under `Decimal`'s 28-decimal floor. The two have to meet at the
    /// threshold: `σ = 0.001` puts `x` exactly on `1e-6`, so the neighbours
    /// below and above are priced by different formulas. The tolerance is
    /// `1e-7`; the genuine `dP/dσ ≈ 22.13` accounts for `2.2e-8` of each
    /// measured gap and the formulas themselves differ by `2.8e-9`.
    #[test]
    fn test_arithmetic_asian_zero_carry_small_variance_series_matches_closed_form() {
        let mut option = create_zero_carry_option(dec!(0.04));

        option.implied_volatility = Positive::new_decimal(dec!(0.000999999)).unwrap();
        let series = asian_black_scholes(&option).unwrap();
        option.implied_volatility = Positive::new_decimal(dec!(0.001)).unwrap();
        let at = asian_black_scholes(&option).unwrap();
        option.implied_volatility = Positive::new_decimal(dec!(0.001000001)).unwrap();
        let closed_form = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(series, at, dec!(1e-7));
        assert_decimal_eq!(closed_form, at, dec!(1e-7));
        assert!(
            series < at && at < closed_form,
            "the price must stay monotone in σ across the threshold: {} {} {}",
            series,
            at,
            closed_form
        );
    }

    /// A near-zero volatility must leave a near-zero price. The closed form
    /// cannot deliver that on its own: at `σ = 1e-7` (`x = 1e-14`) it loses
    /// the whole `x² / 2` to rounding and prices this worthless contract at
    /// `31`. The series keeps the matched volatility at `σ / √3`, so the
    /// price stays the Black value `e^{-0.04} · 100 · (2 N(σ_adj / 2) - 1)`.
    #[test]
    fn test_arithmetic_asian_zero_carry_tiny_volatility_stays_proportional() {
        let mut option = create_zero_carry_option(dec!(0.04));
        option.implied_volatility = Positive::new_decimal(dec!(1e-7)).unwrap();

        let price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(price, dec!(0.0000022129811), dec!(1e-11));
    }

    /// A deterministic path is still averaged: the geometric mean of
    /// `S e^{b t}` over `[0, T]` is `S e^{bT/2}`, not the terminal forward
    /// `S e^{bT}`. `(100 e^{0.025} - 100) e^{-0.05} = 2.4080487528`.
    #[test]
    fn test_geometric_asian_zero_volatility_averages_the_path() {
        let option = create_deterministic_option(AsianAveragingType::Geometric, Positive::ZERO);
        let price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(price, dec!(2.4080487528), dec!(1e-9));

        // And it is the `σ → 0` limit of the Kemna-Vorst legs rather than a
        // separate convention: at `σ = 1e-6` the closed form is already there.
        let near = create_deterministic_option(
            AsianAveragingType::Geometric,
            Positive::new_decimal(dec!(1e-6)).unwrap(),
        );
        assert_decimal_eq!(price, asian_black_scholes(&near).unwrap(), dec!(1e-9));
    }

    /// Same for the arithmetic kernel, where the average of the deterministic
    /// path is `M1 = S (e^{bT} - 1) / (bT)`.
    /// `(102.5421927520 - 100) e^{-0.05} = 2.4182085485`.
    #[test]
    fn test_arithmetic_asian_zero_volatility_averages_the_path() {
        let option = create_deterministic_option(AsianAveragingType::Arithmetic, Positive::ZERO);
        let price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(price, dec!(2.4182085485), dec!(1e-9));

        let near = create_deterministic_option(
            AsianAveragingType::Arithmetic,
            Positive::new_decimal(dec!(1e-6)).unwrap(),
        );
        assert_decimal_eq!(price, asian_black_scholes(&near).unwrap(), dec!(1e-9));
    }

    #[test]
    fn test_geometric_asian_call() {
        let option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        let price = asian_black_scholes(&option).unwrap();
        // Price should be positive and less than vanilla BS price
        assert!(
            price > Decimal::ZERO,
            "Geometric Asian call should be positive: {}",
            price
        );
        assert!(
            price < dec!(15.0),
            "Geometric Asian call should be less than vanilla"
        );
    }

    #[test]
    fn test_geometric_asian_put() {
        let option = create_asian_option(OptionStyle::Put, AsianAveragingType::Geometric);
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Geometric Asian put should be positive: {}",
            price
        );
    }

    #[test]
    fn test_arithmetic_asian_call() {
        let option = create_asian_option(OptionStyle::Call, AsianAveragingType::Arithmetic);
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Arithmetic Asian call should be positive: {}",
            price
        );
    }

    #[test]
    fn test_arithmetic_asian_put() {
        let option = create_asian_option(OptionStyle::Put, AsianAveragingType::Arithmetic);
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Arithmetic Asian put should be positive: {}",
            price
        );
    }

    #[test]
    fn test_geometric_less_than_arithmetic() {
        // For standard cases, geometric average <= arithmetic average
        // So geometric Asian call <= arithmetic Asian call
        let geometric = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        let arithmetic = create_asian_option(OptionStyle::Call, AsianAveragingType::Arithmetic);

        let geo_price = asian_black_scholes(&geometric).unwrap();
        let arith_price = asian_black_scholes(&arithmetic).unwrap();

        // Allow some tolerance for approximation errors
        assert!(
            geo_price <= arith_price + dec!(0.5),
            "Geometric {} should be <= Arithmetic {}",
            geo_price,
            arith_price
        );
    }

    #[test]
    fn test_short_asian_option() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        let long_price = asian_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = asian_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = asian_black_scholes(&option).unwrap();
        assert_decimal_eq!(price, Decimal::ZERO, dec!(1e-10));
    }

    #[test]
    fn test_itm_asian_call() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        option.underlying_price = pos_or_panic!(120.0); // ITM
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price > dec!(10.0),
            "ITM Asian call should have significant value: {}",
            price
        );
    }

    #[test]
    fn test_otm_asian_call() {
        let mut option = create_asian_option(OptionStyle::Call, AsianAveragingType::Geometric);
        option.underlying_price = pos_or_panic!(80.0); // OTM
        let price = asian_black_scholes(&option).unwrap();
        assert!(
            price < dec!(5.0),
            "OTM Asian call should have low value: {}",
            price
        );
    }
    /// `S = K = 100`, `σ = 20%`, `r = 8%`, `q = 4%` — the contract of the
    /// short-maturity table, parameterised by a maturity in days.
    fn create_short_maturity_option(days: Decimal, style: OptionStyle) -> Options {
        Options::new(
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(Positive::new_decimal(days).unwrap()),
            Positive::new_decimal(dec!(0.2)).unwrap(),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.08),
            style,
            Positive::new_decimal(dec!(0.04)).unwrap(),
            None,
        )
    }

    /// `S = K = 100`, `σ = 20%`, `r = 0` at a chosen maturity, so the
    /// dividend yield alone selects the removable boundary: `q = 4%` puts the
    /// carry on `b = -σ²` and `q = 2%` on `b = -σ²/2`. The same two contracts
    /// as `create_turnbull_wakeman_boundary_option`, with the window free.
    fn create_boundary_option_at(dividend_yield: Decimal, days: Decimal) -> Options {
        Options::new(
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(Positive::new_decimal(days).unwrap()),
            Positive::new_decimal(dec!(0.2)).unwrap(),
            Positive::ONE,
            Positive::HUNDRED,
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::new_decimal(dividend_yield).unwrap(),
            None,
        )
    }

    /// Distance from a reference as a fraction of it. The maturities below
    /// span three decades of price, so a single absolute tolerance would mean
    /// something different on every row.
    fn relative_error(value: Decimal, reference: Decimal) -> Decimal {
        ((value - reference) / reference).abs()
    }

    /// Every maturity of the short-maturity table, against 50-digit `mpmath`
    /// quadrature on the integral the second moment is defined by,
    ///
    ///     E[A²] = (2 S² / T²) ∫₀^T e^{b u} ∫₀^u e^{a t} dt du,
    ///
    /// with the moment matching and the normal legs carried at 50 digits too.
    /// The quadrature agrees with the closed form to `1e-38` at every row, so
    /// the reference does not assume either.
    ///
    /// | maturity | reference | before |
    /// |---|---|---|
    /// | 0.5 days | `0.171855815411` | `0.171855815757` |
    /// | 0.2 days | `0.108377120846` | `0.108377149270` |
    /// | 0.1 days | `0.076521744333` | `0.076522709098` |
    /// | 0.05 days | `0.054052666108` | `0.054005346244` |
    /// | 0.02 days | `0.034154202089` | `0.034378185828` |
    /// | 0.01 days | `0.024139351974` | `0.041790507589` |
    /// | 0.005 days | `0.017063436757` | `0.234996014908` |
    /// | 0.002 days | `0.010788684989` | `0.875324687084` |
    /// | 0.001 days | `0.007627618506` | `4.678919422800` |
    ///
    /// As the window closes the quantity the second moment has to deliver,
    /// `σ²_asian T`, falls to `3.7e-8` at eighty-six seconds while the two
    /// terms the closed form used to subtract grow past `2.8e15`, which is
    /// how the last row came to be priced at 612 times its value.
    ///
    /// The bound asserted is `1e-9`; the measured worst case is `6.9e-12`, on
    /// the last row, and it comes from the `f64` normal CDF behind `big_n`
    /// rather than from the second moment.
    #[test]
    fn test_arithmetic_asian_short_maturity_matches_quadrature() {
        let reference = [
            (dec!(0.5), dec!(0.17185581541141069723)),
            (dec!(0.2), dec!(0.10837712084561396701)),
            (dec!(0.1), dec!(0.076521744332875558055)),
            (dec!(0.05), dec!(0.054052666108035095143)),
            (dec!(0.02), dec!(0.034154202088962484802)),
            (dec!(0.01), dec!(0.024139351973902131194)),
            (dec!(0.005), dec!(0.017063436757072324427)),
            (dec!(0.002), dec!(0.010788684989048071953)),
            (dec!(0.001), dec!(0.0076276185063713378631)),
        ];

        for (days, expected) in reference {
            let option = create_short_maturity_option(days, OptionStyle::Call);
            let price = asian_black_scholes(&option).unwrap();
            let error = relative_error(price, expected);
            assert!(
                error < dec!(1e-9),
                "{days} days priced {price} against {expected}, a relative {error}"
            );
        }
    }

    /// The moment-matched Asian is a Black formula on the forward `M1`, so it
    /// owes the ordinary parity `C - P = e^{-rT} (M1 - K)` whatever the
    /// matched volatility turns out to be. At eighty-six seconds `M1 - K` is
    /// `5.48e-6` on a `100` underlying, so the identity is a real constraint
    /// on the last three digits of both legs rather than a restatement of the
    /// formula: `e^{-rT} (M1 - K) = 5.4794510539814525e-6` at 50 digits.
    #[test]
    fn test_arithmetic_asian_short_maturity_holds_put_call_parity() {
        let call = asian_black_scholes(&create_short_maturity_option(
            dec!(0.001),
            OptionStyle::Call,
        ))
        .unwrap();
        let put = asian_black_scholes(&create_short_maturity_option(dec!(0.001), OptionStyle::Put))
            .unwrap();

        assert_decimal_eq!(call - put, dec!(0.0000054794510539814525), dec!(1e-18));
    }

    /// The series and the divided difference have to meet at the threshold
    /// they are separated by, otherwise the switch is a discontinuity in the
    /// price surface rather than a change of method.
    ///
    /// `u = max(|bT|, |cT|) = 0.12 T` here, so `1e-3` sits at `3.0416666…`
    /// days: `3.04166666` is summed and `3.0416667` is evaluated in closed
    /// form. The gap between them is `2.9e-9`, which is the whole of the
    /// genuine `dP/dT ≈ 25.7` across the `1.1e-10` years that separate the
    /// two maturities; the formulas themselves differ by less than `1e-20`.
    /// The tolerance is `1e-7`.
    #[test]
    fn test_arithmetic_asian_series_threshold_is_continuous() {
        let summed = asian_black_scholes(&create_short_maturity_option(
            dec!(3.04166666),
            OptionStyle::Call,
        ))
        .unwrap();
        let closed_form = asian_black_scholes(&create_short_maturity_option(
            dec!(3.0416667),
            OptionStyle::Call,
        ))
        .unwrap();

        assert_decimal_eq!(summed, closed_form, dec!(1e-7));
        assert!(
            summed < closed_form,
            "the price must stay monotone in T across the threshold: {summed} {closed_form}"
        );
    }

    /// Same check at the other threshold, where the divided difference gives
    /// way to the derivative at the midpoint.
    ///
    /// `y = aT` is `1.01e-8` at `q = 0.0399999899` and `9.9e-9` at
    /// `q = 0.0399999901`, so the pair straddles `1e-8` at `T = 1`. Their gap
    /// is `4.5e-9`, all of it the genuine `dP/dq ≈ 22.7` over the `2e-10`
    /// that separates the two yields. The tolerance is `1e-7`, and the exact
    /// boundary `q = 4%` has to sit between them.
    #[test]
    fn test_arithmetic_asian_midpoint_threshold_is_continuous() {
        let closed_form =
            asian_black_scholes(&create_boundary_option_at(dec!(0.0399999899), dec!(365.0)))
                .unwrap();
        let midpoint =
            asian_black_scholes(&create_boundary_option_at(dec!(0.0399999901), dec!(365.0)))
                .unwrap();
        let at = asian_black_scholes(&create_boundary_option_at(dec!(0.04), dec!(365.0))).unwrap();

        assert_decimal_eq!(closed_form, midpoint, dec!(1e-7));
        assert!(
            at < midpoint && midpoint < closed_form,
            "the price must stay monotone in q across the threshold: {closed_form} {midpoint} {at}"
        );
    }

    /// The two removable boundaries `#445` added — `b = -σ²`, which empties
    /// the inner integral, and `b = -σ²/2`, which empties the outer one — at
    /// maturities short enough to have broken them.
    ///
    /// They failed a decade or two later than the general branch rather than
    /// differently: both held to about `1e-4` days and then drifted onto the
    /// spot-volatility price, a factor `√3` too high, by `1e-8` days. Against
    /// the same 50-digit quadrature:
    ///
    /// | contract | maturity | reference | before |
    /// |---|---|---|---|
    /// | `b = -σ²` | 0.01 days | `0.0240845905435` | `+1.2e-10` |
    /// | `b = -σ²` | 0.001 days | `0.0076221400994` | `-5.7e-8` |
    /// | `b = -σ²/2` | 0.01 days | `0.0240982866889` | `-4.0e-10` |
    /// | `b = -σ²/2` | 0.001 days | `0.0076235098840` | `-5.1e-7` |
    ///
    /// Neither is a branch any more: `b = -σ²` is `z = x`, `b = -σ²/2` is
    /// `z = 0`, and at these maturities both land in the series, which never
    /// divides by the rate that vanished.
    #[test]
    fn test_arithmetic_asian_removable_boundaries_hold_at_short_maturity() {
        let reference = [
            (dec!(0.04), dec!(0.01), dec!(0.024084590543502699977)),
            (dec!(0.04), dec!(0.001), dec!(0.0076221400994210520537)),
            (dec!(0.02), dec!(0.01), dec!(0.024098286688876302321)),
            (dec!(0.02), dec!(0.001), dec!(0.0076235098840218887521)),
        ];

        for (dividend_yield, days, expected) in reference {
            let option = create_boundary_option_at(dividend_yield, days);
            let price = asian_black_scholes(&option).unwrap();
            let error = relative_error(price, expected);
            assert!(
                error < dec!(1e-9),
                "q = {dividend_yield} at {days} days priced {price} against {expected}, \
                 a relative {error}"
            );
        }
    }
}
