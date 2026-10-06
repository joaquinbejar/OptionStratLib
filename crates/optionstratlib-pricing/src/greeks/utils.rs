/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::error::greeks::{DeltaNeutralityErrorKind, GreeksError};
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_div, d_exp, d_mul, d_powd, d_sub};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// # Delta Neutrality Threshold
///
/// The default threshold value used to determine if an options strategy is considered delta neutral.
///
/// When evaluating delta neutrality, a strategy's net delta is compared against this threshold value.
/// If the absolute value of the net delta is less than or equal to this threshold, the strategy
/// is considered delta neutral.
///
/// ## Value Significance
/// The small value (0.0001) represents a very tight threshold, meaning the strategy must have
/// extremely minimal directional exposure to be considered neutral. This conservative threshold
/// helps ensure strategies maintain strict delta neutrality for effective risk management.
///
/// ## Usage Context
/// This constant is primarily used within delta neutrality calculations and serves as a default
/// when a custom threshold is not specified. Functions that analyze or adjust strategies for
/// delta neutrality may use this value when determining if additional position adjustments
/// are necessary.
///
/// ## Related Components
/// Owned by the Greeks layer because `calculate_delta_neutral_sizes` needs it;
/// `greeks::DELTA_THRESHOLD` is its only public path.
pub const DELTA_THRESHOLD: Decimal = dec!(0.0001);

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
/// - The normalisation factor \(1 / \sqrt{2 \pi}\) is held as a precomputed
///   `Decimal` constant (`dec!(0.3989422804014326779399461)`), avoiding a
///   runtime fallible `sqrt()` on `Decimal`.
/// - The exponent is computed as \(-\frac{x^2}{2}\).
/// - The PDF value is the product of the normalisation factor and the
///   exponential term.
/// - For very-negative exponents (`pre_pdf < -11.7`) `exp` underflows to
///   `0` in `Decimal` arithmetic, so we short-circuit to `Decimal::ZERO`.
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
/// use tracing::{error, info};
/// use optionstratlib_pricing::greeks::n;
///
/// let x = Decimal::new(100, 2); // 1.00
///
/// match n(x) {
///     Ok(result) => info!("N(x): {}", result),
///     Err(e) => error!("Error calculating N(x): {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// The function uses the `Decimal` type for precision and error handling. The result is returned
#[inline]
pub fn n(x: Decimal) -> Result<Decimal, GreeksError> {
    // 1 / sqrt(2π) — standard normal PDF normalisation constant.
    // Precomputed so we avoid the runtime fallible sqrt on Decimal.
    const NORM_FACTOR: Decimal = dec!(0.3989422804014326779399461);

    // `x²` overflows the `Decimal` range for a large `|x|`, and `powd`
    // panics rather than reporting it. The pdf is already flushed to zero
    // below `|x| = 4.84` (the `-11.7` cut-off underneath), so anything past
    // 5 returns zero without ever squaring it.
    if x.abs() > dec!(5) {
        return Ok(Decimal::ZERO);
    }

    let x_squared = d_powd(x, Decimal::TWO, "greeks::n::x_squared")?;
    let pre_pdf = -d_div(x_squared, Decimal::TWO, "greeks::n::half_x_squared")?;

    // avoid Exp underflowed
    if pre_pdf < dec!(-11.7) {
        return Ok(Decimal::ZERO);
    }

    let pdf = d_exp(pre_pdf, "greeks::n::exp")?;
    Ok(d_mul(NORM_FACTOR, pdf, "greeks::n::normalisation")?) // N(x) = [1 / sqrt(2 * PI)] * e^(-x^2 / 2)
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
#[inline]
pub(crate) fn n_prime(x: Decimal) -> Result<Decimal, GreeksError> {
    Ok(-x * n(x)?) // -x * n(x)
}

/// Calculates the optimal position sizes for two positions to achieve delta neutrality
/// while maintaining a specified total position size.
///
/// # Arguments
/// * `delta1` - Delta of the first position (e.g., short call delta)
/// * `delta2` - Delta of the second position (e.g., short put delta)
/// * `total_size` - Desired total position size (sum of both positions)
///
/// # Returns
/// * `Ok((size1, size2))` - Tuple containing the calculated sizes for each position
/// * `Err(String)` - Error message if calculation is not possible
///
/// # Example
/// ```rust
/// # fn run() -> Result<(), Box<dyn std::error::Error>> {
/// use rust_decimal_macros::dec;
/// use optionstratlib_pricing::greeks::calculate_delta_neutral_sizes;
/// use optionstratlib_core::pos_or_panic;
/// let (call_size, put_size) = calculate_delta_neutral_sizes(
///     dec!(-0.30),  // Short call delta
///     dec!(0.20),   // Short put delta
///     pos_or_panic!(7.0)     // Total desired position size
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`GreeksError::DeltaNeutrality`] with
/// [`DeltaNeutralityErrorKind::ZeroDelta`] when both deltas are zero,
/// [`DeltaNeutralityErrorKind::EqualDeltas`] when `delta1 == delta2`,
/// [`DeltaNeutralityErrorKind::SameSignDeltas`] when the two deltas
/// share the same sign (no neutral mix exists), or
/// [`DeltaNeutralityErrorKind::NegativePositionSize`] when the derived
/// sizes fall below zero for the provided `total_size`.
pub fn calculate_delta_neutral_sizes(
    delta1: Decimal,
    delta2: Decimal,
    total_size: Positive,
) -> Result<(Positive, Positive), GreeksError> {
    // The equation we want to solve is:
    // delta1 * size1 + delta2 * size2 = Decimal::ZERO
    // size1 + size2 = total_size

    if delta1.is_zero() || delta2.is_zero() {
        return Err(DeltaNeutralityErrorKind::ZeroDelta.into());
    }

    // Validate inputs
    if delta1 == delta2 {
        return Err(DeltaNeutralityErrorKind::EqualDeltas.into());
    }

    if delta1.is_sign_positive() == delta2.is_sign_positive() {
        return Err(DeltaNeutralityErrorKind::SameSignDeltas.into());
    }

    // Delta-neutral sizing: size1 = -total_size · delta2 / (delta1 - delta2).
    // The multiplication and subtraction are both monetary (signed position
    // quantities on a potentially large total_size) and the division is the
    // only site where the outcome is projected onto the `Positive` invariant,
    // so overflow here has to surface explicitly rather than silently wrap.
    let weighted = d_mul(
        -total_size.to_dec(),
        delta2,
        "greeks::delta_neutral::size1::weighted",
    )?;
    let spread = d_sub(delta1, delta2, "greeks::delta_neutral::size1::spread")?;
    let size1 = Positive::new_decimal(d_div(weighted, spread, "greeks::delta_neutral::size1")?)?;
    let size2 = total_size - size1;

    // Validate results
    if size1 < Decimal::ZERO || size2 < Decimal::ZERO {
        return Err(DeltaNeutralityErrorKind::NegativePositionSize.into());
    }

    // Verify the solution
    let total_delta: Decimal = size1.to_dec() * delta1 + size2.to_dec() * delta2;
    if total_delta.abs() > DELTA_THRESHOLD {
        // Allow small numerical errors
        return Err(DeltaNeutralityErrorKind::NotAchievable.into());
    }
    let total_size_check = size1 + size2;
    if (total_size_check.to_dec() - total_size.to_dec()).abs() > DELTA_THRESHOLD {
        return Err(DeltaNeutralityErrorKind::SizeMismatch {
            calculated: total_size_check,
            expected: total_size,
        }
        .into());
    }

    Ok((size1, size2))
}

