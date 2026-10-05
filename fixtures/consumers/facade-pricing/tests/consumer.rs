//! A downstream pricing and volatility workflow on the `optionstratlib`
//! facade with `default-features = false, features = ["pricing"]` (#528):
//! the convenience `prelude` for the common path, the canonical module paths
//! for the rest, and proof that both name the items the component crates
//! define. No market, I/O, async or charting code is compiled.

use optionstratlib::prelude::*;

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

/// Hull's Black-Scholes example: S = 42, K = 40, r = 10%, σ = 20%, T = 0.5.
fn hull_option(style: OptionStyle) -> Options {
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
fn test_prelude_prices_hull_example() {
    let call = black_scholes(&hull_option(OptionStyle::Call));
    let put = black_scholes(&hull_option(OptionStyle::Put));
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
fn test_greeks_through_the_facade() {
    let call = hull_option(OptionStyle::Call);
    let put = hull_option(OptionStyle::Put);
    match (call.delta(), put.delta(), call.gamma(), put.gamma()) {
        (Ok(call_delta), Ok(put_delta), Ok(call_gamma), Ok(put_gamma)) => {
            assert!(close(call_delta - put_delta, Decimal::ONE, dec!(0.000001)));
            assert!(close(call_gamma, put_gamma, dec!(0.000001)));
        }
        other => panic!("Greeks failed: {other:?}"),
    }
}

#[test]
fn test_implied_volatility_round_trip_through_the_facade() {
    let mut call = hull_option(OptionStyle::Call);
    let price = match optionstratlib::pricing::black_scholes(&call) {
        Ok(price) => price,
        Err(error) => panic!("price: {error}"),
    };
    let market_price = match Positive::new_decimal(price) {
        Ok(price) => price,
        Err(error) => panic!("price is not positive: {error}"),
    };
    match optionstratlib::volatility::implied_volatility(market_price, &mut call, 100) {
        Ok(volatility) => assert!(
            close(volatility.to_dec(), dec!(0.2), dec!(0.001)),
            "recovered {volatility}"
        ),
        Err(error) => panic!("implied volatility: {error}"),
    }
}

#[test]
fn test_facade_paths_are_the_component_items() {
    same_item(
        optionstratlib::pricing::black_scholes,
        optionstratlib_pricing::pricing::black_scholes,
    );
    same_item(
        optionstratlib::greeks::delta,
        optionstratlib_pricing::greeks::delta,
    );
    same_item(
        optionstratlib::volatility::implied_volatility,
        optionstratlib_pricing::volatility::implied_volatility,
    );
    let option: optionstratlib_core::model::Options = hull_option(OptionStyle::Call);
    let error: optionstratlib_pricing::error::PricingError =
        optionstratlib::error::PricingError::other("probe");
    assert_eq!(option.strike_price, pos_or_panic!(40.0));
    assert!(error.to_string().contains("probe"));
}
