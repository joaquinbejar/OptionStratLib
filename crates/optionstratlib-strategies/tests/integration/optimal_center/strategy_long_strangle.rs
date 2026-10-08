use optionstratlib_core::{model::Positive, pos_or_panic};
use {
    optionstratlib_core::model::ExpirationDate, optionstratlib_core::model::OptionStyle,
    optionstratlib_market::chains::chain::OptionChain,
    optionstratlib_market::chains::utils::FindOptimalSide,
    optionstratlib_strategies::error::StrategyError,
    optionstratlib_strategies::strategies::LongStrangle,
    optionstratlib_strategies::strategies::base::Optimizable,
    optionstratlib_strategies::strategies::base::Positionable,
    optionstratlib_strategies::strategies::base::StrategyType, rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_long_strangle_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the LongStrangle strategy
    let underlying_price = pos_or_panic!(7138.5);

    let mut strategy = LongStrangle::new(
        "CL".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(7450.0), // call_strike
        pos_or_panic!(7050.0), // put_strike
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745), // implied_volatility
        dec!(0.05),            // risk_free_rate
        Positive::ZERO,        // dividend_yield
        Positive::ONE,         // quantity
        pos_or_panic!(84.2),   // premium_short_call
        pos_or_panic!(353.2),  // premium_short_put
        pos_or_panic!(7.0),    // open_fee_short_call
        pos_or_panic!(7.01),   // close_fee_short_call
        pos_or_panic!(7.01),   // open_fee_short_put
        pos_or_panic!(7.01),   // close_fee_short_put
    )?;

    let option_chain = OptionChain::load_from_json(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    ))?;
    // No centred long strangle around the seed spot is valid on this chain:
    // the search reports it and keeps the seed legs.
    let result = strategy.get_best_area(&option_chain, FindOptimalSide::Center);
    assert!(matches!(
        result,
        Err(StrategyError::NoValidCandidate {
            strategy: StrategyType::LongStrangle
        })
    ));
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
