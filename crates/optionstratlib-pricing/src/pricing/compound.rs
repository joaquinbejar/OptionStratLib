/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

//! Compound option pricing module.
//!
//! Compound options are options on options (also called split-fee options).
//! The holder has the right to buy or sell an underlying option at a specified price.
//!
//! # Variants
//!
//! Based on the outer option style and underlying option style:
//! - **Call-on-Call**: Right to buy a call option
//! - **Call-on-Put**: Right to buy a put option
//! - **Put-on-Call**: Right to sell a call option
//! - **Put-on-Put**: Right to sell a put option
//!
//! # Formula
//!
//! The Geske (1979) closed form (Haug, *The Complete Guide to Option Pricing
//! Formulas*, §4.5): the discounted expected value of
//! `max(±(V(T1) - K1), 0)`, where `V` is the Black-Scholes value of the
//! underlying option at the compound's expiry `T1`. The critical spot at
//! which `V = K1` is solved by bisection, and the bivariate normal CDF is
//! Genz's algorithm.
//!
//! # Contract conventions
//!
//! `OptionType::Compound` gives the underlying no style, strike or expiry of
//! its own, so the underlying takes the compound's style and strike and
//! expires at `T2 = 2 T1`: a call is a call on a call and a put a put on a
//! put, at every maturity including expiry (#845).

use crate::error::PricingError;
use crate::kernels::{big_n, d1, d2, discount_factor};
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_exp, d_ln, d_mul, d_sqrt, d_sub, finite_decimal,
};
use optionstratlib_core::model::payoff::{Payoff, PayoffInfo};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use statrs::function::erf::erfc;
use std::f64::consts::PI;

/// Bivariate normal CDF: `P(X <= a, Y <= b)` for standard normals with
/// correlation `rho`, by Genz's algorithm (A. Genz, "Numerical computation
/// of rectangular bivariate and trivariate normal and t probabilities",
/// *Statistics and Computing* 14, 2004), the Drezner-Wesolowsky (1990)
/// method with Gauss-Legendre rules of 6, 12 or 20 points by `|rho|`,
/// accurate to about `1e-15`.
///
/// The quadrature it replaces (#845) integrated with five points and
/// mis-scaled weights: `M(0, 0; 0.5)` came out 0.2633 against the exact
/// 1/3, so every Geske price that reached it was wrong, and for large
/// arguments it multiplied an overflowing exponential by a vanishing one
/// and returned `NaN`.
///
/// # Errors
///
/// Returns [`PricingError::NonFinite`] when an argument has no finite `f64`
/// image, and propagates the `big_n` errors of the degenerate correlations.
fn bivariate_normal_cdf(a: Decimal, b: Decimal, rho: Decimal) -> Result<Decimal, PricingError> {
    // Convert to f64 for computation. A failed conversion is an error
    // (#639); it used to become `0.0`, i.e. `N2(0, 0; 0) = 0.25`.
    let to_f64 = |value: Decimal, context: &'static str| -> Result<f64, PricingError> {
        value
            .to_f64()
            .filter(|converted| converted.is_finite())
            .ok_or_else(|| PricingError::non_finite(context, f64::NAN))
    };
    let a_f = to_f64(a, "pricing::compound::bivariate::a")?;
    let b_f = to_f64(b, "pricing::compound::bivariate::b")?;
    let rho_f = to_f64(rho, "pricing::compound::bivariate::rho")?;

    // Handle special cases
    if rho_f.abs() < 1e-10 {
        // Independent case: P(X <= a, Y <= b) = N(a) * N(b)
        let n_a = big_n(a)?;
        let n_b = big_n(b)?;
        return Ok(d_mul(
            n_a,
            n_b,
            "pricing::compound::bivariate::independent",
        )?);
    }

    if rho_f >= 1.0 - 1e-10 {
        // Perfect correlation: P(X <= a, Y <= b) = N(min(a, b))
        let min_ab = a.min(b);
        return Ok(big_n(min_ab)?);
    }

    if rho_f <= -1.0 + 1e-10 {
        // Perfect negative correlation, `Y = -X`:
        // `P(X <= a, X >= -b) = max(N(a) - N(-b), 0)`. It used to return
        // `N(a)` whenever `a + b >= 0` (#845).
        return Ok(d_sub(
            big_n(a)?,
            big_n(-b)?,
            "pricing::compound::bivariate::perfect_negative",
        )?
        .max(Decimal::ZERO));
    }

    // Guard the f64 → Decimal boundary so a NaN / ±∞ surfaces a
    // `PricingError::NonFinite` instead of being clamped.
    let result = genz_upper_bivariate_normal(-a_f, -b_f, rho_f);
    let result_dec = finite_decimal(result).ok_or_else(|| {
        PricingError::non_finite("pricing::compound::bivariate_normal_cdf", result)
    })?;
    Ok(result_dec.max(Decimal::ZERO).min(Decimal::ONE))
}

/// Gauss-Legendre weights and abscissas on `[-1, 1]` (the positive half) of
/// Genz's `bvnu`: 6 points for `|rho| < 0.3`, 12 below 0.75, 20 above.
const GAUSS_LEGENDRE_6: ([f64; 3], [f64; 3]) = (
    [
        0.171_324_492_379_170_5,
        0.360_761_573_048_138_4,
        0.467_913_934_572_690_4,
    ],
    [
        0.932_469_514_203_152_2,
        0.661_209_386_466_264_7,
        0.238_619_186_083_197,
    ],
);
const GAUSS_LEGENDRE_12: ([f64; 6], [f64; 6]) = (
    [
        0.047_175_336_386_511_77,
        0.106_939_325_995_318_3,
        0.160_078_328_543_346_4,
        0.203_167_426_723_065_9,
        0.233_492_536_538_354_7,
        0.249_147_045_813_402_9,
    ],
    [
        0.981_560_634_246_719_1,
        0.904_117_256_370_475,
        0.769_902_674_194_305,
        0.587_317_954_286_617_1,
        0.367_831_498_998_180_2,
        0.125_233_408_511_469_2,
    ],
);
const GAUSS_LEGENDRE_20: ([f64; 10], [f64; 10]) = (
    [
        0.017_614_007_139_152_12,
        0.040_601_429_800_386_94,
        0.062_672_048_334_109_06,
        0.083_276_741_576_704_75,
        0.101_930_119_817_240_4,
        0.118_194_531_961_518_4,
        0.131_688_638_449_176_6,
        0.142_096_109_318_382_1,
        0.149_172_986_472_603_7,
        0.152_753_387_130_725_9,
    ],
    [
        0.993_128_599_185_094_9,
        0.963_971_927_277_913_8,
        0.912_234_428_251_326,
        0.839_116_971_822_218_8,
        0.746_331_906_460_150_8,
        0.636_053_680_726_515,
        0.510_867_001_950_827_1,
        0.373_706_088_715_419_6,
        0.227_785_851_141_645_1,
        0.076_526_521_133_497_33,
    ],
);

