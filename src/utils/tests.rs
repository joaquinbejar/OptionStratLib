
#[macro_export]
macro_rules! assert_positivef64_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left: Positive = $left;
        let right: Positive = $right;
        let epsilon: Positive = $epsilon;

        let abs_diff: Positive = (left.to_f64() - right.to_f64()).abs().into();
        let max_abs = left.max(right);

        if left == Positive::ZERO || right == Positive::ZERO {
            let non_zero_value = if left == Positive::ZERO { right } else { left };
            assert!(
                non_zero_value <= epsilon,
                "assertion failed: `(left == right)` \
                 (left: `{}`, right: `{}`, expected max value: `{}`, actual value: `{}`)",
                left,
                right,
                epsilon,
                non_zero_value
            );
        } else {
            let relative_diff = abs_diff / max_abs;
            assert!(
                relative_diff <= epsilon,
                "assertion failed: `(left ≈ right)` \
                 (left: `{}`, right: `{}`, expected relative diff: `{}`, real relative diff: `{}`)",
                left,
                right,
                epsilon,
                relative_diff
            );
        }
    }};
}

#[cfg(test)]
mod tests_assert_positivef64_relative_eq {
    use crate::Positive;
    use crate::f2p;

    #[test]
    fn test_exact_equality() {
        let a = f2p!(1.0);
        let b = f2p!(1.0);
        let epsilon = f2p!(0.0001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_close_values() {
        let a = f2p!(1.0);
        let b = f2p!(1.0001);
        let epsilon = f2p!(0.001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_zero_values() {
        let a = Positive::ZERO;
        let b = Positive::ZERO;
        let epsilon = f2p!(0.0001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_zero_and_small_value() {
        let a = Positive::ZERO;
        let b = f2p!(0.00001);
        let epsilon = f2p!(0.00001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_values_exceeding_epsilon() {
        let a = f2p!(1.0);
        let b = f2p!(1.002);
        let epsilon = f2p!(0.001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_large_values() {
        let a = f2p!(1000000.0);
        let b = f2p!(1000001.0);
        let epsilon = f2p!(0.000002);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_very_small_values() {
        let a = f2p!(0.0000001);
        let b = f2p!(0.0000001000001);
        let epsilon = f2p!(0.000002);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_significantly_different_values() {
        let a = f2p!(1.0);
        let b = f2p!(2.0);
        let epsilon = f2p!(0.1);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_equal_within_epsilon() {
        let a = f2p!(100.0);
        let b = f2p!(100.1);
        let epsilon = f2p!(0.002);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_zero_and_large_value() {
        let a = Positive::ZERO;
        let b = f2p!(1.0);
        let epsilon = f2p!(0.0001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }
}
