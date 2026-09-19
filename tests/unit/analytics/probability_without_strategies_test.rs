//! The neutral probability kernels are usable through
//! `optionstratlib::analytics` alone (multi-crate roadmap M1-16, #513): no
//! strategy type, trait or module is imported here. Once
//! `optionstratlib-analytics` is extracted this file becomes the
//! analytics-only fixture for price probabilities.

use optionstratlib::ExpirationDate;
use optionstratlib::analytics::probability::{
    PriceTrend, VolatilityAdjustment, calculate_price_probability,
    calculate_single_point_probability,
};
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;

fn adjustment() -> VolatilityAdjustment {
    VolatilityAdjustment {
        base_volatility: pos_or_panic!(0.20),
        std_dev_adjustment: Positive::ZERO,
    }
}

#[test]
fn test_single_point_probabilities_sum_to_one() {
    let result = calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(105.0),
        Some(adjustment()),
        None,
        &ExpirationDate::Days(pos_or_panic!(30.0)),
        Some(dec!(0.05)),
    );
    let (below, above) = match result {
        Ok(pair) => pair,
        Err(e) => panic!("kernel must evaluate an ordinary input: {e}"),
    };
    let total = below.to_dec() + above.to_dec();
    assert!(
        (total - dec!(1.0)).abs() < dec!(1e-9),
        "below + above = {total}"
    );
    assert!(
        below > above,
        "a 5% out-of-the-money target is more likely to stay below"
    );
}

#[test]
fn test_range_probability_is_bounded_by_its_tails() {
    let result = calculate_price_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(95.0),
        &pos_or_panic!(105.0),
        Some(adjustment()),
        Some(PriceTrend {
            drift_rate: 0.0,
            confidence: 0.5,
        }),
        &ExpirationDate::Days(pos_or_panic!(30.0)),
        Some(dec!(0.05)),
    );
    let (below, in_range, above) = match result {
        Ok(triple) => triple,
        Err(e) => panic!("kernel must evaluate an ordinary input: {e}"),
    };
    let total = below.to_dec() + in_range.to_dec() + above.to_dec();
    assert!(
        (total - dec!(1.0)).abs() < dec!(1e-9),
        "tails + range = {total}"
    );
    assert!(in_range > Positive::ZERO);

    // The interval mass is the difference of the two single-point
    // probabilities of staying below each bound.
    let expiry = ExpirationDate::Days(pos_or_panic!(30.0));
    let trend = || PriceTrend {
        drift_rate: 0.0,
        confidence: 0.5,
    };
    let below_lower = calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(95.0),
        Some(adjustment()),
        Some(trend()),
        &expiry,
        Some(dec!(0.05)),
    )
    .map(|(below, _)| below);
    let below_upper = calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(105.0),
        Some(adjustment()),
        Some(trend()),
        &expiry,
        Some(dec!(0.05)),
    )
    .map(|(below, _)| below);
    match (below_lower, below_upper) {
        (Ok(lower), Ok(upper)) => {
            let expected = upper.to_dec() - lower.to_dec();
            assert!(
                (in_range.to_dec() - expected).abs() < dec!(1e-9),
                "range mass {in_range} vs upper - lower = {expected}"
            );
            assert!((below.to_dec() - lower.to_dec()).abs() < dec!(1e-9));
        }
        other => panic!("single-point kernels must evaluate: {other:?}"),
    }
}

#[test]
fn test_inverted_range_is_reported_not_floored() {
    let result = calculate_price_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(105.0),
        &pos_or_panic!(95.0),
        Some(adjustment()),
        None,
        &ExpirationDate::Days(pos_or_panic!(30.0)),
        None,
    );
    assert!(
        result.is_err(),
        "upper below lower must be an error, got {result:?}"
    );
}
