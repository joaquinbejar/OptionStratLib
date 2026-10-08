//! Covered strategies whose option legs do not cover the shares exactly
//! (#765).
//!
//! `ProtectivePut` and `Collar` on 100 shares bought at 100, with option
//! legs of 100-unit contracts covering half, all or twice the shares. Each
//! figure is checked against a hand-computed value and against the expiry
//! P&L itself: the break-evens are its zeros, the max profit and max loss
//! its extremes on a price grid, and the profit and loss zones the pieces
//! where it is positive and negative.

use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::types::{OptionStyle, Side};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::strategies::base::{BreakEvenable, Positionable};
use optionstratlib_strategies::strategies::probabilities::ProbabilityAnalysis;
use optionstratlib_strategies::strategies::{Collar, ProtectivePut, Strategies};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const CONTRACT: Positive = Positive::HUNDRED;
const SHARES: Positive = Positive::HUNDRED;

fn expiry() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

/// The expiry P&L on `[0, 300]` in steps of 0.5, which includes zero and
/// every strike used here.
fn grid<S: Profit>(strategy: &S) -> Vec<(Decimal, Decimal)> {
    (0..=600)
        .map(|i| {
            let price = Decimal::from(i) / dec!(2);
            let pnl = strategy
                .calculate_profit_at(&Positive::new_decimal(price).unwrap_or(Positive::ZERO))
                .unwrap_or_else(|e| panic!("profit at {price}: {e}"));
            (price, pnl)
        })
        .collect()
}

fn grid_max<S: Profit>(strategy: &S) -> Decimal {
    grid(strategy)
        .into_iter()
        .map(|(_, pnl)| pnl)
        .fold(Decimal::MIN, Decimal::max)
}

fn grid_min<S: Profit>(strategy: &S) -> Decimal {
    grid(strategy)
        .into_iter()
        .map(|(_, pnl)| pnl)
        .fold(Decimal::MAX, Decimal::min)
}

/// Every break-even is a zero of the expiry P&L to the cent it is rounded
/// to, and the P&L changes sign on the grid exactly once per break-even.
fn assert_break_evens_are_zeros<S: Profit + BreakEvenable>(strategy: &S) {
    let break_evens = strategy
        .get_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"));
    for be in break_evens {
        let pnl = strategy
            .calculate_profit_at(be)
            .unwrap_or_else(|e| panic!("{e}"));
        // The steepest slope here is 100 shares per unit of price.
        assert!(pnl.abs() <= dec!(0.5), "P&L at {be} is {pnl}");
    }
    let signs: Vec<bool> = grid(strategy)
        .into_iter()
        .filter(|(_, pnl)| !pnl.is_zero())
        .map(|(_, pnl)| pnl > Decimal::ZERO)
        .collect();
    let crossings = signs.windows(2).filter(|w| w[0] != w[1]).count();
    assert_eq!(crossings, break_evens.len(), "break-evens {break_evens:?}");
}

/// The zones are the pieces between the break-evens, signed like the P&L
/// inside them, and their probabilities add up to one.
fn assert_zones<S: Profit + ProbabilityAnalysis>(strategy: &S) {
    let profit = strategy
        .get_profit_ranges()
        .unwrap_or_else(|e| panic!("{e}"));
    let loss = strategy.get_loss_ranges().unwrap_or_else(|e| panic!("{e}"));
    let probe = |range: &ProfitLossRange| {
        let price = match (range.lower_bound, range.upper_bound) {
            (Some(lower), Some(upper)) => (lower.to_dec() + upper.to_dec()) / dec!(2),
            (Some(lower), None) => lower.to_dec() * dec!(2),
            (None, Some(upper)) => upper.to_dec() / dec!(2),
            (None, None) => dec!(100),
        };
        strategy
            .calculate_profit_at(&Positive::new_decimal(price).unwrap_or(Positive::ZERO))
            .unwrap_or_else(|e| panic!("{e}"))
    };
    for range in &profit {
        assert!(probe(range) > Decimal::ZERO, "profit zone {range:?}");
    }
    for range in &loss {
        assert!(probe(range) < Decimal::ZERO, "loss zone {range:?}");
    }
    let total: Decimal = profit
        .iter()
        .chain(loss.iter())
        .map(|range| range.probability.to_dec())
        .sum();
    assert!(
        (total - Decimal::ONE).abs() < dec!(0.001),
        "probabilities add up to {total}"
    );
}

// --- ProtectivePut ----------------------------------------------------------

/// A protective put on 100 shares bought at 100 whose put is `contracts`
/// contracts of 100 units, with the put fees of 0.01 + 0.01 per contract.
fn protective_put(strike: f64, premium: f64, contracts: Positive) -> ProtectivePut {
    let mut strategy = ProtectivePut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(strike),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        SHARES,
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
    strategy
}

