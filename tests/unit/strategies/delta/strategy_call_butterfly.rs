use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::{ExpirationDate, assert_decimal_eq};
use positive::{Positive, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_call_butterfly_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the CallButterfly strategy
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(5750.0), // long_call_strike
        pos_or_panic!(5800.0), // short_call_low_strike
        pos_or_panic!(5850.0), // short_call_high_strike
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),  // implied_volatility
        dec!(0.05),           // risk_free_rate
        Positive::ZERO,       // dividend_yield
        Positive::ONE,        // long quantity
        pos_or_panic!(85.04), // premium_long_itm
        pos_or_panic!(53.04), // premium_long_otm
        pos_or_panic!(28.85), // premium_short
        pos_or_panic!(0.78),  // premium_short
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.78),  // close_fee_long
        pos_or_panic!(0.73),  // close_fee_short
        pos_or_panic!(0.73),  // close_fee_short
        pos_or_panic!(0.73),  // open_fee_short
    )?;

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.0559), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(-0.0039746331152597696741583943), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(5.8556287094413654835998710868), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(-1.3105235532067897906001598274), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(0.0166909119956756638654876337), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.0177115404599258070268733513), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(-2.6752468799657139996070988074), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(-3.6581444543486345463360272344), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(-0.0074340773385041131160062564), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(0.1235341714774240128543034519), epsilon);
    assert_decimal_eq!(greeks.color, dec!(0.0004306809265636565888984281), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(0.0559),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(-0.4177),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[2].delta,
        dec!(-0.1971),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);

    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos_or_panic!(0.13381901826077533);
    let k = pos_or_panic!(5800.0);
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
