//! Price Range Utilities
//!
//! This module provides utilities for working with optimal pricing strategies,
//! including enumerations for defining search strategies and functions for 
//! generating price ranges.

use std::fmt::Display;
use crate::model::positive::Positive;

/// Defines the strategy for finding optimal pricing sides.
///
/// This enumeration specifies which side of a price curve to consider when
/// finding optimal prices, or allows for searching across all prices or a specific range.
#[derive(Debug, Clone, Copy)]
pub enum FindOptimalSide {
    /// Consider only the upper side of the price curve.
    Upper,
    /// Consider only the lower side of the price curve.
    Lower,
    /// Consider the entire price curve.
    All,
    /// Consider prices within a specific range defined by start and end prices.
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

/// Defines the criteria used for price optimization.
///
/// This enumeration specifies whether optimization should prioritize 
/// price-to-value ratio or overall area under the curve.
pub enum OptimizationCriteria {
    /// Optimize based on price-to-value ratio.
    Ratio,
    /// Optimize based on the area under the price curve.
    Area,
}

/// Generates a vector of price points within a specified range.
///
/// # Arguments
///
/// * `start_price` - The starting price point
/// * `end_price` - The ending price point
/// * `step` - The increment between price points
///
/// # Returns
///
/// A vector containing price points from start_price to at least end_price,
/// incremented by step.
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

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_optimization_criteria_variants() {
        let ratio = OptimizationCriteria::Ratio;
        let area = OptimizationCriteria::Area;

        assert!(matches!(ratio, OptimizationCriteria::Ratio));
        assert!(matches!(area, OptimizationCriteria::Area));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_range_single_step() {
        let start = pos!(100.0);
        let end = pos!(100.0);
        let step = pos!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_range_large_step() {
        let start = pos!(100.0);
        let end = pos!(110.0);
        let step = pos!(20.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_range_empty() {
        let start = pos!(100.0);
        let end = pos!(90.0);
        let step = pos!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 1);
    }
}
