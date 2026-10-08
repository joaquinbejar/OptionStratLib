//! `CustomStrategy` break-evens refreshed by its edits (#784).
//!
//! `add_position`, `modify_position` and `replace_position` leave the
//! strategy with the break-evens of the same legs built through `new`. An
//! edit whose recomputation fails, or that leaves the strategy invalid, is
//! rejected with the strategy left exactly as it was.
//!
//! `find_optimal` skips a candidate whose break-evens cannot be recomputed
//! and, when the best one cannot be applied, leaves the strategy as it was
//! (#791) and reports the error (#793).

use chrono::{DateTime, Utc};
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::{pos_or_panic, spos};
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::FindOptimalSide;
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::{
    BreakEvenable, Optimizable, Positionable, StrategyType,
};
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::utils::OptimizationCriteria;
use rust_decimal_macros::dec;

fn leg(side: Side, style: OptionStyle, strike: f64, premium: Positive) -> Position {
    let option = Options::new(
        OptionType::European,
        side,
        "TEST".to_string(),
        pos_or_panic!(strike),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        style,
        Positive::ZERO,
        None,
    );
    Position::new(
        option,
        premium,
        DateTime::<Utc>::UNIX_EPOCH,
        pos_or_panic!(0.05),
        pos_or_panic!(0.05),
        None,
        None,
    )
}

fn long_call(premium: Positive) -> Position {
    leg(Side::Long, OptionStyle::Call, 105.0, premium)
}

fn short_put(premium: Positive) -> Position {
    leg(Side::Short, OptionStyle::Put, 95.0, premium)
}

fn custom(positions: Vec<Position>) -> CustomStrategy {
    CustomStrategy::new(
        "Custom".to_string(),
        "TEST".to_string(),
        "Long call and short put".to_string(),
        Positive::HUNDRED,
        positions,
        pos_or_panic!(0.001),
        100,
        pos_or_panic!(0.1),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn break_evens(strategy: &CustomStrategy) -> Vec<Positive> {
    strategy
        .get_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"))
        .clone()
}

fn snapshot(strategy: &CustomStrategy) -> serde_json::Value {
    serde_json::to_value(strategy).unwrap_or_else(|e| panic!("{e}"))
}

fn base() -> CustomStrategy {
    custom(vec![
        long_call(pos_or_panic!(3.0)),
        short_put(pos_or_panic!(2.0)),
    ])
}

#[test]
fn test_custom_strategy_modify_position_refreshes_break_evens() {
    let mut strategy = base();
    let edited = custom(vec![
        long_call(pos_or_panic!(4.0)),
        short_put(pos_or_panic!(2.0)),
    ]);
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&long_call(pos_or_panic!(4.0)))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_custom_strategy_add_position_refreshes_break_evens() {
    let mut strategy = base();
    let added = leg(Side::Short, OptionStyle::Call, 115.0, pos_or_panic!(1.0));
    let edited = custom(vec![
        long_call(pos_or_panic!(3.0)),
        short_put(pos_or_panic!(2.0)),
        added.clone(),
    ]);
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .add_position(&added)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_custom_strategy_replace_position_refreshes_break_evens() {
    let mut strategy = base();
    let edited = custom(vec![
        long_call(pos_or_panic!(3.0)),
        short_put(pos_or_panic!(1.0)),
    ]);
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .replace_position(&short_put(pos_or_panic!(1.0)))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_custom_strategy_failed_recomputation_leaves_strategy_unchanged() {
    // The scan walks half to one and a half times the underlying a cent at
    // a time; at a million that is more samples than it will walk, so every
    // edit fails to recompute the break-evens.
    let mut strategy = base();
    strategy.underlying_price = pos_or_panic!(1_000_000.0);
    let before = snapshot(&strategy);
    let before_break_evens = break_evens(&strategy);
    assert!(!before_break_evens.is_empty());

    assert!(
        strategy
            .add_position(&leg(Side::Short, OptionStyle::Call, 115.0, Positive::ONE))
            .is_err()
    );
    assert_eq!(snapshot(&strategy), before);
    assert!(
        strategy
            .modify_position(&long_call(pos_or_panic!(4.0)))
            .is_err()
    );
    assert_eq!(snapshot(&strategy), before);
    assert!(
        strategy
            .replace_position(&short_put(pos_or_panic!(1.0)))
            .is_err()
    );
    assert_eq!(snapshot(&strategy), before);
    assert_eq!(break_evens(&strategy), before_break_evens);
}

#[test]
fn test_custom_strategy_invalid_edit_rejected_and_unchanged() {
    // A short leg without premium fails `Position::validate`. Custom
    // strategies have always rejected such an edit; it is now also undone.
    let mut strategy = base();
    let before = snapshot(&strategy);

    let error = strategy
        .add_position(&leg(Side::Short, OptionStyle::Call, 115.0, Positive::ZERO))
        .expect_err("an invalid leg is rejected");
    assert!(
        error
            .to_string()
            .contains("Strategy is not valid after adding new position")
    );
    assert_eq!(snapshot(&strategy), before);

    let error = strategy
        .modify_position(&short_put(Positive::ZERO))
        .expect_err("an invalid leg is rejected");
    assert!(
        error
            .to_string()
            .contains("Strategy is not valid after modifying position")
    );
    assert_eq!(snapshot(&strategy), before);

    let error = strategy
        .replace_position(&short_put(Positive::ZERO))
        .expect_err("an invalid leg is rejected");
    assert!(
        error
            .to_string()
            .contains("Strategy is not valid after replacing position")
    );
    assert_eq!(snapshot(&strategy), before);
}

#[test]
fn test_custom_strategy_find_optimal_failed_recomputation_leaves_strategy_unchanged() {
    // With every recomputation failing no candidate is eligible: the search
    // reports it and leaves the strategy as it was.
    let mut chain = OptionChain::new(
        "TEST",
        Positive::HUNDRED,
        "2024-12-31".to_string(),
        None,
        None,
    );
    for (strike, call, put) in [(95.0, 7.0, 2.6), (100.0, 3.5, 4.1), (105.0, 1.0, 6.6)] {
        chain.add_option(
            pos_or_panic!(strike),
            spos!(call),
            spos!(call + 0.2),
            spos!(put),
            spos!(put + 0.2),
            pos_or_panic!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
            None,
        );
    }
    let mut strategy = base();
    strategy.underlying_price = pos_or_panic!(1_000_000.0);
    let before = snapshot(&strategy);
    let before_break_evens = break_evens(&strategy);
    assert!(!before_break_evens.is_empty());

    let result = strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

    assert!(matches!(
        result,
        Err(StrategyError::NoValidCandidate {
            strategy: StrategyType::Custom
        })
    ));

    assert_eq!(snapshot(&strategy), before);
    assert_eq!(break_evens(&strategy), before_break_evens);
}
