
/// Asserts that two `Positive` values are relatively equal within a given epsilon.
///
/// This macro compares two `Positive` values and checks if their relative difference
/// is within the specified `epsilon`. It handles cases where one or both values
/// are zero, ensuring that the non-zero value is less than or equal to epsilon.
///
/// # Examples
///
/// ```
/// use optionstratlib::Positive;
/// use optionstratlib::{assert_pos_relative_eq, pos};
///
/// let a = pos!(1.0);
/// let b = pos!(1.0001);
/// let epsilon = pos!(0.001);
/// assert_pos_relative_eq!(a, b, epsilon); // Passes
///
/// let c = pos!(1.0);
/// let d = pos!(2.0);
/// let epsilon = pos!(0.001);
/// #[test]
/// #[should_panic]
/// assert_pos_relative_eq!(c, d, epsilon); // Panics
///
/// let e = pos!(0.0);
/// let f = pos!(0.0001);
/// let epsilon = pos!(0.001);
/// assert_pos_relative_eq!(e, f, epsilon); // Passes
///
/// let g = pos!(0.0);
/// let h = pos!(0.0011);
/// let epsilon = pos!(0.001);
/// #[test]
/// #[should_panic]
/// assert_pos_relative_eq!(g, h, epsilon); // Panics
/// ```
///
/// # Panics
///
/// This macro panics if the relative difference between the two values is greater than
/// the specified `epsilon`, or if one value is zero and the other is greater than epsilon.
/// The panic message includes the values being compared, the expected relative difference,
/// and the actual relative difference.
#[macro_export]
macro_rules! assert_pos_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left: $crate::Positive = $left;
        let right: $crate::Positive = $right;
        let epsilon: $crate::Positive = $epsilon;
        let abs_diff: $crate::Positive = (left.to_f64() - right.to_f64()).abs().into();
        let max_abs = left.max(right);

        if left == $crate::Positive::ZERO || right == $crate::Positive::ZERO {
            let non_zero_value = if left == $crate::Positive::ZERO {
                right
            } else {
                left
            };
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
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_exact_equality() {
        let a = pos!(1.0);
        let b = pos!(1.0);
        let epsilon = pos!(0.0001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_close_values() {
        let a = pos!(1.0);
        let b = pos!(1.0001);
        let epsilon = pos!(0.001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_values() {
        let a = Positive::ZERO;
        let b = Positive::ZERO;
        let epsilon = pos!(0.0001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_and_small_value() {
        let a = Positive::ZERO;
        let b = pos!(0.00001);
        let epsilon = pos!(0.00001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic(expected = "assertion failed")]
    fn test_values_exceeding_epsilon() {
        let a = pos!(1.0);
        let b = pos!(1.002);
        let epsilon = pos!(0.001);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_large_values() {
        let a = pos!(1000000.0);
        let b = pos!(1000001.0);
        let epsilon = pos!(0.000002);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_very_small_values() {
        let a = pos!(0.0000001);
        let b = pos!(0.0000001000001);
        let epsilon = pos!(0.000002);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic(expected = "assertion failed")]
    fn test_significantly_different_values() {
        let a = pos!(1.0);
        let b = pos!(2.0);
        let epsilon = pos!(0.1);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_equal_within_epsilon() {
        let a = pos!(100.0);
        let b = pos!(100.1);
        let epsilon = pos!(0.002);
        assert_pos_relative_eq!(a, b, epsilon);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic(expected = "assertion failed")]
    fn test_zero_and_large_value() {
        let a = Positive::ZERO;
        let b = pos!(1.0);
        let epsilon = pos!(0.0001);
        assert_pos_relative_eq!(a, b, epsilon);
    }
}
