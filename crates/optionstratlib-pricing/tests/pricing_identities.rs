/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-05
******************************************************************************/

//! Deterministic model identities for `optionstratlib-pricing` (#526).
//!
//! Every assertion here is a model-free or model-internal identity evaluated
//! on a fixed parameter grid, so a drift in a parameter mapping, a carry
//! term or a `Decimal` <-> `f64` boundary breaks at least one case. The
//! proptest-driven parity checks live in `put_call_parity_test.rs` and
//! `greeks_bounds_test.rs`; this file is the deterministic counterpart and
//! extends the single-point checks of `identities_test.rs` to grids with
//! dividends, negative rates and long maturities.
//!
//! # Coverage
//!
//! * Put-call parity: Black-Scholes-Merton (`C - P = S e^(-qT) - K e^(-rT)`),
//!   Black-76 on the forward (`C - P = e^(-rT) (F - K)`), Garman-Kohlhagen
//!   with the foreign rate (`C - P = S e^(-r_f T) - K e^(-r_d T)`).
//! * Garman-Kohlhagen is Black-Scholes-Merton with `q = r_f`, bit for bit.
//! * Side symmetry: a short price or Greek is the negated long one, the
//!   binomial American included (#648).
//! * Greek identities: `Δc - Δp = e^(-qT)` (`e^(-rT)` for Black-76,
//!   `e^(-r_f T)` for Garman-Kohlhagen), `Γc = Γp`, `Vc = Vp`,
//!   `ρc - ρp = K T e^(-rT)` and `Θc - Θp = q S e^(-qT) - r K e^(-rT)`,
//!   in the library's units (vega and rho per 1 %, theta per calendar day).
//! * Exotic decompositions: binary (asset-or-nothing minus `K` cash-or-nothing
//!   is the vanilla call; call plus put of each binary is the discounted
//!   payout), barrier in-out parity without rebate, simple chooser at
//!   `t = T` is the straddle, Margrabe exchange parity, power option with
//!   exponent 1 is Black-Scholes, Kemna-Vorst geometric Asian is
//!   Black-Scholes with `σ/√3` and carry `(r - q - σ²/6)/2`, quanto with zero
//!   correlation and unit rate and `r_f = r_d` is Black-Scholes-Merton, and
//!   Kirk's spread approximation tends to Margrabe as the strike vanishes.
//! * Limits where the library defines them: binomial and Barone-Adesi-Whaley
//!   at `T = 0` (intrinsic) and `σ = 0` (discounted deterministic payoff),
//!   the zero-volatility American floors and interior exercise optimum of
//!   Barone-Adesi-Whaley (#648), binary options at `T = 0` and `σ = 0`, Black-Scholes delta at `T = 0`
//!   and `σ = 0`, and deep in/out-of-the-money Black-Scholes asymptotes.
//!   The closed forms (Black-Scholes, Black-76, Garman-Kohlhagen) do not
//!   define the `T = 0` or `σ = 0` limit: `d1` divides by `σ√T`, so they
//!   return a `PricingError`, which is asserted instead of a limit value.
//!
//! # Sources
//!
//! Hull, *Options, Futures, and Other Derivatives*, put-call parity and the
//! Greek letters chapter; Haug, *The Complete Guide to Option Pricing
//! Formulas*, 2nd ed. (2007): generalized Black-Scholes-Merton (§1.1.6),
//! binary decompositions (§4.19), standard barrier options and in-out parity
//! (§4.17.1), simple chooser (§4.12.1), Margrabe exchange (§5.1), geometric
//! average-rate options (§4.20.1), Kirk's spread approximation (§5.4.2);
//! Kemna and Vorst (1990); Margrabe (1978); Merton (1973) for the American
//! call on a non-dividend underlying.
//!
//! # Tolerance policy
//!
//! Prices are `Decimal`, but the normal CDF runs in `f64` and the `ln`/`exp`
//! kernels are `Decimal` series, so two independently assembled sides of an
//! identity need not agree to the last digit. Identities that share every
//! intermediate (side symmetry, `Γc = Γp`, Garman-Kohlhagen against
//! Black-Scholes-Merton) are asserted with exact equality. Identities
//! assembled from different kernels, including the exotic reductions that
//! cross the `f64` power kernel or an irrational `σ/√3`, use
//! [`IDENTITY_TOL`] = `1e-9` absolute on prices of order 100: every case on
//! these grids was measured below `1e-12`, so the bound leaves three orders
//! of margin while still catching parameter or carry drift (one basis point
//! of rate moves an at-the-money price on these grids by more than `1e-4`).
//! The Kirk-to-Margrabe continuity check and the four-decimal quanto value
//! name their own bound at the assertion.
//!
//! # Known discrepancies (filed, tested in the fix)
//!
//! Identities the library breaks today are not in this suite; each lives
//! in its issue with the test code: barrier bounds and `Side` (#646),
//! quanto foreign rate and Kirk second dividend (#650).

use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType,
    RainbowType, Side,
};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::{
    delta, delta_b76, delta_gk, gamma, gamma_b76, gamma_gk, rho, rho_b76, theta, theta_b76, vega,
    vega_b76, vega_gk,
};
use optionstratlib_pricing::pricing::{
    BinomialPricingParams, OptionPricing, barone_adesi_whaley, binary_black_scholes, black_76,
    black_scholes, garman_kohlhagen, generate_binomial_tree, price_binomial, spread_black_scholes,
};
use rust_decimal::Decimal;
use rust_decimal::prelude::MathematicalOps;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::num::NonZeroUsize;

/// Bound for identities whose two sides go through different kernels.
const IDENTITY_TOL: Decimal = dec!(0.000000001);

const SPOTS: [f64; 3] = [80.0, 100.0, 120.0];
const STRIKES: [f64; 3] = [90.0, 100.0, 110.0];
const VOLS: [f64; 3] = [0.10, 0.25, 0.60];
const RATES: [Decimal; 3] = [dec!(-0.01), dec!(0.03), dec!(0.08)];
/// 30 days, half a year (182.5 days) and two years.
const DAYS: [f64; 3] = [30.0, 182.5, 730.0];
const YIELDS: [f64; 2] = [0.0, 0.02];

/// Unwraps a fallible library call, failing the test with context.
fn ok<T, E: Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: unexpected error {err:?}"),
    }
}

/// Lattice step count; every literal passed here is non-zero.
fn step_count(steps: usize) -> NonZeroUsize {
    match NonZeroUsize::new(steps) {
        Some(value) => value,
        None => panic!("step count must be non-zero"),
    }
}

/// Asserts `|actual - expected| <= tol`.
fn assert_close(actual: Decimal, expected: Decimal, tol: Decimal, context: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tol,
        "{context}: actual {actual}, expected {expected}, |diff| {diff} > {tol}"
    );
}

#[allow(clippy::too_many_arguments)]
fn option(
    option_type: OptionType,
    style: OptionStyle,
    side: Side,
    spot: f64,
    strike: f64,
    days: f64,
    vol: f64,
    rate: Decimal,
    dividend: f64,
    exotic: Option<ExoticParams>,
) -> Options {
    Options::new(
        option_type,
        side,
        "TEST".to_string(),
        pos_or_panic!(strike),
        ExpirationDate::Days(pos_or_panic!(days)),
        pos_or_panic!(vol),
        Positive::ONE,
        pos_or_panic!(spot),
        rate,
        style,
        pos_or_panic!(dividend),
        exotic,
    )
}

