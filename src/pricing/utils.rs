/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/8/24
******************************************************************************/
use crate::greeks::utils::{big_n, d2};
use crate::model::option::Options;
use crate::model::types::Side;
use crate::pricing::binomial_model::BinomialPricingParams;
use crate::pricing::constants::{CLAMP_MAX, CLAMP_MIN};
use crate::pricing::payoff::{Payoff, PayoffInfo};
use rand::distributions::Distribution;
use statrs::distribution::Normal;

/// Simulates stock returns based on a normal distribution.
///
/// # Arguments
///
/// * `mean` - The mean return (annualized)
/// * `std_dev` - The standard deviation of returns (annualized)
/// * `length` - The number of returns to simulate
/// * `time_step` - The time step for each return (e.g., 1/252 for daily returns assuming 252 trading days)
///
/// # Returns
///
/// A vector of simulated returns
pub(crate) fn simulate_returns(mean: f64, std_dev: f64, length: usize, time_step: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();

    // Adjust mean and standard deviation for the time step
    let adjusted_mean = mean * time_step;
    let adjusted_std_dev = std_dev * time_step.sqrt();

    let normal = Normal::new(adjusted_mean, adjusted_std_dev).unwrap();

    (0..length).map(|_| normal.sample(&mut rng)).collect()
}

/// Calculates the up factor for an asset's price movement model.
///
/// # Arguments
///
/// * `volatility` - The volatility of the asset, represented as a floating point number.
/// * `dt` - The time increment for the model, typically represented in years as a floating point number.
///
/// # Returns
///
/// * A floating point number representing the up factor calculated based on the given volatility and time increment.
///
pub(crate) fn calculate_up_factor(volatility: f64, dt: f64) -> f64 {
    (volatility * dt.sqrt()).exp()
}

/// Calculates the down factor for a given volatility and time step.
///
/// # Parameters
/// - `volatility`: The volatility of the asset, typically represented by a
///   non-negative floating-point number.
/// - `dt`: The time step size, given as a floating-point number, representing
///   the discrete length of time over which the calculation is to be performed.
///
/// # Returns
/// A floating-point number representing the down factor, calculated using the
/// given volatility and time step.
///
pub(crate) fn calculate_down_factor(volatility: f64, dt: f64) -> f64 {
    (-volatility * dt.sqrt()).exp()
}

/// Calculates the probability using given interest rate, time interval,
/// down factor, and up factor.
///
/// # Arguments
///
/// * `int_rate` - The interest rate as a floating-point number.
/// * `dt` - The time interval as a floating-point number.
/// * `down_factor` - The down factor as a floating-point number.
/// * `up_factor` - The up factor as a floating-point number.
///
/// # Returns
///
/// Returns the calculated probability which is clamped between `CLAMP_MIN` and `CLAMP_MAX`.
pub(crate) fn calculate_probability(
    int_rate: f64,
    dt: f64,
    down_factor: f64,
    up_factor: f64,
) -> f64 {
    (((int_rate * dt).exp() - down_factor) / (up_factor - down_factor)).clamp(CLAMP_MIN, CLAMP_MAX)
}

/// Calculates the discount factor given an interest rate and time period.
///
/// This function computes the discount factor using the formula:
/// `exp(-int_rate * dt)`, where `exp` is the exponential function.
///
/// # Parameters
/// - `int_rate`: The interest rate (as a floating-point number).
/// - `dt`: The time period (as a floating-point number).
///
/// # Returns
/// A floating-point number representing the discount factor.
///
pub(crate) fn calculate_discount_factor(int_rate: f64, dt: f64) -> f64 {
    (-int_rate * dt).exp()
}

/// Calculates the value of an option node in a binomial options pricing model.
///
/// This function computes the value of a node by weighing the possible
/// future values at the next time step by the given probability of moving up.
/// The result is then discounted by a given discount factor to account for the
/// time value of money.
///
/// # Arguments
///
/// * `probability` - A `f64` representing the probability of moving to the next state.
/// * `next` - A mutable reference to a 2D vector containing the future values of the option.
/// * `node` - A `usize` indicating the current node's position.
/// * `discount_factor` - A `f64` used to discount the future values back to the present value.
///
/// # Returns
///
/// * A `f64` representing the calculated value of the current option node.
pub(crate) fn option_node_value_wrapper(
    probability: f64,
    next: &mut [Vec<f64>],
    node: usize,
    discount_factor: f64,
) -> f64 {
    option_node_value(
        probability,
        next[0][node],
        next[0][node + 1],
        discount_factor,
    )
}

/// Calculates the value of an option node in a binomial tree model.
///
/// # Parameters
/// - `probability`: The probability of the price moving up.
/// - `price_up`: The price if the market moves up.
/// - `price_down`: The price if the market moves down.
/// - `discount_factor`: The factor to discount the future value.
///
/// # Returns
/// The discounted expected value of the option node.
pub(crate) fn option_node_value(
    probability: f64,
    price_up: f64,
    price_down: f64,
    discount_factor: f64,
) -> f64 {
    (probability * price_up + (1.0 - probability) * price_down) * discount_factor
}

