/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/2/25
******************************************************************************/

use optionstratlib::geometrics::Plottable;
use optionstratlib::simulation::{SimulationConfig, Simulator, WalkId};
use optionstratlib::strategies::{ShortStrangle, Strategies};
use optionstratlib::surfaces::Surfacable;
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{ExpirationDate, Positive, pos, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let symbol = "SP500";
    let initial_price = pos!(6032.18);

    let days = pos!(2.0);

    // Setup simulation parameters
    let n_steps = (24.0 * 60.0 * days) as usize;
    let mu = 0.0;
    let volatility = pos!(0.2);
    let vov = pos!(0.01);

    // Create simulation config
    let config = SimulationConfig {
        risk_free_rate: Some(Decimal::ZERO),
        dividend_yield: spos!(0.0),
        time_frame: TimeFrame::Hour,
        volatility_window: 20,
        initial_volatility: Some(volatility),
    };

    // Initialize simulator
    let mut simulator = Simulator::new(&config);
    let mut initial_prices = HashMap::new();

    for i in 0..100 {
        let asset_id = WalkId::new(format!("{}_{:02}", symbol, i));
        simulator.add_walk(asset_id.as_str(), format!("{} {:02}", symbol, i));
        initial_prices.insert(asset_id, initial_price);
    }

    // Generate correlated walks
    simulator.generate_random_walks(n_steps, &initial_prices, mu, volatility, vov, config.time_frame, None)?;

    let mut strategy = ShortStrangle::new(
        symbol.to_string(),
        initial_price, // underlying_price
        pos!(6300.0),  // call_strike
        pos!(5725.0),  // put_strike
        ExpirationDate::Days(days),
        pos!(0.2),      // implied_volatility
        dec!(0.0),      // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // quantity
        pos!(22.8),     // premium_short_call
        pos!(48.69),    // premium_short_put
        pos!(2.41),     // open_fee_short_call
        pos!(2.41),     // close_fee_short_call
        pos!(2.41),     // open_fee_short_put
        pos!(2.41),     // close_fee_short_put
    );

    let simulation_result = simulator.simulate_strategy(&mut strategy)?;

    info!(
        "Simulation result iterations: {:?}",
        simulation_result.iterations
    );
    info!(
        "Simulation result profit_probability: {:?}",
        simulation_result.profit_probability
    );
    info!(
        "Simulation result loss_probability: {:?}",
        simulation_result.loss_probability
    );
    info!(
        "Simulation result max_profit: {:?}",
        simulation_result.max_profit
    );
    info!(
        "Simulation result max_loss: {:?}",
        simulation_result.max_loss
    );
    info!(
        "Simulation result average_pnl: {:?}",
        simulation_result.average_pnl
    );
    info!(
        "Simulation result pnl_std_dev: {:?}",
        simulation_result.pnl_std_dev
    );
    info!(
        "Simulation result risk_levels: {}",
        serde_json::to_string_pretty(&simulation_result.risk_levels).unwrap()
    );
    info!(
        "Simulation result pnl_distribution: {}",
        serde_json::to_string_pretty(&simulation_result.pnl_distribution).unwrap()
    );
    info!(
        "Simulation result additional_metrics: {}",
        serde_json::to_string_pretty(&simulation_result.additional_metrics).unwrap()
    );

    simulator.graph(
        GraphBackend::Bitmap {
            file_path: &"Draws/Simulation/simulator_strategy.png",
            size: (1200, 800),
        },
        20,
    )?;

    let surface = simulator.surface()?;

    surface
        .plot()
        .title("Simulated Surface")
        .x_label("Walk")
        .y_label("Time")
        .z_label("Price")
        .save("Draws/Simulation/simulator_strategy_surface.png")?;

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${}",
        strategy.max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    Ok(())
}
