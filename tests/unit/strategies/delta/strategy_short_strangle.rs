use optionstratlib::ExpirationDate;
use optionstratlib::assert_decimal_eq;
use optionstratlib::greeks::Greeks;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::NoAdjustmentNeeded;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::{DELTA_THRESHOLD, ShortStrangle};
use positive::{Positive, pos_or_panic};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_strangle_with_greeks_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortStrangle strategy
    let underlying_price = pos_or_panic!(7140.6);

    let strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(7450.0), // call_strike
        pos_or_panic!(7050.0), // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745), // implied_volatility
        pos_or_panic!(0.3745), // implied_volatility
        dec!(0.05),            // risk_free_rate
        Positive::ZERO,        // dividend_yield
        Positive::ONE,         // quantity
        pos_or_panic!(84.2),   // premium_short_call
        pos_or_panic!(353.2),  // premium_short_put
        pos_or_panic!(7.01),   // open_fee_short_call
        pos_or_panic!(7.01),   // close_fee_short_call
        pos_or_panic!(7.01),   // open_fee_short_put
        pos_or_panic!(7.01),   // close_fee_short_put
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = DELTA_THRESHOLD;

    assert_decimal_eq!(greeks.delta, dec!(0.00001), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0008), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-8.06459200), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(19.569191489), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-0.7052940734), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.00073528), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(0.2742684839), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(2.3016267940), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0032881122), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(-0.00195436), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.00000882), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        Decimal::ZERO,
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(-0.4168540),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(0.4169376),
        DELTA_THRESHOLD
    );
    assert!(strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 1);
    assert_eq!(strategy.delta_adjustments().unwrap()[0], NoAdjustmentNeeded);

    Ok(())
}
