use approx::assert_relative_eq;
use num_traits::ToPrimitive;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::call_butterfly::CallButterfly;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::ExpirationDate;
use optionstratlib::{pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;

#[test]
fn test_call_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the CallButterfly strategy
    let underlying_price = pos!(5781.88);

    let mut strategy = CallButterfly::new(
        "SP500".to_string(),
        underlying_price,   // underlying_price
        pos!(5750.0),   // long_call_strike
        pos!(5800.0),   // short_call_low_strike
        pos!(5850.0),   // short_call_high_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),   // implied_volatility
        dec!(0.05),   // risk_free_rate
        Positive::ZERO,   // dividend_yield
        pos!(1.0),   // long quantity
        pos!(85.04),   // premium_long_itm
        53.04,   // premium_long_otm
        28.85,   // premium_short
        pos!(0.78),   // premium_short
        pos!(0.78),   // open_fee_long
        pos!(0.78),   // close_fee_long
        pos!(0.73),   // close_fee_short
        pos!(0.73),   // close_fee_short
        pos!(0.73),   // open_fee_short
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        68391.6908,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        10660.0,
        epsilon = 0.001
    );

    Ok(())
}
