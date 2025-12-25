use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::constants::ZERO;
use optionstratlib::strategies::LongStrangle;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::{assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_long_strangle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongStrangle strategy
    let underlying_price = pos_or_panic!(7138.5);

    let strategy = LongStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(7450.0),     // call_strike
        pos_or_panic!(7050.0),     // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos_or_panic!(1.0),      // quantity
        pos_or_panic!(84.2),     // premium_short_call
        pos_or_panic!(353.2),    // premium_short_put
        pos_or_panic!(7.0),      // open_fee_short_call
        pos_or_panic!(7.01),     // close_fee_short_call
        pos_or_panic!(7.01),     // open_fee_short_put
        pos_or_panic!(7.01),     // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(
        strategy.get_title(),
        "LongStrangle Strategy: \n\tUnderlying: CL @ $7450 Long Call European Option\n\tUnderlying: CL @ $7050 Long Put European Option"
    );
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        ZERO,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_loss()?, pos_or_panic!(465.4299), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_total_cost()?, pos_or_panic!(465.4299), pos_or_panic!(0.0001));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 28.03);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(pos_or_panic!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points().unwrap();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price.to_f64() * 100.0,
        9.3217,
        epsilon = 0.001
    );

    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range in relation to break even points
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    // Additional strategy-specific validations
    assert!(
        strategy.get_break_even_points().unwrap()[0] < strategy.get_break_even_points().unwrap()[1],
        "Lower break-even point should be less than upper break-even point"
    );

    // Validate strike prices relationship (characteristic of Long Strangle)
    assert!(
        pos_or_panic!(7050.0) < pos_or_panic!(7450.0),
        "Put strike should be less than call strike in a strangle"
    );

    Ok(())
}
