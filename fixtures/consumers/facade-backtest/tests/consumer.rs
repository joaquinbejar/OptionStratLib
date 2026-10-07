//! A downstream backtest on the `optionstratlib` facade with
//! `default-features = false, features = ["backtest"]` (#541): a long call
//! simulated over replayed paths through the `prelude`, and proof that the
//! facade paths name the items the backtest crate defines. No plotly, I/O or
//! async package is resolved. The figures are the ones the backtest crate's
//! golden regression pins (`long_call/path0/exit0`, `long_call/path1/exit0`).

use optionstratlib::backtesting::SimulationStatsResult;
use optionstratlib::prelude::*;
use optionstratlib::simulation::ExitPolicy;
use optionstratlib::simulation::generator_positive;

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, Positive> for Replay {}

/// Long 100 call, 30 days, premium 5, open and close fees 0.5 each.
fn long_call() -> LongCall {
    match LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.20),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    ) {
        Ok(strategy) => strategy,
        Err(error) => panic!("LongCall::new: {error}"),
    }
}

fn backtest(prices: &[f64]) -> SimulationStatsResult {
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
            symbol: Some("TEST".to_string()),
        },
        seed: None,
    };
    let simulator = match Simulator::new("fixture".to_string(), 2, &params, generator_positive) {
        Ok(sim) => sim,
        Err(error) => panic!("simulator: {error}"),
    };
    match long_call().simulate(&simulator, ExitPolicy::Expiration) {
        Ok(stats) => stats,
        Err(error) => panic!("simulate: {error}"),
    }
}

#[test]
fn test_long_call_backtest_through_the_prelude() {
    // In the money at expiry: 20 - premium 5 - fees 1 = 14 per walk.
    let rising = backtest(&[100.0, 105.0, 110.0, 115.0, 120.0]);
    assert_eq!(rising.total_simulations, 2);
    assert_eq!(rising.average_pnl, dec!(14));
    assert_eq!(rising.win_rate, dec!(100));
    assert_eq!(rising.average_holding_period, dec!(4));
    // Worthless at expiry: -premium 5 - fees 1 = -6 per walk.
    let falling = backtest(&[100.0, 95.0, 90.0, 85.0, 80.0]);
    assert_eq!(falling.total_simulations, 2);
    assert_eq!(falling.average_pnl, dec!(-6));
    assert_eq!(falling.win_rate, dec!(0));
    assert_eq!(falling.loss_count, 2);
    assert_eq!(falling.average_holding_period, dec!(4));
}

#[test]
fn test_facade_paths_are_the_backtest_items() {
    same_item(
        optionstratlib::backtesting::results::SimulationStatsResult::from_results,
        optionstratlib_backtest::backtesting::results::SimulationStatsResult::from_results,
    );
    let error: optionstratlib_backtest::error::BacktestError =
        optionstratlib::error::BacktestError::from(
            optionstratlib::error::SimulationError::walk_error("probe"),
        );
    assert!(error.to_string().contains("probe"));
}