#[allow(clippy::too_many_arguments)]
fn european(
    style: OptionStyle,
    side: Side,
    spot: f64,
    strike: f64,
    days: f64,
    vol: f64,
    rate: Decimal,
    dividend: f64,
) -> Options {
    option(
        OptionType::European,
        style,
        side,
        spot,
        strike,
        days,
        vol,
        rate,
        dividend,
        None,
    )
}

/// `e^(-rate * years)` in `Decimal`.
fn discount(rate: Decimal, years: Decimal) -> Decimal {
    (-rate * years).exp()
}

fn years(option: &Options) -> Decimal {
    ok(option.time_to_expiration(), "time to expiration").to_dec()
}

/// Calls `check` once per point of the full European grid.
fn for_each_grid_point(mut check: impl FnMut(f64, f64, f64, Decimal, f64, f64)) {
    for &spot in &SPOTS {
        for &strike in &STRIKES {
            for &vol in &VOLS {
                for &rate in &RATES {
                    for &days in &DAYS {
                        for &dividend in &YIELDS {
                            check(spot, strike, vol, rate, days, dividend);
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Put-call parity
// ---------------------------------------------------------------------------

/// Black-Scholes-Merton parity `C - P = S e^(-qT) - K e^(-rT)` (Hull, put-call
/// parity; Haug §1.1.6).
#[test]
fn test_black_scholes_put_call_parity_with_dividend_grid_holds() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        let call = european(OptionStyle::Call, Side::Long, s, k, days, vol, r, q);
        let put = european(OptionStyle::Put, Side::Long, s, k, days, vol, r, q);
        let c = ok(black_scholes(&call), "bs call");
        let p = ok(black_scholes(&put), "bs put");
        let t = years(&call);
        let q_dec = pos_or_panic!(q).to_dec();
        let rhs = pos_or_panic!(s).to_dec() * discount(q_dec, t)
            - pos_or_panic!(k).to_dec() * discount(r, t);
        assert_close(
            c - p,
            rhs,
            IDENTITY_TOL,
            &format!("bs parity s={s} k={k} vol={vol} r={r} days={days} q={q}"),
        );
    });
}

/// Black-76 parity on the forward: `C - P = e^(-rT) (F - K)` (Black 1976;
/// Haug §1.1.4).
#[test]
fn test_black_76_put_call_parity_on_forward_grid_holds() {
    for_each_grid_point(|f, k, vol, r, days, _| {
        let call = european(OptionStyle::Call, Side::Long, f, k, days, vol, r, 0.0);
        let put = european(OptionStyle::Put, Side::Long, f, k, days, vol, r, 0.0);
        let c = ok(black_76(&call), "b76 call");
        let p = ok(black_76(&put), "b76 put");
        let rhs =
            discount(r, years(&call)) * (pos_or_panic!(f).to_dec() - pos_or_panic!(k).to_dec());
        assert_close(
            c - p,
            rhs,
            IDENTITY_TOL,
            &format!("b76 parity f={f} k={k} vol={vol} r={r} days={days}"),
        );
    });
}

/// Garman-Kohlhagen parity with the foreign rate carried in
/// `dividend_yield`: `C - P = S e^(-r_f T) - K e^(-r_d T)` (Garman and
/// Kohlhagen 1983; Haug §1.1.6). FX-scale spots exercise small numbers.
#[test]
fn test_garman_kohlhagen_put_call_parity_with_foreign_rate_grid_holds() {
    let spots = [1.05, 1.10, 1.25];
    let strikes = [1.00, 1.10, 1.20];
    let foreign_rates = [0.0, 0.02, 0.05];
    for &s in &spots {
        for &k in &strikes {
            for &vol in &VOLS {
                for &rd in &RATES {
                    for &days in &DAYS {
                        for &rf in &foreign_rates {
                            let call =
                                european(OptionStyle::Call, Side::Long, s, k, days, vol, rd, rf);
                            let put =
                                european(OptionStyle::Put, Side::Long, s, k, days, vol, rd, rf);
                            let c = ok(garman_kohlhagen(&call), "gk call");
                            let p = ok(garman_kohlhagen(&put), "gk put");
                            let t = years(&call);
                            let rhs = pos_or_panic!(s).to_dec()
                                * discount(pos_or_panic!(rf).to_dec(), t)
                                - pos_or_panic!(k).to_dec() * discount(rd, t);
                            assert_close(
                                c - p,
                                rhs,
                                IDENTITY_TOL,
                                &format!(
                                    "gk parity s={s} k={k} vol={vol} rd={rd} rf={rf} days={days}"
                                ),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Garman-Kohlhagen is Black-Scholes-Merton with `q = r_f`; the pricer
/// delegates, so the two agree exactly.
#[test]
fn test_garman_kohlhagen_equals_black_scholes_merton_exactly() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let opt = european(style, Side::Long, s, k, days, vol, r, q);
            assert_eq!(
                ok(garman_kohlhagen(&opt), "gk"),
                ok(black_scholes(&opt), "bs"),
                "gk != bsm at s={s} k={k} vol={vol} r={r} days={days} q={q} {style:?}"
            );
        }
    });
}

// ---------------------------------------------------------------------------
// Side symmetry
// ---------------------------------------------------------------------------

/// Short closed-form prices are the negated long prices, exactly.
#[test]
fn test_closed_form_short_price_equals_negated_long_grid() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let long = european(style, Side::Long, s, k, days, vol, r, q);
            let short = european(style, Side::Short, s, k, days, vol, r, q);
            let ctx = format!("s={s} k={k} vol={vol} r={r} days={days} q={q} {style:?}");
            assert_eq!(
                ok(black_scholes(&short), "bs short"),
                -ok(black_scholes(&long), "bs long"),
                "bs {ctx}"
            );
            assert_eq!(
                ok(black_76(&short), "b76 short"),
                -ok(black_76(&long), "b76 long"),
                "b76 {ctx}"
            );
            assert_eq!(
                ok(garman_kohlhagen(&short), "gk short"),
                -ok(garman_kohlhagen(&long), "gk long"),
                "gk {ctx}"
            );
        }
    });
}

/// Short Greeks are the negated long Greeks (position convention), exactly.
#[test]
fn test_greeks_short_equals_negated_long() {
    for &(s, k) in &[(90.0, 100.0), (100.0, 100.0), (110.0, 100.0)] {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let long = european(style, Side::Long, s, k, 182.5, 0.25, dec!(0.03), 0.02);
            let short = european(style, Side::Short, s, k, 182.5, 0.25, dec!(0.03), 0.02);
            let ctx = format!("s={s} k={k} {style:?}");
            assert_eq!(
                ok(delta(&short), "d"),
                -ok(delta(&long), "d"),
                "delta {ctx}"
            );
            assert_eq!(
                ok(gamma(&short), "g"),
                -ok(gamma(&long), "g"),
                "gamma {ctx}"
            );
            assert_eq!(ok(vega(&short), "v"), -ok(vega(&long), "v"), "vega {ctx}");
            assert_eq!(
                ok(theta(&short), "t"),
                -ok(theta(&long), "t"),
                "theta {ctx}"
            );
            assert_eq!(ok(rho(&short), "r"), -ok(rho(&long), "r"), "rho {ctx}");
            assert_eq!(
                ok(delta_b76(&short), "d76"),
                -ok(delta_b76(&long), "d76"),
                "b76 delta {ctx}"
            );
            assert_eq!(
                ok(gamma_b76(&short), "g76"),
                -ok(gamma_b76(&long), "g76"),
                "b76 gamma {ctx}"
            );
            assert_eq!(
                ok(delta_gk(&short), "dgk"),
                -ok(delta_gk(&long), "dgk"),
                "gk delta {ctx}"
            );
        }
    }
}

/// CRR price of a `S = 100, K = 105, σ = 25 %, r = 5 %, T = 0.5` option on
/// 200 steps.
fn binomial_side_price(option_type: &OptionType, style: &OptionStyle, side: &Side) -> Decimal {
    ok(
        price_binomial(BinomialPricingParams {
            asset: pos_or_panic!(100.0),
            volatility: pos_or_panic!(0.25),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(105.0),
            expiry: pos_or_panic!(0.5),
            no_steps: step_count(200),
            option_type,
            option_style: style,
            side,
        }),
        "binomial",
    )
}

/// A short American option is the negated long one: the writer does not
/// hold the exercise right (#648). The CRR backward induction used to apply
/// `max(continuation, intrinsic)` to the side-signed values, so the short
/// position "exercised" whenever that lifted it to zero: at `S = 100,
/// K = 105, σ = 25 %, r = 5 %, T = 0.5`, 200 steps, the long American call
/// is `5.9823` and the short one was `0` instead of `-5.9823`.
#[test]
fn test_binomial_american_short_price_equals_negated_long() {
    for style in [OptionStyle::Call, OptionStyle::Put] {
        let american = OptionType::American;
        assert_eq!(
            binomial_side_price(&american, &style, &Side::Short),
            -binomial_side_price(&american, &style, &Side::Long),
            "binomial American {style:?}"
        );
    }
}

/// The binomial tree is signed by `side` node by node, and
/// `calculate_price_binomial_tree` reads its root without negating it again
/// (#648): a short tree is the negated long tree for European and American
/// options, and its price matches `price_binomial`.
#[test]
fn test_binomial_tree_short_is_negated_long_tree() {
    for option_type in [OptionType::European, OptionType::American] {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let tree = |side: &Side| {
                ok(
                    generate_binomial_tree(&BinomialPricingParams {
                        asset: pos_or_panic!(100.0),
                        volatility: pos_or_panic!(0.25),
                        int_rate: dec!(0.05),
                        strike: pos_or_panic!(105.0),
                        expiry: pos_or_panic!(0.5),
                        no_steps: step_count(50),
                        option_type: &option_type,
                        option_style: &style,
                        side,
                    }),
                    "binomial tree",
                )
                .1
            };
            let long = tree(&Side::Long);
            let short = tree(&Side::Short);
            for (long_row, short_row) in long.iter().zip(short.iter()) {
                for (l, s) in long_row.iter().zip(short_row.iter()) {
                    assert_eq!(*s, -*l, "{option_type:?} {style:?} tree node");
                }
            }
            let short_option = option(
                option_type.clone(),
                style,
                Side::Short,
                100.0,
                105.0,
                182.5,
                0.25,
                dec!(0.05),
                0.0,
                None,
            );
            let mut long_option = short_option.clone();
            long_option.side = Side::Long;
            let short_price = ok(
                short_option.calculate_price_binomial_tree(step_count(50)),
                "short tree price",
            )
            .0;
            let long_price = ok(
                long_option.calculate_price_binomial_tree(step_count(50)),
                "long tree price",
            )
            .0;
            assert!(short_price < Decimal::ZERO, "{option_type:?} {style:?}");
            assert_eq!(short_price, -long_price, "{option_type:?} {style:?}");
        }
    }
}

/// Short European binomial and exotic prices are the negated long prices.
#[test]
fn test_lattice_and_exotic_short_price_equals_negated_long() {
    for style in [OptionStyle::Call, OptionStyle::Put] {
        let european_type = OptionType::European;
        assert_eq!(
            binomial_side_price(&european_type, &style, &Side::Short),
            -binomial_side_price(&european_type, &style, &Side::Long),
            "binomial European {style:?}"
        );
    }
    let exotics = [
        OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        },
        OptionType::Binary {
            binary_type: BinaryType::AssetOrNothing,
        },
        OptionType::Binary {
            binary_type: BinaryType::Gap,
        },
        OptionType::Chooser {
            choice_date: pos_or_panic!(60.0),
        },
        OptionType::Power {
            exponent: Positive::ONE,
        },
        OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        },
        OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        },
        OptionType::Lookback {
            lookback_type: LookbackType::FixedStrike,
        },
        OptionType::Lookback {
            lookback_type: LookbackType::FloatingStrike,
        },
        OptionType::Exchange {
            second_asset: pos_or_panic!(95.0),
        },
        OptionType::Spread {
            second_asset: pos_or_panic!(5.0),
        },
        OptionType::Quanto {
            exchange_rate: Positive::ONE,
        },
        OptionType::Cliquet {
            reset_dates: vec![pos_or_panic!(60.0), pos_or_panic!(120.0)],
        },
        OptionType::Rainbow {
            num_assets: 2,
            rainbow_type: RainbowType::BestOf,
        },
        OptionType::Compound {
            underlying_option: Box::new(OptionType::European),
        },
    ];
    for exotic in exotics {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let long = exotic_for_side(exotic.clone(), style, Side::Long);
            let short = exotic_for_side(exotic.clone(), style, Side::Short);
            assert_eq!(
                ok(black_scholes(&short), "short"),
                -ok(black_scholes(&long), "long"),
                "{exotic:?} {style:?}"
            );
        }
    }
}

