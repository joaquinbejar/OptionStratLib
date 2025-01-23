#[cfg(not(target_arch = "wasm32"))]
use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, LongStraddle, Strategies},
    optionstratlib::utils::setup_logger,
    optionstratlib::{pos, ExpirationDate, Positive},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_long_straddle_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the LongStraddle strategy
    let underlying_price = pos!(7008.5);

    let mut strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7140.0),     // put_strike
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        pos!(84.2),     // premium_short_call
        pos!(353.2),    // premium_short_put
        pos!(7.01),     // open_fee_short_call
        pos!(7.01),     // close_fee_short_call
        pos!(7.01),     // open_fee_short_put
        pos!(7.01),     // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.profit_area().unwrap().to_f64().unwrap(),
        414.1996,
        epsilon = 0.001
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.profit_ratio().unwrap().to_f64().unwrap(),
        200.0,
        epsilon = 0.001
    );

    Ok(())
}
