use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::constants::ZERO;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::{Positive, assert_pos_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_call_butterfly_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the CallButterfly strategy
    let underlying_price = pos!(5781.88);

    let strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_call_strike
        pos!(5800.0),     // short_call_low_strike
        pos!(5850.0),     // short_call_high_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // long quantity
        pos!(85.04),    // premium_long_itm
        pos!(53.04),    // premium_long_otm
        pos!(28.85),    // premium_short
        pos!(0.78),     // premium_short
        pos!(0.78),     // open_fee_long
        pos!(0.78),     // close_fee_long
        pos!(0.73),     // close_fee_short
        pos!(0.73),     // close_fee_short
        pos!(0.73),     // open_fee_short
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(
        strategy.get_title(),
        "CallButterfly Strategy: \n\tUnderlying: SP500 @ $5800 Short Call European Option\n\tUnderlying: SP500 @ $5750 Long Call European Option\n\tUnderlying: SP500 @ $5850 Short Call European Option"
    );
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        ZERO,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_profit()?, pos!(42.319), pos!(0.0001));
    assert_eq!(strategy.get_max_loss()?, Positive::INFINITY);
    assert_pos_relative_eq!(strategy.get_total_cost()?, pos!(89.57), pos!(0.0001));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 4.53);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let range = strategy.get_range_of_profit().unwrap();
    assert_relative_eq!(range.to_f64(), 134.639, epsilon = 0.001);
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price * 100.0,
        1.164,
        epsilon = 0.001
    );

    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points().unwrap();
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
