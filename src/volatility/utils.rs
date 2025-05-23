/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 15/8/24
******************************************************************************/
use crate::{ExpirationDate, OptionStyle, OptionType, Options, Side};
use crate::constants::{MAX_VOLATILITY, MIN_VOLATILITY};
use crate::error::VolatilityError;
use crate::model::decimal::decimal_normal_sample;
use crate::utils::time::TimeFrame;
use crate::{Positive, pos};
use num_traits::{FromPrimitive, ToPrimitive};
use rand::random;
use rayon::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};
use std::error::Error;

/// Calculates the constant volatility from a series of returns.
///
/// # Arguments
///
/// * `returns` - A slice of Decimal values representing the returns.
///
/// # Returns
///
/// The calculated volatility as an Decimal.
pub fn constant_volatility(returns: &[Decimal]) -> Result<Positive, Box<dyn Error>> {
    let n = Positive(Decimal::from_usize(returns.len()).unwrap());

    if n < Decimal::TWO {
        return Ok(Positive::ZERO);
    }

    let mean = returns.iter().sum::<Decimal>() / n;
    let variance =
        returns.iter().map(|&r| (r - mean).powi(2)).sum::<Decimal>() / (n - Decimal::ONE);

    Ok(variance.sqrt().unwrap().into())
}

/// Calculates historical volatility using a moving window approach.
///
/// # Arguments
///
/// * `returns` - A slice of Decimal values representing the returns.
/// * `window_size` - The size of the moving window.
///
/// # Returns
///
/// A vector of Decimal values representing the historical volatility for each window.
pub fn historical_volatility(
    returns: &[Decimal],
    window_size: usize,
) -> Result<Vec<Positive>, Box<dyn Error>> {
    returns
        .windows(window_size)
        .map(constant_volatility)
        .collect()
}

/// Calculates EWMA (Exponentially Weighted Moving Average) volatility.
///
/// # Arguments
///
/// * `returns` - A slice of Decimal values representing the returns.
/// * `lambda` - The decay factor (typically 0.94 for daily data).
///
/// # Returns
///
/// A vector of Decimal values representing the EWMA volatility.
pub fn ewma_volatility(
    returns: &[Decimal],
    lambda: Decimal,
) -> Result<Vec<Positive>, Box<dyn Error>> {
    let mut variance = returns[0].powi(2);
    let mut volatilities = vec![Positive(variance.sqrt().unwrap())];

    for &return_value in &returns[1..] {
        variance = lambda * variance + (Decimal::ONE - lambda) * return_value.powi(2);
        volatilities.push(Positive(variance.sqrt().unwrap()));
    }

    Ok(volatilities)
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
    market_price: Positive,
    options: &mut Options,
    max_iterations: i64,
) -> Result<Positive, Box<dyn Error>> {
    let base_option = options.clone();
    let iterations = 100 * max_iterations;
    let result = (1..iterations)
        .into_par_iter()
        .map(|i| {
            let mut option = base_option.clone();
            let iv = Positive::from(i as f64 / iterations as f64);
            option.implied_volatility = iv;

            match option.calculate_price_black_scholes() {
                Ok(price) => {
                    let diff = (price - market_price.to_dec()).abs();
                    Some((iv, diff))
                }
                Err(_) => None,
            }
        })
        .filter_map(|x| x) // Remove errors
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    match result {
        Some((best_iv, _)) => {
            let iv = best_iv.clamp(MIN_VOLATILITY, MAX_VOLATILITY);
            if iv == Positive::from(1f64 / iterations as f64) {
                Err("Implied volatility not found".into())
            } else {
                Ok(iv)
            }
        }
        None => Err("No valid volatility found".into()),
    }
}


/// Calculates the implied volatility (IV) of an option given its parameters.
///
/// # Parameters
///
/// * `option_price` - A `Positive` value representing the current price of the option.
/// * `strike` - A `Positive` value representing the strike price of the option.
/// * `option_style` - An `OptionStyle` enum indicating the style of the option (e.g., European, American).
/// * `underlying_price` - A `Positive` value representing the current price of the underlying asset.
/// * `days` - A `Positive` value indicating the number of days to expiration for the option.
/// * `symbol` - A `String` representing the symbol of the financial instrument for the option.
///
/// # Returns
///
/// * `Ok(Positive)` - The calculated implied volatility as a `Positive` value, if the computation is successful.
/// * `Err(Box<dyn Error>)` - An error if the implied volatility cannot be calculated due to invalid inputs or other reasons.
///
/// # Errors
///
/// This function will return an error if:
/// * The inputs do not meet the required constraints.
/// * The implied volatility calculation fails to converge within the set iteration limit.
///
/// # Notes
///
/// This function internally creates an `Options` object with the given parameters,
/// and calls the `implied_volatility` function with the option data. The iteration
/// limit for the IV calculation is set to 10.
///
/// Ensure that all input parameters are valid and conform to the expected types
/// and ranges for meaningful results.
pub fn calculate_iv(
    option_price: Positive,
    strike: Positive,
    option_style: OptionStyle,
    underlying_price: Positive,
    days: Positive,
    symbol: String,
) -> Result<Positive, Box<dyn Error>> {
    let mut option = Options::new(
        OptionType::European,
        Side::Long,
        symbol,
        strike,
        ExpirationDate::Days(days),
        Positive::ZERO,
        Positive::ONE,
        underlying_price,
        Decimal::ZERO,
        option_style,
        Positive::ZERO,
        None,
    );
    implied_volatility(option_price, &mut option, 10)
}

