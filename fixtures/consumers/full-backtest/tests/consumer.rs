//! A long call backtested over replayed price paths with the component
//! crates and no facade (#540). Historical walks replay their prices, so the
//! report is deterministic; the figures are the ones the backtest crate's
//! single-leg golden regression pins (`long_call/path0/exit0` and
//! `long_call/path1/exit0`).

use optionstratlib_backtest::backtesting::{Simulate, SimulationStatsResult};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::strategies::LongCall;
use rust_decimal_macros::dec;

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
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walker: Box::new(Replay),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: prices.iter().map(|p| pos_or_panic!(*p)).collect(),
            symbol: Some("TEST".to_string()),
        },
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
fn test_long_call_on_a_rising_path_reports_the_expiry_profit() {
    // Expires 20 in the money: 20 - premium 5 - fees 1 = 14 on each walk.
    let stats = backtest(&[100.0, 105.0, 110.0, 115.0, 120.0]);
    assert_eq!(stats.total_simulations, 2);
    assert_eq!(stats.profitable_count, 2);
    assert_eq!(stats.loss_count, 0);
    assert_eq!(stats.average_pnl, dec!(14));
    assert_eq!(stats.best_pnl, dec!(14));
    assert_eq!(stats.win_rate, dec!(100));
    assert_eq!(stats.average_holding_period, dec!(4));
}

#[test]
fn test_long_call_on_a_falling_path_reports_the_premium_and_fees_lost() {
    // Expires worthless: -premium 5 - fees 1 = -6 on each walk.
    let stats = backtest(&[100.0, 95.0, 90.0, 85.0, 80.0]);
    assert_eq!(stats.total_simulations, 2);
    assert_eq!(stats.profitable_count, 0);
    assert_eq!(stats.loss_count, 2);
    assert_eq!(stats.average_pnl, dec!(-6));
    assert_eq!(stats.worst_pnl, dec!(-6));
    assert_eq!(stats.win_rate, dec!(0));
}
