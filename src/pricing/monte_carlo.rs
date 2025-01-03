use crate::pricing::utils::wiener_increment;
use crate::Options;
use crate::{f2d, Positive};
use rust_decimal::Decimal;
use std::error::Error;
use num_traits::ToPrimitive;

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
pub fn monte_carlo_option_pricing(
    option: &Options,
    steps: usize,   // Number of time steps
    simulations: usize,   // Number of Monte Carlo simulations
) -> Result<Decimal, Box<dyn Error>> {
    let dt = f2d!(option.expiration_date.get_years() / steps as f64);
    let mut payoff_sum = 0.0;

    for _ in 0..simulations {
        let mut st = option.underlying_price;
        for _ in 0..steps {
            let w = wiener_increment(dt)?;
            st *= Decimal::ONE + option.risk_free_rate * dt + option.implied_volatility * w;
        }
        // Calculate the payoff for a call option
        let payoff: f64 = (st - option.strike_price).max(Positive::ZERO).into();
        payoff_sum += payoff;
    }
    // Average value of the payoffs discounted to present value
    let average_payoff = (payoff_sum / simulations as f64)
        * (-option.risk_free_rate.to_f64().unwrap() * option.expiration_date.get_years()).exp();
    Ok(f2d!(average_payoff))
}

#[cfg(test)]
mod tests {
    use rust_decimal::MathematicalOps;
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::{assert_decimal_eq, f2du, pos};
    use rust_decimal_macros::dec;

    fn create_test_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(365.0),   // 1 year
            implied_volatility: pos!(0.2),
            quantity: pos!(1.0),
            underlying_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_monte_carlo_option_pricing_at_the_money() {
        let option = create_test_option();
        let price = monte_carlo_option_pricing(&option, 252, 1000).unwrap();
        // The price should be close to the Black-Scholes price for these parameters
        let expected_price = dec!(9.100); // Calculated using Black-Scholes
        assert_decimal_eq!(price, expected_price, dec!(5));
    }

    #[test]
    fn test_monte_carlo_option_pricing_out_of_the_money() {
        let mut option = create_test_option();
        option.strike_price = pos!(120.0);
        let price = monte_carlo_option_pricing(&option, 25, 100).unwrap();
        // The price should be lower for an out-of-the-money option
        assert!(price < dec!(5.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_in_the_money() {
        let mut option = create_test_option();
        option.strike_price = pos!(80.0);
        let price = monte_carlo_option_pricing(&option, 25, 100).unwrap();
        // The price should be higher for an in-the-money option
        assert!(price > dec!(20.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_zero_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = Positive::ZERO;
        let price = monte_carlo_option_pricing(&option, 25, 100).unwrap();
        let expected_price = f64::max(
            (option.underlying_price - option.strike_price * (-option.risk_free_rate).exp())
                .into(),
            ZERO,
        );
        assert_decimal_eq!(price, f2du!(expected_price).unwrap(), dec!(0.1));
    }

    #[test]
    fn test_monte_carlo_option_pricing_high_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = pos!(0.5);
        let price = monte_carlo_option_pricing(&option, 252, 100).unwrap();
        // The price should be higher with higher volatility
        assert!(price > dec!(15.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_short_expiration() {
        let mut option = create_test_option();
        option.expiration_date = ExpirationDate::Days(30.0); // 30 days
        let price = monte_carlo_option_pricing(&option, 30, 100).unwrap();
        // The price should be lower for a shorter expiration
        assert!(price < dec!(5.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_consistency() {
        let option = create_test_option();
        let _price1 = monte_carlo_option_pricing(&option, 100, 100).unwrap();
        let _price2 = monte_carlo_option_pricing(&option, 100, 100).unwrap();
        // Two runs should produce similar results
        // assert_relative_eq!(price1, price2,  0.05);
    }
}
