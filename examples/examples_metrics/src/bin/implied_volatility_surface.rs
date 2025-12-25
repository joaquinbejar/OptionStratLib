use positive::pos_or_panic;
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Implied Volatility Surface Example
//!
//! This example demonstrates how to generate and visualize an implied volatility
//! surface from an option chain. The IV surface shows how implied volatility
//! varies across both strike prices and time to expiration.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/implied_volatility_surface.png`
//! - HTML interactive: `./Draws/Metrics/implied_volatility_surface.html`

use optionstratlib::error::SurfaceError;
use optionstratlib::metrics::ImpliedVolatilitySurface;
use optionstratlib::prelude::*;

fn main() -> Result<(), SurfaceError> {
    setup_logger();

    // Load option chain from JSON file
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
            .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Loaded option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );
    tracing::info!("Underlying price: {}", option_chain.underlying_price);

    // Define days to expiration for the surface
    let days_to_expiry = vec![
        pos_or_panic!(7.0),  // 1 week
        pos_or_panic!(14.0), // 2 weeks
        pos_or_panic!(21.0), // 3 weeks
        pos_or_panic!(30.0), // 1 month
        pos_or_panic!(45.0), // 1.5 months
        pos_or_panic!(60.0), // 2 months
        pos_or_panic!(90.0), // 3 months
    ];

    tracing::info!(
        "Generating IV surface for {} time horizons",
        days_to_expiry.len()
    );

    // Generate the implied volatility surface
    let iv_surface = option_chain.iv_surface(days_to_expiry)?;

    tracing::info!(
        "IV Surface generated with {} points",
        iv_surface.points.len()
    );

    // Display some sample points
    tracing::info!("Sample IV surface points (Strike, Days, IV):");
    for (i, point) in iv_surface.points.iter().enumerate() {
        if i < 5 || i >= iv_surface.points.len().saturating_sub(3) {
            tracing::info!(
                "  Strike: {:.2}, Days: {:.0}, IV: {:.4}",
                point.x,
                point.y,
                point.z
            );
        } else if i == 5 {
            tracing::info!("  ...");
        }
    }

    // Plot and save the surface
    iv_surface
        .plot()
        .title("Implied Volatility Surface (SP500)")
        .x_label("Strike Price")
        .y_label("Days to Expiration")
        .z_label("Implied Volatility")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/implied_volatility_surface.png")
        .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    iv_surface
        .write_html("./Draws/Metrics/implied_volatility_surface.html".as_ref())
        .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    tracing::info!("IV surface saved to ./Draws/Metrics/implied_volatility_surface.png");
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/implied_volatility_surface.html");

    Ok(())
}
