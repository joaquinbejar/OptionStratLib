/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/
use crate::error::decimal::DecimalError;
use crate::geometrics::HasX;
use num_traits::{FromPrimitive, ToPrimitive};
use rand::distr::Distribution;
use rand_distr::Normal;
use rust_decimal::{Decimal, MathematicalOps, RoundingStrategy};
use rust_decimal_macros::dec;

/// Represents the daily interest rate factor used for financial calculations,
/// approximately equivalent to 1/252 (a standard value for the number of trading days in a year).
///
/// This constant converts annual interest rates to daily rates by providing a division factor.
/// The value 0.00396825397 corresponds to 1/252, where 252 is the typical number of trading
/// days in a financial year.
///
/// # Usage
///
/// This constant is commonly used in financial calculations such as:
/// - Converting annual interest rates to daily rates
/// - Time value calculations for options pricing
/// - Discounting cash flows on a daily basis
/// - Interest accrual calculations
pub const ONE_DAY: Decimal = dec!(0.00396825397);

/// Asserts that two Decimal values are approximately equal within a given epsilon
#[macro_export]
macro_rules! assert_decimal_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = ($left - $right).abs();
        assert!(
            diff <= $epsilon,
            "assertion failed: `(left == right)`\n  left: `{}`\n right: `{}`\n  diff: `{}`\n epsilon: `{}`",
            $left,
            $right,
            diff,
            $epsilon
        );
    };
}

/// Defines statistical operations for collections of decimal values.
///
/// This trait provides methods to calculate common statistical measures
/// for sequences or collections of `Decimal` values. It allows implementing
/// types to offer standardized statistical analysis capabilities.
///
/// ## Key Features
///
/// * Basic statistical calculations for `Decimal` collections
/// * Consistent interface for various collection types
/// * Precision-preserving operations using the `Decimal` type
///
/// ## Available Statistics
///
/// * `mean`: Calculates the arithmetic mean (average) of the values
/// * `std_dev`: Calculates the standard deviation, measuring the dispersion from the mean
///
/// ## Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use optionstratlib::model::decimal::DecimalStats;
///
/// struct DecimalSeries(Vec<Decimal>);
///
/// impl DecimalStats for DecimalSeries {
///     fn mean(&self) -> Decimal {
///         let sum: Decimal = self.0.iter().sum();
///         if self.0.is_empty() {
///             dec!(0)
///         } else {
///             sum / Decimal::from(self.0.len())
///         }
///     }
///     
///     fn std_dev(&self) -> Decimal {
///         // Implementation of standard deviation calculation
///         // ...
///         dec!(0) // Placeholder return
///     }
/// }
/// ```
pub trait DecimalStats {
    /// Calculates the arithmetic mean (average) of the collection.
    ///
    /// The mean is the sum of all values divided by the count of values.
    /// This method should handle empty collections appropriately.
    fn mean(&self) -> Decimal;

    /// Calculates the standard deviation of the collection.
    ///
    /// The standard deviation measures the amount of variation or dispersion
    /// from the mean. A low standard deviation indicates that values tend to be
    /// close to the mean, while a high standard deviation indicates values are
    /// spread out over a wider range.
    fn std_dev(&self) -> Decimal;
}

impl DecimalStats for Vec<Decimal> {
    fn mean(&self) -> Decimal {
        if self.is_empty() {
            return Decimal::ZERO;
        }
        let sum: Decimal = self.iter().sum();
        sum / Decimal::from(self.len())
    }

    fn std_dev(&self) -> Decimal {
        // Population variance of a single value is zero
        if self.len() < 2usize {
            return Decimal::ZERO;
        }
        let mean = self.mean();
        let variance: Decimal = self.iter().map(|x| (x - mean).powd(Decimal::TWO)).sum();
        (variance / Decimal::from(self.len() - 1))
            .sqrt()
            .unwrap_or(Decimal::ZERO)
    }
}

/// Converts a Decimal value to an f64.
///
/// This function attempts to convert a Decimal value to an f64 floating-point number.
/// If the conversion fails, it returns a DecimalError with detailed information about
/// the failure.
///
/// # Parameters
///
/// * `value` - The Decimal value to convert
///
/// # Returns
///
/// * `Result<f64, DecimalError>` - The converted f64 value if successful, or a DecimalError
///   if the conversion fails
///
/// # Errors
///
/// Returns [`DecimalError::ConversionError`] when the `Decimal` operand cannot
/// be represented as an `f64` (e.g. out-of-range magnitude or precision loss
/// beyond the `f64` mantissa).
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use tracing::info;
/// use optionstratlib::model::decimal::decimal_to_f64;
///
/// let decimal = dec!(3.14159);
/// match decimal_to_f64(decimal) {
///     Ok(float) => info!("Converted to f64: {}", float),
///     Err(e) => info!("Conversion error: {:?}", e)
/// }
/// ```
pub fn decimal_to_f64(value: Decimal) -> Result<f64, DecimalError> {
    value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: format!("Decimal: {value}"),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })
}

