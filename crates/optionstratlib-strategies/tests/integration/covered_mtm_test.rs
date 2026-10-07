//! Mark-to-market P&L of the strategies with a share leg (#728).
//!
//! `CoveredCall`, `ProtectivePut` and `Collar` used to answer
//! `calculate_pnl` with the payoff at expiry and ignore the date and the
//! volatility. They now mark each option leg through
//! `Position::calculate_pnl` and value the shares at the current price.
//!
//! Two properties per strategy:
//! - the strategy's P&L is the sum of its legs': each option leg's own
//!   `calculate_pnl` plus the shares' `(price - cost basis) * quantity`;
//! - with every leg entered at its Black-Scholes value and no fees, the
//!   unrealized P&L converges to the expiry P&L as the time to expiry goes
//!   to zero. An option's value tends to its intrinsic value, so
//!   `quantity * (BS(now) - BS(entry))` tends to the payoff less the
//!   premium, which is what the expiry path reports once the premium is the
//!   entry value.

use optionstratlib_analytics::pnl::{PnL, PnLCalculator};
use optionstratlib_core::model::{ExpirationDate, Position, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::pricing::OptionPricing;
use optionstratlib_strategies::strategies::{Collar, CoveredCall, ProtectivePut};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The spot at which every strategy is marked.
fn spot() -> Positive {
    pos_or_panic!(103.0)
}

fn days(n: Decimal) -> ExpirationDate {
    ExpirationDate::Days(Positive::new_decimal(n).unwrap_or(Positive::ONE))
}

/// One leg's mark-to-market P&L at the shared marking inputs.
fn leg_pnl(leg: &Position, expiration: ExpirationDate, volatility: Positive) -> PnL {
    leg.calculate_pnl(&spot(), expiration, &volatility)
        .unwrap_or_else(|e| panic!("leg calculate_pnl: {e}"))
}

/// Asserts that `total` is the sum of the shares' P&L and the option legs'.
fn assert_sum_of_legs(
    total: &PnL,
    shares: Positive,
    cost_basis: Positive,
    share_cost: Positive,
    legs: &[PnL],
) {
    let share_change = (spot().to_dec() - cost_basis.to_dec()) * shares.to_dec();
    let unrealized: Decimal = share_change
        + legs
            .iter()
            .map(|l| l.unrealized.unwrap_or_default())
            .sum::<Decimal>();
    let realized: Decimal = -share_cost.to_dec()
        + legs
            .iter()
            .map(|l| l.realized.unwrap_or_default())
            .sum::<Decimal>();
    let costs: Decimal = share_cost.to_dec()
        + legs
            .iter()
            .map(|l| l.initial_costs.to_dec())
            .sum::<Decimal>();
    let income: Decimal = legs.iter().map(|l| l.initial_income.to_dec()).sum();

    assert_eq!(total.unrealized, Some(unrealized));
    assert_eq!(total.realized, Some(realized));
    assert_eq!(total.initial_costs.to_dec(), costs);
    assert_eq!(total.initial_income.to_dec(), income);
}

/// The leg's premium set to its Black-Scholes value at entry, per contract.
fn fair_premium(leg: &mut Position) {
    let price = leg
        .option
        .calculate_price_black_scholes()
        .unwrap_or_else(|e| panic!("entry price: {e}"));
    leg.premium = Positive::new_decimal(price.abs()).unwrap_or(Positive::ZERO);
}

/// Distance between the mark-to-market and the expiry P&L, at three
/// shrinking times to expiry. It must shrink and end close to zero.
fn assert_converges_to_expiry<S: PnLCalculator>(strategy: &S, volatility: Positive) {
    let at_expiry = strategy
        .calculate_pnl_at_expiration(&spot())
        .unwrap_or_else(|e| panic!("expiry: {e}"))
        .unrealized
        .unwrap_or_default();
    let gaps: Vec<Decimal> = [dec!(1), dec!(0.01), dec!(0.0001)]
        .into_iter()
        .map(|t| {
            let mtm = strategy
                .calculate_pnl(&spot(), days(t), &volatility)
                .unwrap_or_else(|e| panic!("mark to market: {e}"))
                .unrealized
                .unwrap_or_default();
            (mtm - at_expiry).abs()
        })
        .collect();
    assert!(gaps[0] > gaps[1] && gaps[1] >= gaps[2], "gaps {gaps:?}");
    assert!(gaps[2] < dec!(0.01), "gaps {gaps:?}");
}

#[test]
fn test_covered_call_mark_to_market_is_sum_of_legs() {
    let strategy = CoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        days(dec!(30)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(2.4),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.65),
        pos_or_panic!(0.65),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let (expiration, volatility) = (days(dec!(10)), pos_or_panic!(0.3));
    let total = strategy
        .calculate_pnl(&spot(), expiration, &volatility)
        .unwrap_or_else(|e| panic!("{e}"));

    assert_sum_of_legs(
        &total,
        strategy.spot_leg.quantity,
        strategy.spot_leg.cost_basis,
        pos_or_panic!(10002.0),
        &[leg_pnl(&strategy.short_call, expiration, volatility)],
    );
    // The date and the volatility are read: the old answer, the expiry
    // P&L, ignored both.
    let other = strategy
        .calculate_pnl(&spot(), days(dec!(20)), &pos_or_panic!(0.4))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(total.unrealized, other.unrealized);
}

#[test]
fn test_covered_call_mark_to_market_converges_to_expiry() {
    let mut strategy = CoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        days(dec!(30)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        Positive::ONE,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    fair_premium(&mut strategy.short_call);
    assert_converges_to_expiry(&strategy, pos_or_panic!(0.2));
}

#[test]
fn test_protective_put_mark_to_market_is_sum_of_legs() {
    let strategy = ProtectivePut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        days(dec!(30)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(1.5),
        Positive::ONE,
        Positive::ONE,
        pos_or_panic!(0.65),
        pos_or_panic!(0.65),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let (expiration, volatility) = (days(dec!(10)), pos_or_panic!(0.3));
    let total = strategy
        .calculate_pnl(&spot(), expiration, &volatility)
        .unwrap_or_else(|e| panic!("{e}"));

    assert_sum_of_legs(
        &total,
        strategy.spot_leg.quantity,
        strategy.spot_leg.cost_basis,
        pos_or_panic!(10002.0),
        &[leg_pnl(&strategy.long_put, expiration, volatility)],
    );
    let other = strategy
        .calculate_pnl(&spot(), days(dec!(20)), &pos_or_panic!(0.4))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(total.unrealized, other.unrealized);
}

#[test]
fn test_protective_put_mark_to_market_converges_to_expiry() {
    let mut strategy = ProtectivePut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        days(dec!(30)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        Positive::ONE,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    fair_premium(&mut strategy.long_put);
    assert_converges_to_expiry(&strategy, pos_or_panic!(0.2));
}

fn collar(premium_put: Positive, premium_call: Positive, fee: Positive) -> Collar {
    Collar::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        days(dec!(30)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        premium_put,
        premium_call,
        fee,
        fee,
        fee,
        fee,
        fee,
        fee,
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_collar_mark_to_market_is_sum_of_legs() {
    let strategy = collar(pos_or_panic!(1.5), pos_or_panic!(2.4), Positive::ONE);
    let (expiration, volatility) = (days(dec!(10)), pos_or_panic!(0.3));
    let total = strategy
        .calculate_pnl(&spot(), expiration, &volatility)
        .unwrap_or_else(|e| panic!("{e}"));

    assert_sum_of_legs(
        &total,
        strategy.spot_leg.quantity,
        strategy.spot_leg.cost_basis,
        pos_or_panic!(10002.0),
        &[
            leg_pnl(&strategy.long_put, expiration, volatility),
            leg_pnl(&strategy.short_call, expiration, volatility),
        ],
    );
    let other = strategy
        .calculate_pnl(&spot(), days(dec!(20)), &pos_or_panic!(0.4))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(total.unrealized, other.unrealized);
}

#[test]
fn test_collar_mark_to_market_converges_to_expiry() {
    let mut strategy = collar(Positive::ONE, Positive::ONE, Positive::ZERO);
    fair_premium(&mut strategy.long_put);
    fair_premium(&mut strategy.short_call);
    assert_converges_to_expiry(&strategy, pos_or_panic!(0.2));
}

/// The expiry path is unchanged: still the payoff, independent of the
/// marking inputs `calculate_pnl` now reads.
#[test]
fn test_collar_expiry_path_is_the_payoff() {
    use optionstratlib_pricing::pricing::Profit;
    let strategy = collar(pos_or_panic!(1.5), pos_or_panic!(2.4), Positive::ONE);
    let expiry = strategy
        .calculate_pnl_at_expiration(&spot())
        .unwrap_or_else(|e| panic!("{e}"));
    let payoff = strategy
        .calculate_profit_at(&spot())
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(expiry.unrealized, Some(payoff));
}
