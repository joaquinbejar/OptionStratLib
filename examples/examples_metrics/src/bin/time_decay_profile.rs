/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Time Decay Profile Example
//!
//! This example demonstrates how to generate and visualize time decay
//! profile curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/time_decay_curve.png`
//! - PNG image: `./Draws/Metrics/time_decay_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{TimeDecayCurve, TimeDecaySurface};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use positive::pos_or_panic;
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
    // Time Decay Curve (Theta by Strike)
    // ========================================
    tracing::info!("Generating Time Decay Curve (Theta)...");

    let theta_curve = option_chain.time_decay_curve()?;

    tracing::info!(
        "Theta Curve generated with {} points",
        theta_curve.points.len()
    );

    // Find strike with most negative theta (highest decay)
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
        .title("Time Decay Curve - Theta (SPY)")
        .x_label("Strike Price")
        .y_label("Theta (Daily Decay)")
        .save("./Draws/Metrics/time_decay_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    theta_curve
        .write_html("./Draws/Metrics/time_decay_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Theta curve saved to ./Draws/Metrics/time_decay_curve.png");

    // ========================================
    // Time Decay Surface (Price vs Time)
    // ========================================
    tracing::info!("Generating Time Decay Surface...");

    let price_range = (pos_or_panic!(380.0), pos_or_panic!(520.0));
    let days = vec![
        Positive::ONE,
        pos_or_panic!(3.0),
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
    ];

    let decay_surface = option_chain
        .time_decay_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Time Decay Surface generated with {} points",
        decay_surface.points.len()
    );

    // Plot surface
    decay_surface
        .plot()
        .title("Time Decay Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Option Value")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/time_decay_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    decay_surface
        .write_html("./Draws/Metrics/time_decay_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Time decay surface saved to ./Draws/Metrics/time_decay_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