/// Converts an f64 floating-point number to a Decimal.
///
/// This function attempts to convert an f64 floating-point number to a Decimal value.
/// If the conversion fails (for example, if the f64 represents NaN, infinity, or is otherwise
/// not representable as a Decimal), it returns a DecimalError with detailed information about
/// the failure.
///
/// # Parameters
///
/// * `value` - The f64 value to convert
///
/// # Returns
///
/// * `Result<Decimal, DecimalError>` - The converted Decimal value if successful, or a DecimalError
///   if the conversion fails
///
/// # Errors
///
/// Returns [`DecimalError::ConversionError`] when the `f64` operand is not
/// representable as a `Decimal`, for example `NaN`, `±Infinity`, or a value
/// whose magnitude exceeds the `Decimal` range.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use tracing::info;
/// use optionstratlib::model::decimal::f64_to_decimal;
///
/// let float = std::f64::consts::PI;
/// match f64_to_decimal(float) {
///     Ok(decimal) => info!("Converted to Decimal: {}", decimal),
///     Err(e) => info!("Conversion error: {:?}", e)
/// }
/// ```
pub fn f64_to_decimal(value: f64) -> Result<Decimal, DecimalError> {
    Decimal::from_f64(value).ok_or(DecimalError::ConversionError {
        from_type: format!("f64: {value}"),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

/// Attempts to convert a finite `f64` into a `Decimal`.
///
/// Returns `None` when `value` is not finite (`NaN`, `+∞`, `-∞`) or
/// when `Decimal::from_f64` rejects the conversion (the latter is
/// vanishingly rare for representable `f64`). Crate-private helper
/// that standardises the `is_finite()` check paired with
/// `Decimal::from_f64` at every `f64` → `Decimal` boundary inside
/// pricing, Greeks, volatility, and simulation kernels.
///
/// Callers wrap the `None` case with a domain-specific
/// `*Error::NonFinite { context, value }` via `ok_or_else`:
///
/// ```ignore
/// let v = finite_decimal(v_f64)
///     .ok_or_else(|| PricingError::non_finite("pricing::bs::call::d1", v_f64))?;
/// ```
///
/// The guard is enforced at the public boundary of every `f64`
/// numerical kernel per the rules (`rules/global_rules.md`
/// §Arithmetic).
#[must_use]
#[inline]
pub(crate) fn finite_decimal(value: f64) -> Option<Decimal> {
    if value.is_finite() {
        Decimal::from_f64(value)
    } else {
        None
    }
}

/// Generates a random positive value from a standard normal distribution.
///
/// This function samples from a normal distribution with mean 0.0 and standard
/// deviation 1.0, and returns the value as a `Positive` type. Since the normal
/// distribution can produce negative values, the function uses the `pos!` macro
/// to convert the sample to a `Positive` value, which will handle the conversion
/// according to the `Positive` type's implementation.
///
/// # Returns
///
/// A `Positive` value sampled from a standard normal distribution.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::model::decimal::decimal_normal_sample;
/// use positive::Positive;
/// let normal = decimal_normal_sample();
/// ```
///
/// # Panics
///
/// The `unreachable!` branch fires only if
/// `statrs::distribution::Normal::new(0.0, 1.0)` rejects the provided
/// parameters. `statrs` accepts any finite mean together with a
/// strictly positive standard deviation, so `(0.0, 1.0)` is always
/// valid under the `statrs` contract and the arm is unreachable in
/// practice. Kept as a panic rather than `Result` to preserve the
/// infallible sampling API.
#[must_use]
pub fn decimal_normal_sample() -> Decimal {
    let mut t_rng = rand::rng();
    // Normal::new(0.0, 1.0) is provably valid (mean=0, std=1 are accepted
    // by `statrs::distribution::Normal`), so the Err arm is unreachable.
    let normal = match Normal::new(0.0, 1.0) {
        Ok(n) => n,
        Err(_) => unreachable!("standard normal parameters are always valid"),
    };
    Decimal::from_f64(normal.sample(&mut t_rng)).unwrap_or(Decimal::ZERO)
}

impl HasX for Decimal {
    fn get_x(&self) -> Decimal {
        *self
    }
}

/// Scale applied to banker's-rounding divisions in [`d_div`].
///
/// `28` matches `Decimal::MAX_SCALE` so a rounded division preserves every
/// digit of precision the backing 96-bit mantissa can represent without
/// triggering a later rescale overflow. Divisions that need a different
/// scale (for example a P&L that rounds to cents) should apply a subsequent
/// explicit `.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven)`.
pub(crate) const DIV_DEFAULT_SCALE: u32 = 28;

/// Checked `Decimal` addition with operand-preserving overflow reporting.
///
/// Crate-private helper used by every monetary-flow kernel in place of the
/// raw `+` operator. Wraps [`Decimal::checked_add`] and converts `None`
/// into a [`DecimalError::Overflow`] tagged with the static `op` string
/// passed in by the call-site.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the result is outside the
/// representable `Decimal` range.
#[inline]
pub(crate) fn d_add(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    lhs.checked_add(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))
}

/// Checked sum over a slice of `Decimal` values.
///
/// Crate-private helper used by multi-leg strategy P&L aggregations
/// (spreads, condors, butterflies) where each leg already returns a
/// `Result<Decimal, _>` and the sum has to preserve the checked
/// semantics of the individual legs. Returns `Decimal::ZERO` on an
/// empty slice.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] on the first accumulation that
/// exceeds the representable `Decimal` range, tagged with the
/// supplied `op` string so the caller can be identified without a
/// stack trace.
#[inline]
pub(crate) fn d_sum(values: &[Decimal], op: &'static str) -> Result<Decimal, DecimalError> {
    let mut acc = Decimal::ZERO;
    for v in values {
        acc = acc
            .checked_add(*v)
            .ok_or_else(|| DecimalError::overflow(op, acc, *v))?;
    }
    Ok(acc)
}

/// Checked `Decimal` subtraction with operand-preserving overflow reporting.
///
/// Crate-private helper used by every monetary-flow kernel in place of the
/// raw `-` operator. Wraps [`Decimal::checked_sub`] and converts `None`
/// into a [`DecimalError::Overflow`] tagged with the static `op` string
/// passed in by the call-site.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the result is outside the
/// representable `Decimal` range.
#[inline]
pub(crate) fn d_sub(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    lhs.checked_sub(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))
}

/// Checked `Decimal` multiplication with operand-preserving overflow reporting.
///
/// Crate-private helper used by every monetary-flow kernel in place of the
/// raw `*` operator. Wraps [`Decimal::checked_mul`] and converts `None`
/// into a [`DecimalError::Overflow`] tagged with the static `op` string
/// passed in by the call-site.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the result is outside the
/// representable `Decimal` range.
#[inline]
pub(crate) fn d_mul(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))
}

