use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::strategies::straddle::ShortStraddle;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::{assert_positivef64_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortStraddle strategy
    let underlying_price = pos!(7138.5);

    let strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7140.0),     // put_strike
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        84.2,           // premium_short_call
        353.2,          // premium_short_put
        7.01,           // open_fee_short_call
        7.01,           // close_fee_short_call
        7.01,           // open_fee_short_put
        7.01,           // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        409.36,
        epsilon = 0.001
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, pos!(409.36), pos!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 28.04);

    // Test range calculations
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points().unwrap();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price.to_f64() * 100.0,
        5.7345,
        epsilon = 0.001
    );

    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    // Additional strategy-specific validations
    assert!(
        break_even_points[0] < break_even_points[1],
        "Lower break-even point should be less than upper break-even point"
    );

    // Validate that max profit equals net premium received (characteristic of Short Straddle)
    assert_relative_eq!(
        strategy.max_profit()?.to_f64(),
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        epsilon = 0.001
    );

    Ok(())
}