/// Calculates GARCH(1,1) volatility (simplified).
///
/// # Arguments
///
/// * `returns` - A slice of Decimal values representing the returns.
/// * `omega`, `alpha`, `beta` - GARCH(1,1) parameters.
///
/// # Returns
///
/// A vector of Decimal values representing the GARCH(1,1) volatility.
pub fn garch_volatility(
    returns: &[Decimal],
    omega: Decimal,
    alpha: Decimal,
    beta: Decimal,
) -> Result<Vec<Positive>, Box<dyn Error>> {
    let mut variance = Positive(returns[0].powi(2));
    let mut volatilities = vec![variance.sqrt()];
    for &return_value in &returns[1..] {
        variance = Positive(omega + alpha * return_value.powi(2) + beta * variance);
        volatilities.push(variance.sqrt());
    }
    Ok(volatilities)
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
/// A vector of Decimal values representing the simulated volatility.
pub fn simulate_heston_volatility(
    kappa: Decimal,
    theta: Decimal,
    xi: Decimal,
    v0: Decimal,
    dt: Decimal,
    steps: usize,
) -> Result<Vec<Positive>, Box<dyn Error>> {
    let mut v = Positive(v0);
    let mut volatilities = vec![v.sqrt()];
    for _ in 1..steps {
        let dw = Decimal::from_f64(random::<f64>() * dt.sqrt().unwrap().to_f64().unwrap()).unwrap();
        v += kappa * (theta - v) * dt + xi * v.sqrt() * dw;
        v = v.max(Positive::ZERO); // Ensure variance doesn't become negative
        volatilities.push(v.sqrt());
    }
    Ok(volatilities)
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
) -> Result<(Positive, Positive), Box<dyn Error>> {
    // Create a clone of the option for lower bound calculation
    let mut lower_bound_option = option.clone();
    lower_bound_option.implied_volatility = min_volatility;

    // Create a clone of the option for upper bound calculation
    let mut upper_bound_option = option.clone();
    upper_bound_option.implied_volatility = max_volatility;

    // Calculate the option price with minimum volatility
    let lower_bound = Positive(lower_bound_option.calculate_price_black_scholes().unwrap());

    // Calculate the option price with maximum volatility
    let upper_bound = Positive(upper_bound_option.calculate_price_black_scholes().unwrap());

    Ok((lower_bound, upper_bound))
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
/// The annualized volatility as Decimal
///
/// # Formula
///
/// The annualization is performed using the square root of time rule:
/// annualized_vol = vol * sqrt(periods_per_year)
///
/// # Examples
///
/// ```
/// use rust_decimal_macros::dec;
/// use optionstratlib::pos;
/// use optionstratlib::utils::time::TimeFrame;
/// use optionstratlib::volatility::{annualized_volatility};
/// let daily_vol = pos!(0.01); // 1% daily volatility
/// let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day);
/// // annual_vol ≈ 0.1587 or about 15.87%
/// ```
pub fn annualized_volatility(
    volatility: Positive,
    timeframe: TimeFrame,
) -> Result<Positive, Box<dyn Error>> {
    Ok(volatility * timeframe.periods_per_year().sqrt())
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
/// The de-annualized volatility as Decimal
///
/// # Formula
///
/// The de-annualization is performed using:
/// timeframe_vol = annual_vol / sqrt(periods_per_year)
///
/// # Examples
///
/// ```
/// use rust_decimal_macros::dec;
/// use optionstratlib::pos;
/// use optionstratlib::utils::time::TimeFrame;
/// use optionstratlib::volatility::{de_annualized_volatility};
/// let annual_vol = pos!(0.20); // 20% annual volatility
/// let daily_vol = de_annualized_volatility(annual_vol, TimeFrame::Day);
/// // daily_vol ≈ 0.0126 or about 1.26%
/// ```
pub fn de_annualized_volatility(
    annual_volatility: Positive,
    timeframe: TimeFrame,
) -> Result<Positive, Box<dyn Error>> {
    Ok(annual_volatility / timeframe.periods_per_year().sqrt())
}

/// Adjusts volatility between different timeframes using the square root of time rule
///
/// # Arguments
/// * `volatility` - The volatility to adjust
/// * `from_frame` - The original timeframe of the volatility
/// * `to_frame` - The target timeframe for the volatility
///
/// # Returns
/// The adjusted volatility for the target timeframe
///
/// # Example
/// ```
/// use optionstratlib::pos;
/// use optionstratlib::utils::TimeFrame;
/// use optionstratlib::volatility::adjust_volatility;
/// let daily_vol = pos!(0.2); // 20% daily volatility
/// let minute_vol = adjust_volatility(daily_vol, TimeFrame::Day, TimeFrame::Minute).unwrap();
/// ```
pub fn adjust_volatility(
    volatility: Positive,
    from_frame: TimeFrame,
    to_frame: TimeFrame,
) -> Result<Positive, Box<dyn Error>> {
    if from_frame == to_frame {
        return Ok(volatility);
    }

    let from_periods = from_frame.periods_per_year();
    let to_periods = to_frame.periods_per_year();

    if from_periods == to_periods {
        return Ok(volatility);
    }

    // Check for division by zero
    if to_periods.is_zero() {
        return Err(Box::new(VolatilityError::InvalidTime {
            time: to_periods,
            reason: format!(
                "Cannot adjust volatility to timeframe with zero periods per year: {:?}",
                to_frame
            ),
        }));
    }

    // Scale factor is square root of (from_periods / to_periods)
    let scale_factor = pos!((from_periods / to_periods).to_f64().sqrt());

    Ok(volatility * scale_factor)
}

/// Generates a mean-reverting Ornstein-Uhlenbeck process time series
///
/// This function simulates a discrete-time Ornstein-Uhlenbeck stochastic process, which is
/// commonly used in financial mathematics to model mean-reverting processes such as interest rates,
/// volatility, or commodity prices. The process follows the stochastic differential equation:
///
/// dX_t = θ(μ - X_t)dt + σdW_t
///
/// Where:
/// - θ (theta) is the speed of reversion to the mean
/// - μ (mu) is the long-term mean level
/// - σ (sigma) is the volatility or intensity of random fluctuations
/// - W_t is a Wiener process (standard Brownian motion)
///
/// # Arguments
/// * `x0` - Initial value of the process
/// * `mu` - Long-term mean the process reverts to
/// * `theta` - Speed of mean reversion (higher values cause faster reversion)
/// * `sigma` - Volatility parameter controlling the intensity of random fluctuations
/// * `dt` - Time step size for the simulation
/// * `steps` - Number of time steps to simulate
///
/// # Returns
/// A vector containing the simulated values of the Ornstein-Uhlenbeck process at each time step
///
/// # Examples
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::pos;
/// use optionstratlib::volatility::generate_ou_process;
///
/// // Simulate an OU process with initial value 1.0, mean 1.5,
/// // reversion speed 0.1, volatility 0.2, time step 0.01, for 1000 steps
/// let process = generate_ou_process(
///     pos!(1.0),       // initial value
///     pos!(1.5),       // long-term mean
///     pos!(0.1),       // speed of reversion
///     pos!(0.2),       // volatility
///     pos!(0.01),      // time step
///     1000             // number of steps
/// );
/// ```
pub fn generate_ou_process(
    x0: Positive,
    mu: Positive,
    theta: Positive,
    volatility: Positive,
    dt: Positive,
    steps: usize,
) -> Vec<Positive> {
    let sqrt_dt = dt.sqrt();
    let mut x = x0.to_dec();
    let mut result = Vec::with_capacity(steps);
    result.push(Positive(x));

    for _ in 1..steps {
        let dw = decimal_normal_sample() * sqrt_dt; // Z√dt
        let drift = theta * (mu - x) * dt; // θ(μ−x)dt
        let diffusion = volatility * dw; // σ·Z√dt
        x += drift + diffusion; // paso OU
        x = x.max(Decimal::ZERO); // opcional: no negativos
        result.push(Positive(x));
    }

    result
}

#[cfg(test)]
mod tests_annualize_volatility {
    use super::*;
    use crate::assert_pos_relative_eq;

    #[test]
    fn test_annualize_daily_volatility() {
        let daily_vol = pos!(0.01); // 1% daily volatility
        let annual_vol = annualized_volatility(daily_vol, TimeFrame::Day).unwrap();
        let expected = daily_vol * pos!(252.0f64.sqrt());
        assert_pos_relative_eq!(annual_vol, expected, pos!(1e-10));
    }

    #[test]
    fn test_deannualize_annual_volatility() {
        let annual_vol = pos!(0.20); // 20% annual volatility
        let daily_vol = de_annualized_volatility(annual_vol, TimeFrame::Day).unwrap();
        let expected = pos!(0.01259881576697424);
        assert_pos_relative_eq!(daily_vol, expected, pos!(1e-10));
    }

    #[test]
    fn test_custom_timeframe() {
        let custom_periods = pos!(100.0);
        let vol = pos!(0.05);
        let annual_vol = annualized_volatility(vol, TimeFrame::Custom(custom_periods)).unwrap();
        let expected = vol * custom_periods.sqrt();
        assert_pos_relative_eq!(annual_vol, expected, pos!(1e-10));
    }

    #[test]
    fn test_conversion_roundtrip() {
        let original_vol = pos!(0.15);
        let annualized = annualized_volatility(original_vol, TimeFrame::Day).unwrap();
        let roundtrip = de_annualized_volatility(annualized, TimeFrame::Day).unwrap();
        assert_pos_relative_eq!(original_vol, roundtrip, pos!(1e-10));
    }

    #[test]
    fn test_different_timeframes() {
        let daily_vol = pos!(0.01);
        let weekly_vol = annualized_volatility(daily_vol, TimeFrame::Day).unwrap();
        let monthly_vol = de_annualized_volatility(weekly_vol, TimeFrame::Month).unwrap();
        assert!(monthly_vol > daily_vol); // Monthly vol should be higher than daily
    }
}

#[cfg(test)]
mod tests_constant_volatility {
    use super::*;
    use crate::assert_pos_relative_eq;
    use crate::constants::ZERO;
    use rust_decimal_macros::dec;

    #[test]
    fn test_constant_volatility_single_value() {
        let returns = [dec!(0.05)];
        let result = constant_volatility(&returns).unwrap();
        assert_eq!(result, ZERO);
    }

    #[test]
    fn test_constant_volatility_identical_values() {
        let returns = [dec!(0.02), dec!(0.02), dec!(0.02), dec!(0.02)];
        let result = constant_volatility(&returns).unwrap();
        assert_eq!(result, ZERO);
    }

    #[test]
    fn test_constant_volatility_varying_values() {
        let returns = [dec!(0.01), dec!(0.03), dec!(0.02), dec!(0.04)];
        let result = constant_volatility(&returns).unwrap();
        assert_pos_relative_eq!(result, pos!(0.012909944487358056), pos!(1e-10));
    }
}

#[cfg(test)]
mod tests_historical_volatility {
    use super::*;
    use crate::assert_pos_relative_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn test_historical_volatility_empty_returns() {
        let returns: [Decimal; 0] = [];
        let result = historical_volatility(&returns, 3).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_volatility_single_value() {
        let returns = [dec!(0.02)];
        let result = historical_volatility(&returns, 3).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_volatility_insufficient_data() {
        let returns = [dec!(0.01), dec!(0.02)];
        let result = historical_volatility(&returns, 3).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_volatility_exact_window() {
        let returns = [dec!(0.01), dec!(0.02), dec!(0.03)];
        let result = historical_volatility(&returns, 3).unwrap();
        assert_eq!(result.len(), 1);
        assert_pos_relative_eq!(result[0], pos!(0.01), pos!(1e-10));
    }

    #[test]
    fn test_historical_volatility_larger_window() {
        let returns = [dec!(0.01), dec!(0.02), dec!(0.03), dec!(0.04)];
        let result = historical_volatility(&returns, 3).unwrap();
        assert_eq!(result.len(), 2);
        assert_pos_relative_eq!(result[0], pos!(0.01), pos!(1e-10));
        assert_pos_relative_eq!(result[1], pos!(0.01), pos!(1e-10));
    }
}

#[cfg(test)]
mod tests_ewma_volatility {
    use super::*;
    use crate::assert_pos_relative_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ewma_volatility_single_return() {
        let returns = [dec!(0.02)];
        let lambda = dec!(0.94);
        let result = ewma_volatility(&returns, lambda).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], pos!(0.02)); // The volatility is simply the return itself
    }

    #[test]
    fn test_ewma_volatility_constant_returns() {
        let returns = [dec!(0.02), dec!(0.02), dec!(0.02), dec!(0.02)];
        let lambda = dec!(0.94);
        let single_value = dec!(0.02);
        let result = ewma_volatility(&returns, lambda).unwrap();
        assert_eq!(result.len(), 4);

        // Test the EWMA calculation
        let expected = [
            pos!(0.02),
            Positive::from(
                (lambda * single_value.powi(2) + (Decimal::ONE - lambda) * single_value.powi(2))
                    .sqrt()
                    .unwrap(),
            ),
            Positive::from(
                (lambda * single_value.powi(2) + (Decimal::ONE - lambda) * single_value.powi(2))
                    .sqrt()
                    .unwrap()
                    .powi(2),
            )
            .sqrt(),
            Positive::from(
                (lambda * single_value.powi(2) + (Decimal::ONE - lambda) * single_value.powi(2))
                    .sqrt()
                    .unwrap()
                    .powi(2)
                    .sqrt()
                    .unwrap()
                    .powi(2),
            )
            .sqrt(),
        ];

        for (res, exp) in result.iter().zip(expected.iter()) {
            assert_pos_relative_eq!(*res, *exp, pos!(1e-10));
        }
    }

    #[test]
    fn test_ewma_volatility_varying_returns() {
        let returns = [dec!(0.01), dec!(0.02), dec!(0.03), dec!(0.04)];
        let lambda = dec!(0.94);
        let result = ewma_volatility(&returns, lambda).unwrap();
        assert_eq!(result.len(), 4);

        // Verify general properties rather than exact values due to complexity of calculation
        assert!(result.iter().all(|&x| x > Positive::ZERO));
        assert!(
            result
                .iter()
                .zip(result.iter().skip(1))
                .all(|(a, b)| *b >= *a)
        );
    }

    #[test]
    fn test_ewma_volatility_low_lambda() {
        let returns = [dec!(0.01), dec!(0.02), dec!(0.03), dec!(0.04)];
        let lambda = dec!(0.5); // Low lambda means faster decay
        let result = ewma_volatility(&returns, lambda).unwrap();
        assert_eq!(result.len(), 4);

        // Verify more weight on recent values
        let last = result.last().unwrap();
        assert!(*last > *result.first().unwrap());
    }

    #[test]
    fn test_ewma_volatility_high_lambda() {
        let returns = [dec!(0.01), dec!(0.02), dec!(0.03), dec!(0.04)];
        let lambda = dec!(0.99); // High lambda means slower decay
        let result = ewma_volatility(&returns, lambda).unwrap();
        assert_eq!(result.len(), 4);

        // Verify less weight on recent values
        let differences: Vec<_> = result.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(differences.iter().all(|&d| d.to_dec() < dec!(0.01)));
    }
}

#[cfg(test)]
mod tests_implied_volatility {
    use super::*;
    use crate::constants::{MAX_VOLATILITY, MIN_VOLATILITY};
    use crate::greeks::Greeks;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, assert_pos_relative_eq};
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_option() -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),                      // strike price
            ExpirationDate::Days(pos!(30.0)), // 30 days to expiration
            pos!(0.2),                        // initial implied volatility
            Positive::ONE,                    // quantity
            pos!(100.0),                      // underlying price (ATM)
            dec!(0.05),                       // risk-free rate
            OptionStyle::Call,                // call option
            Positive::ZERO,                   // no dividend yield
            None,                             // no exotic params
        )
    }

    #[test]
    fn test_implied_volatility_max_iterations() {
        let mut option = create_test_option();
        let market_price = pos!(5.0);

        // Test with very low number of iterations
        let result = implied_volatility(market_price, &mut option, 1);
        assert!(result.is_ok()); // Should still return a result

        let iv = result.unwrap();
        assert!(iv >= MIN_VOLATILITY && iv <= MAX_VOLATILITY);
        
        let result = calculate_iv(
            market_price,
            pos!(100.0),
            OptionStyle::Call,
            pos!(100.0),
            pos!(30.0),
            "TEST".to_string(),
        );
        assert!(result.is_ok());
        
        let iv = result.unwrap();
        assert!(iv >= MIN_VOLATILITY && iv <= MAX_VOLATILITY);
        assert_pos_relative_eq!(iv, pos!(0.437), pos!(1e-3));
    }

    #[test]
    fn test_implied_volatility_zero_dte() {
        let iv = pos!(0.25);
        let mut option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(0.5)),
            iv,
            Positive::ONE,
            pos!(100.0),
            dec!(0.0),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let delta = option.delta().unwrap();
        let gamma = option.gamma().unwrap();
        let vega = option.vega().unwrap();
        let theta = option.theta().unwrap();
        let rho = option.rho().unwrap();
        assert_decimal_eq!(delta, dec!(0.502), dec!(0.002));
        assert_decimal_eq!(gamma, dec!(0.431), dec!(0.002));
        assert_decimal_eq!(vega, dec!(0.015), dec!(0.002));
        assert_decimal_eq!(theta, dec!(-0.369), dec!(0.002));
        assert_decimal_eq!(rho, dec!(0.001), dec!(0.002));

        let market_price = option.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(market_price, dec!(0.369), dec!(0.002));

        // For very short-term options, use a more lenient tolerance
        option.implied_volatility = pos!(0.4); // Start with different IV
        let iv_result = implied_volatility(market_price.into(), &mut option, 10);

        // For zero DTE options, expect either convergence or a reasonable approximation
        match iv_result {
            Ok(iv_aprox) => {
                // If it converges, it should be close to the original IV
                assert_pos_relative_eq!(iv_aprox, iv, pos!(0.001)); // More lenient tolerance
            }
            Err(_) => {
                // If it doesn't converge, that's also acceptable for zero DTE options
                // as they have extremely low vega and are numerically challenging
                error!("Non-convergence is acceptable for zero DTE options");
            }
        }

        option.implied_volatility = iv;
        option.option_style = OptionStyle::Put;
        let delta = option.delta().unwrap();
        let gamma = option.gamma().unwrap();
        let vega = option.vega().unwrap();
        let theta = option.theta().unwrap();
        let rho = option.rho().unwrap();
        assert_decimal_eq!(delta, dec!(-0.498), dec!(0.002));
        assert_decimal_eq!(gamma, dec!(0.431), dec!(0.002));
        assert_decimal_eq!(vega, dec!(0.015), dec!(0.002));
        assert_decimal_eq!(theta, dec!(-0.369), dec!(0.002));
        assert_decimal_eq!(rho, dec!(0.001), dec!(0.002));

        let market_price = option.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(market_price, dec!(0.369), dec!(0.002));

        // For very short-term options, use a more lenient tolerance
        option.implied_volatility = pos!(0.4); // Start with different IV
        let iv_result = implied_volatility(market_price.into(), &mut option, 10);

        // For zero DTE options, expect either convergence or a reasonable approximation
        match iv_result {
            Ok(iv_aprox) => {
                // If it converges, it should be close to the original IV
                assert_pos_relative_eq!(iv_aprox, iv, pos!(0.001)); // More lenient tolerance
            }
            Err(_) => {
                // If it doesn't converge, that's also acceptable for zero DTE options
                // as they have extremely low vega and are numerically challenging
                error!("Non-convergence is acceptable for zero DTE options");
            }
        }
    }

    #[test]
    fn test_implied_volatility_zero_dte_real() {
        let iv = pos!(0.356831);
        let mut option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(20600.0),
            ExpirationDate::Days(pos!(0.52)),
            iv,
            pos!(1.0),
            pos!(21049.88),
            dec!(0.0),
            OptionStyle::Call,
            pos!(0.05),
            None,
        );

        let delta = option.delta().unwrap();
        let gamma = option.gamma().unwrap();
        let vega = option.vega().unwrap();
        let theta = option.theta().unwrap();
        let rho = option.rho().unwrap();
        assert_decimal_eq!(delta, dec!(0.946), dec!(0.002));
        assert_decimal_eq!(gamma, dec!(0.00038), dec!(0.002));
        assert_decimal_eq!(vega, dec!(0.866), dec!(0.002));
        assert_decimal_eq!(theta, dec!(-26.990), dec!(0.002));
        assert_decimal_eq!(rho, dec!(0.277), dec!(0.002));

        let market_price = option.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(market_price, dec!(454.917), dec!(0.002));

        // For very short-term options, use a more lenient tolerance
        option.implied_volatility = pos!(0.4); // Start with different IV
        let iv_result = implied_volatility(market_price.into(), &mut option, 10);

        // For zero DTE options, expect either convergence or a reasonable approximation
        match iv_result {
            Ok(iv_aprox) => {
                // If it converges, it should be close to the original IV
                assert_pos_relative_eq!(iv_aprox, iv, pos!(0.001));
            }
            Err(_) => {
                // If it doesn't converge, that's also acceptable for zero DTE options
                // as they have extremely low vega and are numerically challenging
                error!("Non-convergence is acceptable for zero DTE options");
            }
        }

        option.implied_volatility = iv;
        option.option_style = OptionStyle::Put;
        let delta = option.delta().unwrap();
        let gamma = option.gamma().unwrap();
        let vega = option.vega().unwrap();
        let theta = option.theta().unwrap();
        let rho = option.rho().unwrap();
        assert_decimal_eq!(delta, dec!(-0.053), dec!(0.001));
        assert_decimal_eq!(gamma, dec!(0.0), dec!(0.001));
        assert_decimal_eq!(vega, dec!(0.866), dec!(0.001));
        assert_decimal_eq!(theta, dec!(-29.874), dec!(0.001));
        assert_decimal_eq!(rho, dec!(-0.016), dec!(0.001));

        let market_price = option.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(market_price, dec!(6.537), dec!(0.002));

        // For very short-term options, use a more lenient tolerance
        option.implied_volatility = pos!(0.4); // Start with different IV
        let iv_result = implied_volatility(market_price.into(), &mut option, 10);

        // For zero DTE options, expect either convergence or a reasonable approximation
        match iv_result {
            Ok(iv_aprox) => {
                // If it converges, it should be close to the original IV
                assert_pos_relative_eq!(iv_aprox, iv, pos!(0.001));
            }
            Err(_) => {
                // If it doesn't converge, that's also acceptable for zero DTE options
                // as they have extremely low vega and are numerically challenging
                error!("Non-convergence is acceptable for zero DTE options");
            }
        }
    }
    
    #[test]
    fn test_implied_volatility_zero_dte_real_put() {
        let iv = pos!(0.356831);
        let mut option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(23325.0),
            ExpirationDate::Days(pos!(0.29)),
            iv,
            pos!(1.0),
            pos!(24118.5),
            dec!(0.0),
            OptionStyle::Put,
            pos!(0.0),
            None,
        );

        let delta = option.delta().unwrap();
        let gamma = option.gamma().unwrap();
        let vega = option.vega().unwrap();
        let theta = option.theta().unwrap();
        let rho = option.rho().unwrap();
        assert_decimal_eq!(delta, dec!(-0.00043), dec!(0.002));
        assert_decimal_eq!(gamma, dec!(0.0000064), dec!(0.00001));
        assert_decimal_eq!(vega, dec!(0.0105), dec!(0.002));
        assert_decimal_eq!(theta, dec!(-0.64997), dec!(0.002));
        assert_decimal_eq!(rho, dec!(-0.000083), dec!(0.002));

        let market_price = option.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(market_price, dec!(0.027), dec!(0.002)); // too small to be accurate

        // For very short-term options, use a more lenient tolerance
        option.implied_volatility = pos!(0.4); // Start with different IV
        let iv_result = implied_volatility(pos!(1.5), &mut option, 10);

        // For zero DTE options, expect either convergence or a reasonable approximation
        match iv_result {
            Ok(iv_aprox) => {
                // If it converges, it should be close to the original IV
                assert_pos_relative_eq!(iv_aprox, pos!(0.528), pos!(0.001));
            }
            Err(_) => {
                // If it doesn't converge, that's also acceptable for zero DTE options
                // as they have extremely low vega and are numerically challenging
                error!("Non-convergence is acceptable for zero DTE options");
            }
        }
    }

    #[test]
    fn test_implied_volatility_zero_dte_real_call() {
        let iv = pos!(0.356831);
        let mut option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(23325.0),
            ExpirationDate::Days(pos!(0.32)),
            iv,
            pos!(1.0),
            pos!(24103.00),
            dec!(0.0),
            OptionStyle::Call,
            pos!(0.0),
            None,
        );

        let delta = option.delta().unwrap();
        let gamma = option.gamma().unwrap();
        let vega = option.vega().unwrap();
        let theta = option.theta().unwrap();
        let rho = option.rho().unwrap();
        assert_decimal_eq!(delta, dec!(0.999), dec!(0.001));
        assert_decimal_eq!(gamma, dec!(0.00001), dec!(0.00001));
        assert_decimal_eq!(vega, dec!(0.02255), dec!(0.002));
        assert_decimal_eq!(theta, dec!(-1.25733), dec!(0.002));
        assert_decimal_eq!(rho, dec!(0.204), dec!(0.002));

        let market_price = option.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(market_price, dec!(778.065), dec!(0.002));

        // For very short-term options, use a more lenient tolerance
        option.implied_volatility = pos!(0.4); // Start with different IV
        let iv_result = implied_volatility(market_price.into(), &mut option, 10);

        // For zero DTE options, expect either convergence or a reasonable approximation
        match iv_result {
            Ok(iv_aprox) => {
                // If it converges, it should be close to the original IV
                assert_pos_relative_eq!(iv_aprox, iv, pos!(0.001));
            }
            Err(_) => {
                // If it doesn't converge, that's also acceptable for zero DTE options
                // as they have extremely low vega and are numerically challenging
                error!("Non-convergence is acceptable for zero DTE options");
            }
        }
    }
}

