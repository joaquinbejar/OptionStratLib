use approx::assert_relative_eq;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::f2p;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;
use num_traits::ToPrimitive;

#[test]
fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronButterfly strategy
    let underlying_price = f2p!(2646.9);

    let mut strategy = IronButterfly::new(
        "GOLD".to_string(),
        underlying_price, // underlying_price
        f2p!(2725.0),     // short_call_strike
        f2p!(2800.0),     // long_call_strike
        f2p!(2500.0),     // long_put_strike
        ExpirationDate::Days(30.0),
        0.1548,    // implied_volatility
        0.05,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(2.0), // quantity
        38.8,      // premium_short_call
        30.4,      // premium_short_put
        23.3,      // premium_long_call
        16.8,      // premium_long_put
        0.96,      // open_fee
        0.96,      // close_fee
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(strategy.profit_area().unwrap().to_f64().unwrap(), 23.3347, epsilon = 0.001);
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(strategy.profit_ratio().unwrap().to_f64().unwrap(), 387.3294, epsilon = 0.001);

    Ok(())
}
