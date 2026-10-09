/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! [`ProfitLossRange`], a price range with its expiry probability, and the
//! capability that fills that probability in.
//!
//! [`ProfitRangeProbability`] fills the `probability` field from the
//! lognormal single-point kernels in [`crate::analytics::probability`]; the
//! inherent `ProfitLossRange::calculate_probability` forwards to it. Both are
//! analytics-owned, and the type is reached as
//! `optionstratlib_analytics::analytics::ProfitLossRange`.

use crate::analytics::probability::{
    PriceTrend, VolatilityAdjustment, calculate_single_point_probability,
};
use crate::error::probability::{
    PriceErrorKind, ProbabilityCalculationErrorKind, ProbabilityError,
};
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::decimal_to_f64;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a price range where a strategy is profitable
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
                    range: format!("[{lower}, {upper}]"),
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
    /// * `volatility` - Volatility to price the range at. Required: the kernel
    ///   has no default volatility (see [`VolatilityAdjustment`]).
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
    /// use optionstratlib_analytics::analytics::ProfitLossRange;
    /// use optionstratlib_core::{model::Positive, pos_or_panic, spos};
    /// use optionstratlib_core::model::ExpirationDate;
    /// use optionstratlib_analytics::analytics::VolatilityAdjustment;
    /// let mut range = ProfitLossRange {
    ///     lower_bound: spos!(50.0),
    ///     upper_bound: spos!(60.0),
    ///     probability: Positive::ZERO,
    /// };
    ///
    /// let result = range.calculate_probability(
    ///     &pos_or_panic!(55.0),
    ///     VolatilityAdjustment {
    ///         base_volatility: pos_or_panic!(0.2),
    ///         std_dev_adjustment: Positive::ONE
    ///     },
    ///     None,
    ///     &ExpirationDate::Days(pos_or_panic!(30.0)),
    ///     Some(dec!(0.03)),
    /// );
    /// ```
    pub fn calculate_probability(
        &mut self,
        current_price: &Positive,
        volatility: VolatilityAdjustment,
        trend: Option<PriceTrend>,
        expiration_date: &ExpirationDate,
        risk_free_rate: Option<Decimal>,
    ) -> Result<(), ProbabilityError> {
        // Compatibility wrapper over the canonical `ProfitRangeProbability`;
        // removed in the batch behind the version bump (#498).
        ProfitRangeProbability::calculate_probability(
            self,
            current_price,
            volatility,
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

/// Fills a price range with the probability that the underlying expires
/// inside it.
///
/// Implemented for [`ProfitLossRange`] by the analytics layer; the inherent
/// method of the same name on `ProfitLossRange` forwards to this
/// implementation.
pub trait ProfitRangeProbability {
    /// Calculates the probability of an asset's price falling within a specified range at expiration.
    ///
    /// This method computes the probability that the underlying asset's price will be between the
    /// lower and upper bounds of a price range at the expiration date, based on various market factors
    /// and statistical models.
    ///
    /// # Parameters
    ///
    /// * `current_price` - The current market price of the underlying asset.
    /// * `volatility` - Volatility to price the range at. Required: the kernel
    ///   has no default volatility (see [`VolatilityAdjustment`]).
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
    /// use optionstratlib_analytics::analytics::ProfitLossRange;
    /// use optionstratlib_core::{model::Positive, pos_or_panic, spos};
    /// use optionstratlib_core::model::ExpirationDate;
    /// use optionstratlib_analytics::analytics::{ProfitRangeProbability, VolatilityAdjustment};
    /// let mut range = ProfitLossRange {
    ///     lower_bound: spos!(50.0),
    ///     upper_bound: spos!(60.0),
    ///     probability: Positive::ZERO,
    /// };
    ///
    /// let result = range.calculate_probability(
    ///     &pos_or_panic!(55.0),
    ///     VolatilityAdjustment {
    ///         base_volatility: pos_or_panic!(0.2),
    ///         std_dev_adjustment: Positive::ONE
    ///     },
    ///     None,
    ///     &ExpirationDate::Days(pos_or_panic!(30.0)),
    ///     Some(dec!(0.03)),
    /// );
    /// ```
    fn calculate_probability(
        &mut self,
        current_price: &Positive,
        volatility: VolatilityAdjustment,
        trend: Option<PriceTrend>,
        expiration_date: &ExpirationDate,
        risk_free_rate: Option<Decimal>,
    ) -> Result<(), ProbabilityError>;
}

impl ProfitRangeProbability for ProfitLossRange {
    fn calculate_probability(
        &mut self,
        current_price: &Positive,
        volatility: VolatilityAdjustment,
        trend: Option<PriceTrend>,
        expiration_date: &ExpirationDate,
        risk_free_rate: Option<Decimal>,
    ) -> Result<(), ProbabilityError> {
        let lower = self.lower_bound.unwrap_or(Positive::ZERO);
        let upper = self.upper_bound.unwrap_or(Positive::MAX);
        if lower > upper {
            return Err(ProbabilityError::PriceError(
                PriceErrorKind::InvalidPriceRange {
                    range: format!("lower_bound: {lower} upper_bound: {upper}"),
                    reason: "Lower bound must be less than upper bound".to_string(),
                },
            ));
        }
        // Calculate probabilities for the lower bound
        let (prob_below_lower, _) = calculate_single_point_probability(
            current_price,
            &self.lower_bound.unwrap_or(Positive::ZERO),
            volatility,
            trend.clone(),
            expiration_date,
            risk_free_rate,
        )?;

        // Calculate probabilities for the upper bound
        let (prob_below_upper, _) = calculate_single_point_probability(
            current_price,
            &self.upper_bound.unwrap_or(Positive::MAX),
            volatility,
            trend,
            expiration_date,
            risk_free_rate,
        )?;

        // A distribution function is monotone, so `upper >= lower` must give
        // `prob_below_upper >= prob_below_lower`; equality carries zero mass,
        // which `sub_or_none` returns. Equality does not mean the bounds
        // coincide: two distinct bounds far into a tail can round to the same
        // `Decimal`, and zero is still the right probability. A smaller
        // probability at the upper bound is a result outside the model's
        // precision, not a property of the range: with a spot near
        // `Positive::MAX` and a volatility of `1e-28`, `(MAX - 4) / MAX`
        // rounds to `0.9999999999999999999999999999` and `Decimal::checked_ln`
        // returns `+9e-28` for it where the true value is `-1e-28`, so the
        // lower bound comes out three standard deviations *above* the spot
        // (`0.9987`) while the upper bound sits at the median (`0.5`). It is
        // reported rather than floored to zero, which would be a probability
        // nobody computed; the raw `Positive` operator aborted on it (#569).
        // Ordinary inputs cannot reach this: the `ln` error is at the 28th
        // decimal and only surfaces once `vol * sqrt(T)` is below `~1e-11`.
        self.probability = match prob_below_upper.sub_or_none(&prob_below_lower.to_dec()) {
            Some(probability) => probability,
            None => {
                // The reported gap is formed from the nearest `f64` of each
                // probability (#828); a failed conversion is its own error.
                let value = decimal_to_f64(prob_below_upper.to_dec())?
                    - decimal_to_f64(prob_below_lower.to_dec())?;
                return Err(ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::InvalidProbability {
                        value,
                        reason: format!(
                            "probability below the upper bound {upper} ({prob_below_upper}) is \
                             smaller than below the lower bound {lower} ({prob_below_lower}); \
                             the inputs are outside the precision of the price model"
                        ),
                    },
                ));
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests_calculate_probability {
    use super::*;
    use crate::analytics::probability::flat_volatility_0_2;
    use optionstratlib_core::{pos_or_panic, spos};

    use optionstratlib_core::constants::DAYS_IN_A_YEAR;

    use rust_decimal_macros::dec;

    fn price_trend(
        drift_rate: rust_decimal::Decimal,
        confidence: rust_decimal::Decimal,
    ) -> PriceTrend {
        match PriceTrend::new(drift_rate, confidence) {
            Ok(trend) => trend,
            Err(e) => panic!("valid trend: {e}"),
        }
    }

    fn create_basic_range() -> ProfitLossRange {
        ProfitLossRange::new(spos!(90.0), spos!(110.0), Positive::ZERO).unwrap()
    }

    #[test]
    fn test_basic_probability_calculation() {
        let mut range = create_basic_range();
        let result = range.calculate_probability(
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= Positive::ONE);
    }

    #[test]
    #[should_panic(expected = "Lower bound must be less than upper bound")]
    fn test_invalid_bounds() {
        let _ = ProfitLossRange::new(spos!(110.0), spos!(90.0), Positive::ZERO).unwrap();
    }

    /// A range whose bounds coincide has probability zero, and that is a
    /// value. `new` rejects two equal explicit bounds, but an open upper bound
    /// resolves to `Positive::MAX`, so a lower bound at `MAX` gives a
    /// zero-width range through the public constructor. `sub_or_none` returns
    /// the zero; this pins that the #569 change did not turn it into an error.
    #[test]
    fn test_calculate_probability_zero_width_range_is_zero() {
        let mut range = ProfitLossRange::new(Some(Positive::MAX), None, Positive::ZERO)
            .expect("an open upper bound is a valid range");
        let result = range.calculate_probability(
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(
            result.is_ok(),
            "a zero-width range is a value, not an error"
        );
        assert_eq!(range.probability, Positive::ZERO);
    }

    #[test]
    fn test_with_volatility_adjustment() {
        let mut range = create_basic_range();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.05),
        };

        let result = range.calculate_probability(
            &Positive::HUNDRED,
            vol_adj,
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_with_upward_trend() {
        let mut range = create_basic_range();
        let trend = Some(price_trend(
            dec!(0.10), // 10% annual upward trend
            dec!(0.95),
        ));

        let result = range.calculate_probability(
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            trend,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_with_downward_trend() {
        let mut range = create_basic_range();
        let trend = Some(price_trend(dec!(-0.10), dec!(0.95)));

        let result = range.calculate_probability(
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            trend,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_infinite_lower_bound() {
        let mut range = ProfitLossRange::new(None, spos!(110.0), Positive::ZERO).unwrap();

        let result = range.calculate_probability(
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_infinite_upper_bound() {
        let mut range = ProfitLossRange::new(spos!(90.0), None, Positive::ZERO).unwrap();

        let result = range.calculate_probability(
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_combined_adjustments() {
        let mut range = create_basic_range();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.05),
        };
        let trend = Some(price_trend(dec!(0.10), dec!(0.95)));

        let result = range.calculate_probability(
            &Positive::HUNDRED,
            vol_adj,
            trend,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_different_expiration_dates() {
        let mut range = create_basic_range();

        let expirations = vec![
            ExpirationDate::Days(Positive::ONE),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            ExpirationDate::Days(pos_or_panic!(90.0)),
            ExpirationDate::Days(DAYS_IN_A_YEAR),
        ];

        for expiration in expirations {
            let result = range.calculate_probability(
                &Positive::HUNDRED,
                flat_volatility_0_2(),
                None,
                &expiration,
                Some(dec!(0.05)),
            );

            assert!(result.is_ok());
            assert!(range.probability > Positive::ZERO);
            assert!(range.probability <= Positive::ONE);
        }
    }

    #[test]
    fn test_extreme_prices() {
        let mut range = create_basic_range();

        let extreme_prices = vec![Positive::ONE, pos_or_panic!(1000.0), pos_or_panic!(10000.0)];

        for price in extreme_prices {
            let result = range.calculate_probability(
                &price,
                flat_volatility_0_2(),
                None,
                &ExpirationDate::Days(pos_or_panic!(30.0)),
                Some(dec!(0.05)),
            );

            assert!(result.is_ok());
            assert!(range.probability >= Positive::ZERO);
            assert!(range.probability <= Positive::ONE);
        }
    }
}

#[cfg(test)]
mod tests_profit_range {
    use super::*;
    use optionstratlib_core::{model::Positive, pos_or_panic, spos};

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
