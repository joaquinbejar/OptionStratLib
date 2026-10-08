use optionstratlib_core::{model::Positive, pos_or_panic};
use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib_core::model::ExpirationDate,
    optionstratlib_market::chains::chain::OptionChain,
    optionstratlib_market::chains::utils::FindOptimalSide,
    optionstratlib_strategies::strategies::base::Optimizable,
    optionstratlib_strategies::strategies::{BullCallLadder, Strategies},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_bull_call_ladder_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the BullCallLadder strategy
    let underlying_price = pos_or_panic!(5781.88);

    let mut strategy = BullCallLadder::new(
        "SP500".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(5750.0), // long_call_strike
        pos_or_panic!(5800.0), // short_call_low_strike
        pos_or_panic!(5850.0), // short_call_high_strike
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),  // implied_volatility
        dec!(0.05),           // risk_free_rate
        Positive::ZERO,       // dividend_yield
        Positive::ONE,        // long quantity
        pos_or_panic!(85.04), // premium_long_itm
        pos_or_panic!(53.04), // premium_long_otm
        pos_or_panic!(28.85), // premium_short
        pos_or_panic!(0.78),  // premium_short
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.78),  // close_fee_long
        pos_or_panic!(0.73),  // close_fee_short
        pos_or_panic!(0.73),  // close_fee_short
        pos_or_panic!(0.73),  // open_fee_short
    )?;

    let option_chain = OptionChain::load_from_json(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    ))?;
    strategy
        .get_best_area(&option_chain, FindOptimalSide::All)
        .unwrap();
    assert_relative_eq!(
        strategy.get_profit_area().unwrap().to_f64().unwrap(),
        75286.2704,
        epsilon = 0.001
    );
    strategy
        .get_best_ratio(&option_chain, FindOptimalSide::Upper)
        .unwrap();
    assert_relative_eq!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
        19628.0,
        epsilon = 0.001
    );

    Ok(())
}
