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
    use optionstratlib_analytics::error::TransactionError;
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::utils::time::now;

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
