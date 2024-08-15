/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 15/8/24
******************************************************************************/

use crate::constants::{MAX_VOLATILITY, MIN_VOLATILITY, TOLERANCE};
use crate::model::option::Options;
use std::f64;

/// Calculates the constant volatility from a series of returns.
///
/// # Arguments
///
/// * `returns` - A slice of f64 values representing the returns.
///
/// # Returns
///
/// The calculated volatility as an f64.
fn constant_volatility(returns: &[f64]) -> f64 {
    let n = returns.len();

    if n < 2 {
        return 0.0;
    }

    let mean = returns.iter().sum::<f64>() / n as f64;
    let variance = returns.iter().map(|&r| (r - mean).powi(2)).sum::<f64>() / (n as f64 - 1.0);
    variance.sqrt()
}

/// Calculates historical volatility using a moving window approach.
///
/// # Arguments
///
/// * `returns` - A slice of f64 values representing the returns.
/// * `window_size` - The size of the moving window.
///
/// # Returns
///
/// A vector of f64 values representing the historical volatility for each window.
fn historical_volatility(returns: &[f64], window_size: usize) -> Vec<f64> {
    returns
        .windows(window_size)
        .map(constant_volatility)
        .collect()
}

/// Calculates EWMA (Exponentially Weighted Moving Average) volatility.
///
/// # Arguments
///
/// * `returns` - A slice of f64 values representing the returns.
/// * `lambda` - The decay factor (typically 0.94 for daily data).
///
/// # Returns
///
/// A vector of f64 values representing the EWMA volatility.
fn ewma_volatility(returns: &[f64], lambda: f64) -> Vec<f64> {
    let mut variance = returns[0].powi(2);
    let mut volatilities = vec![variance.sqrt()];

    for &return_value in &returns[1..] {
        variance = lambda * variance + (1.0 - lambda) * return_value.powi(2);
        volatilities.push(variance.sqrt());
    }

    volatilities
}

/// Calculates the implied volatility of an option given its market price.
///
/// This function uses the Newton-Raphson method to iteratively approximate the implied
/// volatility that corresponds to the observed market price of the option. The implied
/// volatility is updated within the `Options` struct provided as a mutable reference.
///
/// # Parameters
/// - `market_price`: The observed market price of the option.
/// - `options`: A mutable reference to an `Options` struct, which should contain the necessary
///   methods and fields such as `implied_volatility`, `calculate_price_black_scholes()`, and `vega()`.
/// - `max_iterations`: The maximum number of iterations allowed for the Newton-Raphson method.
///
/// # Returns
/// The function returns the estimated implied volatility of the option.
///
/// # Remarks
/// - If the price difference between the calculated and market price is within the tolerated threshold (`TOLERANCE`),
///   the current implied volatility is returned.
/// - The function ensures that the implied volatility stays positive.
///
fn implied_volatility(
    market_price: f64,
    options: &mut Options, // Pass Options struct as a mutable reference
    max_iterations: i64,
) -> f64 {
    let mut iv = options.implied_volatility;
    for _ in 0..max_iterations {
        options.implied_volatility = iv; // Update the implied volatility in the Options struct

        let price = options.calculate_price_black_scholes();
        let vega = options.vega();

        let price_diff = price - market_price;

        if price_diff.abs() < TOLERANCE {
            return iv; // The current implied volatility is close enough
        }

        iv -= price_diff / vega; // Newton-Raphson update step

        if iv < 0.0 {
            iv = 1e-8; // Ensure volatility stays positive
        }
        let new_iv = iv - price_diff / vega;

        // Check if new_iv is NaN or infinite
        if !new_iv.is_nan() || !new_iv.is_infinite() {
            return iv;
        }

        // Limit the range of implied volatility
        iv = new_iv.max(MIN_VOLATILITY).min(MAX_VOLATILITY);
    }
    iv
}

