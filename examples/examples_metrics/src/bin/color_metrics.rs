use positive::pos_or_panic;
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Color Metrics Example
//!
//! This example demonstrates how to generate and visualize color (gamma decay)
//! curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/color_curve.png`
//! - PNG image: `./Draws/Metrics/color_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::ColorSurface;
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
    // Color Curve (by Strike)
    // ========================================
    tracing::info!("Generating Color Curve (Gamma Decay)...");

    let color_curve = option_chain.color_curve()?;

    tracing::info!(
        "Color Curve generated with {} points",
        color_curve.points.len()
    );

    // Plot curve
    color_curve
        .plot()
        .title("Color Curve - Gamma Decay (SPY)")
        .x_label("Strike Price")
        .y_label("Color (Gamma Decay Rate)")
        .save("./Draws/Metrics/color_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    color_curve
        .write_html("./Draws/Metrics/color_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Color curve saved to ./Draws/Metrics/color_curve.png");

    // ========================================
    // Color Surface (Price vs Time)
    // ========================================
    tracing::info!("Generating Color Surface...");

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

    let color_surface = option_chain
        .color_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Color Surface generated with {} points",
        color_surface.points.len()
    );

    // Plot surface
    color_surface
        .plot()
        .title("Color Surface - Gamma Decay (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Color")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/color_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    color_surface
        .write_html("./Draws/Metrics/color_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Color surface saved to ./Draws/Metrics/color_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
