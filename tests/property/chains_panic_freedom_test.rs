//! Property-based tests for panic freedom across the chain internals, the
//! simulation kernels and the option series.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result` (or as the `None` an `Option`-returning
//! accessor already promises), including for inputs that are extreme but
//! structurally valid.
//!
//! This module drives two axes at once. The numeric axis is the usual
//! `Decimal` domain: values at the smallest representable scale, at
//! `Positive::MAX` and `Decimal::MIN`/`MAX`, rates at `±1e6`, and the ordinary
//! values in between. The structural axis is what makes a walk or a quote
//! degenerate rather than merely large: a walk of zero steps and of one step,
//! a zero time step, an expiration already at zero, a locked quote at zero, a
//! book whose two sides sum past the `Decimal` range, a series with a single
//! expiration and one with none. The assertion is deliberately weak: whatever
//! comes back, it must come back.

use optionstratlib::ExpirationDate;
use optionstratlib::chains::OptionData;
use optionstratlib::chains::utils::adjust_volatility;
use optionstratlib::error::{ChainError, SimulationError};
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{
    WalkParams, WalkType, WalkTypeAble, expanding_window_vols, generator_positive, walk_steps,
};
use optionstratlib::utils::TimeFrame;
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`. A ratio that rounds down to it — or
/// past it, to zero — is what turns an ordinary log-moneyness into a domain
/// error.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// A `Positive` from a `Decimal` literal that is positive by construction.
fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Prices, strikes and volatilities across the whole `Positive` range,
/// including the two ends that break the arithmetic.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.0001))),
        Just(pos(dec!(0.2))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(100000000000000))),
        Just(Positive::MAX),
    ]
}

/// Rates, drifts and greeks over the signed `Decimal` range.
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

/// A quote built from two extreme sides, so that `bid + ask` reaches the
/// overflow and `bid == ask == 0` reaches the zero mid.
fn extreme_quote() -> impl Strategy<Value = OptionData> {
    (
        extreme_positive(),
        extreme_positive(),
        extreme_decimal(),
        0usize..4,
    )
        .prop_map(|(bid, ask, gamma, shape)| {
            let mut data = OptionData {
                strike_price: Positive::HUNDRED,
                implied_volatility: pos(dec!(0.2)),
                gamma: Some(gamma),
                ..Default::default()
            };
            match shape {
                // A full two-sided book on both styles.
                0 => {
                    data.call_bid = Some(bid);
                    data.call_ask = Some(ask);
                    data.put_bid = Some(bid);
                    data.put_ask = Some(ask);
                }
                // Half a market: a bid with no ask.
                1 => {
                    data.call_bid = Some(bid);
                    data.put_bid = Some(bid);
                }
                // A mid with no book, which is the arm `apply_spread` widens
                // around rather than out from.
                2 => {
                    data.call_middle = Some(bid);
                    data.put_middle = Some(ask);
                }
                // A book and a mid that disagree.
                _ => {
                    data.call_bid = Some(bid);
                    data.call_ask = Some(ask);
                    data.call_middle = Some(ask);
                    data.put_bid = Some(bid);
                    data.put_ask = Some(ask);
                    data.put_middle = Some(bid);
                }
            }
            data
        })
}

/// A walker with no overrides, so every kernel under test is the built-in one.
#[derive(Clone)]
struct BareWalker;

impl<X, Y> WalkTypeAble<X, Y> for BareWalker
where
    X: Copy + TryInto<Positive> + std::ops::AddAssign + std::fmt::Display,
    Y: TryInto<Positive> + std::fmt::Display + Clone,
{
}

/// The stochastic processes whose kernels live in `simulation/traits.rs`,
/// each parameterised from the same numeric extremes. `dt = 0` collapses the
/// diffusion, `Positive::MAX` volatilities and `Decimal::MAX` drifts overflow
/// the step, and the GARCH arm carries a non-stationary `alpha + beta` that
/// overflows the very sum the stationarity check needs.
///
/// All eleven variants are covered, including the two backed by the
/// Ornstein-Uhlenbeck process and the historical one whose volatility is
/// estimated from the path rather than supplied.
fn extreme_walk_type() -> impl Strategy<Value = WalkType> {
    (
        extreme_positive(),
        extreme_positive(),
        extreme_decimal(),
        0usize..11,
    )
        .prop_map(|(dt, volatility, drift, kind)| match kind {
            0 => WalkType::Brownian {
                dt,
                drift,
                volatility,
            },
            1 => WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            },
            2 => WalkType::LogReturns {
                dt,
                expected_return: drift,
                volatility,
                autocorrelation: Some(dec!(0.5)),
            },
            3 => WalkType::JumpDiffusion {
                dt,
                drift,
                volatility,
                intensity: Positive::ONE,
                jump_mean: drift,
                jump_volatility: volatility,
            },
            4 => WalkType::Garch {
                dt,
                drift,
                volatility,
                alpha: pos(dec!(0.1)),
                beta: pos(dec!(0.8)),
            },
            // `alpha + beta` itself overflows before it can be compared to 1.
            5 => WalkType::Garch {
                dt,
                drift,
                volatility,
                alpha: Positive::MAX,
                beta: Positive::MAX,
            },
            6 => WalkType::Heston {
                dt,
                drift,
                volatility,
                kappa: Positive::MAX,
                theta: Positive::MAX,
                xi: Positive::MAX,
                rho: dec!(-0.5),
            },
            // Delegates to `volatility::utils::generate_ou_process`, whose
            // level is the state being simulated: a step that cannot be
            // represented ends the path.
            7 => WalkType::MeanReverting {
                dt,
                volatility,
                speed: Positive::ONE,
                mean: Positive::MAX,
            },
            8 => WalkType::Custom {
                dt,
                drift,
                volatility,
                vov: volatility,
                vol_speed: Positive::ONE,
                vol_mean: volatility,
            },
            9 => WalkType::Telegraph {
                dt,
                drift,
                volatility,
                lambda_up: Positive::MAX,
                lambda_down: Positive::MAX,
                vol_multiplier_up: Some(Positive::MAX),
                vol_multiplier_down: Some(Positive::MAX),
            },
            // Drives `utils::others::calculate_log_returns` through
            // `walk_volatility` and `expanding_window_vols`: a zero price has
            // no log return and a `1e-28` / `MAX` pair has no representable
            // ratio.
            _ => WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: vec![
                    Positive::HUNDRED,
                    Positive::MAX,
                    pos(TINY),
                    Positive::ZERO,
                    Positive::ONE,
                ],
                symbol: None,
            },
        })
}

/// Price paths that break the estimator structurally as well as
/// numerically: empty, too short to carry a sample variance, flat (so the
/// sample variance is exactly zero), a path through zero (whose log return
/// has no logarithm) and one spanning the whole `Positive` range (whose
/// consecutive ratio has no representable quotient).
fn degenerate_path() -> impl Strategy<Value = Vec<Positive>> {
    (extreme_positive(), extreme_positive(), 0usize..7).prop_map(
        |(value, other, shape)| match shape {
            0 => vec![],
            1 => vec![value],
            2 => vec![value, value],
            3 => vec![value; 5],
            4 => vec![value, other, value, other, value],
            5 => vec![Positive::ZERO, value, Positive::HUNDRED],
            _ => vec![pos(TINY), Positive::MAX, Positive::HUNDRED, Positive::ONE],
        },
    )
}

fn walk_params(
    size: usize,
    walk_type: WalkType,
    start: Positive,
    days: Positive,
) -> WalkParams<Positive, Positive> {
    WalkParams {
        size,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(days),
            start,
        ),
        walk_type,
        walker: Box::new(BareWalker),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every quote accessor returns for a book whose two sides sum past the
    /// `Decimal` range and for a locked quote at zero, whose mid is the
    /// divisor of the percentage spread.
    #[test]
    fn test_option_data_quote_accessors_never_panic(data in extreme_quote()) {
        let _ = data.get_call_spread();
        let _ = data.get_put_spread();
        let _ = data.get_call_spread_per();
        let _ = data.get_put_spread_per();
        let _ = data.get_mid_prices();
        let _ = format!("{data}");
        let mut data = data;
        data.set_mid_prices();
        let _ = data.get_mid_prices();
    }

    /// Widening returns for every spread and every scale, including scales
    /// `Decimal` cannot represent and a spread wider than the whole range.
    /// Applying it twice must also return: the pipeline applies it once, but
    /// nothing stops a caller from applying it again.
    #[test]
    fn test_apply_spread_never_panics(
        data in extreme_quote(),
        spread in extreme_positive(),
        decimal_places in prop_oneof![Just(0u32), Just(2), Just(4), Just(27), Just(28), Just(29), Just(u32::MAX)],
    ) {
        let mut data = data;
        data.apply_spread(spread, decimal_places);
        data.apply_spread(spread, decimal_places);
    }

    /// Pricing a row returns for every combination of underlying, strike,
    /// volatility, rate and time to expiry, including a zero underlying, a
    /// zero strike, a collapsed volatility and an expiration already reached.
    #[test]
    fn test_calculate_prices_never_panics(
        underlying in extreme_positive(),
        strike in extreme_positive(),
        volatility in extreme_positive(),
        days in extreme_positive(),
        rate in extreme_decimal(),
        spread in prop_oneof![Just(None), Just(Some(Positive::ZERO)), Just(Some(pos(dec!(0.02)))), Just(Some(Positive::MAX))],
    ) {
        let mut data = OptionData {
            strike_price: strike,
            implied_volatility: volatility,
            underlying_price: Some(Box::new(underlying)),
            expiration_date: Some(ExpirationDate::Days(days)),
            risk_free_rate: Some(rate),
            ..Default::default()
        };
        let _ = data.calculate_prices(spread);
        data.calculate_delta();
        data.calculate_gamma();
        data.calculate_greeks();
    }

    /// The skew/smile adjustment returns for a strike or an underlying at
    /// either end of the range, where the moneyness ratio underflows to zero
    /// and its logarithm leaves the domain.
    #[test]
    fn test_adjust_volatility_never_panics(
        base in extreme_positive(),
        skew in extreme_decimal(),
        smile in extreme_decimal(),
        strike in extreme_positive(),
        underlying in extreme_positive(),
    ) {
        let _ = adjust_volatility(&Some(base), &Some(skew), &Some(smile), &strike, &underlying);
        let _ = adjust_volatility(&None, &Some(skew), &Some(smile), &strike, &underlying);
    }

    /// Every walk kernel returns for every parameter extreme and for a walk
    /// of zero, one and two steps. Zero steps is the structural case: the
    /// Heston and Custom kernels formed `size - 1`, which underflows there.
    #[test]
    fn test_walk_kernels_never_panic(
        walk_type in extreme_walk_type(),
        size in prop_oneof![Just(0usize), Just(1), Just(2), Just(4)],
        start in extreme_positive(),
        days in extreme_positive(),
    ) {
        let params = walk_params(size, walk_type, start, days);
        let _ = params.walker.generate(&params);
        let _ = params.walker.generate_with_vol(&params);
    }

    /// The shared walk driver and the `Positive` generator return for the
    /// same matrix, so a kernel failure surfaces as an error through the
    /// driver rather than aborting inside it.
    #[test]
    fn test_walk_driver_never_panics(
        walk_type in extreme_walk_type(),
        size in prop_oneof![Just(0usize), Just(1), Just(2), Just(4)],
        start in extreme_positive(),
        days in extreme_positive(),
    ) {
        let params = walk_params(size, walk_type, start, days);
        let _: Result<Vec<Step<Positive, Positive>>, SimulationError> =
            walk_steps(&params, |price, _volatility, _x| Ok(Some(*price)));
        let _: Result<Vec<Step<Positive, Positive>>, ChainError> = generator_positive(&params);
    }

    /// Advancing and rewinding a time step returns for an expiration already
    /// at zero and for one at `Positive::MAX`, where rewinding adds days back
    /// past the representable range.
    #[test]
    fn test_x_step_advance_never_panics(
        days in extreme_positive(),
        step_size in extreme_positive(),
    ) {
        let step = Xstep::new(step_size, TimeFrame::Day, ExpirationDate::Days(days));
        let _ = step.days_left();
        let _ = step.next();
        let _ = step.previous();
        if let Ok(next) = step.next() {
            let _ = next.next();
            let _ = next.previous();
        }
        let _ = Ystep::new(0, days);
    }

    /// The expanding-window volatility estimator returns for a path too short
    /// to carry a sample variance, a flat path whose variance is exactly
    /// zero, and a path spanning the whole range.
    #[test]
    fn test_expanding_window_vols_never_panics(
        prices in degenerate_path(),
        timeframe in prop_oneof![
            Just(TimeFrame::Microsecond),
            Just(TimeFrame::Day),
            Just(TimeFrame::Year),
        ],
    ) {
        let _ = expanding_window_vols(&prices, timeframe);
    }
}
