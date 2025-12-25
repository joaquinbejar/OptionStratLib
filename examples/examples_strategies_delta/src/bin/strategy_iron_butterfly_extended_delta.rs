/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/
//! Example demonstrating extended delta adjustment functionality for IronButterfly.
//!
//! An Iron Butterfly is a neutral strategy that profits from low volatility.
//! This example shows portfolio-level Greeks and adjustment planning.

use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos_or_panic!(5780.0);

    let strategy = IronButterfly::new(
        "SP500".to_string(),
        underlying_price,
        pos_or_panic!(5800.0),
        pos_or_panic!(5750.0),
        pos_or_panic!(5850.0),
        ExpirationDate::Days(pos_or_panic!(60.0)),
        pos_or_panic!(0.18),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(42.0),
        pos_or_panic!(38.0),
        pos_or_panic!(18.0),
        pos_or_panic!(16.0),
        pos_or_panic!(0.96),
        pos_or_panic!(0.96),
    );

    info!("=== IronButterfly Extended Delta Analysis ===");
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
