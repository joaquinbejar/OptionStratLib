/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::option::Options;
use crate::model::types::OptionStyle;

#[allow(dead_code)]
pub fn delta(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => (-option.dividend_yield * time_to_expiry).exp() * big_n(d1),
        OptionStyle::Put => (-option.dividend_yield * time_to_expiry).exp() * (big_n(d1) - 1.0),
    }
}

#[allow(dead_code)]
pub fn gamma(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    (-option.dividend_yield * time_to_expiry).exp() * n(d1)
        / (option.underlying_price * option.implied_volatility * time_to_expiry.sqrt())
}

#[allow(dead_code)]
pub fn theta(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );
    let d2 = d2(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    let common_term = -option.underlying_price
        * option.implied_volatility
        * (-option.dividend_yield * time_to_expiry).exp()
        * n(d1)
        / (2.0 * time_to_expiry.sqrt());

    match option.option_style {
        OptionStyle::Call => {
            common_term
                - option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * time_to_expiry).exp()
                    * big_n(d2)
                + option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * time_to_expiry).exp()
                    * big_n(d1)
        }
        OptionStyle::Put => {
            common_term
                + option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * time_to_expiry).exp()
                    * big_n(-d2)
                - option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * time_to_expiry).exp()
                    * big_n(-d1)
        }
    }
}

#[allow(dead_code)]
pub fn vega(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    option.underlying_price
        * (-option.dividend_yield * time_to_expiry).exp()
        * n(d1)
        * time_to_expiry.sqrt()
}

#[allow(dead_code)]
pub fn rho(option: &Options, time_to_expiry: f64) -> f64 {
    let d2 = d2(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            option.strike_price
                * time_to_expiry
                * (-option.risk_free_rate * time_to_expiry).exp()
                * big_n(d2)
        }
        OptionStyle::Put => {
            -option.strike_price
                * time_to_expiry
                * (-option.risk_free_rate * time_to_expiry).exp()
                * big_n(-d2)
        }
    }
}

#[allow(dead_code)]
pub fn rho_d(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            -time_to_expiry
                * option.underlying_price
                * (-option.dividend_yield * time_to_expiry).exp()
                * big_n(d1)
        }
        OptionStyle::Put => {
            time_to_expiry
                * option.underlying_price
                * (-option.dividend_yield * time_to_expiry).exp()
                * big_n(-d1)
        }
    }
}
