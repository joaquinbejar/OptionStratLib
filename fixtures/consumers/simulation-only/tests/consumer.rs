//! Simulated price paths evaluated by the consumer's own evaluator, on
//! `optionstratlib-simulation` alone (#540): no option chain, strategy,
//! backtest or facade code. Historical walks replay their prices, so every
//! figure here is deterministic.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::{Len, TimeFrame};
use optionstratlib_simulation::error::SimulationError;
use optionstratlib_simulation::simulation::randomwalk::RandomWalk;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, PathEvaluator, PathOutcome, PathStatistics, WalkParams, WalkType, WalkTypeAble,
    evaluate_paths, generator_positive,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, Positive> for Replay {}

/// The consumer's evaluator: P&L is the move from the first to the last
/// price, the holding period the number of steps taken.
struct LastMinusFirst;

impl PathEvaluator<Positive, Positive> for LastMinusFirst {
    type Outcome = PathOutcome;
    type Error = SimulationError;

    fn evaluate_path(
        &self,
        walk: &RandomWalk<Positive, Positive>,
        exit: &ExitPolicy,
    ) -> Result<PathOutcome, SimulationError> {
        let steps = walk.get_steps();
        let (Some(first), Some(last)) = (steps.first(), steps.last()) else {
            return Err(SimulationError::invalid_parameters("empty walk"));
        };
        let pnl: Decimal = last
            .y
            .positive()?
            .to_dec()
            .checked_sub(first.y.positive()?.to_dec())
            .ok_or_else(|| SimulationError::invalid_parameters("P&L overflow"))?;
        let holding_period = steps
            .len()
            .checked_sub(1)
            .ok_or_else(|| SimulationError::invalid_parameters("empty walk"))?;
        Ok(PathOutcome {
            pnl: Some(pnl),
            holding_period,
            exit_reason: exit.clone(),
            expired: true,
            ..PathOutcome::default()
        })
    }
}

fn simulator(prices: Vec<Positive>, walks: usize) -> Simulator<Positive, Positive> {
    let params = WalkParams {
        size: prices.len(),
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walker: Box::new(Replay),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices,
            symbol: Some("XYZ".to_string()),
        },
        seed: None,
    };
    match Simulator::new("fixture".to_string(), walks, &params, generator_positive) {
        Ok(sim) => sim,
        Err(error) => panic!("simulator: {error}"),
    }
}

#[test]
fn test_simulator_replayed_paths_follow_their_prices() {
    let prices = vec![
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        pos_or_panic!(110.0),
    ];
    let sim = simulator(prices.clone(), 2);
    assert_eq!(sim.len(), 2);
    for walk in &sim {
        let replayed: Vec<Positive> = walk
            .get_steps()
            .iter()
            .map(|step| match step.y.positive() {
                Ok(price) => price,
                Err(error) => panic!("step price: {error}"),
            })
            .collect();
        assert_eq!(replayed, prices);
    }
}

#[test]
fn test_evaluate_paths_rising_and_falling_replays_report_statistics() {
    let exit = ExitPolicy::Expiration;
    let rising = simulator(
        vec![
            Positive::HUNDRED,
            pos_or_panic!(104.0),
            pos_or_panic!(108.0),
        ],
        2,
    );
    let falling = simulator(
        vec![Positive::HUNDRED, pos_or_panic!(98.0), pos_or_panic!(96.0)],
        2,
    );
    let mut outcomes = match evaluate_paths(&LastMinusFirst, &rising, &exit) {
        Ok(outcomes) => outcomes,
        Err(error) => panic!("rising: {error}"),
    };
    match evaluate_paths(&LastMinusFirst, &falling, &exit) {
        Ok(more) => outcomes.extend(more),
        Err(error) => panic!("falling: {error}"),
    }
    assert!(
        outcomes
            .iter()
            .all(|o| o.holding_period == 2 && o.exit_reason == exit)
    );
    // P&L {8, 8, -4, -4}: mean 2, median (−4 + 8) / 2 = 2, sample standard
    // deviation sqrt(4 · 36 / 3) = sqrt(48).
    match PathStatistics::from_outcomes(&outcomes) {
        Ok(stats) => {
            assert_eq!(stats.total_paths, 4);
            assert_eq!(stats.profitable_count, 2);
            assert_eq!(stats.loss_count, 2);
            assert_eq!(stats.average_pnl, dec!(2));
            assert_eq!(stats.median_pnl, dec!(2));
            assert_eq!(stats.best_pnl, dec!(8));
            assert_eq!(stats.worst_pnl, dec!(-4));
            assert_eq!(stats.win_rate, dec!(50));
            assert_eq!(stats.average_holding_period, dec!(2));
            let variance = stats.std_dev_pnl * stats.std_dev_pnl;
            assert!(
                (variance - dec!(48)).abs() < dec!(0.000001),
                "std dev {}",
                stats.std_dev_pnl
            );
        }
        Err(error) => panic!("path statistics: {error}"),
    }
}

/// An evaluator that refuses every path, to check the error reaches the
/// caller unchanged.
struct AlwaysFails;

impl PathEvaluator<Positive, Positive> for AlwaysFails {
    type Outcome = PathOutcome;
    type Error = SimulationError;

    fn evaluate_path(
        &self,
        _walk: &RandomWalk<Positive, Positive>,
        _exit: &ExitPolicy,
    ) -> Result<PathOutcome, SimulationError> {
        Err(SimulationError::invalid_parameters(
            "fixture refuses every path",
        ))
    }
}

#[test]
fn test_evaluate_paths_propagates_the_evaluator_error() {
    let sim = simulator(vec![Positive::HUNDRED, pos_or_panic!(101.0)], 2);
    match evaluate_paths(&AlwaysFails, &sim, &ExitPolicy::Expiration) {
        Err(SimulationError::InvalidParameters { .. }) => {}
        other => panic!("expected the evaluator's error, got {other:?}"),
    }
}

#[test]
fn test_evaluate_paths_repeated_runs_are_identical() {
    let prices = vec![Positive::HUNDRED, pos_or_panic!(97.0), pos_or_panic!(103.0)];
    let run = || match evaluate_paths(
        &LastMinusFirst,
        &simulator(prices.clone(), 2),
        &ExitPolicy::Expiration,
    ) {
        Ok(outcomes) => outcomes.iter().map(|o| o.pnl).collect::<Vec<_>>(),
        Err(error) => panic!("evaluate_paths: {error}"),
    };
    let first = run();
    assert_eq!(first, vec![Some(dec!(3)), Some(dec!(3))]);
    assert_eq!(first, run());
}
