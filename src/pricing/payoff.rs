use crate::model::types::OptionStyle;

pub trait Payoff {
    fn payoff(&self, info: &PayoffInfo) -> f64;
}

pub struct PayoffInfo {
    pub spot: f64,
    pub strike: f64,
    pub style: OptionStyle,
    pub spot_prices: Option<Vec<f64>>,
    pub spot_min: Option<f64>,
    pub spot_max: Option<f64>,
}

impl PayoffInfo {
    pub fn spot_prices_len(&self) -> Option<usize> {
        self.spot_prices.as_ref().map(|vec| vec.len())
    }
}