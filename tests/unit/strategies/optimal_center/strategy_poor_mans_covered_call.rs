use {
use positive::pos_or_panic;
    optionstratlib::Side,
    optionstratlib::chains::chain::OptionChain,
    optionstratlib::strategies::base::Optimizable,
    optionstratlib::strategies::base::Positionable,
    optionstratlib::strategies::{FindOptimalSide, PoorMansCoveredCall},
    optionstratlib::{ExpirationDate, Positive},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_poor_mans_covered_call_integration() -> Result<(), Box<dyn Error>> {
    let underlying_price = pos_or_panic!(2703.3);

    let mut strategy = PoorMansCoveredCall::new(
        "GOLD".to_string(),                // underlying_symbol
        underlying_price,                  // underlying_price
        pos_or_panic!(2600.0),                      // long_call_strike
        pos_or_panic!(2800.0),                      // short_call_strike OTM
        ExpirationDate::Days(pos_or_panic!(120.0)), // long_call_expiration
        ExpirationDate::Days(pos_or_panic!(30.0)),  // short_call_expiration 30-45 days delta 0.30 or less
        pos_or_panic!(0.17),                        // implied_volatility
        dec!(0.05),                        // risk_free_rate
        Positive::ZERO,                    // dividend_yield
        Positive::TWO,                         // quantity
        pos_or_panic!(154.7),                       // premium_short_call
        pos_or_panic!(30.8),                        // premium_short_put
        pos_or_panic!(1.74),                        // open_fee_short_call
        pos_or_panic!(1.74),                        // close_fee_short_call
        pos_or_panic!(0.85),                        // open_fee_short_put
        pos_or_panic!(0.85),                        // close_fee_short_put
    );

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    strategy.get_best_area(&option_chain, FindOptimalSide::Center);
    let positions = strategy.get_positions()?;
    for position in positions {
        match position.option.side {
            Side::Long => {
                assert!(position.option.strike_price <= underlying_price)
            }
            Side::Short => {
                assert!(position.option.strike_price >= underlying_price)
            }
        }
    }

    Ok(())
}
