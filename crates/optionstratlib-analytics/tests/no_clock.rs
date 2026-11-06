//! The P&L of a transaction with and without the wall clock (#944).

use chrono::{TimeZone, Utc};
use optionstratlib_analytics::pnl::Transaction;
use optionstratlib_core::model::{OptionStyle, OptionType, Positive, Side, TradeStatus};
use optionstratlib_core::pos_or_panic;

fn transaction(date_time: Option<chrono::DateTime<Utc>>) -> Transaction {
    Transaction::new(
        TradeStatus::Open,
        date_time,
        OptionType::European,
        Side::Long,
        OptionStyle::Call,
        Positive::ONE,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        None,
        None,
        None,
    )
}

#[test]
fn test_dated_transaction_has_a_pnl_in_every_build() {
    let date = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
    assert!(transaction(Some(date)).pnl().is_ok());
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_analytics::analytics::projections::BasicCurves;
    use optionstratlib_analytics::error::{ProjectionError, TransactionError};
    use optionstratlib_analytics::metrics::DollarGammaCurve;
    use optionstratlib_analytics::pnl::PnLCalculator;
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::model::{BasicAxisTypes, ExpirationDate, Options, Position};
    use optionstratlib_core::utils::time::now;
    use optionstratlib_market::chains::OptionChain;
    use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn date() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
    }

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

    /// A 21-strike `DateTime` chain, built from the reference datetime, which is then gone.
    fn dated_chain() -> OptionChain {
        ExpirationDate::set_reference_datetime(Some(date()));
        let built = OptionChain::build_chain(&OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            optionstratlib_core::spos!(1.0),
            dec!(-0.3),
            Decimal::ZERO,
            pos_or_panic!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(Positive::HUNDRED)),
                Some(ExpirationDate::DateTime(
                    date() + chrono::Duration::days(30),
                )),
                Some(dec!(0.05)),
                optionstratlib_core::spos!(0.02),
                Some("TEST".to_string()),
            ),
            pos_or_panic!(0.17),
        ));
        ExpirationDate::set_reference_datetime(None);
        built.unwrap()
    }

    /// A position counts a `Days` from its open date to stamp the record.
    #[test]
    fn test_position_pnl_at_expiration_counts_a_days_from_the_open_date() {
        let position = Position::new(
            option(ExpirationDate::Days(pos_or_panic!(30.0))),
            pos_or_panic!(5.0),
            date(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );
        let pnl = position
            .calculate_pnl_at_expiration(&pos_or_panic!(110.0))
            .unwrap();
        assert_eq!(pnl.date_time, date() + chrono::Duration::days(30));
    }

    /// A bare option has no date of its own to count from, so it needs a base.
    #[test]
    fn test_option_pnl_at_expiration_of_a_days_needs_a_base() {
        let bare = option(ExpirationDate::Days(pos_or_panic!(30.0)));
        assert!(
            bare.calculate_pnl_at_expiration(&pos_or_panic!(110.0))
                .is_err()
        );
        ExpirationDate::set_reference_datetime(Some(date()));
        let pnl = bare.calculate_pnl_at_expiration(&pos_or_panic!(110.0));
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(pnl.unwrap().date_time, date() + chrono::Duration::days(30));
    }

    /// The projection skips a strike it cannot compute, so this used to be an empty curve.
    #[test]
    fn test_projection_curve_without_a_time_is_an_error_not_empty() {
        let result = dated_chain().curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);
        assert!(
            matches!(result, Err(ProjectionError::Chain(_))),
            "{result:?}"
        );
    }

    #[test]
    fn test_projection_curve_with_a_reference_has_every_strike() {
        let chain = dated_chain();
        ExpirationDate::set_reference_datetime(Some(date()));
        let curve = chain.curve(&BasicAxisTypes::Delta, &OptionStyle::Call, &Side::Long);
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(curve.unwrap().points.len(), chain.options.len());
    }

    /// A metric skips a strike it cannot compute, and reports a curve with no points.
    #[test]
    fn test_metric_curve_without_a_time_is_an_error() {
        assert!(
            dated_chain()
                .dollar_gamma_curve(&OptionStyle::Call)
                .is_err()
        );
    }

    /// Fails if anything in this configuration turned the clock on.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    #[test]
    fn test_dated_transaction_stamps_its_pnl_with_its_own_time() {
        let date = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(transaction(Some(date)).pnl().unwrap().date_time, date);
    }

    #[test]
    fn test_undated_transaction_is_a_typed_error() {
        assert!(matches!(
            transaction(None).pnl(),
            Err(TransactionError::Clock(_))
        ));
    }
}
