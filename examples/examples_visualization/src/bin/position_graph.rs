/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use chrono::Utc;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::pos;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::Options;
use optionstratlib::Positive;
use rust_decimal_macros::dec;
use std::error::Error;

fn create_sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        pos!(100.0),
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.2),
        pos!(1.0),
        pos!(105.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let position = Position::new(
        create_sample_option(),
        pos!(5.71),
        Utc::now(),
        Positive::ONE,
        Positive::ONE,
    );
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
