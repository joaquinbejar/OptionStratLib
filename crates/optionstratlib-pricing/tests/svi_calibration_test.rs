//! Calibration of the raw SVI slice through the public API (#945).
//!
//! Every recovery case generates quotes from a known slice, calibrates, and
//! compares the parameters and the smile with the truth, wings included. The
//! tolerances below are the ones the module docs declare.

use optionstratlib_core::model::Positive;
use optionstratlib_pricing::error::SviError;
use optionstratlib_pricing::volatility::{
    SviCalibrationConfig, SviFit, SviObservation, SviParameters, SviSlice, calibrate_svi,
};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Declared recovery tolerance of `a`, `b`, `rho`, `m` and `sigma`.
const PARAMETER_TOLERANCE: Decimal = dec!(0.000001);

/// Declared recovery tolerance of the implied volatility, on and beyond the
/// observed strikes.
const IMPLIED_VOLATILITY_TOLERANCE: Decimal = dec!(0.00000001);

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap()
}

fn slice_of(truth: [Decimal; 5], forward: Positive, expiry: Positive) -> SviSlice {
    let [a, b, rho, m, sigma] = truth;
    SviSlice::try_new(
        SviParameters::try_new(a, b, rho, m, sigma).unwrap(),
        forward,
        expiry,
    )
    .unwrap()
}

/// A strike at the given log moneyness.
fn strike_at(forward: Positive, k: Decimal) -> Positive {
    pos(forward.to_dec() * k.exp())
}

/// Quotes of `slice` at the given log moneyness values, all with weight one.
fn quotes(slice: &SviSlice, log_moneyness: &[Decimal]) -> Vec<SviObservation> {
    log_moneyness
        .iter()
        .map(|&k| {
            let strike = strike_at(slice.forward(), k);
            SviObservation {
                strike,
                implied_volatility: slice.implied_volatility_at_strike(strike).unwrap(),
                weight: Positive::ONE,
            }
        })
        .collect()
}

fn calibrate(observations: &[SviObservation], slice: &SviSlice) -> SviFit {
    calibrate_svi(
        observations,
        slice.forward(),
        slice.time_to_expiry(),
        &SviCalibrationConfig::default(),
    )
    .unwrap()
}

/// Largest absolute difference between the smiles of two slices over a grid
/// that runs to `reach` in log moneyness.
fn smile_distance(left: &SviSlice, right: &SviSlice, reach: Decimal) -> Decimal {
    let mut largest = Decimal::ZERO;
    for node in -100..=100 {
        let k = reach * Decimal::from(node) / dec!(100);
        let difference = left.implied_volatility(k).unwrap().to_dec()
            - right.implied_volatility(k).unwrap().to_dec();
        largest = largest.max(difference.abs());
    }
    largest
}

fn assert_recovers(name: &str, slice: &SviSlice, log_moneyness: &[Decimal]) {
    let fit = calibrate(&quotes(slice, log_moneyness), slice);
    let truth = slice.parameters();
    let found = fit.slice().parameters();
    let reach = log_moneyness
        .iter()
        .fold(Decimal::ZERO, |largest, k| largest.max(k.abs()));
    let errors = [
        ("a", (found.a() - truth.a()).abs()),
        ("b", (found.b() - truth.b()).abs()),
        ("rho", (found.rho() - truth.rho()).abs()),
        ("m", (found.m() - truth.m()).abs()),
        ("sigma", (found.sigma() - truth.sigma()).abs()),
    ];
    for (parameter, error) in errors {
        assert!(
            error <= PARAMETER_TOLERANCE,
            "{name}: {parameter} is off by {error}: found {found:?}, truth {truth:?}"
        );
    }
    let distance = smile_distance(slice, fit.slice(), reach);
    assert!(
        distance <= IMPLIED_VOLATILITY_TOLERANCE,
        "{name}: the smile is off by {distance} within |k| <= {reach}"
    );
    let wing = smile_distance(slice, fit.slice(), reach * dec!(1.5));
    assert!(
        wing <= IMPLIED_VOLATILITY_TOLERANCE,
        "{name}: the smile is off by {wing} beyond the observed wings"
    );
}

