use optionstratlib::greeks::Greeks;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::NoAdjustmentNeeded;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::{DeltaAdjustment, LongStrangle, DELTA_THRESHOLD};
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, assert_pos_relative_eq, pos, OptionStyle};
use optionstratlib::{ExpirationDate, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_long_strangle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongStrangle strategy
    let underlying_price = pos!(7140.6);

    let strategy = LongStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7450.0),     // call_strike
        pos!(7050.0),     // put_strike
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        pos!(84.2),     // premium_short_call
        pos!(353.2),    // premium_short_put
        pos!(7.01),     // open_fee_short_call
        pos!(7.01),     // close_fee_short_call
        pos!(7.01),     // open_fee_short_put
        pos!(7.01),     // close_fee_short_put
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = DELTA_THRESHOLD;

    assert_decimal_eq!(greeks.delta, dec!(0.00001), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0008), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-2943.57608224), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(2507.02263860), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-70.5294073481), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.073528197151), epsilon);

    assert_decimal_eq!(
        strategy.calculate_net_delta().net_delta,
        dec!(-0.0000835),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        dec!(0.41685408),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        dec!(-0.416937),
        DELTA_THRESHOLD
    );
    assert!(strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);
    assert_eq!(strategy.suggest_delta_adjustments()[0], NoAdjustmentNeeded);
    Ok(())
}
