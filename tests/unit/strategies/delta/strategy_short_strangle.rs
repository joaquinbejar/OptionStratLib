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
    )?;

    let greeks = strategy.greeks().unwrap();
    let epsilon = DELTA_THRESHOLD;

    assert_decimal_eq!(greeks.delta, dec!(0.00001), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(-0.0008312506548070177682126771), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(8.064592005817847147001604890), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(-19.569191488864943003335299952), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(0.7052940734385944826090595496), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.0007352819715194783511293371), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(-0.2742684839381314127381931021), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(-2.3016267939924681954653119310), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(-0.0032881122228448042811767833), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(0.0019543614202870773736805913), epsilon);
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
