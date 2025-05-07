use {
    optionstratlib::Side,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{BearPutSpread, FindOptimalSide},
    optionstratlib::utils::setup_logger,
    optionstratlib::{ExpirationDate, Positive, pos},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_bear_put_spread_integration() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Define inputs for the BearPutSpread strategy
    let underlying_price = pos!(5781.88);

    let mut strategy = BearPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5850.0),     // long_strike
        pos!(5720.0),     // short_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // long quantity
        pos!(85.04),    // premium_long
        pos!(29.85),    // premium_short
        pos!(0.78),     // open_fee_long
        pos!(0.78),     // open_fee_long
        pos!(0.73),     // close_fee_long
        pos!(0.73),     // close_fee_short
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.get_best_area(&option_chain, FindOptimalSide::Center);
    let positions = strategy.get_positions()?;
    for position in positions {
        match position.option.side {
            Side::Long => {
                assert!(position.option.strike_price >= underlying_price)
            }
            Side::Short => {
                assert!(position.option.strike_price <= underlying_price)
            }
        }
    }

    Ok(())
}
