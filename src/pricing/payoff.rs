pub trait Payoff {
    fn payoff(&self, spot: f64, strike: f64) -> f64;
}