/// `e^x` in `f64`.
#[inline]
fn exp_f64(x: f64) -> f64 {
    x.exp() // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

/// `√x` in `f64`.
#[inline]
fn sqrt_f64(x: f64) -> f64 {
    x.sqrt() // scan-banned: allow -- f64 `sqrt`: returns NaN for negative input, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

/// `sin x` in `f64`.
#[inline]
fn sin_f64(x: f64) -> f64 {
    x.sin() // scan-banned: allow -- f64 `sin`: total, it does not abort
}

/// Genz's `bvnu`: `P(X > h, Y > k)` for standard normals with correlation
/// `r`, `|r| < 1`. A transcription of Genz's published MATLAB routine.
fn genz_upper_bivariate_normal(h: f64, k: f64, r: f64) -> f64 {
    let two_pi = 2.0 * PI;
    let (weights, abscissas): (&[f64], &[f64]) = if r.abs() < 0.3 {
        (&GAUSS_LEGENDRE_6.0, &GAUSS_LEGENDRE_6.1)
    } else if r.abs() < 0.75 {
        (&GAUSS_LEGENDRE_12.0, &GAUSS_LEGENDRE_12.1)
    } else {
        (&GAUSS_LEGENDRE_20.0, &GAUSS_LEGENDRE_20.1)
    };
    let hk = h * k;
    let bvn = if r.abs() < 0.925 {
        let hs = (h * h + k * k) / 2.0;
        let asr = r.asin() / 2.0;
        let mut sum = 0.0;
        for (&weight, &abscissa) in weights.iter().zip(abscissas) {
            for node in [1.0 - abscissa, 1.0 + abscissa] {
                let sn = sin_f64(asr * node);
                sum += weight * exp_f64((sn * hk - hs) / (1.0 - sn * sn));
            }
        }
        sum * asr / two_pi + standard_normal_cdf(-h) * standard_normal_cdf(-k)
    } else {
        let (k, hk) = if r < 0.0 { (-k, -hk) } else { (k, hk) };
        let mut bvn = 0.0;
        if r.abs() < 1.0 {
            let one_minus_r2 = 1.0 - r * r;
            let a = sqrt_f64(one_minus_r2);
            let bs = (h - k) * (h - k);
            let asr = -(bs / one_minus_r2 + hk) / 2.0;
            let c = (4.0 - hk) / 8.0;
            let d = (12.0 - hk) / 80.0;
            if asr > -100.0 {
                bvn = a
                    * exp_f64(asr)
                    * (1.0 - c * (bs - one_minus_r2) * (1.0 - d * bs) / 3.0
                        + c * d * one_minus_r2 * one_minus_r2);
            }
            if hk > -100.0 {
                let b = sqrt_f64(bs);
                let sp = sqrt_f64(two_pi) * standard_normal_cdf(-b / a);
                bvn -= exp_f64(-hk / 2.0) * sp * b * (1.0 - c * bs * (1.0 - d * bs) / 3.0);
            }
            let half_a = a / 2.0;
            let mut sum = 0.0;
            for (&weight, &abscissa) in weights.iter().zip(abscissas) {
                for node in [1.0 - abscissa, 1.0 + abscissa] {
                    let xs = (half_a * node) * (half_a * node);
                    let asr = -(bs / xs + hk) / 2.0;
                    if asr > -100.0 {
                        let sp = 1.0 + c * xs * (1.0 + 5.0 * d * xs);
                        let rs = sqrt_f64(1.0 - xs);
                        let ep = exp_f64(-(hk / 2.0) * xs / ((1.0 + rs) * (1.0 + rs))) / rs;
                        sum += weight * exp_f64(asr) * (sp - ep);
                    }
                }
            }
            bvn = (half_a * sum - bvn) / two_pi;
        }
        if r > 0.0 {
            bvn + standard_normal_cdf(-h.max(k))
        } else if h >= k {
            -bvn
        } else {
            let l = if h < 0.0 {
                standard_normal_cdf(k) - standard_normal_cdf(h)
            } else {
                standard_normal_cdf(-h) - standard_normal_cdf(-k)
            };
            l - bvn
        }
    };
    bvn.clamp(0.0, 1.0)
}

/// Standard normal CDF (for internal use in bivariate calculation).
///
/// `N(x) = erfc(-x / √2) / 2`, the expression `statrs` evaluates behind
/// `big_n`, kept in `f64` inside this `f64` kernel. It is total: there is no
/// `Decimal` round trip left to fail (#639), where a failed one used to turn
/// into `N(x) = 0` or `0.5`.
#[inline]
fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / std::f64::consts::SQRT_2)
}

/// Prices a Compound option using Geske (1979) framework.
///
/// # Arguments
///
/// * `option` - The compound option to price. Must have `OptionType::Compound`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
///
/// # Errors
///
/// - [`PricingError::MethodError`] when `option` is not an
///   [`OptionType::Compound`] variant, when the expiration cannot be converted
///   to a year fraction, or when no spot is found where an underlying put
///   falls below the compound strike.
/// - [`PricingError::Greeks`] when the `d1` / `d2` kernels reject the
///   inputs.
/// - [`PricingError::NonFinite`] when the Drezner-Wesolowsky bivariate
///   quadrature produces a non-finite value.
/// - [`PricingError::Positive`] when the derived `T2 = 2·T1` maturity is not
///   representable as a `Positive`.
/// - [`PricingError::Decimal`] when an intermediate step leaves the
///   representable `Decimal` range, or when `σ√T1` or the critical-price
///   moneyness collapses to zero: the discount factors, the critical price,
///   `d1_t1` / `d2_t1`, or the final leg composition.
/// - Whatever the inner Black-Scholes valuation of the underlying option
///   propagates.
pub fn compound_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Compound { underlying_option } => price_compound(option, underlying_option),
        _ => Err(PricingError::other(
            "compound_black_scholes requires OptionType::Compound",
        )),
    }
}

