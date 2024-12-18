use approx::assert_relative_eq;
use chrono::Utc;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::model::option::Options;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
use optionstratlib::pos;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;
use tracing::info;

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
                Side::Long,
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
                OptionStyle::Put,
                dividend_yield,
                None,
            ),
            88.0,
            Utc::now(),
            6.68,
            6.68,
        ),
    ];

    let mut strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        underlying_symbol,
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        0.01,
        10,
        0.1,
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::Lower);
    info!("Profit Area: {:.4}", strategy.profit_area());
    assert_relative_eq!(strategy.profit_area(), 75.4005, epsilon = 0.001);
    strategy.best_ratio(&option_chain, FindOptimalSide::All);
    info!("Profit Ratio: {:.4}", strategy.profit_ratio());
    assert_relative_eq!(strategy.profit_ratio(), 15.0989, epsilon = 0.001);

    Ok(())
}
