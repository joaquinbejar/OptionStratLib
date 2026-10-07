//! `calculate_profit_at` on the strategies with a share leg reports a leg
//! that cannot be valued (#731).
//!
//! `CoveredCall`, `ProtectivePut` and `Collar` used to replace a failed
//! option-leg valuation with `unwrap_or(Decimal::ZERO)`, so the payoff, the
//! charts and the expiry P&L silently dropped the leg. A premium at the top
//! of the `Positive` range on a two-contract leg overflows the leg's cost
//! or income, and that overflow must now come back as an error.

use optionstratlib_core::model::{ExpirationDate, Position, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::strategies::{Collar, CoveredCall, ProtectivePut};
use rust_decimal_macros::dec;

fn expiry() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

/// A leg whose premium times its quantity leaves the `Positive` range.
fn overflow(leg: &mut Position) {
    leg.premium = Positive::MAX;
    leg.option.quantity = Positive::TWO;
}

#[test]
fn test_covered_call_profit_at_reports_failing_leg() {
    let mut strategy = CoveredCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(2.4),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_ok());
    overflow(&mut strategy.short_call);
    assert!(strategy.short_call.pnl_at_expiration(&None).is_err());
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_err());
}

#[test]
fn test_protective_put_profit_at_reports_failing_leg() {
    let mut strategy = ProtectivePut::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        expiry(),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::HUNDRED,
        pos_or_panic!(1.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_ok());
    overflow(&mut strategy.long_put);
    assert!(strategy.long_put.pnl_at_expiration(&None).is_err());
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_err());
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
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_collar_profit_at_reports_failing_put_leg() {
    let mut strategy = collar();
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_ok());
    overflow(&mut strategy.long_put);
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_err());
}

#[test]
fn test_collar_profit_at_reports_failing_call_leg() {
    let mut strategy = collar();
    overflow(&mut strategy.short_call);
    assert!(strategy.calculate_profit_at(&Positive::HUNDRED).is_err());
}
