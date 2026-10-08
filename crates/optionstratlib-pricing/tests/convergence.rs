/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-05
******************************************************************************/

//! Convergence and numerical-method tolerances for `optionstratlib-pricing`
//! (#526).
//!
//! # Coverage
//!
//! * Cox-Ross-Rubinstein binomial (`price_binomial`) converges to
//!   Black-Scholes for European calls and puts: the error shrinks from 50 to
//!   500 steps and stays under a documented bound at 1000 steps. This
//!   extends the single at-the-money check of `identities_test.rs` to a
//!   moneyness and maturity grid.
//! * Finite-difference Greeks (`greeks::numerical`) agree with the closed
//!   forms, both directly and through the non-European fallback of
//!   `greeks::delta`; numerical theta (a one-day central difference in
//!   time, #796) agrees with the closed-form theta, satisfies put-call
//!   parity, and degrades near expiry as its error model says.
//! * Monte Carlo is exercised only through supplied terminal prices
//!   (`price_option_monte_carlo`), so nothing depends on an RNG: the price is
//!   the discounted mean payoff, and a deterministic stratified sample of the
//!   exact GBM terminal law converges to Black-Scholes.
//!
//! # Sources
//!
//! Cox, Ross and Rubinstein (1979); Leisen and Reimer (1996) and Diener and
//! Diener (2004) for the `O(1/N)` oscillating error of the CRR tree; Hull,
//! *Binomial Trees in Practice*, for the CRR construction; Glasserman,
//! *Monte Carlo Methods in Financial Engineering* (2003), §4.3, for
//! stratified sampling; Acklam's rational approximation of the normal
//! quantile (relative error below `1.2e-9`).
//!
//! # Tolerance policy
//!
//! Every bound is stated with its error model next to the assertion:
//! `c/N` for the lattice, `h²/6 · f'''` truncation plus price rounding over
//! `h` or `h²` for the central differences (`h = 0.01` in
//! `greeks::numerical`), and the tail-truncation `c/n` of the midpoint
//! quantile rule for the stratified sample. The bounds sit at roughly three
//! times the largest error measured on these grids.
//! * The supplied-path estimator discounts at `r` whatever the dividend
//!   yield (#651).

use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::numerical::{
    numerical_delta, numerical_gamma, numerical_rho, numerical_theta, numerical_vega,
};
use optionstratlib_pricing::greeks::{delta, gamma, rho, theta, vega};
use optionstratlib_pricing::pricing::monte_carlo::price_option_monte_carlo;
use optionstratlib_pricing::pricing::{BinomialPricingParams, black_scholes, price_binomial};
use rust_decimal::Decimal;
use rust_decimal::prelude::MathematicalOps;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::num::NonZeroUsize;

/// Unwraps a fallible library call, failing the test with context.
fn ok<T, E: Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: unexpected error {err:?}"),
    }
}

fn step_count(steps: usize) -> NonZeroUsize {
    match NonZeroUsize::new(steps) {
        Some(value) => value,
        None => panic!("step count must be non-zero"),
    }
}

#[allow(clippy::too_many_arguments)]
fn option(
    option_type: OptionType,
    style: OptionStyle,
    spot: f64,
    strike: f64,
    days: f64,
    vol: f64,
    rate: Decimal,
    dividend: f64,
) -> Options {
    Options::new(
        option_type,
        Side::Long,
        "CONV".to_string(),
        pos_or_panic!(strike),
        ExpirationDate::Days(pos_or_panic!(days)),
        pos_or_panic!(vol),
        Positive::ONE,
        pos_or_panic!(spot),
        rate,
        style,
        pos_or_panic!(dividend),
        None,
    )
}

fn crr_price(opt: &Options, steps: usize) -> Decimal {
    ok(
        price_binomial(BinomialPricingParams {
            asset: opt.underlying_price,
            volatility: opt.implied_volatility,
            int_rate: opt.risk_free_rate,
            strike: opt.strike_price,
            expiry: ok(opt.time_to_expiration(), "expiry"),
            no_steps: step_count(steps),
            option_type: &opt.option_type,
            option_style: &opt.option_style,
            side: &opt.side,
        }),
        "binomial",
    )
}

// ---------------------------------------------------------------------------
// Binomial
// ---------------------------------------------------------------------------

/// Largest admissible `|CRR(1000) - BS|`.
///
/// The CRR error for a European vanilla is `c(N)/N` with a bounded,
/// oscillating `c` (Leisen and Reimer 1996; Diener and Diener 2004). On the
/// grid below the measured `|c|` peaks at `2.9` (at the money,
/// `σ = 30 %, T = 1`); the bound allows `|c| = 10`.
const CRR_ERROR_BOUND_AT_1000: Decimal = dec!(0.01);