/// `S = 100, K = 105, T = 0.5, σ = 25 %, r = 5 %, q = 1 %` with every
/// multi-asset parameter populated.
fn exotic_for_side(option_type: OptionType, style: OptionStyle, side: Side) -> Options {
    let params = ExoticParams {
        exchange_second_asset_volatility: Some(pos_or_panic!(0.2)),
        exchange_correlation: Some(dec!(0.3)),
        spread_second_asset_volatility: Some(pos_or_panic!(0.2)),
        spread_correlation: Some(dec!(0.3)),
        quanto_fx_volatility: Some(pos_or_panic!(0.1)),
        quanto_fx_correlation: Some(dec!(0.2)),
        rainbow_second_asset_price: Some(pos_or_panic!(100.0)),
        rainbow_second_asset_volatility: Some(pos_or_panic!(0.2)),
        rainbow_correlation: Some(dec!(0.3)),
        ..Default::default()
    };
    option(
        option_type,
        style,
        side,
        100.0,
        105.0,
        182.5,
        0.25,
        dec!(0.05),
        0.01,
        Some(params),
    )
}

// ---------------------------------------------------------------------------
// Greek identities
// ---------------------------------------------------------------------------

/// `Δc - Δp = e^(-qT)`, `Γc = Γp`, `Vc = Vp` for Black-Scholes-Merton.
///
/// Gamma and vega do not depend on the style, so equality is exact. The
/// delta difference subtracts two products of the same `f64` CDF value and
/// meets `e^(-qT)` to [`IDENTITY_TOL`].
#[test]
fn test_black_scholes_delta_gamma_vega_identities_grid_hold() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        let call = european(OptionStyle::Call, Side::Long, s, k, days, vol, r, q);
        let put = european(OptionStyle::Put, Side::Long, s, k, days, vol, r, q);
        let ctx = format!("s={s} k={k} vol={vol} r={r} days={days} q={q}");
        let expected = discount(pos_or_panic!(q).to_dec(), years(&call));
        assert_close(
            ok(delta(&call), "delta call") - ok(delta(&put), "delta put"),
            expected,
            IDENTITY_TOL,
            &format!("delta parity {ctx}"),
        );
        assert_eq!(ok(gamma(&call), "g"), ok(gamma(&put), "g"), "gamma {ctx}");
        assert_eq!(ok(vega(&call), "v"), ok(vega(&put), "v"), "vega {ctx}");
    });
}

