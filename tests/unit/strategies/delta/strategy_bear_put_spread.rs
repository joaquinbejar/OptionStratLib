use positive::{assert_pos_relative_eq, pos_or_panic};
use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::bear_put_spread::BearPutSpread;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bear_put_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the BearPutSpread strategy
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = BearPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(5850.0),     // long_strike
        pos_or_panic!(5720.0),     // short_strike
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::TWO,      // long quantity
        pos_or_panic!(85.04),    // premium_long
        pos_or_panic!(29.85),    // premium_short
        pos_or_panic!(0.78),     // open_fee_long
        pos_or_panic!(0.78),     // open_fee_long
        pos_or_panic!(0.73),     // close_fee_long
        pos_or_panic!(0.73),     // close_fee_short
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-1.2018), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0145), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-19.92252244), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(4.7860164881), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-0.645820), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.636651), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(0.0980755896), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(37.8187674591), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0593138779), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(-0.015910), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.001047), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(-1.2018),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(-1.6056),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(0.4038),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);

    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos_or_panic!(5.952144261472912);
    let k = pos_or_panic!(5720.0);
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
