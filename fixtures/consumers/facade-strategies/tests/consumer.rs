//! A downstream strategy workflow on the `optionstratlib` facade with
//! `default-features = false, features = ["strategies"]` (#535): the
//! convenience `prelude` for the common path, the canonical module paths for
//! the rest, and proof that both name the items the strategies crate defines.
//! No simulation, backtesting, I/O, async or charting code is compiled.

use optionstratlib::prelude::*;

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

fn bull_call_spread() -> BullCallSpread {
    match BullCallSpread::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5750.0),
        pos_or_panic!(5820.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::TWO,
        pos_or_panic!(85.04),
        pos_or_panic!(29.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    ) {
        Ok(strategy) => strategy,
        Err(error) => panic!("BullCallSpread::new: {error}"),
    }
}

/// The same figures the strategies crate's own bull call spread regression
/// pins, read here through the facade.
#[test]
fn test_bull_call_spread_through_the_prelude() {
    let strategy = bull_call_spread();
    match strategy.get_break_even_points() {
        Ok(points) => assert_eq!(points.len(), 1),
        Err(error) => panic!("break-even points: {error}"),
    }
    match strategy.get_max_loss() {
        Ok(loss) => assert_eq!(loss, pos_or_panic!(116.42)),
        Err(error) => panic!("max loss: {error}"),
    }
    match strategy.get_total_cost() {
        Ok(cost) => assert_eq!(cost, pos_or_panic!(176.12)),
        Err(error) => panic!("total cost: {error}"),
    }
    match strategy.get_fees() {
        Ok(fees) => assert_eq!(fees, pos_or_panic!(6.04)),
        Err(error) => panic!("fees: {error}"),
    }
}

#[test]
fn test_strategy_probability_through_the_prelude() {
    match strategy_probability(&bull_call_spread()) {
        Ok(probability) => assert!(probability > Positive::ZERO && probability < Positive::ONE),
        Err(error) => panic!("probability of profit: {error}"),
    }
}

fn strategy_probability(strategy: &BullCallSpread) -> Result<Positive, ProbabilityError> {
    strategy.probability_of_profit(None, None)
}

#[test]
fn test_facade_paths_are_the_strategies_items() {
    same_item(
        optionstratlib::strategies::BullCallSpread::new,
        optionstratlib_strategies::strategies::BullCallSpread::new,
    );
    let strategy: optionstratlib_strategies::strategies::BullCallSpread = bull_call_spread();
    let error: optionstratlib_strategies::error::StrategyError =
        optionstratlib::error::StrategyError::NotImplemented;
    assert!(!strategy.get_title().is_empty());
    assert!(!error.to_string().is_empty());
}
