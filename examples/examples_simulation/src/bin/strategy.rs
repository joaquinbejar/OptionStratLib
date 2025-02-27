/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 27/2/25
 ******************************************************************************/

use optionstratlib::simulation::{SimulationConfig, Simulator, WalkId};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::{pos, spos, ExpirationDate, Positive};
use rust_decimal::Decimal;
use std::collections::HashMap;
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::strategies::{ShortStrangle, Strategies};
use optionstratlib::visualization::utils::{Graph, GraphBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let symbol = "CL";
    let initial_price = pos!(7250.0);
    
    let days = pos!(45.0);

    // Setup simulation parameters
    let n_steps = (24.0 * days) as usize;
    let mean = 0.0;
    let std_dev = pos!(11.3);
    let std_dev_change = pos!(0.1);

    // Create simulation config
    let config = SimulationConfig {
        risk_free_rate: Some(Decimal::ZERO),
        dividend_yield: spos!(0.02),
        time_frame: TimeFrame::Hour,
        volatility_window: 20,
        initial_volatility: Some(std_dev),
    };

    // Initialize simulator
    let mut simulator = Simulator::new(config);
    let mut initial_prices = HashMap::new();

    for i in 0..100 {
        let asset_id = WalkId::new(format!("{}_{:02}", symbol, i));
        simulator.add_walk(asset_id.as_str(), format!("{} {:02}",symbol, i));
        initial_prices.insert(asset_id, initial_price);
    }

    // Generate correlated walks
    simulator.generate_random_walks(n_steps, &initial_prices, mean, std_dev, std_dev_change)?;

    let strategy = ShortStrangle::new(
        symbol.to_string(),
        pos!(7250.0), // underlying_price
        initial_price,     // call_strike
        pos!(7050.0),     // put_strike
        ExpirationDate::Days(days),
        pos!(0.3745),   // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(2.0),      // quantity
        pos!(84.2),     // premium_short_call
        pos!(353.2),    // premium_short_put
        pos!(7.01),     // open_fee_short_call
        pos!(7.01),     // close_fee_short_call
        pos!(7.01),     // open_fee_short_put
        pos!(7.01),     // close_fee_short_put
    );
    
    let simulation_result = simulator.simulate_strategy(&strategy)?;
    
    info!("Simulation result: {:?}", simulation_result);

    simulator.graph(
        GraphBackend::Bitmap {
            file_path: &"Draws/Simulation/strategy.png",
            size: (1200, 800),
        },
        20,
    )?;
    
    // 
    // let surface = simulator.surface()?;
    // 
    // surface
    //     .plot()
    //     .title("Simulated Surface")
    //     .x_label("Walk")
    //     .y_label("Time")
    //     .z_label("Price")
    //     .save("Draws/Simulation/surface.png")?;

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
