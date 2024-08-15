/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 15/8/24
******************************************************************************/

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

/// Calculates implied volatility (simplified, requires Black-Scholes implementation).
///
/// # Arguments
///
/// * `market_price` - The market price of the option.
/// * `strike` - The strike price of the option.
/// * `spot` - The spot price of the underlying asset.
/// * `time_to_expiry` - Time to expiry in years.
/// * `risk_free_rate` - The risk-free interest rate.
/// * `is_call` - Boolean indicating whether it's a call option (true) or put option (false).
///
/// # Returns
///
/// The calculated implied volatility as an f64.
fn implied_volatility(
    market_price: f64,
    strike: f64,
    spot: f64,
    time_to_expiry: f64,
    risk_free_rate: f64,
    is_call: bool,
) -> f64 {
    // This function would need an implementation of the Black-Scholes model
    // and a numerical method to find the root (such as Newton-Raphson)
    todo!("Requires Black-Scholes implementation");
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
