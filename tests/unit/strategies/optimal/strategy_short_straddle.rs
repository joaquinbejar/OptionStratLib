use approx::assert_relative_eq;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::straddle::ShortStraddle;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_short_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortStraddle strategy
    let underlying_price = pos!(7138.5);

    let mut strategy = ShortStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7140.0),     // put_strike
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        pos!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.01,      // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(strategy.profit_area(), 58.7383, epsilon = 0.001);
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(strategy.profit_ratio(), 50.0, epsilon = 0.001);

    Ok(())
}
