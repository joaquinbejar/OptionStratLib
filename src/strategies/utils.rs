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
    use crate::f2p;
    use approx::assert_relative_eq;

    #[test]
    fn test_find_optimal_side_variants() {
        let upper = FindOptimalSide::Upper;
        let lower = FindOptimalSide::Lower;
        let all = FindOptimalSide::All;
        let range = FindOptimalSide::Range(f2p!(100.0), f2p!(200.0));

        assert!(matches!(upper, FindOptimalSide::Upper));
        assert!(matches!(lower, FindOptimalSide::Lower));
        assert!(matches!(all, FindOptimalSide::All));
        assert!(matches!(range, FindOptimalSide::Range(_, _)));
    }

    #[test]
    fn test_optimization_criteria_variants() {
        let ratio = OptimizationCriteria::Ratio;
        let area = OptimizationCriteria::Area;

        assert!(matches!(ratio, OptimizationCriteria::Ratio));
        assert!(matches!(area, OptimizationCriteria::Area));
    }

    #[test]
    fn test_calculate_price_range_basic() {
        let start = f2p!(100.0);
        let end = f2p!(110.0);
        let step = f2p!(2.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 7);
        assert_eq!(range[0], f2p!(100.0));
        assert_eq!(range[1], f2p!(102.0));
        assert_eq!(range[2], f2p!(104.0));
        assert_eq!(range[3], f2p!(106.0));
        assert_eq!(range[4], f2p!(108.0));
        assert_eq!(range[5], f2p!(110.0));
        assert_eq!(range[6], f2p!(112.0));
    }

    #[test]
    fn test_calculate_price_range_single_step() {
        let start = f2p!(100.0);
        let end = f2p!(100.0);
        let step = f2p!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], f2p!(100.0));
    }

    #[test]
    fn test_calculate_price_range_large_step() {
        let start = f2p!(100.0);
        let end = f2p!(110.0);
        let step = f2p!(20.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], f2p!(100.0));
    }

    #[test]
    fn test_calculate_price_range_fractional_step() {
        let start = f2p!(1.0);
        let end = f2p!(2.0);
        let step = f2p!(0.3);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 5);
        assert_relative_eq!(range[0].to_f64(), 1.0, epsilon = 1e-16);
        assert_relative_eq!(range[1].to_f64(), 1.3, epsilon = 1e-16);
        assert_relative_eq!(range[2].to_f64(), 1.6, epsilon = 1e-16);
        assert_relative_eq!(range[3].to_f64(), 1.9, epsilon = 1e-16);
    }

    #[test]
    fn test_calculate_price_range_empty() {
        let start = f2p!(100.0);
        let end = f2p!(90.0);
        let step = f2p!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 1);
    }
}
