use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::Positive;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::bear_put_spread::BearPutSpread;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::{assert_positivef64_relative_eq, f2p};
use std::error::Error;

#[test]
fn test_bear_put_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BearPutSpread strategy
    let underlying_price = f2p!(5781.88);

    let strategy = BearPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        f2p!(5850.0),     // long_strike
        f2p!(5720.0),     // short_strike
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
    assert_eq!(strategy.title(), "Bear Put Spread Strategy:\n\tUnderlying: SP500 @ $5850 Long Put European Option\n\tUnderlying: SP500 @ $5720 Short Put European Option");
    assert_eq!(strategy.get_break_even_points().len(), 1);
    assert_relative_eq!(strategy.net_premium_received(), 116.42, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_loss()?, f2p!(116.42), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), f2p!(116.42), f2p!(0.0001));
    assert_eq!(strategy.fees(), 3.02);
    assert!(strategy.profit_area() > 0.0);
    assert!(strategy.profit_ratio() > 0.0);

    // Validate price range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    assert!(price_range[0] < strategy.get_break_even_points()[0]);
    assert!(price_range[price_range.len() - 1] > strategy.get_break_even_points()[0]);

    Ok(())
}
