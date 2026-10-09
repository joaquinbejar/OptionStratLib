/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-10-05
******************************************************************************/

//! Implied-volatility round trips and error paths for
//! `optionstratlib-pricing` (#526).
//!
//! # Coverage
//!
//! Since #859 P4 the three entry points share one solver: Newton's method on
//! the Black-Scholes vega kept inside `σ ∈ [MIN_VOLATILITY, 5]` with a
//! bisection fallback, stopping when a step moves `σ` by at most `1e-13`
//! relative.
//!
//! * `OptionPricing::calculate_implied_volatility`: both sides (a short
//!   target is negated), any rate and dividend.
//! * `volatility::implied_volatility` and `volatility::calculate_iv` (cap 100
//!   steps, `r = q = 0`): long targets.
//!
//! Before #859 P4 the first was a bisection to a `1e-5` price tolerance and
//! a `1e-4` bracket, and the other two a grid search over `σ = i / 1000`.
//! The round trips below still hold those bounds; the realistic-grid test
//! holds the shared solver to `1e-12` relative. Error paths cover an expired
//! option, a target at the volatility floor, a target above the bracket and
//! targets outside the no-arbitrage band (#652).
//!
//! # Sources
//!
//! Hull, *Implied Volatilities*: the Black-Scholes price is strictly
//! increasing in `σ` and bounded by `max(S e^(-qT) - K e^(-rT), 0)` at
//! `σ → 0` and by `S e^(-qT)` (call) or `K e^(-rT)` (put) at `σ → ∞`, so
//! targets inside the band invert uniquely and targets outside it have no
//! implied volatility. Press et al., *Numerical Recipes*, 9.4 (`rtsafe`).
//!
//! # Tolerance policy
//!
//! The recovered volatility is the root of the computed price, so its error
//! against the true `σ` is the price's own rounding error amplified by the
//! condition number `κ = price / (ν σ)` (`ν` the vega per unit volatility):
//! `|σ̂ - σ| / σ ≈ κ · ε_price`. Where `κ ≤ 50` the realistic-grid test
//! requires `1e-12` relative; up to `κ = 1e4` (deep in-the-money options,
//! whose price is almost all intrinsic value) it requires `κ · 1e-13`
//! (`ε_price` measured at up to `1.1e-14`), and
//! beyond that (vega vanishes, `σ` is not identifiable from the price) it
//! checks nothing. For the bisection-era round trips: within the bracket
//! floor `1e-4`, or `IV_TOLERANCE / ν` where `ν < 0.2` (skipped, counted).

use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::error::VolatilityError;
use optionstratlib_pricing::greeks::vega;
use optionstratlib_pricing::pricing::{OptionPricing, black_scholes};
use optionstratlib_pricing::volatility::{calculate_iv, implied_volatility};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;

/// Bracket floor of the bisection: it stops once `high - low < 1e-4`.
const BISECTION_BRACKET: Decimal = dec!(0.0001);

/// Price stopping rule of the bisection, equal to the crate-private
/// `pricing::constants::IV_TOLERANCE`.
const BISECTION_PRICE_TOL: Decimal = dec!(0.00001);

/// One step of the `calculate_iv` grid (`1 / (100 · 10)`).
const GRID_STEP: Decimal = dec!(0.001);

