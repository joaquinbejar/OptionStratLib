use num_traits::{abs, ConstZero, FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::ops::{Add, Div, Mul, Sub};

pub trait FloatLike:
    Sized
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    fn infinity() -> Self;
    fn neg_infinity() -> Self;
    fn zero() -> Self;
    fn two() -> Self;
    fn powf(self, exp: Self) -> Self;
    fn pow_two(self) -> Self;
    fn ln(self) -> Self;
    fn sqrt(self) -> Self;
    fn abs(self) -> Self;
    fn is_infinite(&self) -> bool;
}

impl FloatLike for Decimal {
    fn infinity() -> Self {
        Decimal::MAX
    }
    fn neg_infinity() -> Self {
        Decimal::MIN
    }
    fn zero() -> Self {
        Decimal::ZERO
    }

    fn two() -> Self {
        Decimal::from_f64(2.0).unwrap()
    }

    fn powf(self, exp: Self) -> Self {
        let base_f64 = self.to_f64().unwrap();
        let exp_f64 = exp.to_f64().unwrap();
        let result = base_f64.powf(exp_f64);
        Decimal::from_f64(result).unwrap()
    }

    fn pow_two(self) -> Self {
        self.powf(Decimal::two())
    }

    fn ln(self) -> Self {
        let value_f64 = self.to_f64().unwrap();
        let result = value_f64.ln();
        Decimal::from_f64(result).unwrap()
    }

    fn sqrt(self) -> Self {
        let value_f64 = self.to_f64().unwrap();
        let result = value_f64.sqrt();
        Decimal::from_f64(result).unwrap()
    }

    fn abs(self) -> Self {
        abs(self)
    }

    fn is_infinite(&self) -> bool {
        let max_threshold = Decimal::from_f64(1e24).unwrap();
        self >= &max_threshold || self <= &-max_threshold
    }
}

impl FloatLike for f64 {
    fn infinity() -> Self {
        f64::INFINITY
    }
    fn neg_infinity() -> Self {
        f64::NEG_INFINITY
    }
    fn zero() -> Self {
        f64::ZERO
    }

    fn two() -> Self {
        2.0
    }

    fn powf(self, exp: Self) -> Self {
        f64::powf(self, exp)
    }

    fn pow_two(self) -> Self {
        f64::powf(self, 2.0)
    }

    fn ln(self) -> Self {
        f64::ln(self)
    }

    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }

    fn abs(self) -> Self {
        f64::abs(self)
    }

    fn is_infinite(&self) -> bool {
        f64::is_infinite(*self)
    }
}

#[cfg(test)]
mod tests_float_like {
    use super::*;
    use crate::utils::decimal;

    #[test]
    fn test_decimal_infinity() {
        let inf = Decimal::infinity();
        assert_eq!(inf, Decimal::MAX);
    }

    #[test]
    fn test_f64_infinity() {
        let inf = <f64 as FloatLike>::infinity();
        assert!(inf.is_infinite() && inf.is_sign_positive());
    }

    #[test]
    fn test_decimal_neg_infinity() {
        let neg_inf = Decimal::neg_infinity();
        assert_eq!(neg_inf, Decimal::MIN);
    }

    #[test]
    fn test_f64_neg_infinity() {
        let neg_inf = <f64 as FloatLike>::neg_infinity();
        assert!(neg_inf.is_infinite() && neg_inf.is_sign_negative());
    }

    #[test]
    fn test_decimal_zero() {
        let zero = Decimal::zero();
        assert_eq!(zero, Decimal::ZERO);
    }

    #[test]
    fn test_f64_zero() {
        let zero = f64::zero();
        assert_eq!(zero, 0.0);
    }

    #[test]
    fn test_decimal_two() {
        let two = Decimal::two();
        assert_eq!(two, Decimal::from_f64(2.0).unwrap());
    }

    #[test]
    fn test_f64_two() {
        let two = f64::two();
        assert_eq!(two, 2.0);
    }

    #[test]
    fn test_decimal_powf() {
        let base = Decimal::from_f64(2.0).unwrap();
        let exp = Decimal::from_f64(3.0).unwrap();
        let result = base.powf(exp);
        assert_eq!(result, Decimal::from_f64(8.0).unwrap());
    }

    #[test]
    fn test_f64_powf() {
        let base = 2.0;
        let exp = 3.0;
        let result = decimal::FloatLike::powf(base, exp);
        assert_eq!(result, 8.0);
    }

    #[test]
    fn test_decimal_pow_two() {
        let base = Decimal::from_f64(3.0).unwrap();
        let result = base.pow_two();
        assert_eq!(result, Decimal::from_f64(9.0).unwrap());
    }

    #[test]
    fn test_f64_pow_two() {
        let base = 3.0;
        let result = base.pow_two();
        assert_eq!(result, 9.0);
    }

    #[test]
    fn test_decimal_ln() {
        let value = Decimal::from_f64(10.0).unwrap();
        let ln_value = value.ln();
        let expected = Decimal::from_f64(std::f64::consts::LN_10).unwrap(); // ln(10)
        assert_eq!(ln_value, expected);
    }

    #[test]
    fn test_f64_ln() {
        let value = 10.0;
        let ln_value = decimal::FloatLike::ln(value);
        assert!((ln_value - std::f64::consts::LN_10).abs() < 1e-6); // ln(10) ≈ 2.302585
    }

    #[test]
    fn test_decimal_sqrt() {
        let value = Decimal::from_f64(16.0).unwrap();
        let sqrt_value = value.sqrt();
        assert_eq!(sqrt_value, Decimal::from_f64(4.0).unwrap());
    }

    #[test]
    fn test_f64_sqrt() {
        let value = 16.0;
        let sqrt_value = decimal::FloatLike::sqrt(value);
        assert_eq!(sqrt_value, 4.0);
    }

    #[test]
    fn test_decimal_abs() {
        let value = Decimal::from_f64(-10.0).unwrap();
        let abs_value = value.abs();
        assert_eq!(abs_value, Decimal::from_f64(10.0).unwrap());
    }

    #[test]
    fn test_f64_abs() {
        let value = -10.0;
        let abs_value = FloatLike::abs(value);
        assert_eq!(abs_value, 10.0);
    }

    #[test]
    fn test_decimal_is_infinite() {
        let inf = Decimal::infinity();
        let neg_inf = Decimal::neg_infinity();

        assert!(
            inf.is_infinite(),
            "Decimal::infinity() should be considered infinite"
        );
        assert!(
            neg_inf.is_infinite(),
            "Decimal::neg_infinity() should be considered infinite"
        );

        assert_eq!(
            inf,
            Decimal::MAX,
            "Decimal::infinity() should be equal to Decimal::MAX"
        );
        assert_eq!(
            neg_inf,
            Decimal::MIN,
            "Decimal::neg_infinity() should be equal to Decimal::MIN"
        );
    }

    #[test]
    fn test_f64_is_infinite() {
        let inf = <f64 as FloatLike>::infinity();
        let neg_inf = <f64 as FloatLike>::neg_infinity();
        assert!(inf.is_infinite());
        assert!(neg_inf.is_infinite());
    }
}
