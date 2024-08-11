/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use std::f64::consts::PI;
use statrs::distribution::{ContinuousCDF, Normal, Univariate};

pub(crate) fn d1(s: f64, r: f64, t: f64, sigma: f64) -> f64 {
    (s.ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * t.sqrt())
}

pub(crate) fn d2(s: f64, r: f64, t: f64, sigma: f64) -> f64 {
    d1(s, r, t, sigma) - sigma * t.sqrt()
}


// Helper function to calculate N(x)
pub(crate) fn n(x: f64) -> f64 {
    1.0 / (2.0 * PI).sqrt() * (-x * x / 2.0).exp()
}

// Helper function to calculate N'(x)
pub(crate) fn n_prime(x: f64) -> f64 {
    -x * n(x)
}

// Function to calculate N(x)
pub(crate) fn big_n(x: f64) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    normal.cdf(x)
}
