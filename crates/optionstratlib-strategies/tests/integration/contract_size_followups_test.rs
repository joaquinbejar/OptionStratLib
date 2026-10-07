//! Contract size follow-ups (#760).
//!
//! - A strategy set to contract-sized legs keeps that size when it is
//!   rebuilt: through `Optimizable::create_strategy`, through the
//!   optimisation loop that calls it, and through
//!   `StrategyConstructor::get_strategy`.
//! - The covered strategies re-express their option legs in market
//!   contracts with the same payoff, premium and fees.
//! - `ProtectivePut` reports the zeros of its expiry P&L for a put that
//!   covers fewer or more units than the shares it protects.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::{ExpirationDate, types::OptionStyle};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::StrategyLegs;
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_market::chains::utils::FindOptimalSide;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::strategies::StrategyConstructor;
use optionstratlib_strategies::strategies::base::{BreakEvenable, Optimizable, Positionable};
use optionstratlib_strategies::strategies::{
    BasicAble, BullCallSpread, Collar, CoveredCall, IronCondor, LongButterflySpread, LongStraddle,
    ProtectivePut, ShortStrangle, Strategies,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const CONTRACT: Positive = Positive::HUNDRED;

fn chain() -> OptionChain {
    OptionChain::load_from_json(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    ))
    .unwrap_or_else(|e| panic!("chain fixture: {e}"))
}

fn strikes<S: Positionable>(strategy: &S) -> Vec<Positive> {
    let mut strikes: Vec<Positive> = strategy
        .get_positions()
        .unwrap_or_else(|e| panic!("{e}"))
        .iter()
        .map(|p| p.option.strike_price)
        .collect();
    strikes.sort();
    strikes
}

/// Every option leg carries `size`, and the strategy reports it.
fn assert_sized<S: Positionable + BasicAble>(strategy: &S, size: Positive) {
    assert_eq!(strategy.get_contract_size().ok(), Some(size));
    for leg in strategy.get_positions().unwrap_or_else(|e| panic!("{e}")) {
        assert_eq!(leg.option.contract_size, size);
    }
}

