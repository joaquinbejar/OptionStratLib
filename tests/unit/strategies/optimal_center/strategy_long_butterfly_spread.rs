use positive::pos_or_panic;
use {
    optionstratlib::Side,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{FindOptimalSide, LongButterflySpread},
    optionstratlib::{ExpirationDate, Positive},
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
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
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
    strategy.get_best_area(&option_chain, FindOptimalSide::Center);
    let positions = strategy.get_positions()?;
    for position in positions {
        if position.option.side == Side::Short {
            assert!(position.option.strike_price <= underlying_price)
        }
    }

    Ok(())
}
