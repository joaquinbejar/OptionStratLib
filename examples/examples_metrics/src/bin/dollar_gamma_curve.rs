/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/

//! # Dollar Gamma Curve Example
//!
//! This example demonstrates how to generate and visualize a dollar gamma
//! curve from an option chain. Dollar gamma measures gamma exposure in
//! monetary terms.
//!
//! ## Formula
//! Dollar Gamma = Gamma × Spot² × 0.01
//!
//! This shows how much the delta will change for a 1% move in the underlying.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/dollar_gamma_curve_call.png`
//! - PNG image: `./Draws/Metrics/dollar_gamma_curve_put.png`
//! - HTML interactive: `./Draws/Metrics/dollar_gamma_curve_call.html`
//! - HTML interactive: `./Draws/Metrics/dollar_gamma_curve_put.html`

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::DollarGammaCurve;
use optionstratlib::model::OptionStyle;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain with proper IV for gamma calculation
    // This ensures we get meaningful gamma values across strikes
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,          // 15 strikes on each side of ATM
        spos!(5.0),  // $5 strike intervals
        dec!(-0.15), // Slight negative skew
        dec!(0.08),  // Smile curvature
        pos!(0.02),  // Spread
        2,           // Decimal places
        OptionDataPriceParams::new(
            Some(Box::new(pos!(450.0))),            // Underlying price
            Some(ExpirationDate::Days(pos!(30.0))), // 30 days to expiry
            Some(dec!(0.05)),                       // Risk-free rate
            spos!(0.01),                            // Dividend yield
            Some("SPY".to_string()),
        ),
        pos!(0.20), // Base IV of 20%
    );

    let option_chain = OptionChain::build_chain(&params);

    tracing::info!(
        "Built option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );
    tracing::info!("Underlying price: {}", option_chain.underlying_price);

    // Generate dollar gamma curve for calls
    let dg_curve_call = option_chain.dollar_gamma_curve(&OptionStyle::Call)?;

    tracing::info!(
        "Dollar Gamma Curve (Calls) generated with {} points",
        dg_curve_call.points.len()
    );

    // Find the strike with maximum dollar gamma
    if let Some(max_dg_point) = dg_curve_call
        .points
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    {
        tracing::info!(
            "Maximum Dollar Gamma (Calls): ${:.2} at strike {:.2}",
            max_dg_point.y,
            max_dg_point.x
        );
    }

    // Display sample points
    tracing::info!("Sample Dollar Gamma points (Calls):");
    for (i, point) in dg_curve_call.points.iter().enumerate() {
        if i < 3 || i >= dg_curve_call.points.len().saturating_sub(3) {
            tracing::info!("  Strike: {:.2}, Dollar Gamma: ${:.4}", point.x, point.y);
        } else if i == 3 {
            tracing::info!("  ...");
        }
    }

    // Plot and save call curve
    dg_curve_call
        .plot()
        .title("Dollar Gamma Curve - Calls (SP500)")
        .x_label("Strike Price")
        .y_label("Dollar Gamma ($)")
        .save("./Draws/Metrics/dollar_gamma_curve_call.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_curve_call
        .write_html("./Draws/Metrics/dollar_gamma_curve_call.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Dollar gamma curve (calls) saved to ./Draws/Metrics/dollar_gamma_curve_call.png"
    );

    // Generate dollar gamma curve for puts
    let dg_curve_put = option_chain.dollar_gamma_curve(&OptionStyle::Put)?;

    tracing::info!(
        "Dollar Gamma Curve (Puts) generated with {} points",
        dg_curve_put.points.len()
    );

    // Find the strike with maximum dollar gamma for puts
    if let Some(max_dg_point) = dg_curve_put
        .points
        .iter()
        .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    {
        tracing::info!(
            "Maximum Dollar Gamma (Puts): ${:.2} at strike {:.2}",
            max_dg_point.y,
            max_dg_point.x
        );
    }

    // Plot and save put curve
    dg_curve_put
        .plot()
        .title("Dollar Gamma Curve - Puts (SP500)")
        .x_label("Strike Price")
        .y_label("Dollar Gamma ($)")
        .save("./Draws/Metrics/dollar_gamma_curve_put.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    dg_curve_put
        .write_html("./Draws/Metrics/dollar_gamma_curve_put.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Dollar gamma curve (puts) saved to ./Draws/Metrics/dollar_gamma_curve_put.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
