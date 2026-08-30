//! Property-based tests for panic freedom in the model layer.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! Three families are driven here. `Position`'s cost and premium arithmetic,
//! which accumulates a premium and two fees and scales the total by the
//! contract quantity — the product overflows `Positive` long before the
//! factors do, and the break-even subtracts a per-contract cost from a strike
//! that may be smaller than it. The resolution of a relative expiration, whose
//! `DateTime + TimeDelta` addition overflows past a horizon of roughly
//! 260 000 years. And `mean_and_std`, which sums a sample and divides by its
//! length, so it meets both an overflow and an empty divisor. The assertion is
//! deliberately weak: whatever comes back, it must come back.

use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::model::utils::mean_and_std;
use optionstratlib::model::{
    ExpirationDate, Options, Position, TradeAble, reject_unrepresentable_expiration,
    resolve_expiration_date,
};
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

    /// Every monetary figure a `Position` exposes, over premia and fees that
    /// overflow their own accumulation and quantities that overflow the
    /// product with it.
    #[test]
    fn test_position_costs_never_panic(
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

        let _ = position.total_cost();
        let _ = position.fees();
        let _ = position.net_cost();
        let _ = position.premium_received();
        let _ = position.net_premium_received();
        let _ = position.break_even();
        let _ = position.validate();
        let _ = position.trade();
        let _ = position.pnl_at_expiration(&Some(&probe));
        let _ = position.pnl_at_expiration(&None);
        let _ = position.unrealized_pnl(probe);
        let _ = position.calculate_pnl_at_expiration(&probe);
    }

    /// The relative-expiration resolver over horizons the calendar cannot
    /// hold. Both entry points must report rather than abort, and they must
    /// agree: a day count the guard accepts is one the resolver can resolve.
    #[test]
    fn test_expiration_resolution_never_panics(expiration in extreme_expiration()) {
        let resolved = resolve_expiration_date(&expiration);
        let accepted = reject_unrepresentable_expiration(&expiration);
        if accepted.is_ok() {
            prop_assert!(resolved.is_ok());
        }
    }

    /// The sample mean and standard deviation over values that overflow their
    /// own sum, and over the empty sample that has no mean at all.
    #[test]
    fn test_mean_and_std_never_panics(sample in prop::collection::vec(extreme_positive(), 0..6)) {
        let _ = mean_and_std(sample);
    }
}
