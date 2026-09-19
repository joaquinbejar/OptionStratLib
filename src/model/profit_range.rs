/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/
use crate::analytics::probability::{PriceTrend, VolatilityAdjustment}; // facade-compat: analytics
use crate::analytics::profit_range::ProfitRangeProbability; // facade-compat: analytics
use crate::error::probability::ProbabilityError;
use crate::model::ExpirationDate;
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a price range where a strategy is profitable
#[derive(
    Debug, Clone, ToSchema, Serialize, Deserialize, PartialEq, Eq, Default, Hash, PartialOrd, Ord,
)]
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
    ///
    /// # Errors
    ///
    /// Returns `ProbabilityError::RangeError` wrapping a
    /// `ProfitLossRangeErrorKind::InvalidProfitRange` when both
    /// bounds are present and `lower_bound >= upper_bound`. The
    /// constructor does not currently validate that `probability`
    /// lies in `[0, 1]`; values outside that interval are accepted
    /// and must be validated upstream.
    pub fn new(
        lower_bound: Option<Positive>,
        upper_bound: Option<Positive>,
        probability: Positive,
    ) -> Result<Self, ProbabilityError> {
        // Validate boundaries if both are present
        if let (Some(lower), Some(upper)) = (lower_bound, upper_bound)
            && lower >= upper
        {
            return Err(ProbabilityError::RangeError(
                crate::error::probability::ProfitLossRangeErrorKind::InvalidProfitRange {
                    range: format!("[{}, {}]", lower.to_f64(), upper.to_f64()),
                    reason: "Lower bound must be less than upper bound".to_string(),
                },
            ));
        }

        Ok(ProfitLossRange {
            lower_bound,
            upper_bound,
            probability,
        })
    }

    /// Calculates the probability of an asset's price falling within a specified range at expiration.
    ///
    /// This method computes the probability that the underlying asset's price will be between the
    /// lower and upper bounds of a price range at the expiration date, based on various market factors
    /// and statistical models.
    ///
    /// # Parameters
    ///
    /// * `current_price` - The current market price of the underlying asset.
    /// * `volatility_adj` - Optional adjustment for volatility parameters, including base volatility and
    ///   standard deviation adjustments. If None, default volatility settings will be used.
    /// * `trend` - Optional price trend parameters, including drift rate and confidence level.
    ///   If None, no trend assumption will be applied.
    /// * `expiration_date` - The date when the probability calculation applies, specified either as
    ///   days to expiration or an absolute datetime.
    /// * `risk_free_rate` - Optional risk-free interest rate used in probability calculations.
    ///   If None, a default value will be used.
    ///
    /// # Returns
    ///
    /// * `Result<(), ProbabilityError>` - Returns Ok(()) if the calculation was successful,
    ///   updating the internal probability field. Returns Err with a ProbabilityError if the
    ///   calculation failed, such as due to invalid price ranges.
    ///
    /// # Errors
    ///
    /// This function can return the following errors:
    /// * `ProbabilityError::PriceError` - If the lower bound exceeds the upper bound.
    /// * `ProbabilityError::CalculationError` with `InvalidProbability` - If the
    ///   probability below the upper bound comes out smaller than below the lower
    ///   bound. A distribution function is monotone, so this only happens when
    ///   the inputs sit past the precision of the price model (a spot near
    ///   `Positive::MAX` with a volatility around `1e-28`); it is reported rather
    ///   than floored to zero.
    /// * Other errors may be propagated from the underlying `calculate_single_point_probability` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::model::ProfitLossRange;
    /// use positive::{pos_or_panic, spos, Positive};
    /// use optionstratlib::ExpirationDate;
    /// use optionstratlib::strategies::probabilities::VolatilityAdjustment;
    /// let mut range = ProfitLossRange {
    ///     lower_bound: spos!(50.0),
    ///     upper_bound: spos!(60.0),
    ///     probability: Positive::ZERO,
    /// };
    ///
    /// let result = range.calculate_probability(
    ///     &pos_or_panic!(55.0),
    ///     Some(VolatilityAdjustment {
    ///         base_volatility: pos_or_panic!(0.2),
    ///         std_dev_adjustment: Positive::ONE
    ///     }),
    ///     None,
    ///     &ExpirationDate::Days(pos_or_panic!(30.0)),
    ///     Some(dec!(0.03)),
    /// );
    /// ```
    pub fn calculate_probability(
        &mut self,
        current_price: &Positive,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
        expiration_date: &ExpirationDate,
        risk_free_rate: Option<Decimal>,
    ) -> Result<(), ProbabilityError> {
        // facade-compat: analytics
        ProfitRangeProbability::calculate_probability(
            self,
            current_price,
            volatility_adj,
            trend,
            expiration_date,
            risk_free_rate,
        )
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
    #[must_use]
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
    use positive::{Positive, pos_or_panic, spos};

    #[test]
    fn test_profit_range_creation() {
        let range = ProfitLossRange::new(spos!(100.0), spos!(110.0), pos_or_panic!(0.5));
        assert!(range.is_ok());
    }

    #[test]
    fn test_invalid_bounds() {
        let range = ProfitLossRange::new(spos!(110.0), spos!(100.0), pos_or_panic!(0.5));
        assert!(range.is_err());
    }

    #[test]
    fn test_infinite_bounds() {
        let range = ProfitLossRange::new(None, spos!(100.0), pos_or_panic!(0.5));
        assert!(range.is_ok());

        let range = ProfitLossRange::new(spos!(100.0), None, pos_or_panic!(0.5));
        assert!(range.is_ok());
    }

    #[test]
    fn test_contains() {
        let range = ProfitLossRange::new(spos!(100.0), spos!(110.0), pos_or_panic!(0.5)).unwrap();

        assert!(!range.contains(pos_or_panic!(99.0)));
        assert!(range.contains(Positive::HUNDRED));
        assert!(range.contains(pos_or_panic!(105.0)));
        assert!(range.contains(pos_or_panic!(110.0)));
        assert!(!range.contains(pos_or_panic!(111.0)));
    }

    #[test]
    fn test_contains_infinite_bounds() {
        let lower_infinite = ProfitLossRange::new(None, spos!(100.0), pos_or_panic!(0.5)).unwrap();
        assert!(lower_infinite.contains(pos_or_panic!(50.0)));
        assert!(!lower_infinite.contains(pos_or_panic!(101.0)));

        let upper_infinite = ProfitLossRange::new(spos!(100.0), None, pos_or_panic!(0.5)).unwrap();
        assert!(!upper_infinite.contains(pos_or_panic!(99.0)));
        assert!(upper_infinite.contains(pos_or_panic!(150.0)));
    }
}
