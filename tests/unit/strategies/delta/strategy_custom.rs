use approx::assert_relative_eq;
use chrono::Utc;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::option::Options;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
use optionstratlib::pos;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_custom_strategy_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define common parameters
    let underlying_price = pos!(2340.0);
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
                pos!(2100.0),
                expiration.clone(),
                implied_volatility,
                pos!(2.0),
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
                pos!(2250.0),
                expiration.clone(),
                implied_volatility,
                pos!(2.0),
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
                pos!(2500.0),
                expiration.clone(),
                implied_volatility,
                pos!(1.0),
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
                pos!(2150.0),
                expiration.clone(),
                implied_volatility,
                pos!(2.5),
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
        100,
        0.1,
    );

    let greeks = strategy.greeks();

    assert_relative_eq!(greeks.delta, -1.975, epsilon = 0.001);
    assert_relative_eq!(greeks.gamma, 0.0093, epsilon = 0.001);
    assert_relative_eq!(greeks.theta, -13818.8978, epsilon = 0.001);
    assert_relative_eq!(greeks.vega, 1642.8158, epsilon = 0.001);
    assert_relative_eq!(greeks.rho, 59.0889, epsilon = 0.001);
    assert_relative_eq!(greeks.rho_d, -75.9988, epsilon = 0.001);

    Ok(())
}
