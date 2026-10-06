//! A downstream simulation workflow on the `optionstratlib` facade with
//! `default-features = false, features = ["simulation"]` (#541): replayed
//! paths through the `prelude`, a consumer evaluator through the canonical
//! `simulation` path, and proof that both name the items the simulation crate
//! defines. No market, strategy, backtest or plotting code is compiled.

use optionstratlib::prelude::*;
use optionstratlib::simulation::{
    ExitPolicy, PathEvaluator, PathOutcome, PathStatistics, evaluate_paths,
};

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, Positive> for Replay {}

/// P&L is the move from the first to the last price.
struct LastMinusFirst;

impl PathEvaluator<Positive, Positive> for LastMinusFirst {
    type Outcome = PathOutcome;
    type Error = optionstratlib::error::SimulationError;

    fn evaluate_path(
        &self,
        walk: &RandomWalk<Positive, Positive>,
        exit: &ExitPolicy,
    ) -> Result<PathOutcome, Self::Error> {
        let steps = walk.get_steps();
        let (Some(first), Some(last)) = (steps.first(), steps.last()) else {
            return Err(optionstratlib::error::SimulationError::invalid_parameters(
                "empty walk",
            ));
        };
        let pnl = last
            .y
            .positive()?
            .to_dec()
            .checked_sub(first.y.positive()?.to_dec())
            .ok_or_else(|| {
                optionstratlib::error::SimulationError::invalid_parameters("P&L overflow")
            })?;
        Ok(PathOutcome {
            pnl: Some(pnl),
            holding_period: steps.len().checked_sub(1).ok_or_else(|| {
                optionstratlib::error::SimulationError::invalid_parameters("empty walk")
            })?,
            exit_reason: exit.clone(),
            expired: true,
            ..PathOutcome::default()
        })
    }
}

fn simulator(prices: &[f64]) -> Simulator<Positive, Positive> {
    let params = WalkParams {
        size: prices.len(),
        init_step: Step::new(
            Positive::ONE,
            optionstratlib::utils::TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walker: Box::new(Replay),
        walk_type: WalkType::Historical {
            timeframe: optionstratlib::utils::TimeFrame::Day,
            prices: prices.iter().map(|p| pos_or_panic!(*p)).collect(),
            symbol: Some("XYZ".to_string()),
        },
        seed: None,
    };
    match Simulator::new("fixture".to_string(), 2, &params, generator_positive) {
        Ok(sim) => sim,
        Err(error) => panic!("simulator: {error}"),
    }
}

#[test]
fn test_evaluate_paths_rising_and_falling_replays_report_statistics() {
    let exit = ExitPolicy::Expiration;
    let mut outcomes =
        match evaluate_paths(&LastMinusFirst, &simulator(&[100.0, 104.0, 108.0]), &exit) {
            Ok(outcomes) => outcomes,
            Err(error) => panic!("rising: {error}"),
        };
    match evaluate_paths(&LastMinusFirst, &simulator(&[100.0, 98.0, 96.0]), &exit) {
        Ok(more) => outcomes.extend(more),
        Err(error) => panic!("falling: {error}"),
    }
    match PathStatistics::from_outcomes(&outcomes) {
        Ok(stats) => {
            assert_eq!(stats.total_paths, 4);
            assert_eq!(stats.profitable_count, 2);
            assert_eq!(stats.loss_count, 2);
            assert_eq!(stats.average_pnl, dec!(2));
            assert_eq!(stats.best_pnl, dec!(8));
            assert_eq!(stats.worst_pnl, dec!(-4));
            assert_eq!(stats.win_rate, dec!(50));
            assert_eq!(stats.average_holding_period, dec!(2));
        }
        Err(error) => panic!("path statistics: {error}"),
    }
}

#[test]
fn test_facade_paths_are_the_simulation_items() {
    same_item(
        optionstratlib::simulation::evaluate_paths::<Positive, Positive, LastMinusFirst>,
        optionstratlib_simulation::simulation::evaluate_paths::<Positive, Positive, LastMinusFirst>,
    );
    same_item(
        optionstratlib::simulation::generator_positive,
        optionstratlib_simulation::simulation::generator_positive,
    );
    let error: optionstratlib_simulation::error::SimulationError =
        optionstratlib::error::SimulationError::walk_error("probe");
    assert!(error.to_string().contains("probe"));
}
