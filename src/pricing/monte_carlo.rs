use crate::model::option::Options;
use crate::pricing::utils::wiener_increment;

/// This function performs Monte Carlo simulation to price an option.
///
/// # Arguments
///
/// * `option` - An `Options` struct containing the option's parameters, such as underlying price, strike price, risk-free rate, implied volatility, and expiration date.
/// * `steps` - An integer indicating the number of time steps in the simulation.
/// * `simulations` - An integer indicating the number of Monte Carlo simulations to run.
///
/// # Returns
///
/// * A floating-point number representing the estimated price of the option.
///
/// # Description
///
/// The function follows the below steps:
///
/// 1. Calculate the time increment `dt` based on the number of steps.
/// 2. Initialize a sum variable `payoff_sum` to accumulate the payoffs from each simulation.
/// 3. Loop through the number of simulations:
///     - For each simulation, initialize the stock price `st` to the underlying price.
///     - Loop through the number of steps:
///         - Calculate a Wiener process increment `w`.
///         - Update the stock price `st` using the discrete approximation of the geometric Brownian motion model.
///     - Calculate the payoff of the option for this simulation (for a call option, this is `max(st - strike_price, 0)`).
///     - Add the payoff to the `payoff_sum`.
/// 4. Return the average payoff discounted to its present value.
#[allow(dead_code)]
fn monte_carlo_option_pricing(
    option: &Options,
    steps: usize,       // Number of time steps
    simulations: usize, // Number of Monte Carlo simulations
) -> f64 {
    let dt = option.expiration_date.get_years() / steps as f64;
    let mut payoff_sum = 0.0;
    for _ in 0..simulations {
        let mut st = option.underlying_price;
        for _ in 0..steps {
            let w = wiener_increment(dt);
            st *= 1.0 + option.risk_free_rate * dt + option.implied_volatility * w;
        }
        // Calculate the payoff for a call option
        let payoff = f64::max(st - option.strike_price, 0.0);
        payoff_sum += payoff;
    }
    // Average value of the payoffs discounted to present value
    (payoff_sum / simulations as f64)
        * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;

    fn create_test_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: 100.0,
            expiration_date: ExpirationDate::Days(365.0), // 1 year
            implied_volatility: 0.2,
            quantity: 1,
            underlying_price: 100.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.0,
            exotic_params: None,
        }
    }

    #[test]
    fn test_monte_carlo_option_pricing_at_the_money() {
        let option = create_test_option();
        let price = monte_carlo_option_pricing(&option, 252, 10000);
        // The price should be close to the Black-Scholes price for these parameters
        let expected_price = 10.45; // Calculated using Black-Scholes
        assert_relative_eq!(price, expected_price, epsilon = 0.5);
    }

    #[test]
    fn test_monte_carlo_option_pricing_out_of_the_money() {
        let mut option = create_test_option();
        option.strike_price = 120.0;
        let price = monte_carlo_option_pricing(&option, 252, 10000);
        // The price should be lower for an out-of-the-money option
        assert!(price < 5.0);
    }

    #[test]
    fn test_monte_carlo_option_pricing_in_the_money() {
        let mut option = create_test_option();
        option.strike_price = 80.0;
        let price = monte_carlo_option_pricing(&option, 252, 10000);
        // The price should be higher for an in-the-money option
        assert!(price > 20.0);
    }

    #[test]
    fn test_monte_carlo_option_pricing_zero_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = 0.0;
        let price = monte_carlo_option_pricing(&option, 252, 10000);
        let expected_price = f64::max(
            option.underlying_price
                - option.strike_price * (-option.risk_free_rate * 1.0).exp(),
            0.0,
        );
        assert_relative_eq!(price, expected_price, epsilon = 0.1);
    }

    #[test]
    fn test_monte_carlo_option_pricing_high_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = 0.5;
        let price = monte_carlo_option_pricing(&option, 252, 10000);
        // The price should be higher with higher volatility
        assert!(price > 15.0);
    }

    #[test]
    fn test_monte_carlo_option_pricing_short_expiration() {
        let mut option = create_test_option();
        option.expiration_date = ExpirationDate::Days(30.0); // 30 days
        let price = monte_carlo_option_pricing(&option, 30, 10000);
        // The price should be lower for a shorter expiration
        assert!(price < 5.0);
    }

    #[test]
    fn test_monte_carlo_option_pricing_long_expiration() {
        let mut option = create_test_option();
        option.expiration_date = ExpirationDate::Days(730.0); // 2 years
        let price = monte_carlo_option_pricing(&option, 504, 10000);
        // The price should be higher for a longer expiration
        assert!(price > 15.0);
    }

    #[test]
    fn test_monte_carlo_option_pricing_consistency() {
        let option = create_test_option();
        let price1 = monte_carlo_option_pricing(&option, 252, 10000);
        let price2 = monte_carlo_option_pricing(&option, 252, 10000);
        // Two runs should produce similar results
        assert_relative_eq!(price1, price2, epsilon = 0.5);
    }
}
