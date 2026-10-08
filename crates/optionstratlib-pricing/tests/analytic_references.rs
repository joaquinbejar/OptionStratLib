/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-05
******************************************************************************/

//! Published reference values for `optionstratlib-pricing` (#526).
//!
//! Each test pins one or more prices or Greeks to a value printed in a
//! textbook, so a regression fails against an external number rather than
//! against a snapshot of the library itself.
//!
//! # Coverage and sources
//!
//! * Black-Scholes(-Merton): Hull, *Options, Futures, and Other
//!   Derivatives*, worked example of the Black-Scholes-Merton pricing
//!   formulas (`S = 42, K = 40, r = 10 %, σ = 20 %, T = 0.5`); Haug, *The
//!   Complete Guide to Option Pricing Formulas*, 2nd ed. (2007), §1.1.1 and
//!   the Merton (1973) dividend-yield example of §1.1.
//! * Greeks: Hull, *The Greek Letters* chapter, running example
//!   `S = 49, K = 50, r = 5 %, σ = 20 %, T = 20 weeks`.
//! * Black-76: Hull, *Futures Options and Black's Model*, crude-oil example;
//!   Haug §1.1.4 (Black 1976) example.
//! * Garman-Kohlhagen: Haug §1.1.6 currency-option example.
//! * Binomial (CRR): Hull, *Binomial Trees in Practice*, American put
//!   `S = K = 50, r = 10 %, σ = 40 %, T = 5 months` on 5, 30, 50 and 100
//!   steps.
//! * Barone-Adesi-Whaley: Haug, Chapter 3, the BAW table of American calls on
//!   futures (`K = 100, r = 10 %, b = 0`, `T ∈ {0.1, 0.5}`,
//!   `σ ∈ {15, 25, 35} %`, `S ∈ {90, 100, 110}`).
//! * Binary: Haug §4.19.1 cash-or-nothing and §4.19.2 asset-or-nothing
//!   examples.
//! * Chooser: Haug §4.12.1 (Rubinstein 1991) simple chooser example.
//! * Spread: Haug §5.4.2 Kirk (1995) example and table.
//! * Exchange: Margrabe (1978) reduced to the Hull Black-Scholes example by a
//!   deterministic second asset.
//! * Barrier: Haug §4.17.1 (Reiner-Rubinstein 1991) Table 4-13
//!   (`S = 100, T = 0.5, r = 8 %, b = 4 %, rebate 3`), all eight contracts
//!   at `σ = 25 %` and `30 %` (#646), and the same formulas at zero rebate.
//! * Lookback: Haug §4.15.2 (Conze-Viswanathan 1991) fixed-strike table,
//!   calls and puts (#647).
//! * Payoff: the textbook payoffs `max(S - K, 0)` and `max(K - S, 0)`
//!   through core's `Payoff` via `Options::payoff` / `payoff_at_price`.
//!
//! Families without a matching published value: geometric Asian (Haug's
//! example needs `b > r`, a negative dividend yield, which
//! `Options::dividend_yield: Positive` cannot hold; covered by the
//! Kemna-Vorst reduction in `pricing_identities.rs`), power and quanto
//! (covered by their Black-Scholes reductions there), compound (the pricer
//! uses one strike for both legs, Haug's Geske example has two), rainbow
//! (internal Monte Carlo), cliquet and telegraph (no closed-form published
//! example with this parametrisation).
//!
//! # Tolerance policy
//!
//! The tolerance is half a unit in the last printed digit of the reference:
//! `±0.005` for a two-decimal value, `±0.0005` for three, `±0.00005` for
//! four. A value printed to one decimal (Hull's vega `12.1`) gets `±0.05`.
//! Two exceptions are documented at their tests: the Barone-Adesi-Whaley
//! table (its critical-price iteration stops early) and the Haug values the
//! library reproduces only through a unit rescaling.
//!
//! Time to expiry enters through `ExpirationDate::Days`, converted at
//! 365 days per year, so `T = 0.5` is `182.5` days.

use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::types::{
    BarrierType, BinaryType, LookbackType, OptionStyle, OptionType, Side,
};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::{delta, gamma, rho, theta, vega};
use optionstratlib_pricing::pricing::{
    BinomialPricingParams, barone_adesi_whaley, barrier_black_scholes, black_76, black_scholes,
    garman_kohlhagen, price_binomial,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::num::NonZeroUsize;

/// Half a unit in the second decimal.
const TOL_2DP: Decimal = dec!(0.005);
/// Half a unit in the third decimal.
const TOL_3DP: Decimal = dec!(0.0005);
/// Half a unit in the fourth decimal.
const TOL_4DP: Decimal = dec!(0.00005);

/// Unwraps a fallible library call, failing the test with context.
fn ok<T, E: Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: unexpected error {err:?}"),
    }
}

/// Asserts `|actual - expected| <= tol`.
fn assert_close(actual: Decimal, expected: Decimal, tol: Decimal, context: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tol,
        "{context}: actual {actual}, reference {expected}, |diff| {diff} > {tol}"
    );
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
    exotic: Option<ExoticParams>,
) -> Options {
    Options::new(
        option_type,
        Side::Long,
        "REF".to_string(),
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
        spot,
        strike,
        days,
        vol,
        rate,
        dividend,
        None,
    )
}

// ---------------------------------------------------------------------------
// Black-Scholes, Black-76, Garman-Kohlhagen
// ---------------------------------------------------------------------------

