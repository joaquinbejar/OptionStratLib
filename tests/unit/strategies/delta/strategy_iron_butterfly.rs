use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::{
    ExpirationDate, Positive, Side, assert_decimal_eq, assert_pos_relative_eq, pos,
};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the IronButterfly strategy
    let underlying_price = pos!(2646.9);

    let strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos!(2725.0),     // short_call_strike
        pos!(2800.0),     // long_call_strike
        pos!(2500.0),     // long_put_strike
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.1548),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // quantity
        pos!(38.8),     // premium_short_call
        pos!(30.4),     // premium_short_put
        pos!(23.3),     // premium_long_call
        pos!(16.8),     // premium_long_put
        pos!(0.96),     // open_fee
        pos!(0.96),     // close_fee
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.9103), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0177), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-3.789816117), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(15.84942898), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-1.796019), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(1.597057), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(5.7651822592), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(153.9868702894), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0139491695), epsilon);     

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(0.9103),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(-0.5888889052),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(1.4111110947),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);

    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos!(11.301514988575999);
    let k = pos!(2500.0);
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
            assert_eq!(*option_style, OptionStyle::Put);
            assert_eq!(*side, Side::Long);
        }
        _ => panic!("Invalid suggestion"),
    }
    Ok(())
}
