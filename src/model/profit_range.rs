/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/
use crate::error::probability::{PriceErrorKind, ProbabilityError};
use crate::model::types::{ExpirationDate, Positive, Positive::ZERO, P_INFINITY};
use crate::strategies::probabilities::utils::{
    calculate_single_point_probability, PriceTrend, VolatilityAdjustment,
};

/// Represents a price range where a strategy is profitable
#[derive(Debug, Clone)]
pub struct ProfitLossRange {
    /// Lower price boundary of the profitable range
    /// None represents negative infinity
    pub lower_bound: Option<Positive>,

    /// Upper price boundary of the profitable range
    /// None represents positive infinity
    pub upper_bound: Option<Positive>,

    /// Probability of the underlying price ending within this range
    pub probability: Positive,
}

impl ProfitLossRange {
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
        lower_bound: Option<Positive>,
        upper_bound: Option<Positive>,
        probability: Positive,
    ) -> Result<Self, String> {
        // Validate boundaries if both are present
        if let (Some(lower), Some(upper)) = (lower_bound, upper_bound) {
            if lower >= upper {
                return Err("Lower bound must be less than upper bound".to_string());
            }
        }

        Ok(ProfitLossRange {
            lower_bound,
            upper_bound,
            probability,
        })
    }

    pub fn calculate_probability(
        &mut self,
        current_price: Positive,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
        expiration_date: ExpirationDate,
        risk_free_rate: Option<f64>,
    ) -> Result<(), ProbabilityError> {
        if self.lower_bound.unwrap_or(Positive::ZERO) > self.upper_bound.unwrap_or(P_INFINITY) {
            return Err(ProbabilityError::PriceError(
                PriceErrorKind::InvalidPriceRange {
                    range: format!(
                        "lower_bound: {} upper_bound: {}",
                        self.lower_bound.unwrap().value(),
                        self.upper_bound.unwrap().value()
                    ),
                    reason: "Lower bound must be less than upper bound".to_string(),
                },
            ));
        }
        // Calculate probabilities for the lower bound
        let (prob_below_lower, _) = calculate_single_point_probability(
            current_price,
            self.lower_bound.unwrap_or(Positive::ZERO),
            volatility_adj.clone(),
            trend.clone(),
            expiration_date.clone(),
            risk_free_rate,
        )?;

        // Calculate probabilities for the upper bound
        let (prob_below_upper, _) = calculate_single_point_probability(
            current_price,
            self.upper_bound.unwrap_or(P_INFINITY),
            volatility_adj,
            trend,
            expiration_date,
            risk_free_rate,
        )?;

        self.probability = prob_below_upper - prob_below_lower;
        Ok(())
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
    pub fn contains(&self, price: Positive) -> bool {
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
    use crate::f2p;

    #[test]
    fn test_profit_range_creation() {
        let range = ProfitLossRange::new(Some(f2p!(100.0)), Some(f2p!(110.0)), f2p!(0.5));
        assert!(range.is_ok());
    }

    #[test]
    fn test_invalid_bounds() {
        let range = ProfitLossRange::new(Some(f2p!(110.0)), Some(f2p!(100.0)), f2p!(0.5));
        assert!(range.is_err());
    }

    #[test]
    fn test_infinite_bounds() {
        let range = ProfitLossRange::new(None, Some(f2p!(100.0)), f2p!(0.5));
        assert!(range.is_ok());

        let range = ProfitLossRange::new(Some(f2p!(100.0)), None, f2p!(0.5));
        assert!(range.is_ok());
    }

    #[test]
    fn test_contains() {
        let range = ProfitLossRange::new(Some(f2p!(100.0)), Some(f2p!(110.0)), f2p!(0.5)).unwrap();

        assert!(!range.contains(f2p!(99.0)));
        assert!(range.contains(f2p!(100.0)));
        assert!(range.contains(f2p!(105.0)));
        assert!(range.contains(f2p!(110.0)));
        assert!(!range.contains(f2p!(111.0)));
    }

    #[test]
    fn test_contains_infinite_bounds() {
        let lower_infinite = ProfitLossRange::new(None, Some(f2p!(100.0)), f2p!(0.5)).unwrap();
        assert!(lower_infinite.contains(f2p!(50.0)));
        assert!(!lower_infinite.contains(f2p!(101.0)));

        let upper_infinite = ProfitLossRange::new(Some(f2p!(100.0)), None, f2p!(0.5)).unwrap();
        assert!(!upper_infinite.contains(f2p!(99.0)));
        assert!(upper_infinite.contains(f2p!(150.0)));
    }
}

#[cfg(test)]
mod tests_calculate_probability {
    use super::*;
    use crate::f2p;

    fn create_basic_range() -> ProfitLossRange {
        ProfitLossRange::new(Some(f2p!(90.0)), Some(f2p!(110.0)), f2p!(0.0)).unwrap()
    }

    #[test]
    fn test_basic_probability_calculation() {
        let mut range = create_basic_range();
        let result = range.calculate_probability(
            f2p!(100.0),
            None,
            None,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= f2p!(1.0));
    }

    #[test]
    #[should_panic(expected = "Lower bound must be less than upper bound")]
    fn test_invalid_bounds() {
        let _ = ProfitLossRange::new(Some(f2p!(110.0)), Some(f2p!(90.0)), f2p!(0.0)).unwrap();
    }

    #[test]
    fn test_with_volatility_adjustment() {
        let mut range = create_basic_range();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: f2p!(0.25),
            std_dev_adjustment: f2p!(0.05),
        });

        let result = range.calculate_probability(
            f2p!(100.0),
            vol_adj,
            None,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_with_upward_trend() {
        let mut range = create_basic_range();
        let trend = Some(PriceTrend {
            drift_rate: 0.10, // 10% tendencia alcista anual
            confidence: 0.95,
        });

        let result = range.calculate_probability(
            f2p!(100.0),
            None,
            trend,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_with_downward_trend() {
        let mut range = create_basic_range();
        let trend = Some(PriceTrend {
            drift_rate: -0.10,
            confidence: 0.95,
        });

        let result = range.calculate_probability(
            f2p!(100.0),
            None,
            trend,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_infinite_lower_bound() {
        let mut range = ProfitLossRange::new(None, Some(f2p!(110.0)), f2p!(0.0)).unwrap();

        let result = range.calculate_probability(
            f2p!(100.0),
            None,
            None,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_infinite_upper_bound() {
        let mut range = ProfitLossRange::new(Some(f2p!(90.0)), None, f2p!(0.0)).unwrap();

        let result = range.calculate_probability(
            f2p!(100.0),
            None,
            None,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_combined_adjustments() {
        let mut range = create_basic_range();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: f2p!(0.25),
            std_dev_adjustment: f2p!(0.05),
        });
        let trend = Some(PriceTrend {
            drift_rate: 0.10,
            confidence: 0.95,
        });

        let result = range.calculate_probability(
            f2p!(100.0),
            vol_adj,
            trend,
            ExpirationDate::Days(30.0),
            Some(0.05),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_different_expiration_dates() {
        let mut range = create_basic_range();

        let expirations = vec![
            ExpirationDate::Days(1.0),
            ExpirationDate::Days(30.0),
            ExpirationDate::Days(90.0),
            ExpirationDate::Days(365.0),
        ];

        for expiration in expirations {
            let result =
                range.calculate_probability(f2p!(100.0), None, None, expiration, Some(0.05));

            assert!(result.is_ok());
            assert!(range.probability > Positive::ZERO);
            assert!(range.probability <= f2p!(1.0));
        }
    }

    #[test]
    fn test_extreme_prices() {
        let mut range = create_basic_range();

        let extreme_prices = vec![f2p!(1.0), f2p!(1000.0), f2p!(10000.0)];

        for price in extreme_prices {
            let result = range.calculate_probability(
                price,
                None,
                None,
                ExpirationDate::Days(30.0),
                Some(0.05),
            );

            assert!(result.is_ok());
            assert!(range.probability >= Positive::ZERO);
            assert!(range.probability <= f2p!(1.0));
        }
    }
}