/// Unwraps a fallible library call, failing the test with context.
fn ok<T, E: Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: unexpected error {err:?}"),
    }
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
    Options::new(
        OptionType::European,
        side,
        "IV".to_string(),
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

/// Smallest vega per unit volatility at which the price stopping rule
/// (`1e-5`) still pins the volatility to `2 · 1e-5 / ν ≤ 1e-4`.
const MIN_WELL_CONDITIONED_VEGA: Decimal = dec!(0.2);

/// Vega per unit volatility (`greeks::vega` is per 1 %) of the long option.
fn vega_per_unit(opt: &Options) -> Decimal {
    let mut long = opt.clone();
    long.side = Side::Long;
    ok(vega(&long), "vega") * dec!(100)
}

/// Bisection round trip: `S = 100`, `K ∈ {80, 90, 100, 110, 125}`,
/// `T ∈ {30 d, 0.5 y, 2 y}`, `σ ∈ {12, 35, 80} %`, `r = 3 %`, `q = 1 %`,
/// calls and puts (90 points). Points with `ν < 0.2` are skipped (see the
/// tolerance policy); 84 of the 90 qualify and the test requires at least
/// 80 checked points, so a vega regression cannot empty the grid. Each checked
/// point also reprices at the recovered volatility and requires the target
/// within the larger of `IV_TOLERANCE` and the bracket's price equivalent
/// `2 · 1e-4 · ν`.
#[test]
fn test_bisection_round_trip_recovers_volatility_grid() {
    let mut checked = 0usize;
    for strike in [80.0, 90.0, 100.0, 110.0, 125.0] {
        for days in [30.0, 182.5, 730.0] {
            for vol in [0.12, 0.35, 0.80] {
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    let opt = european(
                        style,
                        Side::Long,
                        100.0,
                        strike,
                        days,
                        vol,
                        dec!(0.03),
                        0.01,
                    );
                    let nu = vega_per_unit(&opt);
                    if nu < MIN_WELL_CONDITIONED_VEGA {
                        continue;
                    }
                    checked += 1;
                    let target = ok(black_scholes(&opt), "price");
                    let solved = ok(opt.calculate_implied_volatility(target), "bisection");
                    let error = (solved.to_dec() - pos_or_panic!(vol).to_dec()).abs();
                    let ctx = format!("{style:?} k={strike} days={days} vol={vol}");
                    assert!(
                        error <= BISECTION_BRACKET,
                        "{ctx}: recovered {solved}, |error| {error} > {BISECTION_BRACKET}"
                    );
                    let mut repriced = opt.clone();
                    repriced.implied_volatility = solved;
                    let price_error = (ok(black_scholes(&repriced), "reprice") - target).abs();
                    let price_bound = BISECTION_PRICE_TOL.max(BISECTION_BRACKET * nu * dec!(2));
                    assert!(
                        price_error <= price_bound,
                        "{ctx}: repriced error {price_error} > {price_bound}"
                    );
                }
            }
        }
    }
    assert!(
        checked >= 80,
        "only {checked} well-conditioned points checked"
    );
}

/// The bisection negates a short target before solving, so a short price
/// inverts to the same volatility as the long one.
#[test]
fn test_bisection_round_trip_short_side_recovers_volatility() {
    for strike in [90.0, 100.0, 110.0] {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let short = european(
                style,
                Side::Short,
                100.0,
                strike,
                182.5,
                0.27,
                dec!(0.03),
                0.0,
            );
            let target = ok(black_scholes(&short), "short price");
            assert!(target < Decimal::ZERO, "short price must be negative");
            assert!(vega_per_unit(&short) >= MIN_WELL_CONDITIONED_VEGA);
            let solved = ok(short.calculate_implied_volatility(target), "bisection");
            let error = (solved.to_dec() - dec!(0.27)).abs();
            assert!(
                error <= BISECTION_BRACKET,
                "short {style:?} k={strike}: recovered {solved}, |error| {error} > {BISECTION_BRACKET}"
            );
        }
    }
}

