use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::strategies::base::{Strategies, Validable};
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::utils::setup_logger;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::{assert_positivef64_relative_eq, f2p};
use std::error::Error;

#[test]
fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronButterfly strategy
    let underlying_price = f2p!(2646.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        f2p!(2725.0),     // short_call_strike
        f2p!(2800.0),     // long_call_strike
        f2p!(2500.0),     // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(2.0), // quantity
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
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        42.839,
        epsilon = 0.001
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, f2p!(42.839), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), f2p!(218.599), f2p!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 7.68);

    // Test range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points().unwrap();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price.to_f64() * 100.0,
        0.809,
        epsilon = 0.001
    );

    assert_eq!(
        price_range[..4],
        vec![2443.924, 2444.924, 2445.924, 2446.924]
    );
    assert_relative_eq!(range.to_f64(), 42.84, epsilon = 0.001);

    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
