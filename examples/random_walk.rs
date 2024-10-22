/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 22/10/24
 ******************************************************************************/
use std::error::Error;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::visualization::utils::Graph;

pub fn visualize_random_walk(
    values: Vec<f64>,
    title: &str,
    file_path: &str,
    title_size: u32,
    canvas_size: (u32, u32),
) -> Result<(), Box<dyn Error>> {
    let x_axis_data: Vec<PositiveF64> = (0..values.len())
        .map(|i| pos!(i as f64))
        .collect();

    let graph = RandomWalkGraph::new(title.to_string());
    graph.graph(&x_axis_data, file_path, title_size, canvas_size)
}

fn main() {

    let n_steps = 1000;
    let initial_price = pos!(100.0);
    let mean = 0.0;
    let std_dev = pos!(1.0);
    let std_dev_change = pos!(0.1);

    let mut values:Vec<f64> = Vec::new();
    values.reserve(n_steps);

    let mut random_walk = RandomWalkGraph::new( "Random Walk".to_string());
    random_walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);

    visualize_random_walk(
        random_walk.get_values(&[]).clone(),
        "Random Walk",
        "Draws/random_walk.png",
        20,
        (800, 600),
    ).unwrap();

}