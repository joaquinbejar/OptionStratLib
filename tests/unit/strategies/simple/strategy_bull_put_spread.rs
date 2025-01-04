use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::strategies::bull_put_spread::BullPutSpread;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::{assert_positivef64_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bull_put_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BullPutSpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = BullPutSpread::new(
        "SP500".to_string(),
        underlying_price,   // underlying_price
        pos!(5750.0),   // long_strike_itm
        pos!(5920.0),   // short_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),   // implied_volatility
        dec!(0.05),   // risk_free_rate
        Positive::ZERO,   // dividend_yield
        pos!(2.0),   // long quantity
        15.04,   // premium_long
        89.85,   // premium_short
        0.78,   // open_fee_long
        0.78,   // open_fee_long
        0.73,   // close_fee_long
        0.73,   // close_fee_short
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.title(), "Bull Put Spread Strategy:\n\tUnderlying: SP500 @ $5750 Long Put European Option\n\tUnderlying: SP500 @ $5920 Short Put European Option");
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
    assert_relative_eq!(
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        143.58,
        epsilon = 0.001
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, pos!(143.58), pos!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), pos!(209.98), pos!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 6.04);
    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range calculations
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    assert!(price_range[0] < strategy.get_break_even_points().unwrap()[0]);
    assert!(price_range[price_range.len() - 1] > strategy.get_break_even_points().unwrap()[0]);

    Ok(())
}
