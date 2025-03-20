use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::bear_put_spread::BearPutSpread;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::setup_logger;
use optionstratlib::{Positive, assert_decimal_eq, assert_pos_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]

fn test_bear_put_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BearPutSpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = BearPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5850.0),     // long_strike
        pos!(5720.0),     // short_strike
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

    assert_decimal_eq!(greeks.delta, dec!(-1.2018), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0145), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-19.92252244), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(4.7860164881), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-0.645820), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.636651), epsilon);

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
    let delta = pos!(5.952144261472912);
    let k = pos!(5720.0);
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
