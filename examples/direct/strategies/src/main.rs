//! A strategy built and read with `optionstratlib-strategies` (#555).
//!
//! A bull call spread on the S&P 500 is constructed from its two strikes and
//! premiums, then asked for its break-even, its maximum loss and profit and
//! the cost of opening it. Only the strategies crate and the layers it sits on
//! (core, math, pricing, market, analytics) are compiled: no simulation,
//! backtest, chart or file I/O. To chart it, see `osl-example-direct-visualization`.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_strategies::strategies::base::BreakEvenable;
use optionstratlib_strategies::strategies::{BullCallSpread, Strategies};
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

/// Long the 5750 call at 85.04, short the 5820 call at 29.85, two days out,
/// two contracts, 0.78 and 0.73 fees on each leg.
fn bull_call_spread() -> Result<BullCallSpread, Box<dyn Error>> {
    Ok(BullCallSpread::new(
        "SP500".to_string(),
        Positive::new(5781.88)?,
        Positive::new(5750.0)?,
        Positive::new(5820.0)?,
        ExpirationDate::Days(Positive::TWO),
        Positive::new(0.18)?,
        dec!(0.05),
        Positive::ZERO,
        Positive::TWO,
        Positive::new(85.04)?,
        Positive::new(29.85)?,
        Positive::new(0.78)?,
        Positive::new(0.78)?,
        Positive::new(0.73)?,
        Positive::new(0.73)?,
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let strategy = bull_call_spread()?;
    info!("break-even points: {:?}", strategy.get_break_even_points()?);
    info!("maximum loss: {}", strategy.get_max_loss()?);
    info!("maximum profit: {}", strategy.get_max_profit()?);
    info!("total cost: {}", strategy.get_total_cost()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bull_call_spread_figures() -> Result<(), Box<dyn Error>> {
        let strategy = bull_call_spread()?;
        let points = strategy.get_break_even_points()?;
        assert_eq!(points.len(), 1);
        // The break-even lies between the two strikes.
        let (low, high) = (Positive::new(5750.0)?, Positive::new(5820.0)?);
        assert!(points.iter().all(|p| *p > low && *p < high), "{points:?}");
        assert_eq!(strategy.get_max_loss()?, Positive::new(116.42)?);
        assert_eq!(strategy.get_total_cost()?, Positive::new(176.12)?);
        Ok(())
    }
}