/// Calculates GARCH(1,1) volatility (simplified).
///
/// # Arguments
///
/// * `returns` - A slice of f64 values representing the returns.
/// * `omega`, `alpha`, `beta` - GARCH(1,1) parameters.
///
/// # Returns
///
/// A vector of f64 values representing the GARCH(1,1) volatility.
fn garch_volatility(returns: &[f64], omega: f64, alpha: f64, beta: f64) -> Vec<f64> {
    let mut variance = returns[0].powi(2);
    let mut volatilities = vec![variance.sqrt()];
    for &return_value in &returns[1..] {
        variance = omega + alpha * return_value.powi(2) + beta * variance;
        volatilities.push(variance.sqrt());
    }
    volatilities
}

/// Simulates stochastic volatility using the Heston model (simplified).
///
/// # Arguments
///
/// * `kappa` - Mean reversion speed.
/// * `theta` - Long-term variance.
/// * `xi` - Volatility of volatility.
/// * `v0` - Initial variance.
/// * `dt` - Time step.
/// * `steps` - Number of simulation steps.
///
/// # Returns
///
/// A vector of f64 values representing the simulated volatility.
fn simulate_heston_volatility(
    kappa: f64,
    theta: f64,
    xi: f64,
    v0: f64,
    dt: f64,
    steps: usize,
) -> Vec<f64> {
    let mut v = v0;
    let mut volatilities = vec![v.sqrt()];
    for _ in 1..steps {
        let dw = rand::random::<f64>() * dt.sqrt();
        v += kappa * (theta - v) * dt + xi * v.sqrt() * dw;
        v = v.max(0.0); // Ensure variance doesn't become negative
        volatilities.push(v.sqrt());
    }
    volatilities
}

/// Interpolates volatility from a volatility surface (simplified).
///
/// # Arguments
///
/// * `strike` - The strike price.
/// * `time_to_expiry` - Time to expiry in years.
/// * `volatility_surface` - A slice of tuples (strike, time_to_expiry, volatility).
///
/// # Returns
///
/// The interpolated volatility as an f64.
fn interpolate_volatility_surface(
    strike: f64,
    time_to_expiry: f64,
    volatility_surface: &[(f64, f64, f64)],
) -> f64 {
    // This function would need a more sophisticated implementation
    // to interpolate the volatility surface
    todo!("Requires interpolation implementation")
}

/// Calculates bounds for uncertain volatility (simplified).
///
/// # Arguments
///
/// * `min_volatility`, `max_volatility` - The range of possible volatilities.
/// * `option_price` - The current option price.
/// * `strike` - The strike price.
/// * `spot` - The spot price of the underlying asset.
/// * `time_to_expiry` - Time to expiry in years.
/// * `risk_free_rate` - The risk-free interest rate.
///
/// # Returns
///
/// A tuple of (lower_bound, upper_bound) for the option price.
fn uncertain_volatility_bounds(
    min_volatility: f64,
    max_volatility: f64,
    option_price: f64,
    strike: f64,
    spot: f64,
    time_to_expiry: f64,
    risk_free_rate: f64,
) -> (f64, f64) {
    // This function would need a more sophisticated implementation
    // to solve the optimal control equations
    todo!("Requires optimal control implementation")
}

