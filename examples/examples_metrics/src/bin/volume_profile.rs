/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Volume Profile Example
//!
//! This example demonstrates how to generate and visualize volume profile
//! curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/volume_profile_curve.png`
//! - PNG image: `./Draws/Metrics/volume_profile_surface.png`
//! - HTML interactive files

use optionstratlib::chains::OptionData;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{VolumeProfileCurve, VolumeProfileSurface};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain with volume data
    let mut chain = OptionChain::new("SPY", pos!(450.0), "2024-12-31".to_string(), None, None);

    // Add options with realistic volume distribution
    // Volume is highest near ATM and decreases for OTM options
    let strikes_data = [
        // (strike, volume)
        (pos!(380.0), pos!(500.0)),
        (pos!(400.0), pos!(1500.0)),
        (pos!(420.0), pos!(3500.0)),
        (pos!(430.0), pos!(5000.0)),
        (pos!(440.0), pos!(8000.0)),
        (pos!(445.0), pos!(12000.0)),
        (pos!(450.0), pos!(18000.0)), // ATM - highest volume
        (pos!(455.0), pos!(14000.0)),
        (pos!(460.0), pos!(9000.0)),
        (pos!(470.0), pos!(5500.0)),
        (pos!(480.0), pos!(3000.0)),
        (pos!(500.0), pos!(1200.0)),
        (pos!(520.0), pos!(400.0)),
    ];

    for (strike, volume) in strikes_data {
        let option_data = OptionData::new(
            strike,
            spos!(10.0),
            spos!(10.5),
            spos!(10.0),
            spos!(10.5),
            pos!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(volume.to_f64()),
            Some(5000),
            Some("SPY".to_string()),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(Box::new(pos!(450.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);
    }

    tracing::info!(
        "Built option chain for {} with {} options",
        chain.symbol,
        chain.options.len()
    );

    // ========================================
    // Volume Profile Curve
    // ========================================
    tracing::info!("Generating Volume Profile Curve...");

    let volume_curve = chain.volume_profile_curve()?;

    tracing::info!(
        "Volume Profile Curve generated with {} points",
        volume_curve.points.len()
    );

    // Find strike with highest volume
    let points: Vec<_> = volume_curve.points.iter().collect();
    if let Some(max_vol) = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "Highest volume: {} contracts at strike {}",
            max_vol.y,
            max_vol.x
        );
    }

    // Calculate total volume
    let total_volume: rust_decimal::Decimal = points.iter().map(|p| p.y).sum();
    tracing::info!(
        "Total volume across all strikes: {} contracts",
        total_volume
    );

    // Plot curve
    volume_curve
        .plot()
        .title("Volume Profile Curve (SPY)")
        .x_label("Strike Price")
        .y_label("Volume (contracts)")
        .save("./Draws/Metrics/volume_profile_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    volume_curve
        .write_html("./Draws/Metrics/volume_profile_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Volume Profile curve saved to ./Draws/Metrics/volume_profile_curve.png");

    // ========================================
    // Volume Profile Surface
    // ========================================
    tracing::info!("Generating Volume Profile Surface...");

    let days = vec![
        pos!(1.0),
        pos!(5.0),
        pos!(10.0),
        pos!(15.0),
        pos!(20.0),
        pos!(25.0),
        pos!(30.0),
    ];

    let volume_surface = chain
        .volume_profile_surface(days)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Volume Profile Surface generated with {} points",
        volume_surface.points.len()
    );

    // Plot surface
    volume_surface
        .plot()
        .title("Volume Profile Surface (SPY)")
        .x_label("Strike Price")
        .y_label("Days to Expiration")
        .z_label("Volume")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/volume_profile_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    volume_surface
        .write_html("./Draws/Metrics/volume_profile_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Volume Profile surface saved to ./Draws/Metrics/volume_profile_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
