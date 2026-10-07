//! Property-based tests for panic freedom across the strategies: the
//! strategy trait defaults, the single-leg and spot-leg strategies, the
//! vertical spreads, straddles, strangles, condors and butterflies, and the
//! custom strategy.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! Two axes are driven at once. The numeric axis is the usual `Decimal`
//! domain: values at the smallest representable scale, at `Positive::MAX`,
//! rates at `±1e6`, and the ordinary values in between. The structural axis is
//! what makes a *strategy* degenerate rather than merely large: a quantity of
//! zero, so every per-contract and per-share divisor collapses; a premium
//! larger than the strike, so the break-even falls below zero and the strategy
//! has none; fees larger than the premium, so a credit turns into a debit; an
//! empty leg set; strikes spread further from the spot than the spot itself.
//! The assertion is deliberately weak: whatever comes back, it must come back.
//!
//! Moved from the facade's `tests/property/strategies_panic_freedom_test.rs`
//! (#534). The probability kernel, SPAN margin and P&L primitive properties
//! of that file live in `optionstratlib-analytics`
//! (`tests/panic_freedom_test.rs`); the price-range walk, which also draws the
//! payoff chart, lives in `optionstratlib-visualization`
//! (`tests/strategies_panic_freedom_test.rs`) beside `Graph`.

use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_core::model::leg::SpotPosition;
use optionstratlib_core::model::types::Action;
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Position, Positive, Side,
};
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::Profit;
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::{
    BasicAble, BreakEvenable, Positionable, Strategies, StrategyType, Validable,
};
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib_strategies::strategies::probabilities::ProbabilityAnalysis;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, Collar,
    CoveredCall, IronButterfly, IronCondor, LongButterflySpread, LongCall, LongStraddle,
    LongStrangle, PoorMansCoveredCall, ProtectivePut, ShortButterflySpread, ShortPut,
    ShortStraddle, ShortStrangle, StrategyConstructor,
};
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

/// Whether `error` is one a strategy constructor returns for a degenerate
/// input: the strategy's own validation (`InvalidStrategy`, #696), a rejected
/// parameter or a checked-arithmetic overflow (both `OperationError`), or a
/// `Positive` range error. Anything else is a constructor reporting a failure
/// it should not have.
fn is_constructor_error(error: &StrategyError) -> bool {
    matches!(
        error,
        StrategyError::InvalidStrategy { .. }
            | StrategyError::OperationError(_)
            | StrategyError::PositiveError(_)
    )
}

/// `strike`, or the spot when `strike` is zero: the default the vertical
/// spreads and the straddles apply in `new`.
fn or_spot(strike: Positive, underlying: Positive) -> Positive {
    if strike == Positive::ZERO {
        underlying
    } else {
        strike
    }
}

/// Places `legs` on a default strategy through `add_position`, in order.
///
/// Since #696 `new` rejects the legs that fail the strategy's own
/// `validate`, which is most of the structural axis of these properties: a
/// zero quantity, a short leg with no premium, inverted or equal strikes.
/// Those legs still reach the post-construction code through the public
/// fields and `Deserialize`, so a rejected case is rebuilt here and driven
/// all the same.
fn assembled<S: Default + Positionable>(legs: &[Position]) -> S {
    let mut strategy = S::default();
    for leg in legs {
        let _ = strategy.add_position(leg);
    }
    strategy
}