/// Hull, Black-Scholes-Merton pricing formulas worked example:
/// `S = 42, K = 40, r = 10 %, σ = 20 %, T = 0.5` gives `c = 4.76`,
/// `p = 0.81`. Two decimals printed, so `±0.005`.
#[test]
fn test_black_scholes_hull_example_matches_reference() {
    let call = european(OptionStyle::Call, 42.0, 40.0, 182.5, 0.20, dec!(0.10), 0.0);
    let put = european(OptionStyle::Put, 42.0, 40.0, 182.5, 0.20, dec!(0.10), 0.0);
    assert_close(
        ok(black_scholes(&call), "call"),
        dec!(4.76),
        TOL_2DP,
        "hull call",
    );
    assert_close(
        ok(black_scholes(&put), "put"),
        dec!(0.81),
        TOL_2DP,
        "hull put",
    );
}

/// Haug §1.1.1: `S = 60, K = 65, T = 0.25, r = 8 %, σ = 30 %` gives a call
/// of `2.1334`. Four decimals printed, so `±0.00005`.
#[test]
fn test_black_scholes_haug_example_matches_reference() {
    let call = european(OptionStyle::Call, 60.0, 65.0, 91.25, 0.30, dec!(0.08), 0.0);
    assert_close(
        ok(black_scholes(&call), "call"),
        dec!(2.1334),
        TOL_4DP,
        "haug bs call",
    );
}

/// Haug §1.1, Merton (1973) with a continuous dividend yield:
/// `S = 100, K = 95, T = 0.5, r = 10 %, q = 5 %, σ = 20 %` gives a put of
/// `2.4648`.
#[test]
fn test_black_scholes_merton_dividend_example_matches_reference() {
    let put = european(OptionStyle::Put, 100.0, 95.0, 182.5, 0.20, dec!(0.10), 0.05);
    assert_close(
        ok(black_scholes(&put), "put"),
        dec!(2.4648),
        TOL_4DP,
        "merton put",
    );
}

/// Hull, *The Greek Letters*: `S = 49, K = 50, r = 5 %, σ = 20 %,
/// T = 20 weeks` gives price `2.40`, `Δ = 0.522`, `Γ = 0.066`,
/// `vega = 12.1` per unit of volatility, `Θ = -4.31` per year and
/// `ρ = 8.91` per unit of rate.
///
/// The library reports vega and rho per 1 % and theta per calendar day, so
/// they are rescaled by 100, 100 and 365 before the comparison. The
/// tolerance follows each printed precision.
#[test]
fn test_black_scholes_greeks_hull_example_match_reference() {
    // 20 weeks = 20/52 years = 140.3846... days at 365 days per year.
    let call = european(
        OptionStyle::Call,
        49.0,
        50.0,
        20.0 / 52.0 * 365.0,
        0.20,
        dec!(0.05),
        0.0,
    );
    assert_close(
        ok(black_scholes(&call), "price"),
        dec!(2.40),
        TOL_2DP,
        "price",
    );
    assert_close(ok(delta(&call), "delta"), dec!(0.522), TOL_3DP, "delta");
    assert_close(ok(gamma(&call), "gamma"), dec!(0.066), TOL_3DP, "gamma");
    assert_close(
        ok(vega(&call), "vega") * dec!(100),
        dec!(12.1),
        dec!(0.05),
        "vega per unit",
    );
    assert_close(
        ok(theta(&call), "theta") * dec!(365),
        dec!(-4.31),
        TOL_2DP,
        "theta per year",
    );
    assert_close(
        ok(rho(&call), "rho") * dec!(100),
        dec!(8.91),
        TOL_2DP,
        "rho per unit",
    );
}

/// Hull, *Futures Options and Black's Model*: a European put on crude-oil
/// futures, `F = K = 20, r = 9 %, T = 4 months, σ = 25 %`, is worth `1.12`;
/// at the money on the forward the call has the same value.
#[test]
fn test_black_76_hull_example_matches_reference() {
    let days = 365.0 / 3.0;
    let put = european(OptionStyle::Put, 20.0, 20.0, days, 0.25, dec!(0.09), 0.0);
    let call = european(OptionStyle::Call, 20.0, 20.0, days, 0.25, dec!(0.09), 0.0);
    assert_close(
        ok(black_76(&put), "put"),
        dec!(1.12),
        TOL_2DP,
        "hull b76 put",
    );
    assert_close(
        ok(black_76(&call), "call"),
        dec!(1.12),
        TOL_2DP,
        "hull b76 call",
    );
}

/// Haug §1.1.4 (Black 1976): `F = K = 19, T = 0.75, r = 10 %, σ = 28 %`
/// gives `c = p = 1.7011`.
#[test]
fn test_black_76_haug_example_matches_reference() {
    let call = european(OptionStyle::Call, 19.0, 19.0, 273.75, 0.28, dec!(0.10), 0.0);
    let put = european(OptionStyle::Put, 19.0, 19.0, 273.75, 0.28, dec!(0.10), 0.0);
    assert_close(
        ok(black_76(&call), "call"),
        dec!(1.7011),
        TOL_4DP,
        "haug b76 call",
    );
    assert_close(
        ok(black_76(&put), "put"),
        dec!(1.7011),
        TOL_4DP,
        "haug b76 put",
    );
}

/// Haug §1.1.6 (Garman-Kohlhagen 1983): `S = 1.56, K = 1.60, T = 0.5,
/// r_d = 6 %, r_f = 8 %, σ = 12 %` gives a call of `0.0291`. The foreign
/// rate travels in `dividend_yield`.
#[test]
fn test_garman_kohlhagen_haug_example_matches_reference() {
    let call = european(OptionStyle::Call, 1.56, 1.60, 182.5, 0.12, dec!(0.06), 0.08);
    assert_close(
        ok(garman_kohlhagen(&call), "call"),
        dec!(0.0291),
        TOL_4DP,
        "haug gk call",
    );
}

