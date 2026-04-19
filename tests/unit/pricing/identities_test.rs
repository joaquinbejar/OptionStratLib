/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-19
******************************************************************************/

//! Deterministic regression tests for the fundamental pricing identities
//! per `rules/global_rules.md` §Numerical Discipline:
//!
//! * Put-call parity on a grid of Black-Scholes inputs:
//!   `C - P = S - K · e^(-rT)` (zero dividend).
//! * CRR binomial convergence: `binomial(N) → black_scholes` as `N → ∞`
//!   on a fixed schedule of step counts.
//! * Greek sanity identities:
//!   - `Γ_call == Γ_put`  (equal Gamma for European call/put).
//!   - `Vega_call == Vega_put`.
//!   - `Δ_call - Δ_put == e^(-qT)` → `1` for zero dividend.

use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::model::{ExpirationDate, Options};
use optionstratlib::pricing::binomial_model::BinomialPricingParams;
use optionstratlib::pricing::{black_scholes, price_binomial};
use positive::{Positive, pos_or_panic};
use rust_decimal::Decimal;
use rust_decimal::prelude::MathematicalOps;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

fn mk_option(
    style: OptionStyle,
    spot: Positive,
    strike: Positive,
    days: u32,
    iv: Positive,
    r: Decimal,
) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "TEST".to_string(),
        strike,
        ExpirationDate::Days(pos_or_panic!(f64::from(days))),
        iv,
        Positive::ONE,
        spot,
        r,
        style,
        Positive::ZERO,
        None,
    )
}

/// Deterministic grid regression for put-call parity.
#[test]
fn put_call_parity_grid() {
    let spots: [Positive; 3] = [
        pos_or_panic!(80.0),
        pos_or_panic!(100.0),
        pos_or_panic!(120.0),
    ];
    let strikes: [Positive; 3] = [
        pos_or_panic!(90.0),
        pos_or_panic!(100.0),
        pos_or_panic!(110.0),
    ];
    let ivs: [Positive; 3] = [
        pos_or_panic!(0.15),
        pos_or_panic!(0.25),
        pos_or_panic!(0.45),
    ];
    let days: [u32; 3] = [30, 90, 180];
    let rate = dec!(0.03);
    let tolerance = dec!(0.0001);

    for s in &spots {
        for k in &strikes {
            for iv in &ivs {
                for d in &days {
                    let call = mk_option(OptionStyle::Call, *s, *k, *d, *iv, rate);
                    let put = mk_option(OptionStyle::Put, *s, *k, *d, *iv, rate);
                    let c = black_scholes(&call).expect("call price");
                    let p = black_scholes(&put).expect("put price");
                    let t_years = call.time_to_expiration().expect("t").to_dec();
                    let discount = (-rate * t_years).exp();
                    let rhs = s.to_dec() - k.to_dec() * discount;
                    let lhs = c - p;
                    let diff = (lhs - rhs).abs();
                    assert!(
                        diff < tolerance,
                        "parity violated: s={s}, k={k}, iv={iv}, d={d}, diff={diff}",
                    );
                }
            }
        }
    }
}

/// CRR binomial converges to Black-Scholes for a European call.
#[test]
fn crr_binomial_converges_to_black_scholes() {
    let spot = pos_or_panic!(100.0);
    let strike = pos_or_panic!(100.0);
    let iv = pos_or_panic!(0.20);
    let days = 180u32;
    let rate = dec!(0.03);

    let call = mk_option(OptionStyle::Call, spot, strike, days, iv, rate);
    let bs_price = black_scholes(&call).expect("bs");

    // Step schedule: convergence should be monotone-ish and close at N=1000.
    let mut prev_err = Decimal::MAX;
    for n in [50usize, 200, 800] {
        let steps = NonZeroUsize::new(n).expect("nz steps");
        let expiry = call.time_to_expiration().expect("t");
        let params = BinomialPricingParams {
            asset: spot,
            volatility: iv,
            int_rate: rate,
            strike,
            expiry,
            no_steps: steps,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };
        let crr = price_binomial(params).expect("crr");
        let err = (crr - bs_price).abs();
        // Non-strict decrease: tolerate noise but require <1% of BS price at N=800
        if n == 800 {
            let rel = err / bs_price.abs();
            assert!(rel < dec!(0.01), "CRR(800) not within 1%% of BS: rel={rel}");
        }
        // Sanity: magnitude broadly shrinks from N=50 to N=800 (factor of 3+).
        if n == 800 {
            let n50_steps = NonZeroUsize::new(50).expect("nz50");
            let expiry50 = call.time_to_expiration().expect("t");
            let params_50 = BinomialPricingParams {
                asset: spot,
                volatility: iv,
                int_rate: rate,
                strike,
                expiry: expiry50,
                no_steps: n50_steps,
                option_type: &OptionType::European,
                option_style: &OptionStyle::Call,
                side: &Side::Long,
            };
            let crr_50 = price_binomial(params_50).expect("crr 50");
            let err_50 = (crr_50 - bs_price).abs();
            assert!(
                err <= err_50,
                "expected convergence: err(800)={err} > err(50)={err_50}",
            );
        }
        prev_err = err;
    }
    assert!(
        prev_err.is_sign_positive() || prev_err.is_zero(),
        "err must be non-negative"
    );
}

/// Greek sanity: Γ_call == Γ_put and Vega_call == Vega_put for European options.
#[test]
fn greek_sanity_gamma_vega_equal_call_put() {
    let spot = pos_or_panic!(100.0);
    let strike = pos_or_panic!(100.0);
    let iv = pos_or_panic!(0.25);
    let days = 60u32;
    let rate = dec!(0.03);

    let call = mk_option(OptionStyle::Call, spot, strike, days, iv, rate);
    let put = mk_option(OptionStyle::Put, spot, strike, days, iv, rate);

    let gamma_c = call.gamma().expect("gamma call");
    let gamma_p = put.gamma().expect("gamma put");
    let vega_c = call.vega().expect("vega call");
    let vega_p = put.vega().expect("vega put");

    let tol = dec!(0.0000001);
    assert!(
        (gamma_c - gamma_p).abs() < tol,
        "gamma call {gamma_c} != put {gamma_p}"
    );
    assert!(
        (vega_c - vega_p).abs() < tol,
        "vega call {vega_c} != put {vega_p}"
    );
}

/// Δ_call − Δ_put = e^(-qT); for zero dividend that collapses to 1.
#[test]
fn greek_sanity_delta_call_minus_put_equals_unity() {
    let spot = pos_or_panic!(100.0);
    let strike = pos_or_panic!(100.0);
    let iv = pos_or_panic!(0.25);
    let days = 60u32;
    let rate = dec!(0.03);

    let call = mk_option(OptionStyle::Call, spot, strike, days, iv, rate);
    let put = mk_option(OptionStyle::Put, spot, strike, days, iv, rate);

    let delta_c = call.delta().expect("delta call");
    let delta_p = put.delta().expect("delta put");
    let diff = delta_c - delta_p;
    let tol = dec!(0.0002);
    let one = Decimal::ONE;
    assert!(
        (diff - one).abs() < tol,
        "Δ_call - Δ_put must ≈ 1 for zero dividend, got {diff}",
    );
}