/// Prices a compound option given the outer and underlying option types.
fn price_compound(
    compound: &Options,
    underlying_type: &OptionType,
) -> Result<Decimal, PricingError> {
    let s = compound.underlying_price;
    let k1 = compound.strike_price; // Strike of compound option
    let r = compound.risk_free_rate;
    let q = compound.dividend_yield.to_dec();
    let sigma = compound.implied_volatility;
    let t1 = compound
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t1 == Positive::ZERO {
        // At expiration of the compound the underlying, which expires at
        // `T2 = 2 T1 = 0` here, is worth its own payoff at the spot. Its
        // Black-Scholes value is undefined at `T = 0`; that failure used to
        // be read as a worthless underlying (#639), which priced every
        // compound put at its full strike.
        let underlying_value = underlying_payoff_at_expiry(compound, underlying_type)?;
        let intrinsic = match compound.option_style {
            OptionStyle::Call => d_sub(
                underlying_value,
                k1.to_dec(),
                "pricing::compound::intrinsic::call",
            )?
            .max(Decimal::ZERO),
            OptionStyle::Put => d_sub(
                k1.to_dec(),
                underlying_value,
                "pricing::compound::intrinsic::put",
            )?
            .max(Decimal::ZERO),
        };
        return Ok(apply_side(intrinsic, compound));
    }

    // The underlying expires at `T2 = 2 T1` and takes the compound's style
    // and strike: `OptionType::Compound` carries neither for it, and this is
    // the convention the expiry branch above (#639) prices. A call is a call
    // on a call and a put a put on a put, at every volatility including zero
    // (#867). The closed form used to value every underlying as a call (a put
    // was priced as a put on a call) and to approximate the critical price,
    // so it disagreed with the value at expiry by about 38 on a put (#845).
    let t2 = Positive::new_decimal(d_mul(t1.to_dec(), dec!(2), "pricing::compound::t2")?)?;
    let is_call = matches!(compound.option_style, OptionStyle::Call);
    let price = geske_price(&GeskeInputs {
        s,
        k1,
        k2: compound.strike_price,
        t1,
        t2,
        market: Market { r, q, sigma },
        outer_call: is_call,
        underlying_call: is_call,
    })?;
    Ok(apply_side(price, compound))
}

/// Inputs of the Geske (1979) compound-option closed form, Haug, *The
/// Complete Guide to Option Pricing Formulas*, §4.5: an option struck at
/// `k1` expiring at `t1` on a European option struck at `k2` expiring at
/// `t2 > t1`, on a spot `s` with yield `q`, rate `r` and volatility
/// `sigma`.
struct GeskeInputs {
    s: Positive,
    k1: Positive,
    k2: Positive,
    t1: Positive,
    t2: Positive,
    market: Market,
    outer_call: bool,
    underlying_call: bool,
}

/// The rate `r`, the yield `q` and the volatility `σ` shared by the
/// compound and its underlying.
#[derive(Clone, Copy)]
struct Market {
    r: Decimal,
    q: Decimal,
    sigma: Positive,
}

impl Market {
    /// The cost of carry `b = r - q`.
    fn carry(&self) -> Result<Decimal, PricingError> {
        Ok(d_sub(self.r, self.q, "pricing::compound::carry")?)
    }
}

/// Bisection steps for the critical price: each halves the bracket, so 100
/// take it far below the 28 places a `Decimal` carries.
const CRITICAL_PRICE_ITERATIONS: usize = 100;

/// Doublings tried to find a spot where an underlying put is worth less than
/// the compound strike; the put falls below any positive strike long before
/// the spot doubles this many times.
const CRITICAL_PRICE_DOUBLINGS: usize = 200;

