/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 30/12/24
 ******************************************************************************/

use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::str::FromStr;
use approx::{AbsDiffEq, RelativeEq};
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::constants::EPSILON;

#[derive(PartialEq, Clone, Copy)]
pub struct Positive(Decimal);

#[macro_export]
macro_rules! pos {
    ($val:expr) => {
        Positive::new($val).unwrap()
    };
}

#[macro_export]
macro_rules! spos {
    ($val:expr) => {
        Some(Positive::new($val).unwrap())
    };
}

#[macro_export]
macro_rules! f2p {
    ($val:expr) => {
        Positive::new(Decimal::from_f64($val).unwrap_or(Decimal::ZERO)).unwrap_or(Positive::ZERO)
    };
}

#[macro_export]
macro_rules! sf2p {
    ($val:expr) => {
        Some(Positive::new(dec!($val)).unwrap())
    };
}


impl Positive {
    
    pub const ZERO: Positive = Positive(Decimal::ZERO);
    
    pub const ONE: Positive = Positive(Decimal::ONE);

    pub const TWO: Positive = Positive(Decimal::TWO);
    
    pub const INFINITY: Positive = Positive(Decimal::MAX);
    
    pub const TEN : Positive = Positive(Decimal::from(10));
    
    pub const HUNDRED : Positive = Positive(Decimal::from(100));
    
    pub const THOUSAND : Positive = Positive(Decimal::from(1000));
    
    pub const MILLION : Positive = Positive(Decimal::from(1000000));
    
    pub  const PI: Positive = Positive(Decimal::PI);
    
    
    pub fn new(value: Decimal) -> Result<Self, String> {
        if value >= Decimal::ZERO {
            Ok(Positive(value))
        } else {
            Err(format!("Positive value must be positive, got {}", value))
        }
    }

    pub fn value(&self) -> Decimal {
        self.0
    }

    pub fn to_dec(&self) -> Decimal {
        self.0
    }

    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap()
    }

    pub fn max(self, other: Positive) -> Positive {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }

    pub fn min(self, other: Positive) -> Positive {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }

    pub fn floor(&self) -> Positive {
        Positive(self.0.floor())
    }
}

impl From<Positive> for Decimal {
    fn from(pos_decimal: Positive) -> Self {
        pos_decimal.0
    }
}

impl From<Positive> for u64 {
    fn from(pos_u64: Positive) -> Self {
        pos_u64.0.to_u64().unwrap()
    }
}

impl PartialEq<Decimal> for Positive {
    fn eq(&self, other: &Decimal) -> bool {
        self.0 == *other
    }
}

impl PartialEq<f64> for Positive {
    fn eq(&self, other: &f64) -> bool {
        self.to_f64() == *other
    }
}

impl fmt::Display for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "{:.1$}", self.0, precision)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl fmt::Debug for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "{:.1$}", self.0, precision)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

impl Serialize for Positive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_Decimal(self.0)
    }
}

impl<'de> Deserialize<'de> for Positive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Decimal::deserialize(deserializer);
        Positive::new(value).map_err(serde::de::Error::custom)
    }
}

impl Add for Positive {
    type Output = Positive;

    fn add(self, other: Positive) -> Positive {
        Positive(self.0 + other.0)
    }
}

impl Sub for Positive {
    type Output = Positive;

    fn sub(self, rhs: Self) -> Self::Output {
        Positive(self.0 - rhs.0)
    }
}

impl Div for Positive {
    type Output = Positive;

    fn div(self, other: Positive) -> Positive {
        Positive(self.0 / other.0)
    }
}

impl Add<Decimal> for Positive {
    type Output = Positive;

    fn add(self, rhs: Decimal) -> Positive {
        Positive(self.0 + rhs)
    }
}

impl Sub<Decimal> for Positive {
    type Output = Positive;

    fn sub(self, rhs: Decimal) -> Positive {
        Positive(self.0 - rhs)
    }
}

impl AddAssign for Positive {
    fn add_assign(&mut self, other: Positive) {
        self.0 += other.0;
    }
}

impl AddAssign<Decimal> for Positive {
    fn add_assign(&mut self, rhs: Decimal) {
        self.0 += rhs;
    }
}

impl MulAssign<Decimal> for Positive {
    fn mul_assign(&mut self, rhs: Decimal) {
        self.0 *= rhs;
    }
}

impl Div<Decimal> for Positive {
    type Output = Positive;

    fn div(self, rhs: Decimal) -> Positive {
        Positive(self.0 / rhs)
    }
}