/// `count + 1` evenly spaced values from `low` to `high`.
fn grid(low: Decimal, high: Decimal, count: i32) -> Vec<Decimal> {
    (0..=count)
        .map(|node| low + (high - low) * Decimal::from(node) / Decimal::from(count))
        .collect()
}

#[test]
fn test_calibrate_svi_equity_skew_recovers_parameters_and_wings() {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    assert_recovers("equity skew", &slice, &grid(dec!(-0.6), dec!(0.4), 20));
}

#[test]
fn test_calibrate_svi_negative_level_recovers_parameters_and_wings() {
    // a < 0 is admissible as long as a + b * sigma * sqrt(1 - rho^2) >= 0.
    let slice = slice_of(
        [
            dec!(-0.0410),
            dec!(0.1331),
            dec!(0.3060),
            dec!(0.3586),
            dec!(0.4153),
        ],
        pos(dec!(1)),
        Positive::ONE,
    );
    assert_recovers("negative level", &slice, &grid(dec!(-1.5), dec!(1.5), 30));
}

#[test]
fn test_calibrate_svi_call_skew_short_expiry_recovers_parameters_and_wings() {
    let slice = slice_of(
        [dec!(0.001), dec!(0.05), dec!(0.5), dec!(-0.03), dec!(0.06)],
        pos(dec!(5000)),
        pos(dec!(0.0833)),
    );
    assert_recovers(
        "call skew, short expiry",
        &slice,
        &grid(dec!(-0.25), dec!(0.3), 16),
    );
}

#[test]
fn test_calibrate_svi_few_points_in_the_wings_recovers_parameters_and_wings() {
    // Dense around the money, two points in each wing, unevenly spaced.
    let slice = slice_of(
        [dec!(0.03), dec!(0.18), dec!(-0.55), dec!(-0.04), dec!(0.2)],
        Positive::HUNDRED,
        pos(dec!(0.5)),
    );
    assert_recovers(
        "few wing points",
        &slice,
        &[
            dec!(-0.9),
            dec!(-0.55),
            dec!(-0.12),
            dec!(-0.08),
            dec!(-0.04),
            dec!(0),
            dec!(0.04),
            dec!(0.08),
            dec!(0.12),
            dec!(0.5),
            dec!(0.7),
        ],
    );
}

