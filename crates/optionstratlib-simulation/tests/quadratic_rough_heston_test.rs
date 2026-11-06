//! The acceptance checks of the quadratic rough Heston simulation (#958).
//!
//! Every Monte Carlo check uses a fixed seed and an interval of `Z_SCORE` standard errors
//! around its target. No parameter set here comes from either paper: all are illustrative.

use num_traits::ToPrimitive;
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::pricing::black_scholes;
use optionstratlib_simulation::error::QuadraticRoughHestonError;
use optionstratlib_simulation::simulation::{
    QuadraticRoughHestonEstimate, QuadraticRoughHestonGrid, QuadraticRoughHestonKernel,
    QuadraticRoughHestonParams, price_vanilla_quadratic_rough_heston,
    simulate_quadratic_rough_heston_path,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use statrs::function::gamma::gamma;
use std::num::NonZeroUsize;

/// Half width of a Monte Carlo interval, in standard errors.
const Z_SCORE: Decimal = dec!(4);

const SEED: u64 = 958;

fn paths(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

fn kernel() -> QuadraticRoughHestonKernel {
    QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5)).unwrap()
}

/// Feedback strong enough to move the variance, and far from any explosion.
fn feedback_params() -> QuadraticRoughHestonParams {
    QuadraticRoughHestonParams::try_new(
        dec!(0.5),
        dec!(0.1),
        dec!(0.01),
        dec!(1.5),
        dec!(1),
        kernel(),
        vec![dec!(-0.05); 5],
        Positive::HUNDRED,
    )
    .unwrap()
}

/// `a = 0`, so `V` stays at `c = 0.04` whatever the factors do.
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

/// `eta` and `a` so large that the factors overflow an `f64` within a few steps.
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

/// Black-Scholes at `sigma^2 = 0.04`, one year, `r = q = 0`, from this repository.
fn black_scholes_price(style: OptionStyle, strike: Positive) -> Decimal {
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "QRH".to_string(),
        strike,
        ExpirationDate::Days(pos_or_panic!(365.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        Decimal::ZERO,
        style,
        Positive::ZERO,
        None,
    );
    black_scholes(&option).unwrap()
}

/// With `a = 0` the spot is an Euler discretization of a geometric Brownian motion, so its
/// price sits within a Monte Carlo interval around Black-Scholes, ATM and away from it. The
/// half widths here run from about `0.14` to `0.32`, and at `N = 100` the bias of the Euler step
/// is at most about `0.01`, measured on 4 million paths.
#[test]
fn test_price_zero_feedback_agrees_with_black_scholes() {
    let params = constant_variance_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(1), 100).unwrap();
    for style in [OptionStyle::Call, OptionStyle::Put] {
        for strike in [pos_or_panic!(90.0), Positive::HUNDRED, pos_or_panic!(110.0)] {
            let estimate = price_vanilla_quadratic_rough_heston(
                &params,
                &grid,
                style,
                strike,
                paths(40_000),
                SEED,
            )
            .unwrap();
            let reference = black_scholes_price(style, strike);
            let half_width = Z_SCORE * estimate.standard_error();
            assert!(
                (estimate.price() - reference).abs() < half_width,
                "{style:?} {strike}: {} against {reference}, half width {half_width}",
                estimate.price()
            );
        }
    }
}

#[test]
fn test_path_zero_feedback_keeps_the_variance_at_c() {
    let params = constant_variance_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 50).unwrap();
    let path = simulate_quadratic_rough_heston_path(&params, &grid, SEED).unwrap();
    assert_eq!(path.variance().len(), 51);
    assert!(
        path.variance()
            .iter()
            .all(|variance| *variance == dec!(0.04))
    );
    assert_eq!(params.initial_variance(), dec!(0.04));
}

