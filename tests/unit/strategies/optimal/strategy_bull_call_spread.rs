use approx::assert_relative_eq;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::pos;
use std::error::Error;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::strategies::utils::FindOptimalSide;

#[test]
fn test_bull_call_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BullCallSpread strategy
    let underlying_price = pos!(5781.88);

    let mut strategy = BullCallSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5820.0),     // short_strike
        ExpirationDate::Days(2.0),
        0.18,      // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(2.0), // long quantity
        85.04,     // premium_long
        29.85,     // premium_short
        0.78,      // open_fee_long
        0.78,      // open_fee_long
        0.73,      // close_fee_long
        0.73,      // close_fee_short
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(strategy.profit_area(), 417.3849, epsilon = 0.001);
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(strategy.profit_ratio(), 473.3944, epsilon = 0.001);

    Ok(())
}
