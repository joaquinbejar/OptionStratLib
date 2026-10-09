use optionstratlib_core::{model::Positive, pos_or_panic};
use {
    approx::assert_relative_eq,
    num_traits::ToPrimitive,
    optionstratlib_core::model::ExpirationDate,
    optionstratlib_market::chains::chain::OptionChain,
    optionstratlib_market::chains::utils::FindOptimalSide,
    optionstratlib_strategies::strategies::base::Optimizable,
    optionstratlib_strategies::strategies::{IronCondor, Strategies},
    rust_decimal_macros::dec,
    std::error::Error,
};

#[test]
fn test_iron_condor_integration() -> Result<(), Box<dyn Error>> {
    // Define inputs for the IronCondor strategy
    let underlying_price = pos_or_panic!(2646.9);

    let mut strategy = IronCondor::new(
        "GOLD".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(2725.0), // short_call_strike
        pos_or_panic!(2560.0), // short_put_strike
        pos_or_panic!(2800.0), // long_call_strike
        pos_or_panic!(2500.0), // long_put_strike
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.1548), // implied_volatility
        dec!(0.05),            // risk_free_rate
        Positive::ZERO,        // dividend_yield
        Positive::TWO,         // quantity
        pos_or_panic!(38.8),   // premium_short_call
        pos_or_panic!(30.4),   // premium_short_put
        pos_or_panic!(23.3),   // premium_long_call
        pos_or_panic!(16.8),   // premium_long_put
        pos_or_panic!(0.96),   // open_fee
        pos_or_panic!(0.96),   // close_fee
    )?;

    let option_chain = OptionChain::load_from_json(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    ))?;
    let input_fees = strategy.get_fees()?;
    strategy
        .get_best_area(&option_chain, FindOptimalSide::All)
        .unwrap();
    // Was 0.0818: each candidate was charged `quantity` times the fees, and
    // the search rebuilt candidates from its last improvement, so the fees
    // compounded to 16 times the input's (245.76) by the end (#875).
    assert_relative_eq!(
        strategy.get_profit_area().unwrap().to_f64().unwrap(),
        19.5799,
        epsilon = 0.001
    );
    assert_eq!(strategy.get_fees()?, input_fees);
    // Was `NoValidCandidate`, read as "no condor lies wholly above the
    // spot": the compounded fees (491.52 per candidate) rejected every
    // candidate. The 6100/6050/6200/5950 condor lies above the 5781.88 spot.
    strategy
        .get_best_ratio(&option_chain, FindOptimalSide::Upper)
        .unwrap();
    assert_relative_eq!(
        strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
        464.0158,
        epsilon = 0.001
    );
    assert_eq!(strategy.get_fees()?, input_fees);

    Ok(())
}