/// The largest relative error of `K^n` against `K(t) = t^(alpha-1) / Gamma(alpha)` on 41 points
/// spaced geometrically over `t` in `[0.01, 0.1]` years, for each ratio.
fn kernel_error(alpha: Decimal, factors: usize, ratio: Decimal) -> f64 {
    let kernel = QuadraticRoughHestonKernel::try_new(alpha, factors, ratio).unwrap();
    let alpha = alpha.to_f64().unwrap();
    (0..=40)
        .map(|point| {
            let t = 0.01 * 10_f64.powf(f64::from(point) / 40.0);
            let exact = t.powf(alpha - 1.0) / gamma(alpha);
            let approximated = kernel
                .value_at(Positive::new(t).unwrap())
                .unwrap()
                .to_f64()
                .unwrap();
            ((approximated - exact) / exact).abs()
        })
        .fold(0.0, f64::max)
}

/// The error falls as `n` grows with a ratio that falls with it, on `t` in `[0.01, 0.1]`. The
/// ratios are a simple schedule for this check, not optima.
#[test]
fn test_kernel_approaches_the_fractional_kernel_as_factors_grow() {
    for alpha in [dec!(0.51), dec!(0.6), dec!(0.7)] {
        let errors: Vec<f64> = [
            (5, dec!(18)),
            (10, dec!(4)),
            (20, dec!(2.1)),
            (40, dec!(1.6)),
            (80, dec!(1.3)),
            (160, dec!(1.15)),
        ]
        .into_iter()
        .map(|(factors, ratio)| kernel_error(alpha, factors, ratio))
        .collect();
        assert!(
            errors.windows(2).all(|pair| pair[1] < pair[0]),
            "{alpha}: {errors:?}"
        );
        assert!(errors[errors.len() - 1] < 0.025, "{alpha}: {errors:?}");
    }
}

/// `S` has no drift, so the mean of `S_T` is `S_0`, with a feedback that moves the variance.
#[test]
fn test_price_mean_terminal_spot_agrees_with_the_initial_spot() {
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 50).unwrap();
    let estimate = price_vanilla_quadratic_rough_heston(
        &feedback_params(),
        &grid,
        OptionStyle::Call,
        Positive::HUNDRED,
        paths(100_000),
        SEED,
    )
    .unwrap();
    assert!(
        (estimate.mean_terminal_spot() - dec!(100)).abs()
            < Z_SCORE * estimate.terminal_spot_standard_error(),
        "{estimate:?}"
    );
    assert!(estimate.terminal_spot_standard_error() > Decimal::ZERO);
    assert_eq!(estimate.paths(), 100_000);
    assert_eq!(estimate.non_positive_paths(), 0);
}

/// The same paths under both styles, so `C - P = mean(S_T) - K` holds path by path.
#[test]
fn test_price_put_call_parity_holds_on_the_same_paths() {
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    let params = feedback_params();
    let strike = pos_or_panic!(105.0);
    let price = |style| {
        price_vanilla_quadratic_rough_heston(&params, &grid, style, strike, paths(2_000), SEED)
            .unwrap()
    };
    let call = price(OptionStyle::Call);
    let put = price(OptionStyle::Put);
    let parity = call.price() - put.price() - (call.mean_terminal_spot() - strike.to_dec());
    assert!(parity.abs() < dec!(0.000000001), "{parity}");
}

/// A step so coarse that `sqrt(V) dW <= -1` happens: the spot goes negative, it is counted
/// and not clamped, and a put on such a path pays more than its strike.
#[test]
fn test_price_non_positive_paths_are_counted_and_not_clamped() {
    let params = QuadraticRoughHestonParams::try_new(
        Decimal::ZERO,
        Decimal::ZERO,
        dec!(4),
        dec!(1),
        dec!(0.1),
        kernel(),
        vec![Decimal::ZERO; 5],
        Positive::HUNDRED,
    )
    .unwrap();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(1), 1).unwrap();
    let estimate = price_vanilla_quadratic_rough_heston(
        &params,
        &grid,
        OptionStyle::Put,
        Positive::ONE,
        paths(10_000),
        SEED,
    )
    .unwrap();
    // `S_1 = 100 (1 + 2 N(0, 1))`, not above zero with probability `Phi(-1/2)`, about 0.31.
    let share = estimate.non_positive_paths() as f64 / 10_000.0;
    assert!((share - 0.3085).abs() < 0.02, "{share}");
    // The mean of a clamped `S_1` would be well above `S_0`, and this one is on it.
    assert!(
        (estimate.mean_terminal_spot() - dec!(100)).abs()
            < Z_SCORE * estimate.terminal_spot_standard_error(),
        "{estimate:?}"
    );
    // A put of strike 1 pays only below it, so a clamp would cap this price at `1`.
    assert!(estimate.price() > Decimal::ONE, "{estimate:?}");
}

