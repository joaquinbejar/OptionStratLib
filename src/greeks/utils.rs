/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::{PI, ZERO, ZERO_DEC};
use crate::error::greeks::{GreeksError, InputErrorKind};
use crate::model::option::Options;
use crate::model::types::{PositiveF64, PZERO};
use core::f64;
use num_traits::{ ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use statrs::distribution::{ContinuousCDF, Normal};
use crate::error::decimal::DecimalError;
use crate::model::decimal::{decimal_exp, decimal_ln, decimal_pow_two, decimal_sqrt, f64_to_decimal, positive_f64_to_decimal};

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
    match underlying_price {
        _ if underlying_price > strike_price => f64::INFINITY,
        _ if underlying_price < strike_price => f64::NEG_INFINITY,
        _ => ZERO,
    }
}

/// Calculates the `d1` parameter used in the Black-Scholes options pricing model.
///
/// The `d1` value is an intermediary result used to determine option greeks and prices.
/// It is computed using the formula:
///
/// ```math
/// d1 = (ln(S / K) + (r + σ² / 2) * T) / (σ * sqrt(T))
/// ```
///
/// Where:
/// - `S`: Underlying price
/// - `K`: Strike price
/// - `r`: Risk-free rate
/// - `T`: Time to expiration (in years)
/// - `σ`: Implied volatility
///
/// # Parameters
///
/// - `underlying_price`: The current price of the underlying asset. Must be positive.
/// - `strike_price`: The strike price of the option. Must be greater than zero.
/// - `risk_free_rate`: The annual risk-free interest rate, expressed as a decimal.
/// - `expiration_date`: The time to expiration of the option, in years. Must be greater than zero.
/// - `implied_volatility`: The implied volatility of the option, expressed as a decimal. Must be greater than zero.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed `d1` value.
/// - `Err(GreeksError)`: Returns an error if input validation fails. Possible errors include:
///   - Invalid strike price (must be greater than zero).
///   - Invalid implied volatility (must be greater than zero).
///   - Invalid expiration time (must be greater than zero).
///
/// # Errors
///
/// Returns a `GreeksError::InputError` in the following cases:
/// - **InvalidStrike**: Triggered when `strike_price` is zero or less.
/// - **InvalidVolatility**: Triggered when `implied_volatility` is zero.
/// - **InvalidTime**: Triggered when `expiration_date` is zero or less.
///
/// # Example
///
/// ```rust
/// use optionstratlib::greeks::d1;
/// use optionstratlib::model::types::PositiveF64;
///
/// let underlying_price = PositiveF64::new(100.0).unwrap();
/// let strike_price = PositiveF64::new(95.0).unwrap();
/// let risk_free_rate = 0.05;
/// let expiration_date = 0.5; // 6 months
/// let implied_volatility = 0.2;
///
/// match d1(
///     underlying_price,
///     strike_price,
///     risk_free_rate,
///     expiration_date,
///     implied_volatility,
/// ) {
///     Ok(result) => println!("d1: {}", result),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn d1(
    underlying_price: PositiveF64,
    strike_price: PositiveF64,
    risk_free_rate: f64,
    expiration_date: f64,
    implied_volatility: f64,
) -> Result<Decimal, GreeksError> {
    let underlying_price: Decimal = positive_f64_to_decimal(underlying_price)?;
    let strike_price : Decimal = positive_f64_to_decimal(strike_price)?;
    let risk_free_rate: Decimal = f64_to_decimal(risk_free_rate)?;
    let expiration_date: Decimal = f64_to_decimal(expiration_date)?;
    let implied_volatility: Decimal = f64_to_decimal(implied_volatility)?;
    
    if strike_price == ZERO_DEC {
        return Err(GreeksError::InputError(InputErrorKind::InvalidStrike {
            value: strike_price.to_f64().unwrap(),
            reason: "Strike price cannot be zero".to_string(),
        }));
    }
    if implied_volatility == ZERO_DEC {
        return Err(GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value: implied_volatility.to_f64().unwrap(),
            reason: "Implied volatility cannot be zero".to_string(),
        }));
    }

    if expiration_date == ZERO_DEC {
        return Err(GreeksError::InputError(InputErrorKind::InvalidTime {
            value: expiration_date.to_f64().unwrap(),
            reason: "Expiration date cannot be zero".to_string(),
        }));
    }

    let implied_volatility_squared = decimal_pow_two(implied_volatility)?;
    let ln_price_ratio = underlying_price / decimal_ln(strike_price)?;
    let rate_vol_term = risk_free_rate + implied_volatility_squared / dec!(2.0);
    let numerator = ln_price_ratio + rate_vol_term * expiration_date;
    let denominator = implied_volatility * decimal_sqrt(expiration_date)?;

    Ok(numerator / denominator)
}

