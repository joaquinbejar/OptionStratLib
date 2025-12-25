use {
use positive::pos_or_panic;
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, LongStraddle, Strategies},
    optionstratlib::{ExpirationDate, Positive, pos_or_panic},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_long_straddle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongStraddle strategy
    let underlying_price = pos_or_panic!(7008.5);

    let mut strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(7140.0),     // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,      // quantity
        pos_or_panic!(84.2),     // premium_short_call
        pos_or_panic!(353.2),    // premium_short_put
        pos_or_panic!(7.01),     // open_fee_short_call
        pos_or_panic!(7.01),     // close_fee_short_call
        pos_or_panic!(7.01),     // open_fee_short_put
        pos_or_panic!(7.01),     // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.get_best_area(&option_chain, FindOptimalSide::All);
    assert_relative_eq!(
        strategy.get_profit_area().unwrap().to_f64().unwrap(),
        414.1996,
        epsilon = 0.001
    );
    strategy.get_best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
        200.0,
        epsilon = 0.001
    );

    Ok(())
}
