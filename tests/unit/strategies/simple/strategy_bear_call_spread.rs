use num_traits::ToPrimitive;
use optionstratlib::strategies::bear_call_spread::BearCallSpread;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::{pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bear_call_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();
    // Define inputs for the BearCallSpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = BearCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5820.0),     // short_strike
        ExpirationDate::Days(2.0),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // long quantity
        85.04,          // premium_long
        29.85,          // premium_short
        0.78,           // open_fee_long
        0.78,           // open_fee_long
        0.73,           // close_fee_long
        0.73,           // close_fee_short
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.title(), "Bear Call Spread Strategy:\n\tUnderlying: SP500 @ $5750 Short Call European Option\n\tUnderlying: SP500 @ $5820 Long Call European Option");
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
    assert_eq!(
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        104.34
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_eq!(strategy.max_profit()?, pos!(104.34));
    assert_eq!(strategy.max_loss()?, pos!(35.66));
    assert_eq!(strategy.total_cost(), pos!(229.58));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 6.04);
    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    Ok(())
}
