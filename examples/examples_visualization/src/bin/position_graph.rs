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
fn main() -> Result<(), Error> {
    let position = Position::new(
        create_sample_option(),
        pos!(5.71),
        Utc::now(),
        Positive::ONE,
        Positive::ONE,
        None,
        None,
    );

    let path: &std::path::Path = "Draws/Position/pnl_at_expiration_chart.png".as_ref();
    position.write_png(path)?;

    Ok(())
}