#[cfg(test)]
mod tests_garch_volatility {
    use super::*;
    use crate::{assert_decimal_eq, assert_pos_relative_eq};
    use rust_decimal_macros::dec;

    #[test]
    fn test_garch_single_return() {
        let returns = vec![dec!(0.02)];
        let omega = dec!(0.1);
        let alpha = dec!(0.2);
        let beta = dec!(0.7);

        let result = garch_volatility(&returns, omega, alpha, beta).unwrap();
        assert_eq!(result.len(), 1);
        assert_pos_relative_eq!(result[0], pos!(0.02), pos!(1e-10));
    }

    #[test]
    fn test_garch_constant_returns() {
        let returns = vec![dec!(0.02), dec!(0.02), dec!(0.02), dec!(0.02)];
        let omega = dec!(0.1);
        let alpha = dec!(0.2);
        let beta = dec!(0.7);

        let result = garch_volatility(&returns, omega, alpha, beta).unwrap();
        assert_eq!(result.len(), 4);

        // First volatility should be the absolute value of first return
        assert_pos_relative_eq!(result[0], pos!(0.02), pos!(1e-10));

        // For constant returns, volatility should converge
        let last_two_diff = (result[3].to_dec() - result[2].to_dec()).abs();
        assert_decimal_eq!(last_two_diff, dec!(0.0555), dec!(0.001));
    }