#[test]
fn test_calibrate_svi_observations_on_one_side_of_the_forward_recovers_parameters() {
    let slice = slice_of(
        [dec!(0.015), dec!(0.2), dec!(0.25), dec!(0.15), dec!(0.1)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    assert_recovers(
        "strikes above the forward",
        &slice,
        &grid(dec!(0.01), dec!(0.6), 12),
    );
}

#[test]
fn test_calibrate_svi_zero_minimum_recovers_parameters_on_the_bound() {
    // The minimum of w is exactly zero: the optimum lies on the bound, and the
    // active set has to reach it rather than clamp near it.
    let b = dec!(0.2);
    let rho = dec!(-0.5);
    let sigma = dec!(0.2);
    let a = -(b * sigma * dec!(0.75).sqrt().unwrap());
    let slice = slice_of(
        [a, b, rho, dec!(0.1), sigma],
        Positive::HUNDRED,
        Positive::ONE,
    );
    assert_recovers("zero minimum", &slice, &grid(dec!(-0.8), dec!(0.8), 20));
}

#[test]
fn test_calibrate_svi_exactly_five_points_interpolates_the_quotes() {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let fit = calibrate(&quotes(&slice, &grid(dec!(-0.5), dec!(0.5), 4)), &slice);
    assert!(fit.rmse() < pos(dec!(0.000001)), "rmse {}", fit.rmse());
}

#[test]
fn test_calibrate_svi_forward_only_relabels_the_strikes() {
    // The kernel sees log moneyness, so the forward changes nothing else.
    let truth = [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)];
    let log_moneyness = grid(dec!(-0.6), dec!(0.4), 20);
    for forward in [dec!(0.01), dec!(10000)] {
        let slice = slice_of(truth, pos(forward), Positive::ONE);
        assert_recovers("forward", &slice, &log_moneyness);
    }
}

/// Calibrates a slice rescaled by `level` in total variance and `width` in log
/// moneyness, and compares each parameter with its rescaled truth, relative to
/// its own scale: `a -> level * a`, `b -> level * b / width`, `m -> width * m`,
/// `sigma -> width * sigma`, and `rho` unchanged.
fn assert_rescaled_recovery(name: &str, level: Decimal, width: Decimal, expiry: Decimal) {
    let [a, b, rho, m, sigma] = [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)];
    let scaled = [a * level, b * level / width, rho, m * width, sigma * width];
    let slice = slice_of(scaled, Positive::HUNDRED, pos(expiry));
    let log_moneyness: Vec<Decimal> = grid(dec!(-0.6), dec!(0.4), 20)
        .iter()
        .map(|k| k * width)
        .collect();
    let fit = calibrate(&quotes(&slice, &log_moneyness), &slice);
    let found = fit.slice().parameters();
    let checks = [
        ("a", found.a(), scaled[0], level),
        ("b", found.b(), scaled[1], level / width),
        ("rho", found.rho(), scaled[2], dec!(1)),
        ("m", found.m(), scaled[3], width),
        ("sigma", found.sigma(), scaled[4], width),
    ];
    for (parameter, value, truth, scale) in checks {
        assert!(
            (value - truth).abs() <= PARAMETER_TOLERANCE * scale,
            "{name}: {parameter} is {value}, expected {truth} at scale {scale}"
        );
    }
}

#[test]
fn test_calibrate_svi_rescaled_slices_recover_the_rescaled_parameters() {
    // Level of volatility: the implied volatility moves by the root of `level`.
    assert_rescaled_recovery("low volatility", dec!(0.0001), dec!(1), dec!(1));
    assert_rescaled_recovery("high volatility", dec!(10000), dec!(1), dec!(1));
    // Maturity: the same implied volatility over another expiry.
    assert_rescaled_recovery("one day", dec!(0.0027), dec!(1), dec!(0.0027));
    assert_rescaled_recovery("ten years", dec!(10), dec!(1), dec!(10));
    // Width of the strike range.
    assert_rescaled_recovery("narrow strikes", dec!(1), dec!(0.1), dec!(1));
    assert_rescaled_recovery("wide strikes", dec!(1), dec!(5), dec!(1));
}

#[test]
fn test_calibrate_svi_zero_minimum_near_the_rho_limit_stays_feasible() {
    // Rounding `rho` to Decimal moves sqrt(1 - rho^2) about 1 / (1 - rho^2)
    // times as much, so these land just below the bound on `a`.
    let mut runs = 0;
    for rho in [dec!(-0.9999), dec!(-0.99996), dec!(0.9999)] {
        for b in [dec!(0.07), dec!(0.2), dec!(0.45)] {
            for sigma in [dec!(0.04), dec!(0.1), dec!(0.25), dec!(0.45)] {
                for m in [dec!(-0.15), dec!(0.03), dec!(0.12)] {
                    let root = (dec!(1) - rho * rho).sqrt().unwrap();
                    let slice = slice_of(
                        [-(b * sigma * root), b, rho, m, sigma],
                        Positive::HUNDRED,
                        Positive::ONE,
                    );
                    let mut observations = quotes(&slice, &grid(dec!(-0.6), dec!(0.54), 19));
                    for (index, observation) in observations.iter_mut().enumerate() {
                        let step = Decimal::from(index * 7 % 5) - dec!(2);
                        let bumped = observation.implied_volatility.to_dec() + step * dec!(0.0001);
                        observation.implied_volatility = pos(bumped.max(dec!(0.0005)));
                    }
                    let result = calibrate_svi(
                        &observations,
                        slice.forward(),
                        slice.time_to_expiry(),
                        &SviCalibrationConfig::default(),
                    );
                    assert!(
                        result.is_ok(),
                        "rho {rho} b {b} sigma {sigma} m {m}: {result:?}"
                    );
                    runs += 1;
                }
            }
        }
    }
    assert_eq!(runs, 108);
}

