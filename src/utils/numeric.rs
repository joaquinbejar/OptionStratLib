/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/9/24
******************************************************************************/
//! Numeric helpers owned by `core`.
//!
//! Tolerance-based `f64` comparison and the logarithmic-return transform
//! over a `Positive` price series. Both are pure functions over `core`
//! types with no layer above them in their signatures, so `core` owns
//! them; `market` and `simulation` are callers, not owners.

use crate::error::DecimalError;
use positive::Positive;
use rust_decimal::Decimal;

/// Precomputed f64 form of `crate::constants::TOLERANCE` (= 1e-8) so the
/// hot-path comparison can avoid the runtime fallible `Decimal::to_f64`.
const TOLERANCE_F64: f64 = 1e-8;

/// Checks for approximate equality between two f64 values within a defined tolerance.
///
/// This function compares two floating-point numbers and returns `true` if the absolute
/// difference between them is less than the predefined `TOLERANCE` constant.  It is useful
/// for comparing floating-point values that may be subject to small rounding errors.
///
/// # Arguments
///
/// * `a` - The first f64 value to compare.
/// * `b` - The second f64 value to compare.
///
/// # Returns
///
/// `true` if the absolute difference between `a` and `b` is less than `TOLERANCE`, `false` otherwise.
///
/// # Example
///
/// ```
/// use optionstratlib::utils::numeric::approx_equal;
///
/// let x = 1.0;
/// let y = 1.00000001;
/// assert!(approx_equal(x, y)); // Returns true
///
/// let x = 1.0;
/// let y = 1.1;
/// assert!(!approx_equal(x, y)); // Returns false
/// ```
#[allow(dead_code)]
#[must_use]
pub fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE_F64
}

/// # Calculate Logarithmic Returns
///
/// Computes the logarithmic returns from a series of close prices. Logarithmic returns
/// are calculated as the natural logarithm of the ratio between consecutive prices.
///
/// Logarithmic returns are commonly used in financial analysis because:
/// - They are additive over time, unlike percentage returns
/// - They better approximate a normal distribution
/// - They're suitable for statistical analysis of financial time series
///
/// ## Parameters
///
/// * `close_prices` - A slice of `Decimal` values representing sequential close prices
///
/// ## Returns
///
/// * `Result<Vec<Decimal>, DecimalError>` - A vector of logarithmic returns if successful,
///   or an error if the calculation fails
///
/// ## Errors
///
/// This function returns a `DecimalError` in the following cases:
/// - If any price value is zero or negative
/// - If there's a failure when converting `Decimal` to `f64` for logarithm calculation
/// - If there's a failure when converting the logarithm result back to `Decimal`
///
/// ## Notes
///
/// - Returns an empty vector if fewer than 2 price points are provided
/// - For consecutive prices P₁ and P₂, the log return is ln(P₂/P₁)
pub fn calculate_log_returns(close_prices: &[Positive]) -> Result<Vec<Decimal>, DecimalError> {
    if close_prices.len() < 2 {
        return Ok(Vec::new());
    }

    let mut log_returns = Vec::with_capacity(close_prices.len() - 1);

    for window in close_prices.windows(2) {
        let [previous_price, current_price] = window else {
            // `windows(2)` only ever yields pairs; the arm exists so the
            // destructuring stays irrefutable without an index.
            continue;
        };

        // Calculate price ratio. `Positive / Positive` aborts on a zero
        // divisor and on a quotient that leaves the representable range, and
        // `Positive::ln` aborts on a ratio that rounded to zero. A zero price
        // is an ordinary gap in a feed, not an adversarial input, and this
        // function's contract already promises an error for it.
        let ratio = current_price.checked_div(previous_price).map_err(|_| {
            DecimalError::arithmetic_error(
                "utils::calculate_log_returns",
                "price ratio is zero or not representable",
            )
        })?;
        log_returns.push(ratio.checked_ln().map_err(|_| {
            DecimalError::arithmetic_error(
                "utils::calculate_log_returns",
                "logarithm of a non-positive price ratio",
            )
        })?);
    }

    Ok(log_returns)
}

#[cfg(test)]
mod tests_approx_equal {
    use super::*;

    #[test]
    fn test_approx_equal_exact_values() {
        assert!(approx_equal(1.0, 1.0));
    }

