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
use rust_decimal::Decimal;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let hours = 23.0; // todo! raise a error if days < 1
    let n_steps = 60 * hours as usize;
    let initial_price = pos!(5781.88);
    let mean = 0.0;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.1);
    let risk_free_rate = Some(Decimal::ZERO);
    let dividend_yield = spos!(0.02);
    let volatility_window = 20;
    let initial_volatility = Some(std_dev);
    

    let mut random_walk = RandomWalkGraph::new(
        "Random Walk VoV".to_string(),
        risk_free_rate,
        dividend_yield,
        TimeFrame::Hour,
        volatility_window,
        initial_volatility,
    );
    random_walk.generate_random_walk_timeframe(
        n_steps,
        initial_price,
        mean,
        std_dev,
        std_dev_change,
        TimeFrame::Minute,
        None,
    )?;

    let _ = random_walk.graph(
        &random_walk.get_x_values(),
        GraphBackend::Bitmap {
            file_path: "Draws/Simulation/random_walk_vov.png",
            size: (1200, 800),
        },
        20,
    );

    let volatilities = random_walk.get_volatilities()?;
    for (i, price_params) in random_walk.enumerate() {
        info!(
            "Step {}: Vol: {} Params: {}",
            i, volatilities[i], price_params
        );
        // info!("Step {}: Params: {}", i, price_params);
    }
    Ok(())
}