#[test]
fn test_calibrate_svi_residuals_follow_the_input_order_and_match_the_slice() {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let mut observations = quotes(&slice, &grid(dec!(-0.6), dec!(0.4), 10));
    observations.reverse();
    let fit = calibrate(&observations, &slice);
    assert_eq!(fit.residuals().len(), observations.len());
    assert!(fit.iterations() > 0);
    for (residual, observation) in fit.residuals().iter().zip(&observations) {
        assert_eq!(residual.strike, observation.strike);
        assert_eq!(
            residual.observed_implied_volatility,
            observation.implied_volatility
        );
        assert_eq!(
            residual.log_moneyness,
            fit.slice().log_moneyness(residual.strike).unwrap()
        );
        assert_eq!(
            residual.fitted_implied_volatility,
            fit.slice()
                .implied_volatility_at_strike(residual.strike)
                .unwrap()
        );
        assert_eq!(
            residual.residual,
            residual.fitted_implied_volatility.to_dec() - observation.implied_volatility.to_dec()
        );
    }
}

/// A smile with a bumped observation, and a weight on that observation.
fn bumped_quotes(weight: Decimal) -> (SviSlice, Vec<SviObservation>, usize) {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.4), dec!(0.02), dec!(0.2)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let mut observations = quotes(&slice, &grid(dec!(-0.5), dec!(0.5), 12));
    let index = 3;
    let bumped = observations[index].implied_volatility.to_dec() + dec!(0.01);
    observations[index].implied_volatility = pos(bumped);
    observations[index].weight = pos(weight);
    (slice, observations, index)
}

#[test]
fn test_calibrate_svi_heavily_weighted_point_is_fitted_closer() {
    let residual_at = |weight: Decimal| {
        let (slice, observations, index) = bumped_quotes(weight);
        calibrate(&observations, &slice).residuals()[index]
            .residual
            .abs()
    };
    let plain = residual_at(dec!(1));
    let heavy = residual_at(dec!(1000));
    assert!(
        heavy * dec!(10) < plain,
        "weight 1 leaves {plain}, weight 1000 leaves {heavy}"
    );
}

#[test]
fn test_calibrate_svi_zero_weight_point_does_not_enter_the_fit_and_has_a_residual() {
    let (slice, observations, index) = bumped_quotes(dec!(0));
    let fit = calibrate(&observations, &slice);
    assert!(
        (fit.residuals()[index].residual + dec!(0.01)).abs() < dec!(0.000001),
        "{}",
        fit.residuals()[index].residual
    );
    assert!(fit.rmse() < pos(dec!(0.000001)), "rmse {}", fit.rmse());
}

#[test]
fn test_calibrate_svi_rmse_is_weighted_in_implied_volatility() {
    let (slice, observations, _) = bumped_quotes(dec!(3));
    let fit = calibrate(&observations, &slice);
    let (mut squares, mut total) = (Decimal::ZERO, Decimal::ZERO);
    for (residual, observation) in fit.residuals().iter().zip(&observations) {
        squares += observation.weight.to_dec() * residual.residual * residual.residual;
        total += observation.weight.to_dec();
    }
    let expected = (squares / total).sqrt().unwrap();
    assert!((fit.rmse().to_dec() - expected).abs() < dec!(1e-12));
}

