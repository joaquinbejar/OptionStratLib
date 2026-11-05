//! Backtest results with and without the wall clock (#944).

use chrono::{TimeZone, Utc};
use optionstratlib_backtest::backtesting::SimulationResult;
use optionstratlib_simulation::simulation::PathOutcome;
use rust_decimal_macros::dec;

fn outcome() -> PathOutcome {
    PathOutcome {
        pnl: Some(dec!(10.0)),
        holding_period: 3,
        ..PathOutcome::default()
    }
}

#[test]
fn test_from_outcome_at_stamps_the_pnl_with_the_given_time() {
    let date = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
    let result = SimulationResult::from_outcome_at(outcome(), date);
    assert_eq!(result.pnl.date_time, date);
    assert_eq!(result.pnl.realized, Some(dec!(10.0)));
    assert_eq!(result.holding_period, 3);
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_backtest::backtesting::results::SimulationStatsResult;
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::utils::time::now;
    use optionstratlib_simulation::error::SimulationError;

    /// Fails if anything in this configuration turned the clock on.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    #[test]
    fn test_from_outcomes_without_a_reference_is_a_typed_error() {
        assert!(matches!(
            SimulationStatsResult::from_outcomes(vec![outcome()]),
            Err(SimulationError::ExpirationDate(_))
        ));
    }

    #[test]
    fn test_from_outcomes_stamps_with_the_reference_datetime() {
        let date = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
        ExpirationDate::set_reference_datetime(Some(date));
        let stats = SimulationStatsResult::from_outcomes(vec![outcome()]);
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(stats.unwrap().results[0].pnl.date_time, date);
    }
}
