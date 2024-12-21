use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::pos;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_iron_condor_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronCondor strategy
    let underlying_price = pos!(2646.9);

    let strategy = IronCondor::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos!(2725.0),     // short_call_strike
        pos!(2560.0),     // short_put_strike
        pos!(2800.0),     // long_call_strike
        pos!(2500.0),     // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(2.0), // quantity
        38.8,      // premium_short_call
        30.4,      // premium_short_put
        23.3,      // premium_long_call
        16.8,      // premium_long_put
        0.96,      // open_fee
        0.96,      // close_fee
    );

    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, -0.1148, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0165, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -1425.3530, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 3256.2375, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, 55.8247, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -63.3206, epsilon = 0.001);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        -0.1148,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        0.2492,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        -0.1611,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 2);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        BuyOptions {
            quantity: pos!(0.9213451734695193),
            strike: pos!(2800.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}