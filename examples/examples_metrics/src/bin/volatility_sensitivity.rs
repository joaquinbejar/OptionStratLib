/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Volatility Sensitivity Example
//!
//! This example demonstrates how to generate and visualize volatility
//! sensitivity curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/volatility_sensitivity_curve.png`
//! - PNG image: `./Draws/Metrics/volatility_sensitivity_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{VolatilitySensitivityCurve, VolatilitySensitivitySurface};
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
    // Volatility Sensitivity Curve (Vega by Strike)
    // ========================================
    tracing::info!("Generating Volatility Sensitivity Curve (Vega)...");

    let vega_curve = option_chain.volatility_sensitivity_curve()?;

    tracing::info!(
        "Vega Curve generated with {} points",
        vega_curve.points.len()
    );

    // Find strike with maximum vega
    let points: Vec<_> = vega_curve.points.iter().collect();
    if let Some(max_vega) = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!("Maximum vega: {:.4} at strike {}", max_vega.y, max_vega.x);
    }

    // Plot curve
    vega_curve
        .plot()
        .title("Volatility Sensitivity Curve - Vega (SPY)")
        .x_label("Strike Price")
        .y_label("Vega")
        .save("./Draws/Metrics/volatility_sensitivity_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    vega_curve
        .write_html("./Draws/Metrics/volatility_sensitivity_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Vega curve saved to ./Draws/Metrics/volatility_sensitivity_curve.png");

    // ========================================
    // Volatility Sensitivity Surface (Price vs Vol)
    // ========================================
    tracing::info!("Generating Volatility Sensitivity Surface...");

    let price_range = (pos_or_panic!(380.0), pos_or_panic!(520.0));
    let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

    let vol_surface = option_chain
        .volatility_sensitivity_surface(price_range, vol_range, 20, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Volatility Surface generated with {} points",
        vol_surface.points.len()
    );

    // Plot surface
    vol_surface
        .plot()
        .title("Volatility Sensitivity Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Implied Volatility")
        .z_label("Option Value")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/volatility_sensitivity_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    vol_surface
        .write_html("./Draws/Metrics/volatility_sensitivity_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Volatility surface saved to ./Draws/Metrics/volatility_sensitivity_surface.png"
    );
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
