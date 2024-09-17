/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/8/24
******************************************************************************/

use statrs::distribution::{ContinuousCDF, Normal};
use std::f64::consts::PI;

pub(crate) fn big_n(x: f64) -> f64 {
    const MEAN: f64 = 0.0;
    const STD_DEV: f64 = 1.0;

    let normal_distribution = Normal::new(MEAN, STD_DEV).unwrap();
    normal_distribution.cdf(x)
}

pub fn black_scholes(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let d1 = ((s / k).ln() + (r + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
    let d2 = d1 - sigma * t.sqrt();

    s * big_n(d1) - k * (-r * t).exp() * big_n(d2)
}

fn vega(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let d1 = ((s / k).ln() + (r + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
    s * t.sqrt() * (-(d1 * d1 / 2.0)).exp() / (2.0 * PI).sqrt()
}

fn implied_volatility(
    s: f64,
    k: f64,
    t: f64,
    r: f64,
    market_price: f64,
    epsilon: f64,
    max_iterations: usize,
) -> Option<f64> {
    let mut sigma = 0.5; // Initial guess
    let mut iteration = 0;

    while iteration < max_iterations {
        let price = black_scholes(s, k, t, r, sigma);
        let diff = price - market_price;

        if diff.abs() < epsilon {
            return Some(sigma);
        }

        let v = vega(s, k, t, r, sigma);
        sigma -= diff / v;

        iteration += 1;
    }

    None // No convergence
}

fn main() {
    let s = 2476.6;
    let k = 2530.0;
    let t = 0.0070;
    let r = 0.0733;
    let market_price = 4.4;
    let epsilon = 0.00001;
    let max_iterations = 1000;

    match implied_volatility(s, k, t, r, market_price, epsilon, max_iterations) {
        Some(iv) => info!("The implied volatility is: {:.4}", iv * 100.0),
        None => info!("Could not converge to a solution"),
    }
}