/// The Geske price of a long compound option, floored at zero.
///
/// With `I` the critical spot at which the underlying is worth `k1` at
/// `t1`, `b = r - q`, `ρ = √(t1 / t2)`,
/// `z1 = [ln(S / I) + (b + σ²/2) t1] / (σ√t1)`, `z2 = z1 - σ√t1`,
/// `y1 = [ln(S / k2) + (b + σ²/2) t2] / (σ√t2)` and `y2 = y1 - σ√t2`:
///
/// - call on call: `S e^(-q t2) M(z1, y1; ρ) - k2 e^(-r t2) M(z2, y2; ρ) - k1 e^(-r t1) N(z2)`
/// - put on call: `k2 e^(-r t2) M(-z2, y2; -ρ) - S e^(-q t2) M(-z1, y1; -ρ) + k1 e^(-r t1) N(-z2)`
/// - call on put: `k2 e^(-r t2) M(-z2, -y2; ρ) - S e^(-q t2) M(-z1, -y1; ρ) - k1 e^(-r t1) N(-z2)`
/// - put on put: `S e^(-q t2) M(z1, -y1; -ρ) - k2 e^(-r t2) M(z2, -y2; -ρ) + k1 e^(-r t1) N(z2)`
///
/// At `σ = 0` the path is deterministic: with `F = S e^(b t2)`, the
/// underlying is worth `V = e^(-r (t2 - t1)) max(±(F - k2), 0)` at `t1`, and
/// the compound `e^(-r t1) max(±(V - k1), 0)`.
///
/// When no critical spot exists the decision at `t1` is the same on every
/// path: a zero `k1` is always exercised by a call on the underlying and
/// never by a put, and a put underlying worth less than `k1` at every spot
/// (`k1 ≥ k2 e^(-r (t2 - t1))`) makes the compound put a forward on it and
/// the compound call worthless.
///
/// # Errors
///
/// Returns [`PricingError`] when an intermediate step leaves the `Decimal`
/// range, when the underlying's `d1` / `d2` reject the inputs, or when the
/// bivariate quadrature is not finite.
fn geske_price(inputs: &GeskeInputs) -> Result<Decimal, PricingError> {
    let GeskeInputs {
        s,
        k1,
        k2,
        t1,
        t2,
        market,
        outer_call,
        underlying_call,
    } = *inputs;
    let Market { r, q, sigma } = market;
    let b = market.carry()?;
    let tau = Positive::new_decimal(d_sub(t2.to_dec(), t1.to_dec(), "pricing::compound::tau")?)?;

    let discount_t1 = discount_factor(
        r,
        t1.to_dec(),
        "pricing::compound::neg_rt1",
        "pricing::compound::discount_t1",
    )?;
    let k1_pv = d_mul(k1.to_dec(), discount_t1, "pricing::compound::k1_pv")?;
    let underlying_now = || european_value(&market, s, k2, t2, underlying_call);

    // Without volatility the path is deterministic: the spot reaches the
    // forward, and the underlying is worth its intrinsic on the forward to
    // its own expiry, discounted over the remaining `tau`. This is the
    // `σ → 0` limit of the closed form below, whose `d` terms divide by `σ`
    // (#867): the branch used to value the underlying through Black-Scholes
    // at `σ = 0`, which rejects it, so every such compound was an error.
    if sigma == Positive::ZERO {
        let forward_t2 = d_mul(
            s.to_dec(),
            d_exp(
                d_mul(b, t2.to_dec(), "pricing::compound::zero_vol::carry_t2")?,
                "pricing::compound::zero_vol::growth",
            )?,
            "pricing::compound::zero_vol::forward",
        )?;
        let underlying_intrinsic = if underlying_call {
            d_sub(
                forward_t2,
                k2.to_dec(),
                "pricing::compound::zero_vol::underlying_call",
            )?
        } else {
            d_sub(
                k2.to_dec(),
                forward_t2,
                "pricing::compound::zero_vol::underlying_put",
            )?
        }
        .max(Decimal::ZERO);
        let underlying_at_t1 = d_mul(
            underlying_intrinsic,
            discount_factor(
                r,
                tau.to_dec(),
                "pricing::compound::zero_vol::neg_r_tau",
                "pricing::compound::zero_vol::discount_tau",
            )?,
            "pricing::compound::zero_vol::underlying",
        )?;
        let compound_intrinsic = if outer_call {
            d_sub(
                underlying_at_t1,
                k1.to_dec(),
                "pricing::compound::zero_vol::call",
            )?
        } else {
            d_sub(
                k1.to_dec(),
                underlying_at_t1,
                "pricing::compound::zero_vol::put",
            )?
        }
        .max(Decimal::ZERO);
        return Ok(d_mul(
            compound_intrinsic,
            discount_t1,
            "pricing::compound::zero_vol::price",
        )?);
    }

    let critical = if k1 == Positive::ZERO {
        None
    } else {
        critical_price(&market, k1, k2, tau, underlying_call)?
    };
    let Some(critical) = critical else {
        // The exercise decision at `t1` does not depend on the path.
        let exercised = if k1 == Positive::ZERO {
            outer_call
        } else {
            // Only a put underlying can lack a critical spot: it is worth
            // less than `k1` everywhere, so only the compound put exercises.
            !outer_call
        };
        if !exercised {
            return Ok(Decimal::ZERO);
        }
        let value = if outer_call {
            d_sub(underlying_now()?, k1_pv, "pricing::compound::forward::call")?
        } else {
            d_sub(k1_pv, underlying_now()?, "pricing::compound::forward::put")?
        };
        return Ok(value.max(Decimal::ZERO));
    };

    let sigma_dec = sigma.to_dec();
    let sqrt_t1 = d_sqrt(t1.to_dec(), "pricing::compound::sqrt_t1")?;
    let sigma_sqrt_t1 = d_mul(sigma_dec, sqrt_t1, "pricing::compound::sigma_sqrt_t1")?;
    let rho = d_sqrt(
        d_div(t1.to_dec(), t2.to_dec(), "pricing::compound::rho_ratio")?,
        "pricing::compound::rho",
    )?;
    let critical_moneyness = d_div(
        s.to_dec(),
        critical,
        "pricing::compound::critical_moneyness",
    )?;
    let z1 = d_div(
        d_add(
            d_ln(
                critical_moneyness,
                "pricing::compound::log_critical_moneyness",
            )?,
            d_mul(
                d_add(
                    b,
                    d_div(
                        d_mul(sigma_dec, sigma_dec, "pricing::compound::variance")?,
                        dec!(2),
                        "pricing::compound::half_variance",
                    )?,
                    "pricing::compound::drift_rate",
                )?,
                t1.to_dec(),
                "pricing::compound::drift",
            )?,
            "pricing::compound::z1_numerator",
        )?,
        sigma_sqrt_t1,
        "pricing::compound::z1",
    )?;
    let z2 = d_sub(z1, sigma_sqrt_t1, "pricing::compound::z2")?;
    let y1 = d1(s, k2, b, t2, sigma)?;
    let y2 = d2(s, k2, b, t2, sigma)?;

    let discount_t2 = discount_factor(
        r,
        t2.to_dec(),
        "pricing::compound::neg_rt2",
        "pricing::compound::discount_t2",
    )?;
    let dividend_discount_t2 = discount_factor(
        q,
        t2.to_dec(),
        "pricing::compound::neg_qt2",
        "pricing::compound::dividend_discount_t2",
    )?;
    let s_pv = d_mul(s.to_dec(), dividend_discount_t2, "pricing::compound::s_pv")?;
    let k2_pv = d_mul(k2.to_dec(), discount_t2, "pricing::compound::k2_pv")?;
    let leg = |base: Decimal, weight: Decimal, op: &'static str| -> Result<Decimal, PricingError> {
        Ok(d_mul(base, weight, op)?)
    };

    let price = match (outer_call, underlying_call) {
        (true, true) => {
            let spot = leg(
                s_pv,
                bivariate_normal_cdf(z1, y1, rho)?,
                "pricing::compound::cc::s",
            )?;
            let strike2 = leg(
                k2_pv,
                bivariate_normal_cdf(z2, y2, rho)?,
                "pricing::compound::cc::k2",
            )?;
            let strike1 = leg(k1_pv, big_n(z2)?, "pricing::compound::cc::k1")?;
            d_sub(
                d_sub(spot, strike2, "pricing::compound::cc::step")?,
                strike1,
                "pricing::compound::cc",
            )?
        }
        (false, true) => {
            let strike2 = leg(
                k2_pv,
                bivariate_normal_cdf(-z2, y2, -rho)?,
                "pricing::compound::pc::k2",
            )?;
            let spot = leg(
                s_pv,
                bivariate_normal_cdf(-z1, y1, -rho)?,
                "pricing::compound::pc::s",
            )?;
            let strike1 = leg(k1_pv, big_n(-z2)?, "pricing::compound::pc::k1")?;
            d_add(
                d_sub(strike2, spot, "pricing::compound::pc::step")?,
                strike1,
                "pricing::compound::pc",
            )?
        }
        (true, false) => {
            let strike2 = leg(
                k2_pv,
                bivariate_normal_cdf(-z2, -y2, rho)?,
                "pricing::compound::cp::k2",
            )?;
            let spot = leg(
                s_pv,
                bivariate_normal_cdf(-z1, -y1, rho)?,
                "pricing::compound::cp::s",
            )?;
            let strike1 = leg(k1_pv, big_n(-z2)?, "pricing::compound::cp::k1")?;
            d_sub(
                d_sub(strike2, spot, "pricing::compound::cp::step")?,
                strike1,
                "pricing::compound::cp",
            )?
        }
        (false, false) => {
            let spot = leg(
                s_pv,
                bivariate_normal_cdf(z1, -y1, -rho)?,
                "pricing::compound::pp::s",
            )?;
            let strike2 = leg(
                k2_pv,
                bivariate_normal_cdf(z2, -y2, -rho)?,
                "pricing::compound::pp::k2",
            )?;
            let strike1 = leg(k1_pv, big_n(z2)?, "pricing::compound::pp::k1")?;
            d_add(
                d_sub(spot, strike2, "pricing::compound::pp::step")?,
                strike1,
                "pricing::compound::pp",
            )?
        }
    };
    Ok(price.max(Decimal::ZERO))
}

