//! The parallel optimisers pick what the serial search picked (#862).
//!
//! `find_optimal` used to walk `filter_combinations`, which built and checked
//! every candidate, then build each survivor again from `self` to score it,
//! replacing `self` on every strictly greater score. It now builds each
//! candidate once and scores it on the rayon pool, breaking ties on the
//! combination index. `serial_find_optimal` below is the old search, written
//! against the public trait, and every case asserts that both searches end
//! on the same strategy, or both find none, on the SP500 fixture chain and
//! on synthetic chains of growing size.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::StrategyLegs;
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::{
    FindOptimalSide, OptionChainBuildParams, OptionDataGroup, OptionDataPriceParams,
};
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::BasicAble;
use optionstratlib_strategies::strategies::base::Optimizable;
use optionstratlib_strategies::strategies::utils::OptimizationCriteria;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, IronButterfly,
    IronCondor, LongButterflySpread, LongStraddle, LongStrangle, ShortButterflySpread,
    ShortStraddle, ShortStrangle, Strategies,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;

const SP500_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
);

fn spot() -> Positive {
    pos_or_panic!(5781.88)
}

fn expiration() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

fn sp500_chain() -> OptionChain {
    OptionChain::load_from_json(SP500_JSON)
        .unwrap_or_else(|e| panic!("the SP500 fixture chain loads: {e}"))
}

/// A synthetic 30-day chain on the SP500 spot with `half_width` strikes
/// either side of the money, as the optimiser benches build it.
fn synthetic_chain(half_width: usize) -> OptionChain {
    let params = OptionChainBuildParams::new(
        "SP500".to_string(),
        Some(pos_or_panic!(100.0)),
        half_width,
        None,
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(spot())),
            Some(expiration()),
            Some(dec!(0.05)),
            Some(Positive::ZERO),
            Some("SP500".to_string()),
        ),
        pos_or_panic!(0.18),
    );
    OptionChain::build_chain(&params).unwrap_or_else(|e| panic!("the synthetic chain builds: {e}"))
}

/// The SP500 chain and synthetic chains of 11, 21 and 41 strikes.
fn chains() -> Vec<(String, OptionChain)> {
    let mut chains = vec![("sp500".to_string(), sp500_chain())];
    for half_width in [5, 10, 20] {
        let chain = synthetic_chain(half_width);
        chains.push((format!("synthetic_{}", chain.options.len()), chain));
    }
    chains
}

fn sides() -> Vec<FindOptimalSide> {
    vec![
        FindOptimalSide::All,
        FindOptimalSide::Upper,
        FindOptimalSide::Lower,
        FindOptimalSide::Center,
        FindOptimalSide::Range(pos_or_panic!(5600.0), pos_or_panic!(6000.0)),
        FindOptimalSide::Deltable(pos_or_panic!(0.3)),
        FindOptimalSide::DeltaRange(dec!(0.1), dec!(0.5)),
    ]
}

/// `OptimizationCriteria` is neither `Copy` nor `Debug`.
fn criteria(ratio: bool) -> OptimizationCriteria {
    if ratio {
        OptimizationCriteria::Ratio
    } else {
        OptimizationCriteria::Area
    }
}

/// How a search ended: `Ok`, no candidate, or another error.
type Outcome = Result<(), String>;

