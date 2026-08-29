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
    BullCallSpread, Collar, CoveredCall, LongCall, LongPut, ProtectivePut, ShortCall, ShortPut,
    StrategyConstructor,
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

/// Premia and fees, bounded below the magnitude at which a *single* leg's own
/// `Position::total_cost` overflows.
///
/// `(premium + open_fee + close_fee) * quantity` is computed with the raw
/// `Positive` operators in `src/model/position.rs`, which this sweep does not
/// touch; that overflow is reported separately. Everything above this bound
/// aborts before the composition layer is reached, so driving it here would
/// only re-find the model-layer defect.
fn extreme_money() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.01))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(100000000000000))),
    ]
}

/// Contract and share counts, bounded for the same reason as
/// [`extreme_money`]. Zero is kept: it is the divisor of every per-contract
/// and per-share figure in this layer.
fn extreme_quantity() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(1000000))),
    ]
}

/// Volatilities, bounded below the magnitude at which
/// `model::utils::mean_and_std` overflows.
///
/// Every `get_profit_ranges` implementation averages the leg volatilities
/// through that helper, which sums them with the raw `Positive` operator and
/// has no error channel (it returns `(Positive, Positive)`). That overflow
/// lives in `src/model`, which this sweep does not touch, and is reported
/// separately.
fn extreme_volatility() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.2))),
        Just(Positive::ONE),
        Just(pos(dec!(1000000))),
        Just(pos(dec!(1000000000000000))),
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

/// Expirations from one already reached, through a sub-second sliver, to a
/// horizon whose `DateTime` arithmetic is itself at the edge.
fn extreme_expiration() -> impl Strategy<Value = ExpirationDate> {
    prop_oneof![
        Just(ExpirationDate::Days(Positive::ZERO)),
        Just(ExpirationDate::Days(pos(TINY))),
        Just(ExpirationDate::Days(Positive::ONE)),
        Just(ExpirationDate::Days(pos(dec!(30)))),
        Just(ExpirationDate::Days(pos(dec!(3650)))),
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
    // `Greeks::greeks` is deliberately absent. Its aggregation loop in
    // `greeks::equations` adds the per-option `alpha`, and `alpha_from`
    // returns `Decimal::MAX` as a sentinel when theta vanishes, so two legs at
    // a sub-contract quantity abort on `Decimal::MAX + Decimal::MAX`. That is
    // in `src/greeks`, outside this sweep, and is reported separately.
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

        // `ShortCall::new` and `LongPut::new` are private; the reachable path
        // is the constructor that recognises a leg pattern.
        let leg = |style: OptionStyle, side: Side| {
            Position::new(
                Options::new(
                    OptionType::European, side, "PROP".to_string(), strike, expiration,
                    volatility, quantity, underlying, rate, style, Positive::ZERO, None,
                ),
                premium, chrono::Utc::now(), fee, fee, None, None,
            )
        };
        if let Ok(mut strategy) =
            <ShortCall as StrategyConstructor>::get_strategy(&[leg(OptionStyle::Call, Side::Short)])
        {
            exercise(&strategy, probe);
            let _ = strategy.update_break_even_points();
        }
        if let Ok(mut strategy) =
            <LongPut as StrategyConstructor>::get_strategy(&[leg(OptionStyle::Put, Side::Long)])
        {
            exercise(&strategy, probe);
            let _ = strategy.update_break_even_points();
        }
    }

    /// The three strategies that carry a spot leg. Every per-share figure they
    /// report divides by the size of that leg, and their break-even is a cost
    /// basis net of a credit that the fees can turn into a debit.
    #[test]
    fn test_spot_leg_strategies_never_panic(
        // The spot price and the probe price are bounded here for the same
        // reason as `extreme_money`: `SpotPosition::pnl_at_price` multiplies
        // their difference by the share count with the raw operator, in
        // `src/model/leg/spot.rs`.
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
