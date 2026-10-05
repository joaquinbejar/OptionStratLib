//! A downstream pricing workflow built on `optionstratlib-core` and
//! `optionstratlib-pricing` alone (#527): build a contract, price it in
//! closed form, read its Greeks and payoff, and recover its implied
//! volatility from the price. Every value crosses between the two crates as
//! the same canonical type; nothing is converted or wrapped.

use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::{OptionPricing, black_scholes};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Hull's Black-Scholes example: S = 42, K = 40, r = 10%, σ = 20%, T = 0.5.
fn hull_call(style: OptionStyle) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(40.0),
        ExpirationDate::Days(pos_or_panic!(182.5)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(42.0),
        dec!(0.10),
        style,
        Positive::ZERO,
        None,
    )
}

fn close(value: Decimal, expected: Decimal, tolerance: Decimal) -> bool {
    (value - expected).abs() <= tolerance
}

#[test]
fn test_closed_form_price_matches_hull() {
    let call = black_scholes(&hull_call(OptionStyle::Call));
    let put = black_scholes(&hull_call(OptionStyle::Put));
    assert!(
        matches!(call, Ok(price) if close(price, dec!(4.76), dec!(0.005))),
        "{call:?}"
    );
    assert!(
        matches!(put, Ok(price) if close(price, dec!(0.81), dec!(0.005))),
        "{put:?}"
    );
}

#[test]
fn test_greeks_of_a_core_option() {
    let call = hull_call(OptionStyle::Call);
    let put = hull_call(OptionStyle::Put);
    match (call.delta(), put.delta(), call.gamma(), put.gamma()) {
        (Ok(call_delta), Ok(put_delta), Ok(call_gamma), Ok(put_gamma)) => {
            assert!(close(call_delta - put_delta, Decimal::ONE, dec!(0.000001)));
            assert!(close(call_gamma, put_gamma, dec!(0.000001)));
            assert!(call_delta > Decimal::ZERO && call_delta < Decimal::ONE);
        }
        other => panic!("Greeks failed: {other:?}"),
    }
}

#[test]
fn test_payoff_at_expiry_from_core() {
    let call = hull_call(OptionStyle::Call);
    assert!(matches!(call.payoff(), Ok(value) if value == dec!(2)));
    assert!(
        matches!(call.payoff_at_price(&pos_or_panic!(35.0)), Ok(value) if value == Decimal::ZERO)
    );
}

#[test]
fn test_implied_volatility_round_trip() {
    let call = hull_call(OptionStyle::Call);
    let price = match black_scholes(&call) {
        Ok(price) => price,
        Err(error) => panic!("price: {error}"),
    };
    match call.calculate_implied_volatility(price) {
        Ok(volatility) => assert!(
            close(volatility.to_dec(), dec!(0.2), dec!(0.001)),
            "recovered {volatility}"
        ),
        Err(error) => panic!("implied volatility: {error}"),
    }
}
