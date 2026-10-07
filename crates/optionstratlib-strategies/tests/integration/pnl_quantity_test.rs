//! A strategy's mark-to-market P&L counts every contract of every leg
//! (#725). `Position::calculate_pnl` reported one contract's change whatever
//! the quantity, so a butterfly's doubled body, or any strategy sized above
//! one lot, summed to the wrong unrealized P&L.

use chrono::Utc;
use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_analytics::pnl::utils::PnL;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::{
    ExpirationDate, Options,
    position::Position,
    types::{OptionStyle, OptionType, Side},
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_strategies::strategies::BullCallSpread;
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn leg(side: Side, strike: f64, quantity: Positive, premium: f64) -> Position {
    Position::new(
        Options::new(
            OptionType::European,
            side,
            "SPX".to_string(),
            pos_or_panic!(strike),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            quantity,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        ),
        pos_or_panic!(premium),
        Utc::now(),
        pos_or_panic!(0.1),
        pos_or_panic!(0.1),
        None,
        None,
    )
}

/// Long 1x95, short 2x100, long 1x105 calls, every quantity times `lots`.
fn butterfly_legs(lots: Decimal) -> Vec<Position> {
    let scaled = |q: Decimal| match Positive::new_decimal(q * lots) {
        Ok(p) => p,
        Err(e) => panic!("quantity {q} x {lots}: {e}"),
    };
    vec![
        leg(Side::Long, 95.0, scaled(Decimal::ONE), 7.2),
        leg(Side::Short, 100.0, scaled(Decimal::TWO), 3.9),
        leg(Side::Long, 105.0, scaled(Decimal::ONE), 1.8),
    ]
}

fn butterfly(lots: Decimal) -> CustomStrategy {
    match CustomStrategy::new(
        "Butterfly".to_string(),
        "SPX".to_string(),
        "1/2/1 call butterfly".to_string(),
        Positive::HUNDRED,
        butterfly_legs(lots),
        pos_or_panic!(0.01),
        100,
        pos_or_panic!(0.1),
    ) {
        Ok(strategy) => strategy,
        Err(e) => panic!("butterfly builds: {e}"),
    }
}

fn mark<T: PnLCalculator>(item: &T) -> PnL {
    match item.calculate_pnl(
        &pos_or_panic!(103.0),
        ExpirationDate::Days(pos_or_panic!(15.0)),
        &pos_or_panic!(0.22),
    ) {
        Ok(pnl) => pnl,
        Err(e) => panic!("mark-to-market evaluates: {e}"),
    }
}

fn unrealized(pnl: &PnL) -> Decimal {
    match pnl.unrealized {
        Some(value) => value,
        None => panic!("calculate_pnl reports an unrealized P&L"),
    }
}

#[test]
fn test_butterfly_calculate_pnl_equals_sum_of_legs() {
    let strategy = butterfly(Decimal::ONE);
    let total = unrealized(&mark(&strategy));
    let legs: Decimal = butterfly_legs(Decimal::ONE)
        .iter()
        .map(|position| unrealized(&mark(position)))
        .sum();
    assert_eq!(total, legs);

    // The body is two contracts: its P&L is twice a one-contract short.
    let body = unrealized(&mark(&leg(Side::Short, 100.0, Positive::TWO, 3.9)));
    let one_body = unrealized(&mark(&leg(Side::Short, 100.0, Positive::ONE, 3.9)));
    assert_eq!(body, one_body * dec!(2));
}

#[test]
fn test_butterfly_calculate_pnl_scales_with_lots() {
    let one = unrealized(&mark(&butterfly(Decimal::ONE)));
    assert_ne!(one, Decimal::ZERO);
    for lots in [dec!(2), dec!(5)] {
        assert_eq!(unrealized(&mark(&butterfly(lots))), one * lots, "x{lots}");
    }
}

#[test]
fn test_bull_call_spread_calculate_pnl_scales_with_quantity() {
    let spread = |quantity: Positive| match BullCallSpread::new(
        "SPX".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        quantity,
        pos_or_panic!(7.2),
        pos_or_panic!(1.8),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    ) {
        Ok(spread) => spread,
        Err(e) => panic!("spread builds: {e}"),
    };
    let one = mark(&spread(Positive::ONE));
    let three = mark(&spread(pos_or_panic!(3.0)));
    assert_eq!(unrealized(&three), unrealized(&one) * dec!(3));
}
