use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::{DELTA_THRESHOLD, ShortStraddle};
use optionstratlib::{ExpirationDate, assert_decimal_eq};
use positive::{Positive, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_straddle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortStraddle strategy
    let underlying_price = pos_or_panic!(7138.5);

    let strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(7140.0), // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
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
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0884), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0008), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-8.2547704), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(19.87604540), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-0.142856), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.778057), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(0.0433370713), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(-0.1206043080), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0031581789), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(-0.00100642), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.00000950), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(-0.0884),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(-0.5442),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(0.4557),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);

    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos_or_panic!(0.19396073893948335);
    let k = pos_or_panic!(7140.0);
    match &binding[1] {
        BuyOptions {
            quantity,
            strike,
            option_style,
            side,
        } => {
            assert_pos_relative_eq!(
                *quantity,
                delta,
                Positive::new_decimal(DELTA_THRESHOLD).unwrap()
            );
            assert_pos_relative_eq!(*strike, k, Positive::new_decimal(DELTA_THRESHOLD).unwrap());
            assert_eq!(*option_style, OptionStyle::Put);
            assert_eq!(*side, optionstratlib::model::types::Side::Short);
        }
        _ => panic!("Invalid suggestion"),
    }

    Ok(())
}