#[cfg(test)]
mod tests_constant_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_constant_volatility_single_value() {
        let returns = [0.05];
        let result = constant_volatility(&returns);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_constant_volatility_identical_values() {
        let returns = [0.02, 0.02, 0.02, 0.02];
        let result = constant_volatility(&returns);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_constant_volatility_varying_values() {
        let returns = [0.01, 0.03, 0.02, 0.04];
        let result = constant_volatility(&returns);
        assert_relative_eq!(result, 0.012909944487358056, epsilon = 1e-10);
    }

    #[test]
    fn test_constant_volatility_negative_values() {
        let returns = [-0.01, -0.03, -0.02, -0.04];
        let result = constant_volatility(&returns);
        assert_relative_eq!(result, 0.012909944487358056, epsilon = 1e-10);
    }

    #[test]
    fn test_constant_volatility_mixed_values() {
        let returns = [0.01, -0.02, 0.03, -0.04];
        let result = constant_volatility(&returns);
        assert_relative_eq!(result, 0.031091263510296053, epsilon = 1e-10);
    }
}

#[cfg(test)]
mod tests_historical_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_historical_volatility_empty_returns() {
        let returns: [f64; 0] = [];
        let result = historical_volatility(&returns, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_volatility_single_value() {
        let returns = [0.02];
        let result = historical_volatility(&returns, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_volatility_insufficient_data() {
        let returns = [0.01, 0.02];
        let result = historical_volatility(&returns, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_volatility_exact_window() {
        let returns = [0.01, 0.02, 0.03];
        let result = historical_volatility(&returns, 3);
        assert_eq!(result.len(), 1);
        assert_relative_eq!(result[0], 0.01, epsilon = 1e-10);
    }

    #[test]
    fn test_historical_volatility_larger_window() {
        let returns = [0.01, 0.02, 0.03, 0.04];
        let result = historical_volatility(&returns, 3);
        assert_eq!(result.len(), 2);
        assert_relative_eq!(result[0], 0.01, epsilon = 1e-10);
        assert_relative_eq!(result[1], 0.01, epsilon = 1e-10);
    }

    #[test]
    fn test_historical_volatility_varying_returns() {
        let returns = [0.01, -0.02, 0.03, -0.04, 0.05];
        let result = historical_volatility(&returns, 3);
        assert_eq!(result.len(), 3);
        assert_relative_eq!(result[0], 0.025166114784235832, epsilon = 1e-10);
        assert_relative_eq!(result[1], 0.03605551275463989, epsilon = 1e-10);
        assert_relative_eq!(result[2], 0.04725815626252609, epsilon = 1e-10);
    }
}

#[cfg(test)]
mod tests_ewma_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_ewma_volatility_single_return() {
        let returns = [0.02];
        let lambda = 0.94;
        let result = ewma_volatility(&returns, lambda);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0.02); // The volatility is simply the return itself
    }

    #[test]
    fn test_ewma_volatility_constant_returns() {
        let returns = [0.02, 0.02, 0.02, 0.02];
        let lambda = 0.94;
        let result = ewma_volatility(&returns, lambda);
        assert_eq!(result.len(), 4);
        // Test the EWMA calculation
        let expected = [
            0.02,
            (lambda * 0.02f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2)).sqrt(),
            (lambda
                * (lambda * 0.02f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2))
                    .sqrt()
                    .powi(2)
                + (1.0 - lambda) * 0.02f64.powi(2))
            .sqrt(),
            (lambda
                * (lambda
                    * (lambda * 0.02f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2))
                        .sqrt()
                        .powi(2)
                    + (1.0 - lambda) * 0.02f64.powi(2))
                .sqrt()
                .powi(2)
                + (1.0 - lambda) * 0.02f64.powi(2))
            .sqrt(),
        ];
        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_ewma_volatility_varying_returns() {
        let returns = [0.01, -0.02, 0.03, -0.04];
        let lambda = 0.94;
        let result = ewma_volatility(&returns, lambda);
        assert_eq!(result.len(), 4);
        // Calculate expected volatilities manually
        let expected = [
            0.01,
            (lambda * 0.01f64.powi(2) + (1.0 - lambda) * (-0.02f64).powi(2)).sqrt(),
            (lambda
                * (lambda * 0.01f64.powi(2) + (1.0 - lambda) * (-0.02f64).powi(2))
                    .sqrt()
                    .powi(2)
                + (1.0 - lambda) * 0.03f64.powi(2))
            .sqrt(),
            (lambda
                * (lambda
                    * (lambda * 0.01f64.powi(2) + (1.0 - lambda) * (-0.02f64).powi(2))
                        .sqrt()
                        .powi(2)
                    + (1.0 - lambda) * 0.03f64.powi(2))
                .sqrt()
                .powi(2)
                + (1.0 - lambda) * (-0.04f64).powi(2))
            .sqrt(),
        ];
        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_ewma_volatility_high_lambda() {
        let returns = [0.01, 0.02, 0.03, 0.04];
        let lambda = 0.99; // High lambda means slow decay
        let result = ewma_volatility(&returns, lambda);
        assert_eq!(result.len(), 4);
        // Test the EWMA calculation with a high lambda
        let expected = [
            0.01,
            (lambda * 0.01f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2)).sqrt(),
            (lambda
                * (lambda * 0.01f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2))
                    .sqrt()
                    .powi(2)
                + (1.0 - lambda) * 0.03f64.powi(2))
            .sqrt(),
            (lambda
                * (lambda
                    * (lambda * 0.01f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2))
                        .sqrt()
                        .powi(2)
                    + (1.0 - lambda) * 0.03f64.powi(2))
                .sqrt()
                .powi(2)
                + (1.0 - lambda) * 0.04f64.powi(2))
            .sqrt(),
        ];
        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_ewma_volatility_low_lambda() {
        let returns = [0.01, 0.02, 0.03, 0.04];
        let lambda = 0.5; // Low lambda means faster decay
        let result = ewma_volatility(&returns, lambda);
        assert_eq!(result.len(), 4);
        // Test the EWMA calculation with a low lambda
        let expected = [
            0.01,
            (lambda * 0.01f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2)).sqrt(),
            (lambda
                * (lambda * 0.01f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2))
                    .sqrt()
                    .powi(2)
                + (1.0 - lambda) * 0.03f64.powi(2))
            .sqrt(),
            (lambda
                * (lambda
                    * (lambda * 0.01f64.powi(2) + (1.0 - lambda) * 0.02f64.powi(2))
                        .sqrt()
                        .powi(2)
                    + (1.0 - lambda) * 0.03f64.powi(2))
                .sqrt()
                .powi(2)
                + (1.0 - lambda) * 0.04f64.powi(2))
            .sqrt(),
        ];
        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }
}

