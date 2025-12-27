/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Put Call Ratio premium weighted Curve Example
//!
//! This example demonstrates how to generate and visualize a premium weighted Put/Call Ratio
//! curve from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/put_call_ratio_premium_weighted_curve.png`
//! - HTML interactive: `./Draws/Metrics/put_call_ratio_premium_weighted_curve.html`
use optionstratlib::prelude::*;

fn main() -> Result<(), CurveError> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
            .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Loaded option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );
    tracing::info!("Underlying price: {}", option_chain.underlying_price);

    let pcr_curve = option_chain.premium_weighted_pcr().unwrap();
    tracing::info!(
        "Put/Call Ratio premium weighted Curve generated with {} points",
        pcr_curve.points.len()
    );

    pcr_curve
        .plot()
        .title("Put/Call Ratio premium weighted Curve")
        .x_label("Strike Price")
        .y_label("Put/Call Ratio")
        .save("./Draws/Metrics/put_call_ratio_premium_weighted_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    pcr_curve
        .write_html("./Draws/Metrics/put_call_ratio_premium_weighted_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Put/Call ratio premium weighted curve saved to ./Draws/Metrics/put_call_ratio_premium_weighted_curve.png"
    );
    tracing::info!(
        "Put/Call ratio premium weighted interactive HTML saved to ./Draws/Metrics/put_call_ratio_premium_weighted_curve.html"
    );

    Ok(())
}
