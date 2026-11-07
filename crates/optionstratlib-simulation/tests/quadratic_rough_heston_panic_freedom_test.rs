//! Property-based tests for panic freedom of the quadratic rough Heston simulation (#958).
//!
//! Every failure must come back as a `Result`, including for inputs that are extreme but
//! structurally valid, as in `ou_panic_freedom_test.rs`. The assertion is deliberately weak:
//! whatever comes back, it must come back.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_simulation::simulation::{
    QuadraticRoughHestonCalibrationConfig, QuadraticRoughHestonFreeParameters,
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonMonteCarloConfig,
    QuadraticRoughHestonParams, QuadraticRoughHestonQuote, QuadraticRoughHestonSurfacePoint,
    QuadraticRoughHestonVix, QuadraticRoughHestonVixPayoff, QuadraticRoughHestonVixQuote,
    calibrate_quadratic_rough_heston, calibrate_quadratic_rough_heston_joint,
    price_surface_quadratic_rough_heston, price_vanilla_quadratic_rough_heston,
    price_vix_quadratic_rough_heston, simulate_quadratic_rough_heston_path,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroU64;
use std::num::NonZeroUsize;

/// The smallest representable positive `Decimal`.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// `Decimal`s at the edges of the domain, on both signs.
fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.5)),
        Just(dec!(0.75)),
        Just(Decimal::ONE),
        Just(Decimal::ONE + TINY),
        Just(dec!(100)),
        Just(dec!(100000000000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(Positive::new_decimal(TINY).unwrap_or(Positive::ZERO)),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(Positive::MAX),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Each constructor returns for every extreme input.
    #[test]
    fn test_constructors_never_panic(
        alpha in extreme_decimal(),
        ratio in extreme_decimal(),
        horizon in extreme_decimal(),
        factors in prop_oneof![0usize..6, Just(1000usize), Just(usize::MAX)],
        steps in prop_oneof![0usize..6, Just(usize::MAX)],
        t in extreme_positive(),
    ) {
        if let Ok(kernel) = QuadraticRoughHestonKernel::try_new(alpha, factors, ratio) {
            let _ = kernel.value_at(t);
        }
        let _ = QuadraticRoughHestonGrid::try_new(horizon, steps);
    }

    /// A simulation and a price return for every extreme parameter of the model.
    #[test]
    fn test_simulation_and_price_never_panic(
        a in extreme_decimal(),
        b in extreme_decimal(),
        c in extreme_decimal(),
        lambda in extreme_decimal(),
        eta in extreme_decimal(),
        factor in extreme_decimal(),
        spot in extreme_positive(),
        strike in extreme_positive(),
        horizon in extreme_decimal(),
        seed in any::<u64>(),
    ) {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.75), 3, dec!(4));
        prop_assert!(kernel.is_ok());
        let Ok(kernel) = kernel else { return Ok(()); };
        let params = QuadraticRoughHestonParams::try_new(
            a, b, c, lambda, eta, kernel, vec![factor; 3], spot,
        );
        let grid = QuadraticRoughHestonGrid::try_new(horizon, 4);
        if let (Ok(params), Ok(grid)) = (params, grid) {
            let _ = simulate_quadratic_rough_heston_path(&params, &grid, seed);
            let _ = price_vanilla_quadratic_rough_heston(
                &params,
                &grid,
                OptionStyle::Put,
                strike,
                NonZeroUsize::new(4).unwrap_or(NonZeroUsize::MIN),
                seed,
            );
        }
    }

    /// The VIX map, its evaluation and a VIX price return for every extreme parameter of the
    /// model, window and state.
    #[test]
    fn test_vix_never_panics(
        a in extreme_decimal(),
        b in extreme_decimal(),
        c in extreme_decimal(),
        lambda in extreme_decimal(),
        eta in extreme_decimal(),
        factor in extreme_decimal(),
        state in extreme_decimal(),
        window in extreme_positive(),
        strike in extreme_positive(),
        horizon in extreme_decimal(),
        seed in any::<u64>(),
    ) {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.75), 3, dec!(4));
        prop_assert!(kernel.is_ok());
        let Ok(kernel) = kernel else { return Ok(()); };
        let params = QuadraticRoughHestonParams::try_new(
            a, b, c, lambda, eta, kernel, vec![factor; 3], Positive::HUNDRED,
        );
        let Ok(params) = params else { return Ok(()); };
        if let Ok(vix) = QuadraticRoughHestonVix::try_new(&params, window) {
            let _ = vix.squared_at(&[state; 3]);
            let _ = vix.at(&[state; 3]);
            let _ = vix.at(&[state; 2]);
        }
        let _ = QuadraticRoughHestonVix::thirty_days(&params);
        if let Ok(grid) = QuadraticRoughHestonGrid::try_new(horizon, 4) {
            for payoff in [
                QuadraticRoughHestonVixPayoff::Future,
                QuadraticRoughHestonVixPayoff::Vanilla { style: OptionStyle::Call, strike },
                QuadraticRoughHestonVixPayoff::Vanilla { style: OptionStyle::Put, strike },
            ] {
                let _ = price_vix_quadratic_rough_heston(
                    &params,
                    &grid,
                    window,
                    payoff,
                    NonZeroUsize::new(4).unwrap_or(NonZeroUsize::MIN),
                    seed,
                );
            }
        }
    }

    /// The surface pricer and the calibration return for every extreme parameter, quote and
    /// configuration, including a search of two iterations.
    #[test]
    fn test_calibration_never_panics(
        a in extreme_decimal(),
        b in extreme_decimal(),
        c in extreme_decimal(),
        lambda in extreme_decimal(),
        eta in extreme_decimal(),
        factor in extreme_decimal(),
        maturity in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_positive(),
        weight in extreme_positive(),
        max_step in extreme_positive(),
        tolerance in extreme_positive(),
        seed in any::<u64>(),
    ) {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.75), 2, dec!(4));
        prop_assert!(kernel.is_ok());
        let Ok(kernel) = kernel else { return Ok(()); };
        let params = QuadraticRoughHestonParams::try_new(
            a, b, c, lambda, eta, kernel, vec![factor; 2], Positive::HUNDRED,
        );
        let Ok(params) = params else { return Ok(()); };
        let monte_carlo = QuadraticRoughHestonMonteCarloConfig {
            paths: NonZeroUsize::new(4).unwrap_or(NonZeroUsize::MIN),
            max_step,
            seed,
        };
        let points = [
            QuadraticRoughHestonSurfacePoint { maturity, strike },
            QuadraticRoughHestonSurfacePoint { maturity: Positive::ONE, strike: Positive::HUNDRED },
        ];
        let _ = price_surface_quadratic_rough_heston(&params, &points, &monte_carlo);
        let quotes = [
            QuadraticRoughHestonQuote { maturity, strike, implied_volatility: volatility, weight },
            QuadraticRoughHestonQuote {
                maturity: Positive::ONE,
                strike: Positive::HUNDRED,
                implied_volatility: Positive::new(0.2).unwrap_or(Positive::ONE),
                weight: Positive::ONE,
            },
        ];
        let config = QuadraticRoughHestonCalibrationConfig {
            monte_carlo,
            max_iterations: NonZeroU64::new(2).unwrap_or(NonZeroU64::MIN),
            tolerance,
            initial_step: Positive::new(0.1).unwrap_or(Positive::ONE),
        };
        let _ = calibrate_quadratic_rough_heston(
            &quotes,
            &params,
            QuadraticRoughHestonFreeParameters::ALL,
            &config,
        );
        let vix_quotes = [
            QuadraticRoughHestonVixQuote {
                maturity,
                payoff: QuadraticRoughHestonVixPayoff::Future,
                price: volatility,
                weight,
            },
            QuadraticRoughHestonVixQuote {
                maturity: Positive::ONE,
                payoff: QuadraticRoughHestonVixPayoff::Vanilla { style: OptionStyle::Call, strike },
                price: Positive::ONE,
                weight: Positive::ONE,
            },
        ];
        let _ = calibrate_quadratic_rough_heston_joint(
            &quotes,
            &vix_quotes,
            &params,
            QuadraticRoughHestonFreeParameters::MODEL,
            &config,
        );
    }
}
