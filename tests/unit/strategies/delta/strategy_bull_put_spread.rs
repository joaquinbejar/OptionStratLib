use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::bull_put_spread::BullPutSpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bull_put_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the BullPutSpread strategy
    let underlying_price = pos_or_panic!(5781.88);

    let strategy = BullPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(5750.0),     // long_strike_itm
        pos_or_panic!(5920.0),     // short_strike
        ExpirationDate::Days(pos_or_panic!(2.0)),
        pos_or_panic!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos_or_panic!(2.0),      // long quantity
        pos_or_panic!(15.04),    // premium_long
        pos_or_panic!(89.85),    // premium_short
        pos_or_panic!(0.78),     // open_fee_long
        pos_or_panic!(0.78),     // open_fee_long
        pos_or_panic!(0.73),     // close_fee_long
        pos_or_panic!(0.73),     // close_fee_short
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(1.2605), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0116), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-15.20725615), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(3.84242415), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-0.833461), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.816525), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(-0.0226839453), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(31.9307265715), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0486186188), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(-0.008209), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.000736), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(1.2605),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(1.9189),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(-0.6583),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);
    let binding = strategy.delta_adjustments().unwrap();
    let suggestion = binding.first().unwrap();
    let delta = pos_or_panic!(3.829496711654006);
    let k = pos_or_panic!(5750.0);
    match suggestion {
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
            assert_eq!(*side, optionstratlib::model::types::Side::Long);
        }
        _ => panic!("Invalid suggestion"),
    }
    Ok(())
}