/// `ρc - ρp = K T e^(-rT)` (per 1 %, so divided by 100) and
/// `Θc - Θp = q S e^(-qT) - r K e^(-rT)` (per calendar day, so divided by
/// 365). Both follow from differentiating put-call parity.
#[test]
fn test_black_scholes_rho_and_theta_parity_grid_hold() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        let call = european(OptionStyle::Call, Side::Long, s, k, days, vol, r, q);
        let put = european(OptionStyle::Put, Side::Long, s, k, days, vol, r, q);
        let ctx = format!("s={s} k={k} vol={vol} r={r} days={days} q={q}");
        let t = years(&call);
        let s_dec = pos_or_panic!(s).to_dec();
        let k_dec = pos_or_panic!(k).to_dec();
        let q_dec = pos_or_panic!(q).to_dec();
        let rho_expected = k_dec * t * discount(r, t) / dec!(100);
        assert_close(
            ok(rho(&call), "rho call") - ok(rho(&put), "rho put"),
            rho_expected,
            IDENTITY_TOL,
            &format!("rho parity {ctx}"),
        );
        let theta_expected =
            (q_dec * s_dec * discount(q_dec, t) - r * k_dec * discount(r, t)) / dec!(365);
        assert_close(
            ok(theta(&call), "theta call") - ok(theta(&put), "theta put"),
            theta_expected,
            IDENTITY_TOL,
            &format!("theta parity {ctx}"),
        );
    });
}

/// Black-76: `Δc - Δp = e^(-rT)`, `Γc = Γp`, `Vc = Vp`, `ρ` and `Θ` agree
/// with the forward parity `C - P = e^(-rT)(F - K)` differentiated.
#[test]
fn test_black_76_greek_identities_grid_hold() {
    for_each_grid_point(|f, k, vol, r, days, _| {
        let call = european(OptionStyle::Call, Side::Long, f, k, days, vol, r, 0.0);
        let put = european(OptionStyle::Put, Side::Long, f, k, days, vol, r, 0.0);
        let ctx = format!("f={f} k={k} vol={vol} r={r} days={days}");
        let t = years(&call);
        assert_close(
            ok(delta_b76(&call), "d") - ok(delta_b76(&put), "d"),
            discount(r, t),
            IDENTITY_TOL,
            &format!("b76 delta parity {ctx}"),
        );
        assert_eq!(ok(gamma_b76(&call), "g"), ok(gamma_b76(&put), "g"), "{ctx}");
        assert_eq!(ok(vega_b76(&call), "v"), ok(vega_b76(&put), "v"), "{ctx}");
        // d/dr of e^(-rT)(F - K) is -T e^(-rT)(F - K); per 1 %.
        let forward_gap = pos_or_panic!(f).to_dec() - pos_or_panic!(k).to_dec();
        assert_close(
            ok(rho_b76(&call), "r") - ok(rho_b76(&put), "r"),
            -t * discount(r, t) * forward_gap / dec!(100),
            IDENTITY_TOL,
            &format!("b76 rho parity {ctx}"),
        );
        // -d/dT of e^(-rT)(F - K) is r e^(-rT)(F - K); per calendar day.
        assert_close(
            ok(theta_b76(&call), "t") - ok(theta_b76(&put), "t"),
            r * discount(r, t) * forward_gap / dec!(365),
            IDENTITY_TOL,
            &format!("b76 theta parity {ctx}"),
        );
    });
}

