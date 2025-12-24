/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/
//! Example demonstrating extended delta adjustment functionality for BullCallSpread.
//!
//! This example shows how to use the new portfolio-level Greeks and
//! optimized adjustment planning features.

use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos!(5781.88);

    let strategy = BullCallSpread::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5750.0),
        pos!(5820.0),
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(2.0),
        pos!(85.04),
        pos!(29.85),
        pos!(0.78),
        pos!(0.78),
        pos!(0.73),
        pos!(0.73),
    );

    info!("=== BullCallSpread Extended Delta Analysis ===");
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

    // Original delta neutrality info
    info!("\n--- Original Delta Neutrality ---");
    info!("Delta Neutral: {}", strategy.is_delta_neutral());
    let delta_info = strategy.delta_neutrality()?;
    info!("Net Delta: {:.4}", delta_info.net_delta);
    info!("Individual Deltas:");
    for pos_info in &delta_info.individual_deltas {
        info!(
            "  Strike {}: delta={:.4}, qty={}, style={:?}, side={:?}",
            pos_info.strike,
            pos_info.delta,
            pos_info.quantity,
            pos_info.option_style,
            pos_info.side
        );
    }

    // NEW: Portfolio-level Greeks
    info!("\n--- Portfolio Greeks (NEW) ---");
    match strategy.portfolio_greeks() {
        Ok(greeks) => {
            info!("Portfolio Delta: {:.4}", greeks.delta);
            info!("Portfolio Gamma: {:.6}", greeks.gamma);
            info!("Portfolio Theta: {:.4}", greeks.theta);
            info!("Portfolio Vega: {:.4}", greeks.vega);
            info!("Portfolio Rho: {:.4}", greeks.rho);
            info!("Is Delta Neutral: {}", greeks.is_delta_neutral(dec!(0.1)));
            info!("Is Gamma Neutral: {}", greeks.is_gamma_neutral(dec!(0.01)));
            info!("Delta Gap from 0: {:.4}", greeks.delta_gap(Decimal::ZERO));
        }
        Err(e) => warn!("Could not calculate portfolio Greeks: {}", e),
    }

    // NEW: Check Greek targets
    info!("\n--- Greek Targets Check (NEW) ---");
    let delta_target = AdjustmentTarget::delta_neutral();
    let delta_gamma_target = AdjustmentTarget::delta_gamma_neutral();

    info!(
        "Meets delta neutral target (tol=0.1): {}",
        strategy.meets_greek_targets(&delta_target, dec!(0.1))
    );
    info!(
        "Meets delta-gamma neutral target (tol=0.1): {}",
        strategy.meets_greek_targets(&delta_gamma_target, dec!(0.1))
    );

    // NEW: Delta gap calculation
    info!("\n--- Delta Gap (NEW) ---");
    match strategy.delta_gap(Decimal::ZERO) {
        Ok(gap) => info!("Delta gap from neutral: {:.4}", gap),
        Err(e) => warn!("Could not calculate delta gap: {}", e),
    }

    // NEW: Optimized adjustment plan (existing legs only)
    info!("\n--- Optimized Adjustment Plan (NEW) ---");
    let config = AdjustmentConfig::default()
        .with_allow_new_legs(false)
        .with_allow_underlying(false);
    let target = AdjustmentTarget::delta_neutral();

    match strategy.optimized_adjustment_plan(config, target) {
        Ok(plan) => {
            info!("Adjustment Plan:");
            info!("  Actions: {}", plan.actions.len());
            for action in &plan.actions {
                info!("    - {}", action);
            }
            info!("  Estimated Cost: ${:.2}", plan.estimated_cost);
            info!("  Residual Delta: {:.4}", plan.residual_delta);
        }
        Err(e) => info!("No adjustment plan needed or available: {}", e),
    }

    // NEW: Optimized adjustment plan with underlying
    info!("\n--- Adjustment Plan with Underlying (NEW) ---");
    let config_with_underlying = AdjustmentConfig::default()
        .with_allow_new_legs(false)
        .with_allow_underlying(true);
    let target = AdjustmentTarget::delta_neutral();

    match strategy.optimized_adjustment_plan(config_with_underlying, target) {
        Ok(plan) => {
            info!("Adjustment Plan (with underlying):");
            info!("  Actions: {}", plan.actions.len());
            for action in &plan.actions {
                info!("    - {}", action);
            }
            info!("  Estimated Cost: ${:.2}", plan.estimated_cost);
            info!("  Residual Delta: {:.4}", plan.residual_delta);
        }
        Err(e) => info!("No adjustment plan available: {}", e),
    }

    // Original delta adjustments for comparison
    info!("\n--- Original Delta Adjustments (for comparison) ---");
    match strategy.delta_adjustments() {
        Ok(adjustments) => {
            for adj in adjustments {
                info!("  {}", adj);
            }
        }
        Err(e) => warn!("Could not get delta adjustments: {}", e),
    }

    info!("\n=== Extended Delta Analysis Complete ===");
    Ok(())
}
