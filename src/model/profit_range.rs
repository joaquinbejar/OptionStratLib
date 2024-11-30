/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 30/11/24
 ******************************************************************************/
use crate::model::types::PositiveF64;

/// Represents a price range where a strategy is profitable
#[derive(Debug, Clone)]
pub struct ProfitRange {
    /// Lower price boundary of the profitable range
    /// None represents negative infinity
    pub lower_bound: Option<PositiveF64>,

    /// Upper price boundary of the profitable range
    /// None represents positive infinity
    pub upper_bound: Option<PositiveF64>,

    /// Probability of the underlying price ending within this range
    pub probability: PositiveF64,
}

impl ProfitRange {
    /// Creates a new profit range
    ///
    /// # Arguments
    ///
    /// * `lower_bound` - Lower boundary price (None for negative infinity)
    /// * `upper_bound` - Upper boundary price (None for positive infinity)
    /// * `probability` - Probability of price ending in this range
    ///
    /// # Returns
    ///
    /// Returns a Result containing the ProfitRange if the boundaries are valid,
    /// or an error if the boundaries are invalid
    pub fn new(
        lower_bound: Option<PositiveF64>,
        upper_bound: Option<PositiveF64>,
        probability: PositiveF64,
    ) -> Result<Self, String> {
        // Validate boundaries if both are present
        if let (Some(lower), Some(upper)) = (lower_bound, upper_bound) {
            if lower >= upper {
                return Err("Lower bound must be less than upper bound".to_string());
            }
        }

        Ok(ProfitRange {
            lower_bound,
            upper_bound,
            probability,
        })
    }

    /// Checks if a given price is within this range
    ///
    /// # Arguments
    ///
    /// * `price` - The price to check
    ///
    /// # Returns
    ///
    /// Returns true if the price is within the range, false otherwise
    pub fn contains(&self, price: PositiveF64) -> bool {
        let above_lower = match self.lower_bound {
            Some(lower) => price >= lower,
            None => true,
        };

        let below_upper = match self.upper_bound {
            Some(upper) => price <= upper,
            None => true,
        };

        above_lower && below_upper
    }
}

#[cfg(test)]
mod tests_profit_range {
    use super::*;
    use crate::pos;

    #[test]
    fn test_profit_range_creation() {
        let range = ProfitRange::new(
            Some(pos!(100.0)),
            Some(pos!(110.0)),
            pos!(0.5),
        );
        assert!(range.is_ok());
    }

    #[test]
    fn test_invalid_bounds() {
        let range = ProfitRange::new(
            Some(pos!(110.0)),
            Some(pos!(100.0)),
            pos!(0.5),
        );
        assert!(range.is_err());
    }

    #[test]
    fn test_infinite_bounds() {
        let range = ProfitRange::new(
            None,
            Some(pos!(100.0)),
            pos!(0.5),
        );
        assert!(range.is_ok());

        let range = ProfitRange::new(
            Some(pos!(100.0)),
            None,
            pos!(0.5),
        );
        assert!(range.is_ok());
    }

    #[test]
    fn test_contains() {
        let range = ProfitRange::new(
            Some(pos!(100.0)),
            Some(pos!(110.0)),
            pos!(0.5),
        ).unwrap();

        assert!(!range.contains(pos!(99.0)));
        assert!(range.contains(pos!(100.0)));
        assert!(range.contains(pos!(105.0)));
        assert!(range.contains(pos!(110.0)));
        assert!(!range.contains(pos!(111.0)));
    }

    #[test]
    fn test_contains_infinite_bounds() {
        let lower_infinite = ProfitRange::new(
            None,
            Some(pos!(100.0)),
            pos!(0.5),
        ).unwrap();
        assert!(lower_infinite.contains(pos!(50.0)));
        assert!(!lower_infinite.contains(pos!(101.0)));

        let upper_infinite = ProfitRange::new(
            Some(pos!(100.0)),
            None,
            pos!(0.5),
        ).unwrap();
        assert!(!upper_infinite.contains(pos!(99.0)));
        assert!(upper_infinite.contains(pos!(150.0)));
    }
}