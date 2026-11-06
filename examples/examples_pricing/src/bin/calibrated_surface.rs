/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-11-06
******************************************************************************/

//! Calibrated implied volatility surface example.
//!
//! Builds a surface from bid and ask quotes across strikes and expiries as of one
//! instant. The quotes come from a known SSVI surface, with a few bad ones among them
//! and one expiry shaped like a frown, which a raw SVI slice cannot follow: with these even
//! spreads the limit on its RMSE refuses it. The example reads what was refused and why, the
//! residuals, the figures that ignore the weights, the comparison with a flat volatility per
//! expiry, the arbitrage report, values inside and outside the data, three shocks and an
//! export. It reads no clock: every time is measured from the as-of time. The
//! report says only what its checks found on its grid, and the surface is not claimed
//! free of arbitrage.

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use optionstratlib::error::ImpliedVolatilitySurfaceError;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::black_scholes_model::black_scholes;
use optionstratlib::volatility::{
    ExtrapolationPolicy, ImpliedVolatilitySurface, ImpliedVolatilitySurfaceConfig, OptionQuote,
    SurfaceShock, SviParameters, SviSlice,
};
use optionstratlib::{ExpirationDate, Options};
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use tracing::info;

/// The continuously compounded rate per year.
const RATE: Decimal = dec!(0.03);
/// The continuous dividend yield per year.
const YIELD: Decimal = dec!(0.01);
/// The spot at the as-of time.
const SPOT: Decimal = dec!(100);

/// The known slice of an expiry: SSVI with `theta = 0.04 * T`, `eta = 1`, `rho = -0.4`.
fn known_slice(days: i64) -> Result<SviSlice, Box<dyn std::error::Error>> {
    let t = Decimal::from(days) / dec!(365);
    let theta = dec!(0.04) * t;
    let rho = dec!(-0.4);
    let phi = Decimal::ONE / theta.sqrt().ok_or("theta has a root")?;
    let room = (Decimal::ONE - rho * rho)
        .sqrt()
        .ok_or("rho is below one")?;
    Ok(SviSlice::try_new(
        SviParameters::try_new(
            theta / dec!(2) * (Decimal::ONE - rho * rho),
            theta * phi / dec!(2),
            rho,
            -rho / phi,
            room / phi,
        )?,
        Positive::new_decimal(SPOT * ((RATE - YIELD) * t).exp())?,
        Positive::new_decimal(t)?,
    )?)
}

/// A quote 1% either side of the Black-Scholes price at `volatility`.
fn quote(
    as_of: DateTime<Utc>,
    days: i64,
    strike: Decimal,
    volatility: Positive,
) -> Result<OptionQuote, Box<dyn std::error::Error>> {
    let strike = Positive::new_decimal(strike)?;
    let style = OptionStyle::Call;
    let mid = black_scholes(&Options::new(
        OptionType::European,
        Side::Long,
        String::new(),
        strike,
        ExpirationDate::Days(Positive::new_decimal(Decimal::from(days))?),
        volatility,
        Positive::ONE,
        Positive::new_decimal(SPOT)?,
        RATE,
        style,
        Positive::new_decimal(YIELD)?,
        None,
    ))?;
    Ok(OptionQuote {
        expiry: as_of + TimeDelta::try_days(days).ok_or("days fit")?,
        strike,
        style,
        bid: Positive::new_decimal(mid * dec!(0.99))?,
        ask: Positive::new_decimal(mid * dec!(1.01))?,
        quoted_at: as_of,
    })
}

/// The quotes of one expiry of the known surface.
fn chain(as_of: DateTime<Utc>, days: i64) -> Result<Vec<OptionQuote>, Box<dyn std::error::Error>> {
    let slice = known_slice(days)?;
    let forward = slice.forward().to_dec();
    let width = (Decimal::from(days) / dec!(365))
        .sqrt()
        .ok_or("time has a root")?
        * dec!(0.1);
    let mut quotes = Vec::new();
    for step in -5_i32..=5 {
        let k = width * Decimal::from(step) / dec!(2.5);
        let strike = (forward * k.exp()).round_dp(2);
        let volatility = slice.implied_volatility_at_strike(Positive::new_decimal(strike)?)?;
        quotes.push(quote(as_of, days, strike, volatility)?);
    }
    Ok(quotes)
}

