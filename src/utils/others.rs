/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/9/24
******************************************************************************/
use crate::constants::TOLERANCE;

#[allow(dead_code)]
pub(crate) fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
}

#[cfg(test)]
mod tests {
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
