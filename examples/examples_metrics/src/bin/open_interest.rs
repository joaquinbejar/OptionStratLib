/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Open Interest Distribution Example
//!
//! This example demonstrates how to generate and visualize an open interest
//! distribution curve from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/open_interest_curve.png`
//! - HTML interactive: `./Draws/Metrics/open_interest_curve.html`

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::OptionData;
use optionstratlib::error::CurveError;
use optionstratlib::metrics::OpenInterestCurve;
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain with open interest data
    let mut chain = OptionChain::new(
        "SPY",
        pos_or_panic!(450.0),
        "2024-12-31".to_string(),
        None,
        None,
    );

    // Add options with realistic OI distribution
    // OI tends to concentrate at round strikes and near ATM
    let strikes_data = [
        // (strike, open_interest)
        (pos_or_panic!(380.0), 2500u64),
        (pos_or_panic!(400.0), 15000u64), // Round strike
        (pos_or_panic!(420.0), 8000u64),
        (pos_or_panic!(430.0), 6000u64),
        (pos_or_panic!(440.0), 12000u64),
        (pos_or_panic!(445.0), 9000u64),
        (pos_or_panic!(450.0), 45000u64), // ATM round strike - highest OI
        (pos_or_panic!(455.0), 11000u64),
        (pos_or_panic!(460.0), 14000u64),
        (pos_or_panic!(470.0), 7500u64),
        (pos_or_panic!(480.0), 10000u64),
        (pos_or_panic!(500.0), 25000u64), // Round strike
        (pos_or_panic!(520.0), 5000u64),
    ];

    for (strike, oi) in strikes_data {
        let option_data = OptionData::new(
            strike,
            spos!(10.0),
            spos!(10.5),
            spos!(10.0),
            spos!(10.5),
            pos_or_panic!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(1000.0),
            Some(oi),
            Some("SPY".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(pos_or_panic!(450.0))),
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
    tracing::info!("Underlying price: {}", chain.underlying_price);

    // Generate the open interest curve
    tracing::info!("Generating Open Interest Distribution Curve...");

    let oi_curve = chain.open_interest_curve()?;

    tracing::info!(
        "Open Interest Curve generated with {} points",
        oi_curve.points.len()
    );

    // Analyze OI distribution
    let points: Vec<_> = oi_curve.points.iter().collect();

    // Find strike with highest OI (potential max pain level)
    if let Some(max_oi) = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "Maximum OI: {} contracts at strike {} (potential max pain)",
            max_oi.y,
            max_oi.x
        );
    }

    // Calculate total OI
    let total_oi: rust_decimal::Decimal = points.iter().map(|p| p.y).sum();
    tracing::info!("Total open interest: {} contracts", total_oi);

    // Find OI concentration (top 3 strikes)
    let mut sorted_points: Vec<_> = points.iter().collect();
    sorted_points.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap());

    tracing::info!("Top 3 strikes by OI:");
    for (i, point) in sorted_points.iter().take(3).enumerate() {
        let pct = point.y / total_oi * dec!(100);
        tracing::info!(
            "  {}. Strike {}: {} contracts ({:.1}% of total)",
            i + 1,
            point.x,
            point.y,
            pct
        );
    }

    // Plot and save the curve
    oi_curve
        .plot()
        .title("Open Interest Distribution (SPY)")
        .x_label("Strike Price")
        .y_label("Open Interest (contracts)")
        .save("./Draws/Metrics/open_interest_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    oi_curve
        .write_html("./Draws/Metrics/open_interest_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Open Interest curve saved to ./Draws/Metrics/open_interest_curve.png");
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/open_interest_curve.html");

    Ok(())
}
