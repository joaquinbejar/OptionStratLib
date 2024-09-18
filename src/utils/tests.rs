#[macro_export]
macro_rules! assert_decimal_relative_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {{
        let left: Decimal = $left;
        let right: Decimal = $right;
        let epsilon: Decimal = $epsilon;

        let abs_diff = (left - right).abs();
        let max_abs = left.abs().max(right.abs());

        if max_abs == Decimal::ZERO {
            assert!(
                abs_diff <= epsilon,
                "assertion failed: `(left == right)` \
                 (left: `{}`, right: `{}`, expected diff: `{}`, real diff: `{}`)",
                left,
                right,
                epsilon,
                abs_diff
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
