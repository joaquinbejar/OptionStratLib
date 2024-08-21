/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use chrono::{NaiveDate, TimeZone, Utc};

#[allow(dead_code)]
pub(crate) fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: f64,
    quantity: u32,
) -> Options {
    let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
        .expect("Invalid date")
        .and_hms_opt(0, 0, 0)
        .expect("Invalid time");
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        100.0,
        ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
        0.2,
        quantity,
        underlying_price,
        0.05,
        option_style,
        0.01,
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn create_sample_option_simplest(option_style: OptionStyle, side: Side) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        100.0,
        ExpirationDate::Days(30.0),
        0.2,
        1,
        100.0,
        0.05,
        option_style,
        0.01,
        None,
    )
}
