/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use chrono::Utc;
use optionstratlib::f2p;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::visualization::utils::Graph;
use optionstratlib::Options;
use optionstratlib::Positive;
use std::error::Error;

fn create_sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        f2p!(100.0),
        ExpirationDate::Days(30.0),
        0.2,
        f2p!(10.0),
        f2p!(105.0),
        0.05,
        OptionStyle::Call,
        0.0,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let position = Position::new(create_sample_option(), 5.71, Utc::now(), 1.0, 1.0);
    let price_range: Vec<Positive> = (50..150)
        .map(|x| Positive::new(x as f64).unwrap())
        .collect();

    position.graph(
        &price_range,
        "Draws/Position/pnl_at_expiration_chart.png",
        25,
        (1400, 933),
    )?;

    Ok(())
}
