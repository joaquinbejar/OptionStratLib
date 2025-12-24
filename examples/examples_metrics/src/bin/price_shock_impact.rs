/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/

//! # Price Shock Impact Example
//!
//! This example demonstrates how to generate and visualize price shock
//! impact curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/price_shock_curve.png`
//! - PNG image: `./Draws/Metrics/price_shock_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{PriceShockCurve, PriceShockSurface};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,
        spos!(5.0),
        dec!(-0.15),
        dec!(0.08),
        pos!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos!(450.0))),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos!(0.20),
    );

    let option_chain = OptionChain::build_chain(&params);

    tracing::info!(
        "Built option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );
    tracing::info!("Underlying price: {}", option_chain.underlying_price);

    // ========================================
    // Price Shock Curve (-10% shock)
    // ========================================
    tracing::info!("Generating Price Shock Curve (-10% shock)...");

    let shock_pct = dec!(-0.10);
    let shock_curve = option_chain.price_shock_curve(shock_pct)?;

    tracing::info!(
        "Price Shock Curve generated with {} points",
        shock_curve.points.len()
    );

    // Analyze P&L distribution
    let points: Vec<_> = shock_curve.points.iter().collect();
    let total_pnl: rust_decimal::Decimal = points.iter().map(|p| p.y).sum();
    tracing::info!(
        "Total P&L from -10% shock across all strikes: {:.2}",
        total_pnl
    );

    // Plot curve
    shock_curve
        .plot()
        .title("Price Shock Impact Curve (-10% Shock, SPY)")
        .x_label("Strike Price")
        .y_label("P&L")
        .save("./Draws/Metrics/price_shock_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    shock_curve
        .write_html("./Draws/Metrics/price_shock_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Price shock curve saved to ./Draws/Metrics/price_shock_curve.png");

    // ========================================
    // Price Shock Surface (Price vs Volatility)
    // ========================================
    tracing::info!("Generating Price Shock Surface (Scenario Analysis)...");

    let price_range = (pos!(380.0), pos!(520.0));
    let vol_range = (pos!(0.10), pos!(0.50));

    let shock_surface = option_chain
        .price_shock_surface(price_range, vol_range, 20, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Price Shock Surface generated with {} points",
        shock_surface.points.len()
    );

    // Plot surface
    shock_surface
        .plot()
        .title("Price Shock Surface - Scenario Analysis (SPY)")
        .x_label("Underlying Price")
        .y_label("Implied Volatility")
        .z_label("Option Value")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/price_shock_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    shock_surface
        .write_html("./Draws/Metrics/price_shock_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Price shock surface saved to ./Draws/Metrics/price_shock_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
