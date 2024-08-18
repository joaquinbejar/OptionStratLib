/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::constants::{INFINITY_NEGATIVE, INFINITY_POSITIVE, ZERO};
use crate::model::option::Options;
use statrs::distribution::{ContinuousCDF, Normal};
use std::f64::consts::PI;

/// Evaluates the option payoff based on comparative values of the underlying price and strike price.
///
/// # Arguments
///
/// * `underlying_price` - The current price of the underlying asset.
/// * `strike_price` - The strike price of the option.
///
/// # Returns
///
/// * Returns positive infinity if the `underlying_price` is greater than the `strike_price`.
/// * Returns negative infinity if the `underlying_price` is less than the `strike_price`.
/// * Returns zero if the `underlying_price` is equal to the `strike_price`.
///
fn handle_zero(underlying_price: f64, strike_price: f64) -> f64 {
    if underlying_price > strike_price {
        INFINITY_POSITIVE
    } else if underlying_price < strike_price {
        INFINITY_NEGATIVE
    } else {
        ZERO
    }
}

/// Calculates the d1 component used in the Black-Scholes option pricing model.
///
/// # Arguments
///
/// * `s` - Current stock price
/// * `r` - Risk-free rate
/// * `t` - Time to expiration in years
/// * `sigma` - Volatility of the stock
///
/// # Returns
///
/// * `f64` - The computed d1 value
///
/// d1 is a crucial component in the Black-Scholes model, vital for determining
/// the price of options. It takes into account factors such as the current stock
/// price, risk-free rate, time to expiration, and stock volatility to produce
/// an important intermediate result.
pub(crate) fn d1(
    underlying_price: f64,
    strike_price: f64,
    risk_free_rate: f64,
    expiration_date: f64,
    implied_volatility: f64,
) -> f64 {
    if implied_volatility == ZERO || expiration_date == ZERO {
        return handle_zero(underlying_price, strike_price);
    }
    ((underlying_price / strike_price).ln()
        + (risk_free_rate + implied_volatility * implied_volatility / 2.0) * expiration_date)
        / (implied_volatility * expiration_date.sqrt())
}

/// Calculates the d2 value commonly used in financial mathematics, specifically in
/// the Black-Scholes option pricing model. The d2 value is derived from the d1
/// value and is used to determine the probability of the option ending up in-the-money.
///
/// # Arguments
///
/// * `s` - The current stock price.
/// * `r` - The risk-free interest rate.
/// * `t` - The time to expiration (in years).
/// * `sigma` - The volatility of the stock's returns.
///
/// # Returns
///
/// * `f64` - The computed d2 value.
pub(crate) fn d2(
    underlying_price: f64,
    strike_price: f64,
    risk_free_rate: f64,
    expiration_date: f64,
    implied_volatility: f64,
) -> f64 {
    if implied_volatility == ZERO || expiration_date == ZERO {
        return handle_zero(underlying_price, strike_price);
    }
    d1(
        underlying_price,
        strike_price,
        risk_free_rate,
        expiration_date,
        implied_volatility,
    ) - implied_volatility * expiration_date.sqrt()
}

/// Calculates the value of the standard normal distribution density function at a given point `x`.
///
/// The formula used is:
/// \[
/// f(x) = \frac{1}{\sqrt{2\pi}} e^{-\frac{x^2}{2}}
/// \]
///
/// # Arguments
///
/// * `x` - A floating point number representing the point at which to evaluate the density function.
///
/// # Returns
///
/// A floating point number representing the value of the density function at point `x`.
///
#[allow(dead_code)]
pub(crate) fn n(x: f64) -> f64 {
    1.0 / (2.0 * PI).sqrt() * (-x * x / 2.0).exp()
}

/// Calculate the derivative of the function `n` at a given point `x`.
///
/// This function represents the negative product of `x` and the result of the
/// function `n` evaluated at `x`.
///
/// # Arguments
///
/// * `x` - A floating point number representing the input to the function `n`.
///
/// # Returns
///
/// This function returns a `f64` which is the derivative of the function `n` at `x`.
///
#[allow(dead_code)]
pub(crate) fn n_prime(x: f64) -> f64 {
    -x * n(x)
}

/// Computes the cumulative distribution function (CDF) of the standard normal distribution
/// for a given value `x`.
///
/// # Arguments
///
/// * `x` - A floating-point number for which to compute the CDF of the standard normal distribution.
///
/// # Returns
///
/// A floating-point number representing the CDF of the standard normal distribution at the given `x`.
///
pub fn big_n(x: f64) -> f64 {
    const MEAN: f64 = 0.0;
    const STD_DEV: f64 = 1.0;

    let normal_distribution = Normal::new(MEAN, STD_DEV).unwrap();
    normal_distribution.cdf(x)
}

/// Calculates the d1 and d2 values used in financial option pricing models such as the Black-Scholes model.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct containing the underlying price,
///              the risk-free rate, and the implied volatility of the option.
/// * `time_to_expiry` - The time remaining until option expiry in years.
///
/// # Returns
///
/// * A tuple containing two `f64` values:
///     - `d1_value`: The calculated d1 value.
///     - `d2_value`: The calculated d2 value.
///
pub(crate) fn calculate_d_values(option: &Options) -> (f64, f64) {
    let d1_value = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );
    let d2_value = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );
    (d1_value, d2_value)
}

#[cfg(test)]
mod tests_calculate_d_values {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;

