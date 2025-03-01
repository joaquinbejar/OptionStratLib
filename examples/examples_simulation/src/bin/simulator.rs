/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/02/25
******************************************************************************/
use optionstratlib::geometrics::Plottable;
use optionstratlib::simulation::{SimulationConfig, Simulator, WalkId};
use optionstratlib::surfaces::Surfacable;
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::visualization::utils::GraphBackend;
use optionstratlib::{pos, spos};
use rust_decimal::Decimal;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Setup simulation parameters
    let years = 3.0;
    let n_steps = 252 * years as usize;
    let mean = 0.0;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.01);

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
        let asset_id = WalkId::new(format!("SP500_{:02}", i));
        simulator.add_walk(asset_id.as_str(), format!("SP500 Index {:02}", i));
        initial_prices.insert(asset_id, pos!(5781.88));
    }

    // Generate correlated walks
    simulator.generate_random_walks(n_steps, &initial_prices, mean, std_dev, std_dev_change)?;

    simulator.graph(
        GraphBackend::Bitmap {
            file_path: &"Draws/Simulation/simulator.png",
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
        .save("Draws/Simulation/surface.png")?;

    Ok(())
}
