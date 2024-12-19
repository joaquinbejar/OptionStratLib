use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::model::types::{PositiveF64, PZERO};
use optionstratlib::strategies::base::{Strategies, Validable};
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::{assert_positivef64_relative_eq, pos};
use std::error::Error;

#[test]
fn test_iron_condor_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronCondor strategy
    let underlying_price = pos!(2646.9);

    let strategy = IronCondor::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos!(2725.0),     // short_call_strike
        pos!(2560.0),     // short_put_strike
        pos!(2800.0),     // long_call_strike
        pos!(2500.0),     // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(2.0), // quantity
        38.8,      // premium_short_call
        30.4,      // premium_short_put
        23.3,      // premium_long_call
        16.8,      // premium_long_put
        0.96,      // open_fee
        0.96,      // close_fee
    );

    // Validate strategy
    assert!(strategy.validate(), "Strategy should be valid");

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().len(), 2);
    assert_relative_eq!(strategy.net_premium_received(), 42.839, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, pos!(42.839), pos!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), pos!(218.5999), pos!(0.0001));
    assert_eq!(strategy.fees(), 7.68);

    // Test range calculations
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.value() / 2.0) / underlying_price.value() * 100.0,
        3.926,
        epsilon = 0.001
    );

    assert!(strategy.profit_area() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
