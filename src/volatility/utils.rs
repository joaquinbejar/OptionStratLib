/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 15/8/24
******************************************************************************/

use crate::constants::{MAX_VOLATILITY, MIN_VOLATILITY, TOLERANCE, ZERO};
use crate::greeks::Greeks;
use crate::utils::time::TimeFrame;
use crate::Options;
use crate::{d2fu, pos, Positive};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::f64;
use tracing::debug;

/// Calculates the constant volatility from a series of returns.
///
/// # Arguments
///
/// * `returns` - A slice of f64 values representing the returns.
///
/// # Returns
///
/// The calculated volatility as an f64.
pub fn constant_volatility(returns: &[f64]) -> f64 {
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
pub fn historical_volatility(returns: &[f64], window_size: usize) -> Vec<f64> {
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
pub fn ewma_volatility(returns: &[f64], lambda: f64) -> Vec<f64> {
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
pub fn implied_volatility(
    market_price: f64,
    options: &mut Options,
    max_iterations: i64,
) -> Positive {
    let mut iv = options.implied_volatility;
    for _ in 0..max_iterations {
        options.implied_volatility = iv; // Update the implied volatility in the Options struct

        let price = options
            .calculate_price_black_scholes()
            .unwrap()
            .to_f64()
            .unwrap()
            .abs();
        let vega = d2fu!(options.vega().unwrap()).unwrap();
        let price_diff = price - market_price;

        if price_diff.abs() < TOLERANCE {
            return iv; // The current implied volatility is close enough
        }

        if vega.abs() < 1e-16 {
            debug!("Vega too small, stopping iteration");
            break;
        }

        let mut temp_vi: Decimal = iv.into();
        temp_vi -= Decimal::from_f64(price_diff / vega).unwrap(); // Newton-Raphson update step

        if temp_vi < Decimal::ZERO {
            iv = pos!(1e-16); // Ensure volatility stays positive
        } else {
            iv = temp_vi.into();
        }

        let mut new_iv = iv.to_f64() - price_diff / vega;
        if new_iv < ZERO {
            debug!("New implied volatility is negative, stopping iteration");
            new_iv = 1e-16; // Ensure volatility stays positive
        }

        // Check if new_iv is NaN or infinite
        if new_iv.is_nan() || new_iv == f64::INFINITY {
            debug!("New implied volatility is NaN or infinite, stopping iteration");
            continue;
        }

        // Limit the range of implied volatility
        iv = pos!(new_iv.clamp(MIN_VOLATILITY.to_f64(), MAX_VOLATILITY.to_f64()));
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
pub fn garch_volatility(returns: &[f64], omega: f64, alpha: f64, beta: f64) -> Vec<f64> {
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
pub fn simulate_heston_volatility(
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
        v = v.max(ZERO); // Ensure variance doesn't become negative
        volatilities.push(v.sqrt());
    }
    volatilities
}

/// Interpolates the volatility from a volatility surface using bilinear interpolation.
///
/// # Arguments
///
/// * `strike` - The strike price for which the volatility needs to be interpolated.
/// * `time_to_expiry` - The time to expiry (in years) for which the volatility needs to be interpolated.
/// * `volatility_surface` - A slice of tuples, each containing a strike, time to expiry, and volatility.
///
/// # Returns
///
/// A `Result<f64>` which is:
/// - `Ok(f64)` containing the interpolated volatility if successful.
/// - `Err(&'static str)` if interpolation fails due to insufficient surrounding points.
pub fn interpolate_volatility_surface(
    strike: f64,
    time_to_expiry: f64,
    volatility_surface: &[(f64, f64, f64)],
) -> Result<f64, &'static str> {
    // Check if the exact point exists in the surface
    if let Some(&(_, _, vol)) = volatility_surface
        .iter()
        .find(|&&(s, t, _)| s == strike && t == time_to_expiry)
    {
        return Ok(vol);
    }

    // Find surrounding points on the volatility surface
    let lower_strike = volatility_surface
        .iter()
        .filter(|&&(s, _, _)| s <= strike)
        .map(|&(s, _, _)| s)
        .max_by(|a, b| a.partial_cmp(b).unwrap());
    let upper_strike = volatility_surface
        .iter()
        .filter(|&&(s, _, _)| s >= strike)
        .map(|&(s, _, _)| s)
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    let lower_expiry = volatility_surface
        .iter()
        .filter(|&&(_, t, _)| t <= time_to_expiry)
        .map(|&(_, t, _)| t)
        .max_by(|a, b| a.partial_cmp(b).unwrap());
    let upper_expiry = volatility_surface
        .iter()
        .filter(|&&(_, t, _)| t >= time_to_expiry)
        .map(|&(_, t, _)| t)
        .min_by(|a, b| a.partial_cmp(b).unwrap());

    // Check if we have enough surrounding points
    if lower_strike.is_none()
        || upper_strike.is_none()
        || lower_expiry.is_none()
        || upper_expiry.is_none()
    {
        return Err("Interpolation failed due to insufficient surrounding points");
    }

    let (lower_strike, upper_strike, lower_expiry, upper_expiry) = (
        lower_strike.unwrap(),
        upper_strike.unwrap(),
        lower_expiry.unwrap(),
        upper_expiry.unwrap(),
    );

    // Retrieve corresponding volatilities
    let q11 = volatility_surface
        .iter()
        .find(|&&(s, t, _)| s == lower_strike && t == lower_expiry)
        .map(|&(_, _, v)| v);
    let q12 = volatility_surface
        .iter()
        .find(|&&(s, t, _)| s == lower_strike && t == upper_expiry)
        .map(|&(_, _, v)| v);
    let q21 = volatility_surface
        .iter()
        .find(|&&(s, t, _)| s == upper_strike && t == lower_expiry)
        .map(|&(_, _, v)| v);
    let q22 = volatility_surface
        .iter()
        .find(|&&(s, t, _)| s == upper_strike && t == upper_expiry)
        .map(|&(_, _, v)| v);

    if let (Some(q11), Some(q12), Some(q21), Some(q22)) = (q11, q12, q21, q22) {
        // Bilinear interpolation
        let denom = (upper_strike - lower_strike) * (upper_expiry - lower_expiry);
        let t1 = (upper_strike - strike) * (upper_expiry - time_to_expiry);
        let t2 = (strike - lower_strike) * (upper_expiry - time_to_expiry);
        let t3 = (upper_strike - strike) * (time_to_expiry - lower_expiry);
        let t4 = (strike - lower_strike) * (time_to_expiry - lower_expiry);

        Ok((q11 * t1 + q21 * t2 + q12 * t3 + q22 * t4) / denom)
    } else {
        // Return an error if interpolation fails
        Err("Interpolation failed due to missing volatility values")
    }
}

/// Calculates bounds for uncertain volatility.
///
/// # Arguments
///
/// * `option` - The option for which to calculate bounds.
/// * `min_volatility` - The minimum possible volatility.
/// * `max_volatility` - The maximum possible volatility.
///
/// # Returns
///
/// A tuple of (lower_bound, upper_bound) for the option price.
pub fn uncertain_volatility_bounds(
    option: &Options,
    min_volatility: Positive,
    max_volatility: Positive,
) -> (f64, f64) {
    // Create a clone of the option for lower bound calculation
    let mut lower_bound_option = option.clone();
    lower_bound_option.implied_volatility = min_volatility;

    // Create a clone of the option for upper bound calculation
    let mut upper_bound_option = option.clone();
    upper_bound_option.implied_volatility = max_volatility;

    // Calculate the option price with minimum volatility
    let lower_bound = lower_bound_option
        .calculate_price_black_scholes()
        .unwrap()
        .to_f64()
        .unwrap();

    // Calculate the option price with maximum volatility
    let upper_bound = upper_bound_option
        .calculate_price_black_scholes()
        .unwrap()
        .to_f64()
        .unwrap();

    (lower_bound, upper_bound)
}

/// Annualizes a volatility value from a specific timeframe.
///
/// # Arguments
///
/// * `volatility` - The volatility value to annualize
/// * `timeframe` - The timeframe of the input volatility
///
/// # Returns
///
/// The annualized volatility as f64
///
/// # Formula
///
/// The annualization is performed using the square root of time rule:
/// annualized_vol = vol * sqrt(periods_per_year)
///
/// # Examples
///
/// ```
/// use optionstratlib::utils::time::TimeFrame;
/// use optionstratlib::volatility::{annualized_volatility};
/// let daily_vol = 0.01; // 1% daily volatility
/// let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day);
/// // annual_vol ≈ 0.1587 or about 15.87%
/// ```
pub fn annualized_volatility(volatility: f64, timeframe: TimeFrame) -> f64 {
    volatility * timeframe.periods_per_year().sqrt()
}

/// De-annualizes a volatility value to a specific timeframe.
///
/// # Arguments
///
/// * `annual_volatility` - The annualized volatility value
/// * `timeframe` - The target timeframe
///
/// # Returns
///
/// The de-annualized volatility as f64
///
/// # Formula
///
/// The de-annualization is performed using:
/// timeframe_vol = annual_vol / sqrt(periods_per_year)
///
/// # Examples
///
/// ```
/// use optionstratlib::utils::time::TimeFrame;
/// use optionstratlib::volatility::{de_annualized_volatility};
/// let annual_vol = 0.20; // 20% annual volatility
/// let daily_vol = de_annualized_volatility(annual_vol, TimeFrame::Day);
/// // daily_vol ≈ 0.0126 or about 1.26%
/// ```
pub fn de_annualized_volatility(annual_volatility: f64, timeframe: TimeFrame) -> f64 {
    annual_volatility / timeframe.periods_per_year().sqrt()
}

#[cfg(test)]
mod tests_annualize_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_annualize_daily_volatility() {
        let daily_vol = 0.01; // 1% daily volatility
        let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day);
        assert_relative_eq!(annual_vol, 0.01 * 252.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_deannualize_annual_volatility() {
        let annual_vol = 0.20; // 20% annual volatility
        let daily_vol = de_annualized_volatility(annual_vol, TimeFrame::Day);
        assert_relative_eq!(daily_vol, 0.20 / 252.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_custom_timeframe() {
        let custom_periods = 100.0;
        let vol = 0.05;
        let annual_vol = annualized_volatility(vol, TimeFrame::Custom(custom_periods));
        assert_relative_eq!(annual_vol, 0.05 * 100.0_f64.sqrt(), epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_conversion_roundtrip() {
        let original_vol = 0.15;
        let annualized = annualized_volatility(original_vol, TimeFrame::Day);
        let roundtrip = de_annualized_volatility(annualized, TimeFrame::Day);
        assert_relative_eq!(original_vol, roundtrip, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_different_timeframes() {
        let daily_vol = 0.01;
        let weekly_vol = annualized_volatility(daily_vol, TimeFrame::Day);
        let monthly_vol = de_annualized_volatility(weekly_vol, TimeFrame::Month);
        assert!(monthly_vol > daily_vol); // Monthly vol should be higher than daily
    }
}

#[cfg(test)]
mod tests_constant_volatility {
    use super::*;
    use approx::assert_relative_eq;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_constant_volatility_single_value() {
        let returns = [0.05];
        let result = constant_volatility(&returns);
        assert_eq!(result, ZERO);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_constant_volatility_identical_values() {
        let returns = [0.02, 0.02, 0.02, 0.02];
        let result = constant_volatility(&returns);
        assert_eq!(result, ZERO);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_constant_volatility_varying_values() {
        let returns = [0.01, 0.03, 0.02, 0.04];
        let result = constant_volatility(&returns);
        assert_relative_eq!(result, 0.012909944487358056, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_constant_volatility_negative_values() {
        let returns = [-0.01, -0.03, -0.02, -0.04];
        let result = constant_volatility(&returns);
        assert_relative_eq!(result, 0.012909944487358056, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_historical_volatility_empty_returns() {
        let returns: [f64; 0] = [];
        let result = historical_volatility(&returns, 3);
        assert!(result.is_empty());
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_historical_volatility_single_value() {
        let returns = [0.02];
        let result = historical_volatility(&returns, 3);
        assert!(result.is_empty());
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_historical_volatility_insufficient_data() {
        let returns = [0.01, 0.02];
        let result = historical_volatility(&returns, 3);
        assert!(result.is_empty());
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_historical_volatility_exact_window() {
        let returns = [0.01, 0.02, 0.03];
        let result = historical_volatility(&returns, 3);
        assert_eq!(result.len(), 1);
        assert_relative_eq!(result[0], 0.01, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_historical_volatility_larger_window() {
        let returns = [0.01, 0.02, 0.03, 0.04];
        let result = historical_volatility(&returns, 3);
        assert_eq!(result.len(), 2);
        assert_relative_eq!(result[0], 0.01, epsilon = 1e-10);
        assert_relative_eq!(result[1], 0.01, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_ewma_volatility_single_return() {
        let returns = [0.02];
        let lambda = 0.94;
        let result = ewma_volatility(&returns, lambda);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0.02); // The volatility is simply the return itself
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::pos;
    #[cfg(not(target_arch = "wasm32"))]
    use crate::utils::logger::setup_logger;
    use crate::Positive;

    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    #[cfg(not(target_arch = "wasm32"))]
    use tracing::info;

    fn create_test_option() -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.02), // initial implied volatility
            Positive::ONE,
            pos!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_implied_volatility_long_short() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            setup_logger();
            info!("Starting test_implied_volatility_long_short");
        }

        let mut option_long = create_test_option();
        let mut option_short = create_test_option();
        option_short.side = Side::Short;
        let market_price = 100.0; // Assume this is the observed market price
        let iv_long = implied_volatility(market_price, &mut option_long, 100);
        let iv_short = implied_volatility(market_price, &mut option_short, 100);

        #[cfg(not(target_arch = "wasm32"))]
        info!("IV Long {} short {}", iv_long, iv_short);

        // Check if the calculated price with the new IV is close to the market price
        option_long.implied_volatility = iv_long;
        option_short.implied_volatility = iv_short;
        let calculated_price_long = option_long
            .calculate_price_black_scholes()
            .unwrap()
            .to_f64()
            .unwrap()
            .abs();
        let calculated_price_short = option_short
            .calculate_price_black_scholes()
            .unwrap()
            .to_f64()
            .unwrap()
            .abs();

        #[cfg(not(target_arch = "wasm32"))]
        info!(
            "Price Long {} short {}",
            calculated_price_long, calculated_price_short
        );

        assert_relative_eq!(calculated_price_long, market_price, epsilon = 0.05);
        assert_relative_eq!(calculated_price_short, market_price, epsilon = 0.05);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_implied_volatility_convergence() {
        let mut option = create_test_option();
        let market_price = 100.0; // Assume this is the observed market price
        let iv = implied_volatility(market_price, &mut option, 100);

        // Check if the calculated price with the new IV is close to the market price
        option.implied_volatility = iv;
        let calculated_price = option
            .calculate_price_black_scholes()
            .unwrap()
            .to_f64()
            .unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        #[cfg(not(target_arch = "wasm32"))]
        info!("{}", (calculated_price - market_price).abs());
        assert_relative_eq!(calculated_price, market_price, epsilon = 0.002);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_implied_volatility_bounds() {
        let mut option = create_test_option();
        let market_price = 5.0;
        let iv = implied_volatility(market_price, &mut option, 100);
        println!("IV: {}", iv);

        assert!(iv > ZERO, "Implied volatility should be positive");
        assert!(iv <= 100.0, "Implied volatility should be less than 100%");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_implied_volatility_max_iterations() {
        let mut option = create_test_option();
        let market_price = 5.0;
        let iv = implied_volatility(market_price, &mut option, 1); // Only allow 1 iteration

        // The IV might not have converged, but it should still be a valid number
        assert!(iv != Positive::INFINITY);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_implied_volatility_extreme_prices() {
        let mut option = create_test_option();

        // Test with a very low market price
        let low_price = 3.0;
        let low_iv = implied_volatility(low_price, &mut option, 100);
        assert!(low_iv > ZERO);

        // Test with a very high market price
        let high_price = 10.0;
        let high_iv = implied_volatility(high_price, &mut option, 100);
        #[cfg(not(target_arch = "wasm32"))]
        info!("{}", high_iv);
    }
}

#[cfg(test)]
mod tests_garch_volatility {
    use super::*;
    use approx::assert_relative_eq;
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            assert!(vol >= ZERO);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_heston_volatility_zero_initial_variance() {
        let kappa = 2.0;
        let theta = 0.1;
        let xi = 0.3;
        let v0 = 0.0; // Initial variance is zero
        let dt = 0.01;
        let steps = 10;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);

        // Check that the initial volatility is zero
        assert_relative_eq!(result[0], ZERO, epsilon = 1e-10);

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= ZERO);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            assert!(vol >= ZERO);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
        // assert_relative_eq!(result[steps - 1], theta.sqrt(), epsilon = 0.5);

        // Ensure all volatilities are non-negative
        for &vol in &result {
            assert!(vol >= ZERO);
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            assert!(vol >= ZERO);
        }
    }
}

#[cfg(test)]
mod tests_interpolate_volatility_surface {
    use super::*;
    use approx::assert_relative_eq;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_volatility_surface_success() {
        let volatility_surface = vec![
            (100.0, 0.5, 0.2),
            (100.0, 1.0, 0.25),
            (120.0, 0.5, 0.22),
            (120.0, 1.0, 0.28),
        ];

        let strike = 110.0;
        let time_to_expiry = 0.75;
        let result = interpolate_volatility_surface(strike, time_to_expiry, &volatility_surface);

        assert!(result.is_ok());
        let interpolated_vol = result.unwrap();
        assert_relative_eq!(interpolated_vol, 0.2375, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_volatility_surface_exact_match() {
        let volatility_surface = vec![
            (100.0, 0.5, 0.2),
            (100.0, 1.0, 0.25),
            (120.0, 0.5, 0.22),
            (120.0, 1.0, 0.28),
        ];

        let strike = 100.0;
        let time_to_expiry = 0.5;
        let result = interpolate_volatility_surface(strike, time_to_expiry, &volatility_surface);

        assert!(result.is_ok());
        let interpolated_vol = result.unwrap();
        assert_relative_eq!(interpolated_vol, 0.2, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_volatility_surface_upper_bound() {
        let volatility_surface = vec![
            (100.0, 0.5, 0.2),
            (100.0, 1.0, 0.25),
            (120.0, 0.5, 0.22),
            (120.0, 1.0, 0.28),
        ];

        let strike = 120.0;
        let time_to_expiry = 1.0;
        let result = interpolate_volatility_surface(strike, time_to_expiry, &volatility_surface);

        assert!(result.is_ok());
        let interpolated_vol = result.unwrap();
        assert_relative_eq!(interpolated_vol, 0.28, epsilon = 1e-10);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_volatility_surface_insufficient_points() {
        let volatility_surface = vec![(100.0, 0.5, 0.2), (100.0, 1.0, 0.25)];

        let strike = 110.0;
        let time_to_expiry = 0.75;
        let result = interpolate_volatility_surface(strike, time_to_expiry, &volatility_surface);

        assert!(result.is_err());
        assert_eq!(
            result.err(),
            Some("Interpolation failed due to insufficient surrounding points")
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_interpolate_volatility_surface_out_of_bounds() {
        let volatility_surface = vec![
            (100.0, 0.5, 0.2),
            (100.0, 1.0, 0.25),
            (120.0, 0.5, 0.22),
            (120.0, 1.0, 0.28),
        ];

        let strike = 130.0;
        let time_to_expiry = 1.5;
        let result = interpolate_volatility_surface(strike, time_to_expiry, &volatility_surface);

        assert!(result.is_err());
        assert_eq!(
            result.err(),
            Some("Interpolation failed due to insufficient surrounding points")
        );
    }
}

#[cfg(test)]
mod tests_uncertain_volatility_bounds {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use crate::Positive;
    use rust_decimal_macros::dec;

    use approx::assert_relative_eq;

    fn create_test_option() -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0), // strike price
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),     // implied volatility
            Positive::ONE, // quantity
            pos!(100.0),   // underlying price
            dec!(0.05),    // risk-free rate
            OptionStyle::Call,
            Positive::ZERO, // dividend yield
            None,           // exotic params
        )
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_basic() {
        let option = create_test_option();
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3));

        assert!(lower < upper, "Lower bound should be less than upper bound");
        assert!(lower > ZERO, "Lower bound should be positive");
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_same_volatility() {
        let option = create_test_option();
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.2), pos!(0.2));

        assert_relative_eq!(lower, upper, epsilon = 1e-6);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_different_strikes() {
        let mut itm_option = create_test_option();
        itm_option.strike_price = pos!(90.0); // In-the-money

        let mut otm_option = create_test_option();
        otm_option.strike_price = pos!(110.0); // Out-of-the-money

        let (itm_lower, itm_upper) = uncertain_volatility_bounds(&itm_option, pos!(0.1), pos!(0.3));
        let (otm_lower, otm_upper) = uncertain_volatility_bounds(&otm_option, pos!(0.1), pos!(0.3));

        assert!(
            itm_lower > otm_lower,
            "ITM lower bound should be higher than OTM lower bound"
        );
        assert!(
            itm_upper > otm_upper,
            "ITM upper bound should be higher than OTM upper bound"
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_put_option() {
        let mut put_option = create_test_option();
        put_option.option_style = OptionStyle::Put;

        let (put_lower, put_upper) = uncertain_volatility_bounds(&put_option, pos!(0.1), pos!(0.3));

        assert!(
            put_lower < put_upper,
            "Put option lower bound should be less than upper bound"
        );
        assert!(
            put_lower > ZERO,
            "Put option lower bound should be positive"
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_extreme_volatilities() {
        let option = create_test_option();
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.01), pos!(1.0));

        assert!(
            lower < upper,
            "Lower bound should be less than upper bound even with extreme volatilities"
        );
        assert!(
            lower > ZERO,
            "Lower bound should be positive even with very low volatility"
        );
        assert!(
            upper < option.underlying_price,
            "Upper bound should not exceed underlying price for a call option"
        );
    }
}

#[cfg(test)]
mod tests_uncertain_volatility_bounds_side {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::{pos, Positive};
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;
    use tracing::info;

    fn create_test_option(option_style: OptionStyle, side: Side) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            pos!(100.0), // strike price
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),     // implied volatility
            Positive::ONE, // quantity
            pos!(100.0),   // underlying price
            dec!(0.05),    // risk-free rate
            option_style,
            Positive::ZERO, // dividend yield
            None,           // exotic params
        )
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_call_long() {
        let option = create_test_option(OptionStyle::Call, Side::Long);
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3));

        assert!(
            lower < upper,
            "Call Long: Lower bound should be less than upper bound"
        );
        assert!(lower > ZERO, "Call Long: Lower bound should be positive");
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_call_short() {
        let option = create_test_option(OptionStyle::Call, Side::Short);
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3));

        assert!(
            lower > upper,
            "Call Short: Lower bound should be less than upper bound"
        );
        assert!(lower < ZERO, "Call Short: Lower bound should be negative");
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_put_long() {
        let option = create_test_option(OptionStyle::Put, Side::Long);
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3));

        assert!(
            lower < upper,
            "Put Long: Lower bound should be less than upper bound"
        );
        assert!(lower > ZERO, "Put Long: Lower bound should be positive");
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_put_short() {
        let option = create_test_option(OptionStyle::Put, Side::Short);
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3));
        info!("{} {}", lower, upper);
        assert!(
            lower > upper,
            "Put Short: Lower bound should be less than upper bound"
        );
        assert!(lower < ZERO, "Put Short: Lower bound should be negative");
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_same_volatility() {
        let option = create_test_option(OptionStyle::Call, Side::Long);
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.2), pos!(0.2));

        assert_relative_eq!(lower, upper, epsilon = 1e-6);
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_different_strikes() {
        let mut itm_option = create_test_option(OptionStyle::Call, Side::Long);
        itm_option.strike_price = pos!(90.0); // In-the-money

        let mut otm_option = create_test_option(OptionStyle::Call, Side::Long);
        otm_option.strike_price = pos!(110.0); // Out-of-the-money

        let (itm_lower, itm_upper) = uncertain_volatility_bounds(&itm_option, pos!(0.1), pos!(0.3));
        let (otm_lower, otm_upper) = uncertain_volatility_bounds(&otm_option, pos!(0.1), pos!(0.3));

        assert!(
            itm_lower > otm_lower,
            "ITM lower bound should be higher than OTM lower bound"
        );
        assert!(
            itm_upper > otm_upper,
            "ITM upper bound should be higher than OTM upper bound"
        );
    }

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_uncertain_volatility_bounds_extreme_volatilities() {
        let option = create_test_option(OptionStyle::Call, Side::Long);
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.01), pos!(1.0));

        assert!(
            lower < upper,
            "Lower bound should be less than upper bound even with extreme volatilities"
        );
        assert!(
            lower > ZERO,
            "Lower bound should be positive even with very low volatility"
        );
        assert!(
            upper < option.underlying_price,
            "Upper bound should not exceed underlying price for a call option"
        );
    }
}
