//! Strategies over option legs that carry a contract multiplier (#733).
//!
//! `Options::contract_size` scales payoff, premium, P&L and Greeks by
//! `quantity × contract_size`, while `Position` fees stay per contract. A
//! leg of `n` contracts of 100 units, with per-contract fees 100 times the
//! per-unit ones, is therefore the same position as a leg of `100 × n`
//! one-unit contracts: every figure a strategy reports must agree.
//!
//! The stock-plus-option strategies are built in shares (#731) and then
//! re-expressed in contracts by swapping their option leg, since their
//! constructors still size the option leg one unit per share.

use chrono::Utc;
use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::{
    ExpirationDate, Options,
    position::Position,
    types::{OptionStyle, OptionType, Side},
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::strategies::base::{BreakEvenable, Positionable};
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::{Collar, CoveredCall, ProtectivePut, Strategies};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const CONTRACT: Positive = Positive::HUNDRED;

fn expiry() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

fn profit<S: Profit>(strategy: &S, price: f64) -> Decimal {
    strategy
        .calculate_profit_at(&pos_or_panic!(price))
        .unwrap_or_else(|e| panic!("profit at {price}: {e}"))
}

const PRICES: [f64; 9] = [0.0, 50.0, 90.0, 95.0, 100.0, 103.0, 105.0, 110.0, 150.0];

/// Re-expresses a one-unit-per-share option leg as one contract of
/// `CONTRACT` units, with its per-unit fees carried to per-contract fees.
fn in_contracts(leg: &Position) -> Position {
    let mut sized = leg.clone();
    let contracts = leg
        .option
        .quantity
        .checked_div(&CONTRACT)
        .unwrap_or_else(|e| panic!("contracts: {e}"));
    sized.option.quantity = contracts;
    sized.option.contract_size = CONTRACT;
    sized.open_fee = leg.open_fee * CONTRACT;
    sized.close_fee = leg.close_fee * CONTRACT;
    sized
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

#[test]
fn test_covered_call_in_contracts_matches_shares() {
    let shares = covered_call();
    let mut contracts = shares.clone();
    contracts.short_call = in_contracts(&shares.short_call);
    contracts
        .update_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"));

    // One 100-share contract covers the 100 shares.
    assert_eq!(contracts.short_call.option.quantity, Positive::ONE);
    assert_eq!(
        contracts.short_call.option.position_size().ok(),
        Some(Positive::HUNDRED)
    );
    for price in PRICES {
        assert_eq!(
            profit(&contracts, price),
            profit(&shares, price),
            "S = {price}"
        );
    }
    // Capped at (105 - 100) × 100 + 2.40 × 100 - 2 - 2 = 736.
    assert_eq!(profit(&contracts, 150.0), dec!(736));
    assert_eq!(
        contracts.max_profit_potential().ok(),
        Some(pos_or_panic!(736.0))
    );
    assert_eq!(
        contracts.max_loss_potential().ok(),
        shares.max_loss_potential().ok()
    );
    assert_eq!(
        contracts.effective_cost_basis().ok(),
        shares.effective_cost_basis().ok()
    );
    assert_eq!(contracts.total_fees().ok(), shares.total_fees().ok());
    assert_eq!(contracts.break_even_points, shares.break_even_points);
    assert_eq!(contracts.net_delta().ok(), shares.net_delta().ok());
}

#[test]
fn test_protective_put_in_contracts_matches_shares() {
    let shares = protective_put();
    let mut contracts = shares.clone();
    contracts.long_put = in_contracts(&shares.long_put);
    contracts
        .update_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"));

    for price in PRICES {
        assert_eq!(
            profit(&contracts, price),
            profit(&shares, price),
            "S = {price}"
        );
    }
    assert_eq!(
        contracts.max_loss_potential().ok(),
        Some(pos_or_panic!(654.0))
    );
    assert_eq!(
        contracts.effective_cost_basis().ok(),
        shares.effective_cost_basis().ok()
    );
    assert_eq!(contracts.break_even_points, shares.break_even_points);
    assert_eq!(contracts.net_delta().ok(), shares.net_delta().ok());
}

#[test]
fn test_collar_in_contracts_matches_shares() {
    let shares = collar();
    let mut contracts = shares.clone();
    contracts.long_put = in_contracts(&shares.long_put);
    contracts.short_call = in_contracts(&shares.short_call);
    contracts
        .update_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"));

    for price in PRICES {
        assert_eq!(
            profit(&contracts, price),
            profit(&shares, price),
            "S = {price}"
        );
    }
    assert_eq!(contracts.net_premium().ok(), shares.net_premium().ok());
    assert_eq!(contracts.net_premium().ok(), Some(dec!(90)));
    assert_eq!(
        contracts.max_profit_potential().ok(),
        shares.max_profit_potential().ok()
    );
    assert_eq!(
        contracts.max_loss_potential().ok(),
        shares.max_loss_potential().ok()
    );
    assert_eq!(contracts.break_even_points, shares.break_even_points);
}

fn leg(
    side: Side,
    strike: f64,
    contracts: Positive,
    contract_size: Positive,
    premium: f64,
) -> Position {
    // Fees are per contract: 0.10 per unit of the underlying.
    let fee = pos_or_panic!(0.1) * contract_size;
    Position::new(
        Options::new(
            OptionType::European,
            side,
            "SPX".to_string(),
            pos_or_panic!(strike),
            expiry(),
            pos_or_panic!(0.2),
            contracts,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        )
        .with_contract_size(contract_size),
        pos_or_panic!(premium),
        Utc::now(),
        fee,
        fee,
        None,
        None,
    )
}

/// Long 1x95, short 2x100, long 1x105 calls in `contract_size` units.
fn butterfly(lots: Positive, contract_size: Positive) -> CustomStrategy {
    CustomStrategy::new(
        "Butterfly".to_string(),
        "SPX".to_string(),
        "1/2/1 call butterfly".to_string(),
        Positive::HUNDRED,
        vec![
            leg(Side::Long, 95.0, lots, contract_size, 7.2),
            leg(Side::Short, 100.0, lots * Positive::TWO, contract_size, 3.9),
            leg(Side::Long, 105.0, lots, contract_size, 1.8),
        ],
        pos_or_panic!(0.01),
        100,
        pos_or_panic!(0.1),
    )
    .unwrap_or_else(|e| panic!("butterfly builds: {e}"))
}

#[test]
fn test_custom_butterfly_in_contracts_matches_unit_contracts() {
    let contracts = butterfly(Positive::ONE, CONTRACT);
    let units = butterfly(CONTRACT, Positive::ONE);

    for price in PRICES {
        assert_eq!(
            profit(&contracts, price),
            profit(&units, price),
            "S = {price}"
        );
    }
    assert_eq!(contracts.get_max_profit().ok(), units.get_max_profit().ok());
    assert_eq!(contracts.get_max_loss().ok(), units.get_max_loss().ok());
    assert_eq!(
        contracts.get_break_even_points().ok(),
        units.get_break_even_points().ok()
    );
    assert_eq!(contracts.get_fees().ok(), units.get_fees().ok());
    assert_eq!(contracts.delta().ok(), units.delta().ok());
    assert_eq!(contracts.gamma().ok(), units.gamma().ok());

    let mark = |strategy: &CustomStrategy| {
        strategy
            .calculate_pnl(
                &pos_or_panic!(103.0),
                ExpirationDate::Days(pos_or_panic!(15.0)),
                &pos_or_panic!(0.22),
            )
            .unwrap_or_else(|e| panic!("mark-to-market: {e}"))
            .unrealized
    };
    assert_eq!(mark(&contracts), mark(&units));
}

#[test]
fn test_custom_butterfly_contract_size_scales_profit() {
    let one_unit = butterfly(Positive::ONE, Positive::ONE);
    let one_contract = butterfly(Positive::ONE, CONTRACT);
    // At the body strike the 1/2/1 butterfly pays 5 per unit of the
    // underlying: 5 for one one-unit lot, 500 for one 100-unit contract lot.
    let intrinsic = |strategy: &CustomStrategy| -> Decimal {
        strategy
            .get_positions()
            .unwrap_or_else(|e| panic!("{e}"))
            .iter()
            .map(|p| {
                p.option
                    .payoff_at_price(&Positive::HUNDRED)
                    .unwrap_or_else(|e| panic!("{e}"))
            })
            .sum()
    };
    assert_eq!(intrinsic(&one_unit), dec!(5));
    assert_eq!(intrinsic(&one_contract), dec!(500));
}
