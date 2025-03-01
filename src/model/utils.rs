/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::model::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::{Options, Positive, pos};
use chrono::{NaiveDateTime, TimeZone, Utc};
use rust_decimal_macros::dec;

pub fn positive_f64_to_f64(vec: Vec<Positive>) -> Vec<f64> {
    vec.into_iter().map(|pos_f64| pos_f64.to_f64()).collect()
}

#[allow(dead_code)]
pub(crate) fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::Days(pos!(30.0)),
        volatility,
        quantity,
        underlying_price,
        dec!(0.05),
        option_style,
        pos!(0.01),
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn create_sample_position(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    implied_volatility: Positive,
) -> Position {
    Position {
        option: Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "AAPL".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(pos!(30.0)),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.05),
            option_style,
            dividend_yield: pos!(0.01),
            exotic_params: None,
        },
        premium: pos!(5.0),
        date: Utc::now(),
        open_fee: pos!(0.5),
        close_fee: pos!(0.5),
    }
}

#[allow(dead_code)]
pub(crate) fn create_sample_option_with_date(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
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
        dec!(0.05),
        option_style,
        pos!(0.01),
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
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.2),
        pos!(1.0),
        pos!(100.0),
        dec!(0.05),
        option_style,
        pos!(0.01),
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn create_sample_option_simplest_strike(
    side: Side,
    option_style: OptionStyle,
    strike: Positive,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike,
        ExpirationDate::Days(pos!(30.0)),
        pos!(0.2),
        pos!(1.0),
        pos!(100.0),
        dec!(0.05),
        option_style,
        pos!(0.01),
        None,
    )
}

/// Computes the mean and standard deviation of a vector containing `Positive` values.
///
/// # Arguments
///
/// * `vec` - A `Vec<Positive>` containing the data for which the mean and standard deviation
///           are to be calculated.
///
/// # Returns
///
/// A tuple containing:
/// - `Positive` - The mean of the provided vector.
/// - `Positive` - The standard deviation of the provided vector.
///
/// # Example
///
/// ```rust
/// use optionstratlib::Positive;
/// use optionstratlib::model::utils::mean_and_std;
///
/// let data = vec![Positive::new(2.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(5.0).unwrap(), Positive::new(5.0).unwrap(), Positive::new(7.0).unwrap(), Positive::new(9.0).unwrap()];
/// let (mean, std) = mean_and_std(data);
///
/// assert_eq!(mean.to_f64(), 5.0);
/// assert_eq!(std.to_f64(), 4.0_f64.sqrt());
/// ```
///
/// # Details
///
/// - The mean is computed by summing the `Positive` values and dividing by the count of elements.
/// - The standard deviation is derived from the variance, which is the average of the squared differences
///   from the mean. The variance is then converted into standard deviation by taking its square root.
/// - This function assumes the vector is non-empty and filled with valid `Positive` values.
///
/// Note: The `Positive` type and associated operations are defined in the `crate::model::types` module.
pub fn mean_and_std(vec: Vec<Positive>) -> (Positive, Positive) {
    let mean = vec.iter().sum::<Positive>() / vec.len() as f64;
    let variance = vec
        .iter()
        .map(|x| pos!((x.to_f64() - mean.to_f64()).powi(2)))
        .sum::<Positive>()
        / vec.len() as f64;
    let std = variance.to_f64().sqrt();
    (mean, pos!(std))
}

#[cfg(test)]
mod tests_positive_f64_to_f64 {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_positive_f64_to_f64_non_empty() {
        let positive_vec = vec![
            Positive::new(10.0).unwrap(),
            Positive::new(20.0).unwrap(),
            Positive::new(30.0).unwrap(),
        ];

        let f64_vec = positive_f64_to_f64(positive_vec);

        assert_eq!(f64_vec, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_positive_f64_to_f64_single_element() {
        let positive_vec = vec![Positive::new(42.0).unwrap()];

        let f64_vec = positive_f64_to_f64(positive_vec);

        assert_eq!(f64_vec, vec![42.0]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic]
    fn test_positive_f64_to_f64_invalid_positivef64() {
        Positive::new(-10.0).unwrap();
    }
}

#[cfg(test)]
mod tests_mean_and_std {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_basic_mean_and_std() {
        let values = vec![
            pos!(2.0),
            pos!(4.0),
            pos!(4.0),
            pos!(4.0),
            pos!(5.0),
            pos!(5.0),
            pos!(7.0),
            pos!(9.0),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 2.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_identical_values() {
        let values = vec![pos!(5.0), pos!(5.0), pos!(5.0), pos!(5.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 0.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_single_value() {
        let values = vec![pos!(3.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 0.0, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_small_numbers() {
        let values = vec![pos!(0.1), pos!(0.2), pos!(0.3)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 0.2, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 0.08164966, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_large_numbers() {
        let values = vec![pos!(1000.0), pos!(2000.0), pos!(3000.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 2000.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 816.4966, epsilon = 0.1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_mixed_range() {
        let values = vec![pos!(0.5), pos!(5.0), pos!(50.0), pos!(500.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 138.875, epsilon = 0.001);
        assert_relative_eq!(std.to_f64(), 209.392, epsilon = 0.001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic]
    fn test_empty_vector() {
        let values: Vec<Positive> = vec![];
        let _ = mean_and_std(values);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_symmetric_distribution() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0), pos!(5.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), std::f64::consts::SQRT_2, epsilon = 0.0001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_result_is_positive() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let (mean, std) = mean_and_std(values);

        assert!(mean > Positive::ZERO);
        assert!(std > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_precision() {
        let values = vec![pos!(1.23456789), pos!(2.34567890), pos!(3.45678901)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 2.34567860, epsilon = 0.00000001);
        assert_relative_eq!(std.to_f64(), 0.90721797, epsilon = 0.00000001);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_precision_bis() {
        let values = vec![pos!(0.123456789), pos!(0.134567890), pos!(0.145678901)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 0.13456786, epsilon = 0.00000001);
        assert_relative_eq!(std.to_f64(), 0.00907213, epsilon = 0.00000001);
    }
}
