//! Building strategies with and without the wall clock (#944).

use chrono::{DateTime, Duration, TimeZone, Utc};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::Positionable;
use optionstratlib_strategies::strategies::{BullCallSpread, IronCondor, LongCall, LongStraddle};
use rust_decimal_macros::dec;

fn date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 9, 30, 0).unwrap()
}

fn spread(date: DateTime<Utc>) -> Result<BullCallSpread, StrategyError> {
    BullCallSpread::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(8.0),
        pos_or_panic!(3.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        date,
    )
}

#[test]
fn test_new_at_opens_every_leg_at_the_given_date() {
    let strategy = spread(date()).unwrap();
    assert_eq!(strategy.long_call.date, date());
    assert_eq!(strategy.short_call.date, date());
}

#[test]
fn test_new_at_is_deterministic() {
    let later = date() + Duration::days(1);
    let (first, second) = (spread(later).unwrap(), spread(later).unwrap());
    assert_eq!(first.break_even_points, second.break_even_points);
    assert_eq!(first.long_call, second.long_call);
}

#[test]
fn test_new_at_builds_single_leg_multi_leg_and_condor_strategies() {
    let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
    let long_call = LongCall::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        expiration,
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        Positive::ZERO,
        Positive::ZERO,
        date(),
    )
    .unwrap();
    let legs = long_call.get_positions().unwrap();
    assert_eq!(legs.len(), 1);
    assert_eq!(legs[0].date, date());

    let straddle = LongStraddle::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        Positive::HUNDRED,
        expiration,
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(5.0),
        pos_or_panic!(5.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        date(),
    )
    .unwrap();
    assert_eq!(straddle.long_put.date, date());

    let condor = IronCondor::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
        pos_or_panic!(90.0),
        pos_or_panic!(120.0),
        pos_or_panic!(80.0),
        expiration,
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(2.0),
        pos_or_panic!(2.0),
        pos_or_panic!(1.0),
        pos_or_panic!(1.0),
        Positive::ZERO,
        Positive::ZERO,
        date(),
    )
    .unwrap();
    assert_eq!(condor.long_put.date, date());
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::utils::time::now;
    use optionstratlib_strategies::strategies::StrategyConstructor;

    /// Fails if anything in this configuration turned the clock on.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    #[test]
    fn test_new_without_a_clock_is_a_typed_error() {
        let result = LongCall::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(5.0),
            Positive::ZERO,
            Positive::ZERO,
        );
        let error = result.err().unwrap().to_string();
        assert!(error.contains("wall clock is unavailable"), "{error}");
    }

    /// Rebuilding from positions keeps the date each leg already carries.
    #[test]
    fn test_get_strategy_keeps_the_date_of_the_legs() {
        let strategy = spread(date()).unwrap();
        let positions: Vec<_> = strategy
            .get_positions()
            .unwrap()
            .into_iter()
            .cloned()
            .collect();
        let rebuilt = BullCallSpread::get_strategy(&positions).unwrap();
        assert_eq!(rebuilt.long_call.date, date());
        assert_eq!(rebuilt.short_call.date, date());
    }
}
