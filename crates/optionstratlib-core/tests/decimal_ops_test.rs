//! `Decimal` arithmetic with `Positive` operands, and `DecimalStats` on an
//! empty sample.
//!
//! This file sat under the facade's `tests/unit/model/` without being
//! declared in any `mod.rs`, so it never compiled. It is revived here with
//! its owner (#519).

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::DecimalStats;
use optionstratlib_core::pos_or_panic;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn decimal_add_and_addassign_with_positive_ref() {
    let p: Positive = pos_or_panic!(2.5);

    // Decimal + &Positive, called through the trait so the by-reference
    // impl is the one exercised.
    let d = dec!(10);
    let sum = <Decimal as std::ops::Add<&Positive>>::add(d, &p);
    assert_eq!(sum, dec!(12.5));

    // AddAssign<&Positive>
    let mut d2 = dec!(3.5);
    d2 += &p;
    assert_eq!(d2, dec!(6.0));
}

#[test]
fn decimal_mulassign_with_positive_ref() {
    let p: Positive = pos_or_panic!(4.0);
    let mut d = dec!(2.5);
    d *= &p;
    assert_eq!(d, dec!(10.0));
}

#[test]
fn decimal_partial_eq_with_positive() {
    let p = pos_or_panic!(7.25);
    let d = Decimal::from(p);
    assert!(d == p);
}

#[test]
fn decimalstats_empty_vec_returns_zeroes() {
    let v: Vec<Decimal> = vec![];
    assert!(matches!(v.mean(), Ok(mean) if mean == Decimal::ZERO));
    assert!(matches!(v.std_dev(), Ok(std_dev) if std_dev == Decimal::ZERO));
}
