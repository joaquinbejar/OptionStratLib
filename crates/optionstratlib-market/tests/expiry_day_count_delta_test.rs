/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/10/26
******************************************************************************/

//! Pins the deltas of the SP500 sample chain at a day-count expiry.
//!
//! The `option_chain_raw_delta` example set the chain's expiry to tomorrow's
//! calendar date. The time to expiry then depended on the hour the example
//! ran: late in the UTC day it was a few hours, every delta collapsed and the
//! strategy's delta-neutrality step failed with `ZeroDelta`. A day count
//! (`"1"`, parsed as `ExpirationDate::Days`) does not depend on the clock, so
//! the deltas below hold at any time of day.

// Loading the sample chain from JSON needs the `io` feature.
#![cfg(feature = "io")]

use optionstratlib_core::model::Positive;
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::OptionChain;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The deltas go through `f64` (the normal CDF), whose last bits may differ
/// across platform math libraries; `1e-12` is far above that and far below
/// any change a pricing or expiry bug would cause.
const DELTA_TOLERANCE: Decimal = dec!(0.000000000001);

fn sample_chain_one_day() -> OptionChain {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    );
    let mut chain = match OptionChain::load_from_json(path) {
        Ok(chain) => chain,
        Err(error) => panic!("sample chain must load: {error}"),
    };
    chain.update_expiration_date("1".to_string());
    chain
}

fn deltas_at(chain: &OptionChain, strike: Positive) -> (Decimal, Decimal) {
    let option = match chain.options.iter().find(|o| o.strike_price == strike) {
        Some(option) => option,
        None => panic!("strike {strike} must be in the sample chain"),
    };
    match (option.delta_call, option.delta_put) {
        (Some(call), Some(put)) => (call, put),
        other => panic!("strike {strike} must carry both deltas, got {other:?}"),
    }
}

fn assert_close(label: &str, got: Decimal, expected: Decimal) {
    let diff = (got - expected).abs();
    assert!(
        diff <= DELTA_TOLERANCE,
        "{label}: got {got}, expected {expected} (|diff| = {diff})"
    );
}

#[test]
fn test_day_count_expiry_gives_clock_independent_deltas() {
    let chain = sample_chain_one_day();
    let (call, put) = deltas_at(&chain, pos_or_panic!(5780.0));
    // Near the money one day out: about one half, far from the collapsed
    // value a few hours before a calendar expiry gave.
    assert_close("ATM call delta", call, dec!(0.523101873793431));
    assert_close("ATM put delta", put, dec!(-0.476898126206569));
    // Put-call parity on deltas with no dividend yield: call - put = 1.
    assert_close("call - put", call - put, Decimal::ONE);
}
