use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, PoorMansCoveredCall, Strategies},
    optionstratlib::{ExpirationDate, Positive, pos},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    let underlying_price = pos!(2703.3);

    let mut strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),                // underlying_symbol
        underlying_price,                  // underlying_price
        pos!(2600.0),                      // long_call_strike
        pos!(2800.0),                      // short_call_strike OTM
        ExpirationDate::Days(pos!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos!(30.0)),  // short_call_expiration 30-45 days delta 0.30 or less
        pos!(0.17),                        // implied_volatility
        dec!(0.05),                        // risk_free_rate
        Positive::ZERO,                    // dividend_yield
        pos!(2.0),                         // quantity
        pos!(154.7),                       // premium_short_call
        pos!(30.8),                        // premium_short_put
        pos!(1.74),                        // open_fee_short_call
        pos!(1.74),                        // close_fee_short_call
        pos!(0.85),                        // open_fee_short_put
        pos!(0.85),                        // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.get_best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.get_profit_area().unwrap().to_f64().unwrap(),
        1785.7483,
        epsilon = 0.001
    );
    strategy.get_best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
        955.5946,
        epsilon = 0.001
    );

    Ok(())
}
