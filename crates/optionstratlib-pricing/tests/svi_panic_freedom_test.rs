//! Property-based tests for panic freedom of the SVI slice and its calibration.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it, so every failure
//! must come back as a typed `SviError`, including for inputs that are extreme
//! but structurally valid.
//!
//! The inputs reach the failures on three axes: the `Decimal` range (values at
//! `Decimal::MAX` and at the smallest representable scale, whose ratios and
//! squares leave the range), the shape of the sample (empty, too short, all
//! weights zero, repeated strikes) and the limits of the configuration (one
//! iteration, a tolerance of zero or above its bound, an RMSE limit of zero). A
//! calibration that succeeds must also keep its promises: the error it reports
//! is within the limit, every observation has a residual, and a flat result only
//! comes from quotes that are flat within the limit.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::decimal_to_f64;
use optionstratlib_pricing::volatility::{
    SviCalibrationConfig, SviObservation, SviParameters, SviSlice, calibrate_svi,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroU64;

/// The smallest representable positive `Decimal`.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

fn positive(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// `Positive` values at the edges of the domain, and ordinary ones between.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(positive(TINY)),
        Just(positive(dec!(0.0001))),
        Just(positive(dec!(0.2))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(positive(dec!(100000000000000))),
        Just(positive(Decimal::MAX)),
    ]
}

/// Strikes that are mostly ordinary, so that a calibration gets past the input
/// checks and reaches the kernel, and at times extreme.
fn strike() -> impl Strategy<Value = Positive> {
    prop_oneof![
        3 => (1_u32..4000).prop_map(|cents| positive(Decimal::from(cents) * dec!(0.1))),
        1 => extreme_positive(),
    ]
}

/// Implied volatilities, mostly ordinary and at times extreme.
fn implied_volatility() -> impl Strategy<Value = Positive> {
    prop_oneof![
        3 => (1_u32..400).prop_map(|points| positive(Decimal::from(points) * dec!(0.01))),
        1 => extreme_positive(),
    ]
}

/// Weights, mostly one and at times extreme.
fn weight() -> impl Strategy<Value = Positive> {
    prop_oneof![
        3 => Just(Positive::ONE),
        1 => extreme_positive(),
    ]
}

fn observation() -> impl Strategy<Value = SviObservation> {
    (strike(), implied_volatility(), weight()).prop_map(|(strike, implied_volatility, weight)| {
        SviObservation {
            strike,
            implied_volatility,
            weight,
        }
    })
}

fn config() -> impl Strategy<Value = SviCalibrationConfig> {
    (
        1_u64..40,
        prop_oneof![
            3 => Just(Positive::ZERO),
            3 => Just(positive(dec!(0.000000001))),
            3 => Just(positive(dec!(0.001))),
            1 => Just(Positive::ONE),
            1 => Just(positive(Decimal::MAX)),
        ],
        prop_oneof![
            Just(Positive::ZERO),
            Just(positive(dec!(0.02))),
            Just(positive(Decimal::MAX)),
        ],
    )
        .prop_map(
            |(iterations, tolerance, max_relative_rmse)| SviCalibrationConfig {
                max_iterations: NonZeroU64::new(iterations).unwrap_or(NonZeroU64::MIN),
                tolerance,
                max_relative_rmse,
            },
        )
}

fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.2)),
        Just(dec!(-0.5)),
        Just(dec!(0.9999999999999999999999999999)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

/// The mean and the standard deviation of `series`, with equal weights.
fn mean_and_deviation(series: &[f64]) -> (f64, f64) {
    let mean = series.iter().sum::<f64>() / series.len() as f64;
    let variance = series
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .sum::<f64>()
        / series.len() as f64;
    (mean, variance.sqrt())
}

proptest! {
    #[test]
    fn calibrate_svi_flat_result_only_comes_from_quotes_flat_within_the_limit(
        volatilities in prop::collection::vec(1_u32..80, 7..16),
    ) {
        let observations: Vec<SviObservation> = volatilities
            .iter()
            .enumerate()
            .map(|(index, &points)| SviObservation {
                strike: positive(dec!(60) + Decimal::from(index) * dec!(7)),
                implied_volatility: positive(Decimal::from(points) * dec!(0.01)),
                weight: Positive::ONE,
            })
            .collect();
        let config = SviCalibrationConfig::default();
        if let Ok(fit) = calibrate_svi(&observations, Positive::HUNDRED, Positive::ONE, &config) {
            let limit = decimal_to_f64(config.max_relative_rmse.to_dec()).unwrap();
            let relative = decimal_to_f64(fit.relative_rmse().to_dec()).unwrap();
            prop_assert!(relative <= limit, "relative rmse {relative}");

            let observed: Vec<f64> = observations
                .iter()
                .map(|observation| {
                    decimal_to_f64(observation.implied_volatility.to_dec()).unwrap()
                })
                .collect();
            let fitted: Vec<f64> = fit
                .residuals()
                .iter()
                .map(|residual| {
                    decimal_to_f64(residual.fitted_implied_volatility.to_dec()).unwrap()
                })
                .collect();
            let (mean, deviation) = mean_and_deviation(&observed);
            let (_, fitted_deviation) = mean_and_deviation(&fitted);
            if fitted_deviation <= 1e-9 {
                prop_assert!(
                    deviation / mean <= limit * (1.0 + 1e-9),
                    "flat fit of quotes {deviation} around {mean}"
                );
            }
        }
    }

    #[test]
    fn calibrate_svi_does_not_panic_on_extreme_input(
        observations in prop::collection::vec(observation(), 0..14),
        forward in extreme_positive(),
        time_to_expiry in extreme_positive(),
        config in config(),
    ) {
        if let Ok(fit) = calibrate_svi(&observations, forward, time_to_expiry, &config) {
            prop_assert!(fit.relative_rmse() <= config.max_relative_rmse);
            prop_assert_eq!(fit.residuals().len(), observations.len());
        }
    }

    #[test]
    fn calibrate_svi_with_ordinary_scale_and_extreme_quotes_does_not_panic(
        observations in prop::collection::vec(observation(), 5..14),
        config in config(),
    ) {
        if let Ok(fit) = calibrate_svi(&observations, Positive::HUNDRED, Positive::ONE, &config) {
            prop_assert!(fit.relative_rmse() <= config.max_relative_rmse);
            prop_assert_eq!(fit.residuals().len(), observations.len());
        }
    }

    #[test]
    fn svi_slice_does_not_panic_on_extreme_parameters(
        a in extreme_decimal(),
        b in extreme_decimal(),
        rho in extreme_decimal(),
        m in extreme_decimal(),
        sigma in extreme_decimal(),
        k in extreme_decimal(),
        strike in extreme_positive(),
        forward in extreme_positive(),
        time_to_expiry in extreme_positive(),
    ) {
        if let Ok(parameters) = SviParameters::try_new(a, b, rho, m, sigma) {
            let _ = parameters.total_variance(k);
            if let Ok(slice) = SviSlice::try_new(parameters, forward, time_to_expiry) {
                let _ = slice.implied_volatility(k);
                let _ = slice.log_moneyness(strike);
                let _ = slice.total_variance_at_strike(strike);
                let _ = slice.implied_volatility_at_strike(strike);
            }
        }
    }
}
