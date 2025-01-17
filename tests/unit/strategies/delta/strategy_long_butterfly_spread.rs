use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::butterfly_spread::LongButterflySpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::DELTA_THRESHOLD;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, assert_pos_relative_eq, pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongButterflySpread strategy
    let underlying_price = pos!(5795.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5710.0),     // long_strike_itm
        pos!(5780.0),     // short_strike
        pos!(5850.0),     // long_strike_otm
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // long quantity
        pos!(113.3),    // premium_long_low
        pos!(64.20),    // premium_short
        pos!(31.65),    // premium_long_high
        pos!(0.07),     // fees
        pos!(0.05),     // fees
        pos!(0.03),     // fees
        pos!(0.07),     // fees
        pos!(0.05),     // fees
        pos!(0.03),     // fees
    );

    let greeks = strategy.greeks().unwrap();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0585), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0168), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-26.93920), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(5.584519985), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(72.3546), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-73.3649), epsilon);

    assert_decimal_eq!(
        strategy.calculate_net_delta().net_delta,
        dec!(-0.0585),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        dec!(0.8744),
        DELTA_THRESHOLD
    );
    assert_decimal_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        dec!(0.2513),
        DELTA_THRESHOLD
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 2);
    let binding = strategy.suggest_delta_adjustments();
    let suggestion = binding.first().unwrap();
    let delta = pos!(0.06699841451825994);
    let k = pos!(5710.0);
    match suggestion {
        BuyOptions {
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
