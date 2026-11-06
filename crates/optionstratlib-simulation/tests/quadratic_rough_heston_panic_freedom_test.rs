//! Property-based tests for panic freedom of the quadratic rough Heston simulation (#958).
//!
//! Every failure must come back as a `Result`, including for inputs that are extreme but
//! structurally valid, as in `ou_panic_freedom_test.rs`. The assertion is deliberately weak:
//! whatever comes back, it must come back.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_simulation::simulation::{
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
    price_vanilla_quadratic_rough_heston, simulate_quadratic_rough_heston_path,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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
}
