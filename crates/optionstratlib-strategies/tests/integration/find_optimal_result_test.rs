//! `Optimizable::find_optimal` reports its outcome (#793).
//!
//! A search over an empty chain has no candidate: every strategy with a
//! search returns `StrategyError::NoValidCandidate` and is left exactly as it
//! was. A strategy without a search returns a not-supported error. The
//! searches that find a candidate are covered, with their pinned results, by
//! the `optimal` and `optimal_center` suites.

use optionstratlib_core::error::OperationErrorKind;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Options, Side};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::FindOptimalSide;
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::{Optimizable, StrategyType};
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::protective_put::ProtectivePut;
use optionstratlib_strategies::strategies::utils::OptimizationCriteria;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, IronButterfly,
    IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle, LongStrangle,
    PoorMansCoveredCall, ShortButterflySpread, ShortCall, ShortPut, ShortStraddle, ShortStrangle,
};
use rust_decimal_macros::dec;
use serde::Serialize;

fn empty_chain() -> OptionChain {
    OptionChain::new(
        "TEST",
        Positive::HUNDRED,
        "2024-12-31".to_string(),
        None,
        None,
    )
}

fn snapshot<S: Serialize>(strategy: &S) -> serde_json::Value {
    serde_json::to_value(strategy).unwrap_or_else(|e| panic!("{e}"))
}

/// Every criterion and the two `get_best_*` wrappers report
/// `NoValidCandidate` and leave `strategy` unchanged.
fn assert_no_valid_candidate<S>(mut strategy: S, expected: StrategyType)
where
    S: Optimizable + Serialize,
{
    let chain = empty_chain();
    let before = snapshot(&strategy);
    let results = [
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio),
        strategy.find_optimal(&chain, FindOptimalSide::Center, OptimizationCriteria::Area),
        strategy.get_best_ratio(&chain, FindOptimalSide::Upper),
        strategy.get_best_area(&chain, FindOptimalSide::Lower),
    ];
    for result in results {
        match result {
            Err(StrategyError::NoValidCandidate { strategy }) => assert_eq!(strategy, expected),
            other => panic!("expected NoValidCandidate for {expected}, got {other:?}"),
        }
    }
    assert_eq!(snapshot(&strategy), before);
}

fn assert_not_supported<S>(mut strategy: S)
where
    S: Optimizable + Serialize,
{
    let chain = empty_chain();
    let before = snapshot(&strategy);
    match strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio) {
        Err(StrategyError::OperationError(OperationErrorKind::NotSupported {
            operation, ..
        })) => assert_eq!(operation, "find_optimal"),
        other => panic!("expected a not-supported error, got {other:?}"),
    }
    assert_eq!(snapshot(&strategy), before);
}

#[test]
fn test_find_optimal_empty_chain_verticals_no_valid_candidate() {
    assert_no_valid_candidate(BullCallSpread::default(), StrategyType::BullCallSpread);
    assert_no_valid_candidate(BearCallSpread::default(), StrategyType::BearCallSpread);
    assert_no_valid_candidate(BullPutSpread::default(), StrategyType::BullPutSpread);
    assert_no_valid_candidate(BearPutSpread::default(), StrategyType::BearPutSpread);
    assert_no_valid_candidate(BullCallLadder::default(), StrategyType::BullCallLadder);
    assert_no_valid_candidate(
        PoorMansCoveredCall::default(),
        StrategyType::PoorMansCoveredCall,
    );
}

#[test]
fn test_find_optimal_empty_chain_butterflies_and_condors_no_valid_candidate() {
    assert_no_valid_candidate(
        LongButterflySpread::default(),
        StrategyType::LongButterflySpread,
    );
    assert_no_valid_candidate(
        ShortButterflySpread::default(),
        StrategyType::ShortButterflySpread,
    );
    assert_no_valid_candidate(IronCondor::default(), StrategyType::IronCondor);
    assert_no_valid_candidate(IronButterfly::default(), StrategyType::IronButterfly);
}

#[test]
fn test_find_optimal_empty_chain_straddles_and_strangles_no_valid_candidate() {
    assert_no_valid_candidate(LongStraddle::default(), StrategyType::LongStraddle);
    assert_no_valid_candidate(ShortStraddle::default(), StrategyType::ShortStraddle);
    assert_no_valid_candidate(LongStrangle::default(), StrategyType::LongStrangle);
    // The short strangle applies the chain's expiration before searching:
    // the failed search restores the seed's.
    assert_no_valid_candidate(ShortStrangle::default(), StrategyType::ShortStrangle);
}

#[test]
fn test_find_optimal_empty_chain_custom_no_valid_candidate() {
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "TEST".to_string(),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let leg = Position::new(
        option,
        pos_or_panic!(1.2),
        chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
        Positive::ZERO,
        Positive::ZERO,
        None,
        None,
    );
    let strategy = CustomStrategy::new(
        "Custom".to_string(),
        "TEST".to_string(),
        "Long call".to_string(),
        Positive::HUNDRED,
        vec![leg],
        pos_or_panic!(0.001),
        100,
        pos_or_panic!(0.1),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_no_valid_candidate(strategy, StrategyType::Custom);
}

#[test]
fn test_find_optimal_without_search_not_supported() {
    assert_not_supported(LongCall::default());
    assert_not_supported(LongPut::default());
    assert_not_supported(ShortCall::default());
    assert_not_supported(ShortPut::default());
    // `ProtectivePut` keeps the trait's default implementation.
    let protective_put = ProtectivePut::new(
        "AAPL".to_string(),
        pos_or_panic!(150.0),
        pos_or_panic!(145.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.25),
        dec!(0.05),
        pos_or_panic!(0.01),
        Positive::HUNDRED,
        pos_or_panic!(3.5),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.65),
        pos_or_panic!(0.65),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_not_supported(protective_put);
}
