//! Property-based tests for panic freedom and the promises of a calibrated surface.
//!
//! The library is embedded in long-running services, where a panic kills the worker
//! thread, so every failure must come back as a typed error or a rejected quote,
//! including for inputs that are extreme but structurally valid: `Decimal` at the edges
//! of its range, instants at the edges of `chrono`, and every malformed quote at once.
//!
//! A surface that comes back must also keep its promises: every quote is accounted for,
//! a quote later than the as-of time changes nothing, and a read says when it left the
//! data.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_pricing::error::ImpliedVolatilitySurfaceError;
use optionstratlib_pricing::volatility::{
    ExtrapolationPolicy, ImpliedVolatilitySurface, ImpliedVolatilitySurfaceConfig, OptionQuote,
    QuoteRejection, QuoteRejectionReason, SurfaceExpiryRegion, SurfaceShock,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable positive `Decimal`.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

fn positive(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

fn as_of() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 12, 0, 0).unwrap()
}

fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(dec!(0.00000000000001)),
        Just(dec!(0.2)),
        Just(dec!(66)),
        Just(dec!(1000000)),
        Just(Decimal::MAX),
    ]
}

/// A price or a strike: mostly ordinary, at times extreme.
fn level() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        4 => (1_i64..30_000).prop_map(|cents| Decimal::new(cents, 2)),
        1 => extreme_decimal(),
    ]
}

fn expiry() -> impl Strategy<Value = DateTime<Utc>> {
    prop_oneof![
        6 => prop_oneof![Just(30_i64), Just(91), Just(365)]
            .prop_map(|days| as_of() + TimeDelta::try_days(days).unwrap_or(TimeDelta::zero())),
        1 => (1_i64..100_000).prop_map(|seconds| {
            as_of() + TimeDelta::try_seconds(seconds).unwrap_or(TimeDelta::zero())
        }),
        1 => Just(as_of()),
        1 => Just(as_of() - TimeDelta::nanoseconds(1)),
        1 => Just(as_of() + TimeDelta::nanoseconds(1)),
        1 => Just(DateTime::<Utc>::MAX_UTC),
        1 => Just(DateTime::<Utc>::MIN_UTC),
    ]
}

fn quoted_at() -> impl Strategy<Value = DateTime<Utc>> {
    prop_oneof![
        6 => Just(as_of()),
        1 => Just(as_of() + TimeDelta::nanoseconds(1)),
        1 => Just(as_of() - TimeDelta::try_seconds(301).unwrap_or(TimeDelta::zero())),
        1 => Just(DateTime::<Utc>::MIN_UTC),
        1 => Just(DateTime::<Utc>::MAX_UTC),
    ]
}

fn style() -> impl Strategy<Value = OptionStyle> {
    prop_oneof![Just(OptionStyle::Call), Just(OptionStyle::Put)]
}

/// Quotes that are often clean enough to reach the solver and the fit, and often not.
fn quote() -> impl Strategy<Value = OptionQuote> {
    (expiry(), level(), style(), level(), level(), quoted_at()).prop_map(
        |(expiry, strike, style, bid, ask, quoted_at)| OptionQuote {
            expiry,
            strike: positive(strike),
            style,
            bid: positive(bid),
            ask: positive(ask),
            quoted_at,
        },
    )
}

/// Quotes that fit one expiry, to reach everything after the fit: a parabola of total
/// variance with random curvature and skew, priced by the crude at-the-money form so that
/// the volatilities are neither far from a slice nor exact.
fn fittable(days: i64) -> impl Strategy<Value = Vec<OptionQuote>> {
    (5_u32..40, 1_u32..30).prop_map(move |(level, width)| {
        (-6_i32..=6)
            .map(|step| {
                let strike = dec!(100) + Decimal::from(step * 3);
                let intrinsic = (dec!(100) - strike).max(Decimal::ZERO);
                let time = Decimal::from(u32::try_from(step.abs()).unwrap_or(0));
                let value = Decimal::from(level) * dec!(0.1)
                    + time * time * Decimal::from(width) * dec!(0.001);
                let mid = intrinsic + value;
                OptionQuote {
                    expiry: as_of() + TimeDelta::try_days(days).unwrap_or(TimeDelta::zero()),
                    strike: positive(strike),
                    style: OptionStyle::Call,
                    bid: positive(mid * dec!(0.99)),
                    ask: positive(mid * dec!(1.01)),
                    quoted_at: as_of(),
                }
            })
            .collect()
    })
}