    #[test]
    fn test_approx_equal_within_tolerance() {
        let a = 1.00000001;
        let b = 1.0;
        assert!(approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_outside_tolerance() {
        let a = 1.0001;
        let b = 1.0;
        assert!(!approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_negative_values() {
        let a = -1.00000001;
        let b = -1.0;
        assert!(approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_large_values_within_tolerance() {
        let a = 1000000.000000001;
        let b = 1000000.0;
        assert!(approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_large_values_outside_tolerance() {
        let a = 1000000.1;
        let b = 1000000.0;
        assert!(!approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_zero() {
        let a = 0.0;
        let b = 0.0;
        assert!(approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_zero_with_small_value() {
        let a = 0.000000001;
        let b = 0.0;
        assert!(approx_equal(a, b));
    }

    #[test]
    fn test_approx_equal_zero_outside_tolerance() {
        let a = 0.01;
        let b = 0.0;
        assert!(!approx_equal(a, b));
    }
}

#[cfg(test)]
mod tests_log_returns {
    use super::*;

    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;

    #[test]
    fn test_empty_input() {
        let prices: Vec<Positive> = Vec::new();
        let result = calculate_log_returns(&prices);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_single_input() {
        let prices = vec![Positive::HUNDRED];
        let result = calculate_log_returns(&prices);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_basic_calculation() {
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(105.0),
        ];
        let result = calculate_log_returns(&prices).unwrap();

        assert_eq!(result.len(), 2);

        // Manually calculate expected values
        // ln(110.0/100.0) ≈ ln(1.1) ≈ 0.09531
        // ln(105.0/110.0) ≈ ln(0.9545) ≈ -0.04652
        assert_relative_eq!(
            result[0].to_f64().unwrap_or(f64::NAN),
            0.09531018,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            result[1].to_f64().unwrap_or(f64::NAN),
            -0.04652,
            epsilon = 0.00001
        );
    }

    /// A zero price is reported, not aborted on. This function's contract
    /// has always promised an error for a non-positive price; before #456 the
    /// `Positive / Positive` divide aborted the caller's process instead.
    #[test]
    fn test_zero_price() {
        let prices = vec![Positive::HUNDRED, Positive::ZERO, pos_or_panic!(105.0)];
        assert!(calculate_log_returns(&prices).is_err());
    }

    #[test]
    #[should_panic]
    fn test_negative_price() {
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(-50.0),
            pos_or_panic!(105.0),
        ];
        let _ = calculate_log_returns(&prices);
    }

    #[test]
    fn test_realistic_stock_prices() {
        let prices = vec![
            pos_or_panic!(150.25),
            pos_or_panic!(151.50),
            pos_or_panic!(149.75),
            pos_or_panic!(152.25),
            pos_or_panic!(153.00),
        ];

        let result = calculate_log_returns(&prices).unwrap();

        assert_eq!(result.len(), 4);

        assert_relative_eq!(
            result[0].to_f64().unwrap_or(f64::NAN),
            0.00829,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            result[1].to_f64().unwrap_or(f64::NAN),
            -0.01161,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            result[2].to_f64().unwrap_or(f64::NAN),
            0.01656,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            result[3].to_f64().unwrap_or(f64::NAN),
            0.00491,
            epsilon = 0.00001
        );
    }

    #[test]
    fn test_large_price_movements() {
        // Test with some large price movements (both up and down)
        let prices = vec![
            Positive::HUNDRED,    // Starting price
            pos_or_panic!(200.0), // 100% increase
            pos_or_panic!(50.0),  // 75% decrease
            pos_or_panic!(300.0), // 500% increase
        ];

        let result = calculate_log_returns(&prices).unwrap();

        assert_eq!(result.len(), 3);

        // Expected returns:
        // ln(200.0/100.0) = ln(2.0) ≈ 0.6931
        // ln(50.0/200.0) = ln(0.25) ≈ -1.3863
        // ln(300.0/50.0) = ln(6.0) ≈ 1.7918

        assert_relative_eq!(
            result[0].to_f64().unwrap_or(f64::NAN),
            std::f64::consts::LN_2,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            result[1].to_f64().unwrap_or(f64::NAN),
            -1.3863,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            result[2].to_f64().unwrap_or(f64::NAN),
            1.7918,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_no_change_prices() {
        // Test with prices that don't change
        let prices = vec![Positive::HUNDRED, Positive::HUNDRED, Positive::HUNDRED];
        let result = calculate_log_returns(&prices).unwrap();

        assert_eq!(result.len(), 2);
        // ln(1.0) = 0
        assert_relative_eq!(
            result[0].to_f64().unwrap_or(f64::NAN),
            0.0,
            epsilon = 0.00001
        );
        assert_relative_eq!(
            result[1].to_f64().unwrap_or(f64::NAN),
            0.0,
            epsilon = 0.00001
        );
    }

    #[test]
    fn test_very_small_price_changes() {
        // Test with very small price changes
        let prices = vec![
            pos_or_panic!(100.000),
            pos_or_panic!(100.001),
            pos_or_panic!(100.002),
        ];

        let result = calculate_log_returns(&prices).unwrap();

        assert_eq!(result.len(), 2);

        // Expected returns are very small
        // ln(100.001/100.000) ≈ ln(1.00001) ≈ 0.00001
        // ln(100.002/100.001) ≈ ln(1.00001) ≈ 0.00001

        assert!(result[0].to_f64().unwrap_or(f64::NAN) > 0.0);
        assert!(result[0].to_f64().unwrap_or(f64::NAN) < 0.0001);
        assert!(result[1].to_f64().unwrap_or(f64::NAN) > 0.0);
        assert!(result[1].to_f64().unwrap_or(f64::NAN) < 0.0001);
    }
}
