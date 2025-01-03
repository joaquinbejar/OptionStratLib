use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::butterfly_spread::ShortButterflySpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, pos, Positive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;
use std::str::FromStr;

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
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-0.0593), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0503), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-29062.9106), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(2699.1274), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(197.1329), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-199.7983), epsilon);

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
            quantity: Positive::new_decimal(Decimal::from_str("0.11409430831966512").unwrap())
                .unwrap(),
            strike: pos!(5780.0),
            option_type: OptionStyle::Call
        }
    );

    Ok(())
}
