/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 11/8/24
 ******************************************************************************/

use crate::greeks::utils::{big_n, d1, d2};
use crate::model::option::Options;
use crate::model::types::OptionStyle;

// Helper function to calculate d1 and d2 values
fn calculate_d_values(option: &Options, time_to_expiry: f64) -> (f64, f64) {
    let d1_value = d1(option.underlying_price, option.risk_free_rate, time_to_expiry, option.implied_volatility);
    let d2_value = d2(option.underlying_price, option.risk_free_rate, time_to_expiry, option.implied_volatility);
    (d1_value, d2_value)
}

pub fn black_scholes(
    option: Options,
    time_to_expiry: Option<f64>,  // Time until expiration in years
) -> f64 {
    let (d1_value, d2_value) = match time_to_expiry {
        None => {
            // calculate years from now until option.expiration_date (DateTime<Utc>)
            let now = chrono::Utc::now();
            let duration = option.expiration_date - now;
            let calculated_time_to_expiry = duration.num_days() as f64 / 365.0;
            calculate_d_values(&option, calculated_time_to_expiry)
        }
        Some(t_v) => calculate_d_values(&option, t_v),
    };

    match option.option_style {
        OptionStyle::Call => {
            option.underlying_price * big_n(d1_value) - option.strike_price * (-option.risk_free_rate * time_to_expiry.unwrap()).exp() * big_n(d2_value)
        }
        OptionStyle::Put => {
            option.strike_price * (-option.risk_free_rate * time_to_expiry.unwrap()).exp() * big_n(-d2_value) - option.underlying_price * big_n(-d1_value)
        }
    }
}