impl PartialOrd for Positive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn ge(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

impl Eq for Positive {}

impl Ord for Positive {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Neg for Positive {
    type Output = Self;

    fn neg(self) -> Self::Output {
        panic!("Cannot negate a Positive value!");
    }
}

impl Mul for Positive {
    type Output = Positive;

    fn mul(self, other: Positive) -> Positive {
        Positive(self.0 * other.0)
    }
}

impl Mul<Decimal> for Positive {
    type Output = Positive;

    fn mul(self, rhs: Decimal) -> Positive {
        Positive(self.0 * rhs)
    }
}

impl Mul<Decimal> for Positive {
    type Output = Positive;

    fn mul(self, rhs: Decimal) -> Positive {
        Positive(self.0 * rhs)
    }
}

impl Default for Positive {
    fn default() -> Self {
        Positive::ZERO
    }
}

impl FromStr for Positive {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Decimal>() {
            Ok(value) if value > Decimal::ZERO => Ok(Positive(value)),
            Ok(value) => Err(format!("Value must be positive, got {}", value)),
            Err(e) => Err(format!("Failed to parse as Decimal: {}", e)),
        }
    }
}

impl From<Decimal> for Positive {
    fn from(value: Decimal) -> Self {
        Positive::new(value).expect("Value must be positive")
    }
}

impl From<Decimal> for Positive {
    fn from(value: Decimal) -> Self {
        Positive::new(value).expect("Value must be positive")
    }
}

impl AbsDiffEq for Positive {
    type Epsilon = Decimal;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        (self.0 - other.0).abs() <= epsilon
    }
}

impl RelativeEq for Positive {
    fn default_max_relative() -> Self::Epsilon {
        EPSILON * Decimal::from(100)
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        // Calculate the absolute difference
        let abs_diff = (self.0 - other.0).abs();

        // Use relative tolerance: max_relative * max(|self.0|, |other.0|)
        let largest = self.0.abs().max(other.0.abs());

        abs_diff <= epsilon || abs_diff <= max_relative * largest
    }
}

impl Sum for Positive {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sum = iter.fold(Decimal::ZERO, |acc, x| acc + x.value());
        Positive::new(sum).unwrap_or(Positive::ZERO)
    }
}

impl<'a> Sum<&'a Positive> for Positive {
    fn sum<I: Iterator<Item = &'a Positive>>(iter: I) -> Self {
        let sum = iter.fold(Decimal::ZERO, |acc, x| acc + x.value());
        Positive::new(sum).unwrap_or(Positive::ZERO)
    }
}