/// Accuracy of the shared solver (#859 P4) against the true volatility:
/// `S = 100`, `K ∈ {70, 80, 90, 95, 100, 105, 110, 120, 130}`,
/// `T ∈ {7, 30, 91, 182.5, 365, 730} d`, `σ ∈ {8, 15, 25, 40, 70, 120} %`,
/// calls and puts, through `calculate_implied_volatility` (`r = 3 %`,
/// `q = 1 %`, both sides) and `calculate_iv` (`r = q = 0`). See the
/// tolerance policy for the bound at each condition number `κ`; at least
/// 1 000 of the 1 296 bisection-entry points must reach `1e-12`.
#[test]
fn test_shared_solver_recovers_volatility_on_realistic_grid() {
    let tight = dec!(1e-12);
    let mut tight_checked = 0usize;
    for strike in [70.0, 80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0, 130.0] {
        for days in [7.0, 30.0, 91.0, 182.5, 365.0, 730.0] {
            for vol in [0.08, 0.15, 0.25, 0.40, 0.70, 1.20] {
                let sigma = pos_or_panic!(vol).to_dec();
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    for (rate, dividend) in [(dec!(0.03), 0.01), (Decimal::ZERO, 0.0)] {
                        let long =
                            european(style, Side::Long, 100.0, strike, days, vol, rate, dividend);
                        let price = ok(black_scholes(&long), "price");
                        let slope = vega_per_unit(&long) * sigma;
                        if price <= Decimal::ZERO || slope <= dec!(1e-20) {
                            continue;
                        }
                        let kappa = price / slope;
                        if kappa > dec!(1e4) {
                            continue;
                        }
                        let bound = if kappa <= dec!(50) {
                            tight
                        } else {
                            kappa * dec!(1e-13)
                        };
                        let ctx = format!(
                            "{style:?} k={strike} days={days} vol={vol} r={rate} kappa={kappa}"
                        );
                        let relative = |solved: Positive| ((solved.to_dec() - sigma) / sigma).abs();
                        let mut solutions = Vec::new();
                        if rate.is_zero() {
                            solutions.push(ok(
                                calculate_iv(
                                    ok(Positive::new_decimal(price), "positive price"),
                                    pos_or_panic!(strike),
                                    style,
                                    pos_or_panic!(100.0),
                                    pos_or_panic!(days),
                                    "IV".to_string(),
                                ),
                                "calculate_iv",
                            ));
                        } else {
                            for side in [Side::Long, Side::Short] {
                                let opt =
                                    european(style, side, 100.0, strike, days, vol, rate, dividend);
                                let target = ok(black_scholes(&opt), "signed price");
                                solutions.push(ok(
                                    opt.calculate_implied_volatility(target),
                                    "calculate_implied_volatility",
                                ));
                                if kappa <= dec!(50) {
                                    tight_checked += 1;
                                }
                            }
                        }
                        for solved in solutions {
                            let error = relative(solved);
                            assert!(
                                error <= bound,
                                "{ctx}: recovered {solved}, error {error} > {bound}"
                            );
                        }
                    }
                }
            }
        }
    }
    assert!(
        tight_checked >= 1000,
        "only {tight_checked} points held to 1e-12"
    );
}

/// Grid-search round trip through `calculate_iv` (`r = q = 0`):
/// `S = 100`, `K ∈ {90, 100, 110}`, `T ∈ {30 d, 0.5 y, 1 y}`,
/// `σ ∈ {12.34, 45.67} %` (off the `1e-3` grid), calls and puts.
#[test]
fn test_grid_search_round_trip_recovers_volatility_grid() {
    for strike in [90.0, 100.0, 110.0] {
        for days in [30.0, 182.5, 365.0] {
            for vol in [0.1234, 0.4567] {
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    let opt = european(
                        style,
                        Side::Long,
                        100.0,
                        strike,
                        days,
                        vol,
                        Decimal::ZERO,
                        0.0,
                    );
                    let target = ok(black_scholes(&opt), "price");
                    let solved = ok(
                        calculate_iv(
                            ok(Positive::new_decimal(target), "positive price"),
                            pos_or_panic!(strike),
                            style,
                            pos_or_panic!(100.0),
                            pos_or_panic!(days),
                            "IV".to_string(),
                        ),
                        "calculate_iv",
                    );
                    let error = (solved.to_dec() - pos_or_panic!(vol).to_dec()).abs();
                    assert!(
                        error <= GRID_STEP,
                        "{style:?} k={strike} days={days} vol={vol}: recovered {solved}, |error| {error} > {GRID_STEP}"
                    );
                }
            }
        }
    }
}

