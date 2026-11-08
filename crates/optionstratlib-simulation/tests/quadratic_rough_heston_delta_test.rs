//! The acceptance checks of the delta along the Brownian of the quadratic rough Heston model
//! (#974). Every Monte Carlo check uses a fixed seed; no parameter set comes from a paper.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::decimal_to_f64;
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_core::pos_or_panic;
use optionstratlib_simulation::error::QuadraticRoughHestonError;
use optionstratlib_simulation::simulation::{
    QuadraticRoughHestonBrownianDelta, QuadraticRoughHestonGrid, QuadraticRoughHestonInstrument,
    QuadraticRoughHestonKernel, QuadraticRoughHestonParams, QuadraticRoughHestonVix,
    QuadraticRoughHestonVixPayoff, brownian_delta_quadratic_rough_heston,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

/// Half width of a Monte Carlo interval, in standard errors.
const Z_SCORE: f64 = 4.0;

const SEED: u64 = 974;

const SPOT: f64 = 100.0;

fn count(paths: usize) -> NonZeroUsize {
    NonZeroUsize::new(paths).unwrap()
}

fn as_f64(value: Decimal) -> f64 {
    decimal_to_f64(value).unwrap()
}

fn window() -> Positive {
    Positive::new_decimal(dec!(30) / dec!(365)).unwrap()
}

fn params(a: Decimal, c: Decimal) -> QuadraticRoughHestonParams {
    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5)).unwrap();
    QuadraticRoughHestonParams::try_new(
        a,
        dec!(0.1),
        c,
        dec!(1),
        dec!(0.2),
        kernel,
        vec![dec!(0.02); 5],
        pos_or_panic!(SPOT),
    )
    .unwrap()
}

fn feedback() -> QuadraticRoughHestonParams {
    params(dec!(0.5), dec!(0.03))
}

fn grid(horizon: Decimal, steps: usize) -> QuadraticRoughHestonGrid {
    QuadraticRoughHestonGrid::try_new(horizon, steps).unwrap()
}

fn vanilla(style: OptionStyle, strike: f64) -> QuadraticRoughHestonInstrument {
    QuadraticRoughHestonInstrument::Vanilla {
        style,
        strike: pos_or_panic!(strike),
    }
}

fn vix_future() -> QuadraticRoughHestonInstrument {
    QuadraticRoughHestonInstrument::Vix {
        window: window(),
        payoff: QuadraticRoughHestonVixPayoff::Future,
    }
}

fn delta(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    instrument: QuadraticRoughHestonInstrument,
    bump: f64,
    paths: usize,
) -> QuadraticRoughHestonBrownianDelta {
    brownian_delta_quadratic_rough_heston(
        params,
        grid,
        instrument,
        pos_or_panic!(bump),
        count(paths),
        SEED,
    )
    .unwrap()
}

fn assert_within(estimate: &QuadraticRoughHestonBrownianDelta, target: f64, tolerance: f64) {
    let value = as_f64(estimate.delta());
    let band = Z_SCORE * as_f64(estimate.standard_error()) + tolerance;
    assert!(
        (value - target).abs() <= band,
        "delta {value} is not within {band} of {target}"
    );
}

#[test]
fn test_constant_volatility_call_has_the_black_scholes_delta() {
    // `a = 0`, `V = 0.04`, `T = 1`, ATM: `d1 = 0.1`, `N(d1) = 0.539827837`.
    let estimate = delta(
        &params(dec!(0), dec!(0.04)),
        &grid(dec!(1), 200),
        vanilla(OptionStyle::Call, SPOT),
        0.02,
        100_000,
    );
    assert_within(&estimate, 0.539_827_837, 0.003);
}

#[test]
fn test_constant_volatility_vix_future_has_no_delta() {
    let estimate = delta(
        &params(dec!(0), dec!(0.04)),
        &grid(dec!(0.25), 20),
        vix_future(),
        0.02,
        1_000,
    );
    assert!(as_f64(estimate.delta()).abs() < 1e-9);
    assert!(as_f64(estimate.standard_error()).abs() < 1e-9);
}

#[test]
fn test_call_far_in_the_money_has_a_delta_of_one_under_feedback() {
    // The payoff is `S_T - K` and the spot a martingale, whatever the factors do.
    let estimate = delta(
        &feedback(),
        &grid(dec!(0.25), 100),
        vanilla(OptionStyle::Call, 1.0),
        0.02,
        50_000,
    );
    assert_within(&estimate, 1.0, 1e-3);
}