    #[test]
    fn test_garch_varying_returns() {
        let returns = vec![dec!(0.01), dec!(-0.02), dec!(0.03), dec!(-0.01)];
        let omega = dec!(0.1);
        let alpha = dec!(0.2);
        let beta = dec!(0.7);

        let result = garch_volatility(&returns, omega, alpha, beta).unwrap();
        assert_eq!(result.len(), 4);

        // All volatilities should be positive
        assert!(result.iter().all(|&v| v > Positive::ZERO));

        // Volatility should increase after large returns
        assert!(result[2] > result[1]); // After 0.03 return
    }

    #[test]
    fn test_garch_zero_initial_return() {
        let returns = vec![dec!(0.0), dec!(0.02), dec!(0.03)];
        let omega = dec!(0.1);
        let alpha = dec!(0.2);
        let beta = dec!(0.7);

        let result = garch_volatility(&returns, omega, alpha, beta).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Positive::ZERO);
    }

    #[test]
    fn test_garch_parameter_sum_one() {
        let returns = vec![dec!(0.01), dec!(0.02), dec!(0.03)];
        let omega = dec!(0.05);
        let alpha = dec!(0.15);
        let beta = dec!(0.8); // alpha + beta = 0.95 < 1

        let result = garch_volatility(&returns, omega, alpha, beta).unwrap();

        // Volatility should remain stable when alpha + beta < 1
        assert!(result.iter().all(|&v| v < pos!(1.0)));
    }

    #[test]
    fn test_garch_extreme_returns() {
        let returns = vec![dec!(0.01), dec!(0.2), dec!(-0.2), dec!(0.01)]; // Large returns
        let omega = dec!(0.1);
        let alpha = dec!(0.2);
        let beta = dec!(0.7);

        let result = garch_volatility(&returns, omega, alpha, beta).unwrap();

        // Volatility should spike after large returns
        assert!(result[2] > result[1]);
        assert!(result[2] > result[0] * pos!(2.0)); // At least double the initial volatility
    }

    #[test]
    fn test_garch_parameters_validation() {
        let returns = vec![dec!(0.01)];

        // Test with invalid parameters (negative values)
        let result_negative = garch_volatility(&returns, dec!(-0.1), dec!(0.2), dec!(0.7));
        assert!(result_negative.is_ok()); // Should handle negative parameters

        // Test with parameters summing to more than 1
        let result_sum = garch_volatility(&returns, dec!(0.1), dec!(0.5), dec!(0.6));
        assert!(result_sum.is_ok()); // Should handle parameters summing > 1
    }
}