/// An expired option has no implied volatility: the grid search finds no
/// candidate that prices (`NoValidVolatility`) and the bisection surfaces the
/// Black-Scholes input error (`VolatilityError::Options`).
#[test]
fn test_expired_option_returns_documented_errors() {
    let grid = calculate_iv(
        pos_or_panic!(5.0),
        pos_or_panic!(100.0),
        OptionStyle::Call,
        pos_or_panic!(120.0),
        Positive::ZERO,
        "IV".to_string(),
    );
    assert!(
        matches!(grid, Err(VolatilityError::NoValidVolatility)),
        "grid search on an expired option: {grid:?}"
    );
    let expired = european(
        OptionStyle::Call,
        Side::Long,
        120.0,
        100.0,
        0.0,
        0.27,
        Decimal::ZERO,
        0.0,
    );
    let bisection = expired.calculate_implied_volatility(dec!(25));
    assert!(
        matches!(bisection, Err(VolatilityError::Options(_))),
        "bisection on an expired option: {bisection:?}"
    );
}

/// A zero target for an out-of-the-money call is already reached at the
/// volatility floor, so no volatility is implied: `calculate_iv` reports
/// `IvNotFound`, as the grid search it replaced did at its lowest point.
#[test]
fn test_target_at_volatility_floor_returns_iv_not_found() {
    let result = calculate_iv(
        Positive::ZERO,
        pos_or_panic!(120.0),
        OptionStyle::Call,
        pos_or_panic!(100.0),
        pos_or_panic!(30.0),
        "IV".to_string(),
    );
    assert!(
        matches!(result, Err(VolatilityError::IvNotFound)),
        "zero target for an otm call: {result:?}"
    );
}

/// `max_iterations` caps solver steps. The grid search sized itself as
/// `100 · max_iterations` and reported `NumericalFailure` when that
/// overflowed; the solver clamps a cap above `u32::MAX` and converges in a
/// handful of steps.
#[test]
fn test_implied_volatility_huge_iteration_cap_converges() {
    let mut opt = european(
        OptionStyle::Call,
        Side::Long,
        100.0,
        100.0,
        30.0,
        0.2,
        Decimal::ZERO,
        0.0,
    );
    let target = ok(black_scholes(&opt), "price");
    let solved = ok(
        implied_volatility(
            ok(Positive::new_decimal(target), "positive price"),
            &mut opt,
            i64::MAX,
        ),
        "huge cap",
    );
    let error = ((solved.to_dec() - dec!(0.2)) / dec!(0.2)).abs();
    assert!(error <= dec!(1e-12), "recovered {solved}, error {error}");
}

/// Targets outside the no-arbitrage band have no implied volatility: a call
/// below its intrinsic value (`S = 120, K = 100, r = 0`, target `15 < 20`)
/// and a call above the spot (target `130 > 120`). The bisection used to
/// return the edges of its `[0, 5]` bracket, `0.0000763` and `4.99992`
/// (#652); it now reports `InvalidPrice`.
#[test]
fn test_bisection_rejects_targets_outside_the_arbitrage_band() {
    let itm = european(
        OptionStyle::Call,
        Side::Long,
        120.0,
        100.0,
        90.0,
        0.27,
        Decimal::ZERO,
        0.0,
    );
    let below = itm.calculate_implied_volatility(dec!(15));
    assert!(below.is_err(), "target below intrinsic: {below:?}");
    assert!(
        matches!(below, Err(VolatilityError::InvalidPrice { .. })),
        "target below intrinsic: {below:?}"
    );
    let above = itm.calculate_implied_volatility(dec!(130));
    assert!(above.is_err(), "target above spot: {above:?}");
    assert!(
        matches!(above, Err(VolatilityError::InvalidPrice { .. })),
        "target above spot: {above:?}"
    );
}

