/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/8/24
******************************************************************************/
use crate::Options;
use crate::error::PricingError;
use crate::error::decimal::DecimalError;
use crate::greeks::{big_n, d2};
use crate::model::decimal::{d_add, d_mul, d_sub, finite_decimal};
use crate::model::types::Side;
use crate::pricing::binomial_model::BinomialPricingParams;
use crate::pricing::constants::{CLAMP_MAX, CLAMP_MIN};
use crate::pricing::payoff::{Payoff, PayoffInfo};
use crate::utils::random_decimal;
use positive::Positive;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Simulates stock returns based on a normal distribution using pure decimal arithmetic.
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
/// A Result containing either:
/// - Ok(`Vec<Decimal>`): A vector of simulated returns as Decimal numbers
/// - Err(DecimalError): If there's an error in decimal calculations
///
/// # Errors
///
/// Returns [`DecimalError::ConversionError`] when the sampled normal
/// variate cannot be represented as a `Decimal` (e.g. NaN or out-of-range
/// float), and [`DecimalError::ArithmeticError`] when the
/// `mean + std_dev * z` combination overflows the `Decimal` range.
pub fn simulate_returns(
    mean: Decimal,
    std_dev: Positive,
    length: usize,
    time_step: Decimal,
) -> Result<Vec<Decimal>, DecimalError> {
    /// Generates a pair of normally distributed random numbers using Box-Muller transform
    fn generate_normal_pair<R: Rng>(rng: &mut R) -> Result<(Decimal, Decimal), DecimalError> {
        // Generate two uniform random numbers between 0 and 1
        let u1 = random_decimal(rng)?;
        let u2 = random_decimal(rng)?;

        // Convert to normal distribution using Box-Muller transform
        let r = (-Decimal::TWO * u1.ln()).sqrt().ok_or_else(|| {
            DecimalError::arithmetic_error("sqrt", "non-finite operand in Box-Muller r")
        })?;
        let theta = Decimal::TWO * Decimal::PI * u2;

        let x1 = r * theta.cos();
        let x2 = r * theta.sin();

        Ok((x1, x2))
    }

    if std_dev < Decimal::ZERO {
        return Err(DecimalError::InvalidValue {
            value: std_dev.to_f64(),
            reason: "Standard deviation cannot be negative".to_string(),
        });
    }

    // Adjust mean and standard deviation for the time step
    let adjusted_mean = mean * time_step;
    let adjusted_std = std_dev
        * time_step.sqrt().ok_or_else(|| {
            DecimalError::arithmetic_error("sqrt", "invalid (negative or non-finite) time_step")
        })?;

    // Special case: if std_dev is 0, return a vector of constant values
    if adjusted_std == Decimal::ZERO {
        return Ok(vec![adjusted_mean; length]);
    }

    let mut returns = Vec::with_capacity(length);
    let mut rng = rand::rng();

    // Generate pairs of normally distributed random numbers using Box-Muller transform
    for _ in 0..length.div_ceil(2) {
        let (n1, n2) = generate_normal_pair(&mut rng)?;

        // Scale the random numbers by mean and std_dev
        let r1 = n1 * adjusted_std + adjusted_mean;
        returns.push(r1);

        if returns.len() < length {
            let r2 = n2 * adjusted_std + adjusted_mean;
            returns.push(r2);
        }
    }

    Ok(returns)
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
pub(crate) fn calculate_up_factor(
    volatility: Positive,
    dt: Decimal,
) -> Result<Decimal, DecimalError> {
    let sqrt_dt = dt
        .sqrt()
        .ok_or_else(|| DecimalError::arithmetic_error("sqrt", "non-finite dt in up factor"))?;
    Ok((sqrt_dt * volatility).exp())
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
pub(crate) fn calculate_down_factor(
    volatility: Positive,
    dt: Decimal,
) -> Result<Decimal, DecimalError> {
    let sqrt_dt = dt
        .sqrt()
        .ok_or_else(|| DecimalError::arithmetic_error("sqrt", "non-finite dt in down factor"))?;
    Ok((dec!(-1.0) * sqrt_dt * volatility.to_dec()).exp())
}

/// Calculates the probability using a given interest rate, time interval,
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
    int_rate: Decimal,
    dt: Decimal,
    down_factor: Decimal,
    up_factor: Decimal,
) -> Result<Decimal, DecimalError> {
    Ok(
        (((int_rate * dt).exp() - down_factor) / (up_factor - down_factor))
            .clamp(CLAMP_MIN, CLAMP_MAX),
    )
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
pub(crate) fn calculate_discount_factor(
    int_rate: Decimal,
    dt: Decimal,
) -> Result<Decimal, DecimalError> {
    Ok((-int_rate * dt).exp())
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
    probability: Decimal,
    next: &mut [Vec<Decimal>],
    node: usize,
    discount_factor: Decimal,
) -> Result<Decimal, DecimalError> {
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
    probability: Decimal,
    price_up: Decimal,
    price_down: Decimal,
    discount_factor: Decimal,
) -> Result<Decimal, DecimalError> {
    let up_branch = d_mul(probability, price_up, "pricing::binomial::node::up_branch")?;
    let down_branch = d_mul(
        d_sub(
            Decimal::ONE,
            probability,
            "pricing::binomial::node::down_weight",
        )?,
        price_down,
        "pricing::binomial::node::down_branch",
    )?;
    let expected = d_add(up_branch, down_branch, "pricing::binomial::node::expected")?;
    d_mul(
        expected,
        discount_factor,
        "pricing::binomial::node::discounted",
    )
}

/// Calculates the option price using the Binomial Pricing Model.
///
/// # Parameters
///
/// * `params`: An instance of `BinomialPricingParams` containing the necessary parameters
///   such as the asset price, strike price, option type, and number of steps.
/// * `u`: A `Decimal` representing the up factor in the binomial tree.
/// * `d`: A `Decimal` representing the down factor in the binomial tree.
/// * `i`: A `usize` representing the current step in the binomial tree.
///
/// # Returns
///
/// `Result<Decimal, PricingError>` — the option price at the given step,
/// or `PricingError::NonFinite` if the payoff `f64` is `NaN` / `±∞`.
///
/// # Errors
///
/// Returns [`PricingError::NonFinite`] when `params.option_type.payoff(...)`
/// produces a non-finite `f64` (NaN / ±Inf), tagged with
/// `"pricing::binomial::option_price::payoff"`.
///
pub(crate) fn calculate_option_price(
    params: BinomialPricingParams,
    u: Decimal,
    d: Decimal,
    i: usize,
) -> Result<Decimal, PricingError> {
    let info = PayoffInfo {
        spot: params.asset * u.powu(i as u64) * d.powi((params.no_steps - i) as i64),
        strike: params.strike,
        style: *params.option_style,
        side: *params.side,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };
    let payoff_f64 = params.option_type.payoff(&info);
    let payoff = finite_decimal(payoff_f64).ok_or_else(|| {
        PricingError::non_finite("pricing::binomial::option_price::payoff", payoff_f64)
    })?;

    Ok(payoff)
}

/// Calculates the discounted payoff for an option based on the binomial pricing model.
///
/// # Parameters
///
/// * `params`: A structure containing parameters needed for the binomial pricing calculation.
///
/// # Returns
///
/// `Result<Decimal, PricingError>` — the discounted payoff (sign-adjusted for
/// `Side::Long` / `Side::Short`), or `PricingError::NonFinite` if the
/// payoff `f64` is non-finite.
///
/// The function takes into account the future asset price, the interest rate, the expiry time,
/// the type of option (call or put), and the style of the option (European or American).
///
/// It adjusts the future asset price with the provided interest rate and expiry time,
/// calculates the payoff, discounts it by the interest rate, and then adjusts for the side
/// of the trade (long or short).
///
/// # Errors
///
/// - [`PricingError::NonFinite`] when `params.option_type.payoff(...)` produces a
///   non-finite `f64`, tagged `"pricing::binomial::discounted_payoff::payoff"`.
/// - [`PricingError::Decimal`] (via `#[from]`) when the checked multiplications
///   `-rate * expiry` or `discount * payoff` overflow.
///
pub(crate) fn calculate_discounted_payoff(
    params: BinomialPricingParams,
) -> Result<Decimal, PricingError> {
    let info = PayoffInfo {
        spot: params.asset * (params.int_rate * params.expiry).exp(),
        strike: params.strike,
        style: *params.option_style,
        side: *params.side,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    let payoff_f64 = params.option_type.payoff(&info);
    let payoff = finite_decimal(payoff_f64).ok_or_else(|| {
        PricingError::non_finite("pricing::binomial::discounted_payoff::payoff", payoff_f64)
    })?;
    // Build the discount exponent through a checked multiplication so
    // that an overflow on `-rate * expiry` is tagged rather than
    // saturating silently before `.exp()` compresses it back into a
    // bounded range.
    let discount_exponent = d_mul(
        -params.int_rate,
        params.expiry.to_dec(),
        "pricing::binomial::discounted_payoff::discount_exponent",
    )?;
    let discount = discount_exponent.exp();
    let discounted_payoff = d_mul(
        discount,
        payoff,
        "pricing::binomial::discounted_payoff::discounted",
    )?;
    match params.side {
        Side::Long => Ok(discounted_payoff),
        Side::Short => Ok(-discounted_payoff),
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
/// `Result<Decimal, PricingError>` — the Wiener process increment for the
/// given time step.
///
/// # Errors
///
/// - [`PricingError::Decimal`] (via `#[from]`) if `Normal::new(0.0, 1.0)` fails
///   (effectively never; parameters are constants) or if `dt.sqrt()` is
///   undefined for the given input (negative or non-finite `dt`).
/// - [`PricingError::NonFinite`] if the sampled normal value is non-finite,
///   tagged `"pricing::monte_carlo::wiener_increment::sample"`.
///
pub(crate) fn wiener_increment(dt: Decimal) -> Result<Decimal, PricingError> {
    let normal = Normal::new(0.0, 1.0)
        .map_err(|e| DecimalError::arithmetic_error("Normal::new(0.0, 1.0)", &e.to_string()))?;
    let mut rng = rand::rng();

    let sample_f64 = normal.sample(&mut rng);
    let sample = finite_decimal(sample_f64).ok_or_else(|| {
        PricingError::non_finite("pricing::monte_carlo::wiener_increment::sample", sample_f64)
    })?;

    let sqrt_dt = dt.sqrt().ok_or_else(|| {
        DecimalError::arithmetic_error("sqrt", "non-finite dt in wiener_increment")
    })?;
    Ok(sample * sqrt_dt)
}

/// Calculates the probability that the option will remain under the strike price.
///
/// # Parameters
/// - `option`: An `Options` struct that contains various attributes necessary for the calculation,
///   such as underlying price, strike price, risk-free rate, expiration date, and implied volatility.
/// - `strike`: An optional `Positive` strike price. If `None`, the function uses
///   the `strike_price` from the `Options` struct.
///
/// # Returns
/// `Result<Decimal, DecimalError>` — the probability `N(-d2)` that the
/// underlying remains under the strike at expiration.
///
/// # Errors
///
/// - `DecimalError::ArithmeticError` if `d2(...)` cannot be evaluated for the
///   given option (e.g., zero volatility / non-positive time to expiration).
/// - Whatever `expiration_date.get_years()?` propagates through
///   `From<ExpirationDateError>`.
pub fn probability_keep_under_strike(
    option: Options,
    strike: Option<Positive>,
) -> Result<Decimal, DecimalError> {
    let strike_price = match strike {
        Some(strike) => strike,
        None => option.strike_price,
    };
    let years = option.expiration_date.get_years()?;
    let d2_val = d2(
        option.underlying_price,
        strike_price,
        option.risk_free_rate,
        years,
        option.implied_volatility,
    )
    .map_err(|e| DecimalError::arithmetic_error("d2", &e.to_string()))?;
    big_n(-d2_val)
}

#[cfg(test)]
mod tests_simulate_returns {
    use super::*;
    use num_traits::FromPrimitive;
    use positive::pos_or_panic;

    use crate::assert_decimal_eq;
    use crate::model::decimal::DecimalStats;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulate_returns() {
        let mean = dec!(0.05); // 5% annual return
        let std_dev = pos_or_panic!(0.2); // 20% annual volatility
        let length = 252; // One year of daily returns
        let time_step = Decimal::from_f64(1.0 / 252.0).unwrap(); // Daily time step

        let returns = simulate_returns(mean, std_dev, length, time_step).unwrap();

        assert_eq!(returns.len(), length);

        // Check that the mean and standard deviation are reasonably close to expected values
        let simulated_mean = returns.clone().mean();
        let simulated_std_dev = returns.std_dev();

        assert_decimal_eq!(simulated_mean, mean * time_step, dec!(0.01));
        assert_decimal_eq!(
            simulated_std_dev,
            std_dev * time_step.sqrt().unwrap(),
            dec!(0.01)
        );
    }
}

#[cfg(test)]
mod tests_simulate_returns_bis {
    use super::*;
    use positive::pos_or_panic;

    use crate::assert_decimal_eq;
    use crate::model::decimal::DecimalStats;
    use num_traits::FromPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulate_returns_length() {
        let length = 1000;
        let returns = simulate_returns(
            dec!(0.05),
            pos_or_panic!(0.2),
            length,
            Decimal::from_f64(1.0 / 252.0).unwrap(),
        )
        .unwrap();
        assert_eq!(returns.len(), length);
    }

    #[test]
    fn test_simulate_returns_zero_mean() {
        let returns = simulate_returns(
            dec!(0.0),
            pos_or_panic!(0.2),
            1000,
            Decimal::from_f64(1.0 / 252.0).unwrap(),
        )
        .unwrap();
        let mean = returns.mean();
        assert!(mean.abs() < dec!(0.01));
    }

    #[test]
    fn test_simulate_returns_zero_volatility() {
        let mean = dec!(0.05);
        let time_step = Decimal::from_f64(1.0 / 252.0).unwrap();
        let returns = simulate_returns(mean, Positive::ZERO, 100, time_step).unwrap();

        let expected = mean * time_step;
        for r in returns {
            assert_decimal_eq!(r, expected, dec!(1e-10));
        }
    }

    #[test]
    fn test_simulate_returns_single_value() {
        let returns = simulate_returns(
            dec!(0.05),
            pos_or_panic!(0.2),
            1,
            Decimal::from_f64(1.0 / 252.0).unwrap(),
        )
        .unwrap();
        assert_eq!(returns.len(), 1);
    }

    #[test]
    fn test_simulate_returns_yearly_step() {
        let returns = simulate_returns(dec!(0.05), pos_or_panic!(0.2), 100, dec!(1.0)).unwrap();
        assert_eq!(returns.len(), 100);
        for r in returns {
            assert!(r > dec!(-1.0));
        }
    }

    #[test]
    #[should_panic]
    fn test_simulate_returns_invalid_std_dev() {
        assert!(
            simulate_returns(
                dec!(0.05),
                pos_or_panic!(-0.2),
                100,
                Decimal::from_f64(1.0 / 252.0).unwrap(),
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod tests_utils {
    use super::*;
    use positive::pos_or_panic;

    use crate::assert_decimal_eq;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_calculate_up_factor() {
        let volatility = pos_or_panic!(0.09531018);
        let dt = dec!(1.0);
        let up_factor = calculate_up_factor(volatility, dt).unwrap();
        let expected_up_factor = (volatility * dt.sqrt().unwrap()).exp();
        assert!(
            (up_factor - expected_up_factor).abs() < EPSILON,
            "Expected {expected_up_factor}, got {up_factor}"
        );
    }

    #[test]
    fn test_calculate_up_factor_2() {
        let volatility = pos_or_panic!(0.17);
        let dt = dec!(1.0);
        let up_factor = calculate_up_factor(volatility, dt).unwrap();
        let expected_up_factor = dec!(1.1853048504885680);
        assert_decimal_eq!(up_factor, expected_up_factor, EPSILON);
    }

    #[test]
    fn test_calculate_down_factor() {
        let volatility = pos_or_panic!(0.09531018);
        let dt = dec!(1.0);
        let down_factor = calculate_down_factor(volatility, dt).unwrap();
        let expected_down_factor = (-dt.sqrt().unwrap() * volatility).exp();
        assert!(
            (down_factor - expected_down_factor).abs() < EPSILON,
            "Expected {expected_down_factor}, got {down_factor}"
        );
    }

    #[test]
    fn test_calculate_down_factor_2() {
        let volatility = pos_or_panic!(0.17);
        let dt = dec!(1.0);
        let up_factor = calculate_down_factor(volatility, dt).unwrap();
        let expected_up_factor = dec!(0.843664817188432427);
        assert_decimal_eq!(up_factor, expected_up_factor, EPSILON);
    }

    #[test]
    fn test_calculate_probability() {
        let int_rate = dec!(0.05);
        let dt = Decimal::ONE;
        let down_factor = dec!(0.909090909);
        let up_factor = dec!(1.1);
        let probability = calculate_probability(int_rate, dt, down_factor, up_factor).unwrap();
        let expected_probability = (((int_rate * dt).exp() - down_factor)
            / (up_factor - down_factor))
            .clamp(CLAMP_MIN, CLAMP_MAX);
        assert!(
            (probability - expected_probability).abs() < EPSILON,
            "Expected {expected_probability}, got {probability}"
        );
    }

    #[test]
    fn test_calculate_probability_ii() {
        let int_rate = dec!(0.05);
        let dt = Decimal::ONE;
        let down_factor = dec!(0.8);
        let up_factor = dec!(1.2);
        let probability = calculate_probability(int_rate, dt, down_factor, up_factor).unwrap();
        assert_decimal_eq!(probability, dec!(0.62817774088541), EPSILON);
    }

    #[test]
    fn test_calculate_discount_factor() {
        let int_rate = dec!(0.05);
        let dt = Decimal::ONE;
        let discount_factor = calculate_discount_factor(int_rate, dt).unwrap();
        let expected_discount_factor = (-int_rate * dt).exp();
        assert!(
            (discount_factor - expected_discount_factor).abs() < EPSILON,
            "Expected {expected_discount_factor}, got {discount_factor}"
        );
    }
}

#[cfg(test)]
mod tests_probability_keep_under_strike {
    use super::*;
    use positive::{Positive, pos_or_panic, spos};

    use crate::model::types::{OptionStyle, OptionType};
    use crate::{ExpirationDate, assert_decimal_eq};
    use positive::constants::DAYS_IN_A_YEAR;
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    fn test_probability_keep_under_strike_with_given_strike() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            risk_free_rate: Decimal::ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(0.001),
            underlying_symbol: "".to_string(),
            quantity: Positive::ONE,
            exotic_params: None,
        };
        let strike = spos!(100.0);
        let probability = probability_keep_under_strike(option, strike).unwrap();
        info!("{:?} {}", strike, probability);
        assert_decimal_eq!(probability, dec!(0.5), dec!(0.001));
    }

    #[test]
    fn test_probability_keep_under_strike_with_default_strike() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(110.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike).unwrap();
        assert!(
            probability > Decimal::ZERO && probability < Decimal::ONE,
            "Probability should be between 0 and 1"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_zero_volatility() {
        // Zero implied volatility makes d2 ill-defined (division by zero in the
        // analytical form). Post panic-free refactor this surfaces as a typed Err
        // instead of a panic.
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: Positive::ZERO,
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            exotic_params: None,
        };
        let strike = None;
        assert!(
            probability_keep_under_strike(option, strike).is_err(),
            "zero volatility should produce a DecimalError, not a panic"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_high_volatility() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(110.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(5.0), // Alta volatilidad
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike).unwrap();
        assert!(
            probability > Decimal::ZERO && probability < Decimal::ONE,
            "Probability should still be valid even with high volatility"
        );
    }

    #[test]
    fn test_probability_keep_under_strike_expired_option() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: pos_or_panic!(110.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            expiration_date: ExpirationDate::Days(Positive::ONE),
            implied_volatility: pos_or_panic!(0.2),
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            exotic_params: None,
        };
        let strike = None;
        let probability = probability_keep_under_strike(option, strike).unwrap();
        assert_eq!(
            probability,
            Decimal::ONE,
            "Expired option should have zero probability of being ITM"
        );
    }
}

#[cfg(test)]
mod tests_calculate_up_down_factor {
    use super::*;
    use positive::pos_or_panic;

    use crate::assert_decimal_eq;
    use crate::model::decimal::ONE_DAY;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_factors_standard_case() {
        let volatility = pos_or_panic!(0.2); // 20% volatility
        let dt = ONE_DAY; // One trading day

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify that up and down factors are reciprocals
        assert_decimal_eq!(up * down, dec!(1.0), EPSILON);
        // Verify values are in expected range
        assert!(up > Decimal::ONE);
        assert!(down < Decimal::ONE);
    }

    #[test]
    fn test_factors_zero_volatility() {
        let volatility = Positive::ZERO;
        let dt = ONE_DAY;

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // With zero volatility, both factors should be 1.0
        assert_decimal_eq!(up, Decimal::ONE, dec!(1e-10));
        assert_decimal_eq!(down, Decimal::ONE, dec!(1e-10));
    }

    #[test]
    fn test_factors_zero_dt() {
        let volatility = pos_or_panic!(0.2);
        let dt = Decimal::ZERO;

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // With zero dt, both factors should be 1.0
        assert_decimal_eq!(up, Decimal::ONE, EPSILON);
        assert_decimal_eq!(down, Decimal::ONE, EPSILON);
    }

    #[test]
    fn test_factors_high_volatility() {
        let volatility = Positive::ONE; // 100% volatility
        let dt = Decimal::ONE; // One year

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify expected behavior for extreme values
        assert!(up > dec!(1.0));
        assert!(down < dec!(1.0));
        assert_decimal_eq!(up * down, Decimal::ONE, dec!(1e-10));
    }

    #[test]
    fn test_factors_small_dt() {
        let volatility = pos_or_panic!(0.2);
        let dt = ONE_DAY / dec!(24.0); // One hour (assuming 24-hour trading day)

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify behavior with very small time steps
        assert!(up > Decimal::ONE);
        assert!(down < Decimal::ONE);
        assert_decimal_eq!(up * down, Decimal::ONE, dec!(1e-10));
    }

    #[test]
    fn test_factors_different_time_periods() {
        let volatility = pos_or_panic!(0.2);
        let daily_dt = ONE_DAY;
        let weekly_dt = dec!(5.0) / dec!(252.0);
        let monthly_dt = dec!(21.0) / dec!(252.0);

        let daily_up = calculate_up_factor(volatility, daily_dt).unwrap();
        let weekly_up = calculate_up_factor(volatility, weekly_dt).unwrap();
        let monthly_up = calculate_up_factor(volatility, monthly_dt).unwrap();

        // Longer periods should have larger factors
        assert!(daily_up < weekly_up);
        assert!(weekly_up < monthly_up);
    }

    #[test]
    fn test_factors_extreme_volatility() {
        let volatility = pos_or_panic!(5.0); // 500% volatility
        let dt = Decimal::ONE; // One year

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Verify behavior with extreme volatility
        assert!(up > Decimal::ONE);
        assert!(down < Decimal::ONE);
        assert_decimal_eq!(up * down, Decimal::ONE, EPSILON);
    }

    #[test]
    fn test_factors_symmetry() {
        let volatility = pos_or_panic!(0.3);
        let dt = dec!(1.0) / dec!(12.0); // One month

        let up = calculate_up_factor(volatility, dt).unwrap();
        let down = calculate_down_factor(volatility, dt).unwrap();

        // Up move should be multiplicative inverse of down move
        assert_decimal_eq!(up, Decimal::ONE / down, dec!(1e-10));
    }

    #[test]
    fn test_factors_consistency() {
        let volatility = pos_or_panic!(0.2);
        let dt1 = ONE_DAY;
        let dt2 = dt1 / dec!(2.0);

        let up1 = calculate_up_factor(volatility, dt1).unwrap();
        let up2 = calculate_up_factor(volatility, dt2).unwrap();

        // Factor for larger dt should be greater
        assert!(up1 > up2);
    }
}
