//! `create_strategy` keeps the fees of the strategy it rebuilds from (#875).
//!
//! The iron condor and the iron butterfly split `get_fees()`, already scaled
//! by the quantity, into the per-contract fee their constructors scale by
//! the quantity again, so a candidate carried `quantity` times the fees.
//! `find_optimal` rebuilt each candidate from its last improvement, so the
//! fees compounded along the search and the result depended on the order
//! the candidates were met.
//!
//! Every strategy with a `create_strategy` is checked here at quantities 1,
//! 2 and 5, with a different fee on every leg so a fee moved to a leg of
//! another quantity changes the total: the rebuilt strategy's `get_fees()`
//! is the total the legs' fees and quantities give. At quantity 2 every
//! optimiser's search keeps the input's fees and ends on the strategy a
//! search building every candidate from the input picks, whatever path the
//! search took to it.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::StrategyLegs;
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::{FindOptimalSide, OptionDataGroup};
use optionstratlib_strategies::strategies::base::Optimizable;
use optionstratlib_strategies::strategies::utils::OptimizationCriteria;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, IronButterfly,
    IronCondor, LongButterflySpread, LongStraddle, LongStrangle, PoorMansCoveredCall,
    ShortButterflySpread, ShortStraddle, ShortStrangle, Strategies,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::fmt::Debug;

const SP500_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
);

const QUANTITIES: [Decimal; 3] = [Decimal::ONE, Decimal::TWO, dec!(5)];

fn chain() -> OptionChain {
    OptionChain::load_from_json(SP500_JSON)
        .unwrap_or_else(|e| panic!("the SP500 fixture chain loads: {e}"))
}

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or_else(|e| panic!("{value} is not positive: {e}"))
}

fn spot() -> Positive {
    pos_or_panic!(5781.88)
}

fn expiration() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

/// `OptimizationCriteria` is neither `Copy` nor `Debug`.
fn criteria(ratio: bool) -> OptimizationCriteria {
    if ratio {
        OptimizationCriteria::Ratio
    } else {
        OptimizationCriteria::Area
    }
}

/// The strategy as JSON without the positions' `date`, which records when
/// each leg was built (`Utc::now()`), not which leg was chosen.
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

