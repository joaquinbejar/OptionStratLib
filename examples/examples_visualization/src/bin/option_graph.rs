/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::f2p;
use optionstratlib::greeks::equations::Greeks;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::Options;
use optionstratlib::Positive;
use std::error::Error;
use tracing::info;

fn create_sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        f2p!(100.0),
        ExpirationDate::Days(30.0),
        0.2,
        f2p!(1.0),
        f2p!(105.0),
        0.05,
        OptionStyle::Call,
        0.0,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let option = create_sample_option();
    info!("Title: {}", option.title());
    info!("Greeks: {:?}", option.greeks());

    // Define a range of prices for the graph
    let price_range: Vec<Positive> = (50..150)
        .map(|x| Positive::new(x as f64).unwrap())
        .collect();

    // Generate the intrinsic value graph
    option.graph(
        &price_range,
        "Draws/Options/intrinsic_value_chart.png",
        25,
        (1400, 933),
    )?;

    Ok(())
}
