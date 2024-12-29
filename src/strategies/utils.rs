/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::Positive;
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum FindOptimalSide {
    Upper,
    Lower,
    All,
    Range(Positive, Positive),
}

impl Display for FindOptimalSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindOptimalSide::Upper => write!(f, "Upper"),
            FindOptimalSide::Lower => write!(f, "Lower"),
            FindOptimalSide::All => write!(f, "All"),
            FindOptimalSide::Range(start, end) => write!(f, "Range: {} - {}", start, end),
        }
    }
}

pub enum OptimizationCriteria {
    Ratio,
    Area,
}

pub(crate) fn calculate_price_range(
    start_price: Positive,
    end_price: Positive,
    step: Positive,
) -> Vec<Positive> {
    let mut range = Vec::new();
    let mut current_price = start_price;
    range.push(current_price);
    while current_price <= end_price {
        current_price += step;
        range.push(current_price);
    }
    range
}

#[cfg(test)]
mod tests_strategies_utils {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_find_optimal_side_variants() {
        let upper = FindOptimalSide::Upper;
        let lower = FindOptimalSide::Lower;
        let all = FindOptimalSide::All;
        let range = FindOptimalSide::Range(pos!(100.0), pos!(200.0));

        assert!(matches!(upper, FindOptimalSide::Upper));
        assert!(matches!(lower, FindOptimalSide::Lower));
        assert!(matches!(all, FindOptimalSide::All));
        assert!(matches!(range, FindOptimalSide::Range(_, _)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_optimization_criteria_variants() {
        let ratio = OptimizationCriteria::Ratio;
        let area = OptimizationCriteria::Area;

        assert!(matches!(ratio, OptimizationCriteria::Ratio));
        assert!(matches!(area, OptimizationCriteria::Area));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_calculate_price_range_basic() {
        let start = pos!(100.0);
        let end = pos!(110.0);
        let step = pos!(2.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 7);
        assert_eq!(range[0], pos!(100.0));
        assert_eq!(range[1], pos!(102.0));
        assert_eq!(range[2], pos!(104.0));
        assert_eq!(range[3], pos!(106.0));
        assert_eq!(range[4], pos!(108.0));
        assert_eq!(range[5], pos!(110.0));
        assert_eq!(range[6], pos!(112.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_calculate_price_range_single_step() {
        let start = pos!(100.0);
        let end = pos!(100.0);
        let step = pos!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_calculate_price_range_large_step() {
        let start = pos!(100.0);
        let end = pos!(110.0);
        let step = pos!(20.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_calculate_price_range_fractional_step() {
        let start = pos!(1.0);
        let end = pos!(2.0);
        let step = pos!(0.3);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 5);
        assert_relative_eq!(range[0].to_f64(), 1.0, epsilon = 1e-16);
        assert_relative_eq!(range[1].to_f64(), 1.3, epsilon = 1e-16);
        assert_relative_eq!(range[2].to_f64(), 1.6, epsilon = 1e-16);
        assert_relative_eq!(range[3].to_f64(), 1.9, epsilon = 1e-16);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_calculate_price_range_empty() {
        let start = pos!(100.0);
        let end = pos!(90.0);
        let step = pos!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 1);
    }
}