/// European CRR prices approach Black-Scholes: `|err(50)| > |err(500)|` and
/// `|err(1000)| < CRR_ERROR_BOUND_AT_1000`. The binomial pricer takes no
/// dividend yield, so the grid has `q = 0`.
///
/// Calls run in, at and out of the money at `σ = 30 %, T = 1` plus at the
/// money at `σ = 20 %, T = 0.5`; the put runs at the money. A European CRR
/// tree satisfies put-call parity exactly, node by node, so a put at the
/// same inputs carries the call's error and further puts add run time, not
/// coverage (debug-build lattices of 1000 steps cost about 0.4 s each).
///
/// The error is not monotone step by step (the odd/even oscillation of the
/// tree can make `|err(1000)| > |err(500)|`), which is why only the 50 to 500
/// decrease and the absolute bound at 1000 are asserted.
#[test]
fn test_binomial_european_converges_to_black_scholes_grid() {
    let cases = [
        (OptionStyle::Call, 80.0, 0.30, 365.0, dec!(0.05)),
        (OptionStyle::Call, 100.0, 0.30, 365.0, dec!(0.05)),
        (OptionStyle::Call, 120.0, 0.30, 365.0, dec!(0.05)),
        (OptionStyle::Put, 100.0, 0.30, 365.0, dec!(0.05)),
        (OptionStyle::Call, 100.0, 0.20, 182.5, dec!(0.03)),
    ];
    for (style, strike, vol, days, rate) in cases {
        let opt = option(
            OptionType::European,
            style,
            100.0,
            strike,
            days,
            vol,
            rate,
            0.0,
        );
        let reference = ok(black_scholes(&opt), "black-scholes");
        let err_50 = (crr_price(&opt, 50) - reference).abs();
        let err_500 = (crr_price(&opt, 500) - reference).abs();
        let err_1000 = (crr_price(&opt, 1000) - reference).abs();
        let ctx = format!("{style:?} k={strike} vol={vol} days={days}");
        assert!(
            err_50 > err_500,
            "{ctx}: err(50) {err_50} not above err(500) {err_500}"
        );
        assert!(
            err_1000 < CRR_ERROR_BOUND_AT_1000,
            "{ctx}: err(1000) {err_1000} >= {CRR_ERROR_BOUND_AT_1000}"
        );
    }
}

/// An American CRR put converges as the tree refines: the change from 200
/// to 400 steps is below the change from 50 to 200 and below
/// `CRR_ERROR_BOUND_AT_1000` (Cauchy-style convergence; no closed form
/// exists to compare against), and the value stays above the European
/// Black-Scholes put. At the money, `σ = 30 %, T = 1, r = 5 %`. The
/// American lattice re-prices the payoff at every node, so it runs on
/// 400 steps rather than 1000 to keep the debug build fast.
#[test]
fn test_binomial_american_put_increments_shrink() {
    let opt = option(
        OptionType::American,
        OptionStyle::Put,
        100.0,
        100.0,
        365.0,
        0.3,
        dec!(0.05),
        0.0,
    );
    let p_50 = crr_price(&opt, 50);
    let p_200 = crr_price(&opt, 200);
    let p_400 = crr_price(&opt, 400);
    let coarse = (p_200 - p_50).abs();
    let fine = (p_400 - p_200).abs();
    assert!(
        fine < coarse,
        "increment 200->400 {fine} not below 50->200 {coarse}"
    );
    assert!(
        fine < CRR_ERROR_BOUND_AT_1000,
        "increment 200->400 {fine} above {CRR_ERROR_BOUND_AT_1000}"
    );
    let european = option(
        OptionType::European,
        OptionStyle::Put,
        100.0,
        100.0,
        365.0,
        0.3,
        dec!(0.05),
        0.0,
    );
    assert!(
        p_400 > ok(black_scholes(&european), "bs put"),
        "american put {p_400} not above the european value"
    );
}

// ---------------------------------------------------------------------------
// Numerical Greeks
// ---------------------------------------------------------------------------

/// Central-difference delta: truncation `h²/6 · ∂³V/∂S³` is below `1e-8`
/// here and the price-rounding term `ε/h` (`ε ≈ 1e-10`) below `1e-8`;
/// asserted at `1e-6`.
const DELTA_TOL: Decimal = dec!(0.000001);

