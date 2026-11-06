//! The promises of a calibrated implied volatility surface, through the public API
//! alone: a synthetic known surface comes back within declared tolerances and keeps its
//! constraints, a quote later than the as-of time changes nothing, and a read beyond the
//! extrapolation policy is a typed error or an explicit flag.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use optionstratlib_core::model::decimal::{d_exp, d_sqrt};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::utils::time::years_to_expiration;
use optionstratlib_pricing::error::ImpliedVolatilitySurfaceError;
use optionstratlib_pricing::pricing::black_scholes_model::black_scholes;
use optionstratlib_pricing::volatility::{
    ExtrapolationPolicy, ImpliedVolatilitySurface, ImpliedVolatilitySurfaceConfig, OptionQuote,
    QuoteRejectionReason, SurfaceExpiryRegion, SviParameters, SviSlice, UnfittedReason,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

const RATE: Decimal = Decimal::from_parts(3, 0, 0, false, 2);
const YIELD: Decimal = Decimal::from_parts(1, 0, 0, false, 2);
const EXPIRIES: [i64; 5] = [30, 91, 182, 365, 730];

/// The declared recovery of implied volatility on noiseless quotes.
const VOLATILITY_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 8);
/// The declared recovery of the raw parameters on noiseless quotes.
const PARAMETER_TOLERANCE: Decimal = Decimal::from_parts(1, 0, 0, false, 6);

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap()
}

fn as_of() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 12, 0, 0).unwrap()
}

fn in_days(days: i64) -> DateTime<Utc> {
    as_of() + TimeDelta::try_days(days).unwrap()
}

fn years(days: i64) -> Positive {
    years_to_expiration(&ExpirationDate::Days(pos(Decimal::from(days)))).unwrap()
}

fn forward(days: i64) -> Positive {
    pos(dec!(100) * d_exp((RATE - YIELD) * years(days).to_dec(), "test").unwrap())
}

/// A synthetic surface in closed form: SSVI, by (3.3) of Gatheral and Jacquier with
/// `theta = 0.04 * T`, `eta = 1` and `rho = -0.4`, within Theorems 4.1 and 4.2.
fn known(days: i64) -> SviSlice {
    let t = years(days);
    let theta = dec!(0.04) * t.to_dec();
    let rho = dec!(-0.4);
    let phi = Decimal::ONE / d_sqrt(theta, "test").unwrap();
    let room = d_sqrt(Decimal::ONE - rho * rho, "test").unwrap();
    SviSlice::try_new(
        SviParameters::try_new(
            theta / dec!(2) * (Decimal::ONE - rho * rho),
            theta * phi / dec!(2),
            rho,
            -rho / phi,
            room / phi,
        )
        .unwrap(),
        forward(days),
        t,
    )
    .unwrap()
}

fn quote(days: i64, strike: Positive, style: OptionStyle, volatility: Positive) -> OptionQuote {
    let mid = black_scholes(&Options::new(
        OptionType::European,
        Side::Long,
        String::new(),
        strike,
        ExpirationDate::Days(pos(Decimal::from(days))),
        volatility,
        Positive::ONE,
        Positive::HUNDRED,
        RATE,
        style,
        pos(YIELD),
        None,
    ))
    .unwrap();
    OptionQuote {
        expiry: in_days(days),
        strike,
        style,
        bid: pos(mid * dec!(0.99)),
        ask: pos(mid * dec!(1.01)),
        quoted_at: as_of(),
    }
}

/// 15 strikes across two standard deviations of one expiry, each a quote of `style` at
/// the known volatility plus `noise`.
fn chain_with(
    days: i64,
    style: impl Fn(Positive) -> OptionStyle,
    noise: &mut impl FnMut() -> Decimal,
) -> Vec<OptionQuote> {
    let slice = known(days);
    let width = d_sqrt(years(days).to_dec(), "test").unwrap() * dec!(0.1);
    (-7_i32..=7)
        .map(|step| {
            let k = width * Decimal::from(step) / dec!(3.5);
            let strike = pos((forward(days).to_dec() * d_exp(k, "test").unwrap()).round_dp(2));
            let volatility = slice.implied_volatility_at_strike(strike).unwrap().to_dec();
            quote(days, strike, style(strike), pos(volatility + noise()))
        })
        .collect()
}