/// Everything the trait defaults and the payoff surface expose, minus the
/// calls that walk a price range; those are driven separately with bounded
/// magnitudes so the property stays a property and not a benchmark.
fn exercise<S>(strategy: &S, probe_price: Positive)
where
    S: Strategies
        + BasicAble
        + BreakEvenable
        + Positionable
        + Validable
        + Profit
        + ProbabilityAnalysis
        + Greeks
        + DeltaNeutrality
        + PnLCalculator,
{
    let _ = strategy.get_max_profit();
    let _ = strategy.get_max_loss();
    let _ = strategy.get_total_cost();
    let _ = strategy.get_net_cost();
    let _ = strategy.get_net_premium_received();
    let _ = strategy.get_fees();
    let _ = strategy.get_profit_area();
    let _ = strategy.get_profit_ratio();
    let _ = strategy.get_max_min_strikes();
    let _ = strategy.get_range_to_show();
    let _ = strategy.get_range_of_profit();
    let _ = strategy.get_title();
    let _ = strategy.get_strikes();
    let _ = strategy.get_underlying_price();
    let _ = strategy.validate();
    let _ = strategy.get_break_even_points();
    let _ = strategy.get_positions();
    let _ = strategy.delta();
    let _ = strategy.gamma();
    let _ = strategy.greeks();
    let _ = strategy.delta_neutrality();
    let _ = strategy.is_delta_neutral();
    let _ = strategy.get_atm_strike();
    let _ = strategy.delta_adjustments();
    let _ = strategy.portfolio_greeks();
    let _ = strategy.delta_gap(Decimal::ZERO);
    let _ = strategy.get_profit_ranges();
    let _ = strategy.get_loss_ranges();
    let _ = strategy.probability_of_profit(None, None);
    let _ = strategy.probability_of_loss(None, None);
    let _ = strategy.calculate_extreme_probabilities(None, None);
    let _ = strategy.calculate_profit_at(&probe_price);
    let _ = strategy.calculate_pnl_at_expiration(&probe_price);
    let _ = strategy.calculate_pnl(
        &probe_price,
        ExpirationDate::Days(Positive::ONE),
        &pos(dec!(0.2)),
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(192))]

    /// The two-leg spread drives the `Strategies` trait defaults: the running
    /// cost and fee totals, the display range widened by a strike distance
    /// larger than the spot, and the profit range across an unordered pair of
    /// break-even points.
    #[test]
    fn test_bull_call_spread_never_panics(
        underlying in extreme_positive(),
        long_strike in extreme_positive(),
        short_strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        fee in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
    ) {
        let leg = |side: Side, strike: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(),
                    or_spot(strike, underlying), expiration, volatility, quantity,
                    underlying, rate, OptionStyle::Call, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };
        let mut strategy = match BullCallSpread::new(
            "PROP".to_string(), underlying, long_strike, short_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<BullCallSpread>(&[
                    leg(Side::Long, long_strike),
                    leg(Side::Short, short_strike),
                ])
            }
        };
        exercise(&strategy, probe);
        let _ = strategy.update_break_even_points();
        let _ = strategy.get_volume();
        let _ = strategy.set_underlying_price(&underlying);
        let _ = strategy.set_implied_volatility(&volatility);
        let _ = strategy.apply_delta_adjustments(Some(Action::Buy));
    }

    /// The four single-leg strategies. Their break-even divides the net cost
    /// by the quantity and their profit area reads the first break-even point,
    /// so a zero quantity and a premium above the strike both land here.
    #[test]
    fn test_single_leg_strategies_never_panic(
        underlying in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        fee in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
    ) {
        let leg = |style: OptionStyle, side: Side| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, rate, style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };
        let mut strategy = match LongCall::new(
            "PROP".to_string(), strike, expiration, volatility, quantity,
            underlying, rate, Positive::ZERO, premium, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<LongCall>(&[leg(OptionStyle::Call, Side::Long)])
            }
        };
        exercise(&strategy, probe);
        let _ = strategy.update_break_even_points();
        let mut strategy = match ShortPut::new(
            "PROP".to_string(), strike, expiration, volatility, quantity,
            underlying, rate, Positive::ZERO, premium, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<ShortPut>(&[leg(OptionStyle::Put, Side::Short)])
            }
        };
        exercise(&strategy, probe);
        let _ = strategy.update_break_even_points();

        // `ShortCall` and `LongPut` are deliberately absent from this
        // property. Their `new` is private and their
        // `StrategyConstructor::get_strategy` always returns
        // `OperationNotSupported`, so nothing an integration test can write
        // constructs one: an earlier version of this file wrapped
        // `get_strategy` in `if let Ok(..)`, and that branch never executed.
        // Dead code shaped like coverage is worse than none, because it reads
        // as tested. Their coverage lives in-file, beside the private
        // constructor — see `tests_long_put_break_even` in `long_put.rs`.
    }

    /// The three strategies that carry a spot leg. Every per-share figure they
    /// report divides by the size of that leg, and their break-even is a cost
    /// basis net of a credit that the fees can turn into a debit.
    #[test]
    fn test_spot_leg_strategies_never_panic(
        // Every monetary argument of this test goes through the spot leg.
        // Until #471 these arguments ran through a bounded `spot_leg_money`
        // generator, because `LegAble::pnl_at_price`, `LegAble::total_cost`
        // and `LegAble::fees` returned a bare `Decimal` or `Positive` and
        // `Collar::net_premium` returned a bare `Decimal`: all four added and
        // multiplied with the raw operators and had nowhere to report an
        // overflow. All four now return `Result`, so the generator is the
        // unbounded `extreme_money` and `Positive::MAX` is back in range.
        underlying in extreme_money(),
        put_strike in extreme_positive(),
        call_strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        fee in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_money(),
    ) {
        // The legs `new` builds, for the rejected cases: the spot leg in
        // shares and the option legs in contracts of a hundred shares.
        let spot_leg = || {
            SpotPosition::new(
                "PROP".to_string(), quantity, underlying, Side::Long,
                chrono::Utc::now(), fee, fee,
            )
        };
        let leg = |style: OptionStyle, side: Side, strike: Positive, contracts: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, contracts, underlying, rate, style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };
        let contracts = quantity / Positive::HUNDRED;

        let mut strategy = match Collar::new(
            "PROP".to_string(), underlying, put_strike, call_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                Collar {
                    name: "Collar".to_string(),
                    kind: StrategyType::Collar,
                    description: String::new(),
                    break_even_points: Vec::new(),
                    spot_leg: spot_leg(),
                    long_put: leg(OptionStyle::Put, Side::Long, put_strike, contracts),
                    short_call: leg(OptionStyle::Call, Side::Short, call_strike, contracts),
                }
            }
        };
        exercise(&strategy, probe);
        let _ = strategy.collar_width();
        let _ = strategy.max_profit_potential();
        let _ = strategy.max_loss_potential();
        let _ = strategy.update_break_even_points();

        let mut strategy = match CoveredCall::new(
            "PROP".to_string(), underlying, call_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                CoveredCall {
                    name: "Covered Call".to_string(),
                    kind: StrategyType::CoveredCall,
                    description: String::new(),
                    break_even_points: Vec::new(),
                    spot_leg: spot_leg(),
                    short_call: leg(OptionStyle::Call, Side::Short, call_strike, contracts),
                }
            }
        };
        exercise(&strategy, probe);
        let _ = strategy.max_profit_potential();
        let _ = strategy.max_loss_potential();
        let _ = strategy.assignment_probability(probe);
        let _ = strategy.update_break_even_points();

        let mut strategy = match ProtectivePut::new(
            "PROP".to_string(), underlying, put_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                ProtectivePut {
                    name: "ProtectivePut_PROP".to_string(),
                    kind: StrategyType::ProtectivePut,
                    description: String::new(),
                    break_even_points: Vec::new(),
                    spot_leg: spot_leg(),
                    long_put: leg(OptionStyle::Put, Side::Long, put_strike, quantity),
                }
            }
        };
        exercise(&strategy, probe);
        let _ = strategy.max_loss_potential();
        let _ = strategy.update_break_even_points();
    }
}

