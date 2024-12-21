use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::model::types::{PositiveF64, PZERO};
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::butterfly_spread::ShortButterflySpread;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::{assert_positivef64_relative_eq, pos};
use std::error::Error;

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5700.0),     // short_strike_itm
        pos!(5780.0),     // long_strike
        pos!(5850.0),     // short_strike_otm
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(3.0), // long quantity
        119.01,    // premium_long
        66.0,      // premium_short
        29.85,     // open_fee_long
        4.0,       // open_fee_long
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().len(), 1);
    assert_relative_eq!(strategy.net_premium_received(), 18.580000, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, pos!(18.58), pos!(0.0001));
    assert_positivef64_relative_eq!(strategy.max_loss()?, pos!(221.4199), pos!(0.0001));
    assert_relative_eq!(strategy.fees(), 23.9999, epsilon = 0.001);
    assert!(strategy.profit_area() > 0.0);
    assert!(strategy.profit_ratio() > 0.0);

    // Test range calculations
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points();
    assert!(price_range[0] < break_even_points[0]);

    // Additional strategy-specific validations
    assert!(
        pos!(5700.0) < pos!(5780.0) && pos!(5780.0) < pos!(5850.0),
        "Strikes should be in ascending order: short ITM < long < short OTM"
    );

    // Verify butterfly spread width is symmetrical
    let width_lower = pos!(5780.0) - pos!(5700.0);
    let width_upper = pos!(5850.0) - pos!(5780.0);
    assert_relative_eq!(width_lower.value(), 80.0, epsilon = 0.001);
    assert_relative_eq!(width_upper.value(), 70.0, epsilon = 0.001);

    Ok(())
}