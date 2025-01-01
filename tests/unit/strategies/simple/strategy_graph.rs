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
fn test_bull_call_spread_basic_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let strategy = BullCallSpread::new(
        "GOLD".to_string(),
        f2p!(2505.8), // underlying_price
        f2p!(2460.0), // long_strike_itm
        f2p!(2515.0), // short_strike
        ExpirationDate::Days(30.0),
        0.2,       // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // quantity
        27.26,     // premium_long
        5.33,      // premium_short
        0.58,      // open_fee_long
        0.58,      // close_fee_long
        0.55,      // close_fee_short
        0.54,      // open_fee_short
    );

    // Validate strategy properties
    assert_eq!(strategy.title(), "Bull Call Spread Strategy:\n\tUnderlying: GOLD @ $2460 Long Call European Option\n\tUnderlying: GOLD @ $2515 Short Call European Option");
    assert_eq!(strategy.get_break_even_points().len(), 1);

    // Validate financial calculations
    assert_relative_eq!(strategy.net_premium_received().unwrap().to_f64().unwrap(), -24.18, epsilon = 0.001);
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, f2p!(30.82), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.max_loss()?, f2p!(24.18), f2p!(0.0001));
    assert_positivef64_relative_eq!(strategy.total_cost(), f2p!(32.66), f2p!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 2.25);

    // Test price range calculations
    let test_price_range: Vec<Positive> = (2400..2600)
        .map(|x| Positive::new(x as f64).unwrap())
        .collect();
    assert!(!test_price_range.is_empty());
    assert_eq!(test_price_range.len(), 200);

    // Validate strike prices relationship
    assert!(
        f2p!(2460.0) < f2p!(2515.0),
        "Long strike should be less than short strike in a bull call spread"
    );

    // Validate break-even point
    let break_even = strategy.break_even();
    assert!(
        break_even[0] > f2p!(2460.0),
        "Break-even should be between strikes"
    );

    Ok(())
}
