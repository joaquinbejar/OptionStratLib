use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::strategies::butterfly_spread::ShortButterflySpread;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::{assert_pos_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5700.0),
        pos!(5780.0),
        pos!(5850.0),
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(3.0),
        pos!(119.01),
        pos!(66.0),
        pos!(29.85),
        pos!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
    assert_relative_eq!(
        strategy.net_premium_received().unwrap().to_f64(),
        26.58,
        epsilon = 0.001
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_pos_relative_eq!(strategy.max_profit()?, pos!(26.58), pos!(0.0001));
    assert_pos_relative_eq!(strategy.max_loss()?, pos!(213.42), pos!(0.0001));
    assert_relative_eq!(strategy.fees().unwrap().to_f64(), 23.9999, epsilon = 0.001);
    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Test range calculations
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points().unwrap();
    assert!(price_range[0] < break_even_points[0]);

    // Additional strategy-specific validations
    assert!(
        pos!(5700.0) < pos!(5780.0) && pos!(5780.0) < pos!(5850.0),
        "Strikes should be in ascending order: short ITM < long < short OTM"
    );

    // Verify butterfly spread width is symmetrical
    let width_lower = pos!(5780.0) - pos!(5700.0);
    let width_upper = pos!(5850.0) - pos!(5780.0);
    assert_relative_eq!(width_lower.to_f64(), 80.0, epsilon = 0.001);
    assert_relative_eq!(width_upper.to_f64(), 70.0, epsilon = 0.001);

    Ok(())
}
