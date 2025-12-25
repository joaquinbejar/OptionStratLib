use approx::assert_relative_eq;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::{assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bull_call_spread_basic_integration() -> Result<(), Box<dyn Error>> {
    let strategy = BullCallSpread::new(
        "GOLD".to_string(),
        pos_or_panic!(2505.8), // underlying_price
        pos_or_panic!(2460.0), // long_strike_itm
        pos_or_panic!(2515.0), // short_strike
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),      // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos_or_panic!(1.0),      // quantity
        pos_or_panic!(27.26),    // premium_long
        pos_or_panic!(5.33),     // premium_short
        pos_or_panic!(0.58),     // open_fee_long
        pos_or_panic!(0.58),     // close_fee_long
        pos_or_panic!(0.55),     // close_fee_short
        pos_or_panic!(0.54),     // open_fee_short
    );

    // Validate strategy properties
    assert_eq!(
        strategy.get_title(),
        "BullCallSpread Strategy: \n\tUnderlying: GOLD @ $2515 Short Call European Option\n\tUnderlying: GOLD @ $2460 Long Call European Option"
    );
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);

    // Validate financial calculations
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        0.0,
        epsilon = 0.001
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(strategy.get_max_profit()?, pos_or_panic!(30.82), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_max_loss()?, pos_or_panic!(24.18), pos_or_panic!(0.0001));
    assert_pos_relative_eq!(strategy.get_total_cost()?, pos_or_panic!(29.51), pos_or_panic!(0.0001));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 2.25);

    // Test price range calculations
    let test_price_range: Vec<Positive> = (2400..2600)
        .map(|x| Positive::new(x as f64).unwrap())
        .collect();
    assert!(!test_price_range.is_empty());
    assert_eq!(test_price_range.len(), 200);

    // Validate strike prices relationship
    assert!(
        pos_or_panic!(2460.0) < pos_or_panic!(2515.0),
        "Long strike should be less than short strike in a bull call spread"
    );

    // Validate break-even point
    let break_even = strategy.get_break_even_points().unwrap();
    assert!(
        break_even[0] > pos_or_panic!(2460.0),
        "Break-even should be between strikes"
    );

    Ok(())
}