/// Checked `Decimal` division with banker's rounding at scale 28.
///
/// Crate-private helper used by every monetary-flow kernel in place of the
/// raw `/` operator. Performs [`Decimal::checked_div`] then re-rounds the
/// quotient with [`RoundingStrategy::MidpointNearestEven`] to the default
/// [`DIV_DEFAULT_SCALE`]. This policy is applied uniformly across the
/// crate so long-chain divisions do not silently accumulate bias.
///
/// # Errors
///
/// - Returns [`DecimalError::Overflow`] when the quotient is outside the
///   representable `Decimal` range.
/// - Returns [`DecimalError::ArithmeticError`] when `rhs` is zero.
#[inline]
pub(crate) fn d_div(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    if rhs.is_zero() {
        return Err(DecimalError::arithmetic_error(op, "division by zero"));
    }
    let raw = lhs
        .checked_div(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))?;
    Ok(raw.round_dp_with_strategy(DIV_DEFAULT_SCALE, RoundingStrategy::MidpointNearestEven))
}

/// Converts a Decimal value to f64 without error checking.
///
/// This macro converts a Decimal type to an f64 floating-point value.
/// It's an "unchecked" version that doesn't handle potential conversion errors.
///
/// # Parameters
/// * `$val` - A Decimal value to be converted to f64
///
/// # Example
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::d2fu;
/// let decimal_value = dec!(10.5);
/// let float_value = d2fu!(decimal_value);
/// ```
#[macro_export]
macro_rules! d2fu {
    ($val:expr) => {
        $crate::model::decimal::decimal_to_f64($val)
    };
}