fn quotes() -> impl Strategy<Value = Vec<OptionQuote>> {
    prop_oneof![
        prop::collection::vec(quote(), 0..24),
        (
            fittable(91),
            fittable(365),
            prop::collection::vec(quote(), 0..8)
        )
            .prop_map(|(mut first, second, noise)| {
                first.extend(second);
                first.extend(noise);
                first
            }),
    ]
}

fn market_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        6 => (-20_i64..20).prop_map(|cents| Decimal::new(cents, 2)),
        1 => Just(dec!(1000000)),
        1 => Just(Decimal::MAX),
        1 => Just(Decimal::MIN),
    ]
}

fn spot() -> impl Strategy<Value = Positive> {
    prop_oneof![
        8 => Just(Positive::HUNDRED),
        1 => extreme_decimal().prop_map(positive),
    ]
}

fn dividend() -> impl Strategy<Value = Positive> {
    prop_oneof![
        8 => Just(Positive::ZERO),
        1 => extreme_decimal().prop_map(positive),
    ]
}

fn shock() -> impl Strategy<Value = SurfaceShock> {
    prop_oneof![
        extreme_decimal().prop_map(|volatility| SurfaceShock::Level { volatility }),
        extreme_decimal().prop_map(|volatility| SurfaceShock::Level {
            volatility: -volatility
        }),
        extreme_decimal().prop_map(|rho| SurfaceShock::Skew { rho }),
        extreme_decimal().prop_map(|rho| SurfaceShock::Skew { rho: -rho }),
        (extreme_decimal(), extreme_decimal()).prop_map(|(slope, pivot)| SurfaceShock::Term {
            slope,
            pivot: positive(pivot),
        }),
        (extreme_decimal(), extreme_decimal()).prop_map(|(slope, pivot)| SurfaceShock::Term {
            slope: -slope,
            pivot: positive(pivot),
        }),
    ]
}

fn config() -> ImpliedVolatilitySurfaceConfig {
    ImpliedVolatilitySurfaceConfig {
        extrapolation: ExtrapolationPolicy {
            max_wing_extension: Positive::ONE,
            max_short_extension: Positive::ONE,
            max_long_extension: Positive::ONE,
        },
        ..ImpliedVolatilitySurfaceConfig::default()
    }
}