/// Black-Scholes-Merton value of a European call or put with carry `b`.
///
/// # Errors
///
/// Returns [`PricingError`] when `d1` / `d2` reject the inputs or a step
/// leaves the `Decimal` range.
fn european_value(
    market: &Market,
    s: Positive,
    k: Positive,
    t: Positive,
    call: bool,
) -> Result<Decimal, PricingError> {
    let Market { r, q, sigma } = *market;
    let b = market.carry()?;
    let d1_value = d1(s, k, b, t, sigma)?;
    let d2_value = d2(s, k, b, t, sigma)?;
    let spot_pv = d_mul(
        s.to_dec(),
        discount_factor(
            q,
            t.to_dec(),
            "pricing::compound::bs::neg_qt",
            "pricing::compound::bs::exp_qt",
        )?,
        "pricing::compound::bs::spot_pv",
    )?;
    let strike_pv = d_mul(
        k.to_dec(),
        discount_factor(
            r,
            t.to_dec(),
            "pricing::compound::bs::neg_rt",
            "pricing::compound::bs::exp_rt",
        )?,
        "pricing::compound::bs::strike_pv",
    )?;
    Ok(if call {
        d_sub(
            d_mul(
                spot_pv,
                big_n(d1_value)?,
                "pricing::compound::bs::call_spot",
            )?,
            d_mul(
                strike_pv,
                big_n(d2_value)?,
                "pricing::compound::bs::call_strike",
            )?,
            "pricing::compound::bs::call",
        )?
    } else {
        d_sub(
            d_mul(
                strike_pv,
                big_n(-d2_value)?,
                "pricing::compound::bs::put_strike",
            )?,
            d_mul(
                spot_pv,
                big_n(-d1_value)?,
                "pricing::compound::bs::put_spot",
            )?,
            "pricing::compound::bs::put",
        )?
    })
}

/// The critical spot `I` at which the underlying, `tau` before its expiry,
/// is worth `k1`: solved by bisection, as the Geske formula requires. It
/// used to be approximated as the forward scaled by `1 ± 0.4 σ√t1` (#845).
///
/// A call grows from zero without bound, so for `k1 > 0` the root lies in
/// `(0, (k1 + k2 e^(-r tau)) e^(q tau)]`, where the call is at least `k1`.
/// A put falls from `k2 e^(-r tau)` to zero, so it has a root only when
/// `k1` is below that bound; `None` reports that it has none.
///
/// # Errors
///
/// Returns [`PricingError`] when a valuation fails or no spot is found
/// where the put falls below `k1`.
fn critical_price(
    market: &Market,
    k1: Positive,
    k2: Positive,
    tau: Positive,
    underlying_call: bool,
) -> Result<Option<Decimal>, PricingError> {
    let Market { r, q, .. } = *market;
    let value_at = |spot: Decimal| -> Result<Decimal, PricingError> {
        european_value(
            market,
            Positive::new_decimal(spot)?,
            k2,
            tau,
            underlying_call,
        )
    };
    let k2_pv = d_mul(
        k2.to_dec(),
        discount_factor(
            r,
            tau.to_dec(),
            "pricing::compound::critical::neg_rt",
            "pricing::compound::critical::exp_rt",
        )?,
        "pricing::compound::critical::k2_pv",
    )?;
    let mut high = if underlying_call {
        d_mul(
            d_add(
                k1.to_dec(),
                k2_pv,
                "pricing::compound::critical::call_bound",
            )?,
            d_exp(
                d_mul(q, tau.to_dec(), "pricing::compound::critical::qt")?,
                "pricing::compound::critical::growth",
            )?,
            "pricing::compound::critical::call_high",
        )?
    } else {
        if k1.to_dec() >= k2_pv {
            return Ok(None);
        }
        let mut high = k2.to_dec();
        let mut found = false;
        for _ in 0..CRITICAL_PRICE_DOUBLINGS {
            if value_at(high)? < k1.to_dec() {
                found = true;
                break;
            }
            high = d_mul(high, dec!(2), "pricing::compound::critical::double")?;
        }
        if !found {
            return Err(PricingError::method_error(
                "compound_black_scholes",
                "no spot found where the underlying put falls below the compound strike",
            ));
        }
        high
    };
    let mut low = Decimal::ZERO;
    for _ in 0..CRITICAL_PRICE_ITERATIONS {
        let mid = d_div(
            d_add(low, high, "pricing::compound::critical::sum")?,
            dec!(2),
            "pricing::compound::critical::mid",
        )?;
        if mid == low || mid == high {
            break;
        }
        // The call increases with the spot and the put decreases.
        let above = value_at(mid)? > k1.to_dec();
        if above == underlying_call {
            high = mid;
        } else {
            low = mid;
        }
    }
    let mid = d_div(
        d_add(low, high, "pricing::compound::critical::final_sum")?,
        dec!(2),
        "pricing::compound::critical::root",
    )?;
    Ok(Some(mid))
}

/// Long payoff of the underlying option at the compound's spot, its value at
/// its own expiry.
///
/// # Errors
///
/// Returns [`PricingError::Options`] when the payoff is not representable
/// as a `Decimal`.
fn underlying_payoff_at_expiry(
    compound: &Options,
    underlying_type: &OptionType,
) -> Result<Decimal, PricingError> {
    Ok(underlying_type.payoff(&PayoffInfo {
        spot: compound.underlying_price,
        strike: compound.strike_price,
        style: compound.option_style,
        side: Side::Long,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
        exotic_params: None,
    })?)
}

