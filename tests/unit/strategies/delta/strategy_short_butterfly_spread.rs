use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::{DELTA_THRESHOLD, DeltaAdjustment, ShortButterflySpread};
use optionstratlib::{ExpirationDate, assert_decimal_eq};
use positive::{Positive, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos_or_panic!(5700.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(3.0),
        pos_or_panic!(119.01), // premium_long
        pos_or_panic!(66.0),   // premium_short
        pos_or_panic!(29.85),  // open_fee_long
        pos_or_panic!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )?;

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0593), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0117158224334441273467339536), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-17.349522936631938224385044096), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(3.8629631462760898341060554328), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-0.0135244886441871056177996109), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.0188176328168945065953874032), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(-0.5636159156577169179006960252), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(-33.022191072577642750667870994), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(-0.0076078027469015844621148210), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(0.0160833354632983691810448494), epsilon);
    // The old -0.00806599 was the unsigned sum and slipped under the shared
    // 1e-3 tolerance by 6.2e-4. Re-derived at 40 digits against the Merton
    // closed forms, and pinned tightly so it cannot hide again.
    assert_decimal_eq!(greeks.color, dec!(-0.0074509235423), dec!(1e-10));

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(-0.0593),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(-2.5914),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[2].delta,
        dec!(-0.5914),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);

    let binding = strategy.delta_adjustments().unwrap();
    let delta = pos_or_panic!(0.11409430831966512);
    let k = pos_or_panic!(5780.0);
    match &binding[1] {
        DeltaAdjustment::BuyOptions {
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
