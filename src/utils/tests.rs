#[macro_export]
macro_rules! assert_positivef64_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left: PositiveF64 = $left;
        let right: PositiveF64 = $right;
        let epsilon: PositiveF64 = $epsilon;

        let abs_diff = pos!((left.value() - right.value()).abs());
        let max_abs = left.max(right);

        if left == PZERO || right == PZERO {
            // Si uno es cero, comparamos con el valor no-cero
            let non_zero_value = if left == PZERO { right } else { left };
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
    use crate::model::types::PositiveF64;
    use crate::model::types::PZERO;
    use crate::pos;

    #[test]
    fn test_exact_equality() {
        let a = pos!(1.0);
        let b = pos!(1.0);
        let epsilon = pos!(0.0001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_close_values() {
        let a = pos!(1.0);
        let b = pos!(1.0001);
        let epsilon = pos!(0.001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_zero_values() {
        let a = PZERO;
        let b = PZERO;
        let epsilon = pos!(0.0001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_zero_and_small_value() {
        let a = PZERO;
        let b = pos!(0.00001);
        let epsilon = pos!(0.00001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_values_exceeding_epsilon() {
        let a = pos!(1.0);
        let b = pos!(1.002);
        let epsilon = pos!(0.001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_large_values() {
        let a = pos!(1000000.0);
        let b = pos!(1000001.0);
        let epsilon = pos!(0.000002);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_very_small_values() {
        let a = pos!(0.0000001);
        let b = pos!(0.0000001000001);
        let epsilon = pos!(0.000002);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_significantly_different_values() {
        let a = pos!(1.0);
        let b = pos!(2.0);
        let epsilon = pos!(0.1);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    fn test_equal_within_epsilon() {
        let a = pos!(100.0);
        let b = pos!(100.1);
        let epsilon = pos!(0.002);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn test_zero_and_large_value() {
        let a = PZERO;
        let b = pos!(1.0);
        let epsilon = pos!(0.0001);
        assert_positivef64_relative_eq!(a, b, epsilon);
    }
}