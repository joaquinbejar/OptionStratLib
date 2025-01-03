use approx::assert_relative_eq;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle};
use optionstratlib::strategies::bull_put_spread::BullPutSpread;
use optionstratlib::strategies::delta_neutral::DeltaAdjustment::BuyOptions;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_bull_put_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BullPutSpread strategy
    let underlying_price = pos!(5781.88);

    let strategy = BullPutSpread::new(
        "SP500".to_string(),
        underlying_price,   // underlying_price
        pos!(5750.0),   // long_strike_itm
        pos!(5920.0),   // short_strike
        ExpirationDate::Days(2.0),
        pos!(0.18),   // implied_volatility
        dec!(0.05),   // risk_free_rate
        Positive::ZERO,   // dividend_yield
        pos!(2.0),   // long quantity
        15.04,   // premium_long
        89.85,   // premium_short
        0.78,   // open_fee_long
        0.78,   // open_fee_long
        0.73,   // close_fee_long
        0.73,   // close_fee_short
    );

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(1.2605), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0116), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-5550.6484), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(608.9101), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(-83.3461), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(81.6525), epsilon);

    assert_relative_eq!(
        strategy.calculate_net_delta().net_delta,
        1.2605,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[0],
        1.9189,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.calculate_net_delta().individual_deltas[1],
        -0.6583,
        epsilon = 0.001
    );
    assert!(!strategy.is_delta_neutral());
    assert_eq!(strategy.suggest_delta_adjustments().len(), 1);

    assert_eq!(
        strategy.suggest_delta_adjustments()[0],
        BuyOptions {
            quantity: pos!(3.829496711654006),
            strike: pos!(5750.0),
            option_type: OptionStyle::Put
        }
    );

    Ok(())
}
