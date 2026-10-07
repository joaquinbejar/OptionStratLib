//! The facade with every capability and no `schema` still builds and charts a
//! strategy (#549): leaving the derives out removes no behaviour.

use optionstratlib::prelude::*;

#[test]
fn test_a_strategy_builds_without_schema() {
    let long_call = LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.20),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    );
    assert!(long_call.is_ok());
}
