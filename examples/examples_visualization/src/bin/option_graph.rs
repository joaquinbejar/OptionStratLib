/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use optionstratlib::prelude::*;

fn create_sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "AAPL".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        pos_or_panic!(1.0),
        pos_or_panic!(105.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Error> {
    let option = create_sample_option();
    info!("Title: {}", option.get_title());
    info!("Greeks: {:?}", option.greeks());

    let path: &std::path::Path = "Draws/Options/intrinsic_value_chart.png".as_ref();
    option.write_png(path)?;
    let path_html: &std::path::Path = "Draws/Options/intrinsic_value_chart.html".as_ref();
    option.write_html(path_html)?;
    Ok(())
}
