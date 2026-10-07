//! Seeded price paths with `optionstratlib-simulation` alone (#555).
//!
//! Geometric Brownian paths are generated from a seed, so a run is
//! reproducible: the same seed gives the same prices, a different seed does
//! not. Only core, math, pricing and the simulation crate are compiled: no
//! option chain, strategy, backtest, chart or file I/O.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

/// Walks that keep every default, which includes the stochastic kernels.
#[derive(Clone)]
struct Gbm;

impl WalkTypeAble<Positive, Positive> for Gbm {}

const WALKS: usize = 50;
const STEPS: usize = 30;

/// 50 daily paths of 30 steps from a spot of 100, 5% drift, 20% volatility.
fn terminal_prices(seed: u64) -> Result<Vec<Positive>, Box<dyn Error>> {
    let params = WalkParams {
        size: STEPS,
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
        "direct-simulation".to_string(),
        WALKS,
        &params,
        generator_positive,
    )?;
    let mut last = Vec::with_capacity(WALKS);
    for walk in &simulator {
        let steps = walk.get_steps();
        let step = steps.last().ok_or("a simulated walk has no steps")?;
        last.push(step.y.positive()?);
    }
    Ok(last)
}

/// Mean of a non-empty list of prices, in `Decimal`.
fn mean(prices: &[Positive]) -> Result<Decimal, Box<dyn Error>> {
    let mut total = Decimal::ZERO;
    for price in prices {
        total = total
            .checked_add(price.to_dec())
            .ok_or("the sum of the prices overflowed")?;
    }
    let count = Decimal::from(u64::try_from(prices.len())?);
    total.checked_div(count).ok_or_else(|| "no prices".into())
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let prices = terminal_prices(42)?;
    info!(
        "{} paths of {STEPS} steps, mean terminal price {:.2}",
        prices.len(),
        mean(&prices)?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_same_seed_replays_the_same_paths() -> Result<(), Box<dyn Error>> {
        let first = terminal_prices(7)?;
        assert_eq!(first.len(), WALKS);
        assert_eq!(first, terminal_prices(7)?);
        assert_ne!(first, terminal_prices(8)?);
        // Drift is small over 30 days: the mean stays near the spot.
        let average = mean(&first)?;
        assert!(average > dec!(90) && average < dec!(112), "mean {average}");
        Ok(())
    }
}