/// Converts a Decimal value to f64 with error propagation.
///
/// This macro converts a Decimal type to an f64 floating-point value.
/// It propagates any errors that might occur during conversion using the `?` operator.
///
/// # Parameters
/// * `$val` - A Decimal value to be converted to f64
///
#[macro_export]
macro_rules! d2f {
    ($val:expr) => {
        $crate::model::decimal::decimal_to_f64($val)?
    };
}

/// Builds a `NonZeroUsize` from a literal or constant expression.
///
/// Ergonomic shorthand for the `NonZeroUsize::new(N).expect(..)` pattern
/// at call sites that know the value is non-zero by construction (tests,
/// examples, benchmarks). Use this whenever passing literal step or
/// simulation counts to one of the public pricing kernels migrated
/// in #337.
///
/// # Panics
///
/// Panics with a descriptive message if `$val` evaluates to zero. For
/// runtime values coming from JSON, CLI, or external APIs prefer
/// `NonZeroUsize::new(x).ok_or_else(..)` at the boundary instead.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::nz;
/// use std::num::NonZeroUsize;
///
/// let steps = nz!(100);
/// assert_eq!(steps.get(), 100);
/// ```
#[macro_export]
macro_rules! nz {
    ($val:expr) => {{
        ::std::num::NonZeroUsize::new($val)
            .unwrap_or_else(|| panic!("nz!({}) must be non-zero", stringify!($val)))
    }};
}

/// Converts an f64 value to Decimal without error checking.
///
/// This macro converts an f64 floating-point value to a Decimal type.
/// It's an "unchecked" version that doesn't handle potential conversion errors.
///
/// # Parameters
/// * `$val` - An f64 value to be converted to Decimal
///
/// # Example
/// ```rust
/// use optionstratlib::f2du;
/// let float_value = 10.5;
/// let decimal_value = f2du!(float_value);
/// ```
#[macro_export]
macro_rules! f2du {
    ($val:expr) => {
        $crate::model::decimal::f64_to_decimal($val)
    };
}

/// Converts an f64 value to Decimal with error propagation.
///
/// This macro converts an f64 floating-point value to a Decimal type.
/// It propagates any errors that might occur during conversion using the `?` operator.
///
/// # Parameters
/// * `$val` - An f64 value to be converted to Decimal
///
#[macro_export]
macro_rules! f2d {
    ($val:expr) => {
        $crate::model::decimal::f64_to_decimal($val)?
    };
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_f64_to_decimal_valid() {
        let value = 42.42;
        let result = f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("42.42").unwrap());
    }

    #[test]
    fn test_f64_to_decimal_zero() {
        let value = 0.0;
        let result = f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("0").unwrap());
    }

    #[test]
    fn test_decimal_to_f64_valid() {
        let decimal = Decimal::from_str("42.42").unwrap();
        let result = decimal_to_f64(decimal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.42);
    }

    #[test]
    fn test_decimal_to_f64_zero() {
        let decimal = Decimal::from_str("0").unwrap();
        let result = decimal_to_f64(decimal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }
}

#[cfg(test)]
mod tests_random_generation {
    use super::*;
    use approx::assert_relative_eq;
    use rand::distr::Distribution;
    use std::collections::HashMap;

    #[test]
    fn test_normal_sample_returns() {
        // Run the function multiple times to ensure it always returns a positive value
        for _ in 0..1000 {
            let sample = decimal_normal_sample();
            assert!(sample <= Decimal::TEN);
            assert!(sample >= -Decimal::TEN);
        }
    }

    #[test]
    fn test_normal_sample_distribution() {
        // Generate a large number of samples to check distribution characteristics
        const NUM_SAMPLES: usize = 10000;
        let mut samples = Vec::with_capacity(NUM_SAMPLES);

        for _ in 0..NUM_SAMPLES {
            samples.push(decimal_normal_sample().to_f64().unwrap());
        }

        // Calculate mean and standard deviation
        let sum: f64 = samples.iter().sum();
        let mean = sum / NUM_SAMPLES as f64;

        let variance_sum: f64 = samples.iter().map(|&x| (x - mean).powi(2)).sum();
        let std_dev = (variance_sum / NUM_SAMPLES as f64).sqrt();

        // Check if the distribution approximately matches a standard normal
        // Note: These tests use wide tolerances since we're working with random samples
        assert_relative_eq!(mean, 0.0, epsilon = 0.04);
        assert_relative_eq!(std_dev, 1.0, epsilon = 0.03);
    }

