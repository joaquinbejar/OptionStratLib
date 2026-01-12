//! Property-based tests for put-call parity
//!
//! Put-call parity is a fundamental relationship in options pricing:
//! C - P = S - K * e^(-rT)
//!
//! Where:
//! - C = Call option price
//! - P = Put option price
//! - S = Spot price
//! - K = Strike price
//! - r = Risk-free rate
//! - T = Time to expiration

use optionstratlib::model::ExpirationDate;
use optionstratlib::model::Options;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::black_scholes;
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::MathematicalOps;
use rust_decimal_macros::dec;

/// Creates a call option with the given parameters
fn create_call_option(
    spot: Positive,
    strike: Positive,
    volatility: Positive,
    days_to_expiry: u32,
    risk_free_rate: Decimal,
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
        risk_free_rate,
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
    risk_free_rate: Decimal,
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
        risk_free_rate,
        OptionStyle::Put,
        Positive::ZERO,
        None,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test put-call parity: C - P = S - K * e^(-rT)
    ///
    /// This is a fundamental relationship that must hold for European options.
    #[test]
    fn test_put_call_parity(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
        rate in 0.01f64..0.10,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();
        let risk_free_rate = Decimal::from_f64_retain(rate).unwrap();

        let call = create_call_option(spot, strike, volatility, days, risk_free_rate);
        let put = create_put_option(spot, strike, volatility, days, risk_free_rate);

        let call_price = black_scholes(&call);
        let put_price = black_scholes(&put);

        // Skip if pricing fails (edge cases)
        if call_price.is_err() || put_price.is_err() {
            return Ok(());
        }

        let call_price = call_price.unwrap();
        let put_price = put_price.unwrap();

        // Calculate time to expiry in years
        let t = Decimal::from(days) / dec!(365);

        // Put-call parity: C - P = S - K * e^(-rT)
        let parity_lhs = call_price - put_price;
        let discount_factor = (-risk_free_rate * t).exp();
        let parity_rhs = spot.to_dec() - strike.to_dec() * discount_factor;

        let diff = (parity_lhs - parity_rhs).abs();

        // Allow for small numerical errors (0.01 tolerance)
        prop_assert!(
            diff < dec!(0.01),
            "Put-call parity violated: C({}) - P({}) = {} != S({}) - K*e^(-rT) = {}, diff = {}",
            call_price, put_price, parity_lhs, spot, parity_rhs, diff
        );
    }

    /// Test that call price is always non-negative
    #[test]
    fn test_call_price_non_negative(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call = create_call_option(spot, strike, volatility, days, dec!(0.05));

        if let Ok(price) = black_scholes(&call) {
            // Allow for tiny numerical errors (1e-10 tolerance)
            prop_assert!(
                price >= dec!(-0.0000000001),
                "Call price should be non-negative (within tolerance), got {}",
                price
            );
        }
    }

    /// Test that put price is always non-negative
    #[test]
    fn test_put_price_non_negative(
        spot in 50.0f64..500.0,
        strike in 50.0f64..500.0,
        volatility in 0.1f64..0.8,
        days in 7u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let put = create_put_option(spot, strike, volatility, days, dec!(0.05));

        if let Ok(price) = black_scholes(&put) {
            // Allow for tiny numerical errors (1e-10 tolerance)
            prop_assert!(
                price >= dec!(-0.0000000001),
                "Put price should be non-negative (within tolerance), got {}",
                price
            );
        }
    }

    /// Test that call price increases with spot price (positive delta)
    #[test]
    fn test_call_price_increases_with_spot(
        spot in 100.0f64..400.0,
        strike in 100.0f64..400.0,
        volatility in 0.1f64..0.8,
        days in 30u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let spot_higher = Positive::new(spot.to_f64() + 10.0).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let call_low = create_call_option(spot, strike, volatility, days, dec!(0.05));
        let call_high = create_call_option(spot_higher, strike, volatility, days, dec!(0.05));

        if let (Ok(price_low), Ok(price_high)) = (black_scholes(&call_low), black_scholes(&call_high)) {
            prop_assert!(
                price_high >= price_low - dec!(0.0000000001),
                "Call price should increase with spot: {} >= {} (spot {} vs {})",
                price_high, price_low, spot_higher, spot
            );
        }
    }

    /// Test that put price decreases with spot price (negative delta)
    #[test]
    fn test_put_price_decreases_with_spot(
        spot in 100.0f64..400.0,
        strike in 100.0f64..400.0,
        volatility in 0.1f64..0.8,
        days in 30u32..365,
    ) {
        let spot = Positive::new(spot).unwrap();
        let spot_higher = Positive::new(spot.to_f64() + 10.0).unwrap();
        let strike = Positive::new(strike).unwrap();
        let volatility = Positive::new(volatility).unwrap();

        let put_low = create_put_option(spot, strike, volatility, days, dec!(0.05));
        let put_high = create_put_option(spot_higher, strike, volatility, days, dec!(0.05));

        if let (Ok(price_low), Ok(price_high)) = (black_scholes(&put_low), black_scholes(&put_high)) {
            prop_assert!(
                price_low >= price_high - dec!(0.0000000001),
                "Put price should decrease with spot: {} >= {} (spot {} vs {})",
                price_low, price_high, spot, spot_higher
            );
        }
    }
}
