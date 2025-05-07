/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::Positive;
use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pos;
use optionstratlib::strategies::BasicAble;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{ExpirationDate, Options};
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

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
    setup_logger();
    let option = create_sample_option();
    info!("Title: {}", option.get_title());
    info!("Greeks: {:?}", option.greeks());

    // Generate the intrinsic value graph
    option.graph(
        GraphBackend::Bitmap {
            file_path: "Draws/Options/intrinsic_value_chart.png",
            size: (1400, 933),
        },
        20,
    )?;

    Ok(())
}
