//! `Optimizable::find_optimal` on a chain that has candidates (#814).
//!
//! `find_optimal_result_test` covers the searches that find nothing. Here
//! every strategy family with a search runs on a small, deterministic
//! Black-Scholes chain (spot 100, strikes 75 to 125 every 5, 30 days),
//! returns `Ok`, moves its legs off the seed onto strikes of the chain,
//! keeps the style and side of every leg, and satisfies the strike order of
//! its family. `CustomStrategy` has its own success case in `custom.rs`.

use optionstratlib_core::model::{ExpirationDate, OptionStyle, Positive, Side};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::{
    FindOptimalSide, OptionChainBuildParams, OptionDataPriceParams,
};
use optionstratlib_strategies::strategies::base::{Optimizable, Positionable, Validable};
use optionstratlib_strategies::strategies::utils::OptimizationCriteria;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, IronButterfly,
    IronCondor, LongButterflySpread, LongStraddle, LongStrangle, PoorMansCoveredCall,
    ShortButterflySpread, ShortStraddle, ShortStrangle,
};
use rust_decimal_macros::dec;

/// One leg as the search sees it: style, side and strike.
type Leg = (OptionStyle, Side, Positive);

const SYMBOL: &str = "TEST";

fn p(value: f64) -> Positive {
    pos_or_panic!(value)
}

fn expiration() -> ExpirationDate {
    ExpirationDate::Days(p(30.0))
}

/// Eleven strikes, 75 to 125 every 5, priced by Black-Scholes at a 20%
/// volatility with a mild skew. Call quotes fall and put quotes rise with
/// the strike, so every family has valid candidates.
fn priced_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        SYMBOL.to_string(),
        Some(Positive::HUNDRED),
        5,
        Some(p(5.0)),
        dec!(-0.2),
        dec!(0.1),
        p(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(expiration()),
            Some(dec!(0.05)),
            Some(Positive::ZERO),
            Some(SYMBOL.to_string()),
        ),
        p(0.2),
    );
    OptionChain::build_chain(&params).unwrap_or_else(|e| panic!("the fixture chain builds: {e}"))
}

fn legs<S: Positionable>(strategy: &S) -> Vec<Leg> {
    strategy
        .get_positions()
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|p| (p.option.option_style, p.option.side, p.option.strike_price))
        .collect()
}

/// The strikes of the legs with `style` and `side`, in leg order.
fn strikes(legs: &[Leg], style: OptionStyle, side: Side) -> Vec<Positive> {
    legs.iter()
        .filter(|(s, d, _)| *s == style && *d == side)
        .map(|(_, _, k)| *k)
        .collect()
}

/// Runs the search on the priced chain and checks what every family must
/// satisfy; returns the legs of the chosen strategy.
fn assert_search_moves_legs<S>(
    seed: &S,
    side: FindOptimalSide,
    criteria: OptimizationCriteria,
) -> Vec<Leg>
where
    S: Optimizable + Positionable + Validable + Clone,
{
    let chain = priced_chain();
    let mut strategy = seed.clone();
    if let Err(e) = strategy.find_optimal(&chain, side, criteria) {
        panic!("the search on the priced chain finds a candidate: {e:?}");
    }
    assert!(strategy.validate(), "the chosen strategy is valid");

    let before = legs(seed);
    let after = legs(&strategy);
    assert_ne!(after, before, "the search moves the legs off the seed");
    let shape = |legs: &[Leg]| legs.iter().map(|(s, d, _)| (*s, *d)).collect::<Vec<_>>();
    assert_eq!(
        shape(&after),
        shape(&before),
        "every leg keeps its style and side"
    );
    for (_, _, strike) in &after {
        assert!(
            chain.options.iter().any(|o| o.strike_price == *strike),
            "strike {strike} is on the chain"
        );
    }
    after
}

/// Both criteria over the whole chain.
fn assert_both_criteria_move_legs<S>(seed: &S, check: impl Fn(&[Leg]))
where
    S: Optimizable + Positionable + Validable + Clone,
{
    for criteria in [OptimizationCriteria::Ratio, OptimizationCriteria::Area] {
        check(&assert_search_moves_legs(
            seed,
            FindOptimalSide::All,
            criteria,
        ));
    }
}

#[test]
fn test_find_optimal_priced_chain_bull_call_spread_moves_legs() {
    let z = Positive::ZERO;
    let seed = BullCallSpread::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(95.0),
        p(105.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(5.91),
        p(0.70),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let long = strikes(legs, OptionStyle::Call, Side::Long);
        let short = strikes(legs, OptionStyle::Call, Side::Short);
        assert!(long[0] < short[0], "long call below short call: {legs:?}");
    });
}

