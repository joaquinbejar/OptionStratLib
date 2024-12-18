use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::model::types::{PositiveF64, PZERO};
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::butterfly_spread::LongButterflySpread;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::{assert_positivef64_relative_eq, pos};
use std::error::Error;

#[test]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongButterflySpread strategy
    let underlying_price = pos!(5795.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5710.0),     // long_strike_itm
        pos!(5780.0),     // short_strike
        pos!(5850.0),     // long_strike_otm
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // long quantity
        113.30,    // premium_long_low
        64.20,     // premium_short
        31.65,     // premium_long_high
        0.07,      // fees
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.title(), "LongButterflySpread Strategy on SP500 Size 1:\n\tLong Call Low Strike: $5710\n\tShort Calls Middle Strike: $5780\n\tLong Call High Strike: $5850\n\tExpire: 2024-12-20");
    assert_eq!(strategy.get_break_even_points().len(), 2);
    assert_relative_eq!(strategy.net_premium_received(), -16.736, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, pos!(53.263), pos!(0.0001));
    assert_positivef64_relative_eq!(strategy.max_loss()?, pos!(16.7366), pos!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), pos!(273.3499), pos!(0.0001));
    assert_eq!(strategy.fees(), 0.14);
    assert!(strategy.profit_area() > 0.0);
    assert!(strategy.profit_ratio() > 0.0);

    // Test range calculations
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points();
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
