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
//! The crate has two solvers, both inverted against Black-Scholes:
//!
//! * `OptionPricing::calculate_implied_volatility`: bisection on
//!   `σ ∈ [0, 5]`, stopping when the repriced value is within `IV_TOLERANCE`
//!   (`1e-5`, crate-private in `pricing::constants`) of the target or when
//!   the bracket is narrower than `1e-4`.
//! * `volatility::calculate_iv` (through `volatility::implied_volatility`
//!   with 10 iterations): a grid search over `σ = i / 1000`,
//!   `i = 1, ..., 999`, at `r = q = 0`.
//!
//! Round trips price an option at a known `σ`, solve for the volatility from
//! that price and require the known `σ` back within the solver's own
//! resolution, across moneyness, maturities, both styles and, for the
//! bisection, both sides. Error paths cover an expired option, an
//! out-of-the-money target at the zero-volatility floor, an overflowing grid
//! size. Targets outside the no-arbitrage band are #652.
//!
//! # Sources
//!
//! Hull, *Implied Volatilities*: the Black-Scholes price is strictly
//! increasing in `σ` and bounded by `max(S e^(-qT) - K e^(-rT), 0)` at
//! `σ → 0` and by `S e^(-qT)` (call) or `K e^(-rT)` (put) at `σ → ∞`, so
//! targets inside the band invert uniquely and targets outside it have no
//! implied volatility.
//!
//! # Tolerance policy
//!
//! The bisection returns when either stopping rule fires, so the recovered
//! volatility is within the bracket floor `1e-4`, or within
//! `IV_TOLERANCE / ν` of the true one when the price rule fires first (`ν`
//! the vega per unit volatility). Where `ν < 0.2` the price rule alone
//! admits volatility errors above `1e-4` (implied volatility is
//! ill-conditioned where vega vanishes, e.g. short-dated deep
//! out-of-the-money options); those grid points are skipped and counted,
//! and every remaining point is held to `1e-4` (`2 · 1e-5 / 0.2`, the
//! factor 2 covering the curvature of the price in `σ`). The grid search
//! picks the grid point whose
//! price is nearest the target, which for a strictly increasing price is one
//! of the two grid neighbours of the true `σ`: the bound is one grid step,
//! `1e-3`. Test volatilities sit off the grid on purpose.
//!
//! # Known discrepancies (filed, tested in the fix)
//!
//! Both solvers return a volatility for targets outside the no-arbitrage
//! band instead of an error (#652); that test lives in the issue.

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

/// A zero target for an out-of-the-money call sits at the `σ → 0` floor:
/// the nearest grid price is the lowest grid point, which the grid search
/// reports as `IvNotFound`.
#[test]
fn test_grid_search_target_at_zero_volatility_floor_returns_iv_not_found() {
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

/// `implied_volatility` sizes its grid as `100 · max_iterations`; an
/// overflowing product is reported, not wrapped.
#[test]
fn test_grid_search_overflowing_grid_size_returns_numerical_failure() {
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
    let result = implied_volatility(pos_or_panic!(2.0), &mut opt, i64::MAX);
    assert!(
        matches!(result, Err(VolatilityError::NumericalFailure { .. })),
        "overflowing grid: {result:?}"
    );
}
