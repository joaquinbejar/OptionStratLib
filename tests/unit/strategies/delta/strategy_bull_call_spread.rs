use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::pos;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_bull_call_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BullCallSpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = BullCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5820.0),     // short_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(2.0), // long quantity
        85.04,     // premium_long
        29.85,     // premium_short
        0.78,      // open_fee_long
        0.78,      // open_fee_long
        0.73,      // close_fee_long
        0.73,      // close_fee_short
    );
    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, 0.7004, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0186, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -10685.1215, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 848.6626, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, 62.0955, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -62.8208, epsilon = 0.001);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        0.7004,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        1.3416,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        -0.6412,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        SellOptions {
            quantity: pos!(2.184538786861787),
            strike: pos!(5820.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
