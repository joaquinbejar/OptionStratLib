#[cfg(not(target_arch = "wasm32"))]
use {
    std::error::Error,
    optionstratlib::{pos, ExpirationDate, Positive},
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::{FindOptimalSide, BearPutSpread, Strategies},
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::utils::setup_logger,
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    rust_decimal_macros::dec,
};

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_bear_put_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BearPutSpread strategy
    let underlying_price = pos!(5781.88);

    let mut strategy = BearPutSpread::new(
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

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        741.8541,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        66.6666,
        epsilon = 0.001
    );

    Ok(())
}
