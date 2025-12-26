/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Strike Concentration premium weighted Curve Example
//!
//! This example demonstrates how to generate and visualize a premium weighted
//! Strike Concentration curve from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/strike_concentration_premium_weighted_curve.png`
//! - HTML interactive: `./Draws/Metrics/strike_concentration_premium_weighted_curve.html`
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

    let strike_concentration_curve = option_chain.premium_concentration().unwrap();
    tracing::info!(
        "Strike Concentration premium weighted Curve generated with {} points",
        strike_concentration_curve.points.len()
    );

    strike_concentration_curve
        .plot()
        .title("Strike Concentration premium weighted Curve")
        .x_label("Strike Price")
        .y_label("Strike Concentration")
        .save("./Draws/Metrics/strike_concentration_premium_weighted_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    strike_concentration_curve
        .write_html("./Draws/Metrics/strike_concentration_premium_weighted_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Strike Concentration premium weighted curve saved to ./Draws/Metrics/strike_concentration_premium_weighted_curve.png"
    );
    tracing::info!(
        "Strike Concentration premium weighted interactive HTML saved to ./Draws/Metrics/strike_concentration_premium_weighted_curve.html"
    );

    Ok(())
}
