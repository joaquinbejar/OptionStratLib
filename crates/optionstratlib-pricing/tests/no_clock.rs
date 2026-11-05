//! Pricing with and without the wall clock (#944).
//!
//! A `Days` expiration needs no clock, so its figures are pinned here to the
//! decimal in every build: the build without `clock` runs the same assertions
//! as the default one. A `DateTime` needs a time to measure from.

use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::delta;
use optionstratlib_pricing::pricing::black_scholes_model::black_scholes;
use rust_decimal_macros::dec;

fn option(expiration_date: ExpirationDate) -> Options {
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
    )
}

#[test]
fn test_days_expiration_prices_identically_in_every_build() {
    let option = option(ExpirationDate::Days(pos_or_panic!(30.0)));
    assert_eq!(
        black_scholes(&option).unwrap(),
        dec!(2.493376819403773537057817006)
    );
    assert_eq!(delta(&option).unwrap(), dec!(0.539963545623085));
    assert_eq!(
        option.time_to_expiration().unwrap(),
        pos_or_panic!(30.0 / 365.0)
    );
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use optionstratlib_core::error::{ClockError, OptionsError};
    use optionstratlib_core::model::ExpirationDateError;
    use optionstratlib_core::utils::time::now;
    use optionstratlib_pricing::error::PricingError;

    /// Fails if anything in this configuration turned the clock on, a
    /// dev-dependency included: features unify, and the run would prove nothing.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    #[test]
    fn test_datetime_expiration_without_a_reference_is_a_typed_error() {
        let expiry = Utc.with_ymd_and_hms(2030, 1, 31, 0, 0, 0).unwrap();
        let error = black_scholes(&option(ExpirationDate::DateTime(expiry))).unwrap_err();
        assert!(matches!(
            error,
            PricingError::Options(OptionsError::ExpirationDate(
                ExpirationDateError::InvalidDateTime(_)
            ))
        ));
    }

    #[test]
    fn test_datetime_expiration_measures_from_the_reference_datetime() {
        let base = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
        ExpirationDate::set_reference_datetime(Some(base));
        let dated = option(ExpirationDate::DateTime(base + Duration::days(30)));
        let by_days = option(ExpirationDate::Days(pos_or_panic!(30.0)));
        let years = dated.time_to_expiration().unwrap();
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(years, by_days.time_to_expiration().unwrap());
    }
}
