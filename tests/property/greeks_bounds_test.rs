//! Property-based tests for Greeks bounds
//!
//! This module tests that the Greeks (delta, gamma, theta, vega, rho)
//! stay within their theoretical bounds across a wide range of inputs.

use optionstratlib::greeks::Greeks;
use optionstratlib::model::ExpirationDate;
use optionstratlib::model::Options;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use positive::Positive;
use proptest::prelude::*;
use rust_decimal_macros::dec;

/// Creates a call option with the given parameters
fn create_call_option(
    spot: Positive,
    strike: Positive,
    volatility: Positive,
    days_to_expiry: u32,
) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "TEST".to_string(),
        strike,
        ExpirationDate::Days(Positive::new(days_to_expiry as f64).unwrap()),
        volatility,
        Positive::ONE,
        spot,
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

/// Creates a put option with the given parameters
fn create_put_option(
    spot: Positive,
    strike: Positive,
    volatility: Positive,
    days_to_expiry: u32,
) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "TEST".to_string(),
        strike,
        ExpirationDate::Days(Positive::new(days_to_expiry as f64).unwrap()),
        volatility,
        Positive::ONE,
        spot,
        dec!(0.05),
        OptionStyle::Put,
        Positive::ZERO,
        None,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test that call delta is between 0 and 1
    #[test]
    fn test_call_delta_bounds(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call = create_call_option(spot, strike, volatility, days);

        if let Ok(delta) = call.delta() {
            prop_assert!(
                delta >= dec!(0.0) && delta <= dec!(1.0),
                "Call delta should be in [0, 1], got {}",
                delta
            );
        }
    }

    /// Test that put delta is between -1 and 0
    #[test]
    fn test_put_delta_bounds(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let put = create_put_option(spot, strike, volatility, days);

        if let Ok(delta) = put.delta() {
            prop_assert!(
                delta >= dec!(-1.0) && delta <= dec!(0.0),
                "Put delta should be in [-1, 0], got {}",
                delta
            );
        }
    }

    /// Test that gamma is always non-negative
    #[test]
    fn test_gamma_non_negative(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call = create_call_option(spot, strike, volatility, days);

        if let Ok(gamma) = call.gamma() {
            prop_assert!(
                gamma >= dec!(0.0),
                "Gamma should be non-negative, got {}",
                gamma
            );
        }
    }

    /// Test that vega is always non-negative for long options
    #[test]
    fn test_vega_non_negative(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call = create_call_option(spot, strike, volatility, days);

        if let Ok(vega) = call.vega() {
            prop_assert!(
                vega >= dec!(0.0),
                "Vega should be non-negative for long options, got {}",
                vega
            );
        }
    }

    /// Test that call and put have the same gamma
    #[test]
    fn test_call_put_same_gamma(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call = create_call_option(spot, strike, volatility, days);
        let put = create_put_option(spot, strike, volatility, days);

        if let (Ok(call_gamma), Ok(put_gamma)) = (call.gamma(), put.gamma()) {
            let diff = (call_gamma - put_gamma).abs();
            prop_assert!(
                diff < dec!(0.0001),
                "Call and put should have same gamma: {} vs {}, diff = {}",
                call_gamma, put_gamma, diff
            );
        }
    }

    /// Test that call and put have the same vega
    #[test]
    fn test_call_put_same_vega(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call = create_call_option(spot, strike, volatility, days);
        let put = create_put_option(spot, strike, volatility, days);

        if let (Ok(call_vega), Ok(put_vega)) = (call.vega(), put.vega()) {
            let diff = (call_vega - put_vega).abs();
            prop_assert!(
                diff < dec!(0.0001),
                "Call and put should have same vega: {} vs {}, diff = {}",
                call_vega, put_vega, diff
            );
        }
    }

    /// Test delta-gamma relationship: delta changes correctly with spot
    #[test]
    fn test_delta_gamma_relationship(
        spot in 100.0f64..400.0,
        strike in 100.0f64..400.0,
        volatility in 0.1f64..0.8,
        days in 30u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let spot_higher = Positive::new(spot.to_f64() + 1.0).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call_low = create_call_option(spot, strike, volatility, days);
        let call_high = create_call_option(spot_higher, strike, volatility, days);

        if let (Ok(delta_low), Ok(delta_high), Ok(gamma)) =
            (call_low.delta(), call_high.delta(), call_low.gamma())
        {
            // Delta change should be approximately gamma * spot_change
            let delta_change = delta_high - delta_low;
            let expected_change = gamma; // spot change is 1.0

            // Allow for some numerical error (gamma is instantaneous, we use finite difference)
            let diff = (delta_change - expected_change).abs();
            prop_assert!(
                diff < dec!(0.1),
                "Delta change ({}) should be close to gamma ({}), diff = {}",
                delta_change, expected_change, diff
            );
        }
    }
}
