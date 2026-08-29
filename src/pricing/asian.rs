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
    let t_sq = d_powd(t_dec, Decimal::TWO, "pricing::asian::arithmetic::t_sq")?;
    let bt = d_mul(b, t_dec, "pricing::asian::arithmetic::bt")?;
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
    //        = (2 S² / (a T²)) [ (e^{c T} - 1) / c - (e^{b T} - 1) / b ],
    //
    // which is the closed form expanded in the `else` branch below. All three
    // of its denominators are removable: at `b = 0` (`r = q`), at `a = 0`
    // (`b = -σ²`) and at `c = 0` (`b = -σ²/2`) the vanishing factor is
    // cancelled by a bracket that vanishes with it, and M2 stays finite. Every
    // limit below was taken by going back to the integral and evaluating the
    // degenerate exponential directly instead of dividing by its rate —
    // `∫₀^T e^{0·u} du = T` for `b = 0`, `∫₀^u e^{0·t} dt = u` for `a = 0`,
    // `∫₀^T e^{0·u} du = T` again for `c = 0` — which is the same value as
    // differentiating the bracket at the zero. They are exact, not
    // approximations. `r = q` lands on `b = 0` and `r = 0, q = 4%, σ = 20%,
    // T = 1` lands exactly on `a = 0`; both are ordinary contracts, so no
    // boundary may raise.
    let m2 = if b.abs() < dec!(1e-10) {
        // `b → 0`, i.e. `r = q`. The outer growth term degenerates to
        // `∫₀^T e^{0·u} du` and the inner rate to `a = σ²`, so
        //
        //     M2 = (2 S² / T²) ∫₀^T ∫₀^u e^{σ² t} dt du
        //        = (2 S² / T²) ∫₀^T (e^{σ² u} - 1) / σ² du
        //        = (2 S² / (σ² T²)) [ (e^{σ² T} - 1) / σ² - T ],
        //
        // the second integration having used `∫₀^T e^{σ² u} du =
        // (e^{σ² T} - 1) / σ²` and `∫₀^T du = T`. The general form tends to
        // the same value: `a, c → σ²` and `(e^{b T} - 1) / b → T`.
        //
        // This is *not* `S² e^{σ² T}`, which is `E[S_T²]` — the second moment
        // of the terminal price rather than of its average. At `σ = 20%,
        // T = 1` they are `1.01347 S²` against `1.04081 S²`, and the moment
        // matching turns that gap into `σ_adj = σ` instead of the correct
        // `σ_adj ≈ σ / √3`. `r = q` is a forward-priced or fully-carried
        // underlying, so this branch has to be the limit and not a stand-in.
        //
        // Collecting the whole expression on `x = σ² T` gives the form
        // evaluated below,
        //
        //     M2 = 2 S² (e^x - 1 - x) / x² = S² (1 + x/3 + x²/12 + …),
        //
        // one division shorter and, since `e^x - 1 - x` is an exact `Decimal`
        // subtraction, carrying every digit `e^x` had.
        let x = d_mul(
            sigma_sq,
            t_dec,
            "pricing::asian::arithmetic::m2::b_limit::var_t",
        )?;
        if x < dec!(1e-6) {
            // `e^x` is stored to 28 decimal places, so the `x² / 2` that
            // leads `e^x - 1 - x` sinks below that floor as `x` shrinks and
            // the closed form loses it: at `x = 1e-14` it valued a worthless
            // contract at 31, and below `x ≈ 1e-13` the `2 S² / x²` scale
            // leaves `Decimal` altogether. The two-term series truncates at
            // `x² / 12`, a relative `x / 4 ≤ 2.5e-7` on the matched variance,
            // against the closed form's `6e-28 / x³`; they cross at
            // `x ≈ 2e-7`, so below `1e-6` the series is the better of the
            // two. It also absorbs `σ² = 0` (σ under `1e-14` underflows the
            // scale), where `M2 = S²` exactly and there is nothing to divide
            // by.
            d_mul(
                s_sq,
                d_add(
                    Decimal::ONE,
                    d_div(x, dec!(3), "pricing::asian::arithmetic::m2::b_limit::third")?,
                    "pricing::asian::arithmetic::m2::b_limit::series",
                )?,
                "pricing::asian::arithmetic::m2::b_limit::small_variance",
            )?
        } else {
            let bracket = d_sub(
                d_sub(
                    d_exp(x, "pricing::asian::arithmetic::m2::b_limit::exp_x")?,
                    Decimal::ONE,
                    "pricing::asian::arithmetic::m2::b_limit::exp_x_less_one",
                )?,
                x,
                "pricing::asian::arithmetic::m2::b_limit::bracket",
            )?;
            d_mul(
                d_div(
                    d_mul(
                        dec!(2),
                        s_sq,
                        "pricing::asian::arithmetic::m2::b_limit::two_s_sq",
                    )?,
                    d_mul(x, x, "pricing::asian::arithmetic::m2::b_limit::x_sq")?,
                    "pricing::asian::arithmetic::m2::b_limit::scale",
                )?,
                bracket,
                "pricing::asian::arithmetic::m2::b_limit",
            )?
        }
    } else if b_plus_var.abs() < dec!(1e-10) {
        // `a → 0`, i.e. `b = -σ²`. The inner integral degenerates to
        // `∫₀^u dt = u`, leaving
        //
        //     M2 = (2 S² / T²) [ T e^{b T} / b - (e^{b T} - 1) / b² ].
        //
        // `b ≈ -σ²` is non-zero here: the driftless branch above owns `b = 0`,
        // and `σ = 0` returned long before this point.
        let exp_bt = d_exp(bt, "pricing::asian::arithmetic::m2::a_limit::exp_bt")?;
        let b_sq = d_mul(b, b, "pricing::asian::arithmetic::m2::a_limit::b_sq")?;
        let bracket = d_sub(
            d_div(
                d_mul(
                    t_dec,
                    exp_bt,
                    "pricing::asian::arithmetic::m2::a_limit::t_exp_bt",
                )?,
                b,
                "pricing::asian::arithmetic::m2::a_limit::first",
            )?,
            d_div(
                d_sub(
                    exp_bt,
                    dec!(1),
                    "pricing::asian::arithmetic::m2::a_limit::exp_bt_less_one",
                )?,
                b_sq,
                "pricing::asian::arithmetic::m2::a_limit::second",
            )?,
            "pricing::asian::arithmetic::m2::a_limit::bracket",
        )?;
        d_mul(
            d_div(
                d_mul(
                    dec!(2),
                    s_sq,
                    "pricing::asian::arithmetic::m2::a_limit::two_s_sq",
                )?,
                t_sq,
                "pricing::asian::arithmetic::m2::a_limit::scale",
            )?,
            bracket,
            "pricing::asian::arithmetic::m2::a_limit",
        )?
    } else if two_b_plus_var.abs() < dec!(1e-10) {
        // `c → 0`, i.e. `b = -σ²/2`. The outer integral's growth term
        // degenerates to `∫₀^T du = T`, leaving
        //
        //     M2 = (2 S² / (a T²)) [ T - (e^{b T} - 1) / b ].
        //
        // `a ≈ σ²/2` and `b ≈ -σ²/2` are both non-zero here for the same
        // reason as in the branch above.
        let exp_bt = d_exp(bt, "pricing::asian::arithmetic::m2::c_limit::exp_bt")?;
        let bracket = d_sub(
            t_dec,
            d_div(
                d_sub(
                    exp_bt,
                    dec!(1),
                    "pricing::asian::arithmetic::m2::c_limit::exp_bt_less_one",
                )?,
                b,
                "pricing::asian::arithmetic::m2::c_limit::second",
            )?,
            "pricing::asian::arithmetic::m2::c_limit::bracket",
        )?;
        d_mul(
            d_div(
                d_mul(
                    dec!(2),
                    s_sq,
                    "pricing::asian::arithmetic::m2::c_limit::two_s_sq",
                )?,
                d_mul(
                    b_plus_var,
                    t_sq,
                    "pricing::asian::arithmetic::m2::c_limit::denominator",
                )?,
                "pricing::asian::arithmetic::m2::c_limit::scale",
            )?,
            bracket,
            "pricing::asian::arithmetic::m2::c_limit",
        )?
    } else {
        let term1_exp = d_exp(
            d_mul(
                two_b_plus_var,
                t_dec,
                "pricing::asian::arithmetic::m2::term1_exponent",
            )?,
            "pricing::asian::arithmetic::m2::term1_exp",
        )?;
        let term1 = d_div(
            d_mul(
                d_mul(dec!(2), s_sq, "pricing::asian::arithmetic::m2::two_s_sq")?,
                term1_exp,
                "pricing::asian::arithmetic::m2::term1_numerator",
            )?,
            d_mul(
                d_mul(
                    b_plus_var,
                    two_b_plus_var,
                    "pricing::asian::arithmetic::m2::term1_roots",
                )?,
                t_sq,
                "pricing::asian::arithmetic::m2::term1_denominator",
            )?,
            "pricing::asian::arithmetic::m2::term1",
        )?;

        let term2 = d_mul(
            d_div(
                d_mul(dec!(2), s_sq, "pricing::asian::arithmetic::m2::two_s_sq2")?,
                d_mul(b, t_sq, "pricing::asian::arithmetic::m2::b_t_sq")?,
                "pricing::asian::arithmetic::m2::term2_scale",
            )?,
            d_sub(
                d_div(
                    dec!(1),
                    two_b_plus_var,
                    "pricing::asian::arithmetic::m2::term2_first",
                )?,
                d_div(
                    d_exp(bt, "pricing::asian::arithmetic::m2::exp_bt")?,
                    b_plus_var,
                    "pricing::asian::arithmetic::m2::term2_second",
                )?,
                "pricing::asian::arithmetic::m2::term2_bracket",
            )?,
            "pricing::asian::arithmetic::m2::term2",
        )?;

        d_add(term1, term2, "pricing::asian::arithmetic::m2")?
    };

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
    if b.abs() < dec!(1e-10) {
        return Ok(s);
    }
    let bt = d_mul(b, t, "pricing::asian::average_forward::bt")?;
    Ok(d_mul(
        s,
        d_div(
            d_sub(
                d_exp(bt, "pricing::asian::average_forward::exp_bt")?,
                dec!(1),
                "pricing::asian::average_forward::numerator",
            )?,
            bt,
            "pricing::asian::average_forward::ratio",
        )?,
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
}
