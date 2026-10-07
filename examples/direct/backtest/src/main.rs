//! A long call backtested over seeded paths with the component crates and no
//! facade (#555).
//!
//! `optionstratlib-simulation` generates the paths, `optionstratlib-strategies`
//! supplies the long call and `optionstratlib-backtest` evaluates the one on
//! the other and reports run statistics. The seed makes the report
//! reproducible. No chart, option chain I/O or async code is compiled; add
//! `optionstratlib-visualization` only to plot the result.

use optionstratlib_backtest::backtesting::{Simulate, SimulationStatsResult};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::strategies::LongCall;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

/// Walks that keep every default, which includes the stochastic kernels.
#[derive(Clone)]
struct Gbm;

impl WalkTypeAble<Positive, Positive> for Gbm {}

const WALKS: usize = 100;

/// Long 100 call, 30 days, 20% volatility, premium 5, 0.5 fee at each end.
fn long_call() -> Result<LongCall, Box<dyn Error>> {
    Ok(LongCall::new(
        "XYZ".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(Positive::new(30.0)?),
        Positive::new(0.20)?,
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        Positive::new(5.0)?,
        Positive::new(0.5)?,
        Positive::new(0.5)?,
    )?)
}

/// The long call held to expiration on `WALKS` seeded daily paths.
fn backtest(seed: u64) -> Result<SimulationStatsResult, Box<dyn Error>> {
    let params = WalkParams {
        size: 30,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(Positive::new(30.0)?),
            Positive::HUNDRED,
        ),
        walker: Box::new(Gbm),
        walk_type: WalkType::GeometricBrownian {
            dt: Positive::new(1.0 / 365.0)?,
            drift: dec!(0.05),
            volatility: Positive::new(0.2)?,
        },
        seed: Some(seed),
    };
    let simulator = Simulator::new(
        "direct-backtest".to_string(),
        WALKS,
        &params,
        generator_positive,
    )?;
    Ok(long_call()?.simulate(&simulator, ExitPolicy::Expiration)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let stats = backtest(42)?;
    info!(
        "{} paths: average P&L {:.2}, win rate {:.1}%, best {:.2}, worst {:.2}",
        stats.total_simulations, stats.average_pnl, stats.win_rate, stats.best_pnl, stats.worst_pnl
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_report_is_reproducible_and_bounded() -> Result<(), Box<dyn Error>> {
        let stats = backtest(7)?;
        assert_eq!(stats.total_simulations, WALKS);
        assert_eq!(stats.profitable_count + stats.loss_count, WALKS);
        assert!(stats.win_rate >= dec!(0) && stats.win_rate <= dec!(100));
        // The most a long call can lose is the premium and the fees: 5 + 1.
        assert!(stats.worst_pnl >= dec!(-6), "worst {}", stats.worst_pnl);
        assert_eq!(stats.average_pnl, backtest(7)?.average_pnl);
        Ok(())
    }
}