#[test]
fn test_calibrate_svi_identical_input_gives_identical_output() {
    let (slice, observations, _) = bumped_quotes(dec!(2));
    let first = calibrate(&observations, &slice);
    let second = calibrate(&observations, &slice);
    assert_eq!(first, second);
}

#[test]
fn test_calibrate_svi_flat_input_comes_out_flat_from_the_fit() {
    let forward = Positive::HUNDRED;
    let observations: Vec<SviObservation> = grid(dec!(-0.5), dec!(0.5), 12)
        .iter()
        .map(|&k| SviObservation {
            strike: strike_at(forward, k),
            implied_volatility: pos(dec!(0.2)),
            weight: Positive::ONE,
        })
        .collect();
    let fit = calibrate_svi(
        &observations,
        forward,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    )
    .unwrap();
    assert!(fit.rmse() < pos(dec!(0.00000001)), "rmse {}", fit.rmse());
    for node in -30..=30 {
        let k = Decimal::from(node) * dec!(0.1);
        let iv = fit.slice().implied_volatility(k).unwrap().to_dec();
        assert!((iv - dec!(0.2)).abs() < dec!(0.000001), "iv {iv} at {k}");
    }
}

#[test]
fn test_calibrate_svi_input_flat_within_the_tolerance_comes_out_flat() {
    // A frown small enough that total variance stays within 0.1% of its mean.
    let observations = quotes_from_function(|k, _| dec!(0.2) - dec!(0.0005) * k * k);
    let fit = calibrate_svi(
        &observations,
        Positive::HUNDRED,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    )
    .unwrap();
    assert!(fit.rmse() < pos(dec!(0.0001)), "rmse {}", fit.rmse());
}

/// Thirteen quotes in `|k| <= 0.5`, from `implied_volatility(k, node)`.
fn quotes_from_function(
    implied_volatility: impl Fn(Decimal, i32) -> Decimal,
) -> Vec<SviObservation> {
    (0..=12)
        .map(|node| {
            let k = dec!(-0.5) + Decimal::from(node) / dec!(12);
            SviObservation {
                strike: strike_at(Positive::HUNDRED, k),
                implied_volatility: pos(implied_volatility(k, node)),
                weight: Positive::ONE,
            }
        })
        .collect()
}

/// A raw SVI slice is convex in `k`, so each of these has no better slice than
/// a flat one, and each is within the default limit of two volatility points.
fn assert_degenerate_under_the_default_limit(
    name: &str,
    implied_volatility: impl Fn(Decimal, i32) -> Decimal,
) {
    let observations = quotes_from_function(implied_volatility);
    let result = calibrate_svi(
        &observations,
        Positive::HUNDRED,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    );
    match result {
        Err(SviError::DegenerateFit {
            rmse,
            observed_dispersion,
            fitted_dispersion,
        }) => {
            assert!(rmse < pos(dec!(0.02)), "{name}: rmse {rmse}");
            assert!(
                observed_dispersion > pos(dec!(0.001)),
                "{name}: observed {observed_dispersion}"
            );
            assert!(
                fitted_dispersion.to_dec() <= observed_dispersion.to_dec() * dec!(0.1),
                "{name}: fitted {fitted_dispersion}, observed {observed_dispersion}"
            );
        }
        other => panic!("{name}: expected DegenerateFit, got {other:?}"),
    }
}

#[test]
fn test_calibrate_svi_frown_is_a_degenerate_fit_not_a_flat_smile() {
    assert_degenerate_under_the_default_limit("frown", |k, _| dec!(0.20) - dec!(0.2) * k * k);
}

#[test]
fn test_calibrate_svi_low_level_frown_is_a_degenerate_fit_not_a_flat_smile() {
    // The absolute limit of two volatility points is above the whole smile.
    assert_degenerate_under_the_default_limit("low level frown", |k, _| {
        dec!(0.015) - dec!(0.02) * k * k
    });
}

