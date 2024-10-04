/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::model::option::Options;
use crate::model::types::{PositiveF64, PZERO};
use core::f64;
use num_traits::Float;
use statrs::distribution::{ContinuousCDF, Normal};
use std::f64::consts::PI;

/// Evaluates the option payoff based on comparative values of the underlying price and strike price.
///
/// This function assesses the value of an option by comparing the current price of the underlying asset (`underlying_price`) to the strike price of the option (`strike_price`). The function returns distinct values based on the comparison:
///
/// * Positive infinity if the current price exceeds the strike price, indicating the option is in-the-money.
/// * Negative infinity if the current price is below the strike price, indicating the option is out-of-the-money.
/// * Zero if the current price is equal to the strike price, indicating the option is at the money.
///
/// # Arguments
///
/// * `underlying_price` - The current price of the underlying asset.
/// * `strike_price` - The strike price of the option.
///
/// # Returns
///
/// This function returns a floating point number of the same type as the input:
///
/// * Positive infinity if the `underlying_price` is greater than the `strike_price`.
/// * Negative infinity if the `underlying_price` is less than the `strike_price`.
/// * Zero if the `underlying_price` is equal to the `strike_price`.
///
/// # Qualifications
///
/// This function requires that the generic type `T` implements the `Float` trait, which provides the methods `T::infinity()`, `T::neg_infinity()`, and `T::zero()`. These methods return positive infinity, negative infinity, and zero respectively, appropriate to the type `T`.
fn handle_zero(underlying_price: PositiveF64, strike_price: PositiveF64) -> f64 {
    if underlying_price > strike_price {
        f64::INFINITY
    } else if underlying_price < strike_price {
        f64::NEG_INFINITY
    } else {
        ZERO
    }
}

/// Calculates the d1 component used in the Black-Scholes option pricing model.
///
/// # Type Parameters
///
/// * `T` - A floating-point type that implements the `num_traits::Float` trait
///
/// # Arguments
///
/// * `underlying_price` - Current stock price
/// * `strike_price` - Option strike price
/// * `risk_free_rate` - Risk-free rate
/// * `expiration_date` - Time to expiration in years
/// * `implied_volatility` - Volatility of the stock
///
/// # Returns
///
/// * `T` - The computed d1 value
///
/// d1 is a crucial component in the Black-Scholes model, vital for determining
/// the price of options. It takes into account factors such as the current stock
/// price, risk-free rate, time to expiration, and stock volatility to produce
/// an important intermediate result.
pub(crate) fn d1(
    underlying_price: PositiveF64,
    strike_price: PositiveF64,
    risk_free_rate: f64,
    expiration_date: f64,
    implied_volatility: f64,
) -> f64 {
    if strike_price == PZERO {
        return f64::INFINITY;
    }

    if implied_volatility == ZERO || expiration_date == ZERO {
        return handle_zero(underlying_price, strike_price);
    }

    let implied_volatility_squared = implied_volatility.powi(2);
    let ln_price_ratio = (underlying_price / strike_price).value().ln();
    let rate_vol_term = risk_free_rate + implied_volatility_squared / 2.0;
    let numerator = ln_price_ratio + rate_vol_term * expiration_date;
    let denominator = implied_volatility * expiration_date.sqrt();

    numerator / denominator
}

