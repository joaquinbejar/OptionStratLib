/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/8/24
******************************************************************************/
use crate::model::types::Side;
use crate::pricing::binomial_model::BinomialPricingParams;
use crate::pricing::constants::{CLAMP_MAX, CLAMP_MIN};
use crate::pricing::payoff::Payoff;

pub(crate) fn calculate_up_factor(volatility: f64, dt: f64) -> f64 {
    (volatility * dt.sqrt()).exp()
}

pub(crate) fn calculate_down_factor(up_factor: f64) -> f64 {
    1.0 / up_factor
}

pub(crate) fn calculate_probability(
    int_rate: f64,
    dt: f64,
    down_factor: f64,
    up_factor: f64,
) -> f64 {
    (((int_rate * dt).exp() - down_factor) / (up_factor - down_factor)).clamp(CLAMP_MIN, CLAMP_MAX)
}

pub(crate) fn calculate_discount_factor(int_rate: f64, dt: f64) -> f64 {
    (-int_rate * dt).exp()
}

pub(crate) fn calculate_payoff(params: BinomialPricingParams) -> f64 {
    let payoff = params
        .option_type
        .payoff(params.asset, params.strike, params.option_style);
    match params.side {
        Side::Long => payoff,
        Side::Short => -payoff,
    }
}

pub(crate) fn calculate_discounted_payoff(params: BinomialPricingParams) -> f64 {
    let future_asset_price = params.asset * (params.int_rate * params.expiry).exp();
    let discounted_payoff = (-params.int_rate * params.expiry).exp()
        * params
            .option_type
            .payoff(future_asset_price, params.strike, params.option_style);
    match params.side {
        Side::Long => discounted_payoff,
        Side::Short => -discounted_payoff,
    }
}

pub(crate) fn calculate_option_price(
    params: BinomialPricingParams,
    u: f64,
    d: f64,
    i: usize,
) -> f64 {
    let price = params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32);
    params
        .option_type
        .payoff(price, params.strike, params.option_style)
}

pub(crate) fn calculate_discounted_value(
    p: f64,
    price_up: f64,
    price_down: f64,
    int_rate: f64,
    dt: f64,
) -> f64 {
    (p * price_up + (1.0 - p) * price_down) * (-int_rate * dt).exp()
}
