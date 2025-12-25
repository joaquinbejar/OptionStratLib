use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::{ShortStraddle, Strategies};
use optionstratlib::{assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;
use positive::pos_or_panic;

#[test]
fn test_short_straddle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortStraddle strategy
    let underlying_price = pos_or_panic!(7138.5);

    let strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(7140.0),     // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,      // quantity
        pos_or_panic!(84.2),     // premium_short_call
        pos_or_panic!(353.2),    // premium_short_put
        pos_or_panic!(7.01),     // open_fee_short_call
        pos_or_panic!(7.01),     // close_fee_short_call
        pos_or_panic!(7.01),     // open_fee_short_put
        pos_or_panic!(7.01),     // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        409.36,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_profit()?, pos_or_panic!(409.36), pos_or_panic!(0.0001));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 28.04);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(Positive::ONE).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points().unwrap();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price.to_f64() * 100.0,
        5.7345,
        epsilon = 0.001
    );

    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

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
        strategy.get_max_profit()?.to_f64(),
        strategy.get_net_premium_received().unwrap().to_f64(),
        epsilon = 0.001
    );

    Ok(())
}
