//! Property-based tests for panic freedom in the leg Greeks, the
//! delta-neutral sizing, the lattice and return-path allocations and the
//! finite-difference Greeks of the pricing crate (#788).
//!
//! Each driver below reached an aborting operator before #788:
//!
//! - `LegGreeks` for futures and perpetuals multiplied `pub` quantities,
//!   prices and funding rates with the raw `Decimal` and `Positive`
//!   operators;
//! - `calculate_delta_neutral_sizes` subtracted the first size from the total
//!   with `Positive - Positive`, which aborts when the quotient rounds above
//!   the total, and checked the result with raw products;
//! - `generate_binomial_tree` sized its lattice with `no_steps + 1`, which
//!   overflows at `usize::MAX`, and `simulate_returns` reserved `length`
//!   slots with `Vec::with_capacity`, which aborts with `capacity overflow`;
//! - the finite-difference Greeks bumped the inputs with `+=` / `-=`.
//!
//! The assertion is deliberately weak: whatever comes back, it must come
//! back.

use optionstratlib_core::model::leg::{FuturePosition, Leg, PerpetualPosition, SpotPosition};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::utils::create_sample_option_simplest;
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::utils::deterministic_rng;
use optionstratlib_pricing::greeks::numerical::{
    numerical_delta, numerical_gamma, numerical_rho, numerical_vega,
};
use optionstratlib_pricing::greeks::{LegGreeks, calculate_delta_neutral_sizes};
use optionstratlib_pricing::pricing::{
    BinomialPricingParams, generate_binomial_tree, simulate_returns,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

/// The smallest representable `Decimal`.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// A `Positive` from a `Decimal` literal that is non-negative by construction.
fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Quantities, prices and sizes across the whole `Positive` range.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.2))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(1000000000000000))),
        Just(Positive::MAX),
    ]
}

/// Deltas, rates and funding rates over the signed `Decimal` range.
fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.5)),
        Just(dec!(-0.5)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

fn extreme_side() -> impl Strategy<Value = Side> {
    prop_oneof![Just(Side::Long), Just(Side::Short)]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The Greeks of every linear leg.
    #[test]
    fn test_leg_greeks_never_panic(
        quantity in extreme_positive(),
        price in extreme_positive(),
        contract_size in extreme_positive(),
        funding_rate in extreme_decimal(),
        days in extreme_positive(),
        side in extreme_side(),
    ) {
        let spot = SpotPosition::new(
            "SPOT".to_string(),
            quantity,
            price,
            side,
            chrono::Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        );
        let future = FuturePosition::new(
            "FUT".to_string(),
            quantity,
            price,
            side,
            ExpirationDate::Days(days),
            contract_size,
            Positive::ONE,
            Positive::ONE,
            chrono::Utc::now(),
            Positive::ZERO,
        );
        let perpetual = PerpetualPosition {
            quantity,
            entry_price: price,
            funding_rate,
            side,
            ..PerpetualPosition::default()
        };
        for leg in [Leg::spot(spot), Leg::future(future), Leg::perpetual(perpetual)] {
            let _ = leg.delta();
            let _ = leg.gamma();
            let _ = leg.theta();
            let _ = leg.vega();
            let _ = leg.rho();
        }
    }

    /// Delta-neutral sizing over deltas and totals at both ends of the
    /// range, where the quotient can round above the total.
    #[test]
    fn test_delta_neutral_sizes_never_panic(
        delta1 in extreme_decimal(),
        delta2 in extreme_decimal(),
        total in extreme_positive(),
    ) {
        let _ = calculate_delta_neutral_sizes(delta1, delta2, total);
    }

    /// Lattices and return paths too large to allocate, next to small ones.
    #[test]
    fn test_unallocatable_sizes_never_panic(
        steps in prop_oneof![Just(1usize), Just(4usize), Just(usize::MAX)],
        length in prop_oneof![Just(0usize), Just(3usize), Just(usize::MAX)],
        volatility in prop_oneof![Just(Positive::ZERO), Just(pos(dec!(0.2)))],
        seed in any::<u64>(),
    ) {
        let no_steps = NonZeroUsize::new(steps).unwrap_or(NonZeroUsize::MIN);
        let params = BinomialPricingParams {
            asset: Positive::HUNDRED,
            volatility: pos(dec!(0.2)),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };
        let _ = generate_binomial_tree(&params);
        let mut rng = deterministic_rng(seed);
        let _ = simulate_returns(dec!(0.05), volatility, length, dec!(0.004), &mut rng);
    }

    /// The finite-difference Greeks with each bumped input at the ends of
    /// its range.
    #[test]
    fn test_numerical_greeks_never_panic(
        spot in extreme_positive(),
        volatility in extreme_positive(),
        rate in extreme_decimal(),
    ) {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.underlying_price = spot;
        option.implied_volatility = volatility;
        option.risk_free_rate = rate;
        let _ = numerical_delta(&option);
        let _ = numerical_gamma(&option);
        let _ = numerical_vega(&option);
        let _ = numerical_rho(&option);
    }
}
