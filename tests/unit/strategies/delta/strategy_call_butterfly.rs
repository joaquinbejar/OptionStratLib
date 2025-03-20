use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::setup_logger;
use optionstratlib::{Positive, assert_decimal_eq, assert_pos_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]

fn test_call_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the CallButterfly strategy
    let underlying_price = pos!(5781.88);

    let strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_call_strike
        pos!(5800.0),     // short_call_low_strike
        pos!(5850.0),     // short_call_high_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // long quantity
        pos!(85.04),    // premium_long_itm
        pos!(53.04),    // premium_long_otm
        pos!(28.85),    // premium_short
        pos!(0.78),     // premium_short
        pos!(0.78),     // open_fee_long
        pos!(0.78),     // close_fee_long
        pos!(0.73),     // close_fee_short
        pos!(0.73),     // close_fee_short
        pos!(0.73),     // open_fee_short
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(0.0559), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0133), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-20.840295476), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(4.40736684), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(0.402857), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.407342), epsilon);

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
    let delta = pos!(0.13381901826077533);
    let k = pos!(5800.0);
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