impl AddAssign<Positive> for Decimal {
    fn add_assign(&mut self, rhs: Positive) {
        *self += rhs.0;
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

impl Mul<Positive> for Decimal {
    type Output = Decimal;

    fn mul(self, rhs: Positive) -> Decimal {
        self * rhs.0
    }
}

impl Add<Positive> for Decimal {
    type Output = Decimal;

    fn add(self, rhs: Positive) -> Decimal {
        self + rhs.0
    }
}

#[cfg(test)]
mod tests_positive_decimal {
    use super::*;
    use std::panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_positive_decimal_creation() {
        assert!(Positive::new(Decimal::ZERO).is_ok());
        assert!(Positive::new(Decimal::ONE).is_ok());
        assert!(Positive::new(Decimal::NEGATIVE_ONE).is_err());
    }

    #[test]
    fn test_positive_decimal_value() {
        let pos = Positive::new(dec!(5.0)).unwrap();
        assert_eq!(pos.to_f64(), 5.0);
    }

    #[test]
    fn test_positive_decimal_from() {
        let pos = Positive::new(dec!(3.0)).unwrap();
        let f: Decimal = pos.into();
        assert_eq!(f, dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_eq() {
        let pos = Positive::new(Decimal::TWO).unwrap();
        assert_eq!(pos, dec!(2.0));
        assert_ne!(pos, dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_display() {
        let pos = Positive::new(dec!(4.5)).unwrap();
        assert_eq!(format!("{}", pos), "4.5");
    }

    #[test]
    fn test_positive_decimal_debug() {
        let pos = Positive::new(dec!(4.5)).unwrap();
        assert_eq!(format!("{:?}", pos), "4.5");
    }

    #[test]
    fn test_positive_decimal_display_decimal_fix() {
        let pos = Positive::new(dec!(4.578923789423789)).unwrap();
        assert_eq!(format!("{:.2}", pos), "4.58");
        assert_eq!(format!("{:.3}", pos), "4.579");
        assert_eq!(format!("{:.0}", pos), "5");
    }

    #[test]
    fn test_positive_decimal_add() {
        let a = Positive::new(dec!(2.0)).unwrap();
        let b = Positive::new(dec!(3.0)).unwrap();
        assert_eq!((a + b).value(), dec!(5.0));
    }

    #[test]
    fn test_positive_decimal_div() {
        let a = Positive::new(dec!(6.0)).unwrap();
        let b = Positive::new(dec!(2.0)).unwrap();
        assert_eq!((a / b).value(), dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_div_decimal() {
        let a = Positive::new(dec!(6.0)).unwrap();
        assert_eq!((a / 2.0).value(), 3.0);
    }

    #[test]
    fn test_decimal_mul_positive_decimal() {
        let a = dec!(2.0);
        let b = Positive::new(dec!(3.0)).unwrap();
        assert_eq!(a * b, dec!(6.0));
    }

    #[test]
    fn test_positive_decimal_mul() {
        let a = Positive::new(dec!(2.0)).unwrap();
        let b = Positive::new(dec!(3.0)).unwrap();
        assert_eq!((a * b).value(), dec!(6.0));
    }

    #[test]
    fn test_positive_decimal_mul_decimal() {
        let a = Positive::new(dec!(2.0)).unwrap();
        assert_eq!((a * 3.0).value(), 6.0);
    }

    #[test]
    fn test_positive_decimal_default() {
        assert_eq!(Positive::default().value(), Decimal::ZERO);
    }

    #[test]
    fn test_decimal_div_positive_decimal() {
        let a = dec!(6.0);
        let b = Positive::new(dec!(2.0)).unwrap();
        assert_eq!(a / b, dec!(3.0));
    }

    #[test]
    fn test_pos_macro() {
        assert_eq!(f2p!(5.0).value(), dec!(5.0));
        let result = panic::catch_unwind(|| f2p!(-1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_constants() {
        assert_eq!(Positive::ZERO.value(), Decimal::ZERO);
        assert_eq!(Positive::ONE.value(), Decimal::ONE);
    }
}

#[cfg(test)]
mod tests_positive_decimal_extended {
    use super::*;

    #[test]
    fn test_positive_decimal_ordering() {
        let a = f2p!(1.0);
        let b = f2p!(2.0);
        let c = f2p!(2.0);

        assert!(a < b);
        assert!(b > a);
        assert!(b >= c);
        assert!(b <= c);
    }

    #[test]
    fn test_positive_decimal_add_assign() {
        let mut a = f2p!(1.0);
        let b = f2p!(2.0);
        a += b;
        assert_eq!(a.value(), dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_mul_assign() {
        let mut a = pos!(Decimal::TWO);
        a *= dec!(3.0);
        assert_eq!(a.value(), dec!(6.0));
    }

    #[test]
    fn test_positive_decimal_from_string() {
        assert_eq!(Positive::from_str("1.5").unwrap().value(), dec!(1.5));
        assert!(Positive::from_str("-1.5").is_err());
        assert!(Positive::from_str("invalid").is_err());
    }

    #[test]
    fn test_positive_decimal_max_min() {
        let a = f2p!(1.0);
        let b = f2p!(2.0);
        assert_eq!(a.max(b).value(), dec!(2.0));
        assert_eq!(a.min(b).value(), dec!(1.0));
    }

    #[test]
    fn test_positive_decimal_floor() {
        let a = f2p!(1.7);
        assert_eq!(a.floor().value(), dec!(1.0));
    }

    #[test]
    #[should_panic(expected = "Cannot negate a Positive value!")]
    fn test_positive_decimal_neg() {
        let a = f2p!(1.0);
        let _ = -a;
    }
}

#[cfg(test)]
mod tests_positive_decimal_sum {
    use super::*;

    #[test]
    fn test_sum_owned_values() {
        let values = vec![f2p!(1.0), f2p!(2.0), f2p!(3.0)];
        let sum: Positive = values.into_iter().sum();
        assert_eq!(sum.to_f64(), 6.0);
    }

    #[test]
    fn test_sum_referenced_values() {
        let values = [f2p!(1.0), f2p!(2.0), f2p!(3.0)];
        let sum: Positive = values.iter().sum();
        assert_eq!(sum.to_f64(), 6.0);
    }

    #[test]
    fn test_sum_empty_iterator() {
        let values: Vec<Positive> = vec![];
        let sum: Positive = values.into_iter().sum();
        assert_eq!(sum.to_f64(), 0.0);
    }
}