/// Garman-Kohlhagen: `Δc - Δp = e^(-r_f T)`, `Γc = Γp`, `Vc = Vp`.
#[test]
fn test_garman_kohlhagen_greek_identities_grid_hold() {
    for &rf in &[0.0, 0.02, 0.05] {
        for &(s, k) in &[(1.05, 1.10), (1.10, 1.10), (1.25, 1.10)] {
            for &days in &DAYS {
                let call = european(
                    OptionStyle::Call,
                    Side::Long,
                    s,
                    k,
                    days,
                    0.12,
                    dec!(0.03),
                    rf,
                );
                let put = european(
                    OptionStyle::Put,
                    Side::Long,
                    s,
                    k,
                    days,
                    0.12,
                    dec!(0.03),
                    rf,
                );
                let ctx = format!("s={s} k={k} rf={rf} days={days}");
                assert_close(
                    ok(delta_gk(&call), "d") - ok(delta_gk(&put), "d"),
                    discount(pos_or_panic!(rf).to_dec(), years(&call)),
                    IDENTITY_TOL,
                    &format!("gk delta parity {ctx}"),
                );
                assert_eq!(ok(gamma_gk(&call), "g"), ok(gamma_gk(&put), "g"), "{ctx}");
                assert_eq!(ok(vega_gk(&call), "v"), ok(vega_gk(&put), "v"), "{ctx}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Exotic decompositions
// ---------------------------------------------------------------------------

/// Binary decompositions (Haug §4.19): asset-or-nothing call minus `K`
/// unit cash-or-nothing calls is the vanilla call; the gap option the
/// library defines with the same strike is that same difference, and the gap
/// put is `K` cash-or-nothing puts minus the asset-or-nothing put, the
/// vanilla put (#649); a binary call plus its put pays the payout for sure,
/// so cash call + cash put = `e^(-rT)` and asset call + asset put =
/// `S e^(-qT)`.
#[test]
fn test_binary_decompositions_grid_hold() {
    let binary = |binary_type| OptionType::Binary { binary_type };
    for_each_grid_point(|s, k, vol, r, days, q| {
        let ctx = format!("s={s} k={k} vol={vol} r={r} days={days} q={q}");
        let price = |option_type: OptionType, style| {
            ok(
                black_scholes(&option(
                    option_type,
                    style,
                    Side::Long,
                    s,
                    k,
                    days,
                    vol,
                    r,
                    q,
                    None,
                )),
                "binary",
            )
        };
        let aon_call = price(binary(BinaryType::AssetOrNothing), OptionStyle::Call);
        let aon_put = price(binary(BinaryType::AssetOrNothing), OptionStyle::Put);
        let con_call = price(binary(BinaryType::CashOrNothing), OptionStyle::Call);
        let con_put = price(binary(BinaryType::CashOrNothing), OptionStyle::Put);
        let gap_call = price(binary(BinaryType::Gap), OptionStyle::Call);
        let gap_put = price(binary(BinaryType::Gap), OptionStyle::Put);
        let vanilla_call = price(OptionType::European, OptionStyle::Call);
        let vanilla_put = price(OptionType::European, OptionStyle::Put);
        let k_dec = pos_or_panic!(k).to_dec();
        let t = {
            let probe = european(OptionStyle::Call, Side::Long, s, k, days, vol, r, q);
            years(&probe)
        };
        assert_close(
            aon_call - k_dec * con_call,
            vanilla_call,
            IDENTITY_TOL,
            &format!("aon - K con = call {ctx}"),
        );
        assert_close(gap_call, vanilla_call, IDENTITY_TOL, &format!("gap {ctx}"));
        assert_close(
            k_dec * con_put - aon_put,
            vanilla_put,
            IDENTITY_TOL,
            &format!("K con - aon = put {ctx}"),
        );
        assert_close(
            gap_put,
            vanilla_put,
            IDENTITY_TOL,
            &format!("gap put {ctx}"),
        );
        assert_close(
            con_call + con_put,
            discount(r, t),
            IDENTITY_TOL,
            &format!("con call + put {ctx}"),
        );
        assert_close(
            aon_call + aon_put,
            pos_or_panic!(s).to_dec() * discount(pos_or_panic!(q).to_dec(), t),
            IDENTITY_TOL,
            &format!("aon call + put {ctx}"),
        );
    });
}

/// A gap put struck at the gap level pays `K - S_T` below `K`, i.e. it is
/// the vanilla put (Haug §4.19.3 with `X1 = X2`), on both sides (#649).
/// `gap_binary_price` used to form `asset - K * cash` for both styles, the
/// negated put: at `S = 100, K = 105, T = 0.5, σ = 25 %, r = 5 %, q = 1 %` a
/// long gap put priced `-8.6574` against a vanilla put of `8.6574`.
#[test]
fn test_binary_gap_put_equals_vanilla_put() {
    for side in [Side::Long, Side::Short] {
        let gap = exotic_for_side(
            OptionType::Binary {
                binary_type: BinaryType::Gap,
            },
            OptionStyle::Put,
            side,
        );
        let vanilla = exotic_for_side(OptionType::European, OptionStyle::Put, side);
        assert_close(
            ok(black_scholes(&gap), "gap put"),
            ok(black_scholes(&vanilla), "vanilla put"),
            IDENTITY_TOL,
            &format!("gap put {side:?}"),
        );
    }
    let long = exotic_for_side(
        OptionType::Binary {
            binary_type: BinaryType::Gap,
        },
        OptionStyle::Put,
        Side::Long,
    );
    assert_close(
        ok(black_scholes(&long), "gap put"),
        dec!(8.6574),
        dec!(0.0001),
        "gap put reference",
    );
}

fn barrier(barrier_type: BarrierType, level: f64) -> OptionType {
    OptionType::Barrier {
        barrier_type,
        barrier_level: pos_or_panic!(level),
        rebate: None,
    }
}

/// In-out parity without rebate (Haug §4.17.1): knock-in + knock-out =
/// vanilla, for down (H = 95) and up (H = 105) barriers, calls and puts,
/// strikes on both sides of the barrier. Haug's Table 4-13 parameters:
/// `S = 100, T = 0.5, r = 8 %, b = 4 % (q = 4 %)`.
#[test]
fn test_barrier_in_out_parity_without_rebate_grid_holds() {
    let pairs = [
        (BarrierType::DownAndIn, BarrierType::DownAndOut, 95.0),
        (BarrierType::UpAndIn, BarrierType::UpAndOut, 105.0),
    ];
    for (knock_in, knock_out, level) in pairs {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for &k in &STRIKES {
                for &vol in &[0.25, 0.30] {
                    let price = |option_type: OptionType| {
                        ok(
                            black_scholes(&option(
                                option_type,
                                style,
                                Side::Long,
                                100.0,
                                k,
                                182.5,
                                vol,
                                dec!(0.08),
                                0.04,
                                None,
                            )),
                            "barrier",
                        )
                    };
                    let sum = price(barrier(knock_in, level)) + price(barrier(knock_out, level));
                    assert_close(
                        sum,
                        price(OptionType::European),
                        IDENTITY_TOL,
                        &format!("{knock_in:?}+{knock_out:?} {style:?} k={k} vol={vol}"),
                    );
                }
            }
        }
    }
}

/// A simple chooser whose choice date is the expiry is a straddle
/// (Haug §4.12.1: at `t = T` the holder keeps whichever leg pays).
#[test]
fn test_chooser_at_expiry_equals_straddle() {
    for &(s, k) in &[(90.0, 100.0), (100.0, 100.0), (115.0, 100.0)] {
        for &r in &RATES {
            let chooser = option(
                OptionType::Chooser {
                    choice_date: pos_or_panic!(182.5),
                },
                OptionStyle::Call,
                Side::Long,
                s,
                k,
                182.5,
                0.3,
                r,
                0.02,
                None,
            );
            let call = european(OptionStyle::Call, Side::Long, s, k, 182.5, 0.3, r, 0.02);
            let put = european(OptionStyle::Put, Side::Long, s, k, 182.5, 0.3, r, 0.02);
            let straddle = ok(black_scholes(&call), "call") + ok(black_scholes(&put), "put");
            assert_close(
                ok(black_scholes(&chooser), "chooser"),
                straddle,
                IDENTITY_TOL,
                &format!("chooser s={s} k={k} r={r}"),
            );
        }
    }
}

fn exchange_params(vol2: f64, q2: f64, rho: Decimal) -> ExoticParams {
    ExoticParams {
        exchange_second_asset_volatility: Some(pos_or_panic!(vol2)),
        exchange_second_asset_dividend: Some(pos_or_panic!(q2)),
        exchange_correlation: Some(rho),
        ..Default::default()
    }
}

/// Margrabe parity: the option to exchange asset 2 for asset 1 minus the
/// option to exchange asset 1 for asset 2 is the forward exchange,
/// `S1 e^(-q1 T) - S2 e^(-q2 T)` (Margrabe 1978; Haug §5.1).
///
/// The `exchange.rs` module docs state the *sum* equals `S1 + S2`; the
/// payoffs `max(S1 - S2, 0)` and `max(S2 - S1, 0)` sum to `|S1 - S2|`, so the
/// difference is the identity asserted here.
#[test]
fn test_exchange_margrabe_parity_grid_holds() {
    for &(s1, s2) in &[(22.0, 20.0), (100.0, 100.0), (80.0, 95.0)] {
        for &(vol1, vol2, rho) in &[
            (0.20, 0.25, dec!(-0.5)),
            (0.30, 0.15, dec!(0.0)),
            (0.25, 0.25, dec!(0.8)),
        ] {
            let (q1, q2) = (0.06, 0.04);
            let one_for_two = option(
                OptionType::Exchange {
                    second_asset: pos_or_panic!(s2),
                },
                OptionStyle::Call,
                Side::Long,
                s1,
                1.0,
                91.25,
                vol1,
                dec!(0.1),
                q1,
                Some(exchange_params(vol2, q2, rho)),
            );
            let two_for_one = option(
                OptionType::Exchange {
                    second_asset: pos_or_panic!(s1),
                },
                OptionStyle::Call,
                Side::Long,
                s2,
                1.0,
                91.25,
                vol2,
                dec!(0.1),
                q2,
                Some(exchange_params(vol1, q1, rho)),
            );
            let t = years(&one_for_two);
            let forward = pos_or_panic!(s1).to_dec() * discount(pos_or_panic!(q1).to_dec(), t)
                - pos_or_panic!(s2).to_dec() * discount(pos_or_panic!(q2).to_dec(), t);
            assert_close(
                ok(black_scholes(&one_for_two), "1 for 2")
                    - ok(black_scholes(&two_for_one), "2 for 1"),
                forward,
                IDENTITY_TOL,
                &format!("margrabe s1={s1} s2={s2} vol1={vol1} vol2={vol2} rho={rho}"),
            );
        }
    }
}

/// A power option with exponent 1 is a vanilla option.
#[test]
fn test_power_option_with_unit_exponent_equals_black_scholes_grid() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let power = option(
                OptionType::Power {
                    exponent: Positive::ONE,
                },
                style,
                Side::Long,
                s,
                k,
                days,
                vol,
                r,
                q,
                None,
            );
            let vanilla = european(style, Side::Long, s, k, days, vol, r, q);
            assert_close(
                ok(black_scholes(&power), "power"),
                ok(black_scholes(&vanilla), "vanilla"),
                IDENTITY_TOL,
                &format!("power n=1 s={s} k={k} vol={vol} r={r} days={days} q={q} {style:?}"),
            );
        }
    });
}

