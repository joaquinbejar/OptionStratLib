use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::pos;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::straddle::ShortStraddle;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_short_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortStraddle strategy
    let underlying_price = pos!(7138.5);

    let strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7140.0),     // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.01,      // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, -0.0884, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0008, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -3012.9912, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 2728.0855, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, -14.2856, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -77.8057, epsilon = 0.001);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        -0.0884,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        -0.5442,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        0.4557,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        SellOptions {
            quantity: pos!(0.19396073893948335),
            strike: pos!(7140.0),
            option_type: OptionStyle::Put
        }
    );

    Ok(())
}
