use num_traits::ToPrimitive;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::{assert_positivef64_relative_eq, f2p};
use std::error::Error;

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = f2p!(2703.3);

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),          // underlying_symbol
        underlying_price,            // underlying_price
        f2p!(2600.0),                // long_call_strike
        f2p!(2800.0),                // short_call_strike OTM
        ExpirationDate::Days(120.0), // long_call_expiration
        ExpirationDate::Days(30.0),  // short_call_expiration 30-45 days delta 0.30 or less
        0.17,                        // implied_volatility
        0.05,                        // risk_free_rate
        0.0,                         // dividend_yield
        f2p!(2.0),                   // quantity
        154.7,                       // premium_short_call
        30.8,                        // premium_short_put
        1.74,                        // open_fee_short_call
        1.74,                        // close_fee_short_call
        0.85,                        // open_fee_short_put
        0.85,                        // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().len(), 1);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, f2p!(141.8399), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.max_loss()?, f2p!(258.16), f2p!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 10.36);
    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Test range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points();
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[0]);

    // Additional strategy-specific validations
    assert!(
        f2p!(2600.0) < f2p!(2800.0),
        "Long call strike should be less than short call strike in a PMCC"
    );

    Ok(())
}
