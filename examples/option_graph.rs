/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::option::Options;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::visualization::utils::Graph;
use std::error::Error;

fn create_sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        100.0,
        ExpirationDate::Days(30.0),
        0.2,
        1,
        105.0,
        0.05,
        OptionStyle::Call,
        0.0,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let option = create_sample_option();
    println!("Title: {}", option.title());
    println!("Greeks: {:?}", option.greeks());

    // Define a range of prices for the graph
    let price_range: Vec<f64> = (50..150).map(|x| x as f64).collect();

    // Generate the intrinsic value graph
    option.graph(
        &price_range,
        "Draws/Options/intrinsic_value_chart.png",
        25,
        (1400, 933),
        (10, 30),
        10,
    )?;

    Ok(())
}
