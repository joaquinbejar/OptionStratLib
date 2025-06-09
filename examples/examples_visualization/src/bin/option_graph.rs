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
use optionstratlib::visualization::Graph;
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
    let option = create_sample_option();
    info!("Title: {}", option.get_title());
    info!("Greeks: {:?}", option.greeks());

    let path: &std::path::Path = "Draws/Options/intrinsic_value_chart.png".as_ref();
    option.write_png(path)?;
    let path_html: &std::path::Path = "Draws/Options/intrinsic_value_chart.html".as_ref();
    option.write_html(path_html)?;
    Ok(())
}
