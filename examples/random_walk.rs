/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let n_steps = 1000;
    let initial_price = pos!(100.0);
    let mean = 0.02;
    let std_dev = pos!(1.0);
    let std_dev_change = pos!(0.1);

    let mut random_walk = RandomWalkGraph::new("Random Walk".to_string());
    random_walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
    random_walk.graph(&[], "Draws/Simulation/random_walk.png", 20, (1200, 800))
}
