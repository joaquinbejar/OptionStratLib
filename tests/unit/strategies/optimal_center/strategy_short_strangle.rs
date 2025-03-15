#[cfg(not(target_arch = "wasm32"))]
use {
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::{FindOptimalSide, ShortStrangle},
    optionstratlib::utils::setup_logger,
    optionstratlib::{ExpirationDate, Positive, pos},
    rust_decimal_macros::dec,
    std::error::Error,
};
use optionstratlib::OptionStyle;
use optionstratlib::strategies::base::Positionable;

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn test_short_strangle_with_greeks_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the ShortStrangle strategy
    let underlying_price = pos!(7138.5);

    let mut strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        pos!(7450.0),     // call_strike
        pos!(7050.0),     // put_strike
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        pos!(84.2),     // premium_short_call
        pos!(35.2),     // premium_short_put
        pos!(7.01),     // open_fee_short_call
        pos!(7.01),     // close_fee_short_call
        pos!(7.01),     // open_fee_short_put
        pos!(7.01),     // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::Center);
    let positions = strategy.get_positions()?;
    for position in positions {
        match position.option.option_style { 
            OptionStyle::Call => {
                assert!(position.option.strike_price >= underlying_price)
            },
            OptionStyle::Put => {
                assert!(position.option.strike_price <= underlying_price)
            },
        }
    }
    Ok(())
}