/// Asserts that a strategy built at every quantity carries
/// `fees_per_unit × quantity`, and that `create_strategy` rebuilds it with
/// the same total on the first combination its own filter yields.
fn check_create_strategy<S>(
    make: impl Fn(Positive) -> S,
    to_legs: impl Fn(OptionDataGroup<'_>) -> Option<StrategyLegs<'_>>,
    fees_per_unit: Decimal,
) where
    S: Optimizable<Strategy = S> + Strategies,
{
    let chain = chain();
    for quantity in QUANTITIES {
        let strategy = make(pos(quantity));
        let group = strategy
            .filter_combinations(&chain, FindOptimalSide::All)
            .next()
            .unwrap_or_else(|| panic!("q = {quantity}: the chain yields a combination"));
        let legs = to_legs(group)
            .unwrap_or_else(|| panic!("q = {quantity}: the combination maps to legs"));
        assert_rebuilt_fees(&strategy, &chain, &legs, fees_per_unit * quantity, quantity);
    }
}

/// Asserts that `strategy` carries `expected` fees and that rebuilding it
/// on `legs` keeps them.
fn assert_rebuilt_fees<S>(
    strategy: &S,
    chain: &OptionChain,
    legs: &StrategyLegs<'_>,
    expected: Decimal,
    quantity: Decimal,
) where
    S: Optimizable<Strategy = S> + Strategies,
{
    let input_fees = strategy
        .get_fees()
        .unwrap_or_else(|e| panic!("q = {quantity}: input fees: {e}"));
    assert_eq!(input_fees.to_dec(), expected, "q = {quantity}: input fees");
    let rebuilt = strategy
        .create_strategy(chain, legs)
        .unwrap_or_else(|e| panic!("q = {quantity}: create_strategy: {e}"));
    let rebuilt_fees = rebuilt
        .get_fees()
        .unwrap_or_else(|e| panic!("q = {quantity}: rebuilt fees: {e}"));
    assert_eq!(
        rebuilt_fees.to_dec(),
        expected,
        "q = {quantity}: rebuilt fees"
    );
}

/// The best candidate when every one is built from `strategy` itself: the
/// search with no path, so no fee can drift along it.
fn path_free_best<S>(
    strategy: &S,
    chain: &OptionChain,
    ratio: bool,
    to_legs: impl Fn(OptionDataGroup<'_>) -> Option<StrategyLegs<'_>>,
) -> Option<S>
where
    S: Optimizable<Strategy = S> + Strategies,
{
    let mut best = None;
    let mut best_value = Decimal::MIN;
    for group in strategy.filter_combinations(chain, FindOptimalSide::All) {
        let Some(legs) = to_legs(group) else {
            continue;
        };
        let Ok(candidate) = strategy.create_strategy(chain, &legs) else {
            continue;
        };
        let metric = if ratio {
            candidate.get_profit_ratio()
        } else {
            candidate.get_profit_area()
        };
        if let Ok(value) = metric
            && value > best_value
        {
            best_value = value;
            best = Some(candidate);
        }
    }
    best
}

/// At quantity 2, both searches keep the input's fees and end on the
/// strategy the path-free search picks.
fn check_optimiser<S>(
    make: impl Fn(Positive) -> S,
    to_legs: impl Fn(OptionDataGroup<'_>) -> Option<StrategyLegs<'_>> + Copy,
) where
    S: Optimizable<Strategy = S> + Strategies + Clone + Debug,
{
    let chain = chain();
    let input = make(Positive::TWO);
    let input_fees = input
        .get_fees()
        .unwrap_or_else(|e| panic!("input fees: {e}"));
    for ratio in [true, false] {
        let label = if ratio { "ratio" } else { "area" };
        let mut searched = input.clone();
        searched
            .find_optimal(&chain, FindOptimalSide::All, criteria(ratio))
            .unwrap_or_else(|e| panic!("{label}: the search finds a candidate: {e}"));
        assert_eq!(
            searched
                .get_fees()
                .unwrap_or_else(|e| panic!("{label}: fees: {e}")),
            input_fees,
            "{label}: the search keeps the input's fees"
        );
        let expected = path_free_best(&input, &chain, ratio, to_legs)
            .unwrap_or_else(|| panic!("{label}: the path-free search finds a candidate"));
        assert_eq!(
            undated(&searched),
            undated(&expected),
            "{label}: the search ends where the path-free search does"
        );
    }
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

// Fixtures. Every leg's open and close fee differs, so a fee handed to a
// leg of another quantity (a butterfly's body holds twice the quantity of
// its wings) changes the total.

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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
        pos_or_panic!(0.50),
        pos_or_panic!(0.60),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
        pos_or_panic!(0.50),
        pos_or_panic!(0.60),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
        pos_or_panic!(0.50),
        pos_or_panic!(0.60),
    )
    .unwrap_or_else(|e| panic!("the short butterfly fixture builds: {e}"))
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
    )
    .unwrap_or_else(|e| panic!("the iron condor fixture builds: {e}"))
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
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
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
    )
    .unwrap_or_else(|e| panic!("the short strangle fixture builds: {e}"))
}

fn poor_mans_covered_call(quantity: Positive) -> PoorMansCoveredCall {
    PoorMansCoveredCall::new(
        "SP500".to_string(),
        spot(),
        pos_or_panic!(5600.0),
        pos_or_panic!(5950.0),
        ExpirationDate::Days(pos_or_panic!(120.0)),
        expiration(),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(300.0),
        pos_or_panic!(30.0),
        pos_or_panic!(0.10),
        pos_or_panic!(0.20),
        pos_or_panic!(0.30),
        pos_or_panic!(0.40),
    )
    .unwrap_or_else(|e| panic!("the poor man's covered call fixture builds: {e}"))
}

/// The chain's option at `strike`.
fn at_strike(chain: &OptionChain, strike: Positive) -> &optionstratlib_market::chains::OptionData {
    chain
        .options
        .iter()
        .find(|option| option.strike_price == strike)
        .unwrap_or_else(|| panic!("the chain lists strike {strike}"))
}

#[test]
fn test_create_strategy_keeps_fees_two_leg_spreads() {
    check_create_strategy(bull_call_spread, two_legs, dec!(1.0));
    check_create_strategy(bear_call_spread, two_legs, dec!(1.0));
    check_create_strategy(bull_put_spread, two_legs, dec!(1.0));
    check_create_strategy(bear_put_spread, two_legs, dec!(1.0));
}