proptest! {
    // A price walk is bounded by the number of samples it needs, so these
    // cases stay at magnitudes a chart can actually cross. The zero step and
    // the range past the sample cap are pinned as unit tests next to
    // `calculate_price_range` instead.
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// The custom strategy over an empty leg set, a single leg and four legs.
    /// Its range is centred on the spot and widened by the strike spread, so
    /// strikes further from the spot than the spot itself put the lower edge
    /// below zero.
    #[test]
    fn test_custom_strategy_never_panics(
        underlying in walkable_positive(),
        low_strike in walkable_positive(),
        high_strike in walkable_positive(),
        volatility in walkable_positive(),
        quantity in prop_oneof![Just(Positive::ONE), Just(Positive::HUNDRED)],
        premium in walkable_money(),
        expiration in extreme_expiration(),
        legs in 0usize..3,
    ) {
        let leg = |style: OptionStyle, side: Side, strike: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, dec!(0.05), style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), premium, premium, None, None,
            )
        };
        let positions = match legs {
            0 => Vec::new(),
            1 => vec![leg(OptionStyle::Call, Side::Long, low_strike)],
            _ => vec![
                leg(OptionStyle::Call, Side::Long, low_strike),
                leg(OptionStyle::Call, Side::Short, high_strike),
                leg(OptionStyle::Put, Side::Long, high_strike),
                leg(OptionStyle::Put, Side::Short, low_strike),
            ],
        };
        // `CustomStrategy` keeps its numeric settings private, so a leg set
        // that `new` rejects cannot be assembled around it; the rejection
        // itself is what is checked.
        match CustomStrategy::new(
            "prop".to_string(), "PROP".to_string(), "property".to_string(),
            underlying, positions, pos(dec!(0.01)), 100, Positive::ONE,
        ) {
            Ok(mut strategy) => {
                exercise(&strategy, underlying);
                let _ = strategy.get_best_range_to_show(Positive::HUNDRED);
                let _ = strategy.update_break_even_points();
            }
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
            }
        }
    }
}

