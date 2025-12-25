//! Price Range Utilities
//!
//! This module provides utilities for working with optimal pricing strategies,
//! including enumerations for defining search strategies and functions for
//! generating price ranges.

use positive::Positive;
use rust_decimal::Decimal;
use std::fmt::Display;

/// Defines the strategy for finding optimal pricing sides.
///
/// This enumeration specifies which side of a price curve to consider when
/// finding optimal prices, or allows for searching across all prices or within
/// a specific range. It's used to control price discovery algorithms and to refine
/// the search space for optimal option pricing strategies.
///
/// # Variants
///
/// * `Upper` - Consider only the upper side of the price curve. This is useful when
///   expecting the market to move upward or when optimizing for maximum upside potential.
///
/// * `Lower` - Consider only the lower side of the price curve. This is useful when
///   expecting the market to move downward or when optimizing for downside protection.
///
/// * `All` - Consider the entire price curve, searching all possible price points
///   without any directional bias.
///
/// * `Range` - Consider prices within a specific range defined by start and end prices.
///   This allows for a more targeted search within a predetermined price band.
///
/// * `Deltable` - Select strikes in a strategy that ensure delta neutrality within
///   the specified threshold. This is useful for creating market-neutral strategies.
///
/// * `Center` - Focus the search around a center point of the price curve, allowing for
///   balanced optimization around a specific price level.
///
/// # Usage
///
/// This enum is typically used in option pricing and strategy optimization contexts
/// to control how the algorithm searches for optimal pricing points.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FindOptimalSide {
    /// Consider only the upper side of the price curve.
    ///
    /// Use this when you expect the market to trend upward or when you want to
    /// optimize for maximum upside potential in your option strategy.
    Upper,

    /// Consider only the lower side of the price curve.
    ///
    /// Use this when you expect the market to trend downward or when you want to
    /// optimize for downside protection in your option strategy.
    Lower,

    /// Consider the entire price curve.
    ///
    /// This performs a comprehensive search across all available price points
    /// without any directional bias. It's useful when you want to find the globally
    /// optimal solution regardless of market direction.
    All,

    /// Consider prices within a specific range defined by start and end prices.
    ///
    /// # Parameters
    ///
    /// * First `Positive` - The starting price of the range (inclusive).
    /// * Second `Positive` - The ending price of the range (inclusive).
    Range(Positive, Positive),

    /// Select strikes in a strategy that ensure delta neutrality within the specified threshold.
    ///
    /// This option is particularly useful for creating market-neutral strategies where the
    /// overall position delta remains close to zero.
    ///
    /// # Parameters
    ///
    /// * `Positive` - The maximum deviation from perfect delta neutrality that is allowed.
    ///   For example, a value of 0.05 means the strategy's delta can range from -0.05 to 0.05.
    Deltable(Positive),

    /// Focus the search around a center point of the price curve.
    ///
    /// This variant is useful when you have a specific price target in mind and want to
    /// optimize strategy parameters around that central point. It's commonly used for
    /// constructing balanced spreads or when you have a precise market outlook.
    Center,

    /// Select strikes in a strategy that ensure delta is within a specified range.
    DeltaRange(Decimal, Decimal),
}

impl Display for FindOptimalSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindOptimalSide::Upper => write!(f, "Upper"),
            FindOptimalSide::Lower => write!(f, "Lower"),
            FindOptimalSide::All => write!(f, "All"),
            FindOptimalSide::Range(start, end) => write!(f, "Range: {start} - {end}"),
            FindOptimalSide::Deltable(threshold) => write!(f, "Deltable: {threshold}"),
            FindOptimalSide::Center => write!(f, "Center"),
            FindOptimalSide::DeltaRange(min, max) => write!(f, "DeltaRange: {min} - {max}"),
        }
    }
}

/// Defines the criteria used for price optimization.
///
/// This enumeration specifies whether optimization should prioritize
/// price-to-value-ratio or overall area under the curve.
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

    use approx::assert_relative_eq;
    use positive::{Positive, pos_or_panic};

    #[test]
    fn test_find_optimal_side_variants() {
        let upper = FindOptimalSide::Upper;
        let lower = FindOptimalSide::Lower;
        let all = FindOptimalSide::All;
        let range = FindOptimalSide::Range(Positive::HUNDRED, pos_or_panic!(200.0));

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
        let start = Positive::HUNDRED;
        let end = pos_or_panic!(110.0);
        let step = Positive::TWO;

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 7);
        assert_eq!(range[0], Positive::HUNDRED);
        assert_eq!(range[1], pos_or_panic!(102.0));
        assert_eq!(range[2], pos_or_panic!(104.0));
        assert_eq!(range[3], pos_or_panic!(106.0));
        assert_eq!(range[4], pos_or_panic!(108.0));
        assert_eq!(range[5], pos_or_panic!(110.0));
        assert_eq!(range[6], pos_or_panic!(112.0));
    }

    #[test]
    fn test_calculate_price_range_single_step() {
        let start = Positive::HUNDRED;
        let end = Positive::HUNDRED;
        let step = Positive::ONE;

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], Positive::HUNDRED);
    }

    #[test]
    fn test_calculate_price_range_large_step() {
        let start = Positive::HUNDRED;
        let end = pos_or_panic!(110.0);
        let step = pos_or_panic!(20.0);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 2);
        assert_eq!(range[0], Positive::HUNDRED);
    }

    #[test]
    fn test_calculate_price_range_fractional_step() {
        let start = Positive::ONE;
        let end = Positive::TWO;
        let step = pos_or_panic!(0.3);

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 5);
        assert_relative_eq!(range[0].to_f64(), 1.0, epsilon = 1e-16);
        assert_relative_eq!(range[1].to_f64(), 1.3, epsilon = 1e-16);
        assert_relative_eq!(range[2].to_f64(), 1.6, epsilon = 1e-16);
        assert_relative_eq!(range[3].to_f64(), 1.9, epsilon = 1e-16);
    }

    #[test]
    fn test_calculate_price_range_empty() {
        let start = Positive::HUNDRED;
        let end = pos_or_panic!(90.0);
        let step = Positive::ONE;

        let range = calculate_price_range(start, end, step);

        assert_eq!(range.len(), 1);
    }
}