fn bull_call_spread() -> BullCallSpread {
    BullCallSpread::new(
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
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn long_straddle() -> LongStraddle {
    LongStraddle::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5780.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(40.0),
        pos_or_panic!(38.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn short_strangle() -> ShortStrangle {
    ShortStrangle::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5850.0),
        pos_or_panic!(5710.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(10.0),
        pos_or_panic!(12.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn long_butterfly() -> LongButterflySpread {
    LongButterflySpread::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5710.0),
        pos_or_panic!(5780.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(113.3),
        pos_or_panic!(64.20),
        pos_or_panic!(31.65),
        pos_or_panic!(0.07),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn iron_condor() -> IronCondor {
    IronCondor::new(
        "SP500".to_string(),
        pos_or_panic!(5781.88),
        pos_or_panic!(5850.0),
        pos_or_panic!(5710.0),
        pos_or_panic!(5900.0),
        pos_or_panic!(5650.0),
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(10.0),
        pos_or_panic!(12.0),
        pos_or_panic!(3.0),
        pos_or_panic!(4.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

/// Sets `CONTRACT` on a strategy and checks the setter's own contract: the
/// quantities stay in contracts.
fn sized<S: Positionable + BasicAble + Clone>(strategy: &S) -> S {
    let mut sized = strategy.clone();
    sized
        .set_contract_size(CONTRACT)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_sized(&sized, CONTRACT);
    let before = strategy.get_positions().unwrap_or_else(|e| panic!("{e}"));
    let after = sized.get_positions().unwrap_or_else(|e| panic!("{e}"));
    for (a, b) in before.iter().zip(after.iter()) {
        assert_eq!(a.option.quantity, b.option.quantity);
        assert_eq!(a.open_fee, b.open_fee);
        assert_eq!(a.close_fee, b.close_fee);
    }
    sized
}

/// Optimising a contract-sized strategy rebuilds it through
/// `create_strategy` and keeps the contract size; the one-unit run, from the
/// same start, moves the legs the same way.
fn assert_optimisation_keeps_size<S>(strategy: S)
where
    S: Optimizable<Strategy = S> + Positionable + BasicAble + Clone,
{
    let chain = chain();
    let start = strikes(&strategy);
    let mut unit = strategy.clone();
    let mut contracts = sized(&strategy);

    unit.get_best_area(&chain, FindOptimalSide::All);
    contracts.get_best_area(&chain, FindOptimalSide::All);

    assert_sized(&unit, Positive::ONE);
    assert_sized(&contracts, CONTRACT);
    assert_ne!(
        strikes(&contracts),
        start,
        "the optimisation rebuilt the legs"
    );
}

#[test]
fn test_bull_call_spread_contract_size_optimisation_keeps_contract_size() {
    assert_optimisation_keeps_size(bull_call_spread());
}

#[test]
fn test_long_straddle_contract_size_optimisation_keeps_contract_size() {
    assert_optimisation_keeps_size(long_straddle());
}

#[test]
fn test_short_strangle_contract_size_optimisation_keeps_contract_size() {
    assert_optimisation_keeps_size(short_strangle());
}

#[test]
fn test_long_butterfly_contract_size_optimisation_keeps_contract_size() {
    assert_optimisation_keeps_size(long_butterfly());
}

#[test]
fn test_iron_condor_contract_size_optimisation_keeps_contract_size() {
    assert_optimisation_keeps_size(iron_condor());
}

#[test]
fn test_bull_call_spread_contract_size_create_strategy_keeps_contract_size() {
    let chain = chain();
    let strike = |k: f64| {
        chain
            .get_single_iter()
            .find(|o| o.strike_price == pos_or_panic!(k))
            .unwrap_or_else(|| panic!("strike {k} in the fixture"))
    };
    let legs = StrategyLegs::TwoLegs {
        first: strike(5750.0),
        second: strike(5800.0),
    };

    let unit = bull_call_spread()
        .create_strategy(&chain, &legs)
        .unwrap_or_else(|e| panic!("{e}"));
    let contracts = sized(&bull_call_spread())
        .create_strategy(&chain, &legs)
        .unwrap_or_else(|e| panic!("{e}"));

    assert_sized(&unit, Positive::ONE);
    assert_sized(&contracts, CONTRACT);
    assert_eq!(strikes(&contracts), strikes(&unit));
    // The fees stay per contract, so the max profit of the contract-sized
    // spread is 100 times the premium part plus the same fees.
    let fees = unit.get_fees().unwrap_or_else(|e| panic!("{e}")).to_dec();
    let unit_profit = unit.get_max_profit().unwrap_or_else(|e| panic!("{e}"));
    let contract_profit = contracts.get_max_profit().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        contract_profit.to_dec(),
        (unit_profit.to_dec() + fees) * CONTRACT.to_dec() - fees
    );
}

#[test]
fn test_bull_call_spread_contract_size_get_strategy_keeps_contract_size() {
    let spread = sized(&bull_call_spread());
    let positions: Vec<Position> = spread
        .get_positions()
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .cloned()
        .collect();
    let rebuilt = BullCallSpread::get_strategy(&positions).unwrap_or_else(|e| panic!("{e}"));
    assert_sized(&rebuilt, CONTRACT);
}

#[test]
fn test_bull_call_spread_contract_size_scales_profit_and_break_even() {
    let unit = bull_call_spread();
    let contracts = sized(&unit);
    let fees = unit.get_fees().unwrap_or_else(|e| panic!("{e}")).to_dec();
    for price in [5700.0, 5750.0, 5790.0, 5820.0, 5900.0] {
        let p = pos_or_panic!(price);
        let a = unit
            .calculate_profit_at(&p)
            .unwrap_or_else(|e| panic!("{e}"));
        let b = contracts
            .calculate_profit_at(&p)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(b, (a + fees) * CONTRACT.to_dec() - fees, "S = {price}");
    }
    // The break-even is the zero of the contract-sized expiry P&L: the
    // per-contract fees spread over 100 times the units.
    let be = contracts
        .get_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(be.len(), 1);
    let at_be = contracts
        .calculate_profit_at(&be[0])
        .unwrap_or_else(|e| panic!("{e}"));
    // Two contracts of 100 units: a cent of rounding moves the P&L by 2.
    assert!(at_be.abs() <= dec!(2), "P&L at break-even {at_be}");
}

#[test]
fn test_strategy_contract_size_zero_is_rejected_and_leaves_legs() {
    let mut spread = bull_call_spread();
    assert!(spread.set_contract_size(Positive::ZERO).is_err());
    assert_sized(&spread, Positive::ONE);

    let mut covered = protective_put();
    assert!(covered.set_contract_size(Positive::ZERO).is_err());
    assert_sized(&covered, Positive::ONE);
}

#[test]
fn test_strategy_contract_size_mixed_legs_have_no_single_size() {
    let mut spread = bull_call_spread();
    spread.short_call.option.contract_size = CONTRACT;
    assert!(spread.get_contract_size().is_err());
    // A rebuild has no single size to carry, so it is refused rather than
    // silently resized.
    let chain = chain();
    let strike = |k: f64| {
        chain
            .get_single_iter()
            .find(|o| o.strike_price == pos_or_panic!(k))
            .unwrap_or_else(|| panic!("strike {k} in the fixture"))
    };
    let legs = StrategyLegs::TwoLegs {
        first: strike(5750.0),
        second: strike(5800.0),
    };
    assert!(spread.create_strategy(&chain, &legs).is_err());
}

// --- Covered strategies -----------------------------------------------------

fn expiry() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

const PRICES: [f64; 9] = [0.0, 50.0, 90.0, 95.0, 100.0, 103.0, 105.0, 110.0, 150.0];

fn profit<S: Profit>(strategy: &S, price: f64) -> Decimal {
    strategy
        .calculate_profit_at(&pos_or_panic!(price))
        .unwrap_or_else(|e| panic!("profit at {price}: {e}"))
}

fn covered_call() -> CoveredCall {
    CoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(2.4),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn protective_put() -> ProtectivePut {
    ProtectivePut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(1.5),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn collar() -> Collar {
    Collar::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(1.5),
        pos_or_panic!(2.4),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

/// A covered strategy re-expressed in 100-unit contracts covers the same
/// shares with one contract per 100 shares, and reports the same figures.
fn assert_covered_re_expression<S>(shares: &S)
where
    S: Strategies + Profit + Clone,
{
    let mut contracts = shares.clone();
    contracts
        .set_contract_size(CONTRACT)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_sized(&contracts, CONTRACT);
    for leg in contracts.get_positions().unwrap_or_else(|e| panic!("{e}")) {
        assert_eq!(leg.option.quantity, Positive::ONE);
        assert_eq!(leg.open_fee, Positive::ONE);
        assert_eq!(leg.close_fee, Positive::ONE);
    }
    for price in PRICES {
        assert_eq!(
            profit(&contracts, price),
            profit(shares, price),
            "S = {price}"
        );
    }
    assert_eq!(
        contracts.get_break_even_points().ok(),
        shares.get_break_even_points().ok()
    );
    assert_eq!(contracts.get_fees().ok(), shares.get_fees().ok());
    assert_eq!(contracts.get_max_loss().ok(), shares.get_max_loss().ok());
    assert_eq!(
        contracts.get_max_profit().ok(),
        shares.get_max_profit().ok()
    );
}

#[test]
fn test_covered_call_contract_size_re_expression_matches_shares() {
    assert_covered_re_expression(&covered_call());
}

#[test]
fn test_protective_put_contract_size_re_expression_matches_shares() {
    assert_covered_re_expression(&protective_put());
}

#[test]
fn test_collar_contract_size_re_expression_matches_shares() {
    assert_covered_re_expression(&collar());
}

// --- ProtectivePut break-even for a mismatched hedge ------------------------

/// A protective put on 100 shares bought at 100, whose put is resized to
/// `contracts` contracts of 100 units, with the put fees per contract.
fn hedged(strike: f64, premium: f64, contracts: Positive) -> ProtectivePut {
    let mut strategy = ProtectivePut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(strike),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(premium),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let mut put = strategy.long_put.clone();
    put.option.quantity = contracts;
    put.option.contract_size = CONTRACT;
    strategy
        .modify_position(&put)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(put.option.option_style, OptionStyle::Put);
    strategy
}

/// Each reported break-even is a zero of the expiry P&L, to the cent it is
/// rounded to.
fn assert_zeros(strategy: &ProtectivePut, slope_bound: Decimal) {
    for be in strategy
        .get_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"))
    {
        let pnl = strategy
            .calculate_profit_at(be)
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(
            pnl.abs() <= slope_bound * dec!(0.005),
            "P&L at {be} is {pnl}"
        );
    }
}

#[test]
fn test_protective_put_under_hedged_break_even_above_strike() {
    // 50 put units against 100 shares, an OTM put struck at 95.
    // F = 1 + 1 + (0.01 + 0.01) × 0.5 = 2.01; above the strike the P&L is
    // 100 (S - 100) - 50 × 1.5 - 2.01, zero at 100.7701. Below the strike
    // it rises with slope 50 and never reaches zero there.
    let strategy = hedged(95.0, 1.5, pos_or_panic!(0.5));
    assert_eq!(
        strategy.get_break_even_points().ok(),
        Some(&vec![pos_or_panic!(100.77)])
    );
    assert_zeros(&strategy, dec!(100));
}

#[test]
fn test_protective_put_under_hedged_break_even_below_strike() {
    // 50 put units against 100 shares, an ITM put struck at 110 bought at
    // 10.5. F = 2.01. At the strike the P&L is 1000 - 525 - 2.01 > 0, so
    // the zero sits below it, on the line of slope 100 - 50:
    // (100 × 100 - 50 × 110 + 50 × 10.5 + 2.01) / 50 = 100.5402.
    let strategy = hedged(110.0, 10.5, pos_or_panic!(0.5));
    assert_eq!(
        strategy.get_break_even_points().ok(),
        Some(&vec![pos_or_panic!(100.54)])
    );
    assert_zeros(&strategy, dec!(100));
}

#[test]
fn test_protective_put_over_hedged_has_two_break_evens() {
    // 200 put units against 100 shares, struck at 95. F = 2 + 0.02 × 2 =
    // 2.04. Above: 100 + (300 + 2.04) / 100 = 103.0204. Below, the P&L
    // falls with slope -100 towards the strike:
    // (10000 - 19000 + 300 + 2.04) / (100 - 200) = 86.9796.
    let strategy = hedged(95.0, 1.5, Positive::TWO);
    assert_eq!(
        strategy.get_break_even_points().ok(),
        Some(&vec![pos_or_panic!(86.98), pos_or_panic!(103.02)])
    );
    assert_zeros(&strategy, dec!(100));
}

#[test]
fn test_protective_put_exact_hedge_in_contracts_keeps_break_even() {
    // One contract of 100 units covers the 100 shares exactly, so the
    // single break-even C + p + F / N holds, with the put fees charged per
    // contract: F = 2 + 0.02 × 1, against 2 + 0.02 × 100 per share.
    let shares = protective_put();
    let contracts = hedged(95.0, 1.5, Positive::ONE);
    assert_eq!(
        contracts.get_break_even_points().ok(),
        Some(&vec![pos_or_panic!(101.52)])
    );
    assert_eq!(
        shares.get_break_even_points().ok(),
        Some(&vec![pos_or_panic!(101.54)])
    );
    assert_zeros(&contracts, dec!(100));
}
