//! Option chains and series with `optionstratlib-market` and no features
//! (#555).
//!
//! The market crate with its defaults compiles no simulation engine, no file
//! I/O and no async runtime: a chain and a series are built from parameters,
//! priced with the pricing crate underneath, and serialized with `serde`.
//! Enable `io` for the CSV, JSON and ZIP readers and writers, `synthetic` for
//! the simulation-backed generators; neither is needed here.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_market::chains::utils::OptionDataPriceParams;
use optionstratlib_market::chains::{OptionChain, OptionChainBuildParams};
use optionstratlib_market::series::{OptionSeries, OptionSeriesBuildParams};
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

/// Strikes every 5 around a spot of 100, 20% volatility at the money with a
/// negative skew, 30 days to expiry.
fn chain_params() -> Result<OptionChainBuildParams, Box<dyn Error>> {
    Ok(OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        5,
        Some(Positive::new(5.0)?),
        dec!(-0.2),
        dec!(0.0),
        Positive::new(0.01)?,
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(Positive::new(30.0)?)),
            Some(dec!(0.05)),
            Some(Positive::new(0.02)?),
            Some("XYZ".to_string()),
        ),
        Positive::new(0.2)?,
    ))
}

/// The chain, its at-the-money strike and its JSON size.
fn chain_summary() -> Result<(usize, Positive, usize), Box<dyn Error>> {
    let chain = OptionChain::build_chain(&chain_params()?)?;
    let atm = chain.atm_option_data()?;
    let json = serde_json::to_string(&chain)?;
    Ok((chain.options.len(), atm.strike_price, json.len()))
}

/// A series: the same chain at three expirations.
fn series_len() -> Result<usize, Box<dyn Error>> {
    let params = OptionSeriesBuildParams::new(
        chain_params()?,
        vec![
            Positive::new(7.0)?,
            Positive::new(14.0)?,
            Positive::new(30.0)?,
        ],
    );
    Ok(OptionSeries::build_series(&params)?.chains.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let (strikes, atm, bytes) = chain_summary()?;
    info!("chain: {strikes} strikes, at the money {atm}, {bytes} bytes of JSON");
    info!("series: {} expirations", series_len()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_and_series_build_without_simulation_or_io() -> Result<(), Box<dyn Error>> {
        let (strikes, atm, bytes) = chain_summary()?;
        assert_eq!(strikes, 11);
        assert_eq!(atm, Positive::HUNDRED);
        assert!(bytes > 0);
        assert_eq!(series_len()?, 3);
        Ok(())
    }
}
