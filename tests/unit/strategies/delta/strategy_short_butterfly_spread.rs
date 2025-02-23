use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::butterfly_spread::ShortButterflySpread;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::{DeltaAdjustment, DELTA_THRESHOLD};
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, assert_pos_relative_eq, pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

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
        strategy.calculate_net_delta().net_delta,
        dec!(-0.0593),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        dec!(-2.5914),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        dec!(-0.5914),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    let binding = strategy.suggest_delta_adjustments();
    let suggestion = binding.first().unwrap();
    let delta = pos!(0.11409430831966512);
    let k = pos!(5780.0);
    match suggestion {
        DeltaAdjustment::BuyOptions {
            quantity,
            strike,
            option_type,
        } => {
            assert_pos_relative_eq!(
                *quantity,
                delta,
                Positive::new_decimal(DELTA_THRESHOLD).unwrap()
            );
            assert_pos_relative_eq!(*strike, k, Positive::new_decimal(DELTA_THRESHOLD).unwrap());
            assert_eq!(*option_type, OptionStyle::Call);
        }
        _ => panic!("Invalid suggestion"),
    }

    Ok(())
}