/// Calculates the option price using the Binomial Pricing Model.
///
/// # Parameters
///
/// * `params`: An instance of `BinomialPricingParams` containing the necessary parameters
///   such as the asset price, strike price, option type, and number of steps.
/// * `u`: A `f64` representing the up factor in the binomial tree.
/// * `d`: A `f64` representing the down factor in the binomial tree.
/// * `i`: An `usize` representing the current step in the binomial tree.
///
/// # Returns
///
/// Returns a `f64` representing the calculated option price at the given step.
///
pub(crate) fn calculate_option_price(
    params: BinomialPricingParams,
    u: f64,
    d: f64,
    i: usize,
) -> f64 {
    let info = PayoffInfo {
        spot: params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32),
        strike: params.strike,
        style: params.option_style.clone(),
        side: params.side.clone(),
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };
    params.option_type.payoff(&info)
}

/// Calculates the discounted payoff for an option based on the binomial pricing model.
///
/// # Parameters
///
/// * `params`: A structure containing parameters needed for the binomial pricing calculation.
///
/// # Returns
///
/// * `f64`: The discounted payoff value of the option.
///
/// The function takes into account the future asset price, the interest rate, the expiry time,
/// the type of option (call or put), and the style of the option (European or American).
///
/// It adjusts the future asset price with the provided interest rate and expiry time,
/// calculates the payoff, discounts it by the interest rate, and then adjusts for the side
/// of the trade (long or short).
///
pub(crate) fn calculate_discounted_payoff(params: BinomialPricingParams) -> f64 {
    let info = PayoffInfo {
        spot: params.asset * (params.int_rate * params.expiry).exp(),
        strike: params.strike,
        style: params.option_style.clone(),
        side: params.side.clone(),
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };
    let discounted_payoff =
        (-params.int_rate * params.expiry).exp() * params.option_type.payoff(&info);
    match params.side {
        Side::Long => discounted_payoff,
        Side::Short => -discounted_payoff,
    }
}

#[cfg(test)]
mod tests_simulate_returns {
    use super::*;
    use approx::assert_relative_eq;
    use statrs::statistics::Statistics;

    #[test]
    fn test_simulate_returns() {
        let mean = 0.05; // 5% annual return
        let std_dev = 0.2; // 20% annual volatility
        let length = 252; // One year of daily returns
        let time_step = 1.0 / 252.0; // Daily time step

        let returns = simulate_returns(mean, std_dev, length, time_step);

        assert_eq!(returns.len(), length);

        // Check that the mean and standard deviation are reasonably close to expected values
        let simulated_mean = returns.clone().mean();
        let simulated_std_dev = returns.std_dev();

        assert_relative_eq!(simulated_mean, mean * time_step, epsilon = 0.01);
        assert_relative_eq!(
            simulated_std_dev,
            std_dev * time_step.sqrt(),
            epsilon = 0.01
        );
    }
}

/// Calculates a Wiener process (Brownian motion) increment over a small-time step `dt`.
///
/// This function uses the standard normal distribution to sample a value and scales it
/// by the square root of `dt` to produce the Wiener increment. The Wiener increment is a
/// random variable with a normal distribution, which is essential for simulating Brownian motion
/// in continuous time.
///
/// # Arguments
///
/// * `dt` - A small time step over which the Wiener increment is calculated.
///
/// # Returns
///
/// * `f64` - The Wiener process increment for the given time step.
///
/// # Panics
///
/// This function will panic if the creation of the normal distribution fails, which is
/// highly unlikely with valid inputs.
///
#[allow(dead_code)]
pub(crate) fn wiener_increment(dt: f64) -> f64 {
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = rand::thread_rng();

    normal.sample(&mut rng) * dt.sqrt()
}

#[allow(dead_code)]
/// Calculates the probability that the option will remain under the strike price.
///
/// # Parameters
/// - `option`: An `Options` struct that contains various attributes necessary for the calculation,
///   such as underlying price, strike price, risk-free rate, expiration date, and implied volatility.
/// - `strike`: An optional `f64` value representing the strike price. If `None` is provided, the function
///   uses the `strike_price` from the `Options` struct.
///
/// # Returns
/// A `f64` value representing the calculated probability.
pub fn probability_keep_under_strike(option: Options, strike: Option<f64>) -> f64 {
    let strike_price = match strike {
        Some(strike) => strike,
        None => option.strike_price,
    };
    big_n(-d2(
        option.underlying_price,
        strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    ))
}

