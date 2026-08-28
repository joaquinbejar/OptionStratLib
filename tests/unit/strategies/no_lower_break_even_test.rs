/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 28/8/26
******************************************************************************/

//! Verifies that a strategy with no lower break-even survives its own public API.
//!
//! When the credit taken in — or the debit paid, on a long structure — exceeds
//! the put strike, the lower break-even sits at or below zero, where the
//! underlying cannot trade. It is reported as `Positive::ZERO`. Every consumer
//! of `break_even_points` has to treat that as "no lower break-even" rather
//! than as an ordinary price, because subtracting a real strike from it, or a
//! premium from it, is a `Positive` subtraction that would go negative.

use optionstratlib::ExpirationDate;
use optionstratlib::strategies::base::{BreakEvenable, Strategies};
use optionstratlib::strategies::{BearPutSpread, LongStrangle, ShortStrangle};
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;

#[test]
fn test_bear_put_spread_profit_area_with_no_lower_break_even() {
    // Debit far above the long strike: the lower break-even is floored to zero.
    let strategy = BearPutSpread::new(
        "TEST".to_string(),
        pos_or_panic!(5.0),
        pos_or_panic!(5.0),
        pos_or_panic!(4.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(50.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .expect("the spread constructs");

    let break_even = strategy
        .get_break_even_points()
        .expect("break-even points are computed");
    assert_eq!(break_even[0], Positive::ZERO, "no lower break-even");

    // The public API must answer rather than abort.
    assert!(strategy.get_profit_area().is_ok());
    assert!(strategy.get_profit_ratio().is_ok());
    assert!(strategy.get_best_range_to_show(Positive::ONE).is_ok());
}

#[test]
fn test_long_strangle_range_with_no_lower_break_even() {
    let strategy = LongStrangle::new(
        "TEST".to_string(),
        pos_or_panic!(5.0),
        pos_or_panic!(6.0),
        pos_or_panic!(5.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(3.0),
        pos_or_panic!(3.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .expect("the strangle constructs");

    let break_even = strategy
        .get_break_even_points()
        .expect("break-even points are computed");
    assert_eq!(break_even[0], Positive::ZERO, "no lower break-even");

    let range = strategy
        .get_best_range_to_show(Positive::ONE)
        .expect("the range is computed");
    assert!(
        !range.is_empty(),
        "a range with no lower break-even is still a range"
    );
    assert!(strategy.get_profit_area().is_ok());
}

#[test]
fn test_short_strangle_range_with_no_lower_break_even() {
    let strategy = ShortStrangle::new(
        "TEST".to_string(),
        pos_or_panic!(5.0),
        pos_or_panic!(6.0),
        pos_or_panic!(5.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(3.0),
        pos_or_panic!(3.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .expect("the strangle constructs");

    let break_even = strategy
        .get_break_even_points()
        .expect("break-even points are computed");
    assert_eq!(break_even[0], Positive::ZERO, "no lower break-even");

    let range = strategy
        .get_best_range_to_show(Positive::ONE)
        .expect("the range is computed");
    assert!(!range.is_empty());
    assert!(strategy.get_profit_area().is_ok());
}
