use positive::{assert_pos_relative_eq, pos_or_panic};
use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::Strategies;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::long_butterfly_spread::LongButterflySpread;

use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongButterflySpread strategy
    let underlying_price = pos_or_panic!(5795.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos_or_panic!(5710.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(113.3),
        pos_or_panic!(64.20),
        pos_or_panic!(31.65),
        pos_or_panic!(0.07),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        0.0,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_profit()?, pos_or_panic!(53.31), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_max_loss()?, pos_or_panic!(16.68999), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_total_cost()?, pos_or_panic!(145.09), pos_or_panic!(0.0001));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 0.14);
    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(Positive::ONE).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points().unwrap();
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