// ---------------------------------------------------------------------------
// Lattice and American approximations
// ---------------------------------------------------------------------------

/// Hull, *Binomial Trees in Practice*: the CRR tree for an American put with
/// `S = K = 50, r = 10 %, σ = 40 %, T = 5 months` prices `4.49` on 5 steps
/// and `4.263`, `4.272`, `4.278` on 30, 50 and 100 steps.
#[test]
fn test_binomial_american_put_hull_example_matches_reference() {
    let expiry = ok(Positive::new_decimal(dec!(5) / dec!(12)), "5 months");
    let cases = [
        (5usize, dec!(4.49), TOL_2DP),
        (30, dec!(4.263), TOL_3DP),
        (50, dec!(4.272), TOL_3DP),
        (100, dec!(4.278), TOL_3DP),
    ];
    for (steps, reference, tol) in cases {
        let Some(no_steps) = NonZeroUsize::new(steps) else {
            panic!("non-zero step count");
        };
        let price = ok(
            price_binomial(BinomialPricingParams {
                asset: pos_or_panic!(50.0),
                volatility: pos_or_panic!(0.40),
                int_rate: dec!(0.10),
                strike: pos_or_panic!(50.0),
                expiry,
                no_steps,
                option_type: &OptionType::American,
                option_style: &OptionStyle::Put,
                side: &Side::Long,
            }),
            "binomial",
        );
        assert_close(
            price,
            reference,
            tol,
            &format!("hull american put n={steps}"),
        );
    }
}

