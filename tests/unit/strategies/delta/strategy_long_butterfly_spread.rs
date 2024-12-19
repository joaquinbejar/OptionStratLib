use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::pos;
use optionstratlib::strategies::butterfly_spread::LongButterflySpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
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
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // long quantity
        113.30,    // premium_long_low
        64.20,     // premium_short
        31.65,     // premium_long_high
        0.07,      // fees
    );

    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, -0.0585, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0168, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -9832.8102, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 991.1053, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, 72.3546, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -73.3649, epsilon = 0.001);

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
            quantity: pos!(0.0669984145182543),
            strike: pos!(5710.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
