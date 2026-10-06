//! Property-based test for panic freedom of the strategy calls that walk a
//! price range, the payoff chart (`Graph::graph_data`) among them.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid. The assertion is deliberately weak: whatever comes
//! back, it must come back.
//!
//! It lives in `optionstratlib-visualization` because `Graph` is a
//! visualization trait; it left the facade with the crate (#542). The other
//! properties of this file moved with their layers (#534): the strategy ones
//! to `optionstratlib-strategies` and the probability kernel, SPAN margin and
//! P&L primitive ones to `optionstratlib-analytics`, both as
//! `tests/panic_freedom_test.rs`.

use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::{OptionStyle, OptionType, Options, Position, Side};
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::BullCallSpread;
use optionstratlib_strategies::strategies::base::{BreakEvenable, Positionable, Strategies};
use optionstratlib_strategies::strategies::probabilities::ProbabilityAnalysis;
use optionstratlib_visualization::visualization::Graph;
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

/// The same range restricted to magnitudes a sampling walk can cross without
/// asking for millions of points. Used only where a test walks a price range.
///
/// `expected_value` samples at one percent of the spot across a range set by
/// the break-even points, so a spot far below the premium turns an ordinary
/// integration into millions of evaluations. These rungs keep the two within
/// two orders of magnitude of each other; the wide-open magnitudes are driven
/// by the tests that do not walk.
fn walkable_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(dec!(0.01))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
    ]
}

/// Spot prices and premia for the walking tests, kept on one scale for the
/// reason given on [`walkable_positive`].
fn walkable_money() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(dec!(0.01))),
        Just(Positive::ONE),
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

proptest! {
    // A price walk is bounded by the number of samples it needs, so these
    // cases stay at magnitudes a chart can actually cross. The zero step and
    // the range past the sample cap are pinned as unit tests next to
    // `calculate_price_range` instead.
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Every call that walks a price range: the display range, the payoff
    /// chart, the expected value integration and the full probability
    /// analysis.
    #[test]
    fn test_price_range_walks_never_panic(
        // `expected_value` samples at one percent of the spot over a range set
        // by the strikes, so the spot alone fixes how many samples the walk
        // takes. A zero spot is kept: it collapses that step to zero, which is
        // the input that used to spin `calculate_price_range` forever.
        underlying in prop_oneof![Just(Positive::ZERO), Just(Positive::HUNDRED)],
        long_strike in walkable_positive(),
        short_strike in walkable_positive(),
        volatility in walkable_positive(),
        quantity in prop_oneof![Just(Positive::ONE), Just(Positive::HUNDRED)],
        premium in walkable_money(),
        step in prop_oneof![
            Just(Positive::ZERO),
            Just(Positive::HUNDRED),
            Just(Positive::MAX),
        ],
        expiration in extreme_expiration(),
    ) {
        let strategy = match BullCallSpread::new(
            "PROP".to_string(), underlying, long_strike, short_strike, expiration,
            volatility, dec!(0.05), Positive::ZERO, quantity, premium, premium,
            Positive::ZERO, Positive::ZERO, Positive::ZERO, Positive::ZERO,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                // Since #696 `new` rejects the legs that fail `validate`
                // (equal or inverted strikes, a short leg with no premium).
                // The public fields still admit them, so the rejected case is
                // assembled by hand and walked all the same.
                prop_assert!(
                    matches!(
                        error,
                        StrategyError::InvalidStrategy { .. } | StrategyError::OperationError(_)
                    ),
                    "unexpected error: {error}"
                );
                let leg = |side: Side, strike: Positive| {
                    let strike = if strike == Positive::ZERO { underlying } else { strike };
                    Position::new(
                        Options::new(
                            OptionType::European, side, "PROP".to_string(), strike,
                            expiration, volatility, quantity, underlying, dec!(0.05),
                            OptionStyle::Call, Positive::ZERO, None,
                        ),
                        premium, chrono::Utc::now(), Positive::ZERO, Positive::ZERO,
                        None, None,
                    )
                };
                let mut strategy = BullCallSpread::default();
                let _ = strategy.add_position(&leg(Side::Long, long_strike));
                let _ = strategy.add_position(&leg(Side::Short, short_strike));
                let _ = strategy.update_break_even_points();
                strategy
            }
        };
        let _ = strategy.get_best_range_to_show(step);
        let _ = strategy.graph_data();
        let _ = strategy.expected_value(None, None);
        let _ = strategy.analyze_probabilities(None, None);
    }

}
