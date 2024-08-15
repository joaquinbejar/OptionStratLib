use crate::model::option::Options;
use crate::pricing::utils::wiener_increment;

#[allow(dead_code)]
fn monte_carlo_option_pricing(
    option: Options,
    steps: usize,       // Número de pasos en el tiempo
    simulations: usize, // Número de simulaciones de Monte Carlo
) -> f64 {
    let dt = option.expiration_date.get_years() / steps as f64;
    let mut payoff_sum = 0.0;

    for _ in 0..simulations {
        let mut st = option.underlying_price;

        for _ in 0..steps {
            let w = wiener_increment(dt);
            st *= 1.0 + option.risk_free_rate * dt + option.implied_volatility * w;
        }

        // Calculate the payoff for a option
        let payoff = f64::max(st - option.strike_price, 0.0);
        payoff_sum += payoff;
    }

    // Average value of the payoffs discounted to present value
    (payoff_sum / simulations as f64)
        * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
}
