/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::model::types::PositiveF64;

pub enum FindOptimalSide {
    Upper,
    Lower,
    All,
    Range(PositiveF64, PositiveF64),
}

pub(crate) enum OptimizationCriteria {
    Ratio,
    Area,
}

pub(crate) fn calculate_price_range(
    start_price: PositiveF64,
    end_price: PositiveF64,
    step: PositiveF64,
) -> Vec<PositiveF64> {
    let mut range = Vec::new();
    let mut current_price = start_price;
    while current_price <= end_price {
        range.push(current_price);
        current_price += step;
    }
    range
}

#[cfg(test)]
mod tests_strategies_utils {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;

    #[test]
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
    fn test_optimization_criteria_variants() {
        let ratio = OptimizationCriteria::Ratio;
        let area = OptimizationCriteria::Area;

        assert!(matches!(ratio, OptimizationCriteria::Ratio));
        assert!(matches!(area, OptimizationCriteria::Area));
    }

    #[test]
    fn test_calculate_price_range_basic() {
        let start = pos!(100.0);
        let end = pos!(110.0);
        let step = pos!(2.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 6);
        assert_eq!(range[0], pos!(100.0));
        assert_eq!(range[1], pos!(102.0));
        assert_eq!(range[2], pos!(104.0));
        assert_eq!(range[3], pos!(106.0));
        assert_eq!(range[4], pos!(108.0));
        assert_eq!(range[5], pos!(110.0));
    }

    #[test]
    fn test_calculate_price_range_single_step() {
        let start = pos!(100.0);
        let end = pos!(100.0);
        let step = pos!(1.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 1);
        assert_eq!(range[0], pos!(100.0));
    }

    #[test]
    fn test_calculate_price_range_large_step() {
        let start = pos!(100.0);
        let end = pos!(110.0);
        let step = pos!(20.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 1);
        assert_eq!(range[0], pos!(100.0));
    }

    #[test]
    fn test_calculate_price_range_fractional_step() {
        let start = pos!(1.0);
        let end = pos!(2.0);
        let step = pos!(0.3);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 4);
        assert_relative_eq!(range[0], pos!(1.0), epsilon = f64::EPSILON);
        assert_relative_eq!(range[1], pos!(1.3), epsilon = f64::EPSILON);
        assert_relative_eq!(range[2], pos!(1.6), epsilon = f64::EPSILON);
        assert_relative_eq!(range[3], pos!(1.9), epsilon = f64::EPSILON);
    }

    #[test]
    fn test_calculate_price_range_empty() {
        let start = pos!(100.0);
        let end = pos!(90.0);
        let step = pos!(1.0);

        let range = calculate_price_range(start, end, step);

        assert!(range.is_empty());
    }
}
