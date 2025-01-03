use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::constants::ZERO;
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::Strategies;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::{assert_positivef64_relative_eq, f2p, Positive};
use std::error::Error;

#[test]
fn test_call_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the CallButterfly strategy
    let underlying_price = f2p!(5781.88);

    let strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        f2p!(5750.0),     // long_call_strike
        f2p!(5800.0),     // short_call_low_strike
        f2p!(5850.0),     // short_call_high_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // long quantity
        85.04,     // premium_long_itm
        53.04,     // premium_long_otm
        28.85,     // premium_short
        0.78,      // premium_short
        0.78,      // open_fee_long
        0.78,      // close_fee_long
        0.73,      // close_fee_short
        0.73,      // close_fee_short
        0.73,      // open_fee_short
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.title(), "Ratio Call Spread Strategy: CallButterfly\n\tUnderlying: SP500 @ $5750 Long Call European Option\n\tUnderlying: SP500 @ $5800 Short Call European Option\n\tUnderlying: SP500 @ $5850 Short Call European Option");
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 2);
    assert_relative_eq!(
        strategy.net_premium_received().unwrap().to_f64().unwrap(),
        ZERO,
        epsilon = 0.001
    );
    assert!(strategy.max_profit().is_ok());
    assert!(strategy.max_loss().is_ok());
    assert_positivef64_relative_eq!(strategy.max_profit()?, f2p!(42.319), f2p!(0.0001));
    assert_eq!(strategy.max_loss()?, Positive::INFINITY);
    assert_positivef64_relative_eq!(strategy.total_cost(), f2p!(89.57), f2p!(0.0001));
    assert_eq!(strategy.fees().unwrap().to_f64().unwrap(), 4.53);

    // Test range calculations
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    assert!(!price_range.is_empty());
    let range = strategy.range_of_profit().unwrap();
    assert_relative_eq!(range.to_f64(), 134.639, epsilon = 0.001);
    assert_relative_eq!(
        (range.to_f64() / 2.0) / underlying_price * 100.0,
        1.164,
        epsilon = 0.001
    );

    assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points().unwrap();
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[1]);

    Ok(())
}
