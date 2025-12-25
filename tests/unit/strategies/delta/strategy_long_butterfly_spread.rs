use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::OptionStyle;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::long_butterfly_spread::LongButterflySpread;
use optionstratlib::{ExpirationDate, Positive, assert_decimal_eq, assert_pos_relative_eq, pos_or_panic};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongButterflySpread strategy
    let underlying_price = pos_or_panic!(5795.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(5710.0),     // long_strike_itm
        pos_or_panic!(5780.0),     // short_strike
        pos_or_panic!(5850.0),     // long_strike_otm
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,      // long quantity
        pos_or_panic!(113.3),    // premium_long_low
        pos_or_panic!(64.20),    // premium_short
        pos_or_panic!(31.65),    // premium_long_high
        pos_or_panic!(0.07),     // fees
        pos_or_panic!(0.05),     // fees
        pos_or_panic!(0.03),     // fees
        pos_or_panic!(0.07),     // fees
        pos_or_panic!(0.05),     // fees
        pos_or_panic!(0.03),     // fees
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0585), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0168), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-26.93920), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(5.584519985), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(0.723546), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-0.733649), epsilon);
    assert_decimal_eq!(greeks.vanna, dec!(-1.0392151544), epsilon);
    assert_decimal_eq!(greeks.vomma, dec!(11.7847657892), epsilon);
    assert_decimal_eq!(greeks.veta, dec!(0.0397196877), epsilon);
    assert_decimal_eq!(greeks.charm, dec!(0.03338228), epsilon);
    assert_decimal_eq!(greeks.color, dec!(-0.00276925), epsilon);

    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().net_delta,
        dec!(-0.0585),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[0].delta,
        dec!(0.8744),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.delta_neutrality().unwrap().individual_deltas[2].delta,
        dec!(0.2513),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.delta_adjustments().unwrap().len(), 3);
    let binding = strategy.delta_adjustments().unwrap();
    let suggestion = binding.first().unwrap();
    let delta = pos_or_panic!(0.06699841451825994);
    let k = pos_or_panic!(5710.0);
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
