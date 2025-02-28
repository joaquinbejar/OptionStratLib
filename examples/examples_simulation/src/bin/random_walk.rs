/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{pos, spos};
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let years = 3.0;
    let n_steps = 252 * years as usize;
    let initial_price = pos!(100.0);
    let mean = 0.04;
    let std_dev = pos!(0.1);
    let std_dev_change = pos!(0.001);
    let risk_free_rate = Some(dec!(0.0));
    let dividend_yield = spos!(0.02);
    let volatility_window = 20;
    let initial_volatility = Some(std_dev);
    let mut random_walk = RandomWalkGraph::new(
        "Random Walk".to_string(),
        risk_free_rate,
        dividend_yield,
        TimeFrame::Day,
        volatility_window,
        initial_volatility,
    );
    random_walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)?;
    random_walk.graph(
        &random_walk.get_x_values(),
        GraphBackend::Bitmap {
            file_path: "Draws/Simulation/random_walk.png",
            size: (1200, 800),
        },
        20,
    )?;

    for (i, price_params) in random_walk.enumerate() {
        info!("Step {}: Params: {}", i, price_params,);
    }
    Ok(())
}