/// Calculates the d2 value commonly used in financial mathematics, specifically in
/// the Black-Scholes option pricing model. The d2 value is derived from the d1
/// value and is used to determine the probability of the option ending up in-the-money.
///
/// # Type Parameters
///
/// * `T` - A floating-point type that implements the `num_traits::Float` trait
///
/// # Arguments
///
/// * `underlying_price` - The current stock price, represented as a floating-point number of type `T`.
/// * `strike_price` - The option strike price, represented as a floating-point number of type `T`.
/// * `risk_free_rate` - The risk-free interest rate, represented as a floating-point number of type `T`.
/// * `expiration_date` - The time to expiration (in years), represented as a floating-point number of type `T`.
/// * `implied_volatility` - The volatility of the stock's returns, represented as a floating-point number of type `T`.
///
/// # Returns
///
/// * `T` - The computed d2 value, which is used in the Black-Scholes option pricing model.
///
/// # Details
///
/// The function first checks if either `implied_volatility` or `expiration_date` is zero. If so,
/// it delegates to the `handle_zero` function to handle this special case.
///
/// If neither of these values is zero, the function calculates the d1 value using the `d1` function
/// and then derives the d2 value by subtracting the product of the `implied_volatility` and the
/// square root of the `expiration_date` from the d1 value.
///
/// The d2 value is crucial for determining the likelihood that an option will finish in-the-money
/// (i.e., the stock price will be above the strike price for a call option or below the strike price
/// for a put option at expiration).
///
pub(crate) fn d2(
    underlying_price: PositiveF64,
    strike_price: PositiveF64,
    risk_free_rate: f64,
    expiration_date: f64,
    implied_volatility: f64,
) -> f64 {
    if implied_volatility == ZERO || expiration_date == ZERO {
        return handle_zero(underlying_price, strike_price);
    }

    let d1_value = d1(
        underlying_price,
        strike_price,
        risk_free_rate,
        expiration_date,
        implied_volatility,
    );

    d1_value - implied_volatility * expiration_date.sqrt()
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
pub(crate) fn n<T>(x: T) -> T
where
    T: Float,
{
    let two = T::from(2.0).unwrap();
    let pi = T::from(PI).unwrap();

    let denominator = (two * pi).sqrt();
    let exponent = -x * x / two;

    T::one() / denominator * exponent.exp()
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
/// This function returns a value of type `T` which is the derivative of the
/// function `n` at `x`.
///
/// # Type Parameters
///
/// * `T`: A type that implements the `Float` trait, which allows for floating point operations.
///
/// # Examples
///
/// Although no example usage is provided, it typically involves calling `n_prime`
/// with a specific floating point number as the argument to obtain the derivative
/// of the function `n` at that point.
///
/// # Panics
///
/// This function does not explicitly handle panics. Ensure that the function `n`
/// and the floating point operations are well-defined for the input value `x`.
///
/// # Safety
///
/// This code assumes that the function `n` is defined and behaves correctly for
/// the provided input type `T`. Improper or undefined behavior of `n` may lead
/// to unexpected results or runtime errors.
#[allow(dead_code)]
pub(crate) fn n_prime<T>(x: T) -> T
where
    T: Float,
{
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
pub fn big_n<T>(x: T) -> T
where
    T: Float + From<f64>,
{
    const MEAN: f64 = 0.0;
    const STD_DEV: f64 = 1.0;

    let normal_distribution = Normal::new(MEAN, STD_DEV).unwrap();
    normal_distribution.cdf(x.to_f64().unwrap()).into()
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
mod tests_handle_zero {
    use super::*;
    use crate::pos;

    #[test]
    fn test_underlying_greater_than_strike_f64() {
        let underlying = pos!(100.50);
        let strike = pos!(100.0);
        assert_eq!(handle_zero(underlying, strike), f64::INFINITY);
    }

    #[test]
    fn test_underlying_less_than_strike_f64() {
        let underlying = pos!(99.50);
        let strike = pos!(100.0);
        assert_eq!(handle_zero(underlying, strike), f64::NEG_INFINITY);
    }

    #[test]
    fn test_underlying_equal_to_strike_f64() {
        let underlying = pos!(100.0);
        let strike = pos!(100.00);
        assert_eq!(handle_zero(underlying, strike), ZERO);
    }

    #[test]
    fn test_with_large_numbers_f64() {
        let underlying = pos!(1000000.01);
        let strike = pos!(1000000.0);
        assert_eq!(handle_zero(underlying, strike), f64::INFINITY);
    }

    #[test]
    fn test_with_small_numbers_f64() {
        let underlying = pos!(0.000001);
        let strike = pos!(0.000002);
        assert_eq!(handle_zero(underlying, strike), f64::NEG_INFINITY);
    }
}

#[cfg(test)]
mod tests_calculate_d_values {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::PositiveF64;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use approx::assert_relative_eq;

    #[test]
    fn test_calculate_d_values() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: pos!(110.0),
            underlying_price: pos!(100.0),
            risk_free_rate: 0.05,
            implied_volatility: 10.12,
            expiration_date: Default::default(),
            quantity: pos!(ZERO),
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
            exotic_params: None,
        };
        let (d1_value, d2_value): (f64, f64) = calculate_d_values(&option);

        assert_relative_eq!(d1_value, 5.0555, epsilon = 0.001);
        assert_relative_eq!(d2_value, -5.064, epsilon = 0.001);
    }
}

#[cfg(test)]
mod tests_src_greeks_utils {
    use super::*;
    use crate::constants::ZERO;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::abs;
    use statrs::distribution::ContinuousCDF;
    use statrs::distribution::Normal;

    #[test]
    fn test_d1() {
        let s = pos!(100.0);
        let k = pos!(100.0);
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
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.05;
        let t = 1.0;
        let sigma = 0.0;
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, ZERO, epsilon = 0.001);
    }

    #[test]
    fn test_d1_zero_t() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.05;
        let t = 0.0;
        let sigma = 0.01;
        let computed_d1 = d1(s, k, r, t, sigma);
        assert_relative_eq!(computed_d1, ZERO, epsilon = 0.001);
    }

    #[test]
    fn test_d2() {
        let s = pos!(100.0);
        let k = pos!(100.0);
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
        let s = pos!(100.0);
        let k = pos!(110.0);
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
        let s = pos!(100.0);
        let k = pos!(95.0);
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
        let s = pos!(100.0);
        let k = pos!(100.0);
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
        let s = pos!(100.0);
        let k = pos!(100.0);
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
            abs(computed_n_prime - expected_n_prime) < 1e-10,
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
    use crate::pos;
    use approx::assert_relative_eq;

    #[test]
    fn test_d1_zero_volatility() {
        // Case where volatility (sigma) is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.0;

        // When volatility is zero, d1 should handle the case correctly
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Expected to handle division by zero or return a reasonable value like zero
        let expected_d1 = handle_zero(underlying_price, strike_price);

        // Assert that the calculated d1 is equal to the expected result
        assert_relative_eq!(calculated_d1, expected_d1, epsilon = 1e-4);
    }

    #[test]
    fn test_d1_zero_time_to_expiry() {
        // Case where time to expiry is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 0.0;
        let implied_volatility = 0.2;

        // When time to expiry is zero, d1 should handle the case correctly
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Expected to handle division by zero or return a reasonable value like zero
        let expected_d1 = handle_zero(underlying_price, strike_price);

        // Assert that the calculated d1 is equal to the expected result
        assert_relative_eq!(calculated_d1, expected_d1, epsilon = 1e-4);
    }

    #[test]
    fn test_d1_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 100.0; // Very high volatility

        // High volatility should result in a small or large value for d1
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Assert the result should be finite and non-infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for high volatility"
        );
    }

    #[test]
    fn test_d1_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = pos!(f64::MAX); // Very high stock price
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very high underlying price should result in a large d1 value
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Assert that d1 is finite and not infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for high underlying price"
        );
    }

    #[test]
    fn test_d1_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = pos!(0.01); // Very low stock price
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very low underlying price should result in a small or negative d1 value
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Assert the result should be finite and not infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for low underlying price"
        );
        assert!(
            calculated_d1.is_sign_negative(),
            "d1 should be negative for very low underlying price"
        );
    }

    #[test]
    fn test_d1_zero_strike_price() {
        // Case where strike price is zero
        let underlying_price = pos!(100.0);
        let strike_price = PZERO;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Expecting positive infinity since underlying_price > strike_price
        assert!(calculated_d1.is_infinite() && calculated_d1.is_sign_positive(), "d1 should return positive infinity when strike price is zero and underlying is greater.");
    }

    #[test]
    fn test_d1_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = f64::MAX; // Very high risk-free rate
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // High risk-free rate should result in a large d1 value, potentially infinite
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Assert that d1 is positive infinity
        assert!(
            calculated_d1.is_infinite() && calculated_d1.is_sign_positive(),
            "d1 should be positive infinity for extremely high risk-free rate"
        );
    }
}

