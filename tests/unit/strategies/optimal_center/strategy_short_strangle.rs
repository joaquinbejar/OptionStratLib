use positive::{Positive, pos_or_panic};
use {
    optionstratlib::ExpirationDate,
    optionstratlib::OptionStyle,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{FindOptimalSide, ShortStrangle},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[cfg_attr(target_os = "windows", ignore)]
#[test]
fn test_short_strangle_with_greeks_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the ShortStrangle strategy
    let underlying_price = pos_or_panic!(24209.5);

    let mut strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price,       // underlying_price
        pos_or_panic!(24100.0), // call_strike
        pos_or_panic!(24300.0), // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745), // implied_volatility
        pos_or_panic!(0.3745), // implied_volatility
        dec!(0.05),            // risk_free_rate
        Positive::ZERO,        // dividend_yield
        Positive::ONE,         // quantity
        pos_or_panic!(84.2),   // premium_short_call
        pos_or_panic!(35.2),   // premium_short_put
        pos_or_panic!(0.1),    // open_fee_short_call
        pos_or_panic!(0.1),    // close_fee_short_call
        pos_or_panic!(0.1),    // open_fee_short_put
        pos_or_panic!(0.1),    // close_fee_short_put
    );

    let option_chain = OptionChain::load_from_json(
        "./examples/Chains/Germany-40-2025-05-27-15-29-00-UTC-24209.json",
    )?;
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
