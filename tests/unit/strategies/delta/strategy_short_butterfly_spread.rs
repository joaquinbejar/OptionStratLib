use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::{DELTA_THRESHOLD, DeltaAdjustment, ShortButterflySpread};
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq, assert_pos_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5700.0),
        pos!(5780.0),
        pos!(5850.0),
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(3.0),
        pos!(119.01), // premium_long
        pos!(66.0),   // premium_short
        pos!(29.85),  // open_fee_long
        pos!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0593), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0503), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-79.624412818), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(16.59913113683), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(1.971329081), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-1.997983), epsilon);

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
    let delta = pos!(0.11409430831966512);
    let k = pos!(5780.0);
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
