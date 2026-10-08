//! Property-based tests for panic freedom across the chain metrics, the chain
//! range and position helpers, and the walk buffers (#788).
//!
//! Each property drives a public entry point with the inputs that used to
//! abort it: a zero or `1e20` spot, a book whose quotes are crossed, locked
//! at zero or summed past the `Decimal` range, a grid whose upper bound sits
//! below its lower bound, a volatility band wider than the spot, quantities
//! and walk sizes at `usize::MAX`, and a step index at `i32::MAX`. As in the
//! sibling modules the assertion is deliberately weak: whatever comes back,
//! it must come back.

use optionstratlib::ExpirationDate;
use optionstratlib::OptionStyle;
use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{RandomPositionsParams, calculate_optimal_price_range};
use optionstratlib::metrics::{
    BidAskSpreadCurve, CharmSurface, ColorSurface, DeltaGammaProfileCurve,
    DeltaGammaProfileSurface, DollarGammaCurve, ImpliedVolatilitySurface, PriceShockCurve,
    PutCallRatioCurve, SmileDynamicsSurface, StrikeConcentrationCurve, ThetaSurface,
    TimeDecaySurface, VannaVolgaSurface, VolatilitySensitivitySurface, VolatilitySkewCurve,
    VolumeProfileSurface,
};
use optionstratlib::simulation::steps::{Step, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::TimeFrame;
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`, the day count that turns
/// `30 / day` into an overflow.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// A `Positive` from a `Decimal` literal that is positive by construction.
fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Prices, quotes, volumes and volatilities across the whole `Positive`
/// range, including the two ends that break the arithmetic.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.2))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(100000000000000000000))),
        Just(Positive::MAX),
    ]
}

/// Shock percentages and other signed inputs over the `Decimal` range.
fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(dec!(0.05)),
        Just(dec!(-0.05)),
        Just(dec!(10000000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

/// Counts at both ends of `usize`: the sums and reservations that used to
/// overflow or abort, and the small counts that must keep working.
fn extreme_count() -> impl Strategy<Value = usize> {
    prop_oneof![
        Just(0usize),
        Just(1),
        Just(2),
        Just(usize::MAX / 2),
        Just(usize::MAX),
    ]
}

/// A chain of up to three strikes whose spot, quotes, volatilities and
/// volumes are drawn from the extremes. Equal bids and asks give a locked
/// book, a bid above the ask a crossed one.
fn extreme_chain() -> impl Strategy<Value = OptionChain> {
    (
        extreme_positive(),
        prop::collection::vec(
            (
                extreme_positive(),
                extreme_positive(),
                extreme_positive(),
                extreme_positive(),
                extreme_positive(),
            ),
            0..4,
        ),
    )
        .prop_map(|(spot, rows)| {
            let mut chain = OptionChain::new("PROP", spot, "2030-01-01".to_string(), None, None);
            for (strike, bid, ask, iv, volume) in rows {
                chain.add_option(
                    strike,
                    Some(bid),
                    Some(ask),
                    Some(bid),
                    Some(ask),
                    iv,
                    None,
                    None,
                    None,
                    Some(volume),
                    Some(10),
                    None,
                );
            }
            chain
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

/// One of each walk kernel family that allocates its path from `size`.
fn walk_type(kind: usize) -> WalkType {
    let dt = pos(dec!(0.004));
    let volatility = pos(dec!(0.2));
    match kind {
        0 => WalkType::Brownian {
            dt,
            drift: Decimal::ZERO,
            volatility,
        },
        1 => WalkType::GeometricBrownian {
            dt,
            drift: Decimal::ZERO,
            volatility,
        },
        2 => WalkType::MeanReverting {
            dt,
            volatility,
            speed: Positive::ONE,
            mean: Positive::HUNDRED,
        },
        3 => WalkType::Garch {
            dt,
            drift: Decimal::ZERO,
            volatility,
            alpha: pos(dec!(0.1)),
            beta: pos(dec!(0.8)),
        },
        4 => WalkType::Heston {
            dt,
            drift: Decimal::ZERO,
            volatility,
            kappa: Positive::ONE,
            theta: volatility,
            xi: volatility,
            rho: Decimal::ZERO,
        },
        _ => WalkType::Telegraph {
            dt,
            drift: Decimal::ZERO,
            volatility,
            lambda_up: Positive::ONE,
            lambda_down: Positive::ONE,
            vol_multiplier_up: None,
            vol_multiplier_down: None,
        },
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Every curve metric returns for a zero spot (no moneyness), a `1e20`
    /// spot (whose square leaves the range), a crossed book, a book locked
    /// at zero (no average premium) and one whose sides sum past the range.
    #[test]
    fn test_chain_curve_metrics_never_panic(
        chain in extreme_chain(),
        shock in extreme_decimal(),
    ) {
        let _ = chain.volatility_skew();
        let _ = chain.premium_weighted_pcr();
        let _ = chain.premium_concentration();
        let _ = chain.dollar_gamma_curve(&OptionStyle::Call);
        let _ = chain.dollar_gamma_curve(&OptionStyle::Put);
        let _ = chain.delta_gamma_curve();
        let _ = chain.bid_ask_spread_curve();
        let _ = chain.price_shock_curve(shock);
    }

    /// Every grid surface returns for reversed price and volatility ranges,
    /// ranges spanning the whole `Positive` domain, and day counts at both
    /// ends.
    #[test]
    fn test_chain_surface_metrics_never_panic(
        chain in extreme_chain(),
        price_range in (extreme_positive(), extreme_positive()),
        vol_range in (extreme_positive(), extreme_positive()),
        steps in 0usize..3,
        day in extreme_positive(),
    ) {
        let days = vec![day];
        let _ = chain.iv_surface(days.clone());
        let _ = chain.vanna_volga_surface(price_range, vol_range, steps, steps);
        let _ = chain.delta_gamma_surface(price_range, days.clone(), steps);
        let _ = chain.smile_dynamics_surface(days.clone());
        let _ = chain.volume_profile_surface(days.clone());
        let _ = chain.volatility_sensitivity_surface(price_range, vol_range, steps, steps);
        let _ = chain.time_decay_surface(price_range, days.clone(), steps);
        let _ = chain.theta_surface(price_range, days.clone(), steps);
        let _ = chain.charm_surface(price_range, days.clone(), steps);
        let _ = chain.color_surface(price_range, days, steps);
    }

    /// The display range returns for a band wider than the spot, a zero spot
    /// and strike, and spots or volatilities at the top of the range.
    #[test]
    fn test_calculate_optimal_price_range_never_panics(
        spot in extreme_positive(),
        strike in extreme_positive(),
        iv in extreme_positive(),
        days in prop_oneof![Just(Positive::ONE), Just(pos(dec!(365))), Just(pos(dec!(36500)))],
    ) {
        let _ = calculate_optimal_price_range(spot, strike, iv, ExpirationDate::Days(days));
    }

    /// Position quantities whose sum overflows `usize`, or whose positions
    /// cannot be reserved, come back as errors.
    #[test]
    fn test_random_positions_never_panic(
        puts in extreme_count(),
        calls in extreme_count(),
    ) {
        let params = RandomPositionsParams::new(
            Some(puts),
            None,
            Some(calls),
            None,
            ExpirationDate::Days(pos(dec!(30))),
            Positive::ONE,
            dec!(0.05),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );
        // Every total drawn here is either at most four, or at least
        // `usize::MAX / 2` positions, which no allocator can reserve, or an
        // overflow: none of them builds a large number of positions.
        let _ = params.total_positions();
        let mut chain = OptionChain::new("PROP", Positive::HUNDRED, "2030-01-01".to_string(), None, None);
        chain.add_option(
            Positive::HUNDRED,
            Some(Positive::ONE),
            Some(Positive::TWO),
            Some(Positive::ONE),
            Some(Positive::TWO),
            pos(dec!(0.2)),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let _ = chain.get_random_positions(params);
    }

    /// Walk kernels reserve their path from `size`: `usize::MAX` overflows
    /// the `size + 1` points and `usize::MAX / 2` cannot be reserved.
    #[test]
    fn test_walk_buffers_never_panic(
        size in extreme_count(),
        kind in 0usize..6,
        seed in any::<u64>(),
    ) {
        let params = WalkParams::<Positive, Positive> {
            size,
            init_step: Step::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(Positive::HUNDRED),
                Positive::HUNDRED,
            ),
            walk_type: walk_type(kind),
            walker: Box::new(BareWalker),
            seed: Some(seed),
        };
        let _ = BareWalker.generate(&params);
    }

    /// Advancing a step index at either end of `i32` returns.
    #[test]
    fn test_y_step_next_never_panics(index in prop_oneof![Just(i32::MIN), Just(0), Just(i32::MAX - 1), Just(i32::MAX)]) {
        let _ = Ystep::new(index, Positive::ONE).next(Positive::ONE);
    }
}