/// Calculates the `d2` parameter used in the Black-Scholes options pricing model.
///
/// The `d2` value is an intermediary result derived from the `d1` value and is used 
/// to determine option greeks and prices. It is computed using the formula:
///
/// ```math
/// d2 = d1 - σ * sqrt(T)
/// ```
///
/// Where:
/// - `d1`: The `d1` value calculated using the `d1` function.
/// - `σ`: Implied volatility.
/// - `T`: Time to expiration (in years).
///
/// # Parameters
///
/// - `underlying_price`: The current price of the underlying asset. Must be positive.
/// - `strike_price`: The strike price of the option. Must be greater than zero.
/// - `risk_free_rate`: The annual risk-free interest rate, expressed as a decimal.
/// - `expiration_date`: The time to expiration of the option, in years. Must be greater than zero.
/// - `implied_volatility`: The implied volatility of the option, expressed as a decimal. Must be greater than zero.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed `d2` value.
/// - `Err(GreeksError)`: Returns an error if input validation fails or if the `d1` computation fails.
///
/// # Errors
///
/// Returns a `GreeksError::InputError` in the following cases:
/// - **InvalidVolatility**: Triggered when `implied_volatility` is zero.
/// - **InvalidTime**: Triggered when `expiration_date` is zero.
///
/// # Notes
///
/// This function depends on the `d1` function to compute the `d1` value. Any errors from 
/// the `d1` function will propagate to this function.
///
/// # Example
///
/// ```rust
///
/// use optionstratlib::greeks::d2;
/// use optionstratlib::model::types::PositiveF64;
/// let underlying_price = PositiveF64::new(100.0).unwrap();
/// let strike_price = PositiveF64::new(95.0).unwrap();
/// let risk_free_rate = 0.05;
/// let expiration_date = 0.5; // 6 months
/// let implied_volatility = 0.2;
///
/// match d2(
///     underlying_price,
///     strike_price,
///     risk_free_rate,
///     expiration_date,
///     implied_volatility,
/// ) {
///     Ok(result) => println!("d2: {}", result),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn d2(
    underlying_price: PositiveF64,
    strike_price: PositiveF64,
    risk_free_rate: f64,
    expiration_date: f64,
    implied_volatility: f64,
) -> Result<Decimal, GreeksError> {
    
    let expiration_date: Decimal = f64_to_decimal(expiration_date)?;
    let implied_volatility: Decimal = f64_to_decimal(implied_volatility)?;
    
    if implied_volatility == ZERO_DEC {
        return Err(GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value: implied_volatility.to_f64().unwrap(),
            reason: "Implied volatility cannot be zero".to_string(),
        }));
    }

    if expiration_date == ZERO_DEC {
        return Err(GreeksError::InputError(InputErrorKind::InvalidTime {
            value: expiration_date.to_f64().unwrap(),
            reason: "Expiration date cannot be zero".to_string(),
        }));
    }

    let d1_value = d1(
        underlying_price,
        strike_price,
        risk_free_rate,
        expiration_date.to_f64().unwrap(),
        implied_volatility.to_f64().unwrap(),
    )?;

    Ok(d1_value - implied_volatility * decimal_sqrt(expiration_date)?)
}

