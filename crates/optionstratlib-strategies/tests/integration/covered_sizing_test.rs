//! The option legs of the stock-plus-option strategies are sized in shares
//! (#731).
//!
//! An option in this library carries no contract multiplier: its payoff is
//! `intrinsic × quantity`. `CoveredCall` and `Collar` used to size their
//! options in 100-share contracts (`quantity / 100`), so one option unit
//! covered one share out of a hundred and their payoffs were neither capped
//! nor floored, while `ProtectivePut` sized its put in shares. All three now
//! cover their shares one for one, with option fees per share.
//!
//! Fixture: 100 shares at 100, put 95 for 1.50, call 105 for 2.40, share
//! fees 1 + 1, option fees 0.01 + 0.01 per share (2.00 per leg).

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::strategies::{Collar, CoveredCall, ProtectivePut};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn expiry() -> ExpirationDate {
    ExpirationDate::Days(pos_or_panic!(30.0))
}

fn option_fee() -> Positive {
    pos_or_panic!(0.01)
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
        option_fee(),
        option_fee(),
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
        option_fee(),
        option_fee(),
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
        option_fee(),
        option_fee(),
        option_fee(),
        option_fee(),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn profit<S: Profit>(strategy: &S, price: f64) -> Decimal {
    strategy
        .calculate_profit_at(&pos_or_panic!(price))
        .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn test_protective_put_and_collar_build_identical_put_legs() {
    let protective = protective_put();
    let collar = collar();
    assert_eq!(protective.long_put.option, collar.long_put.option);
    assert_eq!(protective.long_put.premium, collar.long_put.premium);
    assert_eq!(protective.long_put.open_fee, collar.long_put.open_fee);
    assert_eq!(protective.long_put.close_fee, collar.long_put.close_fee);
    // One put unit per share.
    assert_eq!(protective.long_put.option.quantity, Positive::HUNDRED);
}

#[test]
fn test_covered_call_payoff_capped_and_matches_its_bounds() {
    let strategy = covered_call();
    assert_eq!(strategy.short_call.option.quantity, Positive::HUNDRED);
    // Max profit = (105 - 100) x 100 + 2.40 x 100 - (1 + 1) - 0.02 x 100
    //            = 500 + 240 - 2 - 2 = 736.
    let max_profit = strategy.max_profit_potential().unwrap_or_default();
    assert_eq!(max_profit, pos_or_panic!(736.0));
    for price in [105.0, 150.0, 300.0] {
        assert_eq!(profit(&strategy, price), dec!(736));
    }
    // Max loss = 100 x 100 + 4 - 240 = 9764, at a share price of zero.
    let max_loss = strategy.max_loss_potential().unwrap_or_default();
    assert_eq!(max_loss, pos_or_panic!(9764.0));
    assert_eq!(profit(&strategy, 0.0), dec!(-9764));
}

#[test]
fn test_protective_put_payoff_floored_and_matches_its_bound() {
    let strategy = protective_put();
    // Max loss = (100 - 95) x 100 + 1.50 x 100 + (1 + 1) + 0.02 x 100
    //          = 500 + 150 + 2 + 2 = 654.
    let max_loss = strategy.max_loss_potential().unwrap_or_default();
    assert_eq!(max_loss, pos_or_panic!(654.0));
    for price in [95.0, 50.0, 0.0] {
        assert_eq!(profit(&strategy, price), dec!(-654));
    }
}

#[test]
fn test_collar_payoff_capped_floored_and_matches_its_bounds() {
    let strategy = collar();
    // Net credit 2.40 - 1.50 = 0.90 x 100 = 90; fees 2 + 2 + 2 = 6.
    // Max profit = (105 - 100) x 100 + 90 - 6 = 584.
    let max_profit = strategy.max_profit_potential().unwrap_or_default();
    assert_eq!(max_profit, pos_or_panic!(584.0));
    for price in [105.0, 150.0, 300.0] {
        assert_eq!(profit(&strategy, price), dec!(584));
    }
    // Max loss = (100 - 95) x 100 - 90 + 6 = 416.
    let max_loss = strategy.max_loss_potential().unwrap_or_default();
    assert_eq!(max_loss, pos_or_panic!(416.0));
    for price in [95.0, 50.0, 0.0] {
        assert_eq!(profit(&strategy, price), dec!(-416));
    }
}

/// Each break-even is where the payoff crosses zero, fees included.
#[test]
fn test_covered_strategies_break_even_is_the_payoff_zero() {
    use optionstratlib_strategies::strategies::base::BreakEvenable;
    // Covered call: 100 - 2.40 + (2 + 2) / 100 = 97.64.
    let strategy = covered_call();
    assert_eq!(
        strategy
            .get_break_even_points()
            .unwrap_or_else(|e| panic!("{e}")),
        &vec![pos_or_panic!(97.64)]
    );
    assert_eq!(profit(&strategy, 97.64), Decimal::ZERO);
    // Protective put: 100 + 1.50 + (2 + 2) / 100 = 101.54.
    let strategy = protective_put();
    assert_eq!(
        strategy
            .get_break_even_points()
            .unwrap_or_else(|e| panic!("{e}")),
        &vec![pos_or_panic!(101.54)]
    );
    assert_eq!(profit(&strategy, 101.54), Decimal::ZERO);
    // Collar: 100 - 0.90 + (2 + 2 + 2) / 100 = 99.16.
    let strategy = collar();
    assert_eq!(
        strategy
            .get_break_even_points()
            .unwrap_or_else(|e| panic!("{e}")),
        &vec![pos_or_panic!(99.16)]
    );
    assert_eq!(profit(&strategy, 99.16), Decimal::ZERO);
}

/// `get_volume` counts option contracts and `get_share_volume` counts the
/// share leg in units of the underlying, so the two never mix (#830). With a
/// contract size of 100 the 100 shares are covered by one call contract.
#[test]
fn test_volume_reports_contracts_and_shares_apart() {
    use optionstratlib_strategies::strategies::{BasicAble, Strategies};

    let mut call = covered_call();
    let mut put = protective_put();
    let mut col = collar();
    // At a contract size of 1 each option leg holds 100 contracts; the
    // collar has two option legs.
    for (name, volume, shares, contracts) in [
        (
            "covered call",
            call.get_volume(),
            call.get_share_volume(),
            Positive::HUNDRED,
        ),
        (
            "protective put",
            put.get_volume(),
            put.get_share_volume(),
            Positive::HUNDRED,
        ),
        (
            "collar",
            col.get_volume(),
            col.get_share_volume(),
            pos_or_panic!(200.0),
        ),
    ] {
        assert_eq!(volume.ok(), Some(contracts), "{name} contracts");
        assert_eq!(shares.ok(), Some(Positive::HUNDRED), "{name} shares");
    }

    call.set_contract_size(Positive::HUNDRED)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(call.get_volume().ok(), Some(Positive::ONE));
    assert_eq!(call.get_share_volume().ok(), Some(Positive::HUNDRED));

    col.set_contract_size(Positive::HUNDRED)
        .unwrap_or_else(|e| panic!("{e}"));
    // A collar has two option legs of one contract each.
    assert_eq!(col.get_volume().ok(), Some(Positive::TWO));
    assert_eq!(col.get_share_volume().ok(), Some(Positive::HUNDRED));
}