#[test]
fn test_call_minus_put_delta_is_one() {
    let model = feedback();
    let horizon = grid(dec!(0.25), 100);
    let call = delta(
        &model,
        &horizon,
        vanilla(OptionStyle::Call, SPOT),
        0.02,
        50_000,
    );
    let put = delta(
        &model,
        &horizon,
        vanilla(OptionStyle::Put, SPOT),
        0.02,
        50_000,
    );
    let gap = as_f64(call.delta()) - as_f64(put.delta());
    let band = Z_SCORE * (as_f64(call.standard_error()) + as_f64(put.standard_error()));
    assert!((gap - 1.0).abs() <= band + 1e-3, "gap {gap}, band {band}");
}

#[test]
fn test_short_horizon_vix_future_matches_the_gradient_of_the_index() {
    let model = feedback();
    let index = QuadraticRoughHestonVix::try_new(&model, window()).unwrap();
    let shifted = |shift: Decimal| -> f64 {
        let factors: Vec<Decimal> = model.initial_factors().iter().map(|z| z + shift).collect();
        as_f64(index.at(&factors).unwrap().to_dec())
    };
    let gradient = (shifted(dec!(0.001)) - shifted(dec!(-0.001))) / 0.002;
    let expected = 0.2 * gradient / SPOT;
    assert!(expected.abs() > 1e-4, "degenerate target {expected}");

    let estimate = delta(&model, &grid(dec!(0.0005), 5), vix_future(), 0.001, 20_000);
    assert_within(&estimate, expected, 0.02 * expected.abs());
}

#[test]
fn test_vix_future_converges_in_the_bump_at_second_order() {
    let model = feedback();
    let horizon = grid(dec!(0.25), 50);
    let at = |bump: f64| as_f64(delta(&model, &horizon, vix_future(), bump, 20_000).delta());
    let reference = at(0.002_5);
    let coarse = (at(0.08) - reference).abs();
    let medium = (at(0.04) - reference).abs();
    let fine = (at(0.02) - reference).abs();
    assert!(coarse > medium && medium > fine, "{coarse} {medium} {fine}");
    assert!(coarse / medium > 3.0, "ratio {}", coarse / medium);
    assert!(medium / fine > 3.0, "ratio {}", medium / fine);
}

#[test]
fn test_vanilla_delta_stabilises_as_the_bump_shrinks() {
    let model = feedback();
    let horizon = grid(dec!(0.25), 50);
    let call = vanilla(OptionStyle::Call, SPOT);
    let at = |bump: f64| as_f64(delta(&model, &horizon, call, bump, 50_000).delta());
    let reference = at(0.005);
    assert!((at(0.04) - reference).abs() > (at(0.01) - reference).abs());
}

#[test]
fn test_the_same_seed_gives_the_same_delta() {
    let model = feedback();
    let horizon = grid(dec!(0.25), 20);
    let instrument = QuadraticRoughHestonInstrument::Vix {
        window: window(),
        payoff: QuadraticRoughHestonVixPayoff::Vanilla {
            style: OptionStyle::Call,
            strike: pos_or_panic!(20.0),
        },
    };
    let first = delta(&model, &horizon, instrument, 0.01, 500);
    assert_eq!(first, delta(&model, &horizon, instrument, 0.01, 500));
    assert_eq!(first.paths(), 500);
}

#[test]
fn test_invalid_inputs_return_typed_errors() {
    let model = feedback();
    let horizon = grid(dec!(0.25), 10);
    let call = vanilla(OptionStyle::Call, SPOT);
    for (bump, paths) in [(0.0, 10), (0.2, 10), (0.5, 10), (0.01, 1)] {
        let outcome = brownian_delta_quadratic_rough_heston(
            &model,
            &horizon,
            call,
            pos_or_panic!(bump),
            count(paths),
            SEED,
        );
        assert!(
            matches!(
                outcome,
                Err(QuadraticRoughHestonError::InvalidParameter { .. })
            ),
            "bump {bump}, paths {paths}: {outcome:?}"
        );
    }
}

#[test]
fn test_exploding_paths_return_an_error_and_do_not_panic() {
    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5)).unwrap();
    let wild = QuadraticRoughHestonParams::try_new(
        dec!(1000000),
        dec!(0),
        dec!(1),
        dec!(0.001),
        dec!(100),
        kernel,
        vec![dec!(10); 5],
        pos_or_panic!(SPOT),
    )
    .unwrap();
    let outcome = brownian_delta_quadratic_rough_heston(
        &wild,
        &grid(dec!(10), 50),
        vanilla(OptionStyle::Call, SPOT),
        pos_or_panic!(0.01),
        count(100),
        SEED,
    );
    assert!(outcome.is_err(), "{outcome:?}");
}