/// Drives a multi-leg strategy twice: as its constructor left it, and again
/// with its break-even vector emptied.
///
/// The second pass is the point. `break_even_points` is a `pub` field on every
/// strategy in this crate and every strategy derives `Deserialize`, so a JSON
/// document carrying `"break_even_points": []`, or a struct literal written
/// downstream, reaches the profit area, the profit ratio and the two
/// probability ranges with nothing to read. Until #463 each of those indexed
/// the vector directly and aborted the process.
macro_rules! exercise_with_and_without_break_evens {
    ($strategy:expr, $probe:expr) => {{
        let mut strategy = $strategy;
        exercise(&strategy, $probe);
        let _ = strategy.update_break_even_points();
        exercise(&strategy, $probe);
        strategy.break_even_points = Vec::new();
        exercise(&strategy, $probe);
    }};
}

proptest! {
    // Fourteen strategies over eight generators is a wide product, so these
    // properties run fewer cases each than the single-leg ones above; the
    // magnitudes they draw from are the same.
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// The four vertical spreads and the poor man's covered call. Each carries
    /// a single break-even point computed as a strike offset by a
    /// per-contract premium, so a zero quantity, a premium above the strike
    /// and a strike at the top of the `Decimal` range all land in
    /// `update_break_even_points`; the readers of that point land in
    /// `get_profit_area` and in the probability ranges.
    #[test]
    fn test_vertical_spread_strategies_never_panic(
        underlying in extreme_positive(),
        low_strike in extreme_positive(),
        high_strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        fee in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
    ) {
        let leg = |style: OptionStyle, side: Side, strike: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, rate, style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };
        let (low, high) = (or_spot(low_strike, underlying), or_spot(high_strike, underlying));

        let strategy = match BearCallSpread::new(
            "PROP".to_string(), underlying, low_strike, high_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<BearCallSpread>(&[
                    leg(OptionStyle::Call, Side::Short, low),
                    leg(OptionStyle::Call, Side::Long, high),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match BullPutSpread::new(
            "PROP".to_string(), underlying, low_strike, high_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<BullPutSpread>(&[
                    leg(OptionStyle::Put, Side::Long, low),
                    leg(OptionStyle::Put, Side::Short, high),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match BearPutSpread::new(
            "PROP".to_string(), underlying, high_strike, low_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<BearPutSpread>(&[
                    leg(OptionStyle::Put, Side::Long, high),
                    leg(OptionStyle::Put, Side::Short, low),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match PoorMansCoveredCall::new(
            "PROP".to_string(), underlying, low_strike, high_strike, expiration,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<PoorMansCoveredCall>(&[
                    leg(OptionStyle::Call, Side::Long, low_strike),
                    leg(OptionStyle::Call, Side::Short, high_strike),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);
    }

    /// The straddles and the strangles. They carry two break-even points and
    /// measure widths between them, so a pair that coincides (a premium of
    /// zero) and a pair whose order the strikes invert both reach the
    /// subtraction and the division in `get_profit_area` and
    /// `get_profit_ratio`.
    #[test]
    fn test_straddle_and_strangle_strategies_never_panic(
        underlying in extreme_positive(),
        put_strike in extreme_positive(),
        call_strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        fee in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
    ) {
        let leg = |style: OptionStyle, side: Side, strike: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, rate, style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };
        let straddle_strike = or_spot(call_strike, underlying);

        let strategy = match LongStraddle::new(
            "PROP".to_string(), underlying, call_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, premium, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<LongStraddle>(&[
                    leg(OptionStyle::Call, Side::Long, straddle_strike),
                    leg(OptionStyle::Put, Side::Long, straddle_strike),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match ShortStraddle::new(
            "PROP".to_string(), underlying, call_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, premium, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<ShortStraddle>(&[
                    leg(OptionStyle::Call, Side::Short, straddle_strike),
                    leg(OptionStyle::Put, Side::Short, straddle_strike),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match LongStrangle::new(
            "PROP".to_string(), underlying, call_strike, put_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<LongStrangle>(&[
                    leg(OptionStyle::Call, Side::Long, call_strike),
                    leg(OptionStyle::Put, Side::Long, put_strike),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match ShortStrangle::new(
            "PROP".to_string(), underlying, call_strike, put_strike, expiration,
            volatility, volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<ShortStrangle>(&[
                    leg(OptionStyle::Call, Side::Short, call_strike),
                    leg(OptionStyle::Put, Side::Short, put_strike),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);
    }

    /// The four-leg structures and the butterflies. The butterflies push a
    /// break-even point only where a wing actually crosses zero profit, so
    /// one point, or none at all, is an ordinary outcome here rather than a
    /// broken constructor, and the readers of the second point have to say
    /// so instead of indexing past it.
    #[test]
    fn test_condor_and_butterfly_strategies_never_panic(
        underlying in extreme_positive(),
        low_strike in extreme_positive(),
        high_strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        fee in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
    ) {
        let middle_strike = low_strike.checked_add(&high_strike).unwrap_or(high_strike);
        let body = quantity.checked_add(&quantity).unwrap_or(quantity);
        let leg = |style: OptionStyle, side: Side, strike: Positive, quantity: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, rate, style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };

        let strategy = match IronCondor::new(
            "PROP".to_string(), underlying, high_strike, low_strike, high_strike, low_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, premium, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<IronCondor>(&[
                    leg(OptionStyle::Call, Side::Short, high_strike, quantity),
                    leg(OptionStyle::Put, Side::Short, low_strike, quantity),
                    leg(OptionStyle::Call, Side::Long, high_strike, quantity),
                    leg(OptionStyle::Put, Side::Long, low_strike, quantity),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match IronButterfly::new(
            "PROP".to_string(), underlying, middle_strike, high_strike, low_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, premium, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<IronButterfly>(&[
                    leg(OptionStyle::Call, Side::Short, middle_strike, quantity),
                    leg(OptionStyle::Put, Side::Short, middle_strike, quantity),
                    leg(OptionStyle::Call, Side::Long, high_strike, quantity),
                    leg(OptionStyle::Put, Side::Long, low_strike, quantity),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        // The body goes in first: `add_position` places each wing against it.
        let strategy = match LongButterflySpread::new(
            "PROP".to_string(), underlying, low_strike, middle_strike, high_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, fee, fee, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<LongButterflySpread>(&[
                    leg(OptionStyle::Call, Side::Short, middle_strike, body),
                    leg(OptionStyle::Call, Side::Long, low_strike, quantity),
                    leg(OptionStyle::Call, Side::Long, high_strike, quantity),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);

        let strategy = match ShortButterflySpread::new(
            "PROP".to_string(), underlying, low_strike, middle_strike, high_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, fee, fee, fee, fee, fee, fee,
        ) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<ShortButterflySpread>(&[
                    leg(OptionStyle::Call, Side::Long, middle_strike, body),
                    leg(OptionStyle::Call, Side::Short, low_strike, quantity),
                    leg(OptionStyle::Call, Side::Short, high_strike, quantity),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);
        if let Ok(strategy) = CallButterfly::new(
            "PROP".to_string(), underlying, low_strike, middle_strike, high_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, fee, fee, fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
    }

    /// The leg-set construction path of the two butterfly spreads. Until #463
    /// neither populated the break-even points, so a butterfly assembled from
    /// positions (the route a chain or a persisted strategy takes) reported
    /// an empty vector on a well-formed structure.
    #[test]
    fn test_butterfly_leg_set_construction_never_panics(
        underlying in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_volatility(),
        quantity in extreme_quantity(),
        premium in extreme_money(),
        rate in extreme_decimal(),
        expiration in extreme_expiration(),
        probe in extreme_positive(),
    ) {
        // The strikes have to be symmetric for `get_strategy` to accept them.
        let middle = strike.checked_add(&Positive::TEN).unwrap_or(strike);
        let high = middle.checked_add(&Positive::TEN).unwrap_or(middle);
        let body = quantity.checked_add(&quantity).unwrap_or(quantity);
        let leg = |side: Side, strike: Positive, quantity: Positive| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, rate, OptionStyle::Call,
                    Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), premium, premium, None, None,
            )
        };
        // A rejected leg set is assembled body first, so that `add_position`
        // places each wing against it, and driven all the same.
        let strategy = match LongButterflySpread::get_strategy(&[
            leg(Side::Long, strike, quantity),
            leg(Side::Short, middle, body),
            leg(Side::Long, high, quantity),
        ]) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<LongButterflySpread>(&[
                    leg(Side::Short, middle, body),
                    leg(Side::Long, strike, quantity),
                    leg(Side::Long, high, quantity),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);
        let strategy = match ShortButterflySpread::get_strategy(&[
            leg(Side::Short, strike, quantity),
            leg(Side::Long, middle, body),
            leg(Side::Short, high, quantity),
        ]) {
            Ok(strategy) => strategy,
            Err(error) => {
                prop_assert!(is_constructor_error(&error), "unexpected error: {error}");
                assembled::<ShortButterflySpread>(&[
                    leg(Side::Long, middle, body),
                    leg(Side::Short, strike, quantity),
                    leg(Side::Short, high, quantity),
                ])
            }
        };
        exercise_with_and_without_break_evens!(strategy, probe);
    }
}
