#[cfg(not(target_arch = "wasm32"))]
use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, IronButterfly, Strategies},
    optionstratlib::utils::setup_logger,
    optionstratlib::{ExpirationDate, Positive, pos},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronButterfly strategy
    let underlying_price = pos!(2646.9);

    let mut strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        pos!(2725.0),     // short_call_strike
        pos!(2800.0),     // long_call_strike
        pos!(2500.0),     // long_put_strike
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.1548),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // quantity
        pos!(38.8),     // premium_short_call
        pos!(30.4),     // premium_short_put
        pos!(23.3),     // premium_long_call
        pos!(16.8),     // premium_long_put
        pos!(0.96),     // open_fee
        pos!(0.96),     // close_fee
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        0.2583,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        3.1240,
        epsilon = 0.001
    );

    Ok(())
}