#[test]
fn test_path_same_seed_reproduces_and_another_seed_differs() {
    let params = feedback_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 30).unwrap();
    let first = simulate_quadratic_rough_heston_path(&params, &grid, SEED).unwrap();
    let again = simulate_quadratic_rough_heston_path(&params, &grid, SEED).unwrap();
    let other = simulate_quadratic_rough_heston_path(&params, &grid, SEED + 1).unwrap();
    assert_eq!(first, again);
    assert_ne!(first.spot(), other.spot());
    assert_ne!(first.z(), other.z());
}

#[test]
fn test_path_holds_every_step_from_the_initial_state() {
    let params = feedback_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 30).unwrap();
    let path = simulate_quadratic_rough_heston_path(&params, &grid, SEED).unwrap();
    assert_eq!(path.spot().len(), 31);
    assert_eq!(path.variance().len(), 31);
    assert_eq!(path.z().len(), 31);
    assert_eq!(path.spot()[0], dec!(100));
    assert_eq!(path.z()[0], params.initial_z());
    assert_eq!(path.variance()[0], params.initial_variance());
    assert_eq!(path.factors_at(0), Some(params.initial_factors()));
    assert_eq!(path.factors_at(30).map(<[Decimal]>::len), Some(5));
    assert_eq!(path.factors_at(31), None);
    assert_eq!(path.factors_at(usize::MAX), None);
    // The variance is `a (Z - b)^2 + c`, at each step, up to the rounding of a `Decimal`.
    for (z, variance) in path.z().iter().zip(path.variance()) {
        let expected = dec!(0.5) * (z - dec!(0.1)) * (z - dec!(0.1)) + dec!(0.01);
        assert!((expected - variance).abs() < dec!(0.00000001), "{z}");
    }
}

fn price_on_threads(
    threads: usize,
    params: &QuadraticRoughHestonParams,
    grid: &QuadraticRoughHestonGrid,
) -> Result<QuadraticRoughHestonEstimate, QuadraticRoughHestonError> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(|| {
            price_vanilla_quadratic_rough_heston(
                params,
                grid,
                OptionStyle::Call,
                Positive::HUNDRED,
                paths(5_000),
                SEED,
            )
        })
}

#[test]
fn test_price_does_not_depend_on_the_number_of_threads() {
    let params = feedback_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 20).unwrap();
    let one = price_on_threads(1, &params, &grid).unwrap();
    let several = price_on_threads(4, &params, &grid).unwrap();
    assert_eq!(one, several);
    let other_seed = price_vanilla_quadratic_rough_heston(
        &params,
        &grid,
        OptionStyle::Call,
        Positive::HUNDRED,
        paths(5_000),
        SEED + 1,
    )
    .unwrap();
    assert_ne!(one.price(), other_seed.price());
}

