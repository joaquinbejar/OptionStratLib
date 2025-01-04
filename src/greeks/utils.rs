/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::PI;
use crate::error::decimal::DecimalError;
use crate::error::greeks::{GreeksError, InputErrorKind, MathErrorKind};
use crate::model::decimal::f64_to_decimal;
use crate::Options;
use crate::Positive;
use core::f64;
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use statrs::distribution::{ContinuousCDF, Normal};

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
/// use rust_decimal_macros::dec;
/// use optionstratlib::greeks::d1;
/// use optionstratlib::{pos, Positive};
///
/// let underlying_price = pos!(100.0);
/// let strike_price = pos!(95.0);
/// let risk_free_rate = dec!(0.05);
/// let expiration_date = pos!(0.5); // 6 months
/// let implied_volatility = pos!(0.2);
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
    underlying_price: Positive,
    strike_price: Positive,
    risk_free_rate: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError> {
    let underlying_price: Decimal = underlying_price.to_dec();

    if underlying_price == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidStrike {
            value: underlying_price.to_string(),
            reason: "Underlying price price cannot be zero".to_string(),
        }));
    }

    if strike_price == Positive::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidStrike {
            value: strike_price.to_string(),
            reason: "Strike price cannot be zero".to_string(),
        }));
    }
    if implied_volatility == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value: implied_volatility.to_f64(),
            reason: "Implied volatility cannot be zero".to_string(),
        }));
    }
    if expiration_date == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidTime {
            value: expiration_date,
            reason: "Expiration date cannot be zero".to_string(),
        }));
    }

    // d1 = (ln(S / K) + (r + σ² / 2) * T) / (σ * sqrt(T))
    let implied_volatility_squared = implied_volatility.powd(Decimal::TWO);
    let ln_price_ratio = match strike_price {
        value if value == Positive::INFINITY => Decimal::MIN,
        _ => (underlying_price / strike_price).ln(),
    };

    let rate_vol_term = risk_free_rate + implied_volatility_squared / Decimal::TWO;
    let numerator = ln_price_ratio + rate_vol_term * expiration_date;
    let denominator = implied_volatility * expiration_date.sqrt();

    match numerator.checked_div(denominator.into()) {
        Some(result) => Ok(result),
        None => Err(GreeksError::MathError(MathErrorKind::Overflow)),
    }
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
/// use rust_decimal_macros::dec;
/// use optionstratlib::greeks::d2;
/// use optionstratlib::{pos, Positive};
/// let underlying_price = Positive::new(100.0).unwrap();
/// let strike_price = Positive::new(95.0).unwrap();
/// let risk_free_rate = dec!(0.05);
/// let expiration_date = pos!(0.5); // 6 months
/// let implied_volatility = pos!(0.2);
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
    underlying_price: Positive,
    strike_price: Positive,
    risk_free_rate: Decimal,
    expiration_date: Positive,
    implied_volatility: Positive,
) -> Result<Decimal, GreeksError> {
    if implied_volatility == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value: implied_volatility.to_f64(),
            reason: "Implied volatility cannot be zero".to_string(),
        }));
    }

    if expiration_date == Decimal::ZERO {
        return Err(GreeksError::InputError(InputErrorKind::InvalidTime {
            value: expiration_date,
            reason: "Expiration date cannot be zero".to_string(),
        }));
    }

    let d1_value = d1(
        underlying_price,
        strike_price,
        risk_free_rate,
        expiration_date,
        implied_volatility,
    )?;

    Ok(d1_value - implied_volatility * expiration_date.sqrt())
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
/// The function uses the `Decimal` type for precision and error handling. The result is returned
pub fn n(x: Decimal) -> Result<Decimal, GreeksError> {
    let norm_factor = Decimal::ONE / (Decimal::TWO * PI).sqrt().unwrap();
    let pre_pdf = -x.powd(Decimal::TWO) / Decimal::TWO;

    // avoid Exp underflowed
    if pre_pdf < Decimal::from_f64(-11.7).unwrap() {
        return Ok(Decimal::ZERO);
    }

    let pdf = pre_pdf.exp();
    Ok(norm_factor * pdf) // N(x) = [1 / sqrt(2 * PI)] * e^(-x^2 / 2)
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
pub(crate) fn n_prime(x: Decimal) -> Result<Decimal, GreeksError> {
    Ok(-x * n(x)?) // -x * n(x)
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
    f64_to_decimal(normal_distribution.cdf(x_f64.unwrap()))
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
pub(crate) fn calculate_d_values(option: &Options) -> Result<(Decimal, Decimal), GreeksError> {
    let d1_value = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years()?,
        option.implied_volatility,
    );
    let d2_value = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years()?,
        option.implied_volatility,
    );
    Ok((d1_value?, d2_value?))
}

