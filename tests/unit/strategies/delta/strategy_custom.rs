use chrono::Utc;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::Options;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::utils::setup_logger;
use optionstratlib::{assert_decimal_eq, f2p};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
#[ignore]
fn test_custom_strategy_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define common parameters
    let underlying_price = f2p!(2340.0);
    let underlying_symbol = "GAS".to_string();
    let expiration = ExpirationDate::Days(6.0);
    let implied_volatility = 0.73;
    let risk_free_rate = 0.05;
    let dividend_yield = 0.0;

    // Create positions
    let positions = vec![
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                f2p!(2100.0),
                expiration.clone(),
                implied_volatility,
                f2p!(2.0),
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            192.0,
            Utc::now(),
            7.51,
            7.51,
        ),
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                f2p!(2250.0),
                expiration.clone(),
                implied_volatility,
                f2p!(2.0),
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            88.0,
            Utc::now(),
            6.68,
            6.68,
        ),
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                f2p!(2500.0),
                expiration.clone(),
                implied_volatility,
                f2p!(1.0),
                underlying_price,
                risk_free_rate,
                OptionStyle::Put,
                dividend_yield,
                None,
            ),
            55.0,
            Utc::now(),
            6.68,
            6.68,
        ),
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                f2p!(2150.0),
                expiration.clone(),
                implied_volatility,
                f2p!(2.5),
                underlying_price,
                risk_free_rate,
                OptionStyle::Put,
                dividend_yield,
                None,
            ),
            21.0,
            Utc::now(),
            4.91,
            4.91,
        ),
    ];

    let strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        underlying_symbol,
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        0.01,
        5,
        0.1,
    );

    let greeks = strategy.greeks();
    let epsilon = dec!(0.001);

    assert_decimal_eq!(greeks.delta, dec!(-1.9757), epsilon);
    assert_decimal_eq!(greeks.gamma, dec!(0.0093), epsilon);
    assert_decimal_eq!(greeks.theta, dec!(-13818.8979), epsilon);
    assert_decimal_eq!(greeks.vega, dec!(1642.8158), epsilon);
    assert_decimal_eq!(greeks.rho, dec!(59.0889), epsilon);
    assert_decimal_eq!(greeks.rho_d, dec!(-75.9988), epsilon);

    Ok(())
}
