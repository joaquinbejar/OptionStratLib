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
            dividend_yield: ZERO,
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
        assert_relative_eq!(computed_d1, ZERO, epsilon = 0.001);
    }

    #[test]
    fn test_d1_zero_t() {
        let s = 100.0;
        let k = 100.0;
        let r = 0.05;
        let t = 0.0;
        let sigma = 0.01;
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, ZERO, epsilon = 0.001);
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
        assert_relative_eq!(expected_d1, 0.35000000, epsilon = 0.001);
        assert_relative_eq!(computed_d2, ZERO, epsilon = 0.001);
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
        assert_relative_eq!(expected_d1, ZERO, epsilon = 0.001);
        assert_relative_eq!(computed_d2, ZERO, epsilon = 0.001);
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

#[cfg(test)]
mod calculate_d1_values {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_d1_zero_volatility() {
        // Case where volatility (sigma) is zero
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.0;

        // When volatility is zero, d1 should handle the case correctly
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Expected to handle division by zero or return a reasonable value like zero
        let expected_d1 = handle_zero(underlying_price, strike_price);

        // Assert that the calculated d1 is equal to the expected result
        assert_relative_eq!(calculated_d1, expected_d1, epsilon = 1e-4);
    }

    #[test]
    fn test_d1_zero_time_to_expiry() {
        // Case where time to expiry is zero
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 0.0;
        let implied_volatility = 0.2;

        // When time to expiry is zero, d1 should handle the case correctly
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Expected to handle division by zero or return a reasonable value like zero
        let expected_d1 = handle_zero(underlying_price, strike_price);

        // Assert that the calculated d1 is equal to the expected result
        assert_relative_eq!(calculated_d1, expected_d1, epsilon = 1e-4);
    }

    #[test]
    fn test_d1_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 100.0; // Very high volatility

        // High volatility should result in a small or large value for d1
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Assert the result should be finite and non-infinite
        assert!(calculated_d1.is_finite(), "d1 should not be infinite for high volatility");
    }

    #[test]
    fn test_d1_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = f64::MAX; // Very high stock price
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very high underlying price should result in a large d1 value
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Assert that d1 is finite and not infinite
        assert!(calculated_d1.is_finite(), "d1 should not be infinite for high underlying price");
    }

    #[test]
    fn test_d1_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = 0.01; // Very low stock price
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very low underlying price should result in a small or negative d1 value
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Assert the result should be finite and not infinite
        assert!(calculated_d1.is_finite(), "d1 should not be infinite for low underlying price");
        assert!(calculated_d1.is_sign_negative(), "d1 should be negative for very low underlying price");
    }

    #[test]
    fn test_d1_zero_strike_price() {
        // Case where strike price is zero
        let underlying_price = 100.0;
        let strike_price = 0.0; // Strike price set to zero
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Expecting positive infinity since underlying_price > strike_price
        assert!(calculated_d1.is_infinite() && calculated_d1.is_sign_positive(), "d1 should return positive infinity when strike price is zero and underlying is greater.");
    }

    #[test]
    fn test_d1_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = f64::MAX; // Very high risk-free rate
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // High risk-free rate should result in a large d1 value, potentially infinite
        let calculated_d1 = d1(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Assert that d1 is positive infinity
        assert!(calculated_d1.is_infinite() && calculated_d1.is_sign_positive(), "d1 should be positive infinity for extremely high risk-free rate");
    }

}

#[cfg(test)]
mod calculate_d2_values {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_d2_zero_volatility() {
        // Case where volatility (implied_volatility) is zero
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.0;

        // When volatility is zero, d2 should handle the case correctly using handle_zero
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Since volatility is zero, handle_zero will be invoked
        let expected_d2 = handle_zero(underlying_price, strike_price);

        // Assert that d2 is equal to the expected result from handle_zero
        assert_relative_eq!(calculated_d2, expected_d2, epsilon = 1e-4);
    }

    #[test]
    fn test_d2_zero_time_to_expiry() {
        // Case where time to expiration is zero
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 0.0;
        let implied_volatility = 0.2;

        // When time to expiration is zero, handle_zero should be called
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Since expiration_date is zero, handle_zero will be invoked
        let expected_d2 = handle_zero(underlying_price, strike_price);

        // Assert that d2 is equal to the expected result from handle_zero
        assert_relative_eq!(calculated_d2, expected_d2, epsilon = 1e-4);
    }

    #[test]
    fn test_d2_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 100.0; // Very high volatility

        // High volatility should result in a significant negative shift in d2
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // d2 should be finite and not infinite
        assert!(calculated_d2.is_finite(), "d2 should not be infinite for high volatility");
    }

    #[test]
    fn test_d2_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = f64::MAX; // Very high stock price
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very high underlying price should result in a large d2 value
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // d2 should be finite and not infinite
        assert!(calculated_d2.is_finite(), "d2 should not be infinite for high underlying price");
    }

    #[test]
    fn test_d2_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = 0.01; // Very low stock price
        let strike_price = 100.0;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very low underlying price should result in a small or negative d2 value
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Assert the result should be finite and not infinite
        assert!(calculated_d2.is_finite(), "d2 should not be infinite for low underlying price");
        assert!(calculated_d2.is_sign_negative(), "d2 should be negative for very low underlying price");
    }

    #[test]
    fn test_d2_zero_strike_price() {
        // Case where strike price is zero
        let underlying_price = 100.0;
        let strike_price = 0.0; // Strike price set to zero
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Expecting positive infinity since underlying_price > strike_price
        assert!(calculated_d2.is_infinite() && calculated_d2.is_sign_positive(), "d2 should return positive infinity when strike price is zero and underlying is greater.");
    }

    #[test]
    fn test_d2_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = 100.0;
        let strike_price = 100.0;
        let risk_free_rate = f64::MAX; // Very high risk-free rate
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // High risk-free rate should result in a large d2 value, potentially infinite
        let calculated_d2 = d2(underlying_price, strike_price, risk_free_rate, expiration_date, implied_volatility);

        // Assert that d2 is positive infinity
        assert!(calculated_d2.is_infinite() && calculated_d2.is_sign_positive(), "d2 should be positive infinity for extremely high risk-free rate");
    }
}

