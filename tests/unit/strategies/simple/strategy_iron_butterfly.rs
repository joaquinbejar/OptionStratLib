use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::{BreakEvenable, Strategies, Validable};
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::{assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the IronButterfly strategy
    let underlying_price = pos_or_panic!(2646.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(2725.0),     // short_call_strike
        pos_or_panic!(2800.0),     // long_call_strike
        pos_or_panic!(2500.0),     // long_put_strike
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.1548),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::TWO,      // quantity
        pos_or_panic!(38.8),     // premium_short_call
        pos_or_panic!(30.4),     // premium_short_put
        pos_or_panic!(23.3),     // premium_long_call
        pos_or_panic!(16.8),     // premium_long_put
        pos_or_panic!(0.96),     // open_fee
        pos_or_panic!(0.96),     // close_fee
    );

    // Validate strategy
    assert!(strategy.validate(), "Strategy should be valid");

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        42.84,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_profit()?, pos_or_panic!(42.839), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_total_cost()?, pos_or_panic!(95.56), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_fees()?, pos_or_panic!(15.36), pos_or_panic!(0.0001));

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(Positive::ONE).unwrap();
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

    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
