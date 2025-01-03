use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::butterfly_spread::LongButterflySpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, f2p};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongButterflySpread strategy
    let underlying_price = f2p!(5795.88);

    let strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        f2p!(5710.0),     // long_strike_itm
        f2p!(5780.0),     // short_strike
        f2p!(5850.0),     // long_strike_otm
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // long quantity
        113.30,    // premium_long_low
        64.20,     // premium_short
        31.65,     // premium_long_high
        0.07,      // fees
    );

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0585), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0168), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-9832.8102), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(991.1053), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(72.3546), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-73.3649), epsilon);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        -0.0585,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        0.8744,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        0.2513,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 2);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        BuyOptions {
            quantity: f2p!(0.06699841451825994),
            strike: f2p!(5710.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
