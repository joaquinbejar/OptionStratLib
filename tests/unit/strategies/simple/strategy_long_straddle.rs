use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::constants::ZERO;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::long_straddle::LongStraddle;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::{assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;
use positive::pos_or_panic;

#[test]
fn test_long_straddle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongStraddle strategy
    let underlying_price = pos_or_panic!(7008.5);

    let strategy = LongStraddle::new(
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
        pos_or_panic!(7.0),      // open_fee_short_call
        pos_or_panic!(7.01),     // close_fee_short_call
        pos_or_panic!(7.01),     // open_fee_short_put
        pos_or_panic!(7.01),     // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(
        strategy.get_title(),
        "LongStraddle Strategy: \n\tUnderlying: CL @ $7140 Long Call European Option\n\tUnderlying: CL @ $7140 Long Put European Option"
    );
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        ZERO,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_loss()?, pos_or_panic!(465.429), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_total_cost()?, pos_or_panic!(465.4299), pos_or_panic!(0.0001));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 28.03);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(Positive::ONE).unwrap();
    assert!(!price_range.is_empty());
    let break_even_points = strategy.get_break_even_points().unwrap();
    let range = break_even_points[1] - break_even_points[0];
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price.to_f64() * 100.0,
        6.6409,
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

    // Validate that max loss is equal to net premium paid (characteristic of Long Straddle)
    assert_relative_eq!(strategy.get_max_loss()?.to_f64(), 465.4299, epsilon = 0.001);

    Ok(())
}
