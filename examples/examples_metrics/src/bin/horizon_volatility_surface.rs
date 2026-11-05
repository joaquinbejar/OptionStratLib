/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Horizon Volatility Surface Example
//!
//! This example projects the implied volatility of an option chain over several
//! time horizons with the square root of time rule. The result is the
//! volatility over each horizon, not annualized and not a calibrated implied
//! volatility surface: every horizon comes from one chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/horizon_volatility_surface.png`
//! - HTML interactive: `./Draws/Metrics/horizon_volatility_surface.html`

use optionstratlib::error::SurfaceError;
use optionstratlib::metrics::HorizonVolatilitySurface;
use optionstratlib::prelude::*;
use osl_example_support::setup_logger;
use positive::pos_or_panic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let horizon_days = vec![
        pos_or_panic!(7.0),  // 1 week
        pos_or_panic!(14.0), // 2 weeks
        pos_or_panic!(21.0), // 3 weeks
        pos_or_panic!(30.0), // 1 month
        pos_or_panic!(45.0), // 1.5 months
        pos_or_panic!(60.0), // 2 months
        pos_or_panic!(90.0), // 3 months
    ];

    tracing::info!(
        "Projecting volatility over {} time horizons",
        horizon_days.len()
    );

    // Project the implied volatility over each horizon
    let horizon_volatility_surface = option_chain.horizon_volatility_surface(horizon_days)?;

    tracing::info!(
        "Horizon volatility surface generated with {} points",
        horizon_volatility_surface.points.len()
    );

    // Display some sample points
    tracing::info!("Sample points (Strike, Days, Volatility over the horizon):");
    for (i, point) in horizon_volatility_surface.points.iter().enumerate() {
        if i < 5 || i >= horizon_volatility_surface.points.len().saturating_sub(3) {
            tracing::info!(
                "  Strike: {:.2}, Days: {:.0}, Volatility: {:.4}",
                point.x,
                point.y,
                point.z
            );
        } else if i == 5 {
            tracing::info!("  ...");
        }
    }

    // Plot and save the surface
    horizon_volatility_surface
        .plot()
        .title("Volatility over the Horizon (SP500)")
        .x_label("Strike Price")
        .y_label("Horizon (days)")
        .z_label("Volatility over the horizon")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/horizon_volatility_surface.png")
        .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    horizon_volatility_surface
        .write_html("./Draws/Metrics/horizon_volatility_surface.html".as_ref())
        .map_err(|e| SurfaceError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Horizon volatility surface saved to ./Draws/Metrics/horizon_volatility_surface.png"
    );
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/horizon_volatility_surface.html");

    Ok(())
}
