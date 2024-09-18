/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};

#[allow(dead_code)]
pub(crate) fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: f64,
    quantity: u32,
    strike_price: f64,
    volatility: f64,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::Days(30.0),
        volatility,
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
