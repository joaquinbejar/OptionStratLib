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