#[test]
fn test_calibrate_svi_low_amplitude_zigzag_is_a_degenerate_fit_not_a_flat_smile() {
    assert_degenerate_under_the_default_limit("zigzag", |_, node| {
        if node % 2 == 0 {
            dec!(0.19)
        } else {
            dec!(0.21)
        }
    });
}

#[test]
fn test_calibrate_svi_negligibly_curved_fit_is_a_degenerate_fit() {
    // Here the best slice is not exactly flat: the gate reads what the fit
    // does over the observed strikes, not whether `b` is zero.
    let shape = |_: Decimal, node: i32| dec!(0.2) + dec!(0.01) * Decimal::from(node % 3);
    assert_degenerate_under_the_default_limit("sawtooth", shape);
    let result = calibrate_svi(
        &quotes_from_function(shape),
        Positive::HUNDRED,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    );
    assert!(
        matches!(
            result,
            Err(SviError::DegenerateFit { fitted_dispersion, .. }) if !fitted_dispersion.is_zero()
        ),
        "{result:?}"
    );
}

#[test]
fn test_calibrate_svi_loose_limit_does_not_excuse_a_degenerate_fit() {
    let loose = SviCalibrationConfig {
        max_rmse: Positive::TEN,
        ..SviCalibrationConfig::default()
    };
    let result = calibrate_svi(
        &zigzag(dec!(0.1), dec!(0.6)),
        Positive::HUNDRED,
        Positive::ONE,
        &loose,
    );
    assert!(
        matches!(result, Err(SviError::DegenerateFit { .. })),
        "{result:?}"
    );
}

