/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/
use crate::error::decimal::DecimalError;
use crate::model::types::PositiveF64;
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

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

pub(crate) fn f64_to_decimal(value: f64) -> Result<Decimal, DecimalError> {
    if value == f64::NEG_INFINITY {
        return Ok(Decimal::MIN);
    }
    if value == f64::INFINITY {
        return Ok(Decimal::MAX);
    }
    let result = Decimal::from_f64(value);
    match result {
        Some(decimal) => Ok(decimal),
        None => Err(DecimalError::ConversionError {
            from_type: format!("f64: {}", value),
            to_type: "Decimal".to_string(),
            reason: "Failed to convert f64 to Decimal".to_string(),
        }),
    }
}

pub(crate) fn positive_f64_to_decimal(value: PositiveF64) -> Result<Decimal, DecimalError> {
    let result = Decimal::from_f64(value.value());
    match result {
        Some(decimal) => Ok(decimal),
        None => Err(DecimalError::ConversionError {
            from_type: "PositiveF64".to_string(),
            to_type: "Decimal".to_string(),
            reason: "Failed to convert f64 to Decimal".to_string(),
        }),
    }
}

pub(crate) fn decimal_to_f64(value: Decimal) -> Result<f64, DecimalError> {
    value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: "Decimal".to_string(),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;
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

    #[test]
    fn test_positive_f64_to_decimal() {
        let value = pos!(42.42);
        let result = positive_f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("42.42").unwrap());
    }
}