#[test]
fn test_create_strategy_keeps_fees_straddles_and_strangles() {
    check_create_strategy(long_straddle, one_leg_twice, dec!(1.0));
    check_create_strategy(short_straddle, one_leg_twice, dec!(1.0));
    check_create_strategy(long_strangle, two_legs, dec!(1.0));
    check_create_strategy(short_strangle, two_legs, dec!(1.0));
}

#[test]
fn test_create_strategy_keeps_fees_three_leg_strategies() {
    // Every leg holds the quantity: 0.3 + 0.7 + 1.1 per unit.
    check_create_strategy(bull_call_ladder, three_legs, dec!(2.1));
    // The body holds twice the quantity and takes the first fee pair:
    // 2 × 0.3 + 0.7 + 1.1 per unit.
    check_create_strategy(long_butterfly_spread, three_legs, dec!(2.4));
    check_create_strategy(short_butterfly_spread, three_legs, dec!(2.4));
}

#[test]
fn test_create_strategy_keeps_fees_iron_strategies() {
    // One open and one close fee for each of the four legs: 4 × 0.3 per unit.
    check_create_strategy(iron_condor, four_legs, dec!(1.2));
    check_create_strategy(iron_butterfly, three_legs_body_twice, dec!(1.2));
}

#[test]
fn test_create_strategy_keeps_fees_poor_mans_covered_call() {
    // No combination filter: the legs are picked by strike.
    let chain = chain();
    let legs = StrategyLegs::TwoLegs {
        first: at_strike(&chain, pos_or_panic!(5600.0)),
        second: at_strike(&chain, pos_or_panic!(5950.0)),
    };
    for quantity in QUANTITIES {
        let strategy = poor_mans_covered_call(pos(quantity));
        assert_rebuilt_fees(&strategy, &chain, &legs, dec!(1.0) * quantity, quantity);
    }
}

#[test]
fn test_iron_condor_search_keeps_fees_at_quantity_two() {
    check_optimiser(iron_condor, four_legs);
}

#[test]
fn test_iron_butterfly_search_keeps_fees_at_quantity_two() {
    check_optimiser(iron_butterfly, three_legs_body_twice);
}

#[test]
fn test_two_leg_spread_searches_keep_fees_at_quantity_two() {
    check_optimiser(bull_call_spread, two_legs);
    check_optimiser(bear_call_spread, two_legs);
    check_optimiser(bull_put_spread, two_legs);
    check_optimiser(bear_put_spread, two_legs);
}

#[test]
fn test_straddle_and_strangle_searches_keep_fees_at_quantity_two() {
    check_optimiser(long_straddle, one_leg_twice);
    check_optimiser(short_straddle, one_leg_twice);
    check_optimiser(long_strangle, two_legs);
}

#[test]
fn test_three_leg_searches_keep_fees_at_quantity_two() {
    check_optimiser(bull_call_ladder, three_legs);
    check_optimiser(long_butterfly_spread, three_legs);
    check_optimiser(short_butterfly_spread, three_legs);
}

#[test]
fn test_short_strangle_search_keeps_fees_at_quantity_two() {
    // The short strangle takes the chain's expiration before its search, so
    // the path-free search starts from the strategy with that expiration.
    let chain = chain();
    let mut input = short_strangle(Positive::TWO);
    let input_fees = input
        .get_fees()
        .unwrap_or_else(|e| panic!("input fees: {e}"));
    input
        .find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio)
        .unwrap_or_else(|e| panic!("the search finds a candidate: {e}"));
    assert_eq!(
        input.get_fees().unwrap_or_else(|e| panic!("fees: {e}")),
        input_fees
    );
}

#[test]
fn test_poor_mans_covered_call_search_keeps_fees_at_quantity_two() {
    let chain = chain();
    let mut input = poor_mans_covered_call(Positive::TWO);
    let input_fees = input
        .get_fees()
        .unwrap_or_else(|e| panic!("input fees: {e}"));
    input
        .find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio)
        .unwrap_or_else(|e| panic!("the search finds a candidate: {e}"));
    assert_eq!(
        input.get_fees().unwrap_or_else(|e| panic!("fees: {e}")),
        input_fees
    );
}