fn out_of_the_money(days: i64) -> impl Fn(Positive) -> OptionStyle {
    move |strike| {
        if strike >= forward(days) {
            OptionStyle::Call
        } else {
            OptionStyle::Put
        }
    }
}

fn quotes_of(expiries: &[i64]) -> Vec<OptionQuote> {
    expiries
        .iter()
        .flat_map(|days| chain_with(*days, out_of_the_money(*days), &mut || Decimal::ZERO))
        .collect()
}

fn calibrate_with(
    quotes: &[OptionQuote],
    config: &ImpliedVolatilitySurfaceConfig,
) -> Result<ImpliedVolatilitySurface, ImpliedVolatilitySurfaceError> {
    ImpliedVolatilitySurface::calibrate(
        quotes,
        Positive::HUNDRED,
        RATE,
        pos(YIELD),
        as_of(),
        config,
    )
}

fn calibrate(
    quotes: &[OptionQuote],
) -> Result<ImpliedVolatilitySurface, ImpliedVolatilitySurfaceError> {
    calibrate_with(quotes, &ImpliedVolatilitySurfaceConfig::default())
}

fn assert_parameters_close(found: SviParameters, expected: SviParameters) {
    for (name, found, expected) in [
        ("a", found.a(), expected.a()),
        ("b", found.b(), expected.b()),
        ("rho", found.rho(), expected.rho()),
        ("m", found.m(), expected.m()),
        ("sigma", found.sigma(), expected.sigma()),
    ] {
        assert!(
            (found - expected).abs() <= PARAMETER_TOLERANCE,
            "{name}: {found} against {expected}"
        );
    }
}

#[test]
fn test_known_surface_is_recovered_within_tolerance_and_keeps_its_constraints() {
    let surface = calibrate(&quotes_of(&EXPIRIES)).unwrap();
    assert!(
        surface.rejections().is_empty(),
        "{:?}",
        surface.rejections()
    );
    assert!(surface.unfitted().is_empty(), "{:?}", surface.unfitted());
    assert_eq!(surface.expiries().len(), EXPIRIES.len());

    for (expiry, days) in surface.expiries().iter().zip(EXPIRIES) {
        let truth = known(days);
        assert_eq!(expiry.expiry(), in_days(days));
        assert_eq!(expiry.time_to_expiry(), truth.time_to_expiry());
        assert_eq!(expiry.forward(), truth.forward());
        assert_parameters_close(expiry.slice().parameters(), truth.parameters());
        assert!(expiry.rmse().to_dec() <= VOLATILITY_TOLERANCE);
        for residual in expiry.residuals() {
            let expected = truth.implied_volatility(residual.log_moneyness).unwrap();
            let miss = residual.fitted_implied_volatility.to_dec() - expected.to_dec();
            assert!(miss.abs() <= VOLATILITY_TOLERANCE, "{residual:?}");
            let observed = residual.observed_implied_volatility.to_dec() - expected.to_dec();
            assert!(observed.abs() <= VOLATILITY_TOLERANCE, "{residual:?}");
        }
    }

    // The surface the theorems clear has nothing to report on this grid, and a calendar
    // check did run. This is not a claim that it is free of arbitrage elsewhere.
    assert!(
        surface.arbitrage_report().passed(),
        "{:?}",
        surface.arbitrage_report()
    );
    assert!(surface.calendar_checked());
}

