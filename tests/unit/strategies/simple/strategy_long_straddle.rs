use approx::assert_relative_eq;
use optionstratlib::constants::ZERO;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::straddle::LongStraddle;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::{assert_positivef64_relative_eq, f2p};
use std::error::Error;

#[test]
fn test_long_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongStraddle strategy
    let underlying_price = f2p!(7008.5);

    let strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        f2p!(7140.0),     // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.0,       // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.title(), "Long Straddle Strategy: \n\tUnderlying: CL @ $7140 Long Call European Option\n\tUnderlying: CL @ $7140 Long Put European Option");
    assert_eq!(strategy.get_break_even_points().len(), 2);
    assert_relative_eq!(strategy.net_premium_received(), ZERO, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_loss()?, f2p!(465.429), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), f2p!(465.4299), f2p!(0.0001));
    assert_eq!(strategy.fees(), 28.03);

    // Test range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.value() / 2.0) / underlying_price.value() * 100.0,
        6.6409,
        epsilon = 0.001
    );

    assert!(strategy.profit_area() > 0.0);
    assert!(strategy.profit_ratio() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    // Additional strategy-specific validations
    assert!(
        strategy.get_break_even_points()[0] < strategy.get_break_even_points()[1],
        "Lower break-even point should be less than upper break-even point"
    );

    // Validate that max loss is equal to net premium paid (characteristic of Long Straddle)
    assert_relative_eq!(strategy.max_loss()?.value(), 465.4299, epsilon = 0.001);

    Ok(())
}