/// Halving the step, on the parameters of [`feedback_params`]: `alpha = 0.6`, `n = 5`, `x = 5`,
/// `a = 0.5`, `b = 0.1`, `c = 0.01`, `lambda = 1.5`, `eta = 1`, `Z^i_0 = -0.05`, `S_0 = K = 100`,
/// `T = 0.25`, an ATM call. `V_0 = 0.0497`. The two estimates draw different increments, so the
/// tolerance of `0.07` holds their Monte Carlo noise, four standard errors of a difference at
/// these `100_000` paths, and the discretization error between the steps. The averages of the
/// two prices over 40 seeds differ by about `0.006`.
#[test]
fn test_price_halving_the_step_stays_within_its_tolerance() {
    const TOLERANCE: Decimal = dec!(0.07);
    let params = feedback_params();
    let price = |steps| {
        let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), steps).unwrap();
        price_vanilla_quadratic_rough_heston(
            &params,
            &grid,
            OptionStyle::Call,
            Positive::HUNDRED,
            paths(100_000),
            SEED,
        )
        .unwrap()
        .price()
    };
    let (coarse, fine) = (price(100), price(200));
    assert!((coarse - fine).abs() < TOLERANCE, "{coarse} against {fine}");
}

/// A single path has no standard error, and that is reported.
#[test]
fn test_price_single_path_returns_typed_error() {
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 10).unwrap();
    let result = price_vanilla_quadratic_rough_heston(
        &feedback_params(),
        &grid,
        OptionStyle::Call,
        Positive::HUNDRED,
        NonZeroUsize::MIN,
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
fn test_exploding_state_returns_its_step_and_no_value() {
    let params = exploding_params();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(1), 200).unwrap();
    let on_one = price_on_threads(1, &params, &grid);
    let on_several = price_on_threads(4, &params, &grid);
    let step = match on_one {
        Err(QuadraticRoughHestonError::NonFinite { step, .. }) => step,
        other => panic!("{other:?}"),
    };
    assert!((1..=200).contains(&step), "{step}");
    // The first path that fails is the one reported, whatever the threads.
    assert!(matches!(
        on_several,
        Err(QuadraticRoughHestonError::NonFinite { step: other, .. }) if other == step
    ));
}

/// The state overflows a `Decimal` on the way to overflowing an `f64`.
#[test]
fn test_path_out_of_decimal_range_returns_typed_error() {
    let grid = QuadraticRoughHestonGrid::try_new(dec!(1), 200).unwrap();
    let result = simulate_quadratic_rough_heston_path(&exploding_params(), &grid, SEED);
    assert!(matches!(result, Err(QuadraticRoughHestonError::Decimal(_))));
}

/// A mean that does not fit a `Decimal`: the spot is finite and the path needs no record.
#[test]
fn test_price_out_of_decimal_range_returns_typed_error() {
    let params = QuadraticRoughHestonParams::try_new(
        Decimal::ZERO,
        Decimal::ZERO,
        dec!(1000000),
        dec!(1),
        dec!(1),
        kernel(),
        vec![Decimal::ZERO; 5],
        Positive::new_decimal(dec!(10000000000000000000000000000)).unwrap(),
    )
    .unwrap();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(1), 2).unwrap();
    let result = price_vanilla_quadratic_rough_heston(
        &params,
        &grid,
        OptionStyle::Call,
        Positive::ONE,
        paths(1_000),
        SEED,
    );
    assert!(matches!(result, Err(QuadraticRoughHestonError::Decimal(_))));
}

#[test]
fn test_unallocatable_runs_return_typed_errors() {
    let params = feedback_params();
    let huge_path = QuadraticRoughHestonGrid::try_new(dec!(1), usize::MAX / 2).unwrap();
    assert!(matches!(
        simulate_quadratic_rough_heston_path(&params, &huge_path, SEED),
        Err(QuadraticRoughHestonError::BufferTooLarge { .. })
    ));
    let no_room = QuadraticRoughHestonGrid::try_new(dec!(1), usize::MAX).unwrap();
    assert!(matches!(
        simulate_quadratic_rough_heston_path(&params, &no_room, SEED),
        Err(QuadraticRoughHestonError::BufferTooLarge { .. })
    ));
    let grid = QuadraticRoughHestonGrid::try_new(dec!(1), 1).unwrap();
    assert!(matches!(
        price_vanilla_quadratic_rough_heston(
            &params,
            &grid,
            OptionStyle::Call,
            Positive::ONE,
            paths(usize::MAX),
            SEED
        ),
        Err(QuadraticRoughHestonError::BufferTooLarge { .. })
    ));
}