#[cfg(test)]
mod calculate_d2_values {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;

    #[test]
    fn test_d2_zero_volatility() {
        // Case where volatility (implied_volatility) is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.0;

        // When volatility is zero, d2 should handle the case correctly using handle_zero
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Since volatility is zero, handle_zero will be invoked
        let expected_d2 = handle_zero(underlying_price, strike_price);

        // Assert that d2 is equal to the expected result from handle_zero
        assert_relative_eq!(calculated_d2, expected_d2, epsilon = 1e-4);
    }

    #[test]
    fn test_d2_zero_time_to_expiry() {
        // Case where time to expiration is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 0.0;
        let implied_volatility = 0.2;

        // When time to expiration is zero, handle_zero should be called
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Since expiration_date is zero, handle_zero will be invoked
        let expected_d2 = handle_zero(underlying_price, strike_price);

        // Assert that d2 is equal to the expected result from handle_zero
        assert_relative_eq!(calculated_d2, expected_d2, epsilon = 1e-4);
    }

    #[test]
    fn test_d2_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 100.0; // Very high volatility

        // High volatility should result in a significant negative shift in d2
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // d2 should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for high volatility"
        );
    }

    #[test]
    fn test_d2_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = pos!(f64::MAX);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very high underlying price should result in a large d2 value
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // d2 should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for high underlying price"
        );
    }

    #[test]
    fn test_d2_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = pos!(0.01);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Very low underlying price should result in a small or negative d2 value
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Assert the result should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for low underlying price"
        );
        assert!(
            calculated_d2.is_sign_negative(),
            "d2 should be negative for very low underlying price"
        );
    }

    #[test]
    fn test_d2_zero_strike_price() {
        // Case where strike price is zero
        let underlying_price = pos!(100.0);
        let strike_price = PZERO;
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Expecting positive infinity since underlying_price > strike_price
        assert!(calculated_d2.is_infinite() && calculated_d2.is_sign_positive(), "d2 should return positive infinity when strike price is zero and underlying is greater.");
    }

    #[test]
    fn test_d2_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = f64::MAX; // Very high risk-free rate
        let expiration_date = 1.0;
        let implied_volatility = 0.2;

        // High risk-free rate should result in a large d2 value, potentially infinite
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        );

        // Assert that d2 is positive infinity
        assert!(
            calculated_d2.is_infinite() && calculated_d2.is_sign_positive(),
            "d2 should be positive infinity for extremely high risk-free rate"
        );
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
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6); // if lower epsilon fail
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
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6); // if lower epsilon fail
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
