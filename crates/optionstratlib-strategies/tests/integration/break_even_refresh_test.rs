//! Break-evens refreshed by `add_position` / `modify_position` (#780).
//!
//! For one strategy of each family, an edited leg leaves the strategy with
//! the break-evens of the same legs built through `new`, and an edit whose
//! recomputation fails (a premium total that overflows) is rejected with
//! the strategy left exactly as it was.

use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::Side;
use optionstratlib_core::pos_or_panic;
use optionstratlib_strategies::strategies::base::{BreakEvenable, Positionable};
use optionstratlib_strategies::strategies::{
    BullCallLadder, BullCallSpread, IronCondor, LongButterflySpread, LongCall, LongStraddle,
    PoorMansCoveredCall, ShortStrangle,
};
use rust_decimal_macros::dec;
use serde::Serialize;

fn expiry() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

fn fee() -> Positive {
    pos_or_panic!(0.05)
}

fn break_evens<S: BreakEvenable>(strategy: &S) -> Vec<Positive> {
    strategy
        .get_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"))
        .clone()
}

/// The leg of `side`, for strategies whose legs are not public; each
/// strategy using this holds one leg per side.
fn leg<S: Positionable>(strategy: &S, side: Side) -> Position {
    strategy
        .get_positions()
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .find(|position| position.option.side == side)
        .unwrap_or_else(|| panic!("no {side:?} leg"))
        .clone()
}

fn snapshot<S: Serialize>(strategy: &S) -> serde_json::Value {
    serde_json::to_value(strategy).unwrap_or_else(|e| panic!("{e}"))
}

/// Replaces `leg` in `strategy` with a copy whose premium total, two
/// contracts at `Positive::MAX`, overflows, through `modify_position`, and checks the edit is rejected with the
/// strategy (legs and break-evens) unchanged.
fn assert_failed_edit_leaves_unchanged<S>(strategy: &mut S, leg: &Position)
where
    S: Positionable + BreakEvenable + Serialize,
{
    let before = snapshot(strategy);
    let before_break_evens = break_evens(strategy);
    assert!(!before_break_evens.is_empty());
    let mut unpriceable = leg.clone();
    unpriceable.premium = Positive::MAX;
    unpriceable.option.quantity = Positive::TWO;
    assert!(strategy.modify_position(&unpriceable).is_err());
    assert_eq!(snapshot(strategy), before);
    assert_eq!(break_evens(strategy), before_break_evens);
}

