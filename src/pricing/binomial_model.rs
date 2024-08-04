use crate::model::option::Side;
use crate::model::types::OptionType;
use crate::pricing::payoff::Payoff;

pub struct BinomialPricingParams<'a> {
    pub asset: f64,
    pub volatility: f64,
    pub int_rate: f64,
    pub strike: f64,
    pub expiry: f64,
    pub no_steps: usize,
    pub option_type: &'a OptionType,
    pub side: &'a Side,
}

pub fn price_binomial(params: BinomialPricingParams) -> f64 {
    let time_step = params.expiry / params.no_steps as f64;
    let discount_factor = (-params.int_rate * time_step).exp();
    let temp1 = ((params.int_rate - params.volatility * params.volatility / 2.0) * time_step).exp();
    let u = temp1 * (params.volatility * time_step.sqrt()).exp();
    let d = temp1 / (params.volatility * time_step.sqrt()).exp();
    let p = (discount_factor - d) / (u - d);
    let mut s = vec![0.0; params.no_steps + 1];
    let mut v = vec![0.0; params.no_steps + 1];
    s[0] = params.asset;
    for n in 1..=params.no_steps {
        for j in (1..=n).rev() {
            s[j] = u * s[j - 1];
        }
        s[0] *= d;
    }
    for j in 0..=params.no_steps {
        v[j] = params.option_type.payoff(s[j], params.strike);
    }
    for n in (1..=params.no_steps).rev() {
        for j in 0..n {
            v[j] = (p * v[j + 1] + (1.0 - p) * v[j]) * discount_factor;
        }
    }
    match params.side {
        Side::Long => v[0],
        Side::Short => -v[0],
    }
}