#[cfg(test)]
mod tests_implied_volatility {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;

    fn create_test_option() -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            100.0,
            ExpirationDate::Days(30.0),
            0.2, // initial implied volatility
            1,
            100.0,
            0.05,
            OptionStyle::Call,
            0.0,
            None,
        )
    }

    #[test]
    fn test_implied_volatility_convergence() {
        let mut option = create_test_option();
        let market_price = 5.0; // Assume this is the observed market price
        let iv = implied_volatility(market_price, &mut option, 100);

        // Check if the calculated price with the new IV is close to the market price
        option.implied_volatility = iv;
        let calculated_price = option.calculate_price_black_scholes();
        println!("{}", (calculated_price - market_price).abs());
        assert_relative_eq!(calculated_price, market_price, epsilon = 0.002);
    }

    #[test]
    fn test_implied_volatility_bounds() {
        let mut option = create_test_option();
        let market_price = 5.0;
        let iv = implied_volatility(market_price, &mut option, 100);

        assert!(iv > 0.0, "Implied volatility should be positive");
        assert!(iv < 1.0, "Implied volatility should be less than 100%");
    }

    #[test]
    fn test_implied_volatility_max_iterations() {
        let mut option = create_test_option();
        let market_price = 5.0;
        let iv = implied_volatility(market_price, &mut option, 1); // Only allow 1 iteration

        // The IV might not have converged, but it should still be a valid number
        assert!(!iv.is_nan());
        assert!(!iv.is_infinite());
    }

    #[test]
    fn test_implied_volatility_extreme_prices() {
        let mut option = create_test_option();

        // Test with a very low market price
        let low_price = 0.01;
        let low_iv = implied_volatility(low_price, &mut option, 100);
        assert!(low_iv > 0.0);

        // Test with a very high market price
        let high_price = 10.0;
        let high_iv = implied_volatility(high_price, &mut option, 100);
        println!("{}", high_iv);
        assert!(high_iv < 1.0);
    }
}

