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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let hours = 23.0;
    let n_steps = 60 * hours as usize;
    let initial_price = pos!(5781.88);
    let mean = 0.0;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.1);
    let risk_free_rate = Some(Decimal::ZERO);
    let dividend_yield = spos!(0.02);
    let volatility_window = 10;
    let initial_volatility = Some(std_dev);

    let mut random_walk = RandomWalkGraph::new(
        "Random Walk Iter".to_string(),
        risk_free_rate,
        dividend_yield,
        TimeFrame::Minute,
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
            file_path: "Draws/Simulation/random_walk_iter.png",
            size: (1200, 800),
        },
        20,
    );

    random_walk
        .enumerate()
        .for_each(|(i, params)| println!("{} {}", i, params));
    Ok(())
}
