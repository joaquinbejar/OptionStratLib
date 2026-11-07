//! The acceptance checks of the VIX under the quadratic rough Heston model (#968).
//!
//! Every Monte Carlo check uses a fixed seed and an interval of `Z_SCORE` standard errors
//! around its target. No parameter set here comes from either paper: all are illustrative.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::decimal_to_f64;
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::deterministic_rng;
use optionstratlib_simulation::error::QuadraticRoughHestonError;
use optionstratlib_simulation::simulation::{
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
    QuadraticRoughHestonVix, QuadraticRoughHestonVixEstimate, QuadraticRoughHestonVixPayoff,
    VIX_WINDOW_DAYS, price_vix_quadratic_rough_heston, simulate_quadratic_rough_heston_path,
};
use rand::RngExt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

/// Half width of a Monte Carlo interval, in standard errors.
const Z_SCORE: f64 = 4.0;

const SEED: u64 = 968;

fn paths(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

fn kernel() -> QuadraticRoughHestonKernel {
    QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5)).unwrap()
}

/// `30 / 365` years.
fn window() -> Positive {
    Positive::new_decimal(VIX_WINDOW_DAYS / dec!(365)).unwrap()
}

fn as_f64(value: Decimal) -> f64 {
    decimal_to_f64(value).unwrap()
}

/// Feedback strong enough to move the variance, and far from any explosion.
fn feedback_params(initial_factors: Vec<Decimal>) -> QuadraticRoughHestonParams {
    QuadraticRoughHestonParams::try_new(
        dec!(0.5),
        dec!(0.1),
        dec!(0.01),
        dec!(1.5),
        dec!(1),
        kernel(),
        initial_factors,
        Positive::HUNDRED,
    )
    .unwrap()
}

/// `a = 0`, so `V` stays at `c = 0.04` and the index at 20 points whatever the factors do.
fn constant_variance_params() -> QuadraticRoughHestonParams {
    QuadraticRoughHestonParams::try_new(
        Decimal::ZERO,
        dec!(0.1),
        dec!(0.04),
        dec!(1.5),
        dec!(1),
        kernel(),
        vec![dec!(-0.05); 5],
        Positive::HUNDRED,
    )
    .unwrap()
}

/// `eta` and `a` so large that the factors overflow an `f64` within a few steps of a path,
/// while the conditional moments over the window stay finite.
fn exploding_params() -> QuadraticRoughHestonParams {
    QuadraticRoughHestonParams::try_new(
        dec!(10000000000),
        Decimal::ZERO,
        dec!(1),
        dec!(1),
        dec!(10000000000),
        kernel(),
        vec![Decimal::ZERO; 5],
        Positive::HUNDRED,
    )
    .unwrap()
}

fn price(
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
    payoff: QuadraticRoughHestonVixPayoff,
    count: usize,
    seed: u64,
) -> QuadraticRoughHestonVixEstimate {
    price_vix_quadratic_rough_heston(params, grid, window(), payoff, paths(count), seed).unwrap()
}

fn call(strike: f64) -> QuadraticRoughHestonVixPayoff {
    QuadraticRoughHestonVixPayoff::Vanilla {
        style: OptionStyle::Call,
        strike: Positive::new(strike).unwrap(),
    }
}

fn put(strike: f64) -> QuadraticRoughHestonVixPayoff {
    QuadraticRoughHestonVixPayoff::Vanilla {
        style: OptionStyle::Put,
        strike: Positive::new(strike).unwrap(),
    }
}

