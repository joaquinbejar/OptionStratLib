use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, LongButterflySpread, Strategies},
    optionstratlib::{ExpirationDate, Positive, pos_or_panic},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_long_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongButterflySpread strategy
    let underlying_price = pos_or_panic!(5795.88);

    let mut strategy = LongButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos_or_panic!(5710.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(pos_or_panic!(2.0)),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(1.0),
        pos_or_panic!(113.3), // premium_long_low
        pos_or_panic!(64.20), // premium_short
        pos_or_panic!(31.65), // premium_long_high
        pos_or_panic!(0.07),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.get_best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.get_profit_area().unwrap().to_f64().unwrap(),
        398.9606,
        epsilon = 0.001
    );
    strategy.get_best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
        1745.0184,
        epsilon = 0.001
    );

    Ok(())
}