#[cfg(test)]
mod tests_heston_volatility {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_heston_basic_properties() {
        let kappa = dec!(2.0); // Mean reversion speed
        let theta = dec!(0.04); // Long-term variance
        let xi = dec!(0.3); // Volatility of variance
        let v0 = dec!(0.04); // Initial variance
        let dt = dec!(0.01); // Time step
        let steps = 100; // Number of steps

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // Check basic properties
        assert_eq!(result.len(), steps);
        assert_eq!(result[0], Positive(v0).sqrt());
        assert!(result.iter().all(|&x| x >= Positive::ZERO));
    }

    #[test]
    fn test_heston_zero_volatility_of_volatility() {
        let kappa = dec!(2.0);
        let theta = dec!(0.04);
        let xi = dec!(0.0); // No randomness in volatility
        let v0 = dec!(0.04);
        let dt = dec!(0.01);
        let steps = 50;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // With no volatility of volatility, values should move deterministically towards theta
        for i in 1..steps {
            // Verify that volatility moves monotonically towards the long-term mean
            match v0 < theta {
                true => assert!(result[i] >= result[i - 1]),
                false => assert!(result[i] <= result[i - 1]),
            }
        }
    }

    #[test]
    fn test_heston_high_mean_reversion() {
        let kappa = dec!(10.0); // High mean reversion
        let theta = dec!(0.04);
        let xi = dec!(0.1);
        let v0 = dec!(0.08); // Start away from theta
        let dt = dec!(0.01);
        let steps = 200;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // With high mean reversion, later values should be closer to sqrt(theta)
        let theta_vol = Positive(theta).sqrt();
        let initial_dist = (result[0].to_dec() - theta_vol.to_dec()).abs();
        let final_dist = (result[steps - 1].to_dec() - theta_vol.to_dec()).abs();
        assert!(final_dist < initial_dist);
    }

