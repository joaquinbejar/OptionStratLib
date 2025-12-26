/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # All Risk Metrics Example
//!
//! This comprehensive example demonstrates all risk metrics available in
//! OptionStratLib:
//! - Implied Volatility Curve (by strike)
//! - Implied Volatility Surface (strike vs time)
//! - Risk Reversal Curve (by strike)
//! - Dollar Gamma Curve (by strike)
//!
//! ## Output
//! All graphs are saved to `./Draws/Metrics/`

use optionstratlib::error::CurveError;
use optionstratlib::metrics::{
    DollarGammaCurve, ImpliedVolatilityCurve, ImpliedVolatilitySurface, RiskReversalCurve,
};
use optionstratlib::model::OptionStyle;
use optionstratlib::prelude::*;
use positive::pos_or_panic;
use rust_decimal::Decimal;

fn main() -> Result<(), CurveError> {
    setup_logger();

    tracing::info!("=== OptionStratLib Risk Metrics Demo ===\n");

    // Load option chain from JSON file
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
            .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Option Chain Information:");
    tracing::info!("  Symbol: {}", option_chain.symbol);
    tracing::info!("  Underlying Price: {}", option_chain.underlying_price);
    tracing::info!("  Number of strikes: {}", option_chain.options.len());
    tracing::info!("");

    // ========================================
    // 1. Implied Volatility Curve
    // ========================================
    tracing::info!("1. IMPLIED VOLATILITY CURVE");
    tracing::info!("   Shows how IV varies across strike prices");

    let iv_curve = option_chain.iv_curve()?;
    tracing::info!("   Generated {} points", iv_curve.points.len());

    // Find ATM IV
    let atm_strike = option_chain.underlying_price.to_dec();
    if let Some(atm_point) = iv_curve.points.iter().min_by(|a, b| {
        (a.x - atm_strike)
            .abs()
            .partial_cmp(&(b.x - atm_strike).abs())
            .unwrap()
    }) {
        tracing::info!(
            "   ATM IV (strike ~{:.0}): {:.2}%",
            atm_point.x,
            atm_point.y * Decimal::from(100)
        );
    }

    iv_curve
        .plot()
        .title("Implied Volatility Curve (SP500)")
        .x_label("Strike Price")
        .y_label("Implied Volatility")
        .save("./Draws/Metrics/implied_volatility_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    iv_curve
        .write_html("./Draws/Metrics/implied_volatility_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: implied_volatility_curve.png/html\n");

    // ========================================
    // 2. Implied Volatility Surface
    // ========================================
    tracing::info!("2. IMPLIED VOLATILITY SURFACE");
    tracing::info!("   Shows IV across strikes AND time horizons");

    let days_to_expiry = vec![
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
        pos_or_panic!(90.0),
    ];

    let iv_surface = option_chain
        .iv_surface(days_to_expiry)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", iv_surface.points.len());

    iv_surface
        .plot()
        .title("Implied Volatility Surface (SP500)")
        .x_label("Strike Price")
        .y_label("Days to Expiration")
        .z_label("Implied Volatility")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/implied_volatility_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    iv_surface
        .write_html("./Draws/Metrics/implied_volatility_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: implied_volatility_surface.png/html\n");

    // ========================================
    // 3. Risk Reversal Curve
    // ========================================
    tracing::info!("3. RISK REVERSAL CURVE");
    tracing::info!("   Measures market sentiment (Call IV - Put IV)");

    let rr_curve = option_chain.risk_reversal_curve()?;
    tracing::info!("   Generated {} points", rr_curve.points.len());

    // Analyze sentiment
    let bullish = rr_curve
        .points
        .iter()
        .filter(|p| p.y > Decimal::ZERO)
        .count();
    let bearish = rr_curve
        .points
        .iter()
        .filter(|p| p.y < Decimal::ZERO)
        .count();

    tracing::info!(
        "   Sentiment: {} bullish, {} bearish strikes",
        bullish,
        bearish
    );

    rr_curve
        .plot()
        .title("Risk Reversal Curve (SP500)")
        .x_label("Strike Price")
        .y_label("Risk Reversal")
        .save("./Draws/Metrics/risk_reversal_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    rr_curve
        .write_html("./Draws/Metrics/risk_reversal_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: risk_reversal_curve.png/html\n");

    // ========================================
    // 4. Dollar Gamma Curve
    // ========================================
    tracing::info!("4. DOLLAR GAMMA CURVE");
    tracing::info!("   Gamma exposure in $ terms (Gamma × Spot² × 0.01)");
    tracing::info!("   Building synthetic chain for proper gamma calculation...");

    // Build a synthetic chain with proper IV for meaningful gamma values
    use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use rust_decimal_macros::dec;

    let dg_params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,                  // 15 strikes on each side
        spos!(5.0),          // $5 intervals
        dec!(-0.15),         // Skew
        dec!(0.08),          // Smile
        pos_or_panic!(0.02), // Spread
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(450.0))),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos_or_panic!(0.20), // 20% base IV
    );

    let dg_chain = OptionChain::build_chain(&dg_params);

    let dg_curve_call = dg_chain.dollar_gamma_curve(&OptionStyle::Call)?;
    let dg_curve_put = dg_chain.dollar_gamma_curve(&OptionStyle::Put)?;

    tracing::info!(
        "   Generated {} call points, {} put points",
        dg_curve_call.points.len(),
        dg_curve_put.points.len()
    );

    // Find max dollar gamma
    if let Some(max_dg) = dg_curve_call
        .points
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    {
        tracing::info!(
            "   Max Dollar Gamma: ${:.2} at strike {:.0}",
            max_dg.y,
            max_dg.x
        );
    }

    dg_curve_call
        .plot()
        .title("Dollar Gamma Curve - Calls (SPY)")
        .x_label("Strike Price")
        .y_label("Dollar Gamma ($)")
        .save("./Draws/Metrics/dollar_gamma_curve_call.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_curve_call
        .write_html("./Draws/Metrics/dollar_gamma_curve_call.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_curve_put
        .plot()
        .title("Dollar Gamma Curve - Puts (SPY)")
        .x_label("Strike Price")
        .y_label("Dollar Gamma ($)")
        .save("./Draws/Metrics/dollar_gamma_curve_put.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_curve_put
        .write_html("./Draws/Metrics/dollar_gamma_curve_put.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: dollar_gamma_curve_call.png/html");
    tracing::info!("   Saved: dollar_gamma_curve_put.png/html\n");

    // ========================================
    // Summary
    // ========================================
    tracing::info!("=== Summary ===");
    tracing::info!("All risk metrics generated successfully!");
    tracing::info!("Output directory: ./Draws/Metrics/");
    tracing::info!("");
    tracing::info!("Files created:");
    tracing::info!("  - implied_volatility_curve.png/html");
    tracing::info!("  - implied_volatility_surface.png/html");
    tracing::info!("  - risk_reversal_curve.png/html");
    tracing::info!("  - dollar_gamma_curve_call.png/html");
    tracing::info!("  - dollar_gamma_curve_put.png/html");

    Ok(())
}
