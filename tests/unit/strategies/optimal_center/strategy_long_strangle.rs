use {
    optionstratlib::OptionStyle,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{FindOptimalSide, LongStrangle},
    optionstratlib::{ExpirationDate, Positive, pos_or_panic},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_long_strangle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongStrangle strategy
    let underlying_price = pos_or_panic!(7138.5);

    let mut strategy = LongStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos_or_panic!(7450.0),     // call_strike
        pos_or_panic!(7050.0),     // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,      // quantity
        pos_or_panic!(84.2),     // premium_short_call
        pos_or_panic!(353.2),    // premium_short_put
        pos_or_panic!(7.0),      // open_fee_short_call
        pos_or_panic!(7.01),     // close_fee_short_call
        pos_or_panic!(7.01),     // open_fee_short_put
        pos_or_panic!(7.01),     // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.get_best_area(&option_chain, FindOptimalSide::Center);
    let positions = strategy.get_positions()?;
    for position in positions {
        match position.option.option_style {
            OptionStyle::Call => {
                assert!(position.option.strike_price >= underlying_price)
            }
            OptionStyle::Put => {
                assert!(position.option.strike_price <= underlying_price)
            }
        }
    }

    Ok(())
}
