use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;
use positive::pos_or_panic;

#[test]
fn test_iron_condor_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the IronCondor strategy
    let underlying_price = pos_or_panic!(2646.9);

    let strategy = IronCondor::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(2725.0),     // short_call_strike
        pos_or_panic!(2560.0),     // short_put_strike
        pos_or_panic!(2800.0),     // long_call_strike
        pos_or_panic!(2500.0),     // long_put_strike
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.1548),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::TWO,      // quantity
        pos_or_panic!(38.8),     // premium_short_call
        pos_or_panic!(30.4),     // premium_short_put
        pos_or_panic!(23.3),     // premium_long_call
        pos_or_panic!(16.8),     // premium_long_put
        pos_or_panic!(0.96),     // open_fee
        pos_or_panic!(0.96),     // close_fee
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.1148), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0165), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-3.90507682), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(14.7753319330), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(0.558247), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.633206), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(0.2487598459), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(170.9330480070), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0134886944), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(-0.00665184), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.00003014), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(-0.1148),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[2].delta,
        dec!(0.2492),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[3].delta,
        dec!(-0.1611),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 4);
    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos_or_panic!(0.921345173469528);
    let k = pos_or_panic!(2800.0);
    match &binding[2] {
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
            assert_eq!(*option_style, OptionStyle::Call);
            assert_eq!(*side, optionstratlib::model::types::Side::Long);
        }
        _ => panic!("Invalid suggestion"),
    }

    Ok(())
}
