/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # All Temporal Metrics Example
//!
//! This comprehensive example demonstrates all temporal metrics available in
//! OptionStratLib:
//! - Theta Curve (by strike) and Surface (price vs time)
//! - Charm Curve (by strike) and Surface (price vs time)
//! - Color Curve (by strike) and Surface (price vs time)
//!
//! ## Output
//! All graphs are saved to `./Draws/Metrics/`

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{CharmSurface, ColorSurface, ThetaSurface};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    tracing::info!("=== OptionStratLib Temporal Metrics Demo ===\n");

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

    tracing::info!("Option Chain Information:");
    tracing::info!("  Symbol: {}", option_chain.symbol);
    tracing::info!("  Underlying Price: {}", option_chain.underlying_price);
    tracing::info!("  Number of strikes: {}", option_chain.options.len());
    tracing::info!("");

    let price_range = (pos_or_panic!(380.0), pos_or_panic!(520.0));
    let days = vec![
        pos_or_panic!(1.0),
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
    ];

    // ========================================
    // 1. Theta Curve
    // ========================================
    tracing::info!("1. THETA CURVE (Time Decay)");
    tracing::info!("   Shows theta by strike - rate of time decay");

    let theta_curve = option_chain.theta_curve()?;
    tracing::info!("   Generated {} curve points", theta_curve.points.len());

    theta_curve
        .plot()
        .title("Theta Curve - Time Decay (SPY)")
        .x_label("Strike Price")
        .y_label("Theta")
        .save("./Draws/Metrics/theta_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    theta_curve
        .write_html("./Draws/Metrics/theta_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: theta_curve.png/html\n");

    // ========================================
    // 2. Theta Surface
    // ========================================
    tracing::info!("2. THETA SURFACE");
    tracing::info!("   Shows theta across price and time");

    let theta_surface = option_chain
        .theta_surface(price_range, days.clone(), 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", theta_surface.points.len());

    theta_surface
        .plot()
        .title("Theta Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Theta")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/theta_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    theta_surface
        .write_html("./Draws/Metrics/theta_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: theta_surface.png/html\n");

    // ========================================
    // 3. Charm Curve
    // ========================================
    tracing::info!("3. CHARM CURVE (Delta Decay)");
    tracing::info!("   Shows charm by strike - rate of delta change over time");

    let charm_curve = option_chain.charm_curve()?;
    tracing::info!("   Generated {} curve points", charm_curve.points.len());

    charm_curve
        .plot()
        .title("Charm Curve - Delta Decay (SPY)")
        .x_label("Strike Price")
        .y_label("Charm")
        .save("./Draws/Metrics/charm_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    charm_curve
        .write_html("./Draws/Metrics/charm_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: charm_curve.png/html\n");

    // ========================================
    // 4. Charm Surface
    // ========================================
    tracing::info!("4. CHARM SURFACE");
    tracing::info!("   Shows charm across price and time");

    let charm_surface = option_chain
        .charm_surface(price_range, days.clone(), 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", charm_surface.points.len());

    charm_surface
        .plot()
        .title("Charm Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Charm")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/charm_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    charm_surface
        .write_html("./Draws/Metrics/charm_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: charm_surface.png/html\n");

    // ========================================
    // 5. Color Curve
    // ========================================
    tracing::info!("5. COLOR CURVE (Gamma Decay)");
    tracing::info!("   Shows color by strike - rate of gamma change over time");

    let color_curve = option_chain.color_curve()?;
    tracing::info!("   Generated {} curve points", color_curve.points.len());

    color_curve
        .plot()
        .title("Color Curve - Gamma Decay (SPY)")
        .x_label("Strike Price")
        .y_label("Color")
        .save("./Draws/Metrics/color_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    color_curve
        .write_html("./Draws/Metrics/color_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: color_curve.png/html\n");

    // ========================================
    // 6. Color Surface
    // ========================================
    tracing::info!("6. COLOR SURFACE");
    tracing::info!("   Shows color across price and time");

    let color_surface = option_chain
        .color_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", color_surface.points.len());

    color_surface
        .plot()
        .title("Color Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Color")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/color_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    color_surface
        .write_html("./Draws/Metrics/color_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: color_surface.png/html\n");

    // ========================================
    // Summary
    // ========================================
    tracing::info!("=== Summary ===");
    tracing::info!("All temporal metrics generated successfully!");
    tracing::info!("Output directory: ./Draws/Metrics/");
    tracing::info!("");
    tracing::info!("Files created:");
    tracing::info!("  - theta_curve.png/html");
    tracing::info!("  - theta_surface.png/html");
    tracing::info!("  - charm_curve.png/html");
    tracing::info!("  - charm_surface.png/html");
    tracing::info!("  - color_curve.png/html");
    tracing::info!("  - color_surface.png/html");

    Ok(())
}