/// Kemna-Vorst (1990; Haug §4.20.1): a continuously monitored geometric
/// average-rate option is Black-Scholes with `σ_A = σ/√3`, carry
/// `b_A = (r - q - σ²/6)/2` and discounting at `r`, i.e. Black-Scholes-Merton
/// with yield `q_A = r - b_A`.
#[test]
fn test_geometric_asian_equals_kemna_vorst_adjusted_black_scholes() {
    for &(s, k) in &[(90.0, 100.0), (100.0, 100.0), (110.0, 100.0)] {
        for &vol in &[0.15, 0.30] {
            for &(r, q) in &[(dec!(0.05), 0.0), (dec!(0.08), 0.04)] {
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    let asian = option(
                        OptionType::Asian {
                            averaging_type: AsianAveragingType::Geometric,
                        },
                        style,
                        Side::Long,
                        s,
                        k,
                        182.5,
                        vol,
                        r,
                        q,
                        None,
                    );
                    let sigma = pos_or_panic!(vol).to_dec();
                    let q_dec = pos_or_panic!(q).to_dec();
                    let carry = (r - q_dec - sigma * sigma / dec!(6)) / dec!(2);
                    let sigma_a = match (sigma * sigma / dec!(3)).sqrt() {
                        Some(value) => value,
                        None => panic!("sqrt of a positive variance"),
                    };
                    let reduced = Options::new(
                        OptionType::European,
                        Side::Long,
                        "TEST".to_string(),
                        pos_or_panic!(k),
                        ExpirationDate::Days(pos_or_panic!(182.5)),
                        ok(Positive::new_decimal(sigma_a), "sigma_a"),
                        Positive::ONE,
                        pos_or_panic!(s),
                        r,
                        style,
                        ok(Positive::new_decimal(r - carry), "q_a"),
                        None,
                    );
                    assert_close(
                        ok(black_scholes(&asian), "asian"),
                        ok(black_scholes(&reduced), "reduced"),
                        IDENTITY_TOL,
                        &format!("kemna-vorst s={s} k={k} vol={vol} r={r} q={q} {style:?}"),
                    );
                }
            }
        }
    }
}

fn quanto_params(foreign_rate: Decimal) -> ExoticParams {
    ExoticParams {
        quanto_fx_volatility: Some(pos_or_panic!(0.10)),
        quanto_fx_correlation: Some(Decimal::ZERO),
        quanto_foreign_rate: Some(foreign_rate),
        ..Default::default()
    }
}

/// With zero asset/FX correlation, a unit fixed rate and `r_f = r_d`, the
/// quanto adjustment vanishes and the price is Black-Scholes-Merton
/// (Haug §5.16.1 with `ρ = 0`, `E_p = 1`, `r_f = r`).
#[test]
fn test_quanto_without_correlation_equals_black_scholes_merton() {
    for_each_grid_point(|s, k, vol, r, days, q| {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let quanto = option(
                OptionType::Quanto {
                    exchange_rate: Positive::ONE,
                },
                style,
                Side::Long,
                s,
                k,
                days,
                vol,
                r,
                q,
                Some(quanto_params(r)),
            );
            let vanilla = european(style, Side::Long, s, k, days, vol, r, q);
            assert_close(
                ok(black_scholes(&quanto), "quanto"),
                ok(black_scholes(&vanilla), "vanilla"),
                IDENTITY_TOL,
                &format!("quanto s={s} k={k} vol={vol} r={r} days={days} q={q} {style:?}"),
            );
        }
    });
}

fn spread_params(vol2: f64, q2: f64, rho: Decimal) -> ExoticParams {
    ExoticParams {
        spread_second_asset_volatility: Some(pos_or_panic!(vol2)),
        spread_second_asset_dividend: Some(pos_or_panic!(q2)),
        spread_correlation: Some(rho),
        ..Default::default()
    }
}

fn spread_call(strike: f64, q1: f64, q2: f64) -> Options {
    option(
        OptionType::Spread {
            second_asset: pos_or_panic!(95.0),
        },
        OptionStyle::Call,
        Side::Long,
        100.0,
        strike,
        182.5,
        0.30,
        dec!(0.05),
        q1,
        Some(spread_params(0.25, q2, dec!(0.4))),
    )
}

/// Kirk's approximation (Haug §5.4.2) collapses onto Margrabe's formula as
/// the strike vanishes: at `K = 0` the Kirk weight `F2/(F2 + K)` is one. The
/// library switches to Margrabe below `|K| < 1e-4`. `spread_black_scholes`
/// is called directly because the `black_scholes` dispatcher validates a
/// zero strike before it reaches the spread branch. On a futures-style input
/// (`q1 = q2 = r`, so spots are forwards) the two branches meet. A strike of
/// `1e-3` moves the price by `O(K)`, hence the `2e-3` bound.
#[test]
fn test_spread_kirk_tends_to_margrabe_as_strike_vanishes() {
    let kirk = ok(
        spread_black_scholes(&spread_call(0.001, 0.05, 0.05)),
        "kirk",
    );
    let margrabe = ok(
        spread_black_scholes(&spread_call(0.0, 0.05, 0.05)),
        "margrabe",
    );
    assert_close(
        kirk,
        margrabe,
        dec!(0.002),
        "kirk vs margrabe, futures-style",
    );
}

