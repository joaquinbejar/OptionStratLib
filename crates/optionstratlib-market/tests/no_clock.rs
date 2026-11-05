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

/// A series with these chains, compared from `base` through the reference
/// datetime, so the answer is the same with and without the clock.
fn odte(chains: Vec<(ExpirationDate, &str)>) -> Option<String> {
    let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);
    for (expiration, symbol) in chains {
        series.chains.insert(expiration, chain(symbol));
    }
    ExpirationDate::set_reference_datetime(Some(base()));
    let found = series.odte();
    ExpirationDate::set_reference_datetime(None);
    found.map(|chain| chain.symbol)
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

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::utils::time::now;

    /// Fails if anything in this configuration turned the clock on, the
    /// `io`, `synthetic` and `async` dependencies included.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    fn series(expiration: ExpirationDate) -> OptionSeries {
        let mut series = OptionSeries::new("TEST".to_string(), Positive::HUNDRED);
        series.chains.insert(expiration, chain("TEST"));
        series
    }

    #[test]
    fn test_odte_of_days_needs_no_base() {
        assert!(series(ExpirationDate::Days(Positive::ONE)).odte().is_some());
        assert!(series(ExpirationDate::Days(Positive::TWO)).odte().is_none());
    }

    /// With no base a `DateTime` cannot be placed in time: no answer, rather
    /// than a wrong one.
    #[test]
    fn test_odte_of_a_datetime_without_a_base_is_none() {
        assert!(series(ExpirationDate::DateTime(base())).odte().is_none());
    }

    #[test]
    fn test_expiration_dates_of_days_need_no_base() {
        let days = series(ExpirationDate::Days(Positive::TWO))
            .get_expiration_dates()
            .unwrap();
        assert_eq!(days, vec![Positive::TWO]);
    }
}
