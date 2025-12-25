use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::{ShortButterflySpread, Strategies};
use optionstratlib::{assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;
use positive::pos_or_panic;

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos_or_panic!(5700.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(3.0),
        pos_or_panic!(119.01),
        pos_or_panic!(66.0),
        pos_or_panic!(29.85),
        pos_or_panic!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        26.58,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_profit()?, pos_or_panic!(26.58), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_max_loss()?, pos_or_panic!(213.42), pos_or_panic!(0.0001));
    assert_relative_eq!(
        strategy.get_fees().unwrap().to_f64(),
        23.9999,
        epsilon = 0.001
    );
    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(Positive::ONE).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points().unwrap();
    assert!(price_range[0] < break_even_points[0]);

    // Additional strategy-specific validations
    assert!(
        pos_or_panic!(5700.0) < pos_or_panic!(5780.0) && pos_or_panic!(5780.0) < pos_or_panic!(5850.0),
        "Strikes should be in ascending order: short ITM < long < short OTM"
    );

    // Verify butterfly spread width is symmetrical
    let width_lower = pos_or_panic!(5780.0) - pos_or_panic!(5700.0);
    let width_upper = pos_or_panic!(5850.0) - pos_or_panic!(5780.0);
    assert_relative_eq!(width_lower.to_f64(), 80.0, epsilon = 0.001);
    assert_relative_eq!(width_upper.to_f64(), 70.0, epsilon = 0.001);

    Ok(())
}
