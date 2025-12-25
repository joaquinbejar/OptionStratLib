/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # All Composite Metrics Example
//!
//! This comprehensive example demonstrates all composite metrics available in
//! OptionStratLib:
//! - Vanna-Volga Hedge Surface (price vs volatility)
//! - Delta-Gamma Profile Curve (by strike) and Surface (price vs time)
//! - Smile Dynamics Curve (by strike) and Surface (strike vs time)
//!
//! ## Output
//! All graphs are saved to `./Draws/Metrics/`

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{
    DeltaGammaProfileCurve, DeltaGammaProfileSurface, SmileDynamicsCurve, SmileDynamicsSurface,
    VannaVolgaSurface,
};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    tracing::info!("=== OptionStratLib Composite Metrics Demo ===\n");

    // Build a synthetic option chain with pronounced smile
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,
        spos!(5.0),
        dec!(-0.18), // Negative skew
        dec!(0.10),  // Smile curvature
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

    // ========================================
    // 1. Vanna-Volga Hedge Surface
    // ========================================
    tracing::info!("1. VANNA-VOLGA HEDGE SURFACE");
    tracing::info!("   Shows hedge costs across price and volatility space");

    let price_range = (pos_or_panic!(380.0), pos_or_panic!(520.0));
    let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

    let vv_surface = option_chain
        .vanna_volga_surface(price_range, vol_range, 25, 25)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", vv_surface.points.len());

    vv_surface
        .plot()
        .title("Vanna-Volga Hedge Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Implied Volatility")
        .z_label("Hedge Cost")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/vanna_volga_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    vv_surface
        .write_html("./Draws/Metrics/vanna_volga_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: vanna_volga_surface.png/html\n");

    // ========================================
    // 2. Delta-Gamma Profile Curve
    // ========================================
    tracing::info!("2. DELTA-GAMMA PROFILE CURVE");
    tracing::info!("   Combined delta + gamma exposure by strike");

    let dg_curve = option_chain.delta_gamma_curve()?;
    tracing::info!("   Generated {} curve points", dg_curve.points.len());

    if let Some(max_point) = dg_curve
        .points
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    {
        tracing::info!(
            "   Max exposure: ${:.2} at strike {:.0}",
            max_point.y,
            max_point.x
        );
    }

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

    tracing::info!("   Saved: delta_gamma_curve.png/html\n");

    // ========================================
    // 3. Delta-Gamma Profile Surface
    // ========================================
    tracing::info!("3. DELTA-GAMMA PROFILE SURFACE");
    tracing::info!("   Delta exposure across price and time");

    let days = vec![
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
    ];

    let dg_surface = option_chain
        .delta_gamma_surface(price_range, days.clone(), 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", dg_surface.points.len());

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

    tracing::info!("   Saved: delta_gamma_surface.png/html\n");

    // ========================================
    // 4. Smile Dynamics Curve
    // ========================================
    tracing::info!("4. SMILE DYNAMICS CURVE");
    tracing::info!("   Current volatility smile shape");

    let smile_curve = option_chain.smile_dynamics_curve()?;
    tracing::info!("   Generated {} curve points", smile_curve.points.len());

    smile_curve
        .plot()
        .title("Smile Dynamics Curve (SPY)")
        .x_label("Strike Price")
        .y_label("Implied Volatility")
        .save("./Draws/Metrics/smile_dynamics_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    smile_curve
        .write_html("./Draws/Metrics/smile_dynamics_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: smile_dynamics_curve.png/html\n");

    // ========================================
    // 5. Smile Dynamics Surface
    // ========================================
    tracing::info!("5. SMILE DYNAMICS SURFACE");
    tracing::info!("   Smile evolution across time horizons");

    let smile_days = vec![
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
        pos_or_panic!(90.0),
    ];

    let smile_surface = option_chain
        .smile_dynamics_surface(smile_days)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", smile_surface.points.len());

    smile_surface
        .plot()
        .title("Smile Dynamics Surface (SPY)")
        .x_label("Strike Price")
        .y_label("Days to Expiration")
        .z_label("Implied Volatility")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/smile_dynamics_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    smile_surface
        .write_html("./Draws/Metrics/smile_dynamics_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: smile_dynamics_surface.png/html\n");

    // ========================================
    // Summary
    // ========================================
    tracing::info!("=== Summary ===");
    tracing::info!("All composite metrics generated successfully!");
    tracing::info!("Output directory: ./Draws/Metrics/");
    tracing::info!("");
    tracing::info!("Files created:");
    tracing::info!("  - vanna_volga_surface.png/html");
    tracing::info!("  - delta_gamma_curve.png/html");
    tracing::info!("  - delta_gamma_surface.png/html");
    tracing::info!("  - smile_dynamics_curve.png/html");
    tracing::info!("  - smile_dynamics_surface.png/html");

    Ok(())
}
