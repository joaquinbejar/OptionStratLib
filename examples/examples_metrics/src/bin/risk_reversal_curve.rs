/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Risk Reversal Curve Example
//!
//! This example demonstrates how to generate and visualize a risk reversal
//! curve from an option chain. Risk reversal measures the difference between
//! call and put implied volatilities, indicating market sentiment.
//!
//! ## Interpretation
//! - **Positive RR**: Calls more expensive (bullish sentiment)
//! - **Negative RR**: Puts more expensive (bearish sentiment / demand for protection)
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/risk_reversal_curve.png`
//! - HTML interactive: `./Draws/Metrics/risk_reversal_curve.html`

use optionstratlib::error::CurveError;
use optionstratlib::metrics::RiskReversalCurve;
use optionstratlib::prelude::*;
use rust_decimal::Decimal;

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

    // Generate the risk reversal curve
    let rr_curve = option_chain.risk_reversal_curve()?;

    tracing::info!(
        "Risk Reversal Curve generated with {} points",
        rr_curve.points.len()
    );

    // Analyze market sentiment
    let mut bullish_count = 0;
    let mut bearish_count = 0;
    let mut neutral_count = 0;

    tracing::info!("Risk Reversal Analysis:");
    for point in rr_curve.points.iter() {
        let sentiment = if point.y > Decimal::ZERO {
            bullish_count += 1;
            "bullish"
        } else if point.y < Decimal::ZERO {
            bearish_count += 1;
            "bearish"
        } else {
            neutral_count += 1;
            "neutral"
        };

        // Only show first few and last few points
        let idx = rr_curve.points.iter().position(|p| p == point).unwrap_or(0);
        if idx < 3 || idx >= rr_curve.points.len().saturating_sub(3) {
            tracing::info!(
                "  Strike: {:.2}, RR: {:.4} ({})",
                point.x,
                point.y,
                sentiment
            );
        } else if idx == 3 {
            tracing::info!("  ...");
        }
    }

    tracing::info!("Sentiment Summary:");
    tracing::info!("  Bullish strikes: {}", bullish_count);
    tracing::info!("  Bearish strikes: {}", bearish_count);
    tracing::info!("  Neutral strikes: {}", neutral_count);

    // Plot and save the curve
    rr_curve
        .plot()
        .title("Risk Reversal Curve (SP500)")
        .x_label("Strike Price")
        .y_label("Risk Reversal (Call IV - Put IV)")
        .save("./Draws/Metrics/risk_reversal_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    rr_curve
        .write_html("./Draws/Metrics/risk_reversal_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Risk reversal curve saved to ./Draws/Metrics/risk_reversal_curve.png");
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/risk_reversal_curve.html");

    Ok(())
}
