/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Volatility Skew Curve Example
//!
//! This example demonstrates how to generate and visualize a volatility skew
//! curve from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/volatility_skew_curve.png`
//! - HTML interactive: `./Draws/Metrics/volatility_skew_curve.html`
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

    let skew_curve = option_chain.volatility_skew().unwrap();
    tracing::info!(
        "Volatility Skew Curve generated with {} points",
        skew_curve.points.len()
    );

    skew_curve
        .plot()
        .title("Volatility Skew Curve")
        .x_label("Moneyness")
        .y_label("Implied Volatility")
        .save("./Draws/Metrics/volatility_skew_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    skew_curve
        .write_html("./Draws/Metrics/volatility_skew_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Volatility Skew curve saved to ./Draws/Metrics/volatility_skew_curve.png");
    tracing::info!(
        "Volatility Skew interactive HTML saved to ./Draws/Metrics/volatility_skew_curve.html"
    );

    Ok(())
}
