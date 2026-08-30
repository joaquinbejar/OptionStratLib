//! Property-based tests for panic freedom across the composition layer: the
//! strategy trait defaults, the single-leg and spot-leg strategies, the custom
//! strategy, the probability kernels, the SPAN margin model and the P&L
//! primitives.
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

use optionstratlib::ExpirationDate;
use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{Action, OptionStyle, OptionType, Side};
use optionstratlib::model::{Options, Position};
use optionstratlib::pnl::{PnL, PnLCalculator};
use optionstratlib::pricing::Profit;
use optionstratlib::risk::SPANMargin;
use optionstratlib::strategies::base::{
    BasicAble, BreakEvenable, Positionable, Strategies, Validable,
};
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::probabilities::{
    PriceTrend, ProbabilityAnalysis, VolatilityAdjustment, calculate_price_probability,
    calculate_single_point_probability,
};
use optionstratlib::strategies::{
    BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, Collar,
    CoveredCall, IronButterfly, IronCondor, LongButterflySpread, LongCall, LongStraddle,
    LongStrangle, PoorMansCoveredCall, ProtectivePut, ShortButterflySpread, ShortPut,
    ShortStraddle, ShortStrangle, StrategyConstructor,
};
use optionstratlib::visualization::Graph;
use positive::Positive;
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
        if let Ok(mut strategy) = BullCallSpread::new(
            "PROP".to_string(), underlying, long_strike, short_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise(&strategy, probe);
            let _ = strategy.update_break_even_points();
            let _ = strategy.get_volume();
            let _ = strategy.set_underlying_price(&underlying);
            let _ = strategy.set_implied_volatility(&volatility);
            let _ = strategy.apply_delta_adjustments(Some(Action::Buy));
        }
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
        if let Ok(mut strategy) = LongCall::new(
            "PROP".to_string(), strike, expiration, volatility, quantity,
            underlying, rate, Positive::ZERO, premium, fee, fee,
        ) {
            exercise(&strategy, probe);
            let _ = strategy.update_break_even_points();
        }
        if let Ok(mut strategy) = ShortPut::new(
            "PROP".to_string(), strike, expiration, volatility, quantity,
            underlying, rate, Positive::ZERO, premium, fee, fee,
        ) {
            exercise(&strategy, probe);
            let _ = strategy.update_break_even_points();
        }

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
        if let Ok(mut strategy) = Collar::new(
            "PROP".to_string(), underlying, put_strike, call_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee, fee, fee,
        ) {
            exercise(&strategy, probe);
            let _ = strategy.collar_width();
            let _ = strategy.max_profit_potential();
            let _ = strategy.max_loss_potential();
            let _ = strategy.update_break_even_points();
        }
        if let Ok(mut strategy) = CoveredCall::new(
            "PROP".to_string(), underlying, call_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, fee, fee, fee, fee,
        ) {
            exercise(&strategy, probe);
            let _ = strategy.max_profit_potential();
            let _ = strategy.max_loss_potential();
            let _ = strategy.assignment_probability(probe);
            let _ = strategy.update_break_even_points();
        }
        if let Ok(mut strategy) = ProtectivePut::new(
            "PROP".to_string(), underlying, put_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, fee, fee, fee, fee,
        ) {
            exercise(&strategy, probe);
            let _ = strategy.max_loss_potential();
            let _ = strategy.update_break_even_points();
        }
    }

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
        adjust in any::<bool>(),
        drift in prop_oneof![Just(0.0f64), Just(1e30f64), Just(-1e30f64)],
        confidence in prop_oneof![Just(0.0f64), Just(0.8f64), Just(2.0f64)],
    ) {
        let volatility_adj = adjust.then_some(VolatilityAdjustment {
            base_volatility,
            std_dev_adjustment,
        });
        let trend = Some(PriceTrend { drift_rate: drift, confidence });
        let _ = calculate_single_point_probability(
            &current, &target, volatility_adj.clone(), trend.clone(), &expiration, Some(rate),
        );
        let _ = calculate_price_probability(
            &current, &target, &upper, volatility_adj, trend, &expiration, Some(rate),
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

proptest! {
    // A price walk is bounded by the number of samples it needs, so these
    // cases stay at magnitudes a chart can actually cross. The zero step and
    // the range past the sample cap are pinned as unit tests next to
    // `calculate_price_range` instead.
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Every call that walks a price range: the display range, the payoff
    /// chart, the expected value integration and the full probability
    /// analysis, plus the custom strategy whose break-even scan walks a cent
    /// at a time.
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
        if let Ok(strategy) = BullCallSpread::new(
            "PROP".to_string(), underlying, long_strike, short_strike, expiration,
            volatility, dec!(0.05), Positive::ZERO, quantity, premium, premium,
            Positive::ZERO, Positive::ZERO, Positive::ZERO, Positive::ZERO,
        ) {
            let _ = strategy.get_best_range_to_show(step);
            let _ = strategy.graph_data();
            let _ = strategy.expected_value(None, None);
            let _ = strategy.analyze_probabilities(None, None);
        }
    }

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
        if let Ok(mut strategy) = CustomStrategy::new(
            "prop".to_string(), "PROP".to_string(), "property".to_string(),
            underlying, positions, pos(dec!(0.01)), 100, Positive::ONE,
        ) {
            exercise(&strategy, underlying);
            let _ = strategy.get_best_range_to_show(Positive::HUNDRED);
            let _ = strategy.update_break_even_points();
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
        if let Ok(strategy) = BearCallSpread::new(
            "PROP".to_string(), underlying, low_strike, high_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = BullPutSpread::new(
            "PROP".to_string(), underlying, low_strike, high_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = BearPutSpread::new(
            "PROP".to_string(), underlying, high_strike, low_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = PoorMansCoveredCall::new(
            "PROP".to_string(), underlying, low_strike, high_strike, expiration,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
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
        if let Ok(strategy) = LongStraddle::new(
            "PROP".to_string(), underlying, call_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, premium, fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = ShortStraddle::new(
            "PROP".to_string(), underlying, call_strike, expiration, volatility,
            rate, Positive::ZERO, quantity, premium, premium, fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = LongStrangle::new(
            "PROP".to_string(), underlying, call_strike, put_strike, expiration,
            volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = ShortStrangle::new(
            "PROP".to_string(), underlying, call_strike, put_strike, expiration,
            volatility, volatility, rate, Positive::ZERO, quantity, premium, premium,
            fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
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
        if let Ok(strategy) = IronCondor::new(
            "PROP".to_string(), underlying, high_strike, low_strike, high_strike, low_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, premium, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = IronButterfly::new(
            "PROP".to_string(), underlying, middle_strike, high_strike, low_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, premium, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = LongButterflySpread::new(
            "PROP".to_string(), underlying, low_strike, middle_strike, high_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, fee, fee, fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = ShortButterflySpread::new(
            "PROP".to_string(), underlying, low_strike, middle_strike, high_strike,
            expiration, volatility, rate, Positive::ZERO, quantity, premium, premium,
            premium, fee, fee, fee, fee, fee, fee,
        ) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
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
        if let Ok(strategy) = LongButterflySpread::get_strategy(&[
            leg(Side::Long, strike, quantity),
            leg(Side::Short, middle, body),
            leg(Side::Long, high, quantity),
        ]) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
        if let Ok(strategy) = ShortButterflySpread::get_strategy(&[
            leg(Side::Short, strike, quantity),
            leg(Side::Long, middle, body),
            leg(Side::Short, high, quantity),
        ]) {
            exercise_with_and_without_break_evens!(strategy, probe);
        }
    }
}
