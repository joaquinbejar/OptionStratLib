//! A downstream chain and series workflow on the `optionstratlib` facade with
//! `default-features = false, features = ["market"]` (#528): the convenience
//! `prelude` for the common path, the canonical module paths for the rest,
//! and proof that both name the items the market crate defines. No I/O,
//! simulation, async or charting code is compiled.

use optionstratlib::chains::utils::OptionDataPriceParams;
use optionstratlib::prelude::*;

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

fn chain_params() -> OptionChainBuildParams {
    let price_params = OptionDataPriceParams::new(
        Some(Box::new(Positive::HUNDRED)),
        Some(ExpirationDate::Days(pos_or_panic!(30.0))),
        Some(dec!(0.05)),
        spos!(0.02),
        Some("XYZ".to_string()),
    );
    OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        5,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.0),
        pos_or_panic!(0.01),
        2,
        price_params,
        pos_or_panic!(0.2),
    )
}

#[test]
fn test_build_a_chain_through_the_prelude() {
    let chain = match OptionChain::build_chain(&chain_params()) {
        Ok(chain) => chain,
        Err(error) => panic!("build_chain: {error}"),
    };
    assert_eq!(chain.symbol, "XYZ");
    assert!(!chain.options.is_empty());
    match chain.atm_option_data() {
        Ok(atm) => assert_eq!(atm.strike_price, Positive::HUNDRED),
        Err(error) => panic!("atm_option_data: {error}"),
    }
}

#[test]
fn test_hand_built_chain_reads_back() {
    let mut chain = OptionChain::new("XYZ", Positive::HUNDRED, "2030-01-17".to_string(), None, None);
    chain.add_option(
        Positive::HUNDRED,
        spos!(4.9),
        spos!(5.1),
        spos!(4.4),
        spos!(4.6),
        pos_or_panic!(0.2),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(chain.get_call_price(Positive::HUNDRED), Some(dec!(5.1)));
}

#[test]
fn test_build_a_series_through_the_prelude() {
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
fn test_facade_paths_are_the_component_items() {
    same_item(
        optionstratlib::chains::OptionChain::build_chain,
        optionstratlib_market::chains::OptionChain::build_chain,
    );
    same_item(
        optionstratlib::series::OptionSeries::build_series,
        optionstratlib_market::series::OptionSeries::build_series,
    );
    let error: optionstratlib_market::error::ChainError =
        optionstratlib::error::ChainError::invalid_strike(-1.0, "probe");
    assert!(error.to_string().contains("probe"));
}