#[cfg(test)]
mod tests_exp {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    #[should_panic]
    fn test_calculate_exp() {
        let decimal = dec!(-12.5);
        let _ = decimal.exp();
    }

    #[test]
    fn test_calculate_exp_no_panic() {
        let decimal = dec!(-11.7);
        let _ = decimal.exp();
    }
}

#[cfg(test)]
mod tests_calculate_d_values {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_d_values() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: pos!(110.0),
            underlying_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            implied_volatility: pos!(10.12),
            expiration_date: Default::default(),
            quantity: pos!(1.0),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        };
        let (d1_value, d2_value) = calculate_d_values(&option).unwrap();

        assert_relative_eq!(
            d1_value.to_f64().unwrap(),
            5.055522709505501,
            epsilon = 0.001
        );
        assert_relative_eq!(
            d2_value.to_f64().unwrap(),
            -5.064477290494499,
            epsilon = 0.001
        );
    }
}

#[cfg(test)]
mod tests_src_greeks_utils {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::FloatConst;
    use rust_decimal_macros::dec;
    use statrs::distribution::ContinuousCDF;
    use statrs::distribution::Normal;

    #[test]
    fn test_d1_zero_sigma() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = dec!(0.05);
        let t = Positive::ONE;
        let sigma = Positive::ZERO;
        let _ = d1(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d1_zero_t() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = dec!(0.05);
        let t = Positive::ZERO;
        let sigma = pos!(0.01);
        let _ = d1(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d2_bis_i() {
        let s = pos!(100.0);
        let k = pos!(110.0);
        let r = dec!(0.05);
        let t = Positive::TWO;
        let sigma = pos!(0.2);
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_d1, 0.15800237455184707, epsilon = 0.001);
        assert_relative_eq!(computed_d2, -0.12484033792277195, epsilon = 0.001);
    }

    #[test]
    fn test_d2_bis_ii() {
        let s = pos!(100.0);
        let k = pos!(95.0);
        let r = dec!(0.15);
        let t = Positive::ONE;
        let sigma = pos!(0.2);
        let computed_d2 = d2(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        let computed_d1 = d1(s, k, r, t, sigma).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_d1, 1.1064664719377526, epsilon = 0.001);
        assert_relative_eq!(computed_d2, 0.9064664719377528, epsilon = 0.001);
    }

    #[test]
    fn test_d2_zero_sigma() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = Decimal::ZERO;
        let t = Positive::ONE;
        let sigma = Positive::ZERO;
        let _ = d2(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_d2_zero_t() {
        let s = pos!(100.0);
        let k = pos!(100.0);
        let r = dec!(0.02);
        let t = Positive::ZERO;
        let sigma = pos!(0.01);
        let _ = d2(s, k, r, t, sigma).is_err();
    }

    #[test]
    fn test_n() {
        let x = Decimal::ZERO;
        let expected_n = 1.0 / (2.0 * f64::PI()).sqrt();
        let computed_n = n(x).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_n, expected_n, epsilon = 1e-8);

        let x = Decimal::ONE;
        let expected_n = 1.0 / (2.0 * f64::PI()).sqrt() * (-0.5f64).exp();
        let computed_n = n(x).unwrap().to_f64().unwrap();
        assert_relative_eq!(computed_n, expected_n, epsilon = 1e-8);
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
    use rust_decimal_macros::dec;

    #[test]
    fn test_d1_zero_volatility() {
        // Case where volatility (sigma) is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = Positive::ZERO;

        // When volatility is zero, d1 should handle the case correctly
        assert!(d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }

    #[test]
    fn test_d1_zero_time_to_expiry() {
        // Case where time to expiry is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ZERO;
        let implied_volatility = pos!(0.2);

        // When time to expiry is zero, d1 should handle the case correctly
        assert!(d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }

    #[test]
    fn test_d1_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(100.0); // Very high volatility

        // High volatility should result in a small or large value for d1
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

        // Assert the result should be finite and non-infinite
        assert!(
            calculated_d1.is_finite(),
            "d1 should not be infinite for high volatility"
        );
    }

    #[test]
    fn test_d1_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = Positive::INFINITY; // Very high stock price
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // Very high underlying price should result in a large d1 value
        assert!(d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_ok());
    }

    #[test]
    fn test_d1_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = pos!(0.01); // Very low stock price
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // Very low underlying price should result in a small or negative d1 value
        let calculated_d1 = d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

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
        let strike_price = Positive::ZERO;
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        assert!(d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }

    #[test]
    fn test_d1_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = Decimal::MAX; // Very high risk-free rate
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // High risk-free rate should result in a large d1 value, potentially infinite
        assert!(d1(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }
}

#[cfg(test)]
mod calculate_d1_values_bis {
    use super::*;
    use crate::error::greeks::{GreeksError, InputErrorKind};
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    // Helper function to convert Decimal to f64 for testing
    fn decimal_to_f64_test(d: Decimal) -> f64 {
        d.to_f64().unwrap()
    }

    #[test]
    fn test_d1_basic_calculation() {
        let result = d1(
            pos!(100.0),
            pos!(90.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.8768025782891316, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_in_the_money() {
        let result = d1(
            pos!(110.0),
            pos!(90.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 1.3533534773107558, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_out_of_the_money() {
        let result = d1(
            pos!(90.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, -0.1768025782891315, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_zero_strike_error() {
        let result = d1(pos!(100.0), pos!(0.0), dec!(0.05), Positive::ONE, pos!(0.2));

        assert!(matches!(
            result,
            Err(GreeksError::InputError(
                InputErrorKind::InvalidStrike { .. }
            ))
        ));
    }

    #[test]
    fn test_d1_zero_volatility_error() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.0),
        );

        assert!(matches!(
            result,
            Err(GreeksError::InputError(
                InputErrorKind::InvalidVolatility { .. }
            ))
        ));
    }

    #[test]
    fn test_d1_zero_time_error() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ZERO,
            pos!(0.2),
        );

        assert!(matches!(
            result,
            Err(GreeksError::InputError(InputErrorKind::InvalidTime { .. }))
        ));
    }

    #[test]
    fn test_d1_short_expiry() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            pos!(0.0833),   // approximately one month
            pos!(0.05),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.29583282863806715, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_high_volatility() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.5),   // 50% volatility
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.35, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_zero_interest_rate() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(0.0),
            Positive::ONE,
            pos!(0.5),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.25, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_negative_interest_rate() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(-0.02),   // negative interest rate
            Positive::ONE,
            pos!(0.5),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.21, epsilon = 0.0001);
    }

    #[test]
    fn test_d1_negative_interest_rate_bis() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(-0.02),   // negative interest rate
            Positive::ONE,
            pos!(0.5),
        );

        assert!(result.is_ok());
        let d1_value = decimal_to_f64_test(result.unwrap());
        assert_relative_eq!(d1_value, 0.21, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod calculate_d2_values {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_d2_zero_volatility() {
        // Case where volatility (implied_volatility) is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = Positive::ZERO;

        // When volatility is zero, d2 should handle the case correctly using handle_zero
        assert!(d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }

    #[test]
    fn test_d2_zero_time_to_expiry() {
        // Case where time to expiration is zero
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ZERO;
        let implied_volatility = pos!(0.2);

        // When time to expiration is zero, handle_zero should be called
        assert!(d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }

    #[test]
    fn test_d2_high_volatility() {
        // Case with extremely high volatility
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(100.0); // Very high volatility

        // High volatility should result in a significant negative shift in d2
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

        // d2 should be finite and not infinite
        assert!(
            calculated_d2.is_finite(),
            "d2 should not be infinite for high volatility"
        );
    }

    #[test]
    fn test_d2_high_underlying_price() {
        // Case with extremely high underlying price
        let underlying_price = Positive::INFINITY;
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // Very high underlying price should result in a large d2 value
        assert!(d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_ok());
    }

    #[test]
    fn test_d2_low_underlying_price() {
        // Case with extremely low underlying price (near zero)
        let underlying_price = pos!(0.01);
        let strike_price = pos!(100.0);
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // Very low underlying price should result in a small or negative d2 value
        let calculated_d2 = d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .unwrap()
        .to_f64()
        .unwrap();

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
        let strike_price = Positive::ZERO;
        let risk_free_rate = dec!(0.05);
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // Since strike price is zero, the function should call handle_zero and return positive infinity
        assert!(d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }

    #[test]
    fn test_d2_infinite_risk_free_rate() {
        // Case where risk-free rate is very high (infinite-like)
        let underlying_price = pos!(100.0);
        let strike_price = pos!(100.0);
        let risk_free_rate = Decimal::MAX; // Very high risk-free rate
        let expiration_date = Positive::ONE;
        let implied_volatility = pos!(0.2);

        // High risk-free rate should result in a large d2 value, potentially infinite
        assert!(d2(
            underlying_price,
            strike_price,
            risk_free_rate,
            expiration_date,
            implied_volatility,
        )
        .is_err());
    }
}

#[cfg(test)]
mod calculate_d2_values_bis {
    use super::*;
    use crate::{assert_decimal_eq, pos};
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(0.0001);
    // Normal test cases
    #[test]
    fn test_d2_atm_option() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.15, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_itm_call() {
        let result = d2(
            pos!(110.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_decimal_eq!(result, dec!(0.6265508990216243), EPSILON);
    }

    #[test]
    fn test_d2_otm_call() {
        let result = d2(
            pos!(90.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -0.3768025782891315,
            epsilon = 0.0001
        );
    }

    // Time to expiration variations
    #[test]
    fn test_d2_short_expiry() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            pos!(0.0833),   // 1 month
            pos!(0.5),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -0.04329260906898544,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_long_expiry() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::TWO,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.21213203435596426,
            epsilon = 0.0001
        );
    }

    // Volatility variations
    #[test]
    fn test_d2_low_volatility() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.1),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.45, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_high_volatility() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.5),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), -0.15, epsilon = 0.0001);
    }

    // Interest rate variations
    #[test]
    fn test_d2_zero_interest() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            Decimal::ZERO,
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), -0.1, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_high_interest() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.10),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.4, epsilon = 0.0001);
    }

