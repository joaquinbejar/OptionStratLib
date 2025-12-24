/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/
//! Example demonstrating extended delta adjustment functionality for LongStraddle.
//!
//! A Long Straddle profits from large price movements in either direction.
//! This example shows portfolio-level Greeks and adjustment planning.

use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos!(7138.5);

    let strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price,
        pos!(7140.0),
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),
        dec!(0.05),
        Positive::ZERO,
        pos!(1.0),
        pos!(85.04),
        pos!(85.04),
        pos!(0.78),
        pos!(0.78),
        pos!(0.73),
        pos!(0.73),
    );

    info!("=== LongStraddle Extended Delta Analysis ===");
    info!("Title: {}", strategy.get_title());

    // Basic strategy info
    info!("\n--- Basic Strategy Info ---");
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Max Profit: ${:.2}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${:.2}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );

    // Original delta neutrality
    info!("\n--- Original Delta Neutrality ---");
    info!("Delta Neutral: {}", strategy.is_delta_neutral());
    let delta_info = strategy.delta_neutrality()?;
    info!("Net Delta: {:.4}", delta_info.net_delta);
    info!("Individual Deltas:");
    for pos_info in &delta_info.individual_deltas {
        info!(
            "  Strike {}: delta={:.4}, style={:?}, side={:?}",
            pos_info.strike, pos_info.delta, pos_info.option_style, pos_info.side
        );
    }

    // Portfolio-level Greeks
    info!("\n--- Portfolio Greeks (NEW) ---");
    match strategy.portfolio_greeks() {
        Ok(greeks) => {
            info!("Portfolio Delta: {:.4}", greeks.delta);
            info!("Portfolio Gamma: {:.6}", greeks.gamma);
            info!("Portfolio Theta: {:.4}", greeks.theta);
            info!("Portfolio Vega: {:.4}", greeks.vega);
            info!("Portfolio Rho: {:.4}", greeks.rho);
            info!(
                "Is Delta Neutral (tol=0.1): {}",
                greeks.is_delta_neutral(dec!(0.1))
            );
        }
        Err(e) => warn!("Could not calculate portfolio Greeks: {}", e),
    }

    // Greek targets
    info!("\n--- Greek Targets Check (NEW) ---");
    let delta_target = AdjustmentTarget::delta_neutral();
    info!(
        "Meets delta neutral target (tol=0.1): {}",
        strategy.meets_greek_targets(&delta_target, dec!(0.1))
    );

    // Optimized adjustment plan
    info!("\n--- Optimized Adjustment Plan (NEW) ---");
    let config = AdjustmentConfig::default()
        .with_allow_new_legs(false)
        .with_allow_underlying(true);

    match strategy.optimized_adjustment_plan(config, AdjustmentTarget::delta_neutral()) {
        Ok(plan) => {
            info!("Adjustment Plan:");
            for action in &plan.actions {
                info!("    - {}", action);
            }
            info!("  Residual Delta: {:.4}", plan.residual_delta);
        }
        Err(e) => info!("No adjustment needed or available: {}", e),
    }

    info!("\n=== Extended Delta Analysis Complete ===");
    Ok(())
}
