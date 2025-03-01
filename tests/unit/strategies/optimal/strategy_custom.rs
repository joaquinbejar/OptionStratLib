#[cfg(not(target_arch = "wasm32"))]
use {
    approx::assert_relative_eq,
    chrono::Utc,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::model::Position,
    optionstratlib::strategies::CustomStrategy,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, Strategies},
    optionstratlib::utils::setup_logger,
    optionstratlib::{ExpirationDate, Positive, pos},
    optionstratlib::{OptionStyle, OptionType, Options, Side},
    rust_decimal_macros::dec,
    std::error::Error,
    tracing::info,
};

#[test]
#[ignore]
#[cfg(not(target_arch = "wasm32"))]
fn test_custom_strategy_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

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
                Side::Long,
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
                OptionStyle::Put,
                dividend_yield,
                None,
            ),
            pos!(88.0),
            Utc::now(),
            pos!(6.68),
            pos!(6.68),
        ),
    ];

    let mut strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        underlying_symbol,
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        pos!(0.01),
        5,
        pos!(0.1),
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::Lower);
    info!("Profit Area: {:.4}", strategy.profit_area().unwrap());
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        75.4005,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    info!(
        "Profit Ratio: {:.4}",
        strategy.profit_ratio().unwrap().to_f64().unwrap()
    );
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        15.0989,
        epsilon = 0.001
    );

    Ok(())
}