/// The rejections that are not for being later than the as-of time.
fn without_later(rejections: &[QuoteRejection]) -> Vec<QuoteRejection> {
    rejections
        .iter()
        .filter(|rejection| rejection.reason != QuoteRejectionReason::LaterThanAsOf)
        .cloned()
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Every quote comes back as a residual, a rejection, or a clean quote of an unfitted
    /// expiry, and every read, shock and export returns.
    #[test]
    fn prop_calibrate_never_panics_and_accounts_for_every_quote(
        quotes in quotes(),
        spot in spot(),
        rate in market_decimal(),
        dividend in dividend(),
        shock in shock(),
        k in extreme_decimal(),
        time in extreme_decimal(),
    ) {
        match ImpliedVolatilitySurface::calibrate(&quotes, spot, rate, dividend, as_of(), &config()) {
            Ok(surface) => {
                let clean_unfitted: usize =
                    surface.unfitted().iter().map(|expiry| expiry.clean_quotes).sum();
                prop_assert_eq!(
                    surface.residuals().count() + surface.rejections().len() + clean_unfitted,
                    quotes.len()
                );
                prop_assert!(!surface.expiries().is_empty());

                let time = positive(time);
                let first = surface.expiries().first().map(|expiry| expiry.time_to_expiry());
                let last = surface.expiries().last().map(|expiry| expiry.time_to_expiry());
                for k in [k, -k, dec!(0)] {
                    if let Ok(point) = surface.point(k, time) {
                        prop_assert_eq!(point.log_moneyness, k);
                        // Within the policy, and where the expiries say it is. The times read
                        // here are never within the resolution of a fitted one.
                        prop_assert!(point.wing_extension <= config().extrapolation.max_wing_extension);
                        let region = match (first, last) {
                            (Some(first), _) if time < first => SurfaceExpiryRegion::BeforeFirstExpiry,
                            (_, Some(last)) if time > last => SurfaceExpiryRegion::AfterLastExpiry,
                            _ => SurfaceExpiryRegion::WithinExpiries,
                        };
                        prop_assert_eq!(point.expiry_region, region);
                    }
                }
                // The derivatives read the same place: their value is the point's exactly, and
                // where the point is an error so are they.
                for k in [k, -k, dec!(0)] {
                    for time in surface
                        .expiries()
                        .iter()
                        .map(|expiry| expiry.time_to_expiry())
                        .chain([time])
                    {
                        match (surface.point(k, time), surface.point_with_derivatives(k, time)) {
                            (Ok(point), Ok(read)) => prop_assert_eq!(point, read.point),
                            (Err(point), Err(read)) => {
                                prop_assert_eq!(point.to_string(), read.to_string());
                            }
                            // A derivative can leave the range where the value does not.
                            (Ok(_), Err(_)) => {}
                            (Err(error), Ok(read)) => {
                                prop_assert!(false, "{error} against {read:?}");
                            }
                        }
                    }
                }
                let expiry = as_of() + TimeDelta::nanoseconds(1);
                let _ = surface.point_at_strike_with_derivatives(Positive::HUNDRED, expiry);
                // At a fitted expiry the read is that slice, and inside the expiries.
                for expiry in surface.expiries() {
                    if let Ok(point) = surface.point(k, expiry.time_to_expiry()) {
                        prop_assert_eq!(point.expiry_region, SurfaceExpiryRegion::WithinExpiries);
                        prop_assert!(!point.bridges_unfitted_expiry);
                    }
                }
                let _ = surface.point_at_strike(Positive::HUNDRED, as_of() + TimeDelta::nanoseconds(1));
                let _ = surface.export_surface(&[k], &[time]);
                let _ = surface.check_on_grid(&[k]);
                let _ = surface.age_at(as_of());
                if let Ok(shocked) = surface.shock(shock) {
                    let _ = shocked.point_with_derivatives(k, time);
                    prop_assert_eq!(shocked.shocks(), [shock]);
                    prop_assert_eq!(shocked.rejections(), surface.rejections());
                    prop_assert_eq!(shocked.residuals().count(), surface.residuals().count());
                }
            }
            Err(ImpliedVolatilitySurfaceError::NoFittedExpiry { unfitted, rejections }) => {
                let clean: usize = unfitted.iter().map(|expiry| expiry.clean_quotes).sum();
                prop_assert_eq!(clean + rejections.len(), quotes.len());
            }
            Err(_) => {}
        }
    }

    /// The other quotes are the same, so the result is the same but for the list of what
    /// was refused, and that list grows by exactly the later quotes.
    #[test]
    fn prop_later_quotes_change_nothing_but_the_list_that_names_them(
        quotes in quotes(),
        later in prop::collection::vec(quote(), 0..12),
    ) {
        let on_time: Vec<OptionQuote> =
            quotes.into_iter().filter(|quote| quote.quoted_at <= as_of()).collect();
        let later: Vec<OptionQuote> = later
            .into_iter()
            .map(|quote| OptionQuote { quoted_at: as_of() + TimeDelta::nanoseconds(1), ..quote })
            .collect();
        let mixed = [later.clone(), on_time.clone()].concat();
        let default = ImpliedVolatilitySurfaceConfig::default();
        let calibrate = |quotes: &[OptionQuote]| {
            ImpliedVolatilitySurface::calibrate(
                quotes, Positive::HUNDRED, dec!(0.02), Positive::ZERO, as_of(), &default,
            )
        };
        match (calibrate(&on_time), calibrate(&mixed)) {
            (Ok(base), Ok(with)) => {
                prop_assert_eq!(base.expiries(), with.expiries());
                prop_assert_eq!(base.unfitted(), with.unfitted());
                prop_assert_eq!(base.arbitrage_report(), with.arbitrage_report());
                prop_assert_eq!(base.arbitrage_grid(), with.arbitrage_grid());
                prop_assert_eq!(base.rejections().len() + later.len(), with.rejections().len());
                let later_rejections = with
                    .rejections()
                    .iter()
                    .filter(|rejection| rejection.reason == QuoteRejectionReason::LaterThanAsOf)
                    .count();
                prop_assert_eq!(later_rejections, later.len());
                prop_assert_eq!(without_later(with.rejections()), base.rejections());
            }
            // The same failure, for the same reasons, once the later quotes are set aside.
            (
                Err(ImpliedVolatilitySurfaceError::NoFittedExpiry {
                    unfitted: base_unfitted,
                    rejections: base_rejections,
                }),
                Err(ImpliedVolatilitySurfaceError::NoFittedExpiry {
                    unfitted: with_unfitted,
                    rejections: with_rejections,
                }),
            ) => {
                prop_assert_eq!(base_unfitted, with_unfitted);
                prop_assert_eq!(without_later(&with_rejections), base_rejections);
            }
            (Err(base), Err(with)) => prop_assert_eq!(base.to_string(), with.to_string()),
            (base, with) => prop_assert!(false, "{:?} against {:?}", base.err(), with.err()),
        }
    }
}
