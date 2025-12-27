/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # All Price Metrics Example
//!
//! This comprehensive example demonstrates all price metrics available in
//! OptionStratLib:
//! - Volatility Skew (by moneyness point)
//! - Put/Call Ratio Premium Weighted (by strike)
//! - Strike Concentration Premium Weighted (by strike)
//!
//! ## Output
//! All graphs are saved to `./Draws/Metrics/`

use optionstratlib::prelude::*;

fn main() -> Result<(), CurveError> {
    setup_logger();

    tracing::info!("=== OptionStratLib Price Metrics Demo ===\n");

    let chain = OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Option Chain Information:");
    tracing::info!("  Symbol: {}", chain.symbol);
    tracing::info!("  Underlying Price: {}", chain.underlying_price);
    tracing::info!("  Number of strikes: {}", chain.options.len());
    tracing::info!("");

    // ========================================
    // 1.Volatility Skew Curve
    // ========================================
    tracing::info!("1. VOLATILITY SKEW CURVE");
    tracing::info!("   Shows implied volatility across moneyness points");

    let skew_curve = chain.volatility_skew()?;
    tracing::info!("   Generated {} curve points", skew_curve.points.len());

    skew_curve
        .plot()
        .title("Volatility Skew Curve (SP500)")
        .x_label("Moneyness")
        .y_label("Implied Volatility")
        .save("./Draws/Metrics/volatility_skew_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    skew_curve
        .write_html("./Draws/Metrics/volatility_skew_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: volatility_skew_curve.png/html\n");

    // ========================================
    // 2. Put/Call Ratio Premium Weighted Curve
    // ========================================
    tracing::info!("2. PUT/CALL RATIO PREMIUM WEIGHTED CURVE");
    tracing::info!("   Shows put/call ratio distribution by strike");

    let pcr_curve = chain.premium_weighted_pcr()?;
    tracing::info!("   Generated {} curve points", pcr_curve.points.len());

    pcr_curve
        .plot()
        .title("Put/Call Ratio Premium Weighted (SP500)")
        .x_label("Strike Price")
        .y_label("Put/Call Ratio")
        .save("./Draws/Metrics/put_call_ratio_premium_weighted_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    pcr_curve
        .write_html("./Draws/Metrics/put_call_ratio_premium_weighted_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: put_call_ratio_premium_weighted_curve.png/html\n");

    // ========================================
    // 3. Strike Concentration Premium Weighted Curve
    // ========================================
    tracing::info!("3. STRIKE CONCENTRATION PREMIUM WEIGHTED CURVE");
    tracing::info!("   Shows Strike Concentration distribution by strike");

    let strike_concentration_curve = chain.premium_concentration()?;
    tracing::info!(
        "   Generated {} curve points",
        strike_concentration_curve.points.len()
    );

    strike_concentration_curve
        .plot()
        .title("Strike Concentration Premium Weighted (SP500)")
        .x_label("Strike Price")
        .y_label("Strike Concentration")
        .save("./Draws/Metrics/strike_concentration_premium_weighted_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    strike_concentration_curve
        .write_html("./Draws/Metrics/strike_concentration_premium_weighted_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: strike_concentration_premium_weighted_curve.png/html\n");

    // ========================================
    // Summary
    // ========================================
    tracing::info!("=== Summary ===");
    tracing::info!("All price metrics generated successfully!");
    tracing::info!("Output directory: ./Draws/Metrics/");
    tracing::info!("");
    tracing::info!("Files created:");
    tracing::info!("  - volatility_skew_curve.png/html");
    tracing::info!("  - put_call_ratio_premium_weighted_curve.png/html");
    tracing::info!("  - strike_concentration_premium_weighted_curve.png/html");

    Ok(())
}
