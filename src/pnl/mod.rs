/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use crate::model::option::Options;
use crate::model::types::{OptionStyle, Side};

pub fn pnl(option: &Options, current_price: f64) -> f64 {
    match (&option.side, &option.option_style) {
        (Side::Long, OptionStyle::Call) => calculate_long_call_pnl(option, current_price),
        (Side::Long, OptionStyle::Put) => calculate_long_put_pnl(option, current_price),
        (Side::Short, OptionStyle::Call) => calculate_short_call_pnl(option, current_price),
        (Side::Short, OptionStyle::Put) => calculate_short_put_pnl(option, current_price),
    }
}

fn calculate_long_call_pnl(option: &Options, current_price: f64) -> f64 {
    option.intrinsic_value(current_price) - option.premium
}

fn calculate_long_put_pnl(option: &Options, current_price: f64) -> f64 {
    let intrinsic_value = (option.strike_price - current_price).max(0.0);
    intrinsic_value - option.premium
}

fn calculate_short_call_pnl(option: &Options, current_price: f64) -> f64 {
    let intrinsic_value = (current_price - option.strike_price).max(0.0);
    option.premium - intrinsic_value
}

fn calculate_short_put_pnl(option: &Options, current_price: f64) -> f64 {
    let intrinsic_value = (option.strike_price - current_price).max(0.0);
    option.premium - intrinsic_value
}

pub fn calculate_pnl_at_expiration(option: &Options) -> f64 {
    pnl(option, option.underlying_price)
}