/// Merton (1973): without dividends an American call is never exercised
/// early, so the CRR American and European calls coincide at every step
/// count; the American put carries a non-negative early-exercise premium.
#[test]
fn test_binomial_american_call_without_dividend_equals_european() {
    for &k in &STRIKES {
        for steps in [10usize, 51, 150] {
            let no_steps = step_count(steps);
            let price = |option_type: &OptionType, style: &OptionStyle| {
                ok(
                    price_binomial(BinomialPricingParams {
                        asset: pos_or_panic!(100.0),
                        volatility: pos_or_panic!(0.3),
                        int_rate: dec!(0.05),
                        strike: pos_or_panic!(k),
                        expiry: Positive::ONE,
                        no_steps,
                        option_type,
                        option_style: style,
                        side: &Side::Long,
                    }),
                    "binomial",
                )
            };
            let call = OptionStyle::Call;
            let put = OptionStyle::Put;
            assert_close(
                price(&OptionType::American, &call),
                price(&OptionType::European, &call),
                IDENTITY_TOL,
                &format!("american call k={k} n={steps}"),
            );
            assert!(
                price(&OptionType::American, &put) >= price(&OptionType::European, &put),
                "american put below european at k={k} n={steps}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------

/// The closed forms divide by `σ√T`; at `T = 0` or `σ = 0` they report a
/// `PricingError` rather than a limit value. Asserted so the behaviour is
/// pinned: a silent switch to a limit would be a numerical change.
#[test]
fn test_closed_forms_at_zero_time_or_zero_volatility_return_error() {
    for style in [OptionStyle::Call, OptionStyle::Put] {
        let expired = european(style, Side::Long, 110.0, 100.0, 0.0, 0.2, dec!(0.05), 0.0);
        let frozen = european(style, Side::Long, 110.0, 100.0, 30.0, 0.0, dec!(0.05), 0.0);
        for opt in [&expired, &frozen] {
            assert!(black_scholes(opt).is_err(), "bs {style:?}");
            assert!(black_76(opt).is_err(), "b76 {style:?}");
            assert!(garman_kohlhagen(opt).is_err(), "gk {style:?}");
        }
    }
}

/// Deep in/out-of-the-money asymptotes: a call far in the money is worth its
/// forward intrinsic `S e^(-qT) - K e^(-rT)`, one far out of the money is
/// worth (numerically) nothing, and the put mirrors both.
#[test]
fn test_black_scholes_deep_moneyness_asymptotes() {
    let r = dec!(0.05);
    let deep_itm_call = european(
        OptionStyle::Call,
        Side::Long,
        200.0,
        50.0,
        182.5,
        0.1,
        r,
        0.02,
    );
    let deep_otm_put = european(
        OptionStyle::Put,
        Side::Long,
        200.0,
        50.0,
        182.5,
        0.1,
        r,
        0.02,
    );
    let t = years(&deep_itm_call);
    let forward_intrinsic = dec!(200) * discount(dec!(0.02), t) - dec!(50) * discount(r, t);
    assert_close(
        ok(black_scholes(&deep_itm_call), "itm call"),
        forward_intrinsic,
        IDENTITY_TOL,
        "deep itm call",
    );
    assert_close(
        ok(black_scholes(&deep_otm_put), "otm put"),
        Decimal::ZERO,
        IDENTITY_TOL,
        "deep otm put",
    );
    let deep_itm_put = european(
        OptionStyle::Put,
        Side::Long,
        50.0,
        200.0,
        182.5,
        0.1,
        r,
        0.02,
    );
    let deep_otm_call = european(
        OptionStyle::Call,
        Side::Long,
        50.0,
        200.0,
        182.5,
        0.1,
        r,
        0.02,
    );
    assert_close(
        ok(black_scholes(&deep_itm_put), "itm put"),
        dec!(200) * discount(r, t) - dec!(50) * discount(dec!(0.02), t),
        IDENTITY_TOL,
        "deep itm put",
    );
    assert_close(
        ok(black_scholes(&deep_otm_call), "otm call"),
        Decimal::ZERO,
        IDENTITY_TOL,
        "deep otm call",
    );
}

/// Black-Scholes delta defines both limits: at `T = 0` it is the exercise
/// indicator (`±1` in the money, `0` out), at `σ = 0` the same indicator on
/// spot moneyness.
#[test]
fn test_black_scholes_delta_zero_time_and_zero_volatility_limits() {
    let cases = [
        (OptionStyle::Call, 110.0, Decimal::ONE),
        (OptionStyle::Call, 90.0, Decimal::ZERO),
        (OptionStyle::Put, 90.0, Decimal::NEGATIVE_ONE),
        (OptionStyle::Put, 110.0, Decimal::ZERO),
    ];
    for (style, spot, expected) in cases {
        let expired = european(style, Side::Long, spot, 100.0, 0.0, 0.2, dec!(0.05), 0.0);
        let frozen = european(style, Side::Long, spot, 100.0, 30.0, 0.0, dec!(0.05), 0.0);
        assert_eq!(
            ok(delta(&expired), "delta T=0"),
            expected,
            "T=0 {style:?} s={spot}"
        );
        assert_eq!(
            ok(delta(&frozen), "delta σ=0"),
            expected,
            "σ=0 {style:?} s={spot}"
        );
    }
}

fn binomial_price(
    option_type: &OptionType,
    style: &OptionStyle,
    side: &Side,
    spot: f64,
    expiry: Positive,
    volatility: Positive,
) -> Decimal {
    ok(
        price_binomial(BinomialPricingParams {
            asset: pos_or_panic!(spot),
            volatility,
            int_rate: dec!(0.10),
            strike: pos_or_panic!(100.0),
            expiry,
            no_steps: step_count(100),
            option_type,
            option_style: style,
            side,
        }),
        "binomial",
    )
}

/// Binomial at `T = 0` returns the signed intrinsic value, at `σ = 0` the
/// discounted deterministic payoff `max(S - K e^(-rT), 0)` for a European
/// call, and for an American put the better of exercising now and holding:
/// at `S = 80, K = 100, r = 10 %, T = 1` immediate exercise (`20`) beats the
/// held value `100 e^(-0.1) - 80 = 10.4837`.
#[test]
fn test_binomial_zero_time_and_zero_volatility_limits() {
    let european_type = OptionType::European;
    let american = OptionType::American;
    let call = OptionStyle::Call;
    let put = OptionStyle::Put;
    let one_year = Positive::ONE;
    let vol = pos_or_panic!(0.3);
    assert_eq!(
        binomial_price(
            &european_type,
            &call,
            &Side::Long,
            110.0,
            Positive::ZERO,
            vol
        ),
        dec!(10)
    );
    assert_eq!(
        binomial_price(
            &european_type,
            &call,
            &Side::Short,
            110.0,
            Positive::ZERO,
            vol
        ),
        dec!(-10)
    );
    assert_eq!(
        binomial_price(
            &european_type,
            &put,
            &Side::Long,
            110.0,
            Positive::ZERO,
            vol
        ),
        Decimal::ZERO
    );
    assert_close(
        binomial_price(
            &european_type,
            &call,
            &Side::Long,
            95.0,
            one_year,
            Positive::ZERO,
        ),
        dec!(95) - dec!(100) * discount(dec!(0.10), Decimal::ONE),
        IDENTITY_TOL,
        "european call at zero vol",
    );
    assert_close(
        binomial_price(&american, &put, &Side::Long, 80.0, one_year, Positive::ZERO),
        dec!(20),
        IDENTITY_TOL,
        "american put at zero vol exercises now",
    );
}

/// Barone-Adesi-Whaley at `T = 0` is the intrinsic value; at `σ = 0` an
/// American call on a non-dividend underlying is `max(S - K e^(-rT), 0)`.
#[test]
fn test_barone_adesi_whaley_zero_time_and_zero_volatility_call_limits() {
    let call = OptionStyle::Call;
    let put = OptionStyle::Put;
    let at_expiry = |spot: f64, style: &OptionStyle| {
        ok(
            barone_adesi_whaley(
                pos_or_panic!(spot),
                pos_or_panic!(100.0),
                Positive::ZERO,
                dec!(0.10),
                Positive::ZERO,
                pos_or_panic!(0.3),
                style,
            ),
            "baw T=0",
        )
    };
    assert_eq!(at_expiry(110.0, &call), dec!(10));
    assert_eq!(at_expiry(110.0, &put), Decimal::ZERO);
    assert_eq!(at_expiry(85.0, &put), dec!(15));
    let frozen_call = ok(
        barone_adesi_whaley(
            pos_or_panic!(95.0),
            pos_or_panic!(100.0),
            Positive::ONE,
            dec!(0.10),
            Positive::ZERO,
            Positive::ZERO,
            &call,
        ),
        "baw σ=0 call",
    );
    assert_close(
        frozen_call,
        dec!(95) - dec!(100) * discount(dec!(0.10), Decimal::ONE),
        IDENTITY_TOL,
        "baw call at zero vol",
    );
}

/// An American put can always be exercised, so it is worth at least its
/// intrinsic value; the binomial `σ = 0` branch honours this. Barone-Adesi-
/// Whaley's `σ = 0` branch used to return the European value
/// `max(K e^(-rT) - S e^(-qT), 0)`: at `S = 80, K = 100, r = 10 %, T = 1` it
/// priced `10.4837`, below the intrinsic `20` (#648).
#[test]
fn test_barone_adesi_whaley_zero_volatility_put_is_at_least_intrinsic() {
    let frozen_put = ok(
        barone_adesi_whaley(
            pos_or_panic!(80.0),
            pos_or_panic!(100.0),
            Positive::ONE,
            dec!(0.10),
            Positive::ZERO,
            Positive::ZERO,
            &OptionStyle::Put,
        ),
        "baw σ=0 put",
    );
    assert!(
        frozen_put >= dec!(20),
        "american put {frozen_put} below intrinsic 20"
    );
    // With `q = 0` the put's exercise value `K e^(-rτ) - S` falls in `τ`,
    // so exercising now is the optimum: exactly the intrinsic value.
    assert_eq!(frozen_put, dec!(20));
}

/// At `σ = 0` the exercise value `K e^(-rτ) - S e^(-qτ)` of a put has an
/// interior maximum at `τ* = ln(rK / (qS)) / (r - q)` when `r < q`, and the
/// call's has one when `r > q` (#648). `S = K = 100, T = 50`:
/// with `r = 1 %, q = 5 %` (put) or `r = 5 %, q = 1 %` (call), `τ* = 40.236`
/// and the value is `100 (e^(-0.40236) - e^(-2.01180)) = 53.4992`, above
/// both endpoints (`0` now, `52.4446` at expiry). Recomputed independently
/// in `f64`.
#[test]
fn test_barone_adesi_whaley_zero_volatility_interior_exercise_optimum() {
    let frozen = |rate: Decimal, dividend: f64, style: &OptionStyle| {
        ok(
            barone_adesi_whaley(
                Positive::HUNDRED,
                Positive::HUNDRED,
                pos_or_panic!(50.0),
                rate,
                pos_or_panic!(dividend),
                Positive::ZERO,
                style,
            ),
            "baw σ=0 interior",
        )
    };
    let put = frozen(dec!(0.01), 0.05, &OptionStyle::Put);
    let call = frozen(dec!(0.05), 0.01, &OptionStyle::Call);
    assert_close(put, dec!(53.49922439811375), dec!(0.000001), "put τ*");
    assert_close(call, dec!(53.49922439811375), dec!(0.000001), "call τ*");
}

/// With `q = 0` Barone-Adesi-Whaley returns the European call, which a
/// negative rate pushes below `S - K`; the American floor restores the
/// intrinsic value (#648). `S = 150, K = 100, σ = 10 %, r = -2 %, T = 1`:
/// European `47.9800`, intrinsic `50`.
#[test]
fn test_barone_adesi_whaley_negative_rate_call_is_floored_at_intrinsic() {
    let call = ok(
        barone_adesi_whaley(
            pos_or_panic!(150.0),
            Positive::HUNDRED,
            Positive::ONE,
            dec!(-0.02),
            Positive::ZERO,
            pos_or_panic!(0.1),
            &OptionStyle::Call,
        ),
        "baw negative-rate call",
    );
    assert_eq!(call, dec!(50));
}

/// Binary options define both limits: at `T = 0` the payout if in the money
/// (cash 1, or the spot), at `σ = 0` the discounted payout if the forward
/// finishes in the money. `binary_black_scholes` is called directly: the
/// `black_scholes` dispatcher evaluates the vanilla `d1` first and rejects
/// `T = 0` and `σ = 0` before reaching the binary branch.
#[test]
fn test_binary_zero_time_and_zero_volatility_limits() {
    let cash = OptionType::Binary {
        binary_type: BinaryType::CashOrNothing,
    };
    let asset = OptionType::Binary {
        binary_type: BinaryType::AssetOrNothing,
    };
    let price = |option_type: &OptionType, style, spot, days, vol| {
        ok(
            binary_black_scholes(&option(
                option_type.clone(),
                style,
                Side::Long,
                spot,
                100.0,
                days,
                vol,
                dec!(0.05),
                0.0,
                None,
            )),
            "binary",
        )
    };
    assert_eq!(
        price(&cash, OptionStyle::Call, 110.0, 0.0, 0.2),
        Decimal::ONE
    );
    assert_eq!(
        price(&cash, OptionStyle::Put, 110.0, 0.0, 0.2),
        Decimal::ZERO
    );
    assert_eq!(price(&asset, OptionStyle::Call, 110.0, 0.0, 0.2), dec!(110));
    assert_eq!(
        price(&asset, OptionStyle::Put, 110.0, 0.0, 0.2),
        Decimal::ZERO
    );
    // σ = 0, one year, r = 5 %: the forward of 98 is 103.02 > 100.
    let t = Decimal::ONE;
    assert_close(
        price(&cash, OptionStyle::Call, 98.0, 365.0, 0.0),
        discount(dec!(0.05), t),
        IDENTITY_TOL,
        "cash call at zero vol",
    );
    assert_eq!(
        price(&cash, OptionStyle::Put, 98.0, 365.0, 0.0),
        Decimal::ZERO
    );
    assert_close(
        price(&asset, OptionStyle::Call, 98.0, 365.0, 0.0),
        dec!(98),
        IDENTITY_TOL,
        "asset call at zero vol",
    );
}