fn max_loss<S: Strategies>(strategy: &S) -> Positive {
    strategy.get_max_loss().unwrap_or_else(|e| panic!("{e}"))
}

fn max_profit<S: Strategies>(strategy: &S) -> Positive {
    strategy.get_max_profit().unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_protective_put_exact_hedge_max_loss_is_floored_at_strike() {
    // N (C - K) + U p + F = 100 × 5 + 100 × 1.5 + 2.02.
    let strategy = protective_put(95.0, 1.5, Positive::ONE);
    assert_eq!(max_loss(&strategy), pos_or_panic!(652.02));
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
}

#[test]
fn test_protective_put_exact_hedge_zones_unchanged() {
    // An exact hedge keeps the zones it always had: profit from the
    // break-even up, loss from the strike to the break-even.
    let strategy = protective_put(95.0, 1.5, Positive::ONE);
    let loss = strategy.get_loss_ranges().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(loss.len(), 1);
    assert_eq!(loss[0].lower_bound, Some(pos_or_panic!(95.0)));
    assert_eq!(loss[0].upper_bound, Some(pos_or_panic!(101.52)));
    let profit = strategy
        .get_profit_ranges()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(profit.len(), 1);
    assert_eq!(profit[0].lower_bound, Some(pos_or_panic!(101.52)));
    assert_eq!(profit[0].upper_bound, None);
}

#[test]
fn test_protective_put_under_hedged_max_loss_reached_at_zero() {
    // 50 put units: N C - U K + U p + F = 10000 - 4750 + 75 + 2.01. The
    // unhedged half of the shares keeps falling below the strike.
    let strategy = protective_put(95.0, 1.5, pos_or_panic!(0.5));
    assert_eq!(max_loss(&strategy), pos_or_panic!(5327.01));
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    let at_zero = strategy
        .calculate_profit_at(&Positive::ZERO)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(-at_zero, max_loss(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
    assert_zones(&strategy);
}

#[test]
fn test_protective_put_under_hedged_itm_zones_follow_pnl() {
    // An ITM put struck at 110 on half the shares: the break-even, 100.54,
    // sits below the strike. The exact-hedge zones would have run the loss
    // zone from the strike down to it, an inverted range.
    let strategy = protective_put(110.0, 10.5, pos_or_panic!(0.5));
    assert_eq!(
        strategy.get_break_even_points().ok(),
        Some(&vec![pos_or_panic!(100.54)])
    );
    let loss = strategy.get_loss_ranges().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(loss.len(), 1);
    assert_eq!(loss[0].lower_bound, None);
    assert_eq!(loss[0].upper_bound, Some(pos_or_panic!(100.54)));
    // 10000 - 50 × 110 + 50 × 10.5 + 2.01.
    assert_eq!(max_loss(&strategy), pos_or_panic!(5027.01));
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    assert_zones(&strategy);
}

#[test]
fn test_protective_put_over_hedged_max_loss_at_strike_and_two_zones_of_profit() {
    // 200 put units: the P&L rises again below the strike, so the worst
    // case stays at the strike: 100 × 5 + 200 × 1.5 + 2.04.
    let strategy = protective_put(95.0, 1.5, Positive::TWO);
    assert_eq!(max_loss(&strategy), pos_or_panic!(802.04));
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
    let profit = strategy
        .get_profit_ranges()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(profit.len(), 2);
    assert_eq!(profit[0].lower_bound, None);
    assert_eq!(profit[0].upper_bound, Some(pos_or_panic!(86.98)));
    assert_eq!(profit[1].lower_bound, Some(pos_or_panic!(103.02)));
    assert_eq!(profit[1].upper_bound, None);
    assert_zones(&strategy);
}

// --- Collar -----------------------------------------------------------------

/// A collar on 100 shares bought at 100, a put struck at 95 bought at 1.5
/// and a call struck at 105 sold at 2.4, whose legs are `put` and `call`
/// contracts of 100 units with fees of 0.01 + 0.01 per contract.
fn collar(put: Positive, call: Positive) -> Collar {
    let mut strategy = Collar::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        SHARES,
        pos_or_panic!(1.5),
        pos_or_panic!(2.4),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
        pos_or_panic!(0.01),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    for (contracts, style, side) in [
        (put, OptionStyle::Put, Side::Long),
        (call, OptionStyle::Call, Side::Short),
    ] {
        let mut leg = match style {
            OptionStyle::Put => strategy.long_put.clone(),
            OptionStyle::Call => strategy.short_call.clone(),
        };
        assert_eq!(leg.option.side, side);
        leg.option.quantity = contracts;
        leg.option.contract_size = CONTRACT;
        strategy
            .modify_position(&leg)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    strategy
        .update_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"));
    strategy
}

fn break_evens(strategy: &Collar) -> Vec<Positive> {
    strategy
        .get_break_even_points()
        .unwrap_or_else(|e| panic!("{e}"))
        .clone()
}

#[test]
fn test_collar_exact_hedge_figures_unchanged() {
    // NP = 240 - 150 = 90, F = 2.04: break-even 100 - 87.96 / 100, max
    // profit 500 + 90 - 2.04, max loss 500 - 90 + 2.04.
    let strategy = collar(Positive::ONE, Positive::ONE);
    assert_eq!(break_evens(&strategy), vec![pos_or_panic!(99.12)]);
    assert_eq!(max_profit(&strategy), pos_or_panic!(587.96));
    assert_eq!(max_loss(&strategy), pos_or_panic!(412.04));
    assert_eq!(grid_max(&strategy), max_profit(&strategy).to_dec());
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    let loss = strategy.get_loss_ranges().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(loss[0].lower_bound, Some(pos_or_panic!(95.0)));
    assert_eq!(loss[0].upper_bound, Some(pos_or_panic!(99.12)));
    let profit = strategy
        .get_profit_ranges()
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(profit[0].lower_bound, Some(pos_or_panic!(99.12)));
    assert_eq!(profit[0].upper_bound, Some(pos_or_panic!(105.0)));
}

#[test]
fn test_collar_under_hedged_put_max_loss_reached_at_zero() {
    // 50 put units: NP = 240 - 75 = 165, F = 2.03. Break-even
    // 100 - 162.97 / 100; below the put strike the P&L keeps falling with
    // slope 50, down to -10000 + 4750 + 162.97 at zero.
    let strategy = collar(pos_or_panic!(0.5), Positive::ONE);
    assert_eq!(break_evens(&strategy), vec![pos_or_panic!(98.37)]);
    assert_eq!(max_loss(&strategy), pos_or_panic!(5087.03));
    assert_eq!(max_profit(&strategy), pos_or_panic!(662.97));
    assert_eq!(grid_max(&strategy), max_profit(&strategy).to_dec());
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
    assert_zones(&strategy);
}

#[test]
fn test_collar_over_hedged_put_has_two_break_evens() {
    // 200 put units: NP = 240 - 300 = -60, F = 2.06. Between the strikes
    // the zero is 100 + 62.06 / 100; below the put strike the P&L rises
    // with slope -100 and crosses zero at (10000 - 19000 + 62.06) / -100.
    // The worst case is at the put strike, the best at zero.
    let strategy = collar(Positive::TWO, Positive::ONE);
    assert_eq!(
        break_evens(&strategy),
        vec![pos_or_panic!(89.38), pos_or_panic!(100.62)]
    );
    assert_eq!(max_loss(&strategy), pos_or_panic!(562.06));
    assert_eq!(max_profit(&strategy), pos_or_panic!(8937.94));
    assert_eq!(grid_max(&strategy), max_profit(&strategy).to_dec());
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
    assert_zones(&strategy);
}

#[test]
fn test_collar_under_covered_call_profit_is_unbounded() {
    // 50 call units: half the shares keep rising above the call strike.
    // NP = 120 - 150 = -30, F = 2.03; the floor is -500 - 32.03.
    let strategy = collar(Positive::ONE, pos_or_panic!(0.5));
    assert_eq!(break_evens(&strategy), vec![pos_or_panic!(100.32)]);
    assert_eq!(max_profit(&strategy), Positive::MAX);
    assert_eq!(max_loss(&strategy), pos_or_panic!(532.03));
    assert_eq!(-grid_min(&strategy), max_loss(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
    assert_zones(&strategy);
}

#[test]
fn test_collar_over_covered_call_loss_is_unbounded() {
    // 200 call units: net short 100 units above the call strike. NP =
    // 480 - 150 = 330, F = 2.06. Zeros at 100 - 327.94 / 100 and, above
    // the call strike, (10000 - 21000 - 327.94) / -100.
    let strategy = collar(Positive::ONE, Positive::TWO);
    assert_eq!(
        break_evens(&strategy),
        vec![pos_or_panic!(96.72), pos_or_panic!(113.28)]
    );
    assert_eq!(max_loss(&strategy), Positive::MAX);
    assert_eq!(max_profit(&strategy), pos_or_panic!(827.94));
    assert_eq!(grid_max(&strategy), max_profit(&strategy).to_dec());
    assert_break_evens_are_zeros(&strategy);
    assert_zones(&strategy);
}
