use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::pos;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = pos!(2703.3);

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),          // underlying_symbol
        underlying_price,            // underlying_price
        pos!(2600.0),                // long_call_strike
        pos!(2800.0),                // short_call_strike OTM
        ExpirationDate::Days(120.0), // long_call_expiration
        ExpirationDate::Days(30.0),  // short_call_expiration 30-45 days delta 0.30 or less
        0.17,                        // implied_volatility
        0.05,                        // risk_free_rate
        0.0,                         // dividend_yield
        pos!(2.0),                   // quantity
        154.7,                       // premium_short_call
        30.8,                        // premium_short_put
        1.74,                        // open_fee_short_call
        1.74,                        // close_fee_short_call
        0.85,                        // open_fee_short_put
        0.85,                        // close_fee_short_put
    );

    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, 0.9225, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0075, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -1043.9572, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 2686.1099, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, 1290.9435, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -1420.1310, epsilon = 0.001);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        0.9225,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        1.4628,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        -0.5402,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        SellOptions {
            quantity: pos!(3.415412207592465),
            strike: pos!(2800.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