    #[test]
    fn test_calculate_d_values() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: 110.0,
            underlying_price: 100.0,
            risk_free_rate: 0.05,
            implied_volatility: 10.12,
            expiration_date: Default::default(),
            quantity: 0,
            option_style: OptionStyle::Call,
            dividend_yield: 0.0,
            premium: 0.0,
            exotic_params: None,
        };
        let (d1_value, d2_value) = calculate_d_values(&option);

        assert_relative_eq!(d1_value, 5.0555, epsilon = 0.001);
        assert_relative_eq!(d2_value, -5.064, epsilon = 0.001);
    }
}

#[cfg(test)]
mod tests_src_greeks_utils {
    use super::*;
    use approx::assert_relative_eq;
    use statrs::distribution::ContinuousCDF;
    use statrs::distribution::Normal;

    #[test]
    fn test_d1() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.05;
        let t = 1.0;
        let sigma = 0.2;
        let expected_d1 = (1.0_f64.ln() + (0.05 + 0.02) * 1.0) / (0.2 * 1.0_f64.sqrt());
        let computed_d1 = d1(s, k, r, t, sigma);
        assert!(
            (computed_d1 - expected_d1).abs() < 1e-10,
            "d1 function failed"
        );
    }

    #[test]
    fn test_d1_zero_sigma() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.05;
        let t = 1.0;
        let sigma = 0.0;
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_d1_zero_t() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.05;
        let t = 0.0;
        let sigma = 0.01;
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_d2() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.05;
        let t = 1.0;
        let sigma = 0.2;
        let computed_d2 = d2(s, k, r, t, sigma);
        let expected_d1 = (1.0_f64.ln() + (0.05 + 0.02) * 1.0) / (0.2 * 1.0_f64.sqrt());
        let expected_d2 = expected_d1 - 0.2 * 1.0_f64.sqrt();
        assert!(
            (computed_d2 - expected_d2).abs() < 1e-10,
            "d2 function failed"
        );
    }

    #[test]
    fn test_d2_bis_i() {
        let s = 100.0;
        let k = 110.0;
        let r = 0.05;
        let t = 2.0;
        let sigma = 0.2;
        let computed_d2 = d2(s, k, r, t, sigma);
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, 0.15800237, epsilon = 0.001);
        let expected_d2 = computed_d1 - 0.2 * 1.0_f64.sqrt();
        assert_relative_eq!(computed_d2, -0.124840, epsilon = 0.001);
    }

    #[test]
    fn test_d2_bis_ii() {
        let s = 100.0;
        let k = 95.0;
        let r = 0.15;
        let t = 1.0;
        let sigma = 0.2;
        let computed_d2 = d2(s, k, r, t, sigma);
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, 1.1064664, epsilon = 0.001);
        let expected_d2 = computed_d1 - 0.2 * 1.0_f64.sqrt();
        assert_relative_eq!(computed_d2, 0.9064664, epsilon = 0.001);
    }

    #[test]
    fn test_d2_zero_sigma() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.0;
        let t = 1.0;
        let sigma = 0.0;
        let computed_d2 = d2(s, k, r, t, sigma);
        let expected_d1 = (1.0_f64.ln() + (0.05 + 0.02) * 1.0) / (0.2 * 1.0_f64.sqrt());
        let expected_d2 = expected_d1 - 0.2 * 1.0_f64.sqrt();
        assert_relative_eq!(expected_d1, 0.35000000, epsilon = 0.001);
        assert_relative_eq!(computed_d2, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_d2_zero_t() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.02;
        let t = 0.0;
        let sigma = 0.01;
        let computed_d2 = d2(s, k, r, t, sigma);
        let expected_d1 = d1(s, k, r, t, sigma);
        let expected_d2 = expected_d1 - 0.2 * 1.0_f64.sqrt();
        assert_relative_eq!(expected_d1, 0.0, epsilon = 0.001);
        assert_relative_eq!(computed_d2, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_n() {
        let x = 0.0;
        let expected_n = 1.0 / (2.0 * PI).sqrt();
        let computed_n = n(x);
        assert!((computed_n - expected_n).abs() < 1e-10, "n function failed");

        let x = 1.0;
        let expected_n = 1.0 / (2.0 * PI).sqrt() * (-0.5f64).exp();
        let computed_n = n(x);
        assert!((computed_n - expected_n).abs() < 1e-10, "n function failed");
    }

    #[test]
    fn test_n_prime() {
        let x = 0.0;
        let expected_n_prime = 0.0;
        let computed_n_prime = n_prime(x);
        assert!(
            (computed_n_prime - expected_n_prime).abs() < 1e-10,
            "n_prime function failed"
        );

        let x = 1.0;
        let expected_n_prime = -1.0 * 1.0 / (2.0 * PI).sqrt() * (-0.5f64).exp();
        let computed_n_prime = n_prime(x);
        assert!(
            (computed_n_prime - expected_n_prime).abs() < 1e-10,
            "n_prime function failed"
        );
    }

    #[test]
    fn test_big_n() {
        let x = 0.0;
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x);
        let computed_big_n = big_n(x);
        assert!(
            (computed_big_n - expected_big_n).abs() < 1e-10,
            "big_n function failed"
        );

        let x = 1.0;
        let expected_big_n = normal_distribution.cdf(x);
        let computed_big_n = big_n(x);
        assert!(
            (computed_big_n - expected_big_n).abs() < 1e-10,
            "big_n function failed"
        );
    }
}
