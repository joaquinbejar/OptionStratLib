/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 28/8/26
******************************************************************************/

//! Verifies that a chain quoted with a spread wider than its cheap strikes'
//! mids keeps those strikes.
//!
//! `apply_spread` used to withdraw a quote whose mid was below one full
//! spread, clearing bid, ask and mid alike. `build_chain` stops generating
//! strikes once both wings come back without prices, so a wide spread
//! truncated the chain: the cheap out-of-the-money wings — the long legs of
//! every defined-risk structure — disappeared exactly as they decayed.

use optionstratlib::ExpirationDate;
use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use positive::{Positive, pos_or_panic, spos};
use rust_decimal_macros::dec;

const CHAIN_SIZE: usize = 10;

fn build_params(spread: Positive) -> OptionChainBuildParams {
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
        CHAIN_SIZE,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.1),
        spread,
        2,
        price_params,
        pos_or_panic!(0.2),
    )
}

fn build(spread: Positive) -> OptionChain {
    match OptionChain::build_chain(&build_params(spread)) {
        Ok(chain) => chain,
        Err(e) => panic!("chain should build with a spread of {spread}: {e}"),
    }
}

#[test]
fn test_wide_spread_keeps_every_strike() {
    let tight = build(pos_or_panic!(0.02));
    let wide = build(pos_or_panic!(2.0));

    // Absolute, not relative: `chain_size` is a per-side half-width, so a chain
    // that keeps every strike holds the ATM plus `CHAIN_SIZE` on each side. A
    // relative assertion would pass just as happily on two truncated chains.
    assert_eq!(
        tight.options.len(),
        2 * CHAIN_SIZE + 1,
        "a tightly quoted chain holds the full grid"
    );
    assert_eq!(
        wide.options.len(),
        2 * CHAIN_SIZE + 1,
        "a spread wider than the cheap strikes' mids must not truncate the chain"
    );
}

#[test]
fn test_wide_spread_leaves_every_strike_quoted() {
    let chain = build(pos_or_panic!(2.0));
    let tick = pos_or_panic!(0.01);

    for option_data in &chain.options {
        let strike = option_data.strike_price;
        let (Some(call_bid), Some(call_ask)) = (option_data.call_bid, option_data.call_ask) else {
            panic!("strike {strike} lost its call quote");
        };
        let (Some(put_bid), Some(put_ask)) = (option_data.put_bid, option_data.put_ask) else {
            panic!("strike {strike} lost its put quote");
        };

        assert!(call_bid >= tick, "call bid at strike {strike} below a tick");
        assert!(
            call_bid <= call_ask,
            "crossed call quote at strike {strike}"
        );
        assert!(put_bid >= tick, "put bid at strike {strike} below a tick");
        assert!(put_bid <= put_ask, "crossed put quote at strike {strike}");
        assert!(
            option_data.call_middle.is_some() && option_data.put_middle.is_some(),
            "strike {strike} lost a mid it was given"
        );
    }
}
