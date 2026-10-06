//! Property-based tests for panic freedom of the analytics primitives the
//! strategies compose: the probability kernels, the SPAN margin model and the
//! P&L primitives.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid: values at the smallest representable scale, at
//! `Positive::MAX`, rates at `±1e6`, a quantity of zero, a premium larger than
//! the strike, fees larger than the premium. The assertion is deliberately
//! weak: whatever comes back, it must come back.
//!
//! Moved from the facade's `tests/property/strategies_panic_freedom_test.rs`
//! (#534); the strategy properties of that file live in
//! `optionstratlib-strategies` (`tests/panic_freedom_test.rs`).

use optionstratlib_analytics::analytics::PriceTrend;
use optionstratlib_analytics::analytics::VolatilityAdjustment;
use optionstratlib_analytics::analytics::calculate_price_probability;
use optionstratlib_analytics::analytics::calculate_single_point_probability;
use optionstratlib_analytics::pnl::{PnL, PnLCalculator};
use optionstratlib_analytics::risk::SPANMargin;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_pricing::pricing::Profit;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`. A quantity or a spot at this scale is
/// what turns an ordinary per-contract division into an overflow.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// A `Positive` from a `Decimal` literal that is positive by construction.
fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Prices, strikes, premia and quantities across the whole `Positive` range,
/// including the two ends that break the arithmetic.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.01))),
        Just(pos(dec!(0.2))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(1000000000000000))),
        Just(Positive::MAX),
    ]
}

/// Premia and fees over the whole `Positive` range, up to and including the
/// magnitude at which a single leg's own `Position::total_cost` overflows.
fn extreme_money() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.01))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(100000000000000))),
        Just(Positive::MAX),
    ]
}

/// Contract and share counts. Zero is kept: it is the divisor of every
/// per-contract and per-share figure in this layer.
fn extreme_quantity() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(1000000))),
    ]
}

/// Volatilities over the whole `Positive` range, up to and including the
/// magnitude at which averaging them through `model::utils::mean_and_std`
/// overflows.
fn extreme_volatility() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.2))),
        Just(Positive::ONE),
        Just(pos(dec!(1000000))),
        Just(pos(dec!(1000000000000000))),
        Just(Positive::MAX),
    ]
}

/// Rates and dividends over the signed `Decimal` range.
fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.05)),
        Just(dec!(-0.05)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

/// Expirations from one already reached, through a sub-second sliver, to
/// horizons past the calendar itself: a billion days overflows the
/// `DateTime + TimeDelta` addition, `1e15` days overflows `TimeDelta`, and
/// `Positive::MAX` days does not even fit the `i64` day count.
fn extreme_expiration() -> impl Strategy<Value = ExpirationDate> {
    prop_oneof![
        Just(ExpirationDate::Days(Positive::ZERO)),
        Just(ExpirationDate::Days(pos(TINY))),
        Just(ExpirationDate::Days(Positive::ONE)),
        Just(ExpirationDate::Days(pos(dec!(30)))),
        Just(ExpirationDate::Days(pos(dec!(3650)))),
        Just(ExpirationDate::Days(pos(dec!(1000000000)))),
        Just(ExpirationDate::Days(pos(dec!(1000000000000000)))),
        Just(ExpirationDate::Days(Positive::MAX)),
    ]
}

/// A leg built straight from the extremes, so a zero quantity, a premium above
/// the strike and fees above the premium all reach the composition layer.
fn extreme_position() -> impl Strategy<Value = Position> {
    (
        extreme_positive(),
        extreme_positive(),
        extreme_volatility(),
        extreme_quantity(),
        extreme_money(),
        extreme_decimal(),
        extreme_expiration(),
        0usize..4,
    )
        .prop_map(
            |(underlying, strike, volatility, quantity, premium, rate, expiration, kind)| {
                let (style, side) = match kind {
                    0 => (OptionStyle::Call, Side::Long),
                    1 => (OptionStyle::Call, Side::Short),
                    2 => (OptionStyle::Put, Side::Long),
                    _ => (OptionStyle::Put, Side::Short),
                };
                Position::new(
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
                    premium,
                    premium,
                    None,
                    None,
                )
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(192))]

    /// The probability kernel behind every `probability_of_*` call. A zero
    /// spot leaves the log price ratio undefined, a `1e-28` spot against a
    /// hundred-dollar target overflows it, and the reverse pair rounds it to
    /// zero, where the logarithm has no value.
    #[test]
    fn test_probability_kernels_never_panic(
        current in extreme_positive(),
        target in extreme_positive(),
        upper in extreme_positive(),
        base_volatility in extreme_volatility(),
        std_dev_adjustment in extreme_positive(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        drift in prop_oneof![Just(0.0f64), Just(1e30f64), Just(-1e30f64)],
        confidence in prop_oneof![Just(0.0f64), Just(0.8f64), Just(2.0f64)],
    ) {
        // The kernels take the volatility explicitly; the extreme generators
        // drive it instead of a `None` that used to hide a flat 0.2.
        let volatility = VolatilityAdjustment {
            base_volatility,
            std_dev_adjustment,
        };
        let trend = Some(PriceTrend { drift_rate: drift, confidence });
        let _ = calculate_single_point_probability(
            &current, &target, volatility, trend.clone(), &expiration, Some(rate),
        );
        let _ = calculate_price_probability(
            &current, &target, &upper, volatility, trend, &expiration, Some(rate),
        );
    }

    /// The SPAN scenarios scale the spot and the volatility by `1 ± range`. A
    /// scan range past 100% makes that factor negative, which is outside the
    /// price and volatility domains rather than a number to report.
    #[test]
    fn test_span_margin_never_panics(
        position in extreme_position(),
        price_scan_range in extreme_decimal(),
        volatility_scan_range in extreme_decimal(),
        short_option_minimum in extreme_decimal(),
    ) {
        let span = SPANMargin::new(price_scan_range, volatility_scan_range, short_option_minimum);
        let _ = span.calculate_margin(&position);
    }

    /// The P&L primitives: a total whose two halves do not add, and the
    /// per-leg accounting a `Position` reports.
    #[test]
    fn test_pnl_primitives_never_panic(
        realized in extreme_decimal(),
        unrealized in extreme_decimal(),
        costs in extreme_positive(),
        income in extreme_positive(),
        position in extreme_position(),
        probe in extreme_positive(),
    ) {
        let pnl = PnL::new(Some(realized), Some(unrealized), costs, income, chrono::Utc::now());
        let _ = pnl.total_pnl();
        let _ = format!("{pnl:?}");
        let _ = position.total_cost();
        let _ = position.net_cost();
        let _ = position.net_premium_received();
        let _ = position.fees();
        let _ = position.calculate_profit_at(&probe);
        let _ = position.calculate_pnl_at_expiration(&probe);
    }
}