#[cfg(test)]
mod tests_utils {
    use super::*;
    use approx::assert_relative_eq;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_calculate_up_factor() {
        let volatility = 0.09531018;
        let dt = 1.0;
        let up_factor = calculate_up_factor(volatility, dt);
        let expected_up_factor = (volatility * dt.sqrt()).exp();
        assert!(
            (up_factor - expected_up_factor).abs() < EPSILON,
            "Expected {}, got {}",
            expected_up_factor,
            up_factor
        );
    }

    #[test]
    fn test_calculate_up_factor_2() {
        let volatility = 0.17;
        let dt = 1.0;
        let up_factor = calculate_up_factor(volatility, dt);
        let expected_up_factor = 1.1853;
        assert_relative_eq!(up_factor, expected_up_factor, epsilon = 0.001);
    }

    #[test]
    fn test_calculate_down_factor() {
        let volatility = 0.09531018;
        let dt = 1.0;
        let down_factor = calculate_down_factor(volatility, dt);
        let expected_down_factor = (-volatility * dt.sqrt()).exp();
        assert!(
            (down_factor - expected_down_factor).abs() < EPSILON,
            "Expected {}, got {}",
            expected_down_factor,
            down_factor
        );
    }

    #[test]
    fn test_calculate_down_factor_2() {
        let volatility = 0.17;
        let dt = 1.0;
        let up_factor = calculate_down_factor(volatility, dt);
        let expected_up_factor = 0.8437;
        assert_relative_eq!(up_factor, expected_up_factor, epsilon = 0.001);
    }

    #[test]
    fn test_calculate_probability() {
        let int_rate = 0.05;
        let dt = 1.0;
        let down_factor = 0.909090909;
        let up_factor = 1.1;
        let probability = calculate_probability(int_rate, dt, down_factor, up_factor);
        let expected_probability =
            (((int_rate * dt).exp() - down_factor) / (up_factor - down_factor)).clamp(0.0, 1.0);
        assert!(
            (probability - expected_probability).abs() < EPSILON,
            "Expected {}, got {}",
            expected_probability,
            probability
        );
    }

    #[test]
    fn test_calculate_probability_ii() {
        let int_rate = 0.05;
        let dt = 1.0;
        let down_factor = 0.8;
        let up_factor = 1.2;
        let probability = calculate_probability(int_rate, dt, down_factor, up_factor);
        assert_relative_eq!(probability, 0.6282, epsilon = 0.001);
    }

    #[test]
    fn test_calculate_discount_factor() {
        let int_rate = 0.05;
        let dt = 1.0;
        let discount_factor = calculate_discount_factor(int_rate, dt);
        let expected_discount_factor = (-int_rate * dt).exp();
        assert!(
            (discount_factor - expected_discount_factor).abs() < EPSILON,
            "Expected {}, got {}",
            expected_discount_factor,
            discount_factor
        );
    }
}

#[cfg(test)]
mod tests_probability_keep_under_strike {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType};
    use approx::assert_relative_eq;
    use tracing::info;

    #[test]
    fn test_probability_keep_under_strike_with_given_strike() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
            expiration_date: ExpirationDate::Days(365.0),
            implied_volatility: ZERO,
            underlying_symbol: "".to_string(),
            quantity: 1,
            exotic_params: None,
        };
        let strike = Some(100.0);
        let probability = probability_keep_under_strike(option, strike);
        info!("{:?} {}", strike, probability);
        assert!(
            (0.0..=1.0).contains(&probability),
            "Probability should be between 0 and 1"
        );
        assert_relative_eq!(probability, 0.5, epsilon = 0.001);
    }

    #[test]
    fn test_probability_keep_under_strike_with_default_strike() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 110.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
            expiration_date: ExpirationDate::Days(365.0),
            implied_volatility: 0.2,
            underlying_symbol: "".to_string(),
            quantity: 0,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike);
        assert!(
            (0.0..=1.0).contains(&probability),
            "Probability should be between 0 and 1"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_zero_volatility() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
            expiration_date: ExpirationDate::Days(365.0),
            implied_volatility: ZERO, // Sin volatilidad
            underlying_symbol: "".to_string(),
            quantity: 0,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike);
        assert_eq!(
            probability, 0.5,
            "With zero volatility, the probability should be 0.5"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_high_volatility() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 110.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
            expiration_date: ExpirationDate::Days(365.0),
            implied_volatility: 5.0, // Alta volatilidad
            underlying_symbol: "".to_string(),
            quantity: 0,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike);
        assert!(
            probability > ZERO && probability < 1.0,
            "Probability should still be valid even with high volatility"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_expired_option() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 110.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
            expiration_date: ExpirationDate::Days(ZERO),
            implied_volatility: 0.2,
            underlying_symbol: "".to_string(),
            quantity: 0,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike);
        assert_eq!(
            probability, 1.0,
            "Expired option should have zero probability of being ITM"
        );
    }
}