/// Applies the side (long/short) multiplier to the price.
fn apply_side(price: Decimal, option: &Options) -> Decimal {
    match option.side {
        optionstratlib_core::model::types::Side::Long => price,
        optionstratlib_core::model::types::Side::Short => -price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_compound_option(style: OptionStyle, underlying_type: OptionType) -> Options {
        Options::new(
            OptionType::Compound {
                underlying_option: Box::new(underlying_type),
            },
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(5.0),                         // strike of compound (K1)
            ExpirationDate::Days(pos_or_panic!(91.25)), // ~0.25 years (T1)
            pos_or_panic!(0.25),                        // volatility
            Positive::ONE,                              // quantity
            Positive::HUNDRED,                          // underlying
            dec!(0.05),                                 // risk-free rate
            style,
            Positive::ZERO, // dividend yield
            None,
        )
    }

    /// At the compound's expiry the underlying, which expires with it in this
    /// approximation and takes the compound's style and strike, is worth its
    /// payoff. `S = 100, K = 5`: the underlying call is worth `95`, so the
    /// call-on-call is `90`; the underlying put is worth `0`, so the
    /// put-on-put is `5`. The failed Black-Scholes valuation of the expired
    /// underlying used to be read as zero, pricing the call-on-call at `0`
    /// (#639).
    #[test]
    fn test_compound_at_expiry_uses_the_underlying_payoff() {
        let mut call_on_call = create_compound_option(OptionStyle::Call, OptionType::European);
        call_on_call.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let mut put_on_put = create_compound_option(OptionStyle::Put, OptionType::European);
        put_on_put.expiration_date = ExpirationDate::Days(Positive::ZERO);
        assert_eq!(compound_black_scholes(&call_on_call).unwrap(), dec!(90));
        assert_eq!(compound_black_scholes(&put_on_put).unwrap(), dec!(5));
    }

    /// The bivariate kernel's normal CDF is total in `f64`: there is no
    /// `Decimal` round trip left to fail and fall back to `0` or `0.5`
    /// (#639).
    #[test]
    fn test_standard_normal_cdf_is_total() {
        assert_eq!(standard_normal_cdf(0.0), 0.5);
        // `statrs`'s `erfc` is accurate to about `1e-12` here (it returns
        // `0.975000000001123`), the same kernel `big_n` evaluates.
        let p = standard_normal_cdf(1.959963984540054);
        assert!((p - 0.975).abs() < 1e-11, "{p}");
        assert_eq!(
            Decimal::from_f64(p).unwrap(),
            big_n(dec!(1.959963984540054)).unwrap()
        );
        assert_eq!(standard_normal_cdf(f64::INFINITY), 1.0);
        assert_eq!(standard_normal_cdf(f64::NEG_INFINITY), 0.0);
        assert!((standard_normal_cdf(-40.0)).abs() < 1e-300);
    }

    #[test]
    fn test_bivariate_normal_independent() {
        // When rho=0, M(a,b,0) = N(a)*N(b)
        let a = dec!(0.0);
        let b = dec!(0.0);
        let rho = dec!(0.0);
        let result = bivariate_normal_cdf(a, b, rho).expect("finite CDF");
        // N(0)*N(0) = 0.5 * 0.5 = 0.25
        assert!(
            (result - dec!(0.25)).abs() < dec!(0.01),
            "Independent result: {}",
            result
        );
    }

    #[test]
    fn test_bivariate_normal_perfect_correlation() {
        // When rho=1, M(a,b,1) = N(min(a,b))
        let a = dec!(1.0);
        let b = dec!(0.5);
        let rho = dec!(0.999);
        let result = bivariate_normal_cdf(a, b, rho).expect("finite CDF");
        let n_min = big_n(b).unwrap_or(Decimal::ZERO);
        assert!(
            (result - n_min).abs() < dec!(0.1),
            "Perfect correlation result: {} vs N(0.5)={}",
            result,
            n_min
        );
    }

    #[test]
    fn test_call_on_call() {
        let option = create_compound_option(OptionStyle::Call, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        // Call-on-call should have positive value
        assert!(
            price > Decimal::ZERO,
            "Call-on-call should be positive: {}",
            price
        );
    }

    #[test]
    fn test_call_on_put() {
        let mut option = create_compound_option(OptionStyle::Call, OptionType::European);
        option.option_style = OptionStyle::Put; // Make compound option style be call but underlying behave as put
        // Actually for call-on-put, keep compound as Call but indicate underlying is put-like
        let option = create_compound_option(OptionStyle::Call, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        assert!(
            price >= Decimal::ZERO,
            "Call-on-put should be non-negative: {}",
            price
        );
    }

    #[test]
    fn test_put_on_call() {
        let option = create_compound_option(OptionStyle::Put, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        assert!(
            price >= Decimal::ZERO,
            "Put-on-call should be non-negative: {}",
            price
        );
    }

    #[test]
    fn test_put_on_put() {
        let option = create_compound_option(OptionStyle::Put, OptionType::European);
        let price = compound_black_scholes(&option).unwrap();
        assert!(
            price >= Decimal::ZERO,
            "Put-on-put should be non-negative: {}",
            price
        );
    }

    #[test]
    fn test_short_compound_option() {
        let mut option = create_compound_option(OptionStyle::Call, OptionType::European);
        let long_price = compound_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = compound_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let mut option = create_compound_option(OptionStyle::Call, OptionType::European);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = compound_black_scholes(&option).unwrap();
        // At expiry, intrinsic value
        assert!(price >= Decimal::ZERO, "Zero time result: {}", price);
    }

    #[test]
    fn test_compound_value_reasonable() {
        // Compound option should have reasonable value relative to parameters
        let compound = create_compound_option(OptionStyle::Call, OptionType::European);
        let compound_price = compound_black_scholes(&compound).unwrap();

        // Compound option with K1=5 on underlying worth ~8-10 should have significant value
        // but less than the underlying itself
        assert!(
            compound_price > Decimal::ZERO,
            "Compound should be positive: {}",
            compound_price
        );

        // For a call-on-call, expected range is typically small to moderate
        assert!(
            compound_price < dec!(200.0),
            "Compound price {} seems too high",
            compound_price
        );
    }

    #[test]
    fn test_higher_compound_strike_means_lower_call_value() {
        let low_strike = create_compound_option(OptionStyle::Call, OptionType::European);
        let low_strike_price = compound_black_scholes(&low_strike).unwrap();

        let mut high_strike = low_strike.clone();
        high_strike.strike_price = pos_or_panic!(10.0);
        let high_strike_price = compound_black_scholes(&high_strike).unwrap();

        assert!(
            low_strike_price >= high_strike_price,
            "Lower compound strike should mean higher call value: {} vs {}",
            low_strike_price,
            high_strike_price
        );
    }

    /// Standard normal CDF in `f64` from `statrs`' `erfc`.
    fn normal_cdf_f64(x: f64) -> f64 {
        0.5 * erfc(-x / std::f64::consts::SQRT_2)
    }

    /// Black-Scholes-Merton in `f64`, the underlying of the reference below.
    fn bs_f64(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64, call: bool) -> f64 {
        let sst = sigma * t.sqrt();
        let d1 = ((s / k).ln() + (r - q + sigma * sigma / 2.0) * t) / sst;
        let d2 = d1 - sst;
        if call {
            s * (-q * t).exp() * normal_cdf_f64(d1) - k * (-r * t).exp() * normal_cdf_f64(d2)
        } else {
            k * (-r * t).exp() * normal_cdf_f64(-d2) - s * (-q * t).exp() * normal_cdf_f64(-d1)
        }
    }

    /// An evaluation of a compound option independent of the Geske closed
    /// form: `e^(-r t1) E[max(±(V(S_t1) - k1), 0)]` with `V` the
    /// Black-Scholes value of the underlying at `t1` and `S_t1` lognormal,
    /// integrated over the standard normal by Simpson's rule on `[-10, 10]`
    /// with 40 000 intervals.
    #[allow(clippy::too_many_arguments)]
    fn compound_by_quadrature(
        s: f64,
        k1: f64,
        k2: f64,
        t1: f64,
        t2: f64,
        r: f64,
        q: f64,
        sigma: f64,
        outer_call: bool,
        underlying_call: bool,
    ) -> f64 {
        const INTERVALS: usize = 40_000;
        let h = 20.0 / INTERVALS as f64;
        let mut total = 0.0;
        for i in 0..=INTERVALS {
            let z = -10.0 + i as f64 * h;
            let spot_t1 = s * ((r - q - sigma * sigma / 2.0) * t1 + sigma * t1.sqrt() * z).exp();
            let value = bs_f64(spot_t1, k2, t2 - t1, r, q, sigma, underlying_call);
            let payoff = if outer_call {
                (value - k1).max(0.0)
            } else {
                (k1 - value).max(0.0)
            };
            let weight = if i == 0 || i == INTERVALS {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            total += weight * payoff * (-z * z / 2.0).exp() / (2.0 * PI).sqrt();
        }
        (-r * t1).exp() * total * h / 3.0
    }

    fn geske(k1: f64, outer_call: bool, underlying_call: bool) -> Decimal {
        geske_price(&GeskeInputs {
            s: Positive::HUNDRED,
            k1: pos_or_panic!(k1),
            k2: Positive::HUNDRED,
            t1: pos_or_panic!(0.25),
            t2: Positive::ONE,
            market: Market {
                r: dec!(0.05),
                q: dec!(0.02),
                sigma: pos_or_panic!(0.25),
            },
            outer_call,
            underlying_call,
        })
        .unwrap()
    }

    /// The four Geske contracts with a solved critical price against the
    /// quadrature: `S = K2 = 100, K1 = 6, t1 = 0.25, t2 = 1, r = 5 %,
    /// q = 2 %, σ = 25 %` (#845).
    #[test]
    fn test_geske_matches_an_independent_quadrature() {
        for (outer_call, underlying_call) in
            [(true, true), (false, true), (true, false), (false, false)]
        {
            let reference = compound_by_quadrature(
                100.0,
                6.0,
                100.0,
                0.25,
                1.0,
                0.05,
                0.02,
                0.25,
                outer_call,
                underlying_call,
            );
            let price = geske(6.0, outer_call, underlying_call);
            let gap = (price.to_f64().unwrap() - reference).abs();
            assert!(
                gap < 1e-6,
                "outer call {outer_call}, underlying call {underlying_call}: {price} vs {reference}"
            );
        }
    }

    /// Put-call parity for compounds: a call on `V` minus a put on `V`, both
    /// struck at `k1`, is `V(S, t2) - k1 e^(-r t1)`.
    #[test]
    fn test_geske_compound_put_call_parity() {
        let discounted_k1 = 6.0 * (-0.05_f64 * 0.25).exp();
        for underlying_call in [true, false] {
            let underlying = bs_f64(100.0, 100.0, 1.0, 0.05, 0.02, 0.25, underlying_call);
            let call = geske(6.0, true, underlying_call).to_f64().unwrap();
            let put = geske(6.0, false, underlying_call).to_f64().unwrap();
            assert!(
                (call - put - (underlying - discounted_k1)).abs() < 1e-6,
                "underlying call {underlying_call}: {call} - {put} vs {underlying} - {discounted_k1}"
            );
        }
    }

    /// A put underlying never worth `k1` (`k1 ≥ k2 e^(-r (t2 - t1))`) has no
    /// critical price: the compound put is always exercised and the
    /// compound call never is.
    #[test]
    fn test_geske_put_underlying_without_critical_price() {
        let reference_put = compound_by_quadrature(
            100.0, 110.0, 100.0, 0.25, 1.0, 0.05, 0.02, 0.25, false, false,
        );
        let put = geske(110.0, false, false).to_f64().unwrap();
        assert!(
            (put - reference_put).abs() < 1e-6,
            "{put} vs {reference_put}"
        );
        assert_eq!(geske(110.0, true, false), Decimal::ZERO);
    }

    fn compound_at(style: OptionStyle, spot: f64, strike: f64, days: f64) -> Decimal {
        let mut option = create_compound_option(style, OptionType::European);
        option.underlying_price = pos_or_panic!(spot);
        option.strike_price = pos_or_panic!(strike);
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(days));
        compound_black_scholes(&option).unwrap()
    }

    /// The price tends to its value at expiry as `T1 → 0` (#845). With
    /// `S = 105, K = 100` the put on a put was priced near 62.3 at every
    /// maturity against 100 at expiry: the closed form valued the
    /// underlying as a call and approximated its critical price.
    #[test]
    fn test_compound_is_continuous_at_expiry() {
        for (spot, strike, call_at_expiry, put_at_expiry) in [
            (105.0, 100.0, dec!(0), dec!(100)),
            (100.0, 5.0, dec!(90), dec!(5)),
            (100.0, 105.0, dec!(0), dec!(100)),
        ] {
            assert_eq!(
                compound_at(OptionStyle::Call, spot, strike, 0.0),
                call_at_expiry
            );
            assert_eq!(
                compound_at(OptionStyle::Put, spot, strike, 0.0),
                put_at_expiry
            );
            let one_minute = 1.0 / 1440.0;
            for (style, at_expiry) in [
                (OptionStyle::Call, call_at_expiry),
                (OptionStyle::Put, put_at_expiry),
            ] {
                let near = compound_at(style, spot, strike, one_minute);
                assert!(
                    (near - at_expiry).abs() < dec!(0.0001),
                    "S={spot} K={strike} {style:?}: {near} one minute out, {at_expiry} at expiry"
                );
            }
        }
    }

    /// The issue's put on a put (`S = 105, K = 100`, 90 days, so the
    /// underlying expires at 180) priced through the public kernel against
    /// the quadrature: 94.7987, where it was about 62.3.
    #[test]
    fn test_compound_put_matches_the_quadrature_through_the_kernel() {
        let t1 = 90.0 / 365.0;
        let reference = compound_by_quadrature(
            105.0,
            100.0,
            100.0,
            t1,
            2.0 * t1,
            0.05,
            0.0,
            0.25,
            false,
            false,
        );
        let price = compound_at(OptionStyle::Put, 105.0, 100.0, 90.0);
        assert!(
            (price.to_f64().unwrap() - reference).abs() < 1e-6,
            "{price} vs {reference}"
        );
    }

    /// `P(X <= a, Y <= b)` by Simpson's rule on
    /// `∫_{-14}^{a} φ(x) N((b - ρx) / √(1 - ρ²)) dx`, independent of the
    /// Genz quadrature.
    fn bivariate_by_quadrature(a: f64, b: f64, rho: f64) -> f64 {
        const INTERVALS: usize = 200_000;
        let lower = -14.0;
        let h = (a - lower) / INTERVALS as f64;
        let scale = (1.0 - rho * rho).sqrt();
        let mut total = 0.0;
        for i in 0..=INTERVALS {
            let x = lower + i as f64 * h;
            let weight = if i == 0 || i == INTERVALS {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            total += weight * (-x * x / 2.0).exp() / (2.0 * PI).sqrt()
                * normal_cdf_f64((b - rho * x) / scale);
        }
        total * h / 3.0
    }

    /// The bivariate CDF against the quadrature across the three Genz rules
    /// and both signs of the correlation. The previous five-point rule gave
    /// `M(0, 0; 0.5) = 0.2633` for the exact 1/3 (#845).
    #[test]
    fn test_bivariate_normal_matches_an_independent_quadrature() {
        for (a, b, rho) in [
            (0.0, 0.0, 0.5),
            (0.696, 0.245, 0.5),
            (1.0, -0.5, 0.7),
            (-1.0, 0.3, -0.7),
            (0.5, 0.5, 0.9),
            (0.2, -0.3, 0.3),
            (0.2, -0.3, -0.2),
            (1.2, 0.4, -0.95),
            (-0.4, 0.8, 0.97),
            (3.0, -2.0, 0.1),
        ] {
            let value = bivariate_normal_cdf(
                Decimal::from_f64(a).unwrap(),
                Decimal::from_f64(b).unwrap(),
                Decimal::from_f64(rho).unwrap(),
            )
            .unwrap()
            .to_f64()
            .unwrap();
            let reference = bivariate_by_quadrature(a, b, rho);
            assert!(
                (value - reference).abs() < 1e-9,
                "M({a}, {b}; {rho}) = {value}, quadrature {reference}"
            );
        }
        let third = bivariate_normal_cdf(Decimal::ZERO, Decimal::ZERO, dec!(0.5)).unwrap();
        assert!((third - dec!(0.3333333333333333)).abs() < dec!(0.000000000000001));
    }

    /// With `ρ = -1`, `Y = -X`: `P(X <= a, X >= -b) = max(N(a) - N(-b), 0)`.
    #[test]
    fn test_bivariate_normal_perfect_negative_correlation() {
        let both = bivariate_normal_cdf(dec!(1), dec!(0.5), dec!(-1)).unwrap();
        let expected = big_n(dec!(1)).unwrap() - big_n(dec!(-0.5)).unwrap();
        assert_eq!(both, expected);
        assert_eq!(
            bivariate_normal_cdf(dec!(-1), dec!(0.5), dec!(-1)).unwrap(),
            Decimal::ZERO
        );
    }

    fn geske_at(k1: f64, sigma: Positive, outer_call: bool, underlying_call: bool) -> Decimal {
        geske_price(&GeskeInputs {
            s: Positive::HUNDRED,
            k1: pos_or_panic!(k1),
            k2: Positive::HUNDRED,
            t1: pos_or_panic!(0.25),
            t2: Positive::ONE,
            market: Market {
                r: dec!(0.05),
                q: dec!(0.02),
                sigma,
            },
            outer_call,
            underlying_call,
        })
        .unwrap()
    }

    /// The deterministic compound on `S = K2 = 100, t1 = 0.25, t2 = 1,
    /// r = 5 %, q = 2 %`: `F = 100 e^(0.03)`, the underlying at `t1` is
    /// `e^(-0.05 · 0.75) max(±(F - 100), 0)` and the compound
    /// `e^(-0.05 · 0.25) max(±(V - K1), 0)`.
    fn deterministic(k1: f64, outer_call: bool, underlying_call: bool) -> f64 {
        let forward = 100.0 * 0.03_f64.exp();
        let intrinsic = if underlying_call {
            (forward - 100.0).max(0.0)
        } else {
            (100.0 - forward).max(0.0)
        };
        let underlying = (-0.05_f64 * 0.75).exp() * intrinsic;
        let payoff = if outer_call {
            (underlying - k1).max(0.0)
        } else {
            (k1 - underlying).max(0.0)
        };
        (-0.05_f64 * 0.25).exp() * payoff
    }

    /// At `σ = 0` every combination is its deterministic value, and the
    /// closed form at `σ = 1e-4` converges to it (#867). The zero-volatility
    /// branch used to value the underlying through Black-Scholes at `σ = 0`
    /// and returned an error.
    #[test]
    fn test_geske_zero_volatility_is_the_deterministic_value() {
        // The underlying call is worth about 2.93 at `t1` and the put 0:
        // `K1 = 2` puts every compound away from its kink.
        for (outer_call, underlying_call) in
            [(true, true), (false, true), (true, false), (false, false)]
        {
            let expected = deterministic(2.0, outer_call, underlying_call);
            let at_zero = geske_at(2.0, Positive::ZERO, outer_call, underlying_call);
            assert!(
                (at_zero.to_f64().unwrap() - expected).abs() < 1e-12,
                "outer call {outer_call}, underlying call {underlying_call}: {at_zero} vs {expected}"
            );
            let near_zero = geske_at(2.0, pos_or_panic!(0.0001), outer_call, underlying_call);
            assert!(
                (near_zero - at_zero).abs() < dec!(0.0001),
                "outer call {outer_call}, underlying call {underlying_call}: σ = 1e-4 gives {near_zero}, σ = 0 gives {at_zero}"
            );
        }
        // Spelled out: the call on a call and the put on a put.
        assert!(geske_at(2.0, Positive::ZERO, true, true) > dec!(0.9));
        assert!(geske_at(2.0, Positive::ZERO, false, false) > dec!(1.9));
    }

    /// Through the public kernel a zero-volatility compound before expiry is
    /// priced, not an error, and its value at expiry is unchanged (#867).
    #[test]
    fn test_compound_zero_volatility_through_the_kernel() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let mut option = create_compound_option(style, OptionType::European);
            option.implied_volatility = Positive::ZERO;
            let price = compound_black_scholes(&option).unwrap();
            // The helper's contract: `S = 100`, `K1 = K2 = 5`, `T1 = 91.25`
            // days, `T2 = 2 T1`, `r = 5 %`, `q = 0`. The call on a call pays
            // the discounted forward intrinsic less 5; the put on a put pays
            // the 5 itself, its underlying put being worthless.
            let t1 = 91.25 / 365.0;
            let forward = 100.0 * (0.05_f64 * 2.0 * t1).exp();
            let underlying = match style {
                OptionStyle::Call => (-0.05_f64 * t1).exp() * (forward - 5.0).max(0.0),
                OptionStyle::Put => (-0.05_f64 * t1).exp() * (5.0 - forward).max(0.0),
            };
            let payoff = match style {
                OptionStyle::Call => (underlying - 5.0).max(0.0),
                OptionStyle::Put => (5.0 - underlying).max(0.0),
            };
            let expected = (-0.05_f64 * t1).exp() * payoff;
            assert!(
                (price.to_f64().unwrap() - expected).abs() < 1e-12,
                "{style:?}: {price} vs {expected}"
            );
            option.expiration_date = ExpirationDate::Days(Positive::ZERO);
            let at_expiry = compound_black_scholes(&option).unwrap();
            let expected_at_expiry = match style {
                OptionStyle::Call => dec!(90),
                OptionStyle::Put => dec!(5),
            };
            assert_eq!(at_expiry, expected_at_expiry, "{style:?} at expiry");
        }
    }
}