    // Extreme price differences
    #[test]
    fn test_d2_deep_itm() {
        let result = d2(
            pos!(200.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            3.6157359027997265,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_deep_otm() {
        let result = d2(
            pos!(50.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -3.3157359027997266,
            epsilon = 0.0001
        );
    }

    // Very small values
    #[test]
    fn test_d2_small_price() {
        let result = d2(pos!(0.01), pos!(0.01), dec!(0.05), Positive::ONE, pos!(0.2)).unwrap();
        assert_relative_eq!(result.to_f64().unwrap(), 0.15, epsilon = 0.0001);
    }

    #[test]
    fn test_d2_small_time() {
        let result = d2(pos!(100.0), pos!(100.0), dec!(0.05), pos!(0.001), pos!(0.2)).unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.004743416490252569,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_small_volatility() {
        let result = d2(
            pos!(200.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.01),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            74.30971805599454,
            epsilon = 0.0001
        );
    }

    // Error cases
    #[test]
    fn test_d2_zero_volatility() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.0),
        );
        assert!(matches!(
            result,
            Err(GreeksError::InputError(
                InputErrorKind::InvalidVolatility { .. }
            ))
        ));
    }

    #[test]
    fn test_d2_zero_time() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ZERO,
            pos!(0.2),
        );
        assert!(matches!(
            result,
            Err(GreeksError::InputError(InputErrorKind::InvalidTime { .. }))
        ));
    }

