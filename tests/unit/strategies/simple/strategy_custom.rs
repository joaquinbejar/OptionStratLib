use approx::assert_relative_eq;
use chrono::Utc;
use num_traits::ToPrimitive;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::strategies::Strategies;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::{ExpirationDate, Options};
use optionstratlib::{Positive, pos};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
#[ignore]
fn test_custom_strategy_integration() -> Result<(), Box<dyn Error>> {
    // Define common parameters
    let underlying_price = pos!(2340.0);
    let underlying_symbol = "GAS".to_string();
    let expiration = ExpirationDate::Days(pos!(6.0));
    let implied_volatility = pos!(0.73);
    let risk_free_rate = dec!(0.05);
    let dividend_yield = Positive::ZERO;

    // Create positions
    let positions = vec![
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                pos!(2100.0),
                expiration,
                implied_volatility,
                pos!(2.0),
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            pos!(192.0),
            Utc::now(),
            pos!(7.51),
            pos!(7.51),
        ),
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                pos!(2250.0),
                expiration,
                implied_volatility,
                pos!(2.0),
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            pos!(88.0),
            Utc::now(),
            pos!(6.68),
            pos!(6.68),
        ),
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                pos!(2500.0),
                expiration,
                implied_volatility,
                pos!(1.0),
                underlying_price,
                risk_free_rate,
                OptionStyle::Put,
                dividend_yield,
                None,
            ),
            pos!(55.0),
            Utc::now(),
            pos!(6.68),
            pos!(6.68),
        ),
        Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                pos!(2150.0),
                expiration,
                implied_volatility,
                pos!(2.5),
                underlying_price,
                risk_free_rate,
                OptionStyle::Put,
                dividend_yield,
                None,
            ),
            pos!(21.0),
            Utc::now(),
            pos!(4.91),
            pos!(4.91),
        ),
    ];

    let strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        underlying_symbol,
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        pos!(0.01),
        10,
        pos!(0.1),
    );

    // Test strategy properties and calculations
    assert_relative_eq!(
        strategy.get_net_premium_received().unwrap().to_f64(),
        572.83,
        epsilon = 0.001
    );
    assert_relative_eq!(
        strategy.get_fees().unwrap().to_f64(),
        51.56,
        epsilon = 0.001
    );

    // Test range and break-even points
    let price_range = strategy.get_best_range_to_show(pos!(1.0)).unwrap();
    assert!(!price_range.is_empty());

    // Test profit metrics
    assert!(
        strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0
            && strategy.get_profit_area().unwrap().to_f64().unwrap() <= 100.0,
        "Profit area should be between 0 and 100%"
    );
    assert!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0,
        "Profit ratio should be positive"
    );

    // Test positions
    assert_eq!(
        strategy.positions.len(),
        4,
        "Strategy should have exactly 4 positions"
    );

    // Validate position types
    let calls = strategy
        .positions
        .iter()
        .filter(|p| matches!(p.option.option_style, OptionStyle::Call))
        .count();
    let puts = strategy
        .positions
        .iter()
        .filter(|p| matches!(p.option.option_style, OptionStyle::Put))
        .count();
    assert_eq!(calls, 2, "Strategy should have 2 calls");
    assert_eq!(puts, 2, "Strategy should have 2 puts");

    Ok(())
}
