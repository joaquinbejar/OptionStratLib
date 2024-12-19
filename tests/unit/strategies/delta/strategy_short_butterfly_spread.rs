use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::pos;
use optionstratlib::strategies::butterfly_spread::ShortButterflySpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5700.0),     // short_strike_itm
        pos!(5780.0),     // long_strike
        pos!(5850.0),     // short_strike_otm
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(3.0), // long quantity
        119.01,    // premium_long
        66.0,      // premium_short
        29.85,     // open_fee_long
        4.0,       // open_fee_long
    );

    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, -0.0593, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0503, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -29062.9106, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 2699.1274, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, 197.1329, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -199.7983, epsilon = 0.001);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        -0.0593,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        -2.5914,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        -0.5914,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        BuyOptions {
            quantity: pos!(0.11409430831965134),
            strike: pos!(5780.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
