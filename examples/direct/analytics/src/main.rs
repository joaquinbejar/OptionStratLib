//! Analytics without strategies (#555): profit and loss, margin and the
//! market-implied distribution, from `optionstratlib-analytics`,
//! `optionstratlib-market` and `optionstratlib-core`.
//!
//! A short put is valued at expiration, margined with SPAN, and a chain is
//! turned into its risk-neutral density. No strategy type, simulation, chart
//! or file I/O is compiled.

use chrono::Utc;
use optionstratlib_analytics::analytics::{RNDAnalysis, RNDParameters};
use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_analytics::risk::SPANMargin;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_market::chains::utils::OptionDataPriceParams;
use optionstratlib_market::chains::{OptionChain, OptionChainBuildParams};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

/// Short 95 put on a spot of 100, 30 days, sold for 1.5.
fn short_put() -> Result<Position, Box<dyn Error>> {
    let option = Options::new(
        OptionType::European,
        Side::Short,
        "XYZ".to_string(),
        Positive::new(95.0)?,
        ExpirationDate::Days(Positive::new(30.0)?),
        Positive::new(0.2)?,
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Put,
        Positive::ZERO,
        None,
    );
    Ok(Position::new(
        option,
        Positive::new(1.5)?,
        Utc::now(),
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    ))
}

fn chain() -> Result<OptionChain, Box<dyn Error>> {
    let params = OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        10,
        Some(Positive::new(5.0)?),
        dec!(-0.2),
        dec!(0.1),
        Positive::new(0.02)?,
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(Positive::new(30.0)?)),
            Some(dec!(0.05)),
            Some(Positive::new(0.02)?),
            Some("XYZ".to_string()),
        ),
        Positive::new(0.20)?,
    );
    Ok(OptionChain::build_chain(&params)?)
}

/// P&L of the short put at one price, its SPAN margin, and the mean of the
/// chain's risk-neutral density.
fn analytics(spot: f64) -> Result<(Option<Decimal>, Decimal, Decimal), Box<dyn Error>> {
    let position = short_put()?;
    let pnl = position
        .calculate_pnl_at_expiration(&Positive::new(spot)?)?
        .realized;
    let margin = SPANMargin::new(dec!(0.1), dec!(0.05), dec!(0.1)).calculate_margin(&position)?;
    let density = chain()?.calculate_rnd(&RNDParameters {
        risk_free_rate: dec!(0.05),
        derivative_tolerance: Positive::new(0.1)?,
    })?;
    Ok((pnl, margin, density.statistics.mean))
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    for spot in [100.0, 90.0] {
        let (pnl, margin, mean) = analytics(spot)?;
        info!("spot {spot}: P&L {pnl:?}, SPAN margin {margin}, risk-neutral mean {mean:.2}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_put_pnl_margin_and_density() -> Result<(), Box<dyn Error>> {
        // Above the strike the premium is kept; below it the intrinsic value
        // is lost. With a 10% minimum the margin floor is 0.1 x 100.
        let (kept, margin, mean) = analytics(100.0)?;
        assert_eq!(kept, Some(dec!(1.5)));
        assert_eq!(margin, dec!(10));
        assert!(mean > dec!(95) && mean < dec!(105), "mean {mean}");
        assert_eq!(analytics(90.0)?.0, Some(dec!(-3.5)));
        Ok(())
    }
}
