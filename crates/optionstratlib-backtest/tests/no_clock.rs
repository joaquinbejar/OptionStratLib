//! Backtest results with and without the wall clock (#944).

use chrono::{TimeZone, Utc};
use optionstratlib_backtest::backtesting::{Simulate, SimulationResult};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, PathOutcome, WalkParams, WalkType, WalkTypeAble,
};
use optionstratlib_strategies::strategies::LongCall;
use rust_decimal_macros::dec;
use std::convert::Infallible;

#[derive(Clone)]
struct Walker;
impl WalkTypeAble<Positive, Positive> for Walker {}

fn date() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
}

fn long_call() -> LongCall {
    LongCall::new_at(
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
        date(),
    )
    .unwrap()
}

/// Several paths, so the simulation runs on rayon workers.
fn simulator(first: ExpirationDate) -> Simulator<Positive, Positive> {
    let prices = vec![
        Positive::HUNDRED,
        pos_or_panic!(130.0),
        pos_or_panic!(140.0),
    ];
    let params = WalkParams {
        size: prices.len(),
        init_step: Step::new(Positive::ONE, TimeFrame::Day, first, Positive::HUNDRED),
        walker: Box::new(Walker),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices,
            symbol: Some("TEST".to_string()),
        },
        seed: None,
    };
    Simulator::new(
        "paths".to_string(),
        8,
        &params,
        |walk: &WalkParams<Positive, Positive>| {
            let mut steps = vec![walk.init_step.clone()];
            let mut step = walk.init_step.clone();
            for price in [pos_or_panic!(130.0), pos_or_panic!(140.0)] {
                step = step.next(price).unwrap();
                steps.push(step.clone());
            }
            Ok::<_, Infallible>(steps)
        },
    )
    .unwrap()
}

/// A `Days` strategy over `Days` steps reads no time, and gives the same result in
/// every build.
#[test]
fn test_simulate_days_paths_needs_no_time_in_any_build() {
    let simulator = simulator(ExpirationDate::Days(pos_or_panic!(30.0)));
    let strategy = long_call();
    for exit in [ExitPolicy::Expiration, ExitPolicy::ProfitPercent(dec!(0.5))] {
        let stats = strategy.simulate(&simulator, exit).unwrap();
        assert_eq!(stats.total_simulations, 8);
        assert_eq!(stats.results.len(), 8);
    }
}

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

    /// An exit is stamped with the leg's open date, not with a made-up time.
    #[test]
    fn test_simulate_stamps_an_exit_with_the_leg_date() {
        let simulator = simulator(ExpirationDate::Days(pos_or_panic!(30.0)));
        let stats = long_call()
            .simulate(&simulator, ExitPolicy::ProfitPercent(dec!(0.5)))
            .unwrap();
        assert!(stats.results.iter().all(|result| result.hit_take_profit));
        assert!(
            stats
                .results
                .iter()
                .all(|result| result.pnl.date_time == date())
        );
    }

    /// At expiration the record is stamped with the expiry, counted from the open date.
    #[test]
    fn test_simulate_stamps_an_expiration_from_the_leg_date() {
        let simulator = simulator(ExpirationDate::Days(pos_or_panic!(30.0)));
        let stats = long_call()
            .simulate(&simulator, ExitPolicy::Expiration)
            .unwrap();
        let expiry = date() + chrono::Duration::days(30);
        assert!(
            stats
                .results
                .iter()
                .all(|result| result.pnl.date_time == expiry)
        );
    }

    /// `DateTime` steps read their days on rayon workers, from the caller's reference.
    #[test]
    fn test_simulate_datetime_paths_carries_the_reference_to_the_workers() {
        let first = ExpirationDate::DateTime(date() + chrono::Duration::days(30));
        ExpirationDate::set_reference_datetime(Some(date()));
        let simulator = simulator(first);
        let stats = long_call().simulate(&simulator, ExitPolicy::Expiration);
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(stats.unwrap().total_simulations, 8);
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
