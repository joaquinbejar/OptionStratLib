use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, f2p};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronButterfly strategy
    let underlying_price = f2p!(2646.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        f2p!(2725.0),     // short_call_strike
        f2p!(2800.0),     // long_call_strike
        f2p!(2500.0),     // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(2.0), // quantity
        38.8,      // premium_short_call
        30.4,      // premium_short_put
        23.3,      // premium_long_call
        16.8,      // premium_long_put
        0.96,      // open_fee
        0.96,      // close_fee
    );

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.9103), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0177), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-1383.2828), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(2478.3050), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-179.6019), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(159.7057), epsilon);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        0.9103,
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
            quantity: f2p!(11.301514988575999),
            strike: f2p!(2500.0),
            option_type: OptionStyle::Put
        }
    );

    Ok(())
}