fn bull_call_spread(long_strike: f64, premium_long: Positive) -> BullCallSpread {
    BullCallSpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(long_strike),
        pos_or_panic!(105.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        premium_long,
        pos_or_panic!(1.5),
        fee(),
        fee(),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_bull_call_spread_modify_position_refreshes_break_evens() {
    let mut strategy = bull_call_spread(95.0, pos_or_panic!(6.0));
    let edited = bull_call_spread(95.0, pos_or_panic!(7.0));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&edited.long_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_bull_call_spread_add_position_refreshes_break_evens() {
    // A new long strike goes through `add_position`, which replaces the
    // long leg rather than matching it by strike.
    let mut strategy = bull_call_spread(95.0, pos_or_panic!(6.0));
    let edited = bull_call_spread(90.0, pos_or_panic!(10.0));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .add_position(&edited.long_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_bull_call_spread_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = bull_call_spread(95.0, pos_or_panic!(6.0));
    let leg = strategy.long_call.clone();
    assert_failed_edit_leaves_unchanged(&mut strategy, &leg);
}

#[test]
fn test_bull_call_spread_assembled_from_default_matches_new() {
    // Leg by leg from `Default`: no break-evens while a leg is still a
    // placeholder, those of `new` once both legs are set.
    let built = bull_call_spread(95.0, pos_or_panic!(6.0));
    let mut strategy = BullCallSpread::default();
    strategy
        .add_position(&built.short_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(break_evens(&strategy).is_empty());
    strategy
        .add_position(&built.long_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&built));
}

#[test]
fn test_bull_call_spread_invalid_edit_clears_break_evens() {
    // A long strike above the short one is not a bull call spread:
    // `add_position` accepts it, as it always has, and reports no
    // break-evens instead of the previous legs'.
    let mut strategy = bull_call_spread(95.0, pos_or_panic!(6.0));
    let mut inverted = strategy.long_call.clone();
    inverted.option.strike_price = pos_or_panic!(110.0);
    strategy
        .add_position(&inverted)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(strategy.long_call.option.strike_price, pos_or_panic!(110.0));
    assert!(break_evens(&strategy).is_empty());
}

fn iron_condor(premium_short_call: Positive) -> IronCondor {
    IronCondor::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        pos_or_panic!(95.0),
        pos_or_panic!(110.0),
        pos_or_panic!(90.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        premium_short_call,
        pos_or_panic!(1.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_iron_condor_modify_position_refreshes_break_evens() {
    let mut strategy = iron_condor(pos_or_panic!(1.5));
    let edited = iron_condor(pos_or_panic!(2.5));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&edited.short_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_iron_condor_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = iron_condor(pos_or_panic!(1.5));
    let leg = strategy.short_call.clone();
    assert_failed_edit_leaves_unchanged(&mut strategy, &leg);
}

fn long_butterfly(premium_middle: Positive) -> LongButterflySpread {
    LongButterflySpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(90.0),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(11.0),
        premium_middle,
        pos_or_panic!(0.5),
        fee(),
        fee(),
        fee(),
        fee(),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_long_butterfly_modify_position_refreshes_break_evens() {
    let mut strategy = long_butterfly(pos_or_panic!(4.0));
    let edited = long_butterfly(pos_or_panic!(4.5));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&edited.short_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_long_butterfly_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = long_butterfly(pos_or_panic!(4.0));
    let leg = strategy.short_call.clone();
    assert_failed_edit_leaves_unchanged(&mut strategy, &leg);
}

fn bull_call_ladder(premium_long: Positive) -> BullCallLadder {
    BullCallLadder::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        pos_or_panic!(110.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        premium_long,
        pos_or_panic!(2.0),
        pos_or_panic!(1.0),
        fee(),
        fee(),
        fee(),
        fee(),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_bull_call_ladder_modify_position_refreshes_break_evens() {
    let mut strategy = bull_call_ladder(pos_or_panic!(6.0));
    let edited = bull_call_ladder(pos_or_panic!(5.0));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&edited.long_call)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_bull_call_ladder_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = bull_call_ladder(pos_or_panic!(6.0));
    let leg = strategy.long_call.clone();
    assert_failed_edit_leaves_unchanged(&mut strategy, &leg);
}

fn long_straddle(premium_put: Positive) -> LongStraddle {
    LongStraddle::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        Positive::HUNDRED,
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(3.0),
        premium_put,
        fee(),
        fee(),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_long_straddle_modify_position_refreshes_break_evens() {
    let mut strategy = long_straddle(pos_or_panic!(2.5));
    let edited = long_straddle(pos_or_panic!(3.5));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&edited.long_put)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_long_straddle_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = long_straddle(pos_or_panic!(2.5));
    let leg = strategy.long_put.clone();
    assert_failed_edit_leaves_unchanged(&mut strategy, &leg);
}

fn short_strangle(put_strike: f64, premium_put: Positive) -> ShortStrangle {
    ShortStrangle::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
        pos_or_panic!(put_strike),
        expiry(),
        pos_or_panic!(0.2),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(1.5),
        premium_put,
        fee(),
        fee(),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_short_strangle_add_position_refreshes_break_evens() {
    let mut strategy = short_strangle(90.0, pos_or_panic!(1.2));
    let edited = short_strangle(85.0, pos_or_panic!(0.6));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .add_position(&edited.short_put)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_short_strangle_failed_edit_leaves_strategy_unchanged() {
    // `modify_position` already recomputed the break-evens here, but left
    // the new leg in place when that failed.
    let mut strategy = short_strangle(90.0, pos_or_panic!(1.2));
    let leg = strategy.short_put.clone();
    assert_failed_edit_leaves_unchanged(&mut strategy, &leg);
}

fn poor_mans_covered_call(premium_short: Positive) -> PoorMansCoveredCall {
    PoorMansCoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(80.0),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(365.0)),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(24.0),
        premium_short,
        fee(),
        fee(),
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_poor_mans_covered_call_modify_position_refreshes_break_evens() {
    let mut strategy = poor_mans_covered_call(pos_or_panic!(1.5));
    let edited = poor_mans_covered_call(pos_or_panic!(2.5));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&leg(&edited, Side::Short))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_poor_mans_covered_call_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = poor_mans_covered_call(pos_or_panic!(1.5));
    let short_call = leg(&strategy, Side::Short);
    assert_failed_edit_leaves_unchanged(&mut strategy, &short_call);
}

fn long_call(premium: Positive) -> LongCall {
    LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        expiry(),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        premium,
        fee(),
        fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_long_call_modify_position_refreshes_break_evens() {
    let mut strategy = long_call(pos_or_panic!(4.5));
    let edited = long_call(pos_or_panic!(5.5));
    assert_ne!(break_evens(&strategy), break_evens(&edited));
    strategy
        .modify_position(&leg(&edited, Side::Long))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&edited));
}

#[test]
fn test_long_call_failed_edit_leaves_strategy_unchanged() {
    let mut strategy = long_call(pos_or_panic!(4.5));
    let call = leg(&strategy, Side::Long);
    assert_failed_edit_leaves_unchanged(&mut strategy, &call);
}

#[test]
fn test_long_call_added_to_default_matches_new() {
    // `new` used to return no break-even at all: the strike plus the
    // premium and fees per contract, 100 + 4.5 + 0.1.
    let built = long_call(pos_or_panic!(4.5));
    assert_eq!(break_evens(&built), vec![pos_or_panic!(104.6)]);
    let mut strategy = LongCall::default();
    strategy
        .add_position(&leg(&built, Side::Long))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(break_evens(&strategy), break_evens(&built));
}
