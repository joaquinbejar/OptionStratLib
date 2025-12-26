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
use rust_decimal::{Decimal, MathematicalOps};
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
        if self.is_empty() {
            return Decimal::ZERO;
        }
        let mean = self.mean();
        let variance: Decimal = self.iter().map(|x| (x - mean).powd(Decimal::TWO)).sum();
        (variance / Decimal::from(self.len() - 1)).sqrt().unwrap()
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
pub fn decimal_normal_sample() -> Decimal {
    let mut t_rng = rand::rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    Decimal::from_f64(normal.sample(&mut t_rng)).unwrap()
}

impl HasX for Decimal {
    fn get_x(&self) -> Decimal {
        *self
    }
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
