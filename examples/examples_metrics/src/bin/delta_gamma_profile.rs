/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/

//! # Delta-Gamma Profile Example
//!
//! This example demonstrates how to generate and visualize delta-gamma
//! profile curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/delta_gamma_curve.png`
//! - PNG image: `./Draws/Metrics/delta_gamma_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{DeltaGammaProfileCurve, DeltaGammaProfileSurface};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,
        spos!(5.0),
        dec!(-0.15),
        dec!(0.08),
        pos!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos!(450.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos!(0.20),
    );

    let option_chain = OptionChain::build_chain(&params);

    tracing::info!(
        "Built option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );

    // ========================================
    // Delta-Gamma Profile Curve
    // ========================================
    tracing::info!("Generating Delta-Gamma Profile Curve...");

    let dg_curve = option_chain.delta_gamma_curve()?;

    tracing::info!(
        "Delta-Gamma Curve generated with {} points",
        dg_curve.points.len()
    );

    // Find strike with maximum combined exposure
    if let Some(max_point) = dg_curve
        .points
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    {
        tracing::info!(
            "Maximum combined exposure: ${:.2} at strike {:.2}",
            max_point.y,
            max_point.x
        );
    }

    // Plot curve
    dg_curve
        .plot()
        .title("Delta-Gamma Profile Curve (SPY)")
        .x_label("Strike Price")
        .y_label("Dollar Delta + Dollar Gamma")
        .save("./Draws/Metrics/delta_gamma_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_curve
        .write_html("./Draws/Metrics/delta_gamma_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Delta-Gamma curve saved to ./Draws/Metrics/delta_gamma_curve.png");

    // ========================================
    // Delta-Gamma Profile Surface
    // ========================================
    tracing::info!("Generating Delta-Gamma Profile Surface...");

    let price_range = (pos!(380.0), pos!(520.0));
    let days = vec![
        pos!(7.0),
        pos!(14.0),
        pos!(21.0),
        pos!(30.0),
        pos!(45.0),
        pos!(60.0),
    ];

    let dg_surface = option_chain
        .delta_gamma_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Delta-Gamma Surface generated with {} points",
        dg_surface.points.len()
    );

    // Plot surface
    dg_surface
        .plot()
        .title("Delta-Gamma Profile Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Delta")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/delta_gamma_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_surface
        .write_html("./Draws/Metrics/delta_gamma_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Delta-Gamma surface saved to ./Draws/Metrics/delta_gamma_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
