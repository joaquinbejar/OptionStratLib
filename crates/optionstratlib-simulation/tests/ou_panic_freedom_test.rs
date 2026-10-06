//! Property-based tests for panic freedom of the Ornstein-Uhlenbeck process.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! The parameters are `Positive` values at the edges of the domain, the same
//! extremes the volatility panic-freedom suite in `optionstratlib-pricing`
//! drives the volatility kernels with. The assertion is deliberately weak:
//! whatever comes back, it must come back.

use optionstratlib_core::model::Positive;
use optionstratlib_simulation::simulation::ou::generate_ou_process;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// `Positive` values at the edges of the domain. `Positive` is `>= 0`, so
/// `Positive::ZERO` is a legitimate time step to hand to the process.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(Positive::new_decimal(TINY).unwrap_or(Positive::ZERO)),
        Just(Positive::new_decimal(dec!(0.2)).unwrap_or(Positive::ZERO)),
        Just(Positive::new_decimal(dec!(100000000000000)).unwrap_or(Positive::ZERO)),
        Just(Positive::MAX),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The Ornstein-Uhlenbeck path returns for every extreme parameter,
    /// including a zero time step and a level at `Positive::MAX`.
    #[test]
    fn test_generate_ou_process_never_panics(
        level in extreme_positive(),
        dt in extreme_positive(),
        steps in 0usize..6,
    ) {
        let _ = generate_ou_process(level, level, level, level, dt, steps);
        let _ = generate_ou_process(
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            dt,
            steps,
        );
    }
}
