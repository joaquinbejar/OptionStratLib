use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::strangle::ShortStrangle;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::{assert_positivef64_relative_eq, f2p};
use std::error::Error;

#[test]
fn test_short_strangle_with_greeks_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortStrangle strategy
    let underlying_price = f2p!(7138.5);

    let strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        f2p!(7450.0),     // call_strike
        f2p!(7050.0),     // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.01,      // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().len(), 2);
    assert_relative_eq!(strategy.net_premium_received(), 409.36, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, f2p!(409.36), f2p!(0.0001));
    assert_eq!(strategy.fees(), 28.04);

    // Test range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price.to_f64() * 100.0,
        8.53624,
        epsilon = 0.001
    );

    assert!(strategy.profit_area() > 0.0);
    assert!(strategy.profit_ratio() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    // Additional strategy-specific validations
    assert!(
        f2p!(7050.0) < f2p!(7450.0),
        "Put strike should be less than call strike in a short strangle"
    );

    Ok(())
}