#[cfg(test)]
mod tests_calculate_delta_neutral_sizes {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::{assert_pos_relative_eq, pos_or_panic};
    use rust_decimal_macros::dec;

    #[test]
    fn test_valid_delta_neutral_calculation() {
        let result = calculate_delta_neutral_sizes(
            dec!(-0.30),        // Short call delta
            dec!(0.20),         // Short put delta
            pos_or_panic!(7.0), // Total size
        )
        .unwrap();

        let (size1, size2) = result;

        // Check total size constraint
        assert_pos_relative_eq!(size1 + size2, pos_or_panic!(7.0), pos_or_panic!(0.0001));

        // Check delta neutrality (mixed-sign deltas — compute in Decimal).
        let total_delta = size1.to_dec() * dec!(-0.30) + size2.to_dec() * dec!(0.20);
        assert_decimal_eq!(total_delta, Decimal::ZERO, dec!(0.0001));
    }

    #[test]
    fn test_equal_deltas() {
        let result = calculate_delta_neutral_sizes(dec!(0.25), dec!(0.25), pos_or_panic!(10.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_impossible_neutrality() {
        let result = calculate_delta_neutral_sizes(dec!(-0.95), dec!(-0.90), pos_or_panic!(10.0));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod calculate_n_values {
    use super::*;
    use approx::assert_relative_eq;
    use num_traits::{FloatConst, ToPrimitive};
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
    fn test_precision_limits() {
        let x = dec!(15.0);
        let n_result = n(x).unwrap();
        assert!(n_result.to_f64().unwrap() == 0.0);
    }
}

#[cfg(test)]
mod calculate_n_prime_values {
    use super::*;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
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
mod tests_delta_neutral_extremes {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_large_deltas_on_a_large_size_report_the_overflow() {
        // `size · delta` leaves the `Decimal` range for a large book against
        // large deltas. The raw operator aborts; the caller must get an error.
        let result = calculate_delta_neutral_sizes(dec!(10), dec!(-10), Positive::MAX);
        assert!(result.is_err(), "expected an error, got {result:?}");
    }

    #[test]
    fn test_ordinary_deltas_still_size_the_pair() {
        let (size1, size2) = match calculate_delta_neutral_sizes(
            dec!(0.5),
            dec!(-0.5),
            Positive::new(10.0).expect("literal is positive"),
        ) {
            Ok(sizes) => sizes,
            Err(e) => panic!("sizing should succeed: {e:?}"),
        };
        assert_eq!(size1, size2);
        assert_eq!(
            size1 + size2,
            Positive::new(10.0).expect("literal is positive")
        );
    }
}
