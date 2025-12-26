use num_traits::ToPrimitive;
use optionstratlib::ExpirationDate;
use optionstratlib::strategies::Strategies;
use optionstratlib::strategies::base::BreakEvenable;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use positive::Positive;
use positive::{assert_pos_relative_eq, pos_or_panic};

use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    let underlying_price = pos_or_panic!(2703.3);

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),                         // underlying_symbol
        underlying_price,                           // underlying_price
        pos_or_panic!(2600.0),                      // long_call_strike
        pos_or_panic!(2800.0),                      // short_call_strike OTM
        ExpirationDate::Days(pos_or_panic!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos_or_panic!(30.0)), // short_call_expiration 30-45 days delta 0.30 or less
        pos_or_panic!(0.17),                       // implied_volatility
        dec!(0.05),                                // risk_free_rate
        Positive::ZERO,                            // dividend_yield
        Positive::TWO,                             // quantity
        pos_or_panic!(154.7),                      // premium_short_call
        pos_or_panic!(30.8),                       // premium_short_put
        pos_or_panic!(1.74),                       // open_fee_short_call
        pos_or_panic!(1.74),                       // close_fee_short_call
        pos_or_panic!(0.85),                       // open_fee_short_put
        pos_or_panic!(0.85),                       // close_fee_short_put
    );

    // Assertions to validate strategy properties and computations
    assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
    assert!(strategy.get_max_profit().is_ok());
    assert!(strategy.get_max_loss().is_ok());
    assert_pos_relative_eq!(
        strategy.get_max_profit()?,
        pos_or_panic!(141.8399),
        pos_or_panic!(0.0001)
    );
    assert_pos_relative_eq!(
        strategy.get_max_loss()?,
        pos_or_panic!(258.16),
        pos_or_panic!(0.0001)
    );
    assert_eq!(strategy.get_fees().unwrap().to_f64(), 10.36);
    assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
    assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);

    // Test range calculations
    let price_range = strategy.get_best_range_to_show(Positive::ONE).unwrap();
    assert!(!price_range.is_empty());

    // Validate price range in relation to break even points
    let break_even_points = strategy.get_break_even_points().unwrap();
    assert!(price_range[0] < break_even_points[0]);
    assert!(price_range[price_range.len() - 1] > break_even_points[0]);

    // Additional strategy-specific validations
    assert!(
        pos_or_panic!(2600.0) < pos_or_panic!(2800.0),
        "Long call strike should be less than short call strike in a PMCC"
    );

    Ok(())
}
