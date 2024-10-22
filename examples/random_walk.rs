/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 22/10/24
 ******************************************************************************/
use std::error::Error;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;

pub fn visualize_random_walk(
    values: Vec<PositiveF64>,
    title: &str,
    file_path: &str,
    title_size: u32,
    canvas_size: (u32, u32),
) -> Result<(), Box<dyn Error>> {
    let x_axis_data: Vec<PositiveF64> = (0..values.len())
        .map(|i| pos!(i as f64))
        .collect();

    let graph = RandomWalkGraph::new(values, title.to_string());
    graph.graph(&x_axis_data, file_path, title_size, canvas_size)
}

fn main() {


}