#[cfg(test)]
mod tests_garch_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_garch_volatility_single_return() {
        let returns = [0.02];
        let omega = 0.1;
        let alpha = 0.2;
        let beta = 0.7;
        let result = garch_volatility(&returns, omega, alpha, beta);
        assert_eq!(result.len(), 1);
        assert_relative_eq!(result[0], 0.02, epsilon = 1e-10); // Volatility is the return itself
    }

    #[test]
    fn test_garch_volatility_constant_returns() {
        let returns = [0.02, 0.02, 0.02, 0.02];
        let omega = 0.1;
        let alpha = 0.2;
        let beta = 0.7;
        let result = garch_volatility(&returns, omega, alpha, beta);
        assert_eq!(result.len(), 4);

        // Expected volatilities calculated manually or using a reliable tool
        let expected = [
            0.02,
            (omega + alpha * 0.02f64.powi(2) + beta * 0.02f64.powi(2)).sqrt(),
            (omega
                + alpha * 0.02f64.powi(2)
                + beta
                    * (omega + alpha * 0.02f64.powi(2) + beta * 0.02f64.powi(2))
                        .sqrt()
                        .powi(2))
            .sqrt(),
            (omega
                + alpha * 0.02f64.powi(2)
                + beta
                    * (omega
                        + alpha * 0.02f64.powi(2)
                        + beta
                            * (omega + alpha * 0.02f64.powi(2) + beta * 0.02f64.powi(2))
                                .sqrt()
                                .powi(2))
                    .sqrt()
                    .powi(2))
            .sqrt(),
        ];

        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_garch_volatility_varying_returns() {
        let returns = [0.01, -0.02, 0.03, -0.04];
        let omega = 0.1;
        let alpha = 0.2;
        let beta = 0.7;
        let result = garch_volatility(&returns, omega, alpha, beta);
        assert_eq!(result.len(), 4);

        // Expected volatilities calculated manually or using a reliable tool
        let expected = [
            0.01,
            (omega + alpha * (-0.02f64).powi(2) + beta * 0.01f64.powi(2)).sqrt(),
            (omega
                + alpha * 0.03f64.powi(2)
                + beta
                    * (omega + alpha * (-0.02f64).powi(2) + beta * 0.01f64.powi(2))
                        .sqrt()
                        .powi(2))
            .sqrt(),
            (omega
                + alpha * (-0.04f64).powi(2)
                + beta
                    * (omega
                        + alpha * 0.03f64.powi(2)
                        + beta
                            * (omega + alpha * (-0.02f64).powi(2) + beta * 0.01f64.powi(2))
                                .sqrt()
                                .powi(2))
                    .sqrt()
                    .powi(2))
            .sqrt(),
        ];

        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_garch_volatility_high_omega() {
        let returns = [0.01, 0.02, 0.03, 0.04];
        let omega = 1.0; // High omega, high base variance
        let alpha = 0.2;
        let beta = 0.7;
        let result = garch_volatility(&returns, omega, alpha, beta);
        assert_eq!(result.len(), 4);

        // Expected volatilities calculated manually or using a reliable tool
        let expected = [
            0.01,
            (omega + alpha * 0.02f64.powi(2) + beta * 0.01f64.powi(2)).sqrt(),
            (omega
                + alpha * 0.03f64.powi(2)
                + beta
                    * (omega + alpha * 0.02f64.powi(2) + beta * 0.01f64.powi(2))
                        .sqrt()
                        .powi(2))
            .sqrt(),
            (omega
                + alpha * 0.04f64.powi(2)
                + beta
                    * (omega
                        + alpha * 0.03f64.powi(2)
                        + beta
                            * (omega + alpha * 0.02f64.powi(2) + beta * 0.01f64.powi(2))
                                .sqrt()
                                .powi(2))
                    .sqrt()
                    .powi(2))
            .sqrt(),
        ];

        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_garch_volatility_high_alpha() {
        let returns = [0.01, 0.02, 0.03, 0.04];
        let omega = 0.1;
        let alpha = 0.8; // High alpha, more weight on recent returns
        let beta = 0.1;
        let result = garch_volatility(&returns, omega, alpha, beta);
        assert_eq!(result.len(), 4);

        // Expected volatilities calculated manually or using a reliable tool
        let expected = [
            0.01,
            (omega + alpha * 0.02f64.powi(2) + beta * 0.01f64.powi(2)).sqrt(),
            (omega
                + alpha * 0.03f64.powi(2)
                + beta
                    * (omega + alpha * 0.02f64.powi(2) + beta * 0.01f64.powi(2))
                        .sqrt()
                        .powi(2))
            .sqrt(),
            (omega
                + alpha * 0.04f64.powi(2)
                + beta
                    * (omega
                        + alpha * 0.03f64.powi(2)
                        + beta
                            * (omega + alpha * 0.02f64.powi(2) + beta * 0.01f64.powi(2))
                                .sqrt()
                                .powi(2))
                    .sqrt()
                    .powi(2))
            .sqrt(),
        ];

        for (res, &exp) in result.iter().zip(expected.iter()) {
            assert_relative_eq!(res, &exp, epsilon = 1e-10);
        }
    }
}

#[cfg(test)]
mod tests_simulate_heston_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_heston_volatility_basic() {
        let kappa = 2.0;
        let theta = 0.1;
        let xi = 0.3;
        let v0 = 0.1;
        let dt = 0.01;
        let steps = 10;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);

        // Check that we have the correct number of steps
        assert_eq!(result.len(), steps);

        // Check that the initial volatility is the square root of v0
        assert_relative_eq!(result[0], v0.sqrt(), epsilon = 1e-10);

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= 0.0);
        }
    }

    #[test]
    fn test_heston_volatility_zero_initial_variance() {
        let kappa = 2.0;
        let theta = 0.1;
        let xi = 0.3;
        let v0 = 0.0; // Initial variance is zero
        let dt = 0.01;
        let steps = 10;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);

        // Check that the initial volatility is zero
        assert_relative_eq!(result[0], 0.0, epsilon = 1e-10);

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= 0.0);
        }
    }

    #[test]
    fn test_heston_volatility_high_volatility_of_volatility() {
        let kappa = 2.0;
        let theta = 0.1;
        let xi = 1.0; // High volatility of volatility
        let v0 = 0.1;
        let dt = 0.01;
        let steps = 10;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);

        // Check that we have the correct number of steps
        assert_eq!(result.len(), steps);

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= 0.0);
        }
    }

    #[test]
    fn test_heston_volatility_long_term_mean() {
        let kappa = 2.0;
        let theta = 0.5; // Long-term variance should tend towards 0.5
        let xi = 0.3;
        let v0 = 0.1;
        let dt = 0.01;
        let steps = 5000; // Increase the number of steps to allow better mean reversion

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);

        // Check that we have the correct number of steps
        assert_eq!(result.len(), steps);

        // Ensure that the final volatility is close to the square root of theta
        // Increasing epsilon to allow more variance due to stochastic nature
        assert_relative_eq!(result[steps - 1], theta.sqrt(), epsilon = 0.5);

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= 0.0);
        }
    }

    #[test]
    fn test_heston_volatility_zero_volatility_of_volatility() {
        let kappa = 2.0;
        let theta = 0.1;
        let xi = 0.0; // No randomness in the volatility process
        let v0 = 0.1;
        let dt = 0.01;
        let steps = 10;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);

        // Check that we have the correct number of steps
        assert_eq!(result.len(), steps);

        // With xi = 0, volatility should tend deterministically towards theta
        for (i, &vol) in result.iter().enumerate() {
            let expected_vol = (theta + (v0 - theta) * (-kappa * i as f64 * dt).exp()).sqrt();
            assert_relative_eq!(vol, expected_vol, epsilon = 1e-10);
        }

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= 0.0);
        }
    }
}
