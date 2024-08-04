use crate::model::types::OptionStyle;

pub trait Payoff {
    fn payoff(&self, spot: f64, strike: f64, x: &OptionStyle) -> f64;
}