/// `VIX_t^2` at a factor state against a nested Monte Carlo: paths of the model started from
/// that state, over the window, averaging the trapezoid rule on `V` along each. The nested
/// estimate is of the scheme (A.1) with `N` steps, whose conditional mean of `V` differs from
/// the model's by `O(dt)`, so the interval is widened by `BIAS_ALLOWANCE` squared points.
///
/// The three states cover factors below `b`, above it and spread across the factors. At
/// `N = 120` and 8,000 paths the nested estimates sit `1.3`, `24` and `8.2` squared points from
/// the model, with standard errors of `2.1`, `13` and `3.1`; at `N = 480` and 40,000 paths they
/// sit `0.1`, `2.9` and `2.0` away, with standard errors of `0.9`, `5.9` and `1.4`, as an `O(dt)`
/// bias does.
#[test]
fn test_vix_matches_a_nested_monte_carlo_within_its_tolerance() {
    const INNER_PATHS: usize = 8_000;
    const INNER_STEPS: usize = 120;
    const BIAS_ALLOWANCE: f64 = 1.0;
    let states = [
        vec![dec!(-0.05); 5],
        vec![dec!(0.3); 5],
        vec![dec!(-0.4), dec!(-0.2), dec!(0), dec!(0.2), dec!(0.4)],
    ];
    for state in states {
        let params = feedback_params(state.clone());
        let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();
        let model = as_f64(vix.squared_at(&state).unwrap());

        let grid = QuadraticRoughHestonGrid::try_new(window().to_dec(), INNER_STEPS).unwrap();
        let dt = as_f64(grid.dt());
        let delta = as_f64(window().to_dec());
        let mut master = deterministic_rng(SEED);
        let mut sum = 0.0;
        let mut sum_of_squares = 0.0;
        for _ in 0..INNER_PATHS {
            let path =
                simulate_quadratic_rough_heston_path(&params, &grid, master.random()).unwrap();
            let variance: Vec<f64> = path.variance().iter().map(|v| as_f64(*v)).collect();
            let mut integral = 0.0;
            for pair in variance.windows(2) {
                integral += 0.5 * (pair[0] + pair[1]) * dt;
            }
            let squared = 10_000.0 / delta * integral;
            sum += squared;
            sum_of_squares += squared * squared;
        }
        let count = INNER_PATHS as f64;
        let mean = sum / count;
        let standard_error = ((sum_of_squares / count - mean * mean) / (count - 1.0)).sqrt();
        assert!(
            (model - mean).abs() <= Z_SCORE * standard_error + BIAS_ALLOWANCE,
            "state {state:?}: model {model}, nested {mean} +/- {standard_error}"
        );
    }
}

/// The future is the mean of `VIX_T` over the paths, and that mean agrees with the one read
/// off independently simulated paths, at another seed, through the path API.
#[test]
fn test_vix_future_agrees_with_the_mean_index_at_maturity() {
    const PATHS: usize = 20_000;
    const CHECK_PATHS: usize = 4_000;
    let params = feedback_params(vec![dec!(-0.05); 5]);
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 50).unwrap();
    let estimate = price(
        &params,
        &grid,
        QuadraticRoughHestonVixPayoff::Future,
        PATHS,
        SEED,
    );
    assert_eq!(estimate.price(), estimate.future());
    assert_eq!(estimate.standard_error(), estimate.future_standard_error());
    assert_eq!(estimate.paths(), PATHS);

    let vix = QuadraticRoughHestonVix::try_new(&params, window()).unwrap();
    let mut master = deterministic_rng(SEED + 1);
    let mut sum = 0.0;
    let mut sum_of_squares = 0.0;
    for _ in 0..CHECK_PATHS {
        let path = simulate_quadratic_rough_heston_path(&params, &grid, master.random()).unwrap();
        let index = as_f64(
            vix.at(path.factors_at(grid.steps()).unwrap())
                .unwrap()
                .to_dec(),
        );
        sum += index;
        sum_of_squares += index * index;
    }
    let count = CHECK_PATHS as f64;
    let mean = sum / count;
    let check_error = ((sum_of_squares / count - mean * mean) / (count - 1.0)).sqrt();
    let future = as_f64(estimate.future());
    let error = (as_f64(estimate.future_standard_error()).powi(2) + check_error.powi(2)).sqrt();
    assert!(
        (future - mean).abs() <= Z_SCORE * error,
        "future {future}, independent mean {mean}, error {error}"
    );
}

/// On the same paths, a call struck at zero is the future, and `C - P = F - K` exactly.
#[test]
fn test_vix_call_and_put_satisfy_parity_on_the_same_paths() {
    let params = feedback_params(vec![dec!(-0.05); 5]);
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 50).unwrap();
    let future = price(
        &params,
        &grid,
        QuadraticRoughHestonVixPayoff::Future,
        5_000,
        SEED,
    );
    let zero_strike = price_vix_quadratic_rough_heston(
        &params,
        &grid,
        window(),
        QuadraticRoughHestonVixPayoff::Vanilla {
            style: OptionStyle::Call,
            strike: Positive::ZERO,
        },
        paths(5_000),
        SEED,
    )
    .unwrap();
    assert_eq!(zero_strike.price(), future.price());

    for strike in [10.0, 15.0, 25.0] {
        let call = price(&params, &grid, call(strike), 5_000, SEED);
        let put = price(&params, &grid, put(strike), 5_000, SEED);
        assert_eq!(call.future(), future.future());
        assert_eq!(put.future(), future.future());
        let parity = as_f64(call.price()) - as_f64(put.price());
        let expected = as_f64(future.price()) - strike;
        assert!(
            (parity - expected).abs() < 1e-9,
            "strike {strike}: {parity} against {expected}"
        );
        assert!(call.price() >= Decimal::ZERO && put.price() >= Decimal::ZERO);
    }
}