/// Central second-difference gamma: truncation is below `1e-9`, but the
/// price-rounding term `4ε/h²` dominates. Each price carries about `1e-10`
/// absolute error from the `Decimal` `ln`/`exp` series and the `f64` normal
/// CDF, and dividing by `h² = 1e-4` lifts it to the measured `4.8e-6`
/// (`K = 80`); asserted at `1e-5` absolute.
const GAMMA_TOL: Decimal = dec!(0.00001);

/// Central-difference vega and rho, compared relative to the closed form.
/// The truncation `h²/6 · f'''` with `h = 0.01` in volatility or rate
/// reaches `3.5e-4` relative on this grid (deep in-the-money vega, `K = 80`;
/// an independent `f64` evaluation of the same difference agrees to `1e-9`),
/// so the bound is `1e-3` relative.
const RELATIVE_TOL: Decimal = dec!(0.001);

fn assert_relative(numerical: Decimal, closed: Decimal, context: &str) {
    let relative = ((numerical - closed) / closed).abs();
    assert!(
        relative <= RELATIVE_TOL,
        "{context}: numerical {numerical}, closed form {closed}, relative {relative} > {RELATIVE_TOL}"
    );
}

/// `greeks::numerical` (bump `h = 0.01`, per unit of spot, volatility and
/// rate) against the closed forms (vega and rho per 1 %, so the numerical
/// values are divided by 100), long European calls and puts.
#[test]
fn test_numerical_greeks_agree_with_closed_form_grid() {
    for strike in [80.0, 100.0, 120.0] {
        for (vol, days) in [(0.30, 365.0), (0.20, 182.5)] {
            for style in [OptionStyle::Call, OptionStyle::Put] {
                let opt = option(
                    OptionType::European,
                    style,
                    100.0,
                    strike,
                    days,
                    vol,
                    dec!(0.05),
                    0.0,
                );
                let ctx = format!("{style:?} k={strike} vol={vol} days={days}");
                let delta_err =
                    (ok(numerical_delta(&opt), "num delta") - ok(delta(&opt), "delta")).abs();
                assert!(delta_err <= DELTA_TOL, "{ctx}: delta error {delta_err}");
                let gamma_err =
                    (ok(numerical_gamma(&opt), "num gamma") - ok(gamma(&opt), "gamma")).abs();
                assert!(gamma_err <= GAMMA_TOL, "{ctx}: gamma error {gamma_err}");
                assert_relative(
                    ok(numerical_vega(&opt), "num vega") / dec!(100),
                    ok(vega(&opt), "vega"),
                    &format!("{ctx} vega"),
                );
                assert_relative(
                    ok(numerical_rho(&opt), "num rho") / dec!(100),
                    ok(rho(&opt), "rho"),
                    &format!("{ctx} rho"),
                );
            }
        }
    }
}

/// `greeks::delta` falls back to the numerical delta for non-European
/// types. A power option with exponent 1 is a vanilla, so the fallback must
/// land on the closed-form vanilla delta within [`DELTA_TOL`].
#[test]
fn test_numerical_delta_fallback_matches_closed_form_on_power_option() {
    for strike in [90.0, 100.0, 110.0] {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let power = option(
                OptionType::Power {
                    exponent: Positive::ONE,
                },
                style,
                100.0,
                strike,
                182.5,
                0.25,
                dec!(0.04),
                0.0,
            );
            let vanilla = option(
                OptionType::European,
                style,
                100.0,
                strike,
                182.5,
                0.25,
                dec!(0.04),
                0.0,
            );
            let err = (ok(delta(&power), "fallback delta") - ok(delta(&vanilla), "delta")).abs();
            assert!(err <= DELTA_TOL, "{style:?} k={strike}: delta error {err}");
        }
    }
}

/// Numerical theta against the closed form, absolute per day. The central
/// difference over one day either way has truncation `h^2/6 * V'''` with
/// `h = 1` day; `V'''` grows like `T^(-5/2)` at the money, so the error is
/// largest at the shortest maturity of the grid: `6.6e-6` per day at
/// 30 days (`K = 100`), against `5.9e-8` at 182.5 days and `1.6e-8` at a
/// year. Asserted at `2e-5` absolute: deep out-of-the-money thetas are
/// near zero, so a relative bound would test nothing there.
const THETA_TOL: Decimal = dec!(0.00002);

/// Put-call parity for numerical theta: `C - P = S e^(-qT) - K e^(-rT)`
/// exactly under Black-Scholes, so the difference of the two numerical
/// thetas is a central difference of two exponentials, whose truncation is
/// below `1e-14`. What remains is price rounding, measured at `5.1e-11`;
/// asserted at `1.5e-10`.
const THETA_PARITY_TOL: Decimal = dec!(0.00000000015);

