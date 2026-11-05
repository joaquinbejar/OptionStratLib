/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-11-05
******************************************************************************/

//! Static arbitrage checks example.
//!
//! Runs the butterfly and calendar checks of Gatheral and Jacquier
//! (arXiv:1204.0646) on a slice with a negative density, on two slices that
//! cross, on a surface of the class the paper clears, and on a slice whose total
//! variance reaches zero. A result names each finding rather than being one
//! boolean, and passing means only that these two checks found nothing at the
//! grid points supplied: the same slice can pass on a grid that steps over its
//! violation, which is shown here too.

use optionstratlib::error::StaticArbitrageError;
use optionstratlib::volatility::{
    StaticArbitrageViolation, SviParameters, SviSlice, check_butterfly, check_calendar,
    check_static_arbitrage,
};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

/// A log moneyness grid from `first` to `last`, inclusive.
fn grid(first: Decimal, last: Decimal, step: Decimal) -> Vec<Decimal> {
    let mut points = Vec::new();
    let mut k = first;
    while k <= last {
        points.push(k);
        k += step;
    }
    points
}

fn slice(
    (a, b, rho, m, sigma): (Decimal, Decimal, Decimal, Decimal, Decimal),
    forward: Decimal,
    time_to_expiry: Decimal,
) -> Result<SviSlice, Box<dyn std::error::Error>> {
    Ok(SviSlice::try_new(
        SviParameters::try_new(a, b, rho, m, sigma)?,
        Positive::new_decimal(forward)?,
        Positive::new_decimal(time_to_expiry)?,
    )?)
}

fn log_violation(violation: &StaticArbitrageViolation) {
    match violation {
        StaticArbitrageViolation::Butterfly {
            time_to_expiry,
            log_moneyness,
            strike,
            size,
            ..
        } => info!(
            expiry = %time_to_expiry,
            k = %log_moneyness,
            strike = %strike.to_dec().round_dp(2),
            size = %size.to_dec().round_dp(6),
            "butterfly: g(k) is negative, by this much"
        ),
        StaticArbitrageViolation::Calendar {
            earlier_expiry,
            later_expiry,
            log_moneyness,
            earlier_strike,
            later_strike,
            size,
            ..
        } => info!(
            earlier_expiry = %earlier_expiry,
            later_expiry = %later_expiry,
            k = %log_moneyness,
            earlier_strike = %earlier_strike.to_dec().round_dp(2),
            later_strike = %later_strike.to_dec().round_dp(2),
            size = %size.to_dec().round_dp(6),
            "calendar: total variance decreases, by this much"
        ),
        other => info!(?other, "another kind of violation"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    info!("=== Butterfly: Example 3.1 of the paper ===");
    let vogt = slice(
        (
            dec!(-0.0410),
            dec!(0.1331),
            dec!(0.3060),
            dec!(0.3586),
            dec!(0.4153),
        ),
        dec!(100),
        dec!(1),
    )?;
    let report = check_butterfly(&vogt, &grid(dec!(0.6), dec!(1.3), dec!(0.1)))?;
    for violation in report.violations() {
        log_violation(violation);
    }
    if report.passed() {
        return Err("the slice has a negative density on this grid".into());
    }

    info!("=== The same slice on a grid that steps over the violation ===");
    let coarse = [dec!(-1), dec!(0), dec!(0.5), dec!(1.5), dec!(2)];
    let report = check_butterfly(&vogt, &coarse)?;
    if !report.passed() {
        return Err(format!("expected this grid to step over it, got {report:?}").into());
    }
    info!(
        points = coarse.len(),
        "nothing found at these five points, which says nothing between them"
    );

    info!("=== Calendar: two slices that cross, on different forwards ===");
    let earlier = slice(
        (dec!(0.01), dec!(0.2), dec!(0), dec!(0), dec!(0.2)),
        dec!(100),
        dec!(0.5),
    )?;
    let later = slice(
        (dec!(0.05), dec!(0.1), dec!(0), dec!(0), dec!(0.2)),
        dec!(110),
        dec!(1),
    )?;
    let report = check_calendar(&[later, earlier], &grid(dec!(0), dec!(1), dec!(0.25)))?;
    for violation in report.violations() {
        log_violation(violation);
    }

    info!("=== A surface the theorems of the paper clear ===");
    // Raw parameters of SSVI with phi = 1 / sqrt(theta) and rho = -0.4, at
    // theta = 0.04, 0.09 and 0.16.
    let surface = [
        slice(
            (
                dec!(0.0168),
                dec!(0.1),
                dec!(-0.4),
                dec!(0.08),
                dec!(0.183303),
            ),
            dec!(100),
            dec!(0.25),
        )?,
        slice(
            (
                dec!(0.0378),
                dec!(0.15),
                dec!(-0.4),
                dec!(0.12),
                dec!(0.274955),
            ),
            dec!(102),
            dec!(0.5),
        )?,
        slice(
            (
                dec!(0.0672),
                dec!(0.2),
                dec!(-0.4),
                dec!(0.16),
                dec!(0.366606),
            ),
            dec!(105),
            dec!(1),
        )?,
    ];
    let points = grid(dec!(-2), dec!(2), dec!(0.05));
    let report = check_static_arbitrage(&surface, &points)?;
    if !report.passed() {
        return Err(format!("expected nothing to report, got {report:?}").into());
    }
    info!(
        points = points.len(),
        expiries = surface.len(),
        "nothing found at these points; the checks say no more than that"
    );

    info!("=== Total variance of zero: g has no value there ===");
    let touching = slice(
        (dec!(-0.01), dec!(0.1), dec!(0), dec!(0), dec!(0.1)),
        dec!(100),
        dec!(1),
    )?;
    let report = check_butterfly(&touching, &[dec!(0), dec!(0.1)])?;
    for point in report.unevaluated() {
        info!(
            k = %point.log_moneyness(),
            "unevaluated: neither a violation nor a pass"
        );
    }
    for violation in report.violations() {
        log_violation(violation);
    }

    info!("=== A check that cannot run is an error, not a pass ===");
    match check_butterfly(&vogt, &[]) {
        Err(error @ StaticArbitrageError::EmptyGrid) => info!(%error, "no grid"),
        other => return Err(format!("expected an empty grid, got {other:?}").into()),
    }
    match check_calendar(&[vogt], &[dec!(0)]) {
        Err(error @ StaticArbitrageError::TooFewSlices { .. }) => info!(%error, "one slice"),
        other => return Err(format!("expected too few slices, got {other:?}").into()),
    }
    match check_calendar(&[vogt, vogt], &[dec!(0)]) {
        Err(error @ StaticArbitrageError::DuplicateExpiry { .. }) => {
            info!(%error, "one expiry twice")
        }
        other => return Err(format!("expected a duplicate expiry, got {other:?}").into()),
    }

    Ok(())
}
