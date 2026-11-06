//! Option series and positions with and without the wall clock (#944).

use chrono::{DateTime, Duration, TimeZone, Utc};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::spos;
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::utils::RandomPositionsParams;
#[cfg(not(feature = "clock"))]
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib_market::series::OptionSeries;
#[cfg(not(feature = "clock"))]
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn base() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
}

#[cfg(not(feature = "clock"))]
fn chain_params(chain_size: usize, expiration: ExpirationDate) -> OptionChainBuildParams {
    OptionChainBuildParams::new(
        "TEST".to_string(),
        None,
        chain_size,
        spos!(1.0),
        dec!(-0.3),
        Decimal::ZERO,
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(expiration),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("TEST".to_string()),
        ),
        pos_or_panic!(0.17),
    )
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

fn random_params() -> RandomPositionsParams {
    RandomPositionsParams::new(
        None,
        None,
        Some(2),
        None,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        Positive::ONE,
        dec!(0.05),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    )
}

/// The dated form opens every position at the given date, in every build.
#[test]
fn test_get_random_positions_at_opens_at_the_given_date() {
    let mut built = chain("TEST");
    for strike in [pos_or_panic!(95.0), Positive::HUNDRED, pos_or_panic!(105.0)] {
        built.add_option(
            strike,
            spos!(5.0),
            spos!(5.5),
            spos!(4.0),
            spos!(4.5),
            pos_or_panic!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
        );
    }
    let positions = built
        .get_random_positions_at(random_params(), base())
        .unwrap();
    assert_eq!(positions.len(), 2);
    assert!(positions.iter().all(|position| position.date == base()));
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
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::model::{OptionStyle, OptionType, Options, Position, Side};
    use optionstratlib_core::utils::time::now;
    use optionstratlib_market::chains::UpdateFromOptionData;
    use optionstratlib_market::error::ChainError;

    /// Fails if anything in this configuration turned the clock on, the
    /// `io`, `synthetic` and `async` dependencies included.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    /// Above the parallel threshold every strike is priced on a rayon worker, which does
    /// not have the caller's reference datetime of its own.
    #[test]
    fn test_build_chain_above_the_parallel_threshold_prices_every_strike() {
        ExpirationDate::set_reference_datetime(Some(base()));
        let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
        let built = OptionChain::build_chain(&chain_params(20, expiration));
        ExpirationDate::set_reference_datetime(None);
        let built = built.unwrap();
        assert_eq!(built.options.len(), 41);
        assert!(built.options.iter().all(|option| option.call_bid.is_some()));
        assert!(
            built
                .options
                .iter()
                .all(|option| option.delta_call.is_some())
        );
    }

    /// Serial and parallel builds must agree, to the decimal.
    #[test]
    fn test_build_chain_gives_the_same_strikes_serially_and_in_parallel() {
        ExpirationDate::set_reference_datetime(Some(base()));
        let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
        let parallel = OptionChain::build_chain(&chain_params(20, expiration)).unwrap();
        let serial = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap()
            .install(|| {
                ExpirationDate::set_reference_datetime(Some(base()));
                let built = OptionChain::build_chain(&chain_params(20, expiration));
                ExpirationDate::set_reference_datetime(None);
                built
            })
            .unwrap();
        ExpirationDate::set_reference_datetime(None);
        let bids = |chain: &OptionChain| {
            chain
                .options
                .iter()
                .map(|option| option.call_bid)
                .collect::<Vec<_>>()
        };
        assert_eq!(bids(&parallel), bids(&serial));
    }

    #[test]
    fn test_build_chain_of_a_datetime_without_a_reference_is_a_typed_error() {
        for chain_size in [3, 20] {
            let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
            assert!(matches!(
                OptionChain::build_chain(&chain_params(chain_size, expiration)),
                Err(ChainError::ExpirationDate(_))
            ));
        }
    }

    /// A chain stores a calendar date, so even a `Days` needs a base here.
    #[test]
    fn test_build_chain_of_days_without_a_reference_is_a_typed_error() {
        let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
        assert!(matches!(
            OptionChain::build_chain(&chain_params(20, expiration)),
            Err(ChainError::ExpirationDate(_))
        ));
    }

    fn chain_without_greeks(chain_size: usize) -> OptionChain {
        ExpirationDate::set_reference_datetime(Some(base()));
        let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
        let mut built = OptionChain::build_chain(&chain_params(chain_size, expiration)).unwrap();
        ExpirationDate::set_reference_datetime(None);
        let stripped = built
            .options
            .iter()
            .cloned()
            .map(|mut option| {
                option.delta_call = None;
                option.delta_put = None;
                option.gamma = None;
                option
            })
            .collect();
        built.options = stripped;
        built
    }

    #[test]
    fn test_try_update_greeks_above_the_parallel_threshold_fills_every_strike() {
        let mut chain = chain_without_greeks(20);
        ExpirationDate::set_reference_datetime(Some(base()));
        let updated = chain.try_update_greeks();
        ExpirationDate::set_reference_datetime(None);
        updated.unwrap();
        assert_eq!(chain.options.len(), 41);
        assert!(chain.options.iter().all(|option| {
            option.delta_call.is_some() && option.delta_put.is_some() && option.gamma.is_some()
        }));
        assert_ne!(chain.gamma_exposure().unwrap(), Decimal::ZERO);
        assert_ne!(chain.delta_exposure().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_try_update_greek_snapshots_above_the_parallel_threshold_fills_every_strike() {
        let mut chain = chain_without_greeks(20);
        ExpirationDate::set_reference_datetime(Some(base()));
        let updated = chain.try_update_greek_snapshots();
        ExpirationDate::set_reference_datetime(None);
        updated.unwrap();
        assert!(
            chain
                .options
                .iter()
                .all(|option| option.greeks_call.is_some())
        );
    }

    #[test]
    fn test_try_update_greeks_without_a_reference_is_a_typed_error_and_changes_nothing() {
        let mut chain = chain_without_greeks(20);
        assert!(matches!(
            chain.try_update_greeks(),
            Err(ChainError::ExpirationDate(_))
        ));
        assert!(chain.options.iter().all(|option| option.gamma.is_none()));
    }

    /// An exposure over strikes with no Greeks is not zero: nothing was measured.
    #[test]
    fn test_exposures_over_empty_greeks_are_errors_not_zero() {
        let chain = chain_without_greeks(20);
        assert!(chain.delta_exposure().is_err());
        assert!(chain.gamma_exposure().is_err());
    }

    /// Refreshing a position re-stamps its open date with the current time, and without a
    /// clock it keeps the date it has. The refresh needs no reference.
    #[test]
    fn test_update_from_option_data_keeps_the_date_of_the_position() {
        let chain = chain_without_greeks(3);
        let row = chain.options.iter().next().unwrap().clone();
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
        position.update_from_option_data(&row).unwrap();
        assert_eq!(position.date, base());
        assert_eq!(Some(position.premium), row.call_ask);
    }

    /// A failed time to expiry used to wipe the quotes and report success.
    #[test]
    fn test_calculate_prices_without_a_time_keeps_the_quotes_and_reports_it() {
        let chain = chain_without_greeks(3);
        let mut row = chain.options.iter().nth(3).unwrap().clone();
        let before = (row.call_bid, row.call_ask, row.put_bid, row.put_ask);
        assert!(before.0.is_some());
        assert!(matches!(
            row.calculate_prices(None),
            Err(ChainError::ExpirationDate(_))
        ));
        assert_eq!(
            (row.call_bid, row.call_ask, row.put_bid, row.put_ask),
            before
        );
    }

    #[test]
    fn test_calculate_prices_with_a_reference_prices_the_row() {
        let chain = chain_without_greeks(3);
        let mut row = chain.options.iter().nth(3).unwrap().clone();
        ExpirationDate::set_reference_datetime(Some(base()));
        let priced = row.calculate_prices(None);
        ExpirationDate::set_reference_datetime(None);
        priced.unwrap();
        assert!(row.call_bid.is_some());
    }

    #[test]
    fn test_get_random_positions_needs_the_dated_form() {
        let chain = chain_without_greeks(3);
        assert!(matches!(
            chain.get_random_positions(random_params()),
            Err(ChainError::Clock(ClockError::Unavailable))
        ));
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