fn european(style: OptionStyle, strike: f64, days: f64, vol: f64, dividend: f64) -> Options {
    option(
        OptionType::European,
        style,
        100.0,
        strike,
        days,
        vol,
        dec!(0.05),
        dividend,
    )
}

/// `greeks::numerical::numerical_theta` (one-day bump, per day) against
/// `greeks::theta` (per day), long European calls and puts, with and
/// without a dividend yield (#796).
#[test]
fn test_numerical_theta_agrees_with_closed_form_grid() {
    for strike in [80.0, 100.0, 120.0] {
        for (vol, days) in [(0.30, 365.0), (0.20, 182.5), (0.25, 30.0)] {
            for dividend in [0.0, 0.02] {
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    let opt = european(style, strike, days, vol, dividend);
                    let error =
                        (ok(numerical_theta(&opt), "num theta") - ok(theta(&opt), "theta")).abs();
                    assert!(
                        error <= THETA_TOL,
                        "{style:?} k={strike} vol={vol} days={days} q={dividend}: theta error {error}"
                    );
                }
            }
        }
    }
}

/// `theta_call - theta_put = (q S e^(-qT) - r K e^(-rT)) / 365` per day, from
/// put-call parity, including within days of expiry where each theta alone
/// is coarse.
#[test]
fn test_numerical_theta_satisfies_put_call_parity() {
    for strike in [80.0, 100.0, 120.0] {
        for days in [365.0, 182.5, 30.0, 7.0, 1.5] {
            for dividend in [0.0, 0.02] {
                let call = ok(
                    numerical_theta(&european(OptionStyle::Call, strike, days, 0.25, dividend)),
                    "call theta",
                );
                let put = ok(
                    numerical_theta(&european(OptionStyle::Put, strike, days, 0.25, dividend)),
                    "put theta",
                );
                let t = ok(Decimal::try_from(days), "days") / dec!(365);
                let q = ok(Decimal::try_from(dividend), "dividend");
                let k = ok(Decimal::try_from(strike), "strike");
                let carry = q * dec!(100) * (-q * t).exp();
                let rate = dec!(0.05) * k * (dec!(-0.05) * t).exp();
                let expected = (carry - rate) / dec!(365);
                let error = (call - put - expected).abs();
                assert!(
                    error <= THETA_PARITY_TOL,
                    "k={strike} days={days} q={dividend}: parity error {error}"
                );
            }
        }
    }
}

/// The at-the-money error grows toward expiry as `T^(-5/2)`: measured
/// `1.6e-8`, `6.6e-6` and `2.5e-4` per day at 365, 30 and 7 days. A one-day
/// step is coarse in the last week, as the docs of `numerical_theta` state;
/// this pins that the error orders by maturity.
#[test]
fn test_numerical_theta_error_shrinks_with_maturity() {
    let errors: Vec<Decimal> = [7.0, 30.0, 182.5, 365.0]
        .into_iter()
        .map(|days| {
            let opt = european(OptionStyle::Call, 100.0, days, 0.25, 0.0);
            (ok(numerical_theta(&opt), "num theta") - ok(theta(&opt), "theta")).abs()
        })
        .collect();
    assert!(
        errors.windows(2).all(|pair| pair[0] > pair[1]),
        "errors by maturity {errors:?}"
    );
}

/// Within one day of expiry the difference is one-sided,
/// `P(T) - P(T + 1 day)`: the mean theta over that day. At the money theta
/// grows in magnitude toward expiry, so the result lies between the closed
/// forms at `T` and at `T + 1 day`. At expiry it is `0`, as the closed form.
#[test]
fn test_numerical_theta_near_and_at_expiry() {
    for style in [OptionStyle::Call, OptionStyle::Put] {
        let numerical = ok(
            numerical_theta(&european(style, 100.0, 0.5, 0.25, 0.0)),
            "num theta",
        );
        let at_t = ok(theta(&european(style, 100.0, 0.5, 0.25, 0.0)), "theta");
        let a_day_later = ok(theta(&european(style, 100.0, 1.5, 0.25, 0.0)), "theta");
        assert!(
            at_t < numerical && numerical < a_day_later,
            "{style:?}: {numerical} not between {at_t} and {a_day_later}"
        );

        let expired = european(style, 100.0, 0.0, 0.25, 0.0);
        assert_eq!(
            ok(numerical_theta(&expired), "expired theta"),
            Decimal::ZERO
        );
    }
}

// ---------------------------------------------------------------------------
// Monte Carlo on supplied paths
// ---------------------------------------------------------------------------

fn positives(values: &[f64]) -> Vec<Positive> {
    values.iter().map(|value| pos_or_panic!(*value)).collect()
}

