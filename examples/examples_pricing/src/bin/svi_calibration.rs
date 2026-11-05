/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-11-05
******************************************************************************/

//! Raw SVI calibration example.
//!
//! Fits a raw SVI slice (Gatheral and Jacquier, arXiv:1204.0646) to weighted
//! implied volatility quotes of one expiry, reads the smile back by strike and
//! by log moneyness, and shows the typed errors a caller handles: too few
//! quotes, and a smile SVI cannot follow, which is rejected instead of being
//! returned flat. A calibrated slice is not claimed free of arbitrage.

use optionstratlib::error::SviError;
use optionstratlib::volatility::{SviCalibrationConfig, SviObservation, calibrate_svi};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

/// A quote of `(strike, implied volatility, weight)`.
type Quote = (Decimal, Decimal, Decimal);

/// One expiry of an equity index: a put skew with noisy quotes, and less
/// weight on the illiquid wings.
const QUOTES: [Quote; 11] = [
    (dec!(3000), dec!(0.4312), dec!(0.2)),
    (dec!(3500), dec!(0.3574), dec!(0.5)),
    (dec!(4000), dec!(0.2861), dec!(1)),
    (dec!(4300), dec!(0.2441), dec!(1)),
    (dec!(4600), dec!(0.2041), dec!(2)),
    (dec!(4800), dec!(0.1803), dec!(2)),
    (dec!(5000), dec!(0.1597), dec!(2)),
    (dec!(5200), dec!(0.1442), dec!(2)),
    (dec!(5500), dec!(0.1331), dec!(1)),
    (dec!(6000), dec!(0.1352), dec!(0.5)),
    (dec!(6500), dec!(0.1502), dec!(0.2)),
];

fn observations(quotes: &[Quote]) -> Result<Vec<SviObservation>, Box<dyn std::error::Error>> {
    quotes
        .iter()
        .map(|&(strike, implied_volatility, weight)| {
            Ok(SviObservation {
                strike: Positive::new_decimal(strike)?,
                implied_volatility: Positive::new_decimal(implied_volatility)?,
                weight: Positive::new_decimal(weight)?,
            })
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    let forward = Positive::new_decimal(dec!(5000))?;
    let time_to_expiry = Positive::new_decimal(dec!(0.5))?;
    let config = SviCalibrationConfig::default();

    info!("=== Weighted calibration of one expiry ===");
    let fit = calibrate_svi(&observations(&QUOTES)?, forward, time_to_expiry, &config)?;
    let parameters = fit.slice().parameters();
    info!(
        a = %parameters.a().round_dp(6),
        b = %parameters.b().round_dp(6),
        rho = %parameters.rho().round_dp(6),
        m = %parameters.m().round_dp(6),
        sigma = %parameters.sigma().round_dp(6),
        "fitted parameters"
    );
    info!(
        rmse = %fit.rmse().to_dec().round_dp(6),
        iterations = fit.iterations(),
        "weighted RMSE in implied volatility, and Nelder-Mead iterations"
    );

    info!("=== Residuals, fitted minus observed ===");
    for residual in fit.residuals() {
        info!(
            strike = %residual.strike,
            k = %residual.log_moneyness.round_dp(4),
            observed = %residual.observed_implied_volatility,
            fitted = %residual.fitted_implied_volatility.to_dec().round_dp(4),
            residual = %residual.residual.round_dp(5),
            "quote"
        );
    }

    info!("=== Reading the smile back ===");
    let slice = fit.slice();
    let strike = Positive::new_decimal(dec!(4500))?;
    info!(
        k = %slice.log_moneyness(strike)?.round_dp(4),
        total_variance = %slice.total_variance_at_strike(strike)?.to_dec().round_dp(6),
        implied_volatility = %slice.implied_volatility_at_strike(strike)?.to_dec().round_dp(4),
        "strike 4500"
    );
    info!(
        implied_volatility = %slice.implied_volatility(dec!(0))?.to_dec().round_dp(4),
        "at the money forward"
    );

    info!("=== Typed failures ===");
    let too_few = observations(QUOTES.get(..3).ok_or("the table has three quotes")?)?;
    match calibrate_svi(&too_few, forward, time_to_expiry, &config) {
        Err(error @ SviError::TooFewObservations { .. }) => info!(%error, "too few quotes"),
        other => return Err(format!("expected too few observations, got {other:?}").into()),
    }

    // Alternating levels, which no slice can follow. The closest slice is
    // nearly flat, and it is rejected rather than handed back as a result.
    let zigzag: Vec<Quote> = QUOTES
        .iter()
        .enumerate()
        .map(|(index, &(strike, _, _))| {
            let level = if index % 2 == 0 {
                dec!(0.15)
            } else {
                dec!(0.45)
            };
            (strike, level, dec!(1))
        })
        .collect();
    match calibrate_svi(&observations(&zigzag)?, forward, time_to_expiry, &config) {
        Err(error @ SviError::RmseAboveLimit { .. }) => info!(%error, "unfittable smile"),
        other => return Err(format!("expected a rejected fit, got {other:?}").into()),
    }

    Ok(())
}
