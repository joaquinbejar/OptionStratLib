//! Property-based test for panic freedom of the P&L layer on a `Position`.
//!
//! `Position`'s own arithmetic, the expiration resolver and `mean_and_std`
//! are core and are driven by
//! `crates/optionstratlib-core/tests/model_panic_freedom_test.rs`. What stays
//! here is the P&L trait over the same extreme inputs: premia and fees that
//! overflow their accumulation, quantities that overflow the product, and
//! horizons the calendar cannot hold. Whatever comes back, it must come back.

#![cfg(feature = "clock")]

use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::model::{ExpirationDate, Options, Position};
use optionstratlib::pnl::PnLCalculator;
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`. A quantity at this scale is what
/// turns an ordinary per-contract division into an overflow.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// A `Positive` from a `Decimal` literal that is non-negative by construction.
fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Prices, strikes, premia, fees and quantities across the whole `Positive`
/// range, including the two ends that break the arithmetic.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.01))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(1000))),
        Just(pos(dec!(1000000000000000))),
        Just(Positive::MAX),
    ]
}

/// Rates over the signed `Decimal` range.
fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(dec!(0.05)),
        Just(dec!(-0.05)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

/// Expirations from one already reached to horizons past the calendar itself:
/// a billion days overflows the `DateTime + TimeDelta` addition, `1e15` days
/// overflows `TimeDelta`, and `Positive::MAX` days does not fit the `i64` day
/// count the conversion goes through.
fn extreme_expiration() -> impl Strategy<Value = ExpirationDate> {
    prop_oneof![
        Just(ExpirationDate::Days(Positive::ZERO)),
        Just(ExpirationDate::Days(pos(TINY))),
        Just(ExpirationDate::Days(pos(dec!(30)))),
        Just(ExpirationDate::Days(pos(dec!(3650)))),
        Just(ExpirationDate::Days(pos(dec!(1000000000)))),
        Just(ExpirationDate::Days(pos(dec!(1000000000000000)))),
        Just(ExpirationDate::Days(Positive::MAX)),
        Just(ExpirationDate::DateTime(chrono::Utc::now())),
    ]
}

/// The four side and style combinations, so the break-even reaches both the
/// addition and the subtraction of each.
fn extreme_kind() -> impl Strategy<Value = (Side, OptionStyle)> {
    prop_oneof![
        Just((Side::Long, OptionStyle::Call)),
        Just((Side::Short, OptionStyle::Call)),
        Just((Side::Long, OptionStyle::Put)),
        Just((Side::Short, OptionStyle::Put)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// `PnLCalculator::calculate_pnl_at_expiration` for a `Position`, over
    /// the same extreme inputs the core suite drives through the position's
    /// own arithmetic (`crates/optionstratlib-core/tests/model_panic_freedom_test.rs`).
    #[test]
    fn test_position_pnl_calculator_never_panics(
        strike in extreme_positive(),
        underlying in extreme_positive(),
        premium in extreme_positive(),
        open_fee in extreme_positive(),
        close_fee in extreme_positive(),
        quantity in extreme_positive(),
        volatility in extreme_positive(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
        (side, style) in extreme_kind(),
    ) {
        let position = Position::new(
            Options::new(
                OptionType::European,
                side,
                "PROP".to_string(),
                strike,
                expiration,
                volatility,
                quantity,
                underlying,
                rate,
                style,
                Positive::ZERO,
                None,
            ),
            premium,
            chrono::Utc::now(),
            open_fee,
            close_fee,
            None,
            None,
        );

        let _ = position.calculate_pnl_at_expiration(&probe);
    }
}