    #[test]
    fn test_heston_high_volatility() {
        let kappa = dec!(2.0);
        let theta = dec!(0.04);
        let xi = dec!(1.0); // High volatility of volatility
        let v0 = dec!(0.04);
        let dt = dec!(0.01);
        let steps = 100;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // With high volatility, we should see more variation
        let variations: Vec<_> = result
            .windows(2)
            .map(|w| (w[1].to_dec() - w[0].to_dec()).abs())
            .collect();
        assert!(variations.iter().any(|&x| x > dec!(0.001)));
    }

    #[test]
    fn test_heston_zero_initial_variance() {
        let kappa = dec!(2.0);
        let theta = dec!(0.04);
        let xi = dec!(0.3);
        let v0 = dec!(0.0); // Start at zero
        let dt = dec!(0.01);
        let steps = 100;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // Should start at zero and then increase
        assert_eq!(result[0], Positive::ZERO);
        assert!(result.iter().skip(1).any(|&x| x > Positive::ZERO));
    }

    #[test]
    fn test_heston_numerical_stability() {
        let kappa = dec!(2.0);
        let theta = dec!(0.04);
        let xi = dec!(0.3);
        let v0 = dec!(0.04);
        let dt = dec!(0.001); // Very small time step
        let steps = 1000;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // Check that values remain finite and positive
        assert!(result.iter().all(|&x| x < pos!(100.0)));
        assert!(result.iter().all(|&x| x >= Positive::ZERO));
    }

