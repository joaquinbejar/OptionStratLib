use positive::pos_or_panic;
use {
    optionstratlib::Side,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{BearCallSpread, FindOptimalSide},
    optionstratlib::{ExpirationDate, Positive},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_bear_call_spread_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the BearCallSpread strategy
    let underlying_price = pos_or_panic!(5781.88);
    let mut strategy = BearCallSpread::new(
        "SP500".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(5750.0), // long_strike_itm
        pos_or_panic!(5820.0), // short_strike
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),  // implied_volatility
        dec!(0.05),           // risk_free_rate
        Positive::ZERO,       // dividend_yield
        Positive::TWO,        // long quantity
        pos_or_panic!(85.04), // premium_long
        pos_or_panic!(29.85), // premium_short
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.73),  // close_fee_long
        pos_or_panic!(0.73),  // close_fee_short
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
