/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Theta Metrics Example
//!
//! This example demonstrates how to generate and visualize theta (time decay)
//! curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/theta_curve.png`
//! - PNG image: `./Draws/Metrics/theta_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::ThetaSurface;
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
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(450.0))),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos_or_panic!(0.20),
    );

    let option_chain = OptionChain::build_chain(&params);

    tracing::info!(
        "Built option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );

    // ========================================
    // Theta Curve (by Strike)
    // ========================================
    tracing::info!("Generating Theta Curve...");

    let theta_curve = option_chain.theta_curve()?;

    tracing::info!(
        "Theta Curve generated with {} points",
        theta_curve.points.len()
    );

    // Find strike with most negative theta
    let points: Vec<_> = theta_curve.points.iter().collect();
    if let Some(min_theta) = points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "Maximum theta decay: {:.4} at strike {}",
            min_theta.y,
            min_theta.x
        );
    }

    // Plot curve
    theta_curve
        .plot()
        .title("Theta Curve - Time Decay (SPY)")
        .x_label("Strike Price")
        .y_label("Theta (Daily Decay)")
        .save("./Draws/Metrics/theta_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    theta_curve
        .write_html("./Draws/Metrics/theta_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Theta curve saved to ./Draws/Metrics/theta_curve.png");

    // ========================================
    // Theta Surface (Price vs Time)
    // ========================================
    tracing::info!("Generating Theta Surface...");

    let price_range = (pos_or_panic!(380.0), pos_or_panic!(520.0));
    let days = vec![
        pos_or_panic!(1.0),
        pos_or_panic!(3.0),
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
    ];

    let theta_surface = option_chain
        .theta_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Theta Surface generated with {} points",
        theta_surface.points.len()
    );

    // Plot surface
    theta_surface
        .plot()
        .title("Theta Surface - Time Decay (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Theta")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/theta_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    theta_surface
        .write_html("./Draws/Metrics/theta_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Theta surface saved to ./Draws/Metrics/theta_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
