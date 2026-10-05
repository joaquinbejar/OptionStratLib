//! The imports a core consumer writes, compiled and exercised (#518).
//!
//! `optionstratlib-core` has no prelude by decision; these paths are its
//! intended entry points. A test here breaks when one of them moves.

use optionstratlib_core::model::leg::{Leg, LegAble, SpotPosition};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side, Trade,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::{Len, TimeFrame};
use rust_decimal_macros::dec;

fn sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

#[test]
fn test_domain_types_from_model_root() {
    let position = Position::new(
        sample_option(),
        pos_or_panic!(2.5),
        chrono::Utc::now(),
        Positive::ONE,
        Positive::ONE,
        None,
        None,
    );
    assert_eq!(position.option.side, Side::Long);
    let _trade_type_is_reachable: Option<Trade> = None;
}

#[test]
fn test_leg_types_from_model_leg() {
    let leg = Leg::Spot(SpotPosition::long(
        "XYZ".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(100.0),
    ));
    assert_eq!(leg.get_quantity(), Positive::HUNDRED);
}

struct ThreePoints;

impl Len for ThreePoints {
    fn len(&self) -> usize {
        3
    }
}

#[test]
fn test_utils_from_utils_root() {
    assert_eq!(ThreePoints.len(), 3);
    assert!(!ThreePoints.is_empty());
    assert_ne!(TimeFrame::Day, TimeFrame::Week);
}