/// With supplied terminal prices the estimator is exactly
/// `e^(-rT) · mean(payoff)`; no RNG is involved. The only rounding is the
/// mean's `Decimal` division and the `exp` series, far below `1e-12`.
#[test]
fn test_monte_carlo_supplied_paths_equal_discounted_mean_payoff() {
    let terminal = positives(&[80.0, 95.0, 100.0, 105.0, 120.0, 130.0]);
    let cases = [(OptionStyle::Call, dec!(55)), (OptionStyle::Put, dec!(25))];
    for (style, payoff_sum) in cases {
        let opt = option(
            OptionType::European,
            style,
            100.0,
            100.0,
            365.0,
            0.2,
            dec!(0.05),
            0.0,
        );
        let price = ok(price_option_monte_carlo(&opt, &terminal), "mc").to_dec();
        let expected = (dec!(-0.05)).exp() * payoff_sum / dec!(6);
        let diff = (price - expected).abs();
        assert!(
            diff < dec!(0.000000000001),
            "{style:?}: {price} vs {expected}"
        );
    }
}

/// Discounting does not depend on the dividend yield: the yield shapes the
/// terminal law the caller supplies, the payoff is still discounted at `r`
/// (#651). With `r = 5 %, q = 2 %, T = 1` and payoffs averaging `7.5` the
/// price is `7.5 e^(-0.05) = 7.1342`, not `7.5 e^(-0.03) = 7.2783`.
#[test]
fn test_monte_carlo_supplied_paths_discount_at_risk_free_rate() {
    let terminal = positives(&[90.0, 100.0, 110.0, 120.0]);
    let opt = option(
        OptionType::European,
        OptionStyle::Call,
        100.0,
        100.0,
        365.0,
        0.2,
        dec!(0.05),
        0.02,
    );
    let price = ok(price_option_monte_carlo(&opt, &terminal), "mc").to_dec();
    let expected = (dec!(-0.05)).exp() * dec!(7.5);
    let diff = (price - expected).abs();
    assert!(diff < dec!(0.000000000001), "{price} vs {expected}");
}

/// Acklam's rational approximation of the standard normal quantile,
/// relative error below `1.2e-9` on `(0, 1)`.
#[allow(clippy::excessive_precision)]
fn normal_quantile(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    const P_LOW: f64 = 0.02425;
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    if p < P_LOW {
        tail((-2.0 * p.ln()).sqrt())
    } else if p > 1.0 - P_LOW {
        -tail((-2.0 * (1.0 - p).ln()).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    }
}

/// Terminal prices at the midpoint quantiles `(i - 1/2)/n` of the exact
/// risk-neutral GBM law, `S_T = S e^((r - σ²/2)T + σ√T z)`.
fn stratified_terminal_prices(spot: f64, rate: f64, vol: f64, years: f64, n: u32) -> Vec<Positive> {
    let drift = (rate - vol * vol / 2.0) * years;
    let diffusion = vol * years.sqrt();
    (1..=n)
        .map(|i| {
            let u = (f64::from(i) - 0.5) / f64::from(n);
            let terminal = spot * (drift + diffusion * normal_quantile(u)).exp();
            ok(Positive::new(terminal), "terminal price")
        })
        .collect()
}

/// A deterministic stratified sample of the exact terminal law drives the
/// supplied-path estimator to Black-Scholes. The midpoint rule drops the
/// tails beyond `u = 1/(2n)`, an `O(1/n)` error that is largest for the call
/// (`|err| · n ≈ 7.7` at the money, `σ = 30 %, T = 1`); the bound at
/// `n = 2000` is `0.01`, `2.6×` the expected `0.0039`.
#[test]
fn test_monte_carlo_stratified_sample_converges_to_black_scholes() {
    let rate = dec!(0.05);
    for style in [OptionStyle::Call, OptionStyle::Put] {
        let opt = option(
            OptionType::European,
            style,
            100.0,
            100.0,
            365.0,
            0.3,
            rate,
            0.0,
        );
        let reference = ok(black_scholes(&opt), "black-scholes");
        let error = |n: u32| {
            let terminal = stratified_terminal_prices(100.0, 0.05, 0.3, 1.0, n);
            (ok(price_option_monte_carlo(&opt, &terminal), "mc").to_dec() - reference).abs()
        };
        let coarse = error(100);
        let fine = error(2000);
        assert!(
            fine < coarse,
            "{style:?}: err(2000) {fine} not below err(100) {coarse}"
        );
        assert!(fine < dec!(0.01), "{style:?}: err(2000) {fine} >= 0.01");
    }
}
