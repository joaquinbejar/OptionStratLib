use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::bear_put_spread::BearPutSpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::SellOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, pos, Positive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;
use std::str::FromStr;

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

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-1.2018), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0145), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-7271.7206), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(851.9072), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-64.5820), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(63.6651), epsilon);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        -1.2018,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        -1.6056,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        0.4038,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        SellOptions {
            quantity: Positive::new_decimal(Decimal::from_str("5.952144261472912").unwrap())
                .unwrap(),
            strike: pos!(5720.0),
            option_type: OptionStyle::Put
        }
    );

    Ok(())
}
