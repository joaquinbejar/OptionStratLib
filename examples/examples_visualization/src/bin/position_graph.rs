use positive::pos_or_panic;
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
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(105.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Error> {
    let position = Position::new(
        create_sample_option(),
        pos_or_panic!(5.71),
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