#[test]
fn test_calls_and_puts_of_one_strike_agree_through_their_common_forward() {
    let days = 365;
    let quotes: Vec<OptionQuote> = [
        chain_with(days, |_| OptionStyle::Call, &mut || Decimal::ZERO),
        chain_with(days, |_| OptionStyle::Put, &mut || Decimal::ZERO),
    ]
    .concat();
    let surface = calibrate(&quotes).unwrap();
    assert!(
        surface.rejections().is_empty(),
        "{:?}",
        surface.rejections()
    );
    let expiry = surface.expiries().first().unwrap();
    assert_eq!(expiry.residuals().len(), 30);
    assert_parameters_close(expiry.slice().parameters(), known(days).parameters());
}

#[test]
fn test_quote_later_than_as_of_cannot_change_any_figure_of_the_surface() {
    let quotes = quotes_of(&[91, 365]);
    let base = calibrate(&quotes).unwrap();

    let later = as_of() + TimeDelta::nanoseconds(1);
    let mut mixed: Vec<OptionQuote> = quotes
        .iter()
        .map(|quote| OptionQuote {
            bid: pos(quote.bid.to_dec() * dec!(1.5)),
            ask: pos(quote.ask.to_dec() * dec!(1.5)),
            quoted_at: later,
            ..*quote
        })
        .collect();
    mixed.extend(quotes_of(&[182]).into_iter().map(|quote| OptionQuote {
        quoted_at: later,
        ..quote
    }));
    let count = mixed.len();
    mixed.extend(quotes.iter().copied());
    let with = calibrate(&mixed).unwrap();

    assert_eq!(with.expiries(), base.expiries());
    assert_eq!(with.unfitted(), base.unfitted());
    assert_eq!(with.arbitrage_report(), base.arbitrage_report());
    assert_eq!(with.arbitrage_grid(), base.arbitrage_grid());
    assert_eq!(with.rejections().len(), count);
    assert!(
        with.rejections()
            .iter()
            .all(|rejection| rejection.reason == QuoteRejectionReason::LaterThanAsOf)
    );
    let t = pos(dec!(0.5));
    assert_eq!(
        with.point(dec!(0.01), t).unwrap(),
        base.point(dec!(0.01), t).unwrap()
    );
}

#[test]
fn test_extrapolation_is_a_typed_error_by_default_and_a_flag_within_the_policy() {
    let quotes = quotes_of(&[91, 365]);
    let strict = calibrate(&quotes).unwrap();
    let (first, last) = (years(91), years(365));
    let beyond = strict.expiries().last().unwrap().highest_log_moneyness() + dec!(0.2);

    assert!(matches!(
        strict.point(beyond, last),
        Err(ImpliedVolatilitySurfaceError::WingOutOfPolicy { .. })
    ));
    assert!(matches!(
        strict.point(dec!(0), pos(first.to_dec() / dec!(2))),
        Err(ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy { .. })
    ));
    assert!(matches!(
        strict.point(dec!(0), pos(last.to_dec() * dec!(2))),
        Err(ImpliedVolatilitySurfaceError::LongExpiryOutOfPolicy { .. })
    ));

    let flagged = calibrate_with(
        &quotes,
        &ImpliedVolatilitySurfaceConfig {
            extrapolation: ExtrapolationPolicy {
                max_wing_extension: Positive::ONE,
                max_short_extension: Positive::ONE,
                max_long_extension: Positive::ONE,
            },
            ..ImpliedVolatilitySurfaceConfig::default()
        },
    )
    .unwrap();
    let wing = flagged.point(beyond, last).unwrap();
    assert!(wing.is_extrapolated());
    assert!(!wing.wing_extension.is_zero());
    let short = flagged
        .point(dec!(0), pos(first.to_dec() / dec!(2)))
        .unwrap();
    assert!(short.is_extrapolated());
    assert_eq!(short.expiry_region, SurfaceExpiryRegion::BeforeFirstExpiry);
    let long = flagged
        .point(dec!(0), pos(last.to_dec() * dec!(2)))
        .unwrap();
    assert!(long.is_extrapolated());
    assert_eq!(long.expiry_region, SurfaceExpiryRegion::AfterLastExpiry);
}