#[test]
fn test_find_optimal_priced_chain_bear_call_spread_moves_legs() {
    let z = Positive::ZERO;
    let seed = BearCallSpread::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(95.0),
        p(105.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(5.89),
        p(0.72),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let long = strikes(legs, OptionStyle::Call, Side::Long);
        let short = strikes(legs, OptionStyle::Call, Side::Short);
        assert!(short[0] < long[0], "short call below long call: {legs:?}");
    });
}

#[test]
fn test_find_optimal_priced_chain_bull_put_spread_moves_legs() {
    let z = Positive::ZERO;
    let seed = BullPutSpread::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(90.0),
        p(100.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(0.08),
        p(2.07),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let long = strikes(legs, OptionStyle::Put, Side::Long);
        let short = strikes(legs, OptionStyle::Put, Side::Short);
        assert!(long[0] < short[0], "long put below short put: {legs:?}");
    });
}

#[test]
fn test_find_optimal_priced_chain_bear_put_spread_moves_legs() {
    let z = Positive::ZERO;
    let seed = BearPutSpread::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(105.0),
        p(95.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(5.29),
        p(0.50),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let check = |legs: &[Leg]| {
        let long = strikes(legs, OptionStyle::Put, Side::Long);
        let short = strikes(legs, OptionStyle::Put, Side::Short);
        assert!(short[0] < long[0], "short put below long put: {legs:?}");
    };
    assert_both_criteria_move_legs(&seed, check);
    // The upper search keeps both strikes at or above the spot.
    let upper =
        assert_search_moves_legs(&seed, FindOptimalSide::Upper, OptimizationCriteria::Ratio);
    check(&upper);
    assert!(
        upper.iter().all(|(_, _, k)| *k >= Positive::HUNDRED),
        "{upper:?}"
    );
}

#[test]
fn test_find_optimal_priced_chain_bull_call_ladder_moves_legs() {
    let z = Positive::ZERO;
    let seed = BullCallLadder::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(95.0),
        p(105.0),
        p(110.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(5.91),
        p(0.70),
        p(0.12),
        z,
        z,
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let long = strikes(legs, OptionStyle::Call, Side::Long);
        let short = strikes(legs, OptionStyle::Call, Side::Short);
        assert!(
            long[0] < short[0] && short[0] < short[1],
            "long call < short low < short high: {legs:?}"
        );
    });
}

#[test]
fn test_find_optimal_priced_chain_poor_mans_covered_call_moves_legs() {
    let z = Positive::ZERO;
    let seed = PoorMansCoveredCall::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(90.0),
        p(110.0),
        ExpirationDate::Days(p(120.0)),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(12.0),
        p(0.12),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let check = |legs: &[Leg]| {
        let long = strikes(legs, OptionStyle::Call, Side::Long);
        let short = strikes(legs, OptionStyle::Call, Side::Short);
        assert!(long[0] < short[0], "long call below short call: {legs:?}");
    };
    assert_both_criteria_move_legs(&seed, check);
    // The centred search puts the long call at or below the spot and the
    // short call above it.
    let center =
        assert_search_moves_legs(&seed, FindOptimalSide::Center, OptimizationCriteria::Ratio);
    check(&center);
    assert!(strikes(&center, OptionStyle::Call, Side::Long)[0] <= Positive::HUNDRED);
    assert!(strikes(&center, OptionStyle::Call, Side::Short)[0] > Positive::HUNDRED);
}

#[test]
fn test_find_optimal_priced_chain_long_butterfly_spread_moves_legs() {
    let z = Positive::ZERO;
    let seed = LongButterflySpread::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(90.0),
        p(100.0),
        p(110.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(10.45),
        p(2.48),
        p(0.14),
        z,
        z,
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let long = strikes(legs, OptionStyle::Call, Side::Long);
        let short = strikes(legs, OptionStyle::Call, Side::Short);
        assert!(
            long[0] < short[0] && short[0] < long[1],
            "low < middle < high: {legs:?}"
        );
    });
}

#[test]
fn test_find_optimal_priced_chain_short_butterfly_spread_moves_legs() {
    let z = Positive::ZERO;
    let seed = ShortButterflySpread::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(90.0),
        p(100.0),
        p(110.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(10.43),
        p(2.50),
        p(0.12),
        z,
        z,
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let long = strikes(legs, OptionStyle::Call, Side::Long);
        let short = strikes(legs, OptionStyle::Call, Side::Short);
        assert!(
            short[0] < long[0] && long[0] < short[1],
            "low < middle < high: {legs:?}"
        );
    });
}

