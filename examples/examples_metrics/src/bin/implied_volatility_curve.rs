/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/

//! # Implied Volatility Curve Example
//!
//! This example demonstrates how to generate and visualize an implied volatility
//! curve from an option chain. The IV curve shows how implied volatility varies
//! across different strike prices.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/implied_volatility_curve.png`
//! - HTML interactive: `./Draws/Metrics/implied_volatility_curve.html`

use optionstratlib::error::CurveError;
use optionstratlib::metrics::ImpliedVolatilityCurve;
use optionstratlib::prelude::*;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Load option chain from JSON file
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
            .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Loaded option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );
    tracing::info!("Underlying price: {}", option_chain.underlying_price);

    // Generate the implied volatility curve
    let iv_curve = option_chain.iv_curve()?;

    tracing::info!("IV Curve generated with {} points", iv_curve.points.len());

    // Display some sample points
    tracing::info!("Sample IV curve points:");
    for (i, point) in iv_curve.points.iter().enumerate() {
        if i < 5 || i >= iv_curve.points.len() - 3 {
            tracing::info!(
                "  Strike: {:.2}, IV: {:.4} ({:.2}%)",
                point.x,
                point.y,
                point.y * rust_decimal_macros::dec!(100)
            );
        } else if i == 5 {
            tracing::info!("  ...");
        }
    }

    // Plot and save the curve
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

    tracing::info!("IV curve saved to ./Draws/Metrics/implied_volatility_curve.png");
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/implied_volatility_curve.html");

    Ok(())
}
