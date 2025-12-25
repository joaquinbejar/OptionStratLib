/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # All Stress Metrics Example
//!
//! This comprehensive example demonstrates all stress metrics available in
//! OptionStratLib:
//! - Volatility Sensitivity Curve (by strike) and Surface (price vs volatility)
//! - Time Decay Profile Curve (by strike) and Surface (price vs time)
//! - Price Shock Impact Curve (by strike) and Surface (price vs volatility)
//!
//! ## Output
//! All graphs are saved to `./Draws/Metrics/`

use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{
    PriceShockCurve, PriceShockSurface, TimeDecayCurve, TimeDecaySurface,
    VolatilitySensitivityCurve, VolatilitySensitivitySurface,
};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;
use positive::pos_or_panic;

fn main() -> Result<(), CurveError> {
    setup_logger();

    tracing::info!("=== OptionStratLib Stress Metrics Demo ===\n");

    // Build a synthetic option chain
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),
        None,
        15,
        spos!(5.0),
        dec!(-0.15),
        dec!(0.08),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(pos_or_panic!(450.0))),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.01),
            Some("SPY".to_string()),
        ),
        pos_or_panic!(0.20),
    );

    let option_chain = OptionChain::build_chain(&params);

    tracing::info!("Option Chain Information:");
    tracing::info!("  Symbol: {}", option_chain.symbol);
    tracing::info!("  Underlying Price: {}", option_chain.underlying_price);
    tracing::info!("  Number of strikes: {}", option_chain.options.len());
    tracing::info!("");

    // ========================================
    // 1. Volatility Sensitivity Curve
    // ========================================
    tracing::info!("1. VOLATILITY SENSITIVITY CURVE (Vega)");
    tracing::info!("   Shows vega exposure by strike");

    let vega_curve = option_chain.volatility_sensitivity_curve()?;
    tracing::info!("   Generated {} curve points", vega_curve.points.len());

    vega_curve
        .plot()
        .title("Volatility Sensitivity Curve - Vega (SPY)")
        .x_label("Strike Price")
        .y_label("Vega")
        .save("./Draws/Metrics/volatility_sensitivity_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    vega_curve
        .write_html("./Draws/Metrics/volatility_sensitivity_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: volatility_sensitivity_curve.png/html\n");

    // ========================================
    // 2. Volatility Sensitivity Surface
    // ========================================
    tracing::info!("2. VOLATILITY SENSITIVITY SURFACE");
    tracing::info!("   Shows option value across price and volatility");

    let price_range = (pos_or_panic!(380.0), pos_or_panic!(520.0));
    let vol_range = (pos_or_panic!(0.10), pos_or_panic!(0.40));

    let vol_surface = option_chain
        .volatility_sensitivity_surface(price_range, vol_range, 20, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", vol_surface.points.len());

    vol_surface
        .plot()
        .title("Volatility Sensitivity Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Implied Volatility")
        .z_label("Option Value")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/volatility_sensitivity_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    vol_surface
        .write_html("./Draws/Metrics/volatility_sensitivity_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: volatility_sensitivity_surface.png/html\n");

    // ========================================
    // 3. Time Decay Curve
    // ========================================
    tracing::info!("3. TIME DECAY CURVE (Theta)");
    tracing::info!("   Shows theta exposure by strike");

    let theta_curve = option_chain.time_decay_curve()?;
    tracing::info!("   Generated {} curve points", theta_curve.points.len());

    theta_curve
        .plot()
        .title("Time Decay Curve - Theta (SPY)")
        .x_label("Strike Price")
        .y_label("Theta (Daily Decay)")
        .save("./Draws/Metrics/time_decay_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    theta_curve
        .write_html("./Draws/Metrics/time_decay_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: time_decay_curve.png/html\n");

    // ========================================
    // 4. Time Decay Surface
    // ========================================
    tracing::info!("4. TIME DECAY SURFACE");
    tracing::info!("   Shows option value across price and time");

    let days = vec![
        Positive::ONE,
        pos_or_panic!(7.0),
        pos_or_panic!(14.0),
        pos_or_panic!(21.0),
        pos_or_panic!(30.0),
        pos_or_panic!(45.0),
        pos_or_panic!(60.0),
    ];

    let decay_surface = option_chain
        .time_decay_surface(price_range, days, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", decay_surface.points.len());

    decay_surface
        .plot()
        .title("Time Decay Surface (SPY)")
        .x_label("Underlying Price")
        .y_label("Days to Expiration")
        .z_label("Option Value")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/time_decay_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    decay_surface
        .write_html("./Draws/Metrics/time_decay_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: time_decay_surface.png/html\n");

    // ========================================
    // 5. Price Shock Curve
    // ========================================
    tracing::info!("5. PRICE SHOCK CURVE");
    tracing::info!("   Shows P&L impact from -10% price shock");

    let shock_curve = option_chain.price_shock_curve(dec!(-0.10))?;
    tracing::info!("   Generated {} curve points", shock_curve.points.len());

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

    tracing::info!("   Saved: price_shock_curve.png/html\n");

    // ========================================
    // 6. Price Shock Surface
    // ========================================
    tracing::info!("6. PRICE SHOCK SURFACE");
    tracing::info!("   Shows option value across price and volatility scenarios");

    let vol_range_stress = (pos_or_panic!(0.10), pos_or_panic!(0.50));

    let shock_surface = option_chain
        .price_shock_surface(price_range, vol_range_stress, 20, 20)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Generated {} surface points", shock_surface.points.len());

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

    tracing::info!("   Saved: price_shock_surface.png/html\n");

    // ========================================
    // Summary
    // ========================================
    tracing::info!("=== Summary ===");
    tracing::info!("All stress metrics generated successfully!");
    tracing::info!("Output directory: ./Draws/Metrics/");
    tracing::info!("");
    tracing::info!("Files created:");
    tracing::info!("  - volatility_sensitivity_curve.png/html");
    tracing::info!("  - volatility_sensitivity_surface.png/html");
    tracing::info!("  - time_decay_curve.png/html");
    tracing::info!("  - time_decay_surface.png/html");
    tracing::info!("  - price_shock_curve.png/html");
    tracing::info!("  - price_shock_surface.png/html");

    Ok(())
}
