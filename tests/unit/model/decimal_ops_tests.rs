use optionstratlib::model::decimal::DecimalStats;
use optionstratlib::{pos, Positive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[test]
fn decimal_add_and_addassign_with_positive_ref() {
    let p: Positive = pos!(2.5);

    // Decimal + &Positive
    let d = dec!(10);
    let sum = d + &p;
    assert_eq!(sum, dec!(12.5));

    // AddAssign<&Positive>
    let mut d2 = dec!(3.5);
    d2 += &p;
    assert_eq!(d2, dec!(6.0));
}

#[test]
fn decimal_mulassign_with_positive_ref() {
    let p: Positive = pos!(4);
    let mut d = dec!(2.5);
    d *= &p; // MulAssign<&Positive>
    assert_eq!(d, dec!(10.0));
}

#[test]
fn decimal_partial_eq_with_positive() {
    let p = pos!(7.25);
    let d = Decimal::from(p.clone());
    assert!(d == p);
}

#[test]
fn decimalstats_empty_vec_returns_zeroes() {
    let v: Vec<Decimal> = vec![];
    assert_eq!(v.mean(), Decimal::ZERO);
    assert_eq!(v.std_dev(), Decimal::ZERO);
}
