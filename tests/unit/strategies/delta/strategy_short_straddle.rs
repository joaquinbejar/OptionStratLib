use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::Positive;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::straddle::ShortStraddle;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::{assert_decimal_eq, f2p};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortStraddle strategy
    let underlying_price = f2p!(7138.5);

    let strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        f2p!(7140.0),     // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.01,      // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0884), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0008), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-3012.9912), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(2728.0855), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-14.2856), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-77.8057), epsilon);

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
            quantity: f2p!(0.19396073893948335),
            strike: f2p!(7140.0),
            option_type: OptionStyle::Put
        }
    );

    Ok(())
}
