/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
use crate::pos;
use chrono::{NaiveDateTime, TimeZone, Utc};

#[allow(dead_code)]
pub fn positive_f64_to_f64(vec: Vec<PositiveF64>) -> Vec<f64> {
    vec.into_iter().map(|pos_f64| pos_f64.value()).collect()
}

#[allow(dead_code)]
pub(crate) fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: PositiveF64,
    quantity: PositiveF64,
    strike_price: PositiveF64,
    volatility: f64,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::Days(30.0),
        volatility,
        quantity,
        underlying_price,
        0.05,
        option_style,
        0.01,
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn create_sample_option_with_date(
    option_style: OptionStyle,
    side: Side,
    underlying_price: PositiveF64,
    quantity: PositiveF64,
    strike_price: PositiveF64,
    volatility: f64,
    naive_date: NaiveDateTime,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
        volatility,
        quantity,
        underlying_price,
        0.05,
        option_style,
        0.01,
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn create_sample_option_simplest(option_style: OptionStyle, side: Side) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        pos!(100.0),
        ExpirationDate::Days(30.0),
        0.2,
        pos!(1.0),
        pos!(100.0),
        0.05,
        option_style,
        0.01,
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn create_sample_option_simplest_strike(
    side: Side,
    option_style: OptionStyle,
    strike: PositiveF64,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike,
        ExpirationDate::Days(30.0),
        0.2,
        pos!(1.0),
        pos!(100.0),
        0.05,
        option_style,
        0.01,
        None,
    )
}

/// Computes the mean and standard deviation of a vector containing `PositiveF64` values.
///
/// # Arguments
///
/// * `vec` - A `Vec<PositiveF64>` containing the data for which the mean and standard deviation
///           are to be calculated.
///
/// # Returns
///
/// A tuple containing:
/// - `PositiveF64` - The mean of the provided vector.
/// - `PositiveF64` - The standard deviation of the provided vector.
///
/// # Example
///
/// ```rust
/// use optionstratlib::model::types::PositiveF64;
/// use optionstratlib::model::utils::mean_and_std;
///
/// let data = vec![PositiveF64::new(2.0).unwrap(), PositiveF64::new(4.0).unwrap(), PositiveF64::new(4.0).unwrap(), PositiveF64::new(4.0).unwrap(), PositiveF64::new(5.0).unwrap(), PositiveF64::new(5.0).unwrap(), PositiveF64::new(7.0).unwrap(), PositiveF64::new(9.0).unwrap()];
/// let (mean, std) = mean_and_std(data);
///
/// assert_eq!(mean.value(), 5.0);
/// assert_eq!(std.value(), (4.0_f64.sqrt()));
/// ```
///
/// # Details
///
/// - The mean is computed by summing the `PositiveF64` values and dividing by the count of elements.
/// - The standard deviation is derived from the variance, which is the average of the squared differences
///   from the mean. The variance is then converted into standard deviation by taking its square root.
/// - This function assumes the vector is non-empty and filled with valid `PositiveF64` values.
///
/// Note: The `PositiveF64` type and associated operations are defined in the `crate::model::types` module.
pub fn mean_and_std(vec: Vec<PositiveF64>) -> (PositiveF64, PositiveF64) {
    let mean = vec.iter().sum::<PositiveF64>() / vec.len() as f64;
    let variance = vec
        .iter()
        .map(|x| pos!((x.value() - mean.value()).powi(2)))
        .sum::<PositiveF64>()
        / vec.len() as f64;
    let std = variance.value().sqrt();
    (mean, pos!(std))
}

#[cfg(test)]
mod tests_positive_f64_to_f64 {
    use super::*;

    #[test]
    fn test_positive_f64_to_f64_non_empty() {
        let positive_vec = vec![
            PositiveF64::new(10.0).unwrap(),
            PositiveF64::new(20.0).unwrap(),
            PositiveF64::new(30.0).unwrap(),
        ];

        let f64_vec = positive_f64_to_f64(positive_vec);

        assert_eq!(f64_vec, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_positive_f64_to_f64_single_element() {
        let positive_vec = vec![PositiveF64::new(42.0).unwrap()];

        let f64_vec = positive_f64_to_f64(positive_vec);

        assert_eq!(f64_vec, vec![42.0]);
    }

    #[test]
    #[should_panic]
    fn test_positive_f64_to_f64_invalid_positivef64() {
        // Esto provocará un panic ya que estamos intentando crear un `PositiveF64` con un valor negativo
        PositiveF64::new(-10.0).unwrap();
    }
}

#[cfg(test)]
mod tests_mean_and_std {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use crate::model::types::PZERO;

    #[test]
    fn test_basic_mean_and_std() {
        let values = vec![pos!(2.0), pos!(4.0), pos!(4.0), pos!(4.0), pos!(5.0), pos!(5.0), pos!(7.0), pos!(9.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(std.value(), 2.0, epsilon = 0.0001);
    }

    #[test]
    fn test_identical_values() {
        let values = vec![pos!(5.0), pos!(5.0), pos!(5.0), pos!(5.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(std.value(), 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_single_value() {
        let values = vec![pos!(3.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(std.value(), 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_small_numbers() {
        let values = vec![pos!(0.1), pos!(0.2), pos!(0.3)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 0.2, epsilon = 0.0001);
        assert_relative_eq!(std.value(), 0.08164966, epsilon = 0.0001);
    }

    #[test]
    fn test_large_numbers() {
        let values = vec![pos!(1000.0), pos!(2000.0), pos!(3000.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 2000.0, epsilon = 0.0001);
        assert_relative_eq!(std.value(), 816.4966, epsilon = 0.1);
    }

    #[test]
    fn test_mixed_range() {
        let values = vec![pos!(0.5), pos!(5.0), pos!(50.0), pos!(500.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 138.875, epsilon = 0.001);
        assert_relative_eq!(std.value(), 209.392, epsilon = 0.001);
    }

    #[test]
    #[should_panic]
    fn test_empty_vector() {
        let values: Vec<PositiveF64> = vec![];
        let _ = mean_and_std(values);
    }

    #[test]
    fn test_symmetric_distribution() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0), pos!(5.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(std.value(), std::f64::consts::SQRT_2, epsilon = 0.0001);
    }

    #[test]
    fn test_result_is_positive() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let (mean, std) = mean_and_std(values);

        assert!(mean > PZERO);
        assert!(std > PZERO);
    }

    #[test]
    fn test_precision() {
        let values = vec![
            pos!(1.23456789),
            pos!(2.34567890),
            pos!(3.45678901),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 2.34567860, epsilon = 0.00000001);
        assert_relative_eq!(std.value(), 0.90721797, epsilon = 0.00000001);
    }

    #[test]
    fn test_precision_bis() {
        let values = vec![
            pos!(0.123456789),
            pos!(0.134567890),
            pos!(0.145678901),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.value(), 0.13456785, epsilon = 0.00000001);
        assert_relative_eq!(std.value(), 0.00907213, epsilon = 0.00000001);
    }
}