#[cfg(test)]
mod calculate_n_values {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_n_zero() {
        // Case where x = 0.0
        let x = 0.0f64;

        // The PDF of the standard normal distribution at x = 0 is 1/sqrt(2*pi)
        let expected_n = 1.0f64 / (2.0 * PI).sqrt();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_positive_small_value() {
        // Case where x is a small positive value
        let x = 0.5f64;

        // Expected result for n(0.5), can be precomputed
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-0.5f64 * 0.5f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_negative_small_value() {
        // Case where x is a small negative value
        let x = -0.5f64;

        // Expected result for n(-0.5), which should be the same as n(0.5) due to symmetry
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-0.5f64 * 0.5f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_large_positive_value() {
        // Case where x is a large positive value
        let x = 5.0f64;

        // Expected result for n(5.0), should be a very small value
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-5.0f64 * 5.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_large_negative_value() {
        // Case where x is a large negative value
        let x = -5.0f64;

        // Expected result for n(-5.0), should be the same as n(5.0) due to symmetry
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-5.0f64 * 5.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_extreme_positive_value() {
        // Case where x is a very large positive value
        let x = 100.0f64;

        // Expected result for n(100.0), should be extremely close to 0
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-100.0f64 * 100.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that n(x) is effectively 0 for such a large input
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-100);
    }

    #[test]
    fn test_n_extreme_negative_value() {
        // Case where x is a very large negative value
        let x = -100.0f64;

        // Expected result for n(-100.0), should be extremely close to 0
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-100.0f64 * 100.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x);

        // Assert that n(x) is effectively 0 for such a large negative input
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-100);
    }
}

#[cfg(test)]
mod calculate_n_prime_values {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_n_prime_zero() {
        // Case where x = 0.0
        let x = 0.0f64;

        // The derivative of the PDF at x = 0 should be 0 because -x * n(x) = 0 * n(0) = 0
        let expected_n_prime = 0.0f64;

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_positive_small_value() {
        // Case where x is a small positive value
        let x = 0.5f64;

        // Expected result for n_prime(0.5), we calculate -x * n(x)
        let expected_n_prime = -x * n(x);

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_negative_small_value() {
        // Case where x is a small negative value
        let x = -0.5f64;

        // Expected result for n_prime(-0.5), we calculate -x * n(x)
        let expected_n_prime = -x * n(x);

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_large_positive_value() {
        // Case where x is a large positive value
        let x = 5.0f64;

        // Expected result for n_prime(5.0), we calculate -x * n(x)
        let expected_n_prime = -x * n(x);

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_large_negative_value() {
        // Case where x is a large negative value
        let x = -5.0f64;

        // Expected result for n_prime(-5.0), we calculate -x * n(x)
        let expected_n_prime = -x * n(x);

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_extreme_positive_value() {
        // Case where x is a very large positive value
        let x = 100.0f64;

        // Expected result for n_prime(100.0), should be extremely close to 0
        let expected_n_prime = -x * n(x);

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that n_prime(x) is effectively 0 for such a large input
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-100);
    }

    #[test]
    fn test_n_prime_extreme_negative_value() {
        // Case where x is a very large negative value
        let x = -100.0f64;

        // Expected result for n_prime(-100.0), should be extremely close to 0
        let expected_n_prime = -x * n(x);

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x);

        // Assert that n_prime(x) is effectively 0 for such a large negative input
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-100);
    }
}

#[cfg(test)]
mod calculate_big_n_values {
    use super::*;
    use approx::assert_relative_eq;
    use statrs::distribution::Normal;

    #[test]
    fn test_big_n_zero() {
        // Case where x = 0.0
        let x = 0.0f64;

        // The CDF of the standard normal distribution at x = 0 is 0.5
        let expected_big_n = 0.5f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_positive_small_value() {
        // Case where x is a small positive value
        let x = 0.5f64;

        // The expected CDF for the standard normal distribution at x = 0.5 can be precomputed
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x);

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_negative_small_value() {
        // Case where x is a small negative value
        let x = -0.5f64;

        // The expected CDF for the standard normal distribution at x = -0.5 can be precomputed
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x);

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_large_positive_value() {
        // Case where x is a large positive value
        let x = 5.0f64;

        // The CDF for large positive x should be very close to 1
        let expected_big_n = 1.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6);// if lower epsilon fail
    }

    #[test]
    fn test_big_n_large_negative_value() {
        // Case where x is a large negative value
        let x = -5.0f64;

        // The CDF for large negative x should be very close to 0
        let expected_big_n = 0.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6);// if lower epsilon fail
    }

    #[test]
    fn test_big_n_extreme_positive_value() {
        // Case where x is an extremely large positive value
        let x = 100.0f64;

        // The CDF for an extremely large positive x should be effectively 1
        let expected_big_n = 1.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that big_n(x) is effectively 1 for such a large positive input
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-12);
    }

    #[test]
    fn test_big_n_extreme_negative_value() {
        // Case where x is an extremely large negative value
        let x = -100.0f64;

        // The CDF for an extremely large negative x should be effectively 0
        let expected_big_n = 0.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x);

        // Assert that big_n(x) is effectively 0 for such a large negative input
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-12);
    }
}