fn assert_iron_condor_order(legs: &[Leg]) {
    let long_put = strikes(legs, OptionStyle::Put, Side::Long)[0];
    let short_put = strikes(legs, OptionStyle::Put, Side::Short)[0];
    let short_call = strikes(legs, OptionStyle::Call, Side::Short)[0];
    let long_call = strikes(legs, OptionStyle::Call, Side::Long)[0];
    assert!(
        long_put < short_put && short_put < short_call && short_call < long_call,
        "long put < short put < short call < long call: {legs:?}"
    );
}

#[test]
fn test_find_optimal_priced_chain_iron_condor_moves_legs() {
    let z = Positive::ZERO;
    let seed = IronCondor::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(105.0),
        p(95.0),
        p(110.0),
        p(90.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(0.70),
        p(0.50),
        p(0.14),
        p(0.08),
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, assert_iron_condor_order);
    // The centred search keeps the short put at or below the spot and the
    // short call at or above it; the upper search keeps every leg at or
    // above it.
    let center =
        assert_search_moves_legs(&seed, FindOptimalSide::Center, OptimizationCriteria::Ratio);
    assert_iron_condor_order(&center);
    assert!(strikes(&center, OptionStyle::Put, Side::Short)[0] <= Positive::HUNDRED);
    assert!(strikes(&center, OptionStyle::Call, Side::Short)[0] >= Positive::HUNDRED);
    let upper =
        assert_search_moves_legs(&seed, FindOptimalSide::Upper, OptimizationCriteria::Ratio);
    assert_iron_condor_order(&upper);
    assert!(
        upper.iter().all(|(_, _, k)| *k >= Positive::HUNDRED),
        "{upper:?}"
    );
}

#[test]
fn test_find_optimal_priced_chain_iron_butterfly_moves_legs() {
    let z = Positive::ZERO;
    let seed = IronButterfly::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(100.0),
        p(110.0),
        p(90.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(2.48),
        p(2.07),
        p(0.14),
        p(0.08),
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| {
        let short_call = strikes(legs, OptionStyle::Call, Side::Short)[0];
        let short_put = strikes(legs, OptionStyle::Put, Side::Short)[0];
        let long_call = strikes(legs, OptionStyle::Call, Side::Long)[0];
        let long_put = strikes(legs, OptionStyle::Put, Side::Long)[0];
        assert_eq!(
            short_call, short_put,
            "the short legs share a strike: {legs:?}"
        );
        assert!(
            long_put < short_put && short_call < long_call,
            "the wings sit outside the body: {legs:?}"
        );
    });
}

fn assert_straddle_shape(legs: &[Leg], side: Side) {
    let call = strikes(legs, OptionStyle::Call, side);
    let put = strikes(legs, OptionStyle::Put, side);
    assert_eq!(call, put, "both legs share a strike: {legs:?}");
}

#[test]
fn test_find_optimal_priced_chain_long_straddle_moves_legs() {
    let z = Positive::ZERO;
    let seed = LongStraddle::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(100.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(2.50),
        p(2.09),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| assert_straddle_shape(legs, Side::Long));
}

#[test]
fn test_find_optimal_priced_chain_short_straddle_moves_legs() {
    let z = Positive::ZERO;
    let seed = ShortStraddle::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(100.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(2.48),
        p(2.07),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| assert_straddle_shape(legs, Side::Short));
}

fn assert_strangle_shape(legs: &[Leg], side: Side) {
    let call = strikes(legs, OptionStyle::Call, side)[0];
    let put = strikes(legs, OptionStyle::Put, side)[0];
    assert!(put < call, "put strike below call strike: {legs:?}");
}

#[test]
fn test_find_optimal_priced_chain_long_strangle_moves_legs() {
    let z = Positive::ZERO;
    let seed = LongStrangle::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(110.0),
        p(90.0),
        expiration(),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(0.14),
        p(0.08),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| assert_strangle_shape(legs, Side::Long));
    // The centred search puts the put at or below the spot and the call at
    // or above it.
    let center =
        assert_search_moves_legs(&seed, FindOptimalSide::Center, OptimizationCriteria::Ratio);
    assert_strangle_shape(&center, Side::Long);
    assert!(strikes(&center, OptionStyle::Put, Side::Long)[0] <= Positive::HUNDRED);
    assert!(strikes(&center, OptionStyle::Call, Side::Long)[0] >= Positive::HUNDRED);
}

#[test]
fn test_find_optimal_priced_chain_short_strangle_moves_legs() {
    let z = Positive::ZERO;
    let seed = ShortStrangle::new(
        SYMBOL.to_string(),
        Positive::HUNDRED,
        p(110.0),
        p(90.0),
        expiration(),
        p(0.2),
        p(0.2),
        dec!(0.05),
        z,
        Positive::ONE,
        p(0.12),
        p(0.06),
        z,
        z,
        z,
        z,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_both_criteria_move_legs(&seed, |legs| assert_strangle_shape(legs, Side::Short));
}
