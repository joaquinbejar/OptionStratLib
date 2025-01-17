use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::butterfly_spread::ShortButterflySpread;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::ExpirationDate;
use optionstratlib::{pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_short_butterfly_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortButterflySpread strategy
    let underlying_price = pos!(5781.88);

    let mut strategy = ShortButterflySpread::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5700.0),
        pos!(5780.0),
        pos!(5850.0),
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(3.0),
        pos!(119.01),
        pos!(66.0),
        pos!(29.85),
        pos!(4.0),
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
        1002.06,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        428.3178,
        epsilon = 0.001
    );

    Ok(())
}
