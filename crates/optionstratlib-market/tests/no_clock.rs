//! Option series and positions with and without the wall clock (#944).

use chrono::{DateTime, Duration, TimeZone, Utc};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::series::OptionSeries;

fn base() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
}

fn chain(symbol: &str) -> OptionChain {
    OptionChain::new(
        symbol,
        Positive::HUNDRED,
        "2030-01-31".to_string(),
        None,
        None,
    )
}

fn series(chains: Vec<(ExpirationDate, &str)>) -> OptionSeries {
    let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);
    for (expiration, symbol) in chains {
        series.chains.insert(expiration, chain(symbol));
    }
    series
}

/// The chain found from `base`: the pure form, so the answer is the same with and
/// without the clock.
fn odte(chains: Vec<(ExpirationDate, &str)>) -> Option<String> {
    series(chains).odte_at(base()).map(|chain| chain.symbol)
}

/// `expiration_date` 0.6 orders a `DateTime` after every `Days`, so a plain
/// `<=` against `Days(1)` would never hold for one.
#[test]
fn test_odte_datetime_within_a_day_is_found() {
    let expiration = ExpirationDate::DateTime(base() + Duration::hours(6));
    assert_eq!(odte(vec![(expiration, "NEAR")]), Some("NEAR".to_string()));
}

#[test]
fn test_odte_datetime_beyond_a_day_is_not_found() {
    let expiration = ExpirationDate::DateTime(base() + Duration::days(2));
    assert_eq!(odte(vec![(expiration, "FAR")]), None);
}

/// The first key is the `Days`, as `Ord` puts it first; the earliest chain in
/// time is the `DateTime`.
#[test]
fn test_odte_compares_mixed_variants_in_time() {
    let near = ExpirationDate::DateTime(base() + Duration::hours(6));
    let far = ExpirationDate::Days(pos_or_panic!(30.0));
    assert_eq!(
        odte(vec![(far, "FAR"), (near, "NEAR")]),
        Some("NEAR".to_string())
    );
    let not_near = ExpirationDate::DateTime(base() + Duration::days(10));
    assert_eq!(odte(vec![(far, "FAR"), (not_near, "MID")]), None);
}

/// A `Days` of half a day, ahead of a `DateTime`, is still the earliest.
#[test]
fn test_odte_days_earlier_than_a_datetime_is_found() {
    let near = ExpirationDate::Days(pos_or_panic!(0.5));
    let far = ExpirationDate::DateTime(base() + Duration::days(10));
    assert_eq!(
        odte(vec![(far, "FAR"), (near, "NEAR")]),
        Some("NEAR".to_string())
    );
}

/// The expiration is 40 hours from `base`, so a later base brings it within a day.
#[test]
fn test_odte_at_depends_only_on_the_base() {
    let expiration = ExpirationDate::DateTime(base() + Duration::hours(40));
    let series = series(vec![(expiration, "TEST")]);
    assert!(series.odte_at(base()).is_none());
    assert!(series.odte_at(base() + Duration::hours(20)).is_some());
}

#[test]
fn test_odte_at_of_days_counts_from_any_base() {
    let near = series(vec![(ExpirationDate::Days(Positive::ONE), "TEST")]);
    let far = series(vec![(ExpirationDate::Days(Positive::TWO), "TEST")]);
    assert!(near.odte_at(base()).is_some());
    assert!(far.odte_at(base()).is_none());
}

/// `odte` is `odte_at` from the reference datetime or the current time.
#[cfg(feature = "clock")]
#[test]
fn test_odte_delegates_to_odte_at() {
    let far_future = base() + Duration::days(60_000);
    let expiration = ExpirationDate::DateTime(far_future + Duration::hours(6));
    let series = series(vec![(expiration, "TEST")]);
    ExpirationDate::set_reference_datetime(Some(far_future));
    let found = series.odte();
    ExpirationDate::set_reference_datetime(None);
    assert!(found.is_some());
    // From the wall clock it is far from a day away.
    assert!(series.odte().is_none());
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_core::error::{ClockError, PositionError};
    use optionstratlib_core::model::{OptionStyle, OptionType, Options, Position, Side};
    use optionstratlib_core::utils::time::now;
    use optionstratlib_market::chains::{OptionData, UpdateFromOptionData};
    use optionstratlib_market::error::ChainError;
    use rust_decimal::Decimal;

    /// Fails if anything in this configuration turned the clock on, the
    /// `io`, `synthetic` and `async` dependencies included.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    /// Refreshing a position re-stamps its open date, and there is no time to give it.
    #[test]
    fn test_update_from_option_data_is_a_typed_error() {
        let mut position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                Decimal::ZERO,
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos_or_panic!(5.0),
            base(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );
        let updated = position.update_from_option_data(&OptionData::default());
        assert!(matches!(updated, Err(PositionError::Clock(_))));
    }

    #[test]
    fn test_chain_error_keeps_the_clock_error_as_such() {
        assert!(matches!(
            ChainError::from(ClockError::Unavailable),
            ChainError::Clock(ClockError::Unavailable)
        ));
    }

    /// `get_expiration_dates` has an error channel, so a `DateTime` is an error
    /// rather than a missing entry.
    #[test]
    fn test_expiration_dates_of_a_datetime_without_a_base_is_an_error() {
        let dated = series(vec![(ExpirationDate::DateTime(base()), "TEST")]);
        assert!(dated.get_expiration_dates().is_err());
    }

    #[test]
    fn test_expiration_dates_of_days_need_no_base() {
        let days = series(vec![(ExpirationDate::Days(Positive::TWO), "TEST")])
            .get_expiration_dates()
            .unwrap();
        assert_eq!(days, vec![Positive::TWO]);
    }
}
