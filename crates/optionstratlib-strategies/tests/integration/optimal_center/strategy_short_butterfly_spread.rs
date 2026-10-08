use optionstratlib_core::{model::Positive, pos_or_panic};
use {
    optionstratlib_core::model::ExpirationDate, optionstratlib_core::model::Side,
    optionstratlib_market::chains::chain::OptionChain,
    optionstratlib_market::chains::utils::FindOptimalSide,
    optionstratlib_strategies::strategies::ShortButterflySpread,
    optionstratlib_strategies::strategies::base::Optimizable,
    optionstratlib_strategies::strategies::base::Positionable, rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos_or_panic!(5781.88);

    let mut strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos_or_panic!(5700.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(3.0),
        pos_or_panic!(119.01),
        pos_or_panic!(66.0),
        pos_or_panic!(29.85),
        pos_or_panic!(4.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )?;

    let option_chain = OptionChain::load_from_json(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    ))?;
    strategy
        .get_best_area(&option_chain, FindOptimalSide::Center)
        .unwrap();
    let positions = strategy.get_positions()?;
    for position in positions {
        if position.option.side == Side::Long {
            assert!(position.option.strike_price <= underlying_price)
        }
    }

    Ok(())
}
