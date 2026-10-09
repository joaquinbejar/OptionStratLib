//! The single-leg backtest evaluates its paths on the rayon pool (#863):
//! the run must equal a serial, walk-by-walk evaluation of the same
//! seeded simulator, path for path and in the aggregate.
//!
//! The serial reference prices every path with
//! [`SingleLegPathEvaluator::evaluate_path`] in walk order and aggregates
//! with [`SimulationStatsResult::from_results`], which is what
//! `simulate_single_leg` did before #863. Both runs are serialised without
//! the `date_time` stamps the P&L takes from the clock and compared whole.

use optionstratlib_backtest::backtesting::Simulate;
use optionstratlib_backtest::backtesting::results::SimulationStatsResult;
use optionstratlib_backtest::backtesting::strategy_simulation::{
    SingleLegPathEvaluator, SingleLegSimulation,
};
use optionstratlib_backtest::error::BacktestError;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::utils::{Len, TimeFrame};
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, PathEvaluator, WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::strategies::base::Positionable;
use optionstratlib_strategies::strategies::{LongCall, ShortPut};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;

/// Points per walk, the initial price included.
const STEPS: usize = 30;

#[derive(Clone)]
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or_else(|e| panic!("{value} is not positive: {e}"))
}

fn simulator(seed: u64, walks: usize) -> Simulator<Positive, Positive> {
    let params = WalkParams {
        size: STEPS,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos(dec!(30))),
            Positive::HUNDRED,
        ),
        walk_type: WalkType::GeometricBrownian {
            dt: pos(dec!(0.004)),
            drift: dec!(0.05),
            volatility: pos(dec!(0.4)),
        },
        walker: Box::new(Walker),
        seed: Some(seed),
    };
    Simulator::new("parallel".to_string(), walks, &params, generator_positive)
        .unwrap_or_else(|e| panic!("simulator failed: {e}"))
}

fn leg(side: Side, style: OptionStyle, premium: Decimal) -> Position {
    let option = Options::new(
        OptionType::European,
        side,
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos(dec!(30))),
        pos(dec!(0.4)),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        style,
        Positive::ZERO,
        None,
    );
    Position::new(
        option,
        pos(premium),
        chrono::Utc::now(),
        pos(dec!(0.5)),
        pos(dec!(0.5)),
        None,
        None,
    )
}

fn long_call() -> LongCall {
    let mut strategy = LongCall::default();
    strategy
        .add_position(&leg(Side::Long, OptionStyle::Call, dec!(4.5)))
        .unwrap_or_else(|e| panic!("{e}"));
    strategy
}

fn short_put() -> ShortPut {
    let mut strategy = ShortPut::default();
    strategy
        .add_position(&leg(Side::Short, OptionStyle::Put, dec!(4.0)))
        .unwrap_or_else(|e| panic!("{e}"));
    strategy
}

fn exit_policies() -> Vec<ExitPolicy> {
    vec![
        ExitPolicy::ProfitPercent(dec!(0.3)),
        ExitPolicy::LossPercent(dec!(0.3)),
        ExitPolicy::TimeSteps(4),
        ExitPolicy::Expiration,
    ]
}

/// The pre-#863 orchestration: every path in walk order on this thread.
fn serial<S: SingleLegSimulation>(
    strategy: &S,
    sim: &Simulator<Positive, Positive>,
    exit: &ExitPolicy,
) -> Result<SimulationStatsResult, BacktestError> {
    let evaluator = SingleLegPathEvaluator::new(strategy)?;
    let mut results = Vec::with_capacity(sim.len());
    for walk in sim {
        results.push(evaluator.evaluate_path(walk, exit)?);
    }
    Ok(SimulationStatsResult::from_results(results)?)
}

/// A run serialised without the clock-dependent `date_time` stamps.
fn comparable(run: &SimulationStatsResult) -> Value {
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("date_time");
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut value = serde_json::to_value(run).unwrap_or_else(|e| panic!("{e}"));
    strip(&mut value);
    value
}

fn assert_parallel_matches_serial<S>(name: &str, strategy: &S)
where
    S: SingleLegSimulation + Simulate<Positive, Positive>,
{
    for (seed, walks) in [(863_u64, 1_usize), (863, 2), (7, 64), (42, 257)] {
        let sim = simulator(seed, walks);
        for exit in exit_policies() {
            let parallel = strategy
                .simulate(&sim, exit.clone())
                .unwrap_or_else(|e| panic!("{name} {exit:?}: {e}"));
            let reference =
                serial(strategy, &sim, &exit).unwrap_or_else(|e| panic!("{name} {exit:?}: {e}"));
            assert_eq!(parallel.results.len(), walks, "{name} {exit:?}");
            assert_eq!(
                comparable(&parallel),
                comparable(&reference),
                "{name}, seed {seed}, {walks} walks, {exit:?}: the parallel run differs"
            );
        }
    }
}

#[test]
fn test_parallel_long_call_backtest_matches_serial() {
    assert_parallel_matches_serial("long call", &long_call());
}

#[test]
fn test_parallel_short_put_backtest_matches_serial() {
    assert_parallel_matches_serial("short put", &short_put());
}