fn outcome(result: Result<(), StrategyError>) -> Outcome {
    match result {
        Ok(()) => Ok(()),
        Err(StrategyError::NoValidCandidate { .. }) => Err("no valid candidate".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// The search `find_optimal` ran before #862. `prepare` is the strategy's
/// own set-up before the search (the short strangle takes the chain's
/// expiration); a search that finds nothing leaves the strategy as it was.
fn serial_find_optimal<S>(
    strategy: &mut S,
    chain: &OptionChain,
    side: FindOptimalSide,
    criteria: OptimizationCriteria,
    prepare: impl Fn(&mut S, &OptionChain) -> Result<(), StrategyError>,
    to_legs: impl Fn(OptionDataGroup<'_>) -> Option<StrategyLegs<'_>>,
) -> Outcome
where
    S: Optimizable<Strategy = S> + Strategies + Clone,
{
    let original = strategy.clone();
    if let Err(e) = prepare(strategy, chain) {
        *strategy = original;
        return outcome(Err(e));
    }
    let mut best_value = Decimal::MIN;
    let mut found = false;
    let filter = strategy.clone();
    for group in filter.filter_combinations(chain, side) {
        let Some(legs) = to_legs(group) else {
            continue;
        };
        let Ok(candidate) = strategy.create_strategy(chain, &legs) else {
            continue;
        };
        let metric = match criteria {
            OptimizationCriteria::Ratio => candidate.get_profit_ratio(),
            OptimizationCriteria::Area => candidate.get_profit_area(),
        };
        let Ok(value) = metric else {
            continue;
        };
        if value > best_value {
            best_value = value;
            *strategy = candidate;
            found = true;
        }
    }
    if found {
        Ok(())
    } else {
        *strategy = original;
        Err("no valid candidate".to_string())
    }
}

/// The strategy as JSON without the positions' `date`, which records when
/// each leg was built (`Utc::now()`), not which leg the search chose.
fn undated<S: Debug>(strategy: &S) -> serde_json::Value {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("date");
                map.values_mut().for_each(strip);
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut value: serde_json::Value = serde_json::from_str(&format!("{strategy:?}"))
        .unwrap_or_else(|e| panic!("the strategy's Debug is JSON: {e}"));
    strip(&mut value);
    value
}

/// Runs both searches from `strategy` on every chain, side and criterion and
/// asserts they end the same way, on the same strategy. Returns how many
/// cases found one.
fn assert_identical<S>(
    label: &str,
    strategy: &S,
    prepare: impl Fn(&mut S, &OptionChain) -> Result<(), StrategyError> + Copy,
    to_legs: impl Fn(OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> + Copy,
) -> usize
where
    S: Optimizable<Strategy = S> + Strategies + Clone + Debug,
{
    let mut found_cases = 0;
    for (name, chain) in chains() {
        for side in sides() {
            for ratio in [true, false] {
                let context = format!(
                    "{label}, {name}, {side:?}, {}",
                    if ratio { "ratio" } else { "area" }
                );
                let mut serial = strategy.clone();
                let serial_outcome = serial_find_optimal(
                    &mut serial,
                    &chain,
                    side,
                    criteria(ratio),
                    prepare,
                    to_legs,
                );
                let mut parallel = strategy.clone();
                let parallel_outcome =
                    outcome(parallel.find_optimal(&chain, side, criteria(ratio)));
                assert_eq!(parallel_outcome, serial_outcome, "{context}: outcome");
                if parallel_outcome.is_ok() {
                    found_cases += 1;
                }
                assert_eq!(
                    undated(&parallel),
                    undated(&serial),
                    "{context}: chosen strategy"
                );
            }
        }
    }
    found_cases
}

/// No set-up before the search.
fn no_preparation<S>(_: &mut S, _: &OptionChain) -> Result<(), StrategyError> {
    Ok(())
}

fn iron_condor(quantity: Positive) -> IronCondor {
    IronCondor::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5900.0),
        pos_or_panic!(5650.0),
        pos_or_panic!(5950.0),
        pos_or_panic!(5600.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(42.0),
        pos_or_panic!(48.0),
        pos_or_panic!(28.0),
        pos_or_panic!(33.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the iron condor fixture builds: {e}"))
}

fn one_leg_twice(group: OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> {
    match group {
        OptionDataGroup::One(both) => Some(StrategyLegs::TwoLegs {
            first: both,
            second: both,
        }),
        _ => None,
    }
}

fn two_legs(group: OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> {
    match group {
        OptionDataGroup::Two(first, second) => Some(StrategyLegs::TwoLegs { first, second }),
        _ => None,
    }
}

fn three_legs(group: OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> {
    match group {
        OptionDataGroup::Three(first, second, third) => Some(StrategyLegs::ThreeLegs {
            first,
            second,
            third,
        }),
        _ => None,
    }
}

/// The iron butterfly's body: the middle strike twice.
fn three_legs_body_twice(group: OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> {
    match group {
        OptionDataGroup::Three(low, mid, high) => Some(StrategyLegs::FourLegs {
            first: low,
            second: mid,
            third: mid,
            fourth: high,
        }),
        _ => None,
    }
}

fn four_legs(group: OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> {
    match group {
        OptionDataGroup::Four(first, second, third, fourth) => Some(StrategyLegs::FourLegs {
            first,
            second,
            third,
            fourth,
        }),
        _ => None,
    }
}

/// Asserts both searches agree at quantities 1 and 2 and that at least one
/// case found a candidate, so the comparison is not vacuous. At quantity 2
/// the serial search rebuilt candidates from its last improvement; since
/// #875 a candidate keeps the input's fees, so that path cannot change the
/// result.
fn check<S>(
    make: impl Fn(Positive) -> S,
    prepare: impl Fn(&mut S, &OptionChain) -> Result<(), StrategyError> + Copy,
    to_legs: impl Fn(OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> + Copy,
) where
    S: Optimizable<Strategy = S> + Strategies + Clone + Debug,
{
    for quantity in [Positive::ONE, Positive::TWO] {
        let found = assert_identical(
            &format!("q = {quantity}"),
            &make(quantity),
            prepare,
            to_legs,
        );
        assert!(
            found > 0,
            "q = {quantity}: no case found a candidate, so nothing was compared"
        );
    }
}

#[test]
fn test_iron_condor_optimiser_is_unchanged() {
    check(iron_condor, no_preparation, four_legs);
}

fn bull_call_spread(quantity: Positive) -> BullCallSpread {
    BullCallSpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5750.0),
        pos_or_panic!(5820.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(85.04),
        pos_or_panic!(29.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    )
    .unwrap_or_else(|e| panic!("the bull call spread fixture builds: {e}"))
}

fn bear_call_spread(quantity: Positive) -> BearCallSpread {
    BearCallSpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5750.0),
        pos_or_panic!(5820.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(85.04),
        pos_or_panic!(29.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    )
    .unwrap_or_else(|e| panic!("the bear call spread fixture builds: {e}"))
}

fn bull_put_spread(quantity: Positive) -> BullPutSpread {
    BullPutSpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5750.0),
        pos_or_panic!(5920.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(15.04),
        pos_or_panic!(89.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    )
    .unwrap_or_else(|e| panic!("the bull put spread fixture builds: {e}"))
}

fn bear_put_spread(quantity: Positive) -> BearPutSpread {
    BearPutSpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5850.0),
        pos_or_panic!(5720.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(85.04),
        pos_or_panic!(29.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    )
    .unwrap_or_else(|e| panic!("the bear put spread fixture builds: {e}"))
}

fn bull_call_ladder(quantity: Positive) -> BullCallLadder {
    BullCallLadder::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5750.0),
        pos_or_panic!(5800.0),
        pos_or_panic!(5850.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(85.04),
        pos_or_panic!(53.04),
        pos_or_panic!(28.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
        pos_or_panic!(0.73),
    )
    .unwrap_or_else(|e| panic!("the bull call ladder fixture builds: {e}"))
}

fn long_butterfly_spread(quantity: Positive) -> LongButterflySpread {
    LongButterflySpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5700.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5860.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(120.0),
        pos_or_panic!(70.0),
        pos_or_panic!(35.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the long butterfly fixture builds: {e}"))
}

fn short_butterfly_spread(quantity: Positive) -> ShortButterflySpread {
    ShortButterflySpread::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5700.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(119.01),
        pos_or_panic!(66.0),
        pos_or_panic!(29.85),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the short butterfly fixture builds: {e}"))
}

fn iron_butterfly(quantity: Positive) -> IronButterfly {
    IronButterfly::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5780.0),
        pos_or_panic!(5880.0),
        pos_or_panic!(5680.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(95.0),
        pos_or_panic!(90.0),
        pos_or_panic!(50.0),
        pos_or_panic!(45.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the iron butterfly fixture builds: {e}"))
}

fn long_straddle(quantity: Positive) -> LongStraddle {
    LongStraddle::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5780.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(110.0),
        pos_or_panic!(105.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the long straddle fixture builds: {e}"))
}

fn short_straddle(quantity: Positive) -> ShortStraddle {
    ShortStraddle::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5780.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(110.0),
        pos_or_panic!(105.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the short straddle fixture builds: {e}"))
}

fn long_strangle(quantity: Positive) -> LongStrangle {
    LongStrangle::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5950.0),
        pos_or_panic!(5600.0),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(30.0),
        pos_or_panic!(35.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the long strangle fixture builds: {e}"))
}

fn short_strangle(quantity: Positive) -> ShortStrangle {
    ShortStrangle::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5950.0),
        pos_or_panic!(5600.0),
        expiration(),
        pos_or_panic!(0.18),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(30.0),
        pos_or_panic!(35.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
        pos_or_panic!(0.78),
    )
    .unwrap_or_else(|e| panic!("the short strangle fixture builds: {e}"))
}

/// The short strangle's own set-up: it takes the chain's expiration.
fn take_chain_expiration(
    strategy: &mut ShortStrangle,
    chain: &OptionChain,
) -> Result<(), StrategyError> {
    match chain.get_expiration() {
        Some(expiration) => strategy.set_expiration_date(expiration),
        None => Ok(()),
    }
}

#[test]
fn test_bull_call_spread_optimiser_is_unchanged() {
    check(bull_call_spread, no_preparation, two_legs);
}

#[test]
fn test_bear_call_spread_optimiser_is_unchanged() {
    check(bear_call_spread, no_preparation, two_legs);
}

#[test]
fn test_bull_put_spread_optimiser_is_unchanged() {
    check(bull_put_spread, no_preparation, two_legs);
}

#[test]
fn test_bear_put_spread_optimiser_is_unchanged() {
    check(bear_put_spread, no_preparation, two_legs);
}

#[test]
fn test_bull_call_ladder_optimiser_is_unchanged() {
    check(bull_call_ladder, no_preparation, three_legs);
}

#[test]
fn test_long_butterfly_spread_optimiser_is_unchanged() {
    check(long_butterfly_spread, no_preparation, three_legs);
}

#[test]
fn test_short_butterfly_spread_optimiser_is_unchanged() {
    check(short_butterfly_spread, no_preparation, three_legs);
}

#[test]
fn test_iron_butterfly_optimiser_is_unchanged() {
    check(iron_butterfly, no_preparation, three_legs_body_twice);
}

#[test]
fn test_long_straddle_optimiser_is_unchanged() {
    check(long_straddle, no_preparation, one_leg_twice);
}

#[test]
fn test_short_straddle_optimiser_is_unchanged() {
    check(short_straddle, no_preparation, one_leg_twice);
}

#[test]
fn test_long_strangle_optimiser_is_unchanged() {
    check(long_strangle, no_preparation, two_legs);
}

#[test]
fn test_short_strangle_optimiser_is_unchanged() {
    check(short_strangle, take_chain_expiration, two_legs);
}
