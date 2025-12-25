use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, ShortStrangle, Strategies},
    optionstratlib::{ExpirationDate, Positive},
    rust_decimal_macros::dec,
    std::error::Error,
};
use positive::pos_or_panic;

#[test]
fn test_short_strangle_with_greeks_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortStrangle strategy
    let underlying_price = pos_or_panic!(7138.5);

    let mut strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(7450.0),     // call_strike
        pos_or_panic!(7050.0),     // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),   // implied_volatility
        pos_or_panic!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,      // quantity
        pos_or_panic!(84.2),     // premium_short_call
        pos_or_panic!(35.2),     // premium_short_put
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
        12.0366,
        epsilon = 0.001
    );
    strategy.get_best_ratio(&option_chain, FindOptimalSide::Upper);
    assert_relative_eq!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
        47.4665,
        epsilon = 0.001
    );

    Ok(())
}