#[test]
fn test_calibrate_svi_skewed_input_does_not_come_back_flat() {
    let slice = slice_of(
        [dec!(0.01), dec!(0.2), dec!(-0.9), dec!(0.05), dec!(0.1)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let fit = calibrate(&quotes(&slice, &grid(dec!(-0.5), dec!(0.5), 12)), &slice);
    let parameters = fit.slice().parameters();
    assert!(parameters.b() > dec!(0.1), "{parameters:?}");
    assert!(parameters.rho() < dec!(-0.5), "{parameters:?}");
    let low = fit.slice().implied_volatility(dec!(-0.5)).unwrap().to_dec();
    let high = fit.slice().implied_volatility(dec!(0.5)).unwrap().to_dec();
    assert!(low > high + dec!(0.1), "low {low} high {high}");
}

/// Implied volatility alternating between two levels, which no SVI slice can
/// follow. The least bad slice is close to flat.
fn zigzag(low: Decimal, high: Decimal) -> Vec<SviObservation> {
    (0..=12)
        .map(|node| SviObservation {
            strike: strike_at(
                Positive::HUNDRED,
                dec!(-0.5) + Decimal::from(node) * dec!(0.08),
            ),
            implied_volatility: pos(if node % 2 == 0 { low } else { high }),
            weight: Positive::ONE,
        })
        .collect()
}

#[test]
fn test_calibrate_svi_unfittable_input_is_an_error_naming_rmse_and_limit() {
    let result = calibrate_svi(
        &zigzag(dec!(0.1), dec!(0.6)),
        Positive::HUNDRED,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    );
    match result {
        Err(error @ SviError::RmseAboveLimit { rmse, limit }) => {
            assert!(rmse > limit);
            assert_eq!(limit.to_dec(), dec!(0.02));
            let message = error.to_string();
            assert!(message.contains(&rmse.to_string()), "{message}");
            assert!(message.contains("0.02"), "{message}");
        }
        other => panic!("expected RmseAboveLimit, got {other:?}"),
    }
}

#[test]
fn test_calibrate_svi_max_rmse_limit_is_what_decides() {
    // A smile with one quote far below it: the fit is not flat, and is poor.
    let slice = slice_of(
        [dec!(0.02), dec!(0.2), dec!(-0.4), dec!(0.02), dec!(0.2)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let mut observations = quotes(&slice, &grid(dec!(-0.5), dec!(0.5), 12));
    observations[6].implied_volatility = pos(dec!(0.02));

    let strict = calibrate_svi(
        &observations,
        Positive::HUNDRED,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    );
    assert!(
        matches!(strict, Err(SviError::RmseAboveLimit { .. })),
        "{strict:?}"
    );

    let loose = SviCalibrationConfig {
        max_rmse: Positive::TEN,
        ..SviCalibrationConfig::default()
    };
    let fit = calibrate_svi(&observations, Positive::HUNDRED, Positive::ONE, &loose).unwrap();
    assert!(fit.rmse() > pos(dec!(0.02)), "rmse {}", fit.rmse());
    assert!(fit.rmse() < Positive::TEN);
}

#[test]
fn test_calibrate_svi_tolerance_above_its_bound_is_a_typed_error() {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let observations = quotes(&slice, &grid(dec!(-0.6), dec!(0.4), 20));
    let with_tolerance = |tolerance: Decimal| {
        calibrate_svi(
            &observations,
            slice.forward(),
            slice.time_to_expiry(),
            &SviCalibrationConfig {
                tolerance: pos(tolerance),
                ..SviCalibrationConfig::default()
            },
        )
    };
    for tolerance in [dec!(0.00100001), dec!(1), Decimal::MAX] {
        assert!(
            matches!(
                with_tolerance(tolerance),
                Err(SviError::InvalidConfig {
                    parameter: "tolerance",
                    value,
                    ..
                }) if value == tolerance
            ),
            "tolerance {tolerance}"
        );
    }
    // The bound itself is valid and still gives a usable fit.
    let fit = with_tolerance(dec!(0.001)).unwrap();
    assert!(fit.rmse() < pos(dec!(0.00001)), "rmse {}", fit.rmse());
}

#[test]
fn test_calibrate_svi_finite_result_out_of_range_is_a_decimal_error() {
    // Total variance of 1e40 is a finite f64 and no Decimal.
    let observations: Vec<SviObservation> = (0..7)
        .map(|node| {
            observation(
                dec!(70) + Decimal::from(node) * dec!(10),
                dec!(100000000000000000000),
                dec!(1),
            )
        })
        .collect();
    let result = calibrate_default(&observations);
    assert!(matches!(result, Err(SviError::Decimal(_))), "{result:?}");
}

#[test]
fn test_calibrate_svi_iteration_limit_is_a_no_convergence_error() {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let config = SviCalibrationConfig {
        max_iterations: std::num::NonZeroU64::MIN,
        ..SviCalibrationConfig::default()
    };
    let result = calibrate_svi(
        &quotes(&slice, &grid(dec!(-0.6), dec!(0.4), 20)),
        slice.forward(),
        slice.time_to_expiry(),
        &config,
    );
    assert!(matches!(
        result,
        Err(SviError::NoConvergence {
            iterations: 20,
            limit: 1
        })
    ));
}

fn observation(strike: Decimal, implied_volatility: Decimal, weight: Decimal) -> SviObservation {
    SviObservation {
        strike: pos(strike),
        implied_volatility: pos(implied_volatility),
        weight: pos(weight),
    }
}

#[test]
fn test_calibrate_svi_too_few_points_is_a_typed_error() {
    for count in 0..5 {
        let observations: Vec<SviObservation> = (0..count)
            .map(|node| observation(dec!(90) + Decimal::from(node) * dec!(5), dec!(0.2), dec!(1)))
            .collect();
        let result = calibrate_svi(
            &observations,
            Positive::HUNDRED,
            Positive::ONE,
            &SviCalibrationConfig::default(),
        );
        assert!(
            matches!(
                result,
                Err(SviError::TooFewObservations { got, required: 5 }) if got == count
            ),
            "{result:?}"
        );
    }
}

fn six_strikes(weight: Decimal) -> Vec<SviObservation> {
    [80, 90, 100, 110, 120, 130]
        .iter()
        .map(|&strike| observation(Decimal::from(strike), dec!(0.2), weight))
        .collect()
}

fn calibrate_default(observations: &[SviObservation]) -> Result<SviFit, SviError> {
    calibrate_svi(
        observations,
        Positive::HUNDRED,
        Positive::ONE,
        &SviCalibrationConfig::default(),
    )
}

#[test]
fn test_calibrate_svi_all_weights_zero_is_a_typed_error() {
    assert!(matches!(
        calibrate_default(&six_strikes(dec!(0))),
        Err(SviError::NonPositiveTotalWeight)
    ));
}

#[test]
fn test_calibrate_svi_too_few_weighted_points_is_a_typed_error() {
    let mut observations = six_strikes(dec!(1));
    observations[0].weight = Positive::ZERO;
    observations[1].weight = Positive::ZERO;
    assert!(matches!(
        calibrate_default(&observations),
        Err(SviError::TooFewObservations {
            got: 4,
            required: 5
        })
    ));
}

#[test]
fn test_calibrate_svi_duplicate_strikes_are_a_typed_error() {
    let mut observations = six_strikes(dec!(1));
    observations[1].strike = observations[0].strike;
    observations[2].strike = observations[0].strike;
    assert!(matches!(
        calibrate_default(&observations),
        Err(SviError::InsufficientDistinctStrikes {
            distinct: 4,
            required: 5
        })
    ));
}

#[test]
fn test_calibrate_svi_repeated_quote_with_enough_strikes_is_accepted() {
    let slice = slice_of(
        [dec!(0.02), dec!(0.12), dec!(-0.7), dec!(0.02), dec!(0.15)],
        Positive::HUNDRED,
        Positive::ONE,
    );
    let mut observations = quotes(&slice, &grid(dec!(-0.6), dec!(0.4), 10));
    let repeated = observations[4];
    observations.push(repeated);
    let fit = calibrate(&observations, &slice);
    assert!(fit.rmse() < pos(dec!(0.000001)), "rmse {}", fit.rmse());
}

#[test]
fn test_calibrate_svi_zero_inputs_are_typed_errors() {
    let observations = six_strikes(dec!(1));
    let config = SviCalibrationConfig::default();
    assert!(matches!(
        calibrate_svi(&observations, Positive::ZERO, Positive::ONE, &config),
        Err(SviError::ZeroForward)
    ));
    assert!(matches!(
        calibrate_svi(&observations, Positive::ONE, Positive::ZERO, &config),
        Err(SviError::ZeroTimeToExpiry)
    ));

    let mut zero_strike = observations.clone();
    zero_strike[2].strike = Positive::ZERO;
    assert!(matches!(
        calibrate_default(&zero_strike),
        Err(SviError::InvalidObservation { index: 2, .. })
    ));

    let mut zero_volatility = observations;
    zero_volatility[3].implied_volatility = Positive::ZERO;
    assert!(matches!(
        calibrate_default(&zero_volatility),
        Err(SviError::InvalidObservation { index: 3, .. })
    ));
}

#[test]
fn test_svi_parameters_invalid_values_are_typed_errors() {
    assert!(matches!(
        SviParameters::try_new(dec!(0.1), dec!(-1), dec!(0), dec!(0), dec!(0.1)),
        Err(SviError::InvalidParameter { parameter: "b", .. })
    ));
    assert!(matches!(
        SviParameters::try_new(dec!(0.1), dec!(1), dec!(1), dec!(0), dec!(0.1)),
        Err(SviError::InvalidParameter {
            parameter: "rho",
            ..
        })
    ));
    assert!(matches!(
        SviParameters::try_new(dec!(0.1), dec!(1), dec!(0), dec!(0), dec!(0)),
        Err(SviError::InvalidParameter {
            parameter: "sigma",
            ..
        })
    ));
    assert!(matches!(
        SviParameters::try_new(dec!(-1), dec!(1), dec!(0), dec!(0), dec!(0.1)),
        Err(SviError::NegativeMinimumVariance { .. })
    ));
}
