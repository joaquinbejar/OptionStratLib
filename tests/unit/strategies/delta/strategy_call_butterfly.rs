use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::{assert_decimal_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_call_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the CallButterfly strategy
    let underlying_price = pos!(5781.88);

    let strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_call_strike
        pos!(5800.0),     // short_call_low_strike
        pos!(5850.0),     // short_call_high_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // long quantity
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

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.0559), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0133), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-7606.7078), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(550.2891), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(40.2857), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-40.7342), epsilon);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        0.0559,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        -0.4177,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        -0.1971,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 2);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        SellOptions {
            quantity: pos!(0.13381901826077533),
            strike: pos!(5800.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
