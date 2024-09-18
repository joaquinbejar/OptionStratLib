use optionstratlib::greeks::utils::big_n;
use optionstratlib::utils::logger::setup_logger;
use tracing::info;

fn black_scholes_call(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let d1 = (s.ln() / k + (r + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
    let d2 = d1 - sigma * t.sqrt();

    s * big_n(d1) - k * (-r * t).exp() * big_n(d2)
}

fn implied_risk_free_rate(
    s: f64,
    k: f64,
    t: f64,
    sigma: f64,
    market_price: f64,
    epsilon: f64,
    max_iterations: usize,
) -> Option<f64> {
    let mut r_low = -1.0;
    let mut r_high = 1.0;

    for _ in 0..max_iterations {
        let r_mid = (r_low + r_high) / 2.0;
        let price = black_scholes_call(s, k, t, r_mid, sigma);

        if (price - market_price).abs() < epsilon {
            return Some(r_mid);
        }

        if price > market_price {
            r_high = r_mid;
        } else {
            r_low = r_mid;
        }
    }

    None
}

fn main() {
    setup_logger();
    let s = 2476.6; // Current gold price
    let k = 2470.0; // Strike price
    let t = 3.0 / 365.0; // Time to expiration in years
    let sigma = 0.216875; // Volatility
    let market_price = 22.5; // Market price of the option
    let epsilon = 0.0001; // Desired precision
    let max_iterations = 1000;

    match implied_risk_free_rate(s, k, t, sigma, market_price, epsilon, max_iterations) {
        Some(r) => info!("The implied risk-free rate is: {:.4}%", r * 100.0),
        None => info!("Could not converge to a solution"),
    }

    // Test with the other set of parameters
    let k2 = 2390.0;
    let sigma2 = 0.199375;
    let market_price2 = 87.8;

    match implied_risk_free_rate(s, k2, t, sigma2, market_price2, epsilon, max_iterations) {
        Some(r) => info!(
            "The implied risk-free rate for the second set is: {:.4}%",
            r * 100.0
        ),
        None => info!("Could not converge to a solution for the second set"),
    }
}