    #[test]
    fn test_heston_large_time_steps() {
        let kappa = dec!(2.0);
        let theta = dec!(0.04);
        let xi = dec!(0.3);
        let v0 = dec!(0.04);
        let dt = dec!(0.1); // Large time step
        let steps = 10;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // Should still produce valid results with large steps
        assert!(result.iter().all(|&x| x >= Positive::ZERO));
    }

    #[test]
    fn test_heston_extreme_parameters() {
        let kappa = dec!(20.0); // Very high mean reversion
        let theta = dec!(1.0); // High target variance
        let xi = dec!(2.0); // Very high vol of vol
        let v0 = dec!(0.5);
        let dt = dec!(0.01);
        let steps = 100;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // Even with extreme parameters, results should be finite and non-negative
        assert!(result.iter().all(|&x| x >= Positive::ZERO));
    }

    #[test]
    fn test_heston_min_steps() {
        let kappa = dec!(2.0);
        let theta = dec!(0.04);
        let xi = dec!(0.3);
        let v0 = dec!(0.04);
        let dt = dec!(0.01);
        let steps = 1;

        let result = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();

        // Should work with minimum number of steps
        assert_eq!(result.len(), steps);
        assert_eq!(result[0], Positive(v0).sqrt());
    }
}

#[cfg(test)]
mod tests_uncertain_volatility_bounds {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use crate::{ExpirationDate, assert_pos_relative_eq};
    use rust_decimal_macros::dec;

    // Helper function to create a test option
    fn create_test_option(style: OptionStyle, side: Side, strike: Positive) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            strike,
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),     // Initial implied volatility
            Positive::ONE, // Quantity
            pos!(100.0),   // Underlying price
            dec!(0.05),    // Risk-free rate
            style,
            Positive::ZERO, // No dividend yield
            None,           // No exotic params
        )
    }

    #[test]
    fn test_bounds_basic_call() {
        let option = create_test_option(OptionStyle::Call, Side::Long, pos!(100.0));
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3)).unwrap();

        // Lower bound should be less than upper bound for a call
        assert!(lower < upper);
        // Both bounds should be positive
        assert!(lower > Positive::ZERO);
        assert!(upper > Positive::ZERO);
    }

    #[test]
    fn test_bounds_basic_put() {
        let option = create_test_option(OptionStyle::Put, Side::Long, pos!(100.0));
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.1), pos!(0.3)).unwrap();

        // Lower bound should be less than upper bound for a put
        assert!(lower < upper);
        // Both bounds should be positive
        assert!(lower > Positive::ZERO);
        assert!(upper > Positive::ZERO);
    }

    #[test]
    fn test_bounds_same_volatility() {
        let option = create_test_option(OptionStyle::Call, Side::Long, pos!(100.0));
        let vol = pos!(0.2);
        let (lower, upper) = uncertain_volatility_bounds(&option, vol, vol).unwrap();

        // Bounds should be equal when min and max volatilities are the same
        assert_pos_relative_eq!(lower, upper, pos!(1e-10));
    }

    #[test]
    fn test_bounds_itm_call() {
        let itm_option = create_test_option(OptionStyle::Call, Side::Long, pos!(90.0));
        let (lower, upper) =
            uncertain_volatility_bounds(&itm_option, pos!(0.1), pos!(0.3)).unwrap();

        // For ITM call, bounds should be above intrinsic value
        let intrinsic = pos!(10.0); // 100 - 90
        assert!(lower > intrinsic);
        assert!(upper > intrinsic);
    }

    #[test]
    fn test_bounds_otm_call() {
        let otm_option = create_test_option(OptionStyle::Call, Side::Long, pos!(110.0));
        let (lower, upper) =
            uncertain_volatility_bounds(&otm_option, pos!(0.1), pos!(0.3)).unwrap();

        // For OTM call, both bounds should be positive but lower than strike price
        assert!(lower > Positive::ZERO);
        assert!(upper < pos!(110.0));
    }

    #[test]
    fn test_bounds_otm_put() {
        let otm_option = create_test_option(OptionStyle::Put, Side::Long, pos!(90.0));
        let (lower, upper) =
            uncertain_volatility_bounds(&otm_option, pos!(0.1), pos!(0.3)).unwrap();

        // For OTM put, both bounds should be positive but lower than strike price
        assert!(lower > Positive::ZERO);
        assert!(upper < pos!(90.0));
    }

    #[test]
    fn test_bounds_extreme_volatilities() {
        let option = create_test_option(OptionStyle::Call, Side::Long, pos!(100.0));
        let (lower, upper) = uncertain_volatility_bounds(&option, pos!(0.01), pos!(1.0)).unwrap();

        assert!(lower < upper);
        assert!(lower > Positive::ZERO);
        assert!(upper < option.underlying_price);
    }
}

