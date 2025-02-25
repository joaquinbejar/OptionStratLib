use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, assert_pos_relative_eq, pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_bull_call_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BullCallSpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = BullCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5820.0),     // short_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // long quantity
        pos!(85.04),    // premium_long
        pos!(29.85),    // premium_short
        pos!(0.78),     // open_fee_long
        pos!(0.78),     // open_fee_long
        pos!(0.73),     // close_fee_long
        pos!(0.73),     // close_fee_short
    );
    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.7004), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0186), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-29.2743055), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(6.16042600), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(0.620955), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.628208), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(0.7004),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(1.3416),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(-0.6412),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 1);
    let binding = strategy.delta_adjustments().unwrap();
    let suggestion = binding.first().unwrap();
    let delta = pos!(2.184538786861796);
    let k = pos!(5820.0);
    match suggestion {
        SellOptions {
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
