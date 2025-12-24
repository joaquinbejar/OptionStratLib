/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/
//! Example demonstrating extended delta adjustment functionality for PoorMansCoveredCall.
//!
//! A Poor Man's Covered Call is a bullish strategy using LEAPS.
//! This example shows portfolio-level Greeks and adjustment planning.

use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let underlying_price = pos!(5780.0);

    let strategy = PoorMansCoveredCall::new(
        "SP500".to_string(),
        underlying_price,
        pos!(5500.0),
        pos!(5900.0),
        ExpirationDate::Days(pos!(120.0)),
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.18),
        dec!(0.05),
        Positive::ZERO,
        pos!(1.0),
        pos!(320.0),
        pos!(45.0),
        pos!(0.78),
        pos!(0.78),
        pos!(0.73),
        pos!(0.73),
    );

    info!("=== PoorMansCoveredCall Extended Delta Analysis ===");
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
            info!(
                "Is Delta Neutral (tol=0.1): {}",
                greeks.is_delta_neutral(dec!(0.1))
            );
        }
        Err(e) => warn!("Could not calculate portfolio Greeks: {}", e),
    }

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
