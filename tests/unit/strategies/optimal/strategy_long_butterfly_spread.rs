#[cfg(not(target_arch = "wasm32"))]
use {
    std::error::Error,
    optionstratlib::{pos, ExpirationDate, Positive},
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::{FindOptimalSide, LongButterflySpread, Strategies},
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::utils::setup_logger,
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    rust_decimal_macros::dec,
};

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongButterflySpread strategy
    let underlying_price = pos!(5795.88);

    let mut strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5710.0),
        pos!(5780.0),
        pos!(5850.0),
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(1.0),
        pos!(113.3), // premium_long_low
        pos!(64.20), // premium_short
        pos!(31.65), // premium_long_high
        pos!(0.07),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        398.9606,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        1349.2753,
        epsilon = 0.001
    );

    Ok(())
}
