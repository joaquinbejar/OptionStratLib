//! `OptionChain` keeps implementing the pricing-owned `AtmIvProvider` after
//! the implementation moved from `volatility` to `chains` (multi-crate
//! roadmap M1-13, #510): the trait stays generic in pricing and the impl
//! lives with the market-owned type.

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::error::VolatilityError;
use optionstratlib::volatility::AtmIvProvider;
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;

#[test]
fn test_option_chain_provides_atm_iv() {
    let mut chain = OptionChain::new(
        "TEST",
        Positive::HUNDRED,
        "2025-12-31".to_string(),
        Some(dec!(0.05)),
        None,
    );
    chain.add_option(
        Positive::HUNDRED,
        Some(pos_or_panic!(5.0)),
        Some(pos_or_panic!(5.5)),
        Some(pos_or_panic!(4.5)),
        Some(pos_or_panic!(5.0)),
        pos_or_panic!(0.25),
        Some(dec!(0.5)),
        Some(dec!(-0.5)),
        Some(dec!(0.05)),
        None,
        None,
        None,
    );

    let iv = chain.atm_iv();
    assert!(
        iv.is_ok(),
        "ATM IV must resolve on a chain with an ATM strike: {iv:?}"
    );
    assert_eq!(iv.ok().copied(), Some(pos_or_panic!(0.25)));
}

#[test]
fn test_empty_chain_maps_chain_error_into_atm_iv_unavailable() {
    let chain = OptionChain::new(
        "TEST",
        Positive::HUNDRED,
        "2025-12-31".to_string(),
        None,
        None,
    );
    match chain.atm_iv() {
        Err(VolatilityError::AtmIvUnavailable { .. }) => {}
        other => panic!("expected AtmIvUnavailable, got {other:?}"),
    }
}
