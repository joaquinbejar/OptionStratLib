use num_traits::ToPrimitive;
use optionstratlib::f2p;
use optionstratlib::strategies::bear_call_spread::BearCallSpread;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use std::error::Error;

#[test]
fn test_bear_call_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();
    // Define inputs for the BearCallSpread strategy
    let underlying_price = f2p!(5781.88);

    let strategy = BearCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        f2p!(5750.0),     // long_strike_itm
        f2p!(5820.0),     // short_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(2.0), // long quantity
        85.04,     // premium_long
        29.85,     // premium_short
        0.78,      // open_fee_long
        0.78,      // open_fee_long
        0.73,      // close_fee_long
        0.73,      // close_fee_short
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.title(), "Bear Call Spread Strategy:\n\tUnderlying: SP500 @ $5750 Short Call European Option\n\tUnderlying: SP500 @ $5820 Long Call European Option");
    assert_eq!(strategy.get_break_even_points().len(), 1);
    assert_eq!(
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        104.34
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_eq!(strategy.max_profit()?, f2p!(104.34));
    assert_eq!(strategy.max_loss()?, f2p!(35.66));
    assert_eq!(strategy.total_cost(), f2p!(229.58));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 3.02);
    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    Ok(())
}