    #[test]
    fn test_normal_distribution_transformation() {
        let mut t_rng = rand::rng();
        let normal = Normal::new(-1.0, 0.5).unwrap(); // Deliberately using a distribution with negative mean

        // Count occurrences of values after transformation
        let mut value_counts: HashMap<i32, usize> = HashMap::new();
        const SAMPLES: usize = 5000;

        for _ in 0..SAMPLES {
            let raw_sample = normal.sample(&mut t_rng);
            let positive_sample = raw_sample.to_f64().unwrap();

            // Bucket values to the nearest integer for counting
            let bucket = (positive_sample.round() as i32).max(0);
            *value_counts.entry(bucket).or_insert(0) += 1;
        }

        // Verify that zero values appear frequently (due to negative values being transformed)
        assert!(value_counts.get(&0).unwrap_or(&0) > &(SAMPLES / 10));

        // Verify that we have a range of positive values
        let max_bucket = value_counts.keys().max().unwrap_or(&0);
        assert!(*max_bucket > 0);
    }

    #[test]
    fn test_normal_sample_consistency() {
        // This test ensures that multiple calls in sequence produce different values
        let sample1 = decimal_normal_sample();
        let sample2 = decimal_normal_sample();
        let sample3 = decimal_normal_sample();

        // It's statistically extremely unlikely to get the same value three times in a row
        // This verifies that the RNG is properly producing different values
        assert!(sample1 != sample2 || sample2 != sample3);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod checked_helpers_tests {
    use super::*;

    #[test]
    fn d_add_happy_path() {
        let result = d_add(dec!(1.25), dec!(2.50), "test::add");
        assert_eq!(result.unwrap(), dec!(3.75));
    }

    #[test]
    fn d_add_overflow_on_max_plus_max() {
        let err = d_add(Decimal::MAX, Decimal::MAX, "test::add").unwrap_err();
        match err {
            DecimalError::Overflow { operation, .. } => assert_eq!(operation, "test::add"),
            other => panic!("expected Overflow, got {other:?}"),
        }
    }

    #[test]
    fn d_sub_happy_path() {
        let result = d_sub(dec!(10), dec!(3.5), "test::sub");
        assert_eq!(result.unwrap(), dec!(6.5));
    }

    #[test]
    fn d_sub_overflow_on_min_minus_max() {
        let err = d_sub(Decimal::MIN, Decimal::MAX, "test::sub").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::sub")
        );
    }

    #[test]
    fn d_mul_happy_path() {
        let result = d_mul(dec!(2.5), dec!(4), "test::mul");
        assert_eq!(result.unwrap(), dec!(10.0));
    }

    #[test]
    fn d_mul_overflow_on_max_times_two() {
        let err = d_mul(Decimal::MAX, dec!(2), "test::mul").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::mul")
        );
    }

    #[test]
    fn d_div_happy_path_exact() {
        let result = d_div(dec!(10), dec!(4), "test::div");
        assert_eq!(result.unwrap(), dec!(2.5));
    }

    #[test]
    fn d_div_applies_banker_rounding() {
        // Divide by three is recurring and must round half-to-even at the default scale.
        let result = d_div(dec!(1), dec!(3), "test::div").unwrap();
        // `1 / 3` at scale 28 under banker's rounding produces the canonical
        // `0.3333333333333333333333333333` (scale 28). Any other trailing digit
        // would signal the rounding strategy drifted.
        assert_eq!(result, dec!(0.3333333333333333333333333333));
    }

    #[test]
    fn d_div_zero_denominator_returns_arithmetic_error() {
        let err = d_div(dec!(1), Decimal::ZERO, "test::div").unwrap_err();
        assert!(matches!(err, DecimalError::ArithmeticError { .. }));
    }

    #[test]
    fn d_div_tag_is_preserved_on_overflow() {
        // `Decimal::MIN / 0.5` overflows because the quotient is 2 * MIN.
        let err = d_div(Decimal::MIN, dec!(0.5), "test::div").unwrap_err();
        match err {
            DecimalError::Overflow { operation, .. } => assert_eq!(operation, "test::div"),
            other => panic!("expected Overflow, got {other:?}"),
        }
    }

    #[test]
    fn d_sum_empty_returns_zero() {
        assert_eq!(d_sum(&[], "test::sum").unwrap(), Decimal::ZERO);
    }

    #[test]
    fn d_sum_happy_path() {
        let result = d_sum(&[dec!(1.5), dec!(2.25), dec!(-0.75), dec!(10)], "test::sum");
        assert_eq!(result.unwrap(), dec!(13));
    }

    #[test]
    fn d_sum_overflow_returns_tagged_error() {
        let err = d_sum(&[Decimal::MAX, Decimal::MAX], "test::sum").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::sum")
        );
    }
}
