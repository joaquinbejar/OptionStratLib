//! Pricing with and without the wall clock (#944).
//!
//! A `Days` expiration needs no clock, so its figures are pinned here to the
//! decimal in every build: the build without `clock` runs the same assertions
//! as the default one. A `DateTime` needs a time to measure from.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::delta;
use optionstratlib_pricing::pricing::black_scholes_model::black_scholes;
use optionstratlib_pricing::volatility::{
    ImpliedVolatilitySurface, ImpliedVolatilitySurfaceConfig, OptionQuote, SviParameters, SviSlice,
};
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

/// A surface measures every expiry from its as-of time, so the same quotes give the same
/// answer whatever the build and whatever the reference datetime, and with no `cfg` here:
/// this test runs without the clock exactly as it runs with it.
#[test]
fn test_calibrated_surface_measures_from_as_of_in_every_build() {
    let as_of: DateTime<Utc> = Utc.with_ymd_and_hms(2030, 1, 1, 12, 0, 0).unwrap();
    let forward = Positive::HUNDRED;
    let known = SviSlice::try_new(
        SviParameters::try_new(dec!(0.02), dec!(0.1), dec!(-0.4), dec!(0.05), dec!(0.2)).unwrap(),
        forward,
        Positive::ONE,
    )
    .unwrap();
    let mut quotes = Vec::new();
    for strike in [70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 135.0] {
        let strike = pos_or_panic!(strike);
        let mut quoted = option(ExpirationDate::Days(pos_or_panic!(365.0)));
        quoted.strike_price = strike;
        quoted.risk_free_rate = dec!(0);
        quoted.implied_volatility = known.implied_volatility_at_strike(strike).unwrap();
        let price = Positive::new_decimal(black_scholes(&quoted).unwrap()).unwrap();
        quotes.push(OptionQuote {
            expiry: as_of + TimeDelta::try_days(365).unwrap(),
            strike,
            style: OptionStyle::Call,
            bid: price,
            ask: price,
            quoted_at: as_of,
        });
    }
    let calibrate = || {
        ImpliedVolatilitySurface::calibrate(
            &quotes,
            forward,
            dec!(0),
            Positive::ZERO,
            as_of,
            &ImpliedVolatilitySurfaceConfig::default(),
        )
        .unwrap()
    };

    let surface = calibrate();
    let expiry = surface.expiries().first().unwrap();
    assert_eq!(expiry.time_to_expiry(), Positive::ONE);
    assert!(surface.rejections().is_empty());
    let point = surface.point(dec!(0), Positive::ONE).unwrap();
    let miss =
        point.implied_volatility.to_dec() - known.implied_volatility(dec!(0)).unwrap().to_dec();
    assert!(miss.abs() < dec!(0.00000001), "{point:?}");
    assert_eq!(
        surface
            .age_at(as_of + TimeDelta::try_hours(2).unwrap())
            .unwrap(),
        TimeDelta::try_hours(2).unwrap()
    );
    let shocked = surface
        .shock(optionstratlib_pricing::volatility::SurfaceShock::Level {
            volatility: dec!(0.01),
        })
        .unwrap();
    assert_eq!(shocked.as_of(), as_of);

    // A reference datetime, which a `DateTime` expiration would read, changes nothing.
    ExpirationDate::set_reference_datetime(Some(as_of + TimeDelta::try_days(100).unwrap()));
    let referenced = calibrate();
    ExpirationDate::set_reference_datetime(None);
    assert_eq!(referenced, surface);
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use optionstratlib_core::error::{ClockError, OptionsError};
    use optionstratlib_core::model::ExpirationDateError;
    use optionstratlib_core::utils::time::now;
    use optionstratlib_pricing::error::PricingError;
    use optionstratlib_pricing::pricing::OptionPricing;

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

    /// The solver's first guess tolerates a missing time to expiry, so the failure
    /// has to come out of the solve: a `DateTime` with no time to measure from is an
    /// error, not an unsolved volatility.
    #[test]
    fn test_implied_volatility_of_a_datetime_without_a_reference_is_an_error() {
        let expiry = Utc.with_ymd_and_hms(2030, 1, 31, 0, 0, 0).unwrap();
        let solved =
            option(ExpirationDate::DateTime(expiry)).calculate_implied_volatility(dec!(5.0));
        assert!(solved.is_err());
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
