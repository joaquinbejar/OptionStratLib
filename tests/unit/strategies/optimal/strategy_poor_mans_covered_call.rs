use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::f2p;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use std::error::Error;

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let underlying_price = f2p!(2703.3);

    let mut strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),          // underlying_symbol
        underlying_price,            // underlying_price
        f2p!(2600.0),                // long_call_strike
        f2p!(2800.0),                // short_call_strike OTM
        ExpirationDate::Days(120.0), // long_call_expiration
        ExpirationDate::Days(30.0),  // short_call_expiration 30-45 days delta 0.30 or less
        0.17,                        // implied_volatility
        0.05,                        // risk_free_rate
        0.0,                         // dividend_yield
        f2p!(2.0),                   // quantity
        154.7,                       // premium_short_call
        30.8,                        // premium_short_put
        1.74,                        // open_fee_short_call
        1.74,                        // close_fee_short_call
        0.85,                        // open_fee_short_put
        0.85,                        // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        817.2115,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        408.9058,
        epsilon = 0.001
    );

    Ok(())
}
