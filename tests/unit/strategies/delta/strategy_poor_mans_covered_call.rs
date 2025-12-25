use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    let underlying_price = pos_or_panic!(2703.3);

    let strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),                // underlying_symbol
        underlying_price,                  // underlying_price
        pos_or_panic!(2600.0),                      // long_call_strike
        pos_or_panic!(2800.0),                      // short_call_strike OTM
        ExpirationDate::Days(pos_or_panic!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos_or_panic!(30.0)),  // short_call_expiration 30-45 days delta 0.30 or less
        pos_or_panic!(0.17),                        // implied_volatility
        dec!(0.05),                        // risk_free_rate
        Positive::ZERO,                    // dividend_yield
        Positive::TWO,                         // quantity
        pos_or_panic!(154.7),                       // premium_short_call
        pos_or_panic!(30.8),                        // premium_short_put
        pos_or_panic!(1.74),                        // open_fee_short_call
        pos_or_panic!(1.74),                        // close_fee_short_call
        pos_or_panic!(0.85),                        // open_fee_short_put
        pos_or_panic!(0.85),                        // close_fee_short_put
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.9225), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0075), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-2.8601567), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(15.3494934566), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(12.909435), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-14.201310), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(0.5565729391), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(62.9867489281), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0051055965), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(-0.00864689), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.00005040), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(0.9225),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(1.4628),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(-0.5402),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);
    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos_or_panic!(3.415412207592464);
    let k = pos_or_panic!(2800.0);
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
            assert_eq!(*option_style, OptionStyle::Call);
            assert_eq!(*side, optionstratlib::model::types::Side::Short);
        }
        _ => panic!("Invalid suggestion"),
    }

    Ok(())
}
