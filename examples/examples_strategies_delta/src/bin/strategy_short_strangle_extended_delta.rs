/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/
//! Example demonstrating extended delta adjustment functionality for ShortStrangle.
//!
//! A Short Strangle profits from low volatility and time decay.
//! This example shows portfolio-level Greeks and adjustment planning.

use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos!(7138.5);

    let strategy = ShortStrangle::new(
        "CL".to_string(),
        underlying_price,
        pos!(7450.0),
        pos!(7250.0),
        ExpirationDate::Days(pos!(45.0)),
        pos!(0.3745),
        pos!(0.3745),
        dec!(0.05),
        Positive::ZERO,
        pos!(1.0),
        pos!(84.2),
        pos!(353.2),
        pos!(7.01),
        pos!(7.01),
        pos!(7.01),
        pos!(7.01),
    );

    info!("=== ShortStrangle Extended Delta Analysis ===");
    info!("Title: {}", strategy.get_title());

    // Basic strategy info
    info!("\n--- Basic Strategy Info ---");
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.get_net_premium_received()?
    );
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
            info!(
                "Is Gamma Neutral (tol=0.01): {}",
                greeks.is_gamma_neutral(dec!(0.01))
            );
        }
        Err(e) => warn!("Could not calculate portfolio Greeks: {}", e),
    }

    // Greek targets
    info!("\n--- Greek Targets Check (NEW) ---");
    let targets = [
        ("Delta Neutral", AdjustmentTarget::delta_neutral()),
        (
            "Delta-Gamma Neutral",
            AdjustmentTarget::delta_gamma_neutral(),
        ),
    ];

    for (name, target) in &targets {
        info!(
            "Meets {} target (tol=0.1): {}",
            name,
            strategy.meets_greek_targets(target, dec!(0.1))
        );
    }

    // Optimized adjustment plan
    info!("\n--- Optimized Adjustment Plan (NEW) ---");
    let config = AdjustmentConfig::default()
        .with_allow_new_legs(false)
        .with_allow_underlying(false);

    match strategy.optimized_adjustment_plan(config, AdjustmentTarget::delta_neutral()) {
        Ok(plan) => {
            info!("Adjustment Plan (existing legs only):");
            for action in &plan.actions {
                info!("    - {}", action);
            }
            info!("  Estimated Cost: ${:.2}", plan.estimated_cost);
            info!("  Residual Delta: {:.4}", plan.residual_delta);
        }
        Err(e) => info!("No adjustment needed or available: {}", e),
    }

    // Adjustment with underlying
    info!("\n--- Adjustment Plan with Underlying (NEW) ---");
    let config_underlying = AdjustmentConfig::default()
        .with_allow_new_legs(false)
        .with_allow_underlying(true);

    match strategy.optimized_adjustment_plan(config_underlying, AdjustmentTarget::delta_neutral()) {
        Ok(plan) => {
            info!("Adjustment Plan (with underlying):");
            for action in &plan.actions {
                info!("    - {}", action);
            }
            info!("  Residual Delta: {:.4}", plan.residual_delta);
        }
        Err(e) => info!("No adjustment available: {}", e),
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
