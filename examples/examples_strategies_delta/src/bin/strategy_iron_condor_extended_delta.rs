/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
******************************************************************************/
//! Example demonstrating extended delta adjustment functionality for IronCondor.
//!
//! Iron Condor is a 4-leg strategy that benefits from low volatility.
//! This example shows portfolio-level Greeks and adjustment planning.

use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos!(2800.0);

    let strategy = IronCondor::new(
        "GOLD".to_string(),
        underlying_price,
        pos!(2725.0),
        pos!(2560.0),
        pos!(2800.0),
        pos!(2500.0),
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.1548),
        Decimal::ZERO,
        Positive::ZERO,
        pos!(2.0),
        pos!(38.8),
        pos!(30.4),
        pos!(23.3),
        pos!(16.8),
        pos!(0.96),
        pos!(0.96),
    );

    info!("=== IronCondor Extended Delta Analysis ===");
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
    info!(
        "Individual Deltas ({} legs):",
        delta_info.individual_deltas.len()
    );
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

    // Greek targets check
    info!("\n--- Greek Targets Check (NEW) ---");
    let targets = [
        ("Delta Neutral", AdjustmentTarget::delta_neutral()),
        (
            "Delta-Gamma Neutral",
            AdjustmentTarget::delta_gamma_neutral(),
        ),
        ("Full Neutral", AdjustmentTarget::full_neutral()),
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
        .with_allow_underlying(false)
        .with_delta_tolerance(dec!(0.05));
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

    info!("\n=== Extended Delta Analysis Complete ===");
    Ok(())
}
