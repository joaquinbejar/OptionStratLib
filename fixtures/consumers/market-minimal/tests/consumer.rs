//! Option chains and series from `optionstratlib-market` with no features
//! (#537): no simulation engine and no file I/O is compiled, yet chains and
//! series build, price and serialize.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::{pos_or_panic, spos};
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib_market::series::{OptionSeries, OptionSeriesBuildParams};
use rust_decimal_macros::dec;

fn chain_params() -> OptionChainBuildParams {
    OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        5,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.0),
        pos_or_panic!(0.01),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("XYZ".to_string()),
        ),
        pos_or_panic!(0.2),
    )
}

#[test]
fn test_build_a_chain_without_simulation_or_io() {
    let chain = match OptionChain::build_chain(&chain_params()) {
        Ok(chain) => chain,
        Err(error) => panic!("build_chain: {error}"),
    };
    assert_eq!(chain.options.len(), 11);
    match chain.atm_option_data() {
        Ok(atm) => assert_eq!(atm.strike_price, Positive::HUNDRED),
        Err(error) => panic!("atm_option_data: {error}"),
    }
}

#[test]
fn test_build_a_series_without_simulation_or_io() {
    let params = OptionSeriesBuildParams::new(
        chain_params(),
        vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0)],
    );
    match OptionSeries::build_series(&params) {
        Ok(series) => assert_eq!(series.chains.len(), 3),
        Err(error) => panic!("build_series: {error}"),
    }
}

#[test]
fn test_chain_round_trips_through_json_without_io() {
    let chain = match OptionChain::build_chain(&chain_params()) {
        Ok(chain) => chain,
        Err(error) => panic!("build_chain: {error}"),
    };
    let json = match serde_json::to_string(&chain) {
        Ok(json) => json,
        Err(error) => panic!("serialize: {error}"),
    };
    match serde_json::from_str::<OptionChain>(&json) {
        Ok(back) => assert_eq!(back.options.len(), chain.options.len()),
        Err(error) => panic!("deserialize: {error}"),
    }
}
