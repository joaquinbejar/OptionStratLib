/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Vanna-Volga Hedge Surface Example
//!
//! This example demonstrates how to generate and visualize a Vanna-Volga
//! hedge surface from an option chain. The surface shows hedge costs across
//! different underlying prices and volatility levels.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/vanna_volga_surface.png`
//! - HTML interactive: `./Draws/Metrics/vanna_volga_surface.html`

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::SurfaceError;
use optionstratlib::metrics::VannaVolgaSurface;
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), SurfaceError> {
    setup_logger();

    // Build a synthetic option chain with proper IV
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,          // 15 strikes on each side
        spos!(5.0),  // $5 intervals
        dec!(-0.15), // Negative skew
        dec!(0.08),  // Smile curvature
        pos!(0.02),  // Spread
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos!(450.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos!(0.20), // 20% base IV
    );

    let option_chain = OptionChain::build_chain(&params);

    tracing::info!(
        "Built option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );
    tracing::info!("Underlying price: {}", option_chain.underlying_price);

    // Define price and volatility ranges for the surface
    let price_range = (pos!(380.0), pos!(520.0)); // ±15% from ATM
    let vol_range = (pos!(0.10), pos!(0.40)); // 10% to 40% volatility

    tracing::info!("Generating Vanna-Volga surface...");
    tracing::info!("  Price range: {} to {}", price_range.0, price_range.1);
    tracing::info!("  Vol range: {} to {}", vol_range.0, vol_range.1);

    // Generate the Vanna-Volga surface
    let vv_surface = option_chain.vanna_volga_surface(price_range, vol_range, 25, 25)?;

    tracing::info!(
        "Vanna-Volga Surface generated with {} points",
        vv_surface.points.len()
    );

    // Find maximum hedge cost
    if let Some(max_point) = vv_surface
        .points
        .iter()
        .max_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
    {
        tracing::info!(
            "Maximum hedge cost: {:.4} at price={:.2}, vol={:.2}%",
            max_point.z,
            max_point.x,
            max_point.y * rust_decimal_macros::dec!(100)
        );
    }

    // Plot and save the surface
    vv_surface
        .plot()
        .title("Vanna-Volga Hedge Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Implied Volatility")
        .z_label("Hedge Cost")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/vanna_volga_surface.png")
        .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    vv_surface
        .write_html("./Draws/Metrics/vanna_volga_surface.html".as_ref())
        .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    tracing::info!("Vanna-Volga surface saved to ./Draws/Metrics/vanna_volga_surface.png");
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/vanna_volga_surface.html");

    Ok(())
}
