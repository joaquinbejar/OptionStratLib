use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::straddle::LongStraddle;
use optionstratlib::utils::setup_logger;
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq, assert_pos_relative_eq, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]

fn test_long_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongStraddle strategy
    let underlying_price = pos!(7008.5);

    let strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7140.0),     // put_strike
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        pos!(84.2),     // premium_short_call
        pos!(353.2),    // premium_short_put
        pos!(7.01),     // open_fee_short_call
        pos!(7.01),     // close_fee_short_call
        pos!(7.01),     // open_fee_short_put
        pos!(7.01),     // close_fee_short_put
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0229), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0008), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-8.0431075), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(19.626624258), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-1.113739), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(0.198109), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(-0.0229),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(0.4885),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[1].delta,
        dec!(-0.5114),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);
    let binding = strategy.delta_adjustments().unwrap();
    let suggestion = binding.first().unwrap();
    let delta = pos!(0.04693144355338067);
    let k = pos!(7140.0);
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
            assert_eq!(*option_style, OptionStyle::Call);
            assert_eq!(*side, optionstratlib::model::types::Side::Long);
        }
        _ => panic!("Invalid suggestion"),
    }
    Ok(())
}
