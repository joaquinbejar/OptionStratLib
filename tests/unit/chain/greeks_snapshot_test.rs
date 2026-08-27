/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/8/26
******************************************************************************/

//! Verifies that a chain built through the normal path carries the full
//! twelve-greek snapshot on every strike, for both option styles.

use optionstratlib::ExpirationDate;
use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use positive::{Positive, pos_or_panic, spos};
use rust_decimal_macros::dec;

fn build_params() -> OptionChainBuildParams {
    let price_params = OptionDataPriceParams::new(
        Some(Box::new(Positive::HUNDRED)),
        Some(ExpirationDate::Days(pos_or_panic!(30.0))),
        Some(dec!(0.05)),
        spos!(0.02),
        Some("TEST".to_string()),
    );

    OptionChainBuildParams::new(
        "TEST".to_string(),
        None,
        10,
        spos!(1.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        price_params,
        pos_or_panic!(0.2),
    )
}

#[test]
fn test_built_chain_carries_greek_snapshots_on_every_strike() {
    let chain = match OptionChain::build_chain(&build_params()) {
        Ok(chain) => chain,
        Err(e) => panic!("chain should build: {e}"),
    };
    assert!(!chain.options.is_empty(), "chain should not be empty");

    for option_data in &chain.options {
        let (Some(call), Some(put)) = (
            option_data.greeks_call.as_ref(),
            option_data.greeks_put.as_ref(),
        ) else {
            panic!(
                "strike {} is missing a greek snapshot",
                option_data.strike_price
            );
        };

        // Mirror fields must agree with the snapshots they mirror.
        assert_eq!(Some(call.delta), option_data.delta_call);
        assert_eq!(Some(put.delta), option_data.delta_put);
        assert_eq!(Some(call.gamma), option_data.gamma);

        // Gamma carries no option-style branch, so both styles agree.
        assert_eq!(call.gamma, put.gamma);

        // The snapshots are genuinely per style, not one value copied twice.
        assert_ne!(
            call.charm, put.charm,
            "charm should differ between styles at strike {}",
            option_data.strike_price
        );

        // The optional fields are populated for a live, non-degenerate strike.
        assert!(call.rho.is_some());
        assert!(call.rho_d.is_some());
        assert!(put.rho.is_some());
    }
}

#[test]
fn test_update_greeks_populates_snapshots_on_an_existing_chain() {
    let mut chain = match OptionChain::build_chain(&build_params()) {
        Ok(chain) => chain,
        Err(e) => panic!("chain should build: {e}"),
    };

    // Clear the snapshots, then let the public refresh path put them back.
    chain.mutate_single_options(|option| {
        option.greeks_call = None;
        option.greeks_put = None;
    });
    assert!(chain.options.iter().all(|o| o.greeks_call.is_none()));

    chain.update_greeks();

    assert!(
        chain
            .options
            .iter()
            .all(|o| o.greeks_call.is_some() && o.greeks_put.is_some()),
        "update_greeks should repopulate every strike"
    );
}