    // Negative interest rate
    #[test]
    fn test_d2_negative_interest() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            -dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_decimal_eq!(result, dec!(-0.35), EPSILON);
    }

    // Combined extreme cases
    #[test]
    fn test_d2_combined_extremes_high() {
        let result = d2(pos!(1000.0), pos!(100.0), dec!(0.15), pos!(5.0), pos!(0.8)).unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.812019752759385,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_d2_combined_extremes_low() {
        let result = d2(pos!(10.0), pos!(100.0), dec!(0.01), pos!(0.1), pos!(0.05)).unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -145.57292814518308,
            epsilon = 0.0001
        );
    }

    // Edge cases with very large numbers
    #[test]
    fn test_d2_large_price_ratio() {
        let result = d2(
            pos!(1_000_000.0),
            pos!(1.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            69.22755278982137,
            epsilon = 0.0001
        );
    }

    // Special case: ATM LEAPS (Long-term equity anticipation securities)
    #[test]
    fn test_d2_leaps() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            pos!(2.5),   // 2.5 years
            pos!(0.15),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            0.40846086443841567,
            epsilon = 0.0001
        );
    }

    // Near-zero but valid cases
    #[test]
    fn test_d2_near_zero_valid_values() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            dec!(0.0001),
            pos!(0.01),
            pos!(0.001),
        )
        .unwrap();
        assert!(result.to_f64().unwrap().abs() < 1.0);
    }

    // Test with maximum realistic market values
    #[test]
    fn test_d2_max_realistic_values() {
        let result = d2(
            pos!(10000.0),
            pos!(5000.0),
            dec!(0.20),
            pos!(3.0),
            pos!(1.5),
        )
        .unwrap();
        assert_relative_eq!(
            result.to_f64().unwrap(),
            -0.8013055238112647,
            epsilon = 0.0001
        );
    }
}

