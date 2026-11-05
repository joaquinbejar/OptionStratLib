//! The core model with and without the wall clock (#944): the pure forms give
//! the same answer in every build.

use chrono::{DateTime, Duration, TimeZone, Utc};
use optionstratlib_core::model::leg::{
    FuturePosition, MarginType, PerpetualPosition, SpotPosition,
};
use optionstratlib_core::model::types::Action;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side, Trade, TradeStatus,
};
use optionstratlib_core::pos_or_panic;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 9, 30, 0).unwrap()
}

fn position(expiration_date: ExpirationDate) -> Position {
    Position::new(
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            expiration_date,
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        ),
        pos_or_panic!(5.0),
        at(),
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    )
}

#[test]
fn test_trade_new_at_records_the_given_timestamp() {
    let trade = Trade::new_at(
        uuid::Uuid::nil(),
        Action::Buy,
        Side::Long,
        OptionStyle::Call,
        Positive::ZERO,
        None,
        Positive::HUNDRED,
        at() + Duration::days(30),
        Positive::ONE,
        pos_or_panic!(5.0),
        Positive::HUNDRED,
        None,
        TradeStatus::Open,
        at(),
    )
    .unwrap();
    assert_eq!(Some(trade.timestamp), at().timestamp_nanos_opt());
}

#[test]
fn test_position_pure_forms_read_no_clock() {
    let dated = position(ExpirationDate::DateTime(at() + Duration::days(30)));
    assert_eq!(
        dated.days_to_expiration_at(at()).unwrap(),
        pos_or_panic!(30.0)
    );
    assert_eq!(
        dated.days_held_at(at() + Duration::days(3)).unwrap(),
        pos_or_panic!(3.0)
    );
    let trade = dated.trade_at(at()).unwrap();
    assert_eq!(trade.expiry, at() + Duration::days(30));
    assert_eq!(Some(trade.timestamp), at().timestamp_nanos_opt());
}

#[test]
fn test_position_trade_at_counts_a_days_expiration_from_the_given_time() {
    let trade = position(ExpirationDate::Days(pos_or_panic!(30.0)))
        .trade_at(at())
        .unwrap();
    assert_eq!(trade.expiry, at() + Duration::days(30));
}

#[test]
fn test_position_days_to_expiration_of_a_days_needs_no_clock() {
    let by_days = position(ExpirationDate::Days(pos_or_panic!(30.0)));
    assert_eq!(by_days.days_to_expiration().unwrap(), pos_or_panic!(30.0));
}

#[test]
fn test_position_blank_at_is_opened_at_the_given_date() {
    assert_eq!(Position::blank_at(at()).date, at());
}

#[test]
fn test_leg_constructors_with_an_explicit_date_read_no_clock() {
    let spot = SpotPosition::new(
        "TEST".to_string(),
        Positive::ONE,
        Positive::HUNDRED,
        Side::Long,
        at(),
        Positive::ZERO,
        Positive::ZERO,
    );
    assert_eq!(spot.date, at());
    let future = FuturePosition::new(
        "TEST".to_string(),
        Positive::ONE,
        Positive::HUNDRED,
        Side::Long,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        Positive::ONE,
        pos_or_panic!(10.0),
        pos_or_panic!(8.0),
        at(),
        Positive::ZERO,
    );
    assert_eq!(future.date, at());
    let perpetual = PerpetualPosition::new(
        "TEST".to_string(),
        Positive::ONE,
        Positive::HUNDRED,
        Side::Long,
        Positive::TWO,
        pos_or_panic!(50.0),
        MarginType::Isolated,
        Decimal::ZERO,
        at(),
        Positive::ZERO,
    );
    assert_eq!(perpetual.date, at());
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_core::error::{ClockError, PositionError, TradeError};
    use optionstratlib_core::model::TradeAble;
    use optionstratlib_core::model::resolve_expiration_date;
    use optionstratlib_core::utils::time::now;

    /// Fails if anything in this configuration turned the clock on.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    #[test]
    fn test_trade_new_is_a_typed_error() {
        let trade = Trade::new(
            uuid::Uuid::nil(),
            Action::Buy,
            Side::Long,
            OptionStyle::Call,
            Positive::ZERO,
            None,
            Positive::HUNDRED,
            at(),
            Positive::ONE,
            pos_or_panic!(5.0),
            Positive::HUNDRED,
            None,
            TradeStatus::Open,
        );
        assert!(matches!(
            trade,
            Err(TradeError::Clock(ClockError::Unavailable))
        ));
    }

    #[test]
    fn test_position_clock_forms_are_typed_errors() {
        let dated = position(ExpirationDate::DateTime(at()));
        assert!(matches!(dated.days_held(), Err(PositionError::Clock(_))));
        assert!(matches!(
            dated.days_to_expiration(),
            Err(PositionError::Clock(_))
        ));
        assert!(matches!(dated.trade(), Err(TradeError::Clock(_))));
    }

    #[test]
    fn test_the_reference_datetime_stands_in_for_the_clock() {
        let dated = position(ExpirationDate::DateTime(at() + Duration::days(30)));
        ExpirationDate::set_reference_datetime(Some(at()));
        let days = dated.days_to_expiration();
        let resolved = resolve_expiration_date(&ExpirationDate::Days(pos_or_panic!(1.0)));
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(days.unwrap(), pos_or_panic!(30.0));
        assert_eq!(resolved.unwrap(), at() + Duration::days(1));
    }

    #[test]
    fn test_relative_expiration_without_a_base_is_a_typed_error() {
        assert!(resolve_expiration_date(&ExpirationDate::Days(pos_or_panic!(1.0))).is_err());
    }
}