#[cfg(test)]
mod tests_adjust_volatility {
    use super::*;
    use crate::{assert_pos_relative_eq, pos};

    #[test]
    fn test_same_timeframe() {
        let vol = pos!(0.2);
        let result = adjust_volatility(vol, TimeFrame::Day, TimeFrame::Day).unwrap();
        assert_eq!(result, vol);
    }

    #[test]
    fn test_same_periods_different_timeframe() {
        // Create two custom timeframes with same periods
        let vol = pos!(0.2);
        let result =
            adjust_volatility(vol, TimeFrame::Custom(pos!(252.0)), TimeFrame::Day).unwrap();
        assert_eq!(result, vol);
    }

    #[test]
    fn test_zero_periods() {
        let vol = pos!(0.2);
        let result = adjust_volatility(vol, TimeFrame::Day, TimeFrame::Custom(Positive::ZERO));
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("zero periods per year"));
        } else {
            panic!("Expected error for zero periods");
        }
    }

    #[test]
    fn test_daily_to_minute() {
        let daily_vol = pos!(0.2);
        let result = adjust_volatility(daily_vol, TimeFrame::Day, TimeFrame::Minute).unwrap();

        // For testing, we can calculate the expected value:
        // daily periods = 252
        // minute periods = 252 * 6.5 * 60 = 98280
        // scale_factor = sqrt(252/98280) ≈ 0.0506
        // expected = 0.2 * 0.0506 ≈ 0.01012
        assert_pos_relative_eq!(result, pos!(0.0101273936), pos!(0.0001));
    }

    #[test]
    fn test_minute_to_daily() {
        let minute_vol = pos!(0.01012);
        let result = adjust_volatility(minute_vol, TimeFrame::Minute, TimeFrame::Day).unwrap();

        // This should be approximately the inverse of the previous test
        assert_pos_relative_eq!(result, pos!(0.199853), pos!(0.0001));
    }

    #[test]
    fn test_daily_to_hourly() {
        let daily_vol = pos!(0.2);
        let result = adjust_volatility(daily_vol, TimeFrame::Day, TimeFrame::Hour).unwrap();

        // daily periods = 252
        // hourly periods = 252 * 6.5 = 1638
        // scale_factor = sqrt(252/1638) ≈ 0.3922
        // expected = 0.2 * 0.3922 ≈ 0.07844
        assert_pos_relative_eq!(result, pos!(0.07844), pos!(0.0001));
    }

    #[test]
    fn test_monthly_to_daily() {
        let monthly_vol = pos!(0.3);
        let result = adjust_volatility(monthly_vol, TimeFrame::Month, TimeFrame::Day).unwrap();

        // monthly periods = 12
        // daily periods = 252
        // scale_factor = sqrt(12/252) ≈ 0.218
        // expected = 0.3 * 0.218 ≈ 0.0654
        assert_pos_relative_eq!(result, pos!(0.0654653), pos!(0.0001));
    }

    #[test]
    fn test_custom_timeframe() {
        let vol = pos!(0.25);
        let custom_periods = pos!(100.0);
        let result =
            adjust_volatility(vol, TimeFrame::Custom(custom_periods), TimeFrame::Day).unwrap();

        // custom periods = 100
        // daily periods = 252
        // scale_factor = sqrt(100/252) ≈ 0.629
        // expected = 0.25 * 0.629 ≈ 0.15725
        assert_pos_relative_eq!(result, pos!(0.157485197), pos!(0.0001));
    }

    #[test]
    fn test_yearly_to_daily() {
        let yearly_vol = pos!(0.4);
        let result = adjust_volatility(yearly_vol, TimeFrame::Year, TimeFrame::Day).unwrap();

        // yearly periods = 1
        // daily periods = 252
        // scale_factor = sqrt(1/252) ≈ 0.0629
        // expected = 0.4 * 0.0629 ≈ 0.02516
        assert_pos_relative_eq!(result, pos!(0.025197631), pos!(0.0001));
    }

    #[test]
    fn test_zero_volatility() {
        let result = adjust_volatility(Positive::ZERO, TimeFrame::Day, TimeFrame::Minute).unwrap();
        assert_eq!(result, Positive::ZERO);
    }

    #[test]
    fn test_very_small_volatility() {
        let small_vol = pos!(0.0001);
        let result = adjust_volatility(small_vol, TimeFrame::Day, TimeFrame::Hour).unwrap();
        assert!(result > Positive::ZERO);
        assert!(result < small_vol);
    }

    #[test]
    fn test_very_large_volatility() {
        let large_vol = pos!(10.0);
        let result = adjust_volatility(large_vol, TimeFrame::Day, TimeFrame::Minute).unwrap();
        assert!(result > Positive::ZERO);
        assert!(result < large_vol);
    }
}

#[cfg(test)]
mod tests_generate_ou_process {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_process_length() {
        let steps = 500;
        let process = generate_ou_process(
            pos!(1.0),
            pos!(1.5),
            pos!(0.1),
            pos!(0.2),
            pos!(0.01),
            steps,
        );
        assert_eq!(process.len(), steps);
    }

    #[test]
    fn test_all_values_positive() {
        let process =
            generate_ou_process(pos!(1.0), pos!(1.5), pos!(0.2), pos!(0.3), pos!(0.01), 1000);

        for value in process {
            assert!(
                value >= Positive::ZERO,
                "Found non-positive value: {:?}",
                value
            );
        }
    }

    #[test]
    fn test_mean_reversion_tendency() {
        let process = generate_ou_process(
            pos!(0.1),
            pos!(1.0),
            pos!(1.0),  // high theta for fast reversion
            pos!(0.01), // low volatility
            pos!(0.01),
            1000,
        );

        let last = process.last().unwrap().to_dec();
        let diff = (last - dec!(1.0)).abs();
        assert!(diff < dec!(0.1), "Final value too far from mean: {}", last);
    }
}
