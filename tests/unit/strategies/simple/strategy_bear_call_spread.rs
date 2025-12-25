use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::bear_call_spread::BearCallSpread;
use optionstratlib::strategies::{BasicAble, Strategies};
use optionstratlib::{Positive, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bear_call_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the BearCallSpread strategy
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = BearCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(5750.0),     // long_strike_itm
        pos_or_panic!(5820.0),     // short_strike
        ExpirationDate::Days(pos_or_panic!(2.0)),
        pos_or_panic!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos_or_panic!(2.0),      // long quantity
        pos_or_panic!(85.04),    // premium_long
        pos_or_panic!(29.85),    // premium_short
        pos_or_panic!(0.78),     // open_fee_long
        pos_or_panic!(0.78),     // open_fee_long
        pos_or_panic!(0.73),     // close_fee_long
        pos_or_panic!(0.73),     // close_fee_short
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(
        strategy.get_title(),
        "BearCallSpread Strategy: \n\tUnderlying: SP500 @ $5750 Short Call European Option\n\tUnderlying: SP500 @ $5820 Long Call European Option"
    );
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
    assert_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        104.34
    );
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_eq!(strategy.get_max_profit()?, pos_or_panic!(104.34));
    assert_eq!(strategy.get_max_loss()?, pos_or_panic!(35.66));
    assert_eq!(strategy.get_total_cost()?, pos_or_panic!(65.74));
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 6.04);
    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    Ok(())
}