/// An expiry whose implied volatility falls away from the money, a frown.
fn frown(as_of: DateTime<Utc>, days: i64) -> Result<Vec<OptionQuote>, Box<dyn std::error::Error>> {
    let mut quotes = Vec::new();
    for step in -4_i32..=4 {
        let volatility = dec!(0.5) - dec!(0.05) * Decimal::from(step.abs());
        let strike = SPOT + Decimal::from(step) * dec!(2);
        quotes.push(quote(
            as_of,
            days,
            strike,
            Positive::new_decimal(volatility)?,
        )?);
    }
    Ok(quotes)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    let as_of = Utc
        .with_ymd_and_hms(2030, 1, 1, 12, 0, 0)
        .single()
        .ok_or("a valid date")?;
    let spot = Positive::new_decimal(SPOT)?;
    let dividend_yield = Positive::new_decimal(YIELD)?;

    let mut quotes = chain(as_of, 30)?;
    quotes.extend(chain(as_of, 91)?);
    quotes.extend(chain(as_of, 365)?);
    quotes.extend(frown(as_of, 182)?);

    // A quote from after the as-of time, a crossed one and a stale one.
    let good = quotes.first().copied().ok_or("the chain has quotes")?;
    quotes.push(OptionQuote {
        quoted_at: as_of + TimeDelta::try_seconds(1).ok_or("a second fits")?,
        ..good
    });
    quotes.push(OptionQuote {
        strike: Positive::new_decimal(good.strike.to_dec() + dec!(0.25))?,
        bid: good.ask,
        ask: good.bid,
        ..good
    });
    quotes.push(OptionQuote {
        strike: Positive::new_decimal(good.strike.to_dec() + dec!(0.5))?,
        quoted_at: as_of - TimeDelta::try_hours(1).ok_or("an hour fits")?,
        ..good
    });

    let config = ImpliedVolatilitySurfaceConfig {
        extrapolation: ExtrapolationPolicy {
            max_wing_extension: Positive::new_decimal(dec!(0.2))?,
            max_short_extension: Positive::ZERO,
            max_long_extension: Positive::ZERO,
        },
        ..ImpliedVolatilitySurfaceConfig::default()
    };
    let surface =
        ImpliedVolatilitySurface::calibrate(&quotes, spot, RATE, dividend_yield, as_of, &config)?;

    info!("=== What did not enter, and why ===");
    for rejection in surface.rejections() {
        info!(
            strike = %rejection.quote.strike,
            reason = ?rejection.reason,
            "quote rejected"
        );
    }
    for unfitted in surface.unfitted() {
        info!(
            expiry = %unfitted.expiry,
            clean_quotes = unfitted.clean_quotes,
            reason = ?unfitted.reason,
            "expiry has no slice"
        );
    }

    info!("=== Each fitted expiry against a flat volatility ===");
    for expiry in surface.expiries() {
        let baseline = expiry.baseline();
        info!(
            time_to_expiry = %expiry.time_to_expiry().to_dec().round_dp(4),
            forward = %expiry.forward().to_dec().round_dp(4),
            rmse = %expiry.rmse().to_dec().round_dp(8),
            flat_volatility = %baseline.baseline_implied_volatility.to_dec().round_dp(4),
            flat_rmse = %baseline.baseline_rmse.to_dec().round_dp(6),
            unweighted_rmse = %expiry.unweighted_rmse().to_dec().round_dp(8),
            quotes_outside_bid_ask = expiry.quotes_outside_bid_ask(),
            "in sample, five parameters against one"
        );
    }
    for residual in surface.residuals().take(3) {
        info!(
            strike = %residual.strike,
            observed = %residual.observed_implied_volatility.to_dec().round_dp(4),
            fitted = %residual.fitted_implied_volatility.to_dec().round_dp(4),
            weight = %residual.weight.to_dec().round_dp(4),
            "residual"
        );
    }

    info!("=== The arbitrage report: only these checks, on this grid ===");
    let report = surface.arbitrage_report();
    info!(
        grid_points = surface.arbitrage_grid().len(),
        calendar_checked = surface.calendar_checked(),
        violations = report.violations().len(),
        unevaluated = report.unevaluated().len(),
        passed = report.passed(),
        "report"
    );

    info!("=== Reading values ===");
    let expiry = as_of + TimeDelta::try_days(200).ok_or("days fit")?;
    let inside = surface.point_at_strike(spot, expiry)?;
    info!(
        k = %inside.log_moneyness.round_dp(4),
        implied_volatility = %inside.implied_volatility.to_dec().round_dp(4),
        bridges_unfitted_expiry = inside.bridges_unfitted_expiry,
        extrapolated = inside.is_extrapolated(),
        "at the money spot, 200 days, between 91 and 365 where 182 has no slice"
    );
    let last = surface.expiries().last().ok_or("a surface has an expiry")?;
    let wing = surface.point(
        last.highest_log_moneyness() + dec!(0.1),
        last.time_to_expiry(),
    )?;
    info!(
        wing_extension = %wing.wing_extension.to_dec().round_dp(4),
        extrapolated = wing.is_extrapolated(),
        "within the policy: a flag"
    );
    match surface.point(dec!(5), last.time_to_expiry()) {
        Err(error @ ImpliedVolatilitySurfaceError::WingOutOfPolicy { .. }) => {
            info!(%error, "beyond the policy: a typed error");
        }
        other => return Err(format!("expected a wing error, got {other:?}").into()),
    }
    let tomorrow = as_of + TimeDelta::try_days(1).ok_or("a day fits")?;
    match surface.point_at_strike(spot, tomorrow) {
        Err(error @ ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy { .. }) => {
            info!(%error, "before the first expiry: a typed error");
        }
        other => return Err(format!("expected a short expiry error, got {other:?}").into()),
    }
    info!(age = %surface.age_at(tomorrow)?, "the surface a day later");

    info!("=== Shocks: each with its own report ===");
    for shock in [
        SurfaceShock::Level {
            volatility: dec!(0.01),
        },
        SurfaceShock::Skew { rho: dec!(-0.05) },
        SurfaceShock::Term {
            slope: dec!(-0.5),
            pivot: Positive::ONE,
        },
    ] {
        let shocked = surface.shock(shock)?;
        let report = shocked.arbitrage_report();
        info!(
            ?shock,
            at_the_money = %shocked.point(dec!(0), last.time_to_expiry())?.implied_volatility.to_dec().round_dp(4),
            violations = report.violations().len(),
            "shocked"
        );
    }
    match surface.shock(SurfaceShock::Skew { rho: dec!(2) }) {
        Err(error @ ImpliedVolatilitySurfaceError::InvalidShockedParameters { .. }) => {
            info!(%error, "a shock past the parameter bounds: a typed error");
        }
        other => return Err(format!("expected invalid parameters, got {other:?}").into()),
    }

    info!("=== Export to a generic surface ===");
    let export = surface.export_surface(
        &[dec!(-0.05), dec!(0), dec!(0.05)],
        &[
            surface
                .expiries()
                .first()
                .ok_or("a surface has an expiry")?
                .time_to_expiry(),
            inside.time_to_expiry,
            last.time_to_expiry(),
        ],
    )?;
    info!(
        points = export.surface.points.len(),
        extrapolated = export.extrapolated.len(),
        bridging_unfitted = export.bridging_unfitted.len(),
        "x is log moneyness, y is years, z is implied volatility"
    );
    Ok(())
}