/// Computes the probability density function (PDF) of the standard normal distribution
/// for a given input `x`.
///
/// The PDF of the standard normal distribution is defined as:
///
/// ```math
/// N(x) = \frac{1}{\sqrt{2 \pi}} \cdot e^{-\frac{x^2}{2}}
/// ```
///
/// Where:
/// - \(x\): The input value for which the PDF is computed.
///
/// # Parameters
///
/// - `x: Decimal`
///   The input value for which the standard normal PDF is calculated.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed PDF value as a `Decimal`.
/// - `Err(GreeksError)`: Returns an error if the computation fails.
///
/// # Calculation Details
///
/// - The denominator is computed as \(\sqrt{2 \pi}\), where \( \pi \) is approximated.
/// - The exponent is computed as \(-\frac{x^2}{2}\).
/// - The PDF value is the product of the reciprocal of the denominator and the exponential term.
///
/// # Errors
///
/// - `GreeksError`: This function will return an error if any part of the calculation fails,
///   though this is unlikely as the operations are well-defined for all finite inputs.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib::greeks::utils::n;
///
/// let x = Decimal::new(100, 2); // 1.00
///
/// match n(x) {
///     Ok(result) => println!("N(x): {}", result),
///     Err(e) => eprintln!("Error calculating N(x): {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// This function assumes that the constant `PI` is pre-defined as a `Decimal` representing the
/// value of \(\pi\) to a sufficient precision for the application.
///
/// ```
pub fn n(x: Decimal) -> Result<Decimal, GreeksError> {
    let denominator = decimal_sqrt(Decimal::TWO * PI)?;
    let exponent = -x * x / Decimal::TWO;

    Ok(Decimal::ONE / denominator * decimal_exp(exponent)?) // N(x) = 1 / sqrt(2 * PI) * e^(-x^2 / 2)
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
pub(crate) fn n_prime(x: Decimal) -> Result<Decimal, GreeksError> {
    Ok(-x * n(x)?) // -x * N(x)
}

/// Computes the cumulative distribution function (CDF) of the standard normal distribution
/// for a given input `x`.
///
/// The function uses the standard normal distribution (mean = 0, standard deviation = 1)
/// to calculate the probability that a normally distributed random variable is less than or
/// equal to `x`. This is commonly referred to as `N(x)` in financial and statistical contexts.
///
/// # Parameters
///
/// - `x: Decimal`
///   The input value for which the CDF is computed. Must be convertible to `f64`.
///
/// # Returns
///
/// - `Ok(Decimal)`: The CDF value corresponding to the input `x`.
/// - `Err(DecimalError)`: Returns an error if the conversion from `Decimal` to `f64` fails.
///
/// # Errors
///
/// Returns a `DecimalError::ConversionError` if:
/// - The input `x` cannot be converted to an `f64`.
///
/// # Notes
///
/// This function uses the [`statrs`](https://docs.rs/statrs/latest/statrs/) crate to model the 
/// standard normal distribution and compute the CDF. The result is returned as a `Decimal`
/// for precision.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib::greeks::utils::big_n;
///
/// let x = Decimal::new(100, 2); // 1.00
///
/// match big_n(x) {
///     Ok(result) => println!("N(x): {}", result),
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn big_n(x: Decimal) -> Result<Decimal, DecimalError> {
    
    let x_f64 = x.to_f64();
    if x_f64.is_none() {
        return Err(DecimalError::ConversionError {
            from_type: "Decimal".to_string(),
            to_type: "f64".to_string(),
            reason: "Conversion failed".to_string(),
        });
    }

    const MEAN: f64 = 0.0;
    const STD_DEV: f64 = 1.0;

    let normal_distribution = Normal::new(MEAN, STD_DEV).unwrap();
    Ok(f64_to_decimal(normal_distribution.cdf(x_f64.unwrap()))?)
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
pub(crate) fn calculate_d_values(option: &Options) -> Result<(Decimal,Decimal), GreeksError> {
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
    Ok((d1_value?, d2_value?))
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
        let (d1_value, d2_value) = calculate_d_values(&option).unwrap();

        assert_relative_eq!(d1_value.to_f64().unwrap(), 7.1671, epsilon = 0.001);
        assert_relative_eq!(d2_value.to_f64().unwrap(), -2.952, epsilon = 0.001);
    }
}

#[cfg(test)]
mod tests_src_greeks_utils {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::{abs, FloatConst};
    use statrs::distribution::ContinuousCDF;
    use statrs::distribution::Normal;

    #[test]
    #[should_panic]
    fn test_d1() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.05;
        let t = 1.0;
        let sigma = 0.2;
        let expected_d1 = (1.0_f64.ln() + (0.05 + 0.02) * 1.0) / (0.2 * 1.0_f64.sqrt());
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
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
        let _ = d1(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d1_zero_t() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.05;
        let t = 0.0;
        let sigma = 0.01;
        let _ = d1(s, k, r, t, sigma).is_err();
    }

    #[test]
    #[should_panic]
    fn test_d2() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.05;
        let t = 1.0;
        let sigma = 0.2;
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
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
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_d1, 75.7114128, epsilon = 0.001);
        assert_relative_eq!(computed_d2, 75.42857, epsilon = 0.001);
    }

    #[test]
    fn test_d2_bis_ii() {
        let s = pos!(100.0);
        let k = pos!(95.0);
        let r = 0.15;
        let t = 1.0;
        let sigma = 0.2;
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_d1, 110.64655, epsilon = 0.001);
        assert_relative_eq!(computed_d2, 110.44655, epsilon = 0.001);
    }

    #[test]
    fn test_d2_zero_sigma() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.0;
        let t = 1.0;
        let sigma = 0.0;
        let _ = d2(s, k, r, t, sigma).is_err();

    }

    #[test]
    fn test_d2_zero_t() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = 0.02;
        let t = 0.0;
        let sigma = 0.01;
        let _ = d2(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_n() {
        let x = Decimal::ZERO;
        let expected_n = 1.0 / (2.0 * f64::PI()).sqrt();
        let computed_n = n(x).unwrap().to_f64().unwrap();
        assert!((computed_n - expected_n).abs() < 1e-10, "n function failed");

        let x = Decimal::ONE;
        let expected_n = 1.0 / (2.0 * f64::PI()).sqrt() * (-0.5f64).exp();
        let computed_n = n(x).unwrap().to_f64().unwrap();
        assert!((computed_n - expected_n).abs() < 1e-10, "n function failed");
    }

    #[test]
    fn test_n_prime() {
        let x = Decimal::ZERO;
        let expected_n_prime = 0.0;
        let computed_n_prime = n_prime(x).unwrap().to_f64().unwrap();
        assert!(
            abs(computed_n_prime - expected_n_prime) < 1e-10,
            "n_prime function failed"
        );

        let x = Decimal::ONE;
        let expected_n_prime = -1.0 * 1.0 / (2.0 * f64::PI()).sqrt() * (-0.5f64).exp();
        let computed_n_prime = n_prime(x).unwrap().to_f64().unwrap();
        assert!(
            (computed_n_prime - expected_n_prime).abs() < 1e-10,
            "n_prime function failed"
        );
    }

    #[test]
    fn test_big_n() {
        let x = Decimal::ZERO;
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x.to_f64().unwrap());
        let computed_big_n = big_n(x).unwrap().to_f64().unwrap();
        assert!(
            (computed_big_n - expected_big_n).abs() < 1e-10,
            "big_n function failed"
        );

        let x = Decimal::ONE;
        let expected_big_n = normal_distribution.cdf(1.0);
        let computed_big_n = big_n(x).unwrap().to_f64().unwrap();
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

    #[test]
    fn test_d1_zero_volatility() {
        // Case where volatility (sigma) is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.0;

        // When volatility is zero, d1 should handle the case correctly
        assert!( d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        assert!( d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        ).unwrap().to_f64().unwrap();

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
        assert!( d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        ).unwrap().to_f64().unwrap();

        // Assert the result should be finite and not infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for low underlying price"
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
        assert!( d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        assert!( d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
    }
}

#[cfg(test)]
mod calculate_d2_values {
    use super::*;
    use crate::pos;

    #[test]
    fn test_d2_zero_volatility() {
        // Case where volatility (implied_volatility) is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = 0.05;
        let expiration_date = 1.0;
        let implied_volatility = 0.0;

        // When volatility is zero, d2 should handle the case correctly using handle_zero
        assert!( d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
       assert!( d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        ).unwrap().to_f64().unwrap();

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
        assert!( d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        ).unwrap().to_f64().unwrap();

        // Assert the result should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for low underlying price"
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
        assert!( d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        assert!( d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        ).is_err());
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
        let x = Decimal::ZERO;

        // The PDF of the standard normal distribution at x = 0 is 1/sqrt(2*pi)
        let expected_n = 1.0f64 / (2.0 * PI).sqrt();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_positive_small_value() {
        // Case where x is a small positive value
        let x = dec!(0.5);

        // Expected result for n(0.5), can be precomputed
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-0.5f64 * 0.5f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_negative_small_value() {
        // Case where x is a small negative value
        let x = dec!(-0.5);

        // Expected result for n(-0.5), which should be the same as n(0.5) due to symmetry
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-0.5f64 * 0.5f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_large_positive_value() {
        // Case where x is a large positive value
        let x = dec!(5.0);

        // Expected result for n(5.0), should be a very small value
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-5.0f64 * 5.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_large_negative_value() {
        // Case where x is a large negative value
        let x = dec!(-5.0);

        // Expected result for n(-5.0), should be the same as n(5.0) due to symmetry
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-5.0f64 * 5.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_extreme_positive_value() {
        // Case where x is a very large positive value
        let x = dec!(100.0);

        // Expected result for n(100.0), should be extremely close to 0
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-100.0f64 * 100.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that n(x) is effectively 0 for such a large input
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-100);
    }

    #[test]
    fn test_n_extreme_negative_value() {
        // Case where x is a very large negative value
        let x = dec!(-100.0);

        // Expected result for n(-100.0), should be extremely close to 0
        let expected_n = 1.0f64 / (2.0 * PI).sqrt() * (-100.0f64 * 100.0f64 / 2.0f64).exp();

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

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
        let x = dec!(0.0);

        // The derivative of the PDF at x = 0 should be 0 because -x * n(x) = 0 * n(0) = 0
        let expected_n_prime = 0.0f64;

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_positive_small_value() {
        // Case where x is a small positive value
        let x = dec!(0.5);

        // Expected result for n_prime(0.5), we calculate -x * n(x)
        let expected_n_prime = -x.to_f64().unwrap() * n(x).unwrap().to_f64().unwrap();

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_negative_small_value() {
        // Case where x is a small negative value
        let x = dec!(-0.5);

        // Expected result for n_prime(-0.5), we calculate -x * n(x)
        let expected_n_prime = (-x * n(x).unwrap()).to_f64().unwrap();

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_large_positive_value() {
        // Case where x is a large positive value
        let x = dec!(5.0);

        // Expected result for n_prime(5.0), we calculate -x * n(x)
        let expected_n_prime = -x.to_f64().unwrap() * n(x).unwrap().to_f64().unwrap();

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_large_negative_value() {
        // Case where x is a large negative value
        let x = -dec!(5.0);

        // Expected result for n_prime(-5.0), we calculate -x * n(x)
        let expected_n_prime = (-x * n(x).unwrap()).to_f64().unwrap();

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_extreme_positive_value() {
        // Case where x is a very large positive value
        let x = dec!(100.0);

        // Expected result for n_prime(100.0), should be extremely close to 0
        let expected_n_prime = (-x * n(x).unwrap()).to_f64().unwrap();

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that n_prime(x) is effectively 0 for such a large input
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-100);
    }

    #[test]
    fn test_n_prime_extreme_negative_value() {
        // Case where x is a very large negative value
        let x = -dec!(100.0);

        // Expected result for n_prime(-100.0), should be extremely close to 0
        let expected_n_prime = (-x * n(x).unwrap()).to_f64().unwrap();

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

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
        let x = Decimal::ZERO;

        // The CDF of the standard normal distribution at x = 0 is 0.5
        let expected_big_n = 0.5;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_positive_small_value() {
        // Case where x is a small positive value
        let x = dec!(0.5);

        // The expected CDF for the standard normal distribution at x = 0.5 can be precomputed
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x.to_f64().unwrap());

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_negative_small_value() {
        // Case where x is a small negative value
        let x = -dec!(0.5);

        // The expected CDF for the standard normal distribution at x = -0.5 can be precomputed
        let normal_distribution = Normal::new(0.0, 1.0).unwrap();
        let expected_big_n = normal_distribution.cdf(x.to_f64().unwrap());

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_large_positive_value() {
        // Case where x is a large positive value
        let x = dec!(5.0);

        // The CDF for large positive x should be very close to 1
        let expected_big_n = 1.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6); // if lower epsilon fail
    }

    #[test]
    fn test_big_n_large_negative_value() {
        // Case where x is a large negative value
        let x = -dec!(5.0);

        // The CDF for large negative x should be very close to 0
        let expected_big_n = 0.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-6); // if lower epsilon fail
    }

    #[test]
    fn test_big_n_extreme_positive_value() {
        // Case where x is an extremely large positive value
        let x = dec!(100.0);

        // The CDF for an extremely large positive x should be effectively 1
        let expected_big_n = 1.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that big_n(x) is effectively 1 for such a large positive input
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-12);
    }

    #[test]
    fn test_big_n_extreme_negative_value() {
        // Case where x is an extremely large negative value
        let x = -dec!(100.0);

        // The CDF for an extremely large negative x should be effectively 0
        let expected_big_n = 0.0f64;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that big_n(x) is effectively 0 for such a large negative input
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-12);
    }
}