/// A deterministic standard-ish normal: a linear congruential generator, summed.
fn normal(seed: &mut u64) -> f64 {
    (0..12)
        .map(|_| {
            *seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            f64::from((*seed >> 33) as u32) / 2_147_483_648.0 - 0.5
        })
        .sum()
}

fn noisy_quotes(volatility_points: f64) -> Vec<OptionQuote> {
    let mut seed = 42;
    EXPIRIES
        .iter()
        .flat_map(|days| {
            chain_with(*days, out_of_the_money(*days), &mut || {
                Decimal::try_from(volatility_points * 0.01 * normal(&mut seed)).unwrap()
            })
        })
        .collect()
}

#[test]
fn test_noise_below_half_a_volatility_point_fits_every_expiry_near_the_truth() {
    let surface = calibrate(&noisy_quotes(0.25)).unwrap();
    assert_eq!(surface.expiries().len(), EXPIRIES.len());
    assert!(surface.unfitted().is_empty());
    for (expiry, days) in surface.expiries().iter().zip(EXPIRIES) {
        let truth = known(days);
        for residual in expiry.residuals() {
            let miss = residual.fitted_implied_volatility.to_dec()
                - truth
                    .implied_volatility(residual.log_moneyness)
                    .unwrap()
                    .to_dec();
            assert!(miss.abs() < dec!(0.01), "{residual:?}");
        }
    }
}

#[test]
fn test_noise_above_the_limit_loses_expiries_with_a_reason_and_a_loose_limit_keeps_them() {
    let quotes = noisy_quotes(0.8);
    let surface = calibrate(&quotes).unwrap();
    assert!(surface.expiries().len() < EXPIRIES.len());
    assert_eq!(
        surface.expiries().len() + surface.unfitted().len(),
        EXPIRIES.len()
    );
    for unfitted in surface.unfitted() {
        assert!(
            matches!(unfitted.reason, UnfittedReason::RmseAboveLimit { .. }),
            "{unfitted:?}"
        );
        assert_eq!(unfitted.clean_quotes, 15);
    }

    let mut loose = ImpliedVolatilitySurfaceConfig::default();
    loose.svi.max_relative_rmse = pos(dec!(0.1));
    let surface = calibrate_with(&quotes, &loose).unwrap();
    assert_eq!(surface.expiries().len(), EXPIRIES.len());
    assert!(surface.unfitted().is_empty());
}

#[test]
fn test_heavy_noise_leaves_no_surface_and_the_error_says_why_for_each_expiry() {
    match calibrate(&noisy_quotes(3.0)).unwrap_err() {
        ImpliedVolatilitySurfaceError::NoFittedExpiry {
            unfitted,
            rejections,
        } => {
            assert_eq!(unfitted.len(), EXPIRIES.len());
            assert!(rejections.is_empty(), "{rejections:?}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_shocked_surface_stays_a_surface_with_its_own_report() {
    use optionstratlib_pricing::volatility::SurfaceShock;
    let surface = calibrate(&quotes_of(&[91, 365])).unwrap();
    let up = surface
        .shock(SurfaceShock::Level {
            volatility: dec!(0.02),
        })
        .unwrap();
    let k = dec!(0);
    let t = years(365);
    let before = surface.point(k, t).unwrap().implied_volatility.to_dec();
    let after = up.point(k, t).unwrap().implied_volatility.to_dec();
    assert!((after - before - dec!(0.02)).abs() < VOLATILITY_TOLERANCE);
    assert_eq!(up.shocks().len(), 1);
    assert!(
        up.arbitrage_report().passed(),
        "{:?}",
        up.arbitrage_report()
    );
    let age = up
        .age_at(as_of() + TimeDelta::try_hours(1).unwrap())
        .unwrap();
    assert_eq!(age, TimeDelta::try_hours(1).unwrap());
}