#[cfg(test)]
mod calculate_n_values {
    use super::*;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;
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
    fn test_n_one() {
        // Case where x = 0.0
        let x = Decimal::ONE;

        // The PDF of the standard normal distribution at x = 1 is 0.24197072535043143
        let expected_n = 0.24197072535043143;

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, expected_n, epsilon = 1e-8);
    }

    #[test]
    fn test_n_two() {
        // Case where x = 0.0
        let x = Decimal::TWO;

        // The PDF of the standard normal distribution at x = 2 is 0.05399096672219953
        let expected_n = 0.05399096672219953;

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

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_n_large_negative_value() {
        // Case where x is a large negative value
        let x = dec!(-5.0);

        // Compute n(x)
        let calculated_n = n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n, 0.0, epsilon = 1e-8);
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
    use rust_decimal_macros::dec;

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
    fn test_n_prime_one() {
        // Case where x = 0.0
        let x = Decimal::ONE;

        // The derivative of the PDF at x = 1 should be 0.24197072535043143f64 (n(1) = 0.24197072535043143)
        let expected_n_prime = -0.24197072535043143f64;

        // Compute n_prime(x)
        let calculated_n_prime = n_prime(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_n_prime, expected_n_prime, epsilon = 1e-8);
    }

    #[test]
    fn test_n_prime_two() {
        // Case where x = 0.0
        let x = Decimal::TWO;

        // The derivative of the PDF at x = 2 should be -0.10798193344439906 (n(2) = 0.05399096672219953)
        let expected_n_prime = -0.10798193344439906;

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
    use rust_decimal_macros::dec;
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
    fn test_big_n_one() {
        // Case where x = 0.0
        let x = Decimal::ONE;

        // The CDF of the standard normal distribution at x = 1 is 0.841344746054943
        let expected_big_n = 0.841344746054943;

        // Compute big_n(x)
        let calculated_big_n = big_n(x).unwrap().to_f64().unwrap();

        // Assert that the calculated value is close to the expected value
        assert_relative_eq!(calculated_big_n, expected_big_n, epsilon = 1e-8);
    }

    #[test]
    fn test_big_n_two() {
        // Case where x = 0.0
        let x = Decimal::TWO;

        // The CDF of the standard normal distribution at x = 2 is 0.977249868052837
        let expected_big_n = 0.977249868052837;

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

#[cfg(test)]
mod tests_d1_d2_edge_cases {
    use super::*;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_d1_zero_underlying_price() {
        let result = d1(
            Positive::ZERO,
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(0.2),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_d2_negative_rates_and_high_volatility() {
        let result = d2(
            pos!(100.0),
            pos!(100.0),
            -dec!(0.05),   // tasa negativa
            Positive::ONE,
            pos!(0.8),   // alta volatilidad
        )
        .unwrap();
        assert_decimal_eq!(result, dec!(-0.4625), dec!(0.000001));
    }

    #[test]
    fn test_d1_d2_combination_extreme_values() {
        let result_d1 = d1(
            pos!(1000.0),
            pos!(10.0),
            dec!(0.15),
            Positive::TEN,
            pos!(0.9),
        )
        .unwrap();
        let result_d2 = d2(
            pos!(1000.0),
            pos!(10.0),
            dec!(0.15),
            Positive::TEN,
            pos!(0.9),
        )
        .unwrap();
        assert_decimal_eq!(result_d1, dec!(3.5681), dec!(0.0001));
        assert_decimal_eq!(result_d2, dec!(0.7221), dec!(0.0001));
    }
}

#[cfg(test)]
mod tests_probability_density {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_n_prime_symmetry() {
        let x = dec!(1.5);
        let n_prime_pos = n_prime(x).unwrap();
        let n_prime_neg = n_prime(-x).unwrap();
        assert_eq!(n_prime_pos, -n_prime_neg);
    }

    #[test]
    fn test_n_integration_limits() {
        let x_very_large = dec!(10.0);
        let result = n(x_very_large).unwrap();
        assert!(result < dec!(0.0001));
    }

    #[test]
    fn test_n_prime_zero_crossing() {
        let result = n_prime(dec!(0.0)).unwrap();
        assert_eq!(result, dec!(0.0));
    }
}

#[cfg(test)]
mod tests_cumulative_distribution {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_big_n_continuity() {
        let x1 = dec!(0.001);
        let x2 = dec!(-0.001);
        let result1 = big_n(x1).unwrap();
        let result2 = big_n(x2).unwrap();
        assert!((result1 - result2).abs() < dec!(0.001));
    }

    #[test]
    fn test_big_n_conversion_error() {
        let x = Decimal::MAX;
        let result = big_n(x);
        assert!(result.is_ok());
    }

    #[test]
    fn test_big_n_boundary_values() {
        let result_zero = big_n(dec!(0.0)).unwrap();
        assert_eq!(result_zero, dec!(0.5));
    }
}

#[cfg(test)]
mod tests_calculate_d_values_bis {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::model::ExpirationDate;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_d_values_with_expiration() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: pos!(100.0),
            underlying_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            implied_volatility: pos!(0.5),
            expiration_date: ExpirationDate::Days(pos!(30.0)),
            quantity: pos!(1.0),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        };
        let (d1, d2) = calculate_d_values(&option).unwrap();
        assert_decimal_eq!(d1, dec!(0.1003), dec!(0.0001));
        assert_decimal_eq!(d2, dec!(-0.0430), dec!(0.0001));
    }
}

#[cfg(test)]
mod tests_edge_cases_and_errors {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_extreme_volatility_values() {
        let result = d1(
            pos!(100.0),
            pos!(100.0),
            dec!(0.05),
            Positive::ONE,
            pos!(1000.0),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_precision_limits() {
        let x = dec!(15.0);
        let n_result = n(x).unwrap();
        assert!(n_result.to_f64().unwrap() == 0.0);
    }
}
