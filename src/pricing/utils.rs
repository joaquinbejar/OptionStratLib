/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/8/24
******************************************************************************/
use crate::model::types::Side;
use crate::pricing::binomial_model::BinomialPricingParams;
use crate::pricing::constants::{CLAMP_MAX, CLAMP_MIN};
use crate::pricing::payoff::Payoff;

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
pub(crate) fn calculate_option_node_value(
    probability: f64,
    next: &mut [Vec<f64>],
    node: usize,
    discount_factor: f64,
) -> f64 {
    (probability * next[0][node] + (1.0 - probability) * next[0][node + 1]) * discount_factor
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
    let future_asset_price = params.asset * (params.int_rate * params.expiry).exp();
    let discounted_payoff = (-params.int_rate * params.expiry).exp()
        * params
            .option_type
            .payoff(future_asset_price, params.strike, params.option_style);
    match params.side {
        Side::Long => discounted_payoff,
        Side::Short => -discounted_payoff,
    }
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
    let price = params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32);
    params
        .option_type
        .payoff(price, params.strike, params.option_style)
}

/// Calculates the discounted value based on the given parameters.
///
/// The function calculates the expected value when there is a certain
/// probability `p` of a price moving up, and a probability `1 - p` of the price moving down.
/// This is then discounted back to the present value using the interest rate `int_rate`
/// over a time period `dt`.
///
/// # Parameters
/// - `p`: The probability of the price moving up.
/// - `price_up`: The price if it moves up.
/// - `price_down`: The price if it moves down.
/// - `int_rate`: The interest rate for discounting.
/// - `dt`: The time period over which the discounting occurs.
///
pub(crate) fn calculate_discounted_value(
    p: f64,
    price_up: f64,
    price_down: f64,
    int_rate: f64,
    dt: f64,
) -> f64 {
    (p * price_up + (1.0 - p) * price_down) * (-int_rate * dt).exp()
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
