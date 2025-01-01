use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::{assert_positivef64_relative_eq, f2p};
use std::error::Error;
use num_traits::ToPrimitive;

#[test]
fn test_bull_call_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BullCallSpread strategy
    let underlying_price = f2p!(5781.88);

    let strategy = BullCallSpread::new(
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
    assert_eq!(strategy.title(), "Bull Call Spread Strategy:\n\tUnderlying: SP500 @ $5750 Long Call European Option\n\tUnderlying: SP500 @ $5820 Short Call European Option");
    assert_eq!(strategy.get_break_even_points().len(), 1);
    assert_relative_eq!(strategy.net_premium_received().unwrap().to_f64().unwrap(), -116.42, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_loss()?, f2p!(116.42), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), f2p!(229.98), f2p!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 3.02);
    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    assert!(price_range[0] < strategy.get_break_even_points()[0]);
    assert!(price_range[price_range.len() - 1] > strategy.get_break_even_points()[0]);

    Ok(())
}
