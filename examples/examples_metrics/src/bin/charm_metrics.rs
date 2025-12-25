/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Charm Metrics Example
//!
//! This example demonstrates how to generate and visualize charm (delta decay)
//! curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/charm_curve.png`
//! - PNG image: `./Draws/Metrics/charm_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::CharmSurface;
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
    // Charm Curve (by Strike)
    // ========================================
    tracing::info!("Generating Charm Curve (Delta Decay)...");

    let charm_curve = option_chain.charm_curve()?;

    tracing::info!(
        "Charm Curve generated with {} points",
        charm_curve.points.len()
    );

    // Plot curve
    charm_curve
        .plot()
        .title("Charm Curve - Delta Decay (SPY)")
        .x_label("Strike Price")
        .y_label("Charm (Delta Decay Rate)")
        .save("./Draws/Metrics/charm_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    charm_curve
        .write_html("./Draws/Metrics/charm_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Charm curve saved to ./Draws/Metrics/charm_curve.png");

    // ========================================
    // Charm Surface (Price vs Time)
    // ========================================
    tracing::info!("Generating Charm Surface...");

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

    let charm_surface = option_chain
        .charm_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Charm Surface generated with {} points",
        charm_surface.points.len()
    );

    // Plot surface
    charm_surface
        .plot()
        .title("Charm Surface - Delta Decay (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Charm")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/charm_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    charm_surface
        .write_html("./Draws/Metrics/charm_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Charm surface saved to ./Draws/Metrics/charm_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
