/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Smile Dynamics Example
//!
//! This example demonstrates how to generate and visualize smile dynamics
//! curves and surfaces from an option chain.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/smile_dynamics_curve.png`
//! - PNG image: `./Draws/Metrics/smile_dynamics_surface.png`
//! - HTML interactive files

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{SmileDynamicsCurve, SmileDynamicsSurface};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use positive::pos_or_panic;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain with pronounced smile
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,
        spos!(5.0),
        dec!(-0.20), // Strong negative skew
        dec!(0.12),  // Pronounced curvature
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(450.0))),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos_or_panic!(0.18), // 18% base IV
    );

    let option_chain = OptionChain::build_chain(&params).unwrap();

    tracing::info!(
        "Built option chain for {} with {} options",
        option_chain.symbol,
        option_chain.options.len()
    );

    // ========================================
    // Smile Dynamics Curve
    // ========================================
    tracing::info!("Generating Smile Dynamics Curve...");

    let smile_curve = option_chain.smile_dynamics_curve()?;

    tracing::info!(
        "Smile Dynamics Curve generated with {} points",
        smile_curve.points.len()
    );

    // Analyze smile shape
    let points: Vec<_> = smile_curve.points.iter().collect();
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        let atm_point = points
            .iter()
            .min_by(|a, b| {
                let a_dist = (a.x - dec!(450.0)).abs();
                let b_dist = (b.x - dec!(450.0)).abs();
                a_dist.partial_cmp(&b_dist).unwrap()
            })
            .unwrap();

        tracing::info!("Smile Analysis:");
        tracing::info!(
            "  OTM Put IV (strike {}): {:.2}%",
            first.x,
            first.y * dec!(100)
        );
        tracing::info!(
            "  ATM IV (strike {}): {:.2}%",
            atm_point.x,
            atm_point.y * dec!(100)
        );
        tracing::info!(
            "  OTM Call IV (strike {}): {:.2}%",
            last.x,
            last.y * dec!(100)
        );
    }

    // Plot curve
    smile_curve
        .plot()
        .title("Smile Dynamics Curve (SPY)")
        .x_label("Strike Price")
        .y_label("Implied Volatility")
        .save("./Draws/Metrics/smile_dynamics_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    smile_curve
        .write_html("./Draws/Metrics/smile_dynamics_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Smile dynamics curve saved to ./Draws/Metrics/smile_dynamics_curve.png");

    // ========================================
    // Smile Dynamics Surface
    // ========================================
    tracing::info!("Generating Smile Dynamics Surface...");

    let days = vec![
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
        pos_or_panic!(90.0),
    ];

    let smile_surface = option_chain
        .smile_dynamics_surface(days)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "Smile Dynamics Surface generated with {} points",
        smile_surface.points.len()
    );

    // Plot surface
    smile_surface
        .plot()
        .title("Smile Dynamics Surface (SPY)")
        .x_label("Strike Price")
        .y_label("Days to Expiration")
        .z_label("Implied Volatility")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/smile_dynamics_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    smile_surface
        .write_html("./Draws/Metrics/smile_dynamics_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("Smile dynamics surface saved to ./Draws/Metrics/smile_dynamics_surface.png");
    tracing::info!("Interactive HTML files saved to ./Draws/Metrics/");

    Ok(())
}