/// Same targets through the grid search. It used to return `0.046` below
/// intrinsic (a deep in-the-money price that rounds a hair under 20 at that
/// grid point won the argmin) and `0.999`, the top of its grid, above the
/// spot (#652).
#[test]
fn test_grid_search_rejects_targets_outside_the_arbitrage_band() {
    let solve = |target: f64| {
        calculate_iv(
            pos_or_panic!(target),
            pos_or_panic!(100.0),
            OptionStyle::Call,
            pos_or_panic!(120.0),
            pos_or_panic!(90.0),
            "IV".to_string(),
        )
    };
    let below = solve(15.0);
    assert!(
        matches!(below, Err(VolatilityError::IvNotFound)),
        "target below intrinsic: {below:?}"
    );
    let above = solve(130.0);
    assert!(above.is_err(), "target above spot: {above:?}");
}

/// A put target above `K e^(-rT)` and a short call target whose magnitude
/// is below intrinsic sit outside the band on the other legs of the check:
/// the put's upper bound and the short side's sign flip.
#[test]
fn test_bisection_rejects_put_above_discounted_strike_and_short_below_intrinsic() {
    let put = european(
        OptionStyle::Put,
        Side::Long,
        100.0,
        100.0,
        365.0,
        0.2,
        dec!(0.05),
        0.0,
    );
    // K e^(-rT) = 100 e^(-0.05) = 95.1229.
    let above = put.calculate_implied_volatility(dec!(96));
    assert!(
        matches!(above, Err(VolatilityError::InvalidPrice { .. })),
        "put above discounted strike: {above:?}"
    );
    let short = european(
        OptionStyle::Call,
        Side::Short,
        120.0,
        100.0,
        90.0,
        0.27,
        Decimal::ZERO,
        0.0,
    );
    let below = short.calculate_implied_volatility(dec!(-15));
    assert!(
        matches!(below, Err(VolatilityError::InvalidPrice { .. })),
        "short target below intrinsic: {below:?}"
    );
}

/// A target inside the band whose implied volatility exceeds the `500 %`
/// bracket is not bracketed: the bisection reports `NoConvergence` instead
/// of the bracket top (#652). `S = K = 100, T = 1, r = q = 0`: the price
/// at `σ = 5` is `100 (2 N(2.5) - 1) = 98.758`, the band's upper bound is
/// `100`, and the target is `99.5`.
#[test]
fn test_bisection_target_above_bracket_top_returns_no_convergence() {
    let atm = european(
        OptionStyle::Call,
        Side::Long,
        100.0,
        100.0,
        365.0,
        0.2,
        Decimal::ZERO,
        0.0,
    );
    let result = atm.calculate_implied_volatility(dec!(99.5));
    assert!(
        matches!(result, Err(VolatilityError::NoConvergence { .. })),
        "target above the bracket top: {result:?}"
    );
    // The bracket top itself still inverts.
    let mut top = atm.clone();
    top.implied_volatility = pos_or_panic!(4.99);
    let target = ok(black_scholes(&top), "price at the bracket top");
    let solved = ok(atm.calculate_implied_volatility(target), "bisection");
    assert!(
        (solved.to_dec() - dec!(4.99)).abs() <= BISECTION_BRACKET,
        "recovered {solved}"
    );
}

/// A target whose implied volatility exceeds the old grid (`σ = 1.5 >
/// 0.999`) used to be `IvNotFound` (#652); the solver's bracket reaches
/// `σ = 5`, so it is found.
#[test]
fn test_target_above_old_grid_top_is_found() {
    let opt = european(
        OptionStyle::Call,
        Side::Long,
        100.0,
        100.0,
        365.0,
        1.5,
        Decimal::ZERO,
        0.0,
    );
    let target = ok(black_scholes(&opt), "price");
    let solved = ok(
        calculate_iv(
            ok(Positive::new_decimal(target), "positive price"),
            pos_or_panic!(100.0),
            OptionStyle::Call,
            pos_or_panic!(100.0),
            pos_or_panic!(365.0),
            "IV".to_string(),
        ),
        "calculate_iv",
    );
    let error = ((solved.to_dec() - dec!(1.5)) / dec!(1.5)).abs();
    assert!(error <= dec!(1e-12), "recovered {solved}, error {error}");
}
