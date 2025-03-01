/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/
use crate::Positive;
use crate::error::decimal::DecimalError;
use crate::geometrics::HasX;
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

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

pub trait DecimalStats {
    fn mean(&self) -> Decimal;

    fn std_dev(&self) -> Decimal;
}

impl From<Positive> for Decimal {
    fn from(pos: Positive) -> Self {
        pos.0
    }
}

impl From<&Positive> for Decimal {
    fn from(pos: &Positive) -> Self {
        pos.0
    }
}

impl Mul<Positive> for Decimal {
    type Output = Decimal;

    fn mul(self, rhs: Positive) -> Decimal {
        self * rhs.0
    }
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

impl Div<Positive> for Decimal {
    type Output = Decimal;

    fn div(self, rhs: Positive) -> Decimal {
        self / rhs.0
    }
}

impl Sub<Positive> for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: Positive) -> Self::Output {
        self - rhs.0
    }
}

impl Sub<&Positive> for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: &Positive) -> Self::Output {
        self - rhs.0
    }
}

impl Add<Positive> for Decimal {
    type Output = Decimal;

    fn add(self, rhs: Positive) -> Self::Output {
        self + rhs.0
    }
}

impl Add<&Positive> for Decimal {
    type Output = Decimal;

    fn add(self, rhs: &Positive) -> Decimal {
        self + rhs.0
    }
}

impl AddAssign<Positive> for Decimal {
    fn add_assign(&mut self, rhs: Positive) {
        *self += rhs.0;
    }
}

impl AddAssign<&Positive> for Decimal {
    fn add_assign(&mut self, rhs: &Positive) {
        *self += rhs.0;
    }
}

impl MulAssign<Positive> for Decimal {
    fn mul_assign(&mut self, rhs: Positive) {
        *self *= rhs.0;
    }
}

impl MulAssign<&Positive> for Decimal {
    fn mul_assign(&mut self, rhs: &Positive) {
        *self *= rhs.0;
    }
}

impl PartialEq<Positive> for Decimal {
    fn eq(&self, other: &Positive) -> bool {
        *self == other.0
    }
}

pub fn decimal_to_f64(value: Decimal) -> Result<f64, DecimalError> {
    value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: format!("Decimal: {}", value),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })
}

pub fn f64_to_decimal(value: f64) -> Result<Decimal, DecimalError> {
    Decimal::from_f64(value).ok_or(DecimalError::ConversionError {
        from_type: format!("f64: {}", value),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

impl HasX for Decimal {
    fn get_x(&self) -> Decimal {
        *self
    }
}

#[macro_export]
macro_rules! d2fu {
    ($val:expr) => {
        $crate::model::decimal::decimal_to_f64($val)
    };
}

#[macro_export]
macro_rules! d2f {
    ($val:expr) => {
        $crate::model::decimal::decimal_to_f64($val)?
    };
}

#[macro_export]
macro_rules! f2du {
    ($val:expr) => {
        $crate::model::decimal::f64_to_decimal($val)
    };
}

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_f64_to_decimal_valid() {
        let value = 42.42;
        let result = f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("42.42").unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_f64_to_decimal_zero() {
        let value = 0.0;
        let result = f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("0").unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_decimal_to_f64_valid() {
        let decimal = Decimal::from_str("42.42").unwrap();
        let result = decimal_to_f64(decimal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.42);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_decimal_to_f64_zero() {
        let decimal = Decimal::from_str("0").unwrap();
        let result = decimal_to_f64(decimal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }
}