/// With `a = 0` the index is 20 points on every path: the future is 20 with no error, a call
/// at 15 is 5, a put at 15 is 0, and the index at the initial state agrees.
#[test]
fn test_vix_zero_feedback_prices_are_exact() {
    let params = constant_variance_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.5), 20).unwrap();
    let close = |actual: Decimal, expected: Decimal| (actual - expected).abs() < dec!(0.000000001);
    let future = price(
        &params,
        &grid,
        QuadraticRoughHestonVixPayoff::Future,
        100,
        SEED,
    );
    assert!(close(future.price(), dec!(20)), "{future:?}");
    assert!(close(future.standard_error(), Decimal::ZERO), "{future:?}");
    let call = price(&params, &grid, call(15.0), 100, SEED);
    assert!(close(call.price(), dec!(5)), "{call:?}");
    let put = price(&params, &grid, put(15.0), 100, SEED);
    assert!(close(put.price(), Decimal::ZERO), "{put:?}");
    let vix = QuadraticRoughHestonVix::thirty_days(&params).unwrap();
    assert!(close(
        vix.at(params.initial_factors()).unwrap().to_dec(),
        dec!(20)
    ));
}

/// The same seed reproduces a price, and another seed changes it.
#[test]
fn test_vix_price_same_seed_reproduces_and_another_seed_differs() {
    let params = feedback_params(vec![dec!(-0.05); 5]);
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    let first = price(&params, &grid, call(20.0), 500, SEED);
    let again = price(&params, &grid, call(20.0), 500, SEED);
    let other = price(&params, &grid, call(20.0), 500, SEED + 1);
    assert_eq!(first, again);
    assert_ne!(first.price(), other.price());
}

fn price_on_threads(threads: usize, params: &QuadraticRoughHestonParams) -> Decimal {
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(|| price(params, &grid, call(20.0), 2_000, SEED).price())
}

#[test]
fn test_vix_price_does_not_depend_on_the_number_of_threads() {
    let params = feedback_params(vec![dec!(-0.05); 5]);
    let single = price_on_threads(1, &params);
    let several = price_on_threads(4, &params);
    assert_eq!(single, several);
}

#[test]
fn test_vix_price_single_path_returns_typed_error() {
    let params = feedback_params(vec![dec!(-0.05); 5]);
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    let result = price_vix_quadratic_rough_heston(
        &params,
        &grid,
        window(),
        QuadraticRoughHestonVixPayoff::Future,
        paths(1),
        SEED,
    );
    assert!(matches!(
        result,
        Err(QuadraticRoughHestonError::InvalidParameter {
            parameter: "paths",
            ..
        })
    ));
}

#[test]
fn test_vix_price_zero_window_returns_typed_error() {
    let params = feedback_params(vec![dec!(-0.05); 5]);
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    let result = price_vix_quadratic_rough_heston(
        &params,
        &grid,
        Positive::ZERO,
        QuadraticRoughHestonVixPayoff::Future,
        paths(100),
        SEED,
    );
    assert!(matches!(
        result,
        Err(QuadraticRoughHestonError::InvalidParameter {
            parameter: "window",
            ..
        })
    ));
}

/// A path that stops being finite fails the price with its step, and the moments over the
/// window fail on their own when they overflow.
#[test]
fn test_vix_exploding_runs_return_typed_errors() {
    let params = exploding_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    let result = price_vix_quadratic_rough_heston(
        &params,
        &grid,
        pos_or_panic!(0.000001),
        QuadraticRoughHestonVixPayoff::Future,
        paths(100),
        SEED,
    );
    assert!(
        matches!(
            result,
            Err(QuadraticRoughHestonError::NonFinite { .. })
                | Err(QuadraticRoughHestonError::VixMomentsNotFinite { .. })
        ),
        "{result:?}"
    );
    assert!(matches!(
        QuadraticRoughHestonVix::try_new(&params, window()),
        Err(QuadraticRoughHestonError::VixMomentsNotFinite { .. })
    ));
}
