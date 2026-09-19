//! Market-layer strike selection compiles and runs without touching the
//! strategies module (multi-crate roadmap M1-04, #501).
//!
//! This file deliberately imports only `optionstratlib::chains` and the core
//! model: once `optionstratlib-market` is extracted, it becomes the
//! market-only consumer fixture for chain selection.

use optionstratlib::ExpirationDate;
use optionstratlib::chains::{FindOptimalSide, OptionData};
use positive::{Positive, pos_or_panic, spos};
use rust_decimal_macros::dec;

fn option_at(strike: Positive) -> OptionData {
    OptionData::new(
        strike,
        spos!(5.0),
        spos!(5.5),
        spos!(4.5),
        spos!(5.0),
        pos_or_panic!(0.20),
        Some(dec!(0.5)),
        Some(dec!(-0.5)),
        Some(dec!(0.05)),
        spos!(1000.0),
        Some(5000),
        Some("TEST".to_string()),
        Some(ExpirationDate::Days(pos_or_panic!(30.0))),
        Some(Box::new(Positive::HUNDRED)),
        Some(dec!(0.05)),
        spos!(0.02),
        None,
        None,
    )
}

#[test]
fn test_find_optimal_side_is_reachable_from_chains() {
    let underlying = Positive::HUNDRED;
    let above = option_at(pos_or_panic!(110.0));
    let below = option_at(pos_or_panic!(90.0));

    assert!(above.is_valid_optimal_side(&underlying, &FindOptimalSide::Upper));
    assert!(!above.is_valid_optimal_side(&underlying, &FindOptimalSide::Lower));
    assert!(below.is_valid_optimal_side(&underlying, &FindOptimalSide::Lower));
    assert!(above.is_valid_optimal_side(&underlying, &FindOptimalSide::All));
    assert!(above.is_valid_optimal_side(
        &underlying,
        &FindOptimalSide::Range(pos_or_panic!(105.0), pos_or_panic!(115.0))
    ));
    assert!(!below.is_valid_optimal_side(
        &underlying,
        &FindOptimalSide::Range(pos_or_panic!(105.0), pos_or_panic!(115.0))
    ));
}

#[test]
fn test_center_is_rejected_at_the_market_layer() {
    // `Center` needs a concrete strategy to resolve the centre point, so the
    // market-layer filter rejects it instead of guessing.
    let underlying = Positive::HUNDRED;
    assert!(
        !option_at(Positive::HUNDRED).is_valid_optimal_side(&underlying, &FindOptimalSide::Center)
    );
}
