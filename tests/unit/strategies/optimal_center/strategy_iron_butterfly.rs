use {
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{FindOptimalSide, IronButterfly},
    optionstratlib::utils::setup_logger,
    optionstratlib::{ExpirationDate, Positive, pos},
    optionstratlib::{OptionStyle, Side, assert_pos_relative_eq},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]

fn test_iron_butterfly_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the IronButterfly strategy
    let underlying_price = pos!(5672.9);

    let mut strategy = IronButterfly::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5672.0),     // short_call_strike
        pos!(5672.0),     // long_call_strike
        pos!(5672.0),     // long_put_strike
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.1548),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // quantity
        pos!(38.8),     // premium_short_call
        pos!(30.4),     // premium_short_put
        pos!(23.3),     // premium_long_call
        pos!(16.8),     // premium_long_put
        pos!(0.96),     // open_fee
        pos!(0.96),     // close_fee
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.best_area(&option_chain, FindOptimalSide::Center);
    let positions = strategy.get_positions()?;
    for position in positions {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Long) => {
                assert!(position.option.strike_price >= underlying_price)
            }
            (OptionStyle::Put, Side::Long) => {
                assert!(position.option.strike_price <= underlying_price)
            }
            _ => {
                let atm_strike = option_chain.atm_strike()?;
                assert_pos_relative_eq!(position.option.strike_price, *atm_strike, pos!(0.01));
            }
        }
    }

    Ok(())
}