/// Haug, Chapter 3, Barone-Adesi-Whaley (1987) American calls on futures:
/// `K = 100, r = 10 %, b = 0` (so `q = r`).
///
/// Tolerance `±0.005`, not the `±0.00005` four printed decimals would give.
/// BAW is defined through the critical price `S*`, found iteratively, and
/// the printed values sit up to `3e-3` above a fully converged `S*` near the
/// exercise boundary (printed `10.0089` at `S = 110, T = 0.1, σ = 15 %`
/// against `10.0061` converged), consistent with the published iteration
/// stopping early. An independent, fully converged BAW implementation
/// matches the library to `1e-4` on all eighteen points. `±0.005` is the
/// two-decimal precision of the original BAW (1987) tables.
#[test]
fn test_barone_adesi_whaley_haug_futures_call_table_matches_reference() {
    let table: [(f64, f64, [Decimal; 3]); 6] = [
        (0.1, 0.15, [dec!(0.0206), dec!(1.8771), dec!(10.0089)]),
        (0.1, 0.25, [dec!(0.3159), dec!(3.1280), dec!(10.3919)]),
        (0.1, 0.35, [dec!(0.9495), dec!(4.3777), dec!(11.1679)]),
        (0.5, 0.15, [dec!(0.8208), dec!(4.0842), dec!(10.8087)]),
        (0.5, 0.25, [dec!(2.7437), dec!(6.8015), dec!(13.0170)]),
        (0.5, 0.35, [dec!(5.0063), dec!(9.5106), dec!(15.5689)]),
    ];
    for (t, vol, references) in table {
        for (spot, reference) in [90.0, 100.0, 110.0].into_iter().zip(references) {
            let price = ok(
                barone_adesi_whaley(
                    pos_or_panic!(spot),
                    pos_or_panic!(100.0),
                    pos_or_panic!(t),
                    dec!(0.10),
                    pos_or_panic!(0.10),
                    pos_or_panic!(vol),
                    &OptionStyle::Call,
                ),
                "baw",
            );
            assert_close(
                price,
                reference,
                TOL_2DP,
                &format!("baw call s={spot} t={t} vol={vol}"),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Exotics
// ---------------------------------------------------------------------------

/// Haug §4.19.1: cash-or-nothing put, `S = 100, K = 80, cash = 10,
/// T = 0.75, r = 6 %, b = 0 (q = 6 %), σ = 35 %` is worth `2.6710`.
///
/// The library's cash-or-nothing pays a fixed unit amount, so the price is
/// scaled by 10; the scaling is exact, and `±0.00005` on the scaled value
/// is `±0.000005` on the unit price.
#[test]
fn test_binary_cash_or_nothing_haug_example_matches_reference() {
    let put = option(
        OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        },
        OptionStyle::Put,
        100.0,
        80.0,
        273.75,
        0.35,
        dec!(0.06),
        0.06,
        None,
    );
    assert_close(
        ok(black_scholes(&put), "cash put") * dec!(10),
        dec!(2.6710),
        TOL_4DP,
        "haug cash-or-nothing put",
    );
}

/// Haug §4.19.2: asset-or-nothing put, `S = 70, K = 65, T = 0.5, r = 7 %,
/// q = 5 %, σ = 27 %` is worth `20.2069`.
#[test]
fn test_binary_asset_or_nothing_haug_example_matches_reference() {
    let put = option(
        OptionType::Binary {
            binary_type: BinaryType::AssetOrNothing,
        },
        OptionStyle::Put,
        70.0,
        65.0,
        182.5,
        0.27,
        dec!(0.07),
        0.05,
        None,
    );
    assert_close(
        ok(black_scholes(&put), "asset put"),
        dec!(20.2069),
        TOL_4DP,
        "haug asset-or-nothing put",
    );
}

/// Haug §4.12.1 (Rubinstein 1991): simple chooser, `S = K = 50, t = 0.25,
/// T = 0.5, r = b = 8 %, σ = 25 %` is worth `6.1071`. The choice date is
/// given in days (`91.25`).
#[test]
fn test_chooser_haug_example_matches_reference() {
    let chooser = option(
        OptionType::Chooser {
            choice_date: pos_or_panic!(91.25),
        },
        OptionStyle::Call,
        50.0,
        50.0,
        182.5,
        0.25,
        dec!(0.08),
        0.0,
        None,
    );
    assert_close(
        ok(black_scholes(&chooser), "chooser"),
        dec!(6.1071),
        TOL_4DP,
        "haug chooser",
    );
}

fn spread_params(vol2: f64, rho: Decimal, dividend2: f64) -> ExoticParams {
    ExoticParams {
        spread_second_asset_volatility: Some(pos_or_panic!(vol2)),
        spread_second_asset_dividend: Some(pos_or_panic!(dividend2)),
        spread_correlation: Some(rho),
        ..Default::default()
    }
}

/// Haug §5.4.2, Kirk (1995) on futures: `F1 = 28, F2 = 20, K = 7,
/// T = 0.25, r = 5 %, σ1 = 29 %, σ2 = 36 %, ρ = 0.42` gives a call of
/// `2.1670`; the Kirk table entry `F1 = 122, F2 = 120, K = 3, T = 0.1,
/// r = 10 %, σ1 = σ2 = 20 %, ρ = -0.5` gives `4.7530`.
///
/// Futures carry no drift, so each asset's yield is set to `r`
/// (`S e^(-qT) = F e^(-rT)`).
#[test]
fn test_spread_kirk_haug_examples_match_reference() {
    let example = option(
        OptionType::Spread {
            second_asset: pos_or_panic!(20.0),
        },
        OptionStyle::Call,
        28.0,
        7.0,
        91.25,
        0.29,
        dec!(0.05),
        0.05,
        Some(spread_params(0.36, dec!(0.42), 0.05)),
    );
    assert_close(
        ok(black_scholes(&example), "kirk"),
        dec!(2.1670),
        TOL_4DP,
        "haug kirk example",
    );
    let table = option(
        OptionType::Spread {
            second_asset: pos_or_panic!(120.0),
        },
        OptionStyle::Call,
        122.0,
        3.0,
        36.5,
        0.20,
        dec!(0.10),
        0.10,
        Some(spread_params(0.20, dec!(-0.5), 0.10)),
    );
    assert_close(
        ok(black_scholes(&table), "kirk"),
        dec!(4.7530),
        TOL_4DP,
        "haug kirk table",
    );
}

/// Margrabe (1978) with a deterministic second asset (`σ2 = 0, q2 = 0`) is a
/// vanilla call struck at the second asset's forward: exchanging
/// `S2 = 40 e^(-0.1 * 0.5) = 38.049177` for `S1 = 42` reproduces Hull's
/// Black-Scholes example, `c = 4.76`. Rounding `S2` to six decimals moves
/// the price by less than `1e-6`, well inside `±0.005`.
#[test]
fn test_exchange_margrabe_reduces_to_hull_example() {
    let params = ExoticParams {
        exchange_second_asset_volatility: Some(Positive::ZERO),
        exchange_second_asset_dividend: Some(Positive::ZERO),
        exchange_correlation: Some(Decimal::ZERO),
        ..Default::default()
    };
    let exchange = option(
        OptionType::Exchange {
            second_asset: pos_or_panic!(38.049177),
        },
        OptionStyle::Call,
        42.0,
        40.0,
        182.5,
        0.20,
        dec!(0.10),
        0.0,
        Some(params),
    );
    assert_close(
        ok(black_scholes(&exchange), "exchange"),
        dec!(4.76),
        TOL_2DP,
        "margrabe",
    );
}

fn barrier(barrier_type: BarrierType, level: f64, rebate: Option<Positive>) -> OptionType {
    OptionType::Barrier {
        barrier_type,
        barrier_level: pos_or_panic!(level),
        rebate,
    }
}

/// Prices a Haug Table 4-13 barrier: `S = 100, T = 0.5, r = 8 %, q = 4 %,
/// σ = 25 %`.
fn haug_barrier(option_type: OptionType, style: OptionStyle, strike: f64) -> Decimal {
    haug_barrier_at(option_type, style, strike, 0.25)
}

/// [`haug_barrier`] at volatility `vol`.
fn haug_barrier_at(option_type: OptionType, style: OptionStyle, strike: f64, vol: f64) -> Decimal {
    ok(
        black_scholes(&option(
            option_type,
            style,
            100.0,
            strike,
            182.5,
            vol,
            dec!(0.08),
            0.04,
            None,
        )),
        "barrier",
    )
}

/// Down-and-out and down-and-in calls without rebate, Haug §4.17.1
/// (Reiner-Rubinstein 1991) on the Table 4-13 inputs (`S = 100, H = 95,
/// T = 0.5, r = 8 %, q = 4 %, σ = 25 %`).
///
/// Haug prints the table with a rebate of 3 only, so these four-decimal
/// values are the published formulas evaluated at rebate 0 by an
/// independent implementation that reproduces every printed Table 4-13
/// entry.
#[test]
fn test_barrier_down_calls_without_rebate_match_reiner_rubinstein() {
    let cases = [
        (
            BarrierType::DownAndOut,
            [dec!(6.7447), dec!(4.5126), dec!(2.5960)],
        ),
        (
            BarrierType::DownAndIn,
            [dec!(7.0886), dec!(3.3368), dec!(1.3835)],
        ),
    ];
    for (barrier_type, references) in cases {
        for (strike, reference) in [90.0, 100.0, 110.0].into_iter().zip(references) {
            assert_close(
                haug_barrier(barrier(barrier_type, 95.0, None), OptionStyle::Call, strike),
                reference,
                TOL_4DP,
                &format!("{barrier_type:?} call k={strike} no rebate"),
            );
        }
    }
}

/// Haug Table 4-13, rebate 3, `σ = 25 %`: down-and-out call `9.0246,
/// 6.7924, 4.8759` and down-and-in call `7.7627, 4.0109, 2.0576` for
/// `K = 90, 100, 110`, `H = 95`.
///
/// The library used to add the knock-in rebate leg (paid at expiry if the
/// barrier is never hit, Haug's `E`) to the knock-out and the knock-out leg
/// (paid at the hit, Haug's `F`) to the knock-in, returning `7.4188, 5.1867,
/// 3.2701` and `9.3684, 5.6167, 3.6633` (#646).
#[test]
fn test_barrier_down_calls_with_rebate_match_haug_table() {
    let rebate = Some(pos_or_panic!(3.0));
    let cases = [
        (
            BarrierType::DownAndOut,
            [dec!(9.0246), dec!(6.7924), dec!(4.8759)],
        ),
        (
            BarrierType::DownAndIn,
            [dec!(7.7627), dec!(4.0109), dec!(2.0576)],
        ),
    ];
    for (barrier_type, references) in cases {
        for (strike, reference) in [90.0, 100.0, 110.0].into_iter().zip(references) {
            assert_close(
                haug_barrier(
                    barrier(barrier_type, 95.0, rebate),
                    OptionStyle::Call,
                    strike,
                ),
                reference,
                TOL_4DP,
                &format!("{barrier_type:?} call k={strike} rebate 3"),
            );
        }
    }
}

/// Haug Table 4-13, rebate 3, `σ = 25 %`, `K = 90, 100, 110`:
/// up-and-out call (`H = 105`) `2.6789, 2.3580, 2.3453`; up-and-in call
/// `14.1112, 8.4482, 4.5910`; down-and-out put (`H = 95`) `2.2798, 2.2947,
/// 2.6252`; down-and-in put `2.9586, 6.5677, 11.9752`; up-and-out put
/// `3.7760, 5.4932, 7.5187`; up-and-in put `1.4653, 3.3721, 7.0846`.
///
/// The library used to return up-and-out call `5.9407, 0.0890, 2.3453`;
/// up-and-in call `12.5833, 12.4511, 6.3249`; down-and-out put `10.0631,
/// 9.3879, 9.2602`; down-and-in put `-4.8247, -0.5254, 5.3403`; up-and-out
/// put `1.7989, 3.3847, 0.6114`; up-and-in put `3.4424, 5.4806, 13.9918`:
/// the case selection for up barriers and for puts, not only the rebate,
/// departed from Reiner-Rubinstein (#646).
#[test]
fn test_barrier_up_calls_and_puts_match_haug_table() {
    let rebate = Some(pos_or_panic!(3.0));
    let cases = [
        (
            BarrierType::UpAndOut,
            105.0,
            OptionStyle::Call,
            [dec!(2.6789), dec!(2.3580), dec!(2.3453)],
        ),
        (
            BarrierType::UpAndIn,
            105.0,
            OptionStyle::Call,
            [dec!(14.1112), dec!(8.4482), dec!(4.5910)],
        ),
        (
            BarrierType::DownAndOut,
            95.0,
            OptionStyle::Put,
            [dec!(2.2798), dec!(2.2947), dec!(2.6252)],
        ),
        (
            BarrierType::DownAndIn,
            95.0,
            OptionStyle::Put,
            [dec!(2.9586), dec!(6.5677), dec!(11.9752)],
        ),
        (
            BarrierType::UpAndOut,
            105.0,
            OptionStyle::Put,
            [dec!(3.7760), dec!(5.4932), dec!(7.5187)],
        ),
        (
            BarrierType::UpAndIn,
            105.0,
            OptionStyle::Put,
            [dec!(1.4653), dec!(3.3721), dec!(7.0846)],
        ),
    ];
    for (barrier_type, level, style, references) in cases {
        for (strike, reference) in [90.0, 100.0, 110.0].into_iter().zip(references) {
            assert_close(
                haug_barrier(barrier(barrier_type, level, rebate), style, strike),
                reference,
                TOL_4DP,
                &format!("{barrier_type:?} {style:?} k={strike} rebate 3"),
            );
        }
    }
}

/// Haug Table 4-13, rebate 3, the `σ = 30 %` column and the `H = 100` rows
/// (the barrier at the spot): a knock-out is worth its rebate, paid now, and
/// a knock-in is the vanilla. Every value was recomputed independently from
/// the §4.17.1 formulas in `f64` (#646).
#[test]
fn test_barrier_haug_table_sigma_30_and_barrier_at_spot() {
    let rebate = Some(pos_or_panic!(3.0));
    let sigma_30 = [
        (
            BarrierType::DownAndOut,
            95.0,
            OptionStyle::Call,
            [dec!(8.8334), dec!(7.0285), dec!(5.4137)],
        ),
        (
            BarrierType::UpAndOut,
            105.0,
            OptionStyle::Call,
            [dec!(2.6340), dec!(2.4389), dec!(2.4315)],
        ),
        (
            BarrierType::DownAndIn,
            95.0,
            OptionStyle::Call,
            [dec!(9.0093), dec!(5.1370), dec!(2.8517)],
        ),
        (
            BarrierType::UpAndIn,
            105.0,
            OptionStyle::Call,
            [dec!(15.2098), dec!(9.7278), dec!(5.8350)],
        ),
        (
            BarrierType::DownAndOut,
            95.0,
            OptionStyle::Put,
            [dec!(2.4170), dec!(2.4258), dec!(2.6246)],
        ),
        (
            BarrierType::UpAndOut,
            105.0,
            OptionStyle::Put,
            [dec!(4.2292), dec!(5.8033), dec!(7.5650)],
        ),
        (
            BarrierType::DownAndIn,
            95.0,
            OptionStyle::Put,
            [dec!(3.8769), dec!(7.7988), dec!(13.3077)],
        ),
        (
            BarrierType::UpAndIn,
            105.0,
            OptionStyle::Put,
            [dec!(2.0658), dec!(4.4226), dec!(8.3686)],
        ),
    ];
    for (barrier_type, level, style, references) in sigma_30 {
        for (strike, reference) in [90.0, 100.0, 110.0].into_iter().zip(references) {
            assert_close(
                haug_barrier_at(barrier(barrier_type, level, rebate), style, strike, 0.30),
                reference,
                TOL_4DP,
                &format!("{barrier_type:?} {style:?} k={strike} σ=30% rebate 3"),
            );
        }
    }
    let at_spot = [
        (
            BarrierType::DownAndOut,
            OptionStyle::Call,
            [dec!(3), dec!(3), dec!(3)],
        ),
        (
            BarrierType::DownAndOut,
            OptionStyle::Put,
            [dec!(3), dec!(3), dec!(3)],
        ),
        (
            BarrierType::DownAndIn,
            OptionStyle::Call,
            [dec!(13.8333), dec!(7.8494), dec!(3.9795)],
        ),
        (
            BarrierType::DownAndIn,
            OptionStyle::Put,
            [dec!(2.2845), dec!(5.9085), dec!(11.6465)],
        ),
    ];
    for (barrier_type, style, references) in at_spot {
        for (strike, reference) in [90.0, 100.0, 110.0].into_iter().zip(references) {
            assert_close(
                haug_barrier(barrier(barrier_type, 100.0, rebate), style, strike),
                reference,
                TOL_4DP,
                &format!("{barrier_type:?} {style:?} k={strike} H=S rebate 3"),
            );
        }
    }
}

/// Standard normal CDF in `f64`, from the complementary error function of
/// Numerical Recipes (`erfcc`, fractional error below `1.2e-7`): an
/// evaluation independent of the library's `big_n`.
fn normal_cdf_f64(x: f64) -> f64 {
    let z = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = -z * z - 1.265_512_23
        + t * (1.000_023_68
            + t * (0.374_091_96
                + t * (0.096_784_18
                    + t * (-0.186_288_06
                        + t * (0.278_868_07
                            + t * (-1.135_203_98
                                + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77))))))));
    let erfc = t * poly.exp();
    if x >= 0.0 {
        1.0 - 0.5 * erfc
    } else {
        0.5 * erfc
    }
}

/// Reiner-Rubinstein (Haug §4.17.1) `C + E` in `f64`: the price of a
/// down-and-in call with `K > H` (`η = φ = 1`) or an up-and-in put with
/// `K < H` (`η = φ = -1`), on `S = 100, r = 8 %, q = 4 %, σ = 25 %`.
fn reiner_rubinstein_c_plus_e(eta: f64, k: f64, h: f64, rebate: f64, t: f64) -> f64 {
    let (s, r, q, sigma) = (100.0_f64, 0.08_f64, 0.04_f64, 0.25_f64);
    let phi = eta;
    let b = r - q;
    let mu = (b - sigma * sigma / 2.0) / (sigma * sigma);
    let sst = sigma * t.sqrt();
    let x2 = (s / h).ln() / sst + (1.0 + mu) * sst;
    let y1 = (h * h / (s * k)).ln() / sst + (1.0 + mu) * sst;
    let y2 = (h / s).ln() / sst + (1.0 + mu) * sst;
    let hs = h / s;
    let c = phi * s * ((b - r) * t).exp() * hs.powf(2.0 * (mu + 1.0)) * normal_cdf_f64(eta * y1)
        - phi * k * (-r * t).exp() * hs.powf(2.0 * mu) * normal_cdf_f64(eta * y1 - eta * sst);
    let e = rebate
        * (-r * t).exp()
        * (normal_cdf_f64(eta * x2 - eta * sst)
            - hs.powf(2.0 * mu) * normal_cdf_f64(eta * y2 - eta * sst));
    c + e
}

/// A barrier priced on `S = 100, r = 8 %, q = 4 %, σ = 25 %` and the given
/// expiry in days, for `side`.
fn barrier_at_days(
    option_type: OptionType,
    style: OptionStyle,
    strike: f64,
    days: f64,
    side: Side,
) -> Decimal {
    let mut contract = option(
        option_type,
        style,
        100.0,
        strike,
        days,
        0.25,
        dec!(0.08),
        0.04,
        None,
    );
    contract.side = side;
    // The barrier kernel itself: `black_scholes` computes `d1` before it
    // dispatches, which rejects `T = 0`.
    ok(barrier_black_scholes(&contract), "barrier")
}

/// An unhit knock-in carries its rebate to expiry (Haug's `E`), so the
/// Reiner-Rubinstein price tends to the rebate as `T → 0`, and the price at
/// `T = 0` is that rebate (#826). It used to be zero. The closed form at
/// `T = 10, 1` days and one hour is checked against an independent `f64`
/// evaluation of the same `C + E` terms.
#[test]
fn test_barrier_unhit_knock_in_rebate_is_continuous_at_expiry() {
    let rebate = Some(pos_or_panic!(3.0));
    let cases = [
        (BarrierType::DownAndIn, 95.0, OptionStyle::Call, 1.0, 100.0),
        (BarrierType::DownAndIn, 95.0, OptionStyle::Call, 1.0, 110.0),
        (BarrierType::UpAndIn, 105.0, OptionStyle::Put, -1.0, 100.0),
        (BarrierType::UpAndIn, 105.0, OptionStyle::Put, -1.0, 90.0),
    ];
    for (barrier_type, level, style, eta, strike) in cases {
        let contract = barrier(barrier_type, level, rebate);
        for days in [10.0, 1.0, 1.0 / 24.0] {
            let reference = reiner_rubinstein_c_plus_e(eta, strike, level, 3.0, days / 365.0);
            assert_close(
                barrier_at_days(contract.clone(), style, strike, days, Side::Long),
                Decimal::try_from(reference).unwrap_or(Decimal::ZERO),
                TOL_4DP,
                &format!("{barrier_type:?} {style:?} k={strike} T={days}d"),
            );
        }
        // One minute before expiry the price is the rebate to four places,
        // and at expiry it is the rebate, for either side.
        let near = barrier_at_days(contract.clone(), style, strike, 1.0 / 1440.0, Side::Long);
        assert_close(
            near,
            dec!(3),
            TOL_4DP,
            &format!("{barrier_type:?} one minute"),
        );
        assert_eq!(
            barrier_at_days(contract.clone(), style, strike, 0.0, Side::Long),
            dec!(3),
            "{barrier_type:?} {style:?} k={strike} at expiry"
        );
        assert_eq!(
            barrier_at_days(contract, style, strike, 0.0, Side::Short),
            dec!(-3),
            "{barrier_type:?} {style:?} k={strike} short at expiry"
        );
    }
}

/// All eight contracts, rebate 3, `K ∈ {90, 100, 110}`, with the spot short
/// of the barrier (unhit), at it and through it (hit): the price at expiry
/// is the payoff the contract defines, for both sides, and away from the
/// money (`K = 90, 110`) the price one minute before expiry agrees with it
/// to three places (#826).
#[test]
fn test_barrier_price_is_continuous_at_expiry_with_rebate() {
    let rebate = Some(pos_or_panic!(3.0));
    let vanilla = |style: OptionStyle, strike: f64| -> Decimal {
        let spot = dec!(100);
        let k = Decimal::try_from(strike).unwrap_or(Decimal::ZERO);
        match style {
            OptionStyle::Call => (spot - k).max(Decimal::ZERO),
            OptionStyle::Put => (k - spot).max(Decimal::ZERO),
        }
    };
    let contracts = [
        (BarrierType::DownAndIn, true, true),
        (BarrierType::DownAndOut, true, false),
        (BarrierType::UpAndIn, false, true),
        (BarrierType::UpAndOut, false, false),
    ];
    for (barrier_type, is_down, knock_in) in contracts {
        // Unhit, at the barrier, through it.
        let levels = if is_down {
            [95.0, 100.0, 105.0]
        } else {
            [105.0, 100.0, 95.0]
        };
        for (index, level) in levels.into_iter().enumerate() {
            let hit = index > 0;
            for style in [OptionStyle::Call, OptionStyle::Put] {
                for strike in [90.0, 100.0, 110.0] {
                    let expected = match (knock_in, hit) {
                        (true, true) | (false, false) => vanilla(style, strike),
                        (true, false) | (false, true) => dec!(3),
                    };
                    for (side, sign) in [
                        (Side::Long, Decimal::ONE),
                        (Side::Short, Decimal::NEGATIVE_ONE),
                    ] {
                        let contract = barrier(barrier_type, level, rebate);
                        let context =
                            format!("{barrier_type:?} H={level} {style:?} k={strike} {side:?}");
                        let at_expiry = barrier_at_days(contract.clone(), style, strike, 0.0, side);
                        assert_eq!(at_expiry, expected * sign, "{context} at expiry");
                        // At the money the vanilla keeps a time value of
                        // order `Sσ√T` (`≈ 0.014` one minute out), which
                        // vanishes as `√T`, not within a tolerance.
                        if strike == 100.0 {
                            continue;
                        }
                        let near = barrier_at_days(contract, style, strike, 1.0 / 1440.0, side);
                        assert_close(near, at_expiry, TOL_3DP, &format!("{context} one minute"));
                    }
                }
            }
        }
    }
}

/// The price at expiry is per unit, like the closed form: it does not scale
/// by the position size (#826). It used to return `Options::payoff`, which
/// does.
#[test]
fn test_barrier_price_at_expiry_is_per_unit() {
    let mut contract = option(
        barrier(BarrierType::DownAndOut, 95.0, Some(pos_or_panic!(3.0))),
        OptionStyle::Call,
        100.0,
        90.0,
        0.0,
        0.25,
        dec!(0.08),
        0.04,
        None,
    );
    contract.quantity = pos_or_panic!(5.0);
    contract.contract_size = Positive::HUNDRED;
    assert_eq!(ok(barrier_black_scholes(&contract), "per unit"), dec!(10));
}

/// Haug §4.15.2 (Conze-Viswanathan 1991), fixed-strike lookback on a new
/// contract (`S = S_max = S_min = 100`), `T = 0.5, r = b = 10 %`, on
/// Haug's table grid `K ∈ {95, 100, 105}`, `σ ∈ {10, 20, 30} %`.
fn fixed_lookback(style: OptionStyle, strike: f64, vol: f64) -> Decimal {
    ok(
        black_scholes(&option(
            OptionType::Lookback {
                lookback_type: LookbackType::FixedStrike,
            },
            style,
            100.0,
            strike,
            182.5,
            vol,
            dec!(0.10),
            0.0,
            None,
        )),
        "lookback",
    )
}

/// Fixed-strike lookback calls: `13.2687, 18.9263, 24.9858`; `8.5126,
/// 14.1702, 20.2296`; `4.3908, 9.8905, 15.8512` (rows `K = 95, 100, 105`,
/// columns `σ = 10, 20, 30 %`), the published formula evaluated
/// independently in `f64` (`24.98576` at `K = 95, σ = 30 %`).
///
/// The library used to return `10.8209, 12.6609, 15.1705`; `6.8087,
/// 9.4397, 12.3250`; `3.8380, 6.8563, 9.9210`, between `0.55` and `9.8`
/// below the reference, the gap growing with volatility (#647).
#[test]
fn test_lookback_fixed_strike_call_matches_haug_table() {
    let table = [
        (95.0, [dec!(13.2687), dec!(18.9263), dec!(24.9858)]),
        (100.0, [dec!(8.5126), dec!(14.1702), dec!(20.2296)]),
        (105.0, [dec!(4.3908), dec!(9.8905), dec!(15.8512)]),
    ];
    for (strike, references) in table {
        for (vol, reference) in [0.10, 0.20, 0.30].into_iter().zip(references) {
            assert_close(
                fixed_lookback(OptionStyle::Call, strike, vol),
                reference,
                TOL_4DP,
                &format!("fixed lookback call k={strike} vol={vol}"),
            );
        }
    }
}

/// Fixed-strike lookback puts on the same grid: `0.6899, 4.4448, 8.9213`;
/// `3.3917, 8.3177, 13.1579`; `8.1478, 13.0739, 17.9140`, evaluated
/// independently from the same formulas (#647).
#[test]
fn test_lookback_fixed_strike_put_matches_haug_table() {
    let table = [
        (95.0, [dec!(0.6899), dec!(4.4448), dec!(8.9213)]),
        (100.0, [dec!(3.3917), dec!(8.3177), dec!(13.1579)]),
        (105.0, [dec!(8.1478), dec!(13.0739), dec!(17.9140)]),
    ];
    for (strike, references) in table {
        for (vol, reference) in [0.10, 0.20, 0.30].into_iter().zip(references) {
            assert_close(
                fixed_lookback(OptionStyle::Put, strike, vol),
                reference,
                TOL_4DP,
                &format!("fixed lookback put k={strike} vol={vol}"),
            );
        }
    }
}

/// At zero carry (`r = q = 5 %`) the reflection term's `σ²/(2b)` is
/// replaced by its limit; `S = K = 100, T = 0.5, σ = 25 %` gives a call of
/// `14.5364` and a put of `13.0124`, and a carry of `1e-6` agrees to `1e-4`
/// (independent `f64` evaluation: `14.536358`, `13.012437` at `b = 0`;
/// `14.536386`, `13.012415` at `b = 1e-6`) (#647).
#[test]
fn test_lookback_fixed_strike_zero_carry_limit() {
    let price = |style: OptionStyle, dividend: f64| {
        ok(
            black_scholes(&option(
                OptionType::Lookback {
                    lookback_type: LookbackType::FixedStrike,
                },
                style,
                100.0,
                100.0,
                182.5,
                0.25,
                dec!(0.05),
                dividend,
                None,
            )),
            "lookback",
        )
    };
    assert_close(
        price(OptionStyle::Call, 0.05),
        dec!(14.5364),
        TOL_4DP,
        "call b=0",
    );
    assert_close(
        price(OptionStyle::Put, 0.05),
        dec!(13.0124),
        TOL_4DP,
        "put b=0",
    );
    assert_close(
        price(OptionStyle::Call, 0.049999),
        dec!(14.5364),
        dec!(0.0001),
        "call b=1e-6",
    );
    assert_close(
        price(OptionStyle::Put, 0.049999),
        dec!(13.0124),
        dec!(0.0001),
        "put b=1e-6",
    );
}

// ---------------------------------------------------------------------------
// Payoffs
// ---------------------------------------------------------------------------

/// Expiry payoffs `max(S - K, 0)` and `max(K - S, 0)`, signed by side and
/// scaled by quantity, through core's `Payoff` implementation. Exact.
#[test]
fn test_payoff_values_match_definition() {
    let with = |style: OptionStyle, side: Side, spot: f64, quantity: f64| {
        Options::new(
            OptionType::European,
            side,
            "REF".to_string(),
            pos_or_panic!(100.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            pos_or_panic!(quantity),
            pos_or_panic!(spot),
            dec!(0.05),
            style,
            Positive::ZERO,
            None,
        )
    };
    let cases = [
        (OptionStyle::Call, Side::Long, 110.0, 1.0, dec!(10)),
        (OptionStyle::Call, Side::Long, 90.0, 1.0, dec!(0)),
        (OptionStyle::Call, Side::Short, 110.0, 1.0, dec!(-10)),
        (OptionStyle::Call, Side::Long, 110.0, 3.0, dec!(30)),
        (OptionStyle::Put, Side::Long, 90.0, 1.0, dec!(10)),
        (OptionStyle::Put, Side::Long, 110.0, 1.0, dec!(0)),
        (OptionStyle::Put, Side::Short, 90.0, 2.0, dec!(-20)),
        (OptionStyle::Put, Side::Long, 100.0, 1.0, dec!(0)),
    ];
    for (style, side, spot, quantity, expected) in cases {
        let opt = with(style, side, spot, quantity);
        assert_eq!(
            ok(opt.payoff(), "payoff"),
            expected,
            "payoff {style:?} {side:?} s={spot} qty={quantity}"
        );
    }
    let put = with(OptionStyle::Put, Side::Long, 100.0, 1.0);
    assert_eq!(
        ok(put.payoff_at_price(&pos_or_panic!(87.5)), "payoff"),
        dec!(12.5)
    );
    assert_eq!(
        ok(put.payoff_at_price(&pos_or_panic!(120.0)), "payoff"),
        dec!(0)
    );
}
