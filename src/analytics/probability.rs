/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/

//! Neutral price-probability kernels.
//!
//! [`calculate_single_point_probability`] and [`calculate_price_probability`]
//! evaluate a lognormal price distribution at expiry from a spot price, a
//! volatility, a time to expiry and optional [`VolatilityAdjustment`] and
//! [`PriceTrend`] inputs. Nothing here knows what a strategy is; the
//! strategy-bound aggregation lives in `strategies::probabilities`.

use crate::error::probability::{
    ExpirationErrorKind, PriceErrorKind, ProbabilityCalculationErrorKind, ProbabilityError,
};
use crate::f2du;
use crate::greeks::big_n;
use crate::model::ExpirationDate;
use crate::model::decimal::p_sqrt;
use num_traits::ToPrimitive;
use positive::Positive;
#[cfg(test)]
use positive::pos_or_panic;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Struct to hold volatility adjustment parameters
#[derive(Debug, Clone)]
pub struct VolatilityAdjustment {
    /// Base volatility
    pub base_volatility: Positive,
    /// Number of standard deviations to adjust
    pub std_dev_adjustment: Positive,
}

/// Struct to hold price trend parameters
#[derive(Debug, Clone)]
pub struct PriceTrend {
    /// Annual drift rate (positive for upward trend, negative for downward)
    pub drift_rate: f64,
    /// Confidence level for the trend (0 to 1)
    pub confidence: f64,
}

/// Calculates the probability of a stock price reaching a target price within a given timeframe.
///
/// This function estimates the probability of a stock following a log-normal distribution
/// to reach a specified target price before expiration. It also provides the probability
/// of the stock price being below or above the target price at the expiration date.
///
/// # Parameters
///
/// - `current_price`: The current stock price, represented as a `Positive`.
/// - `target_price`: The target stock price to evaluate, represented as a `Positive`.
/// - `volatility_adj`: An optional `VolatilityAdjustment` which includes base volatility
///   and a standard deviation adjustment.
/// - `trend`: An optional `PriceTrend` providing the annual drift rate and confidence
///   level for the trend.
/// - `expiration_date`: The date to which the probability is calculated, of type `ExpirationDate`.
/// - `risk_free_rate`: An optional risk-free rate (annual), defaulting to zero if not provided.
///
/// # Returns
///
/// Returns a `Result` containing a tuple of two `Positive` values:
/// - `prob_below`: The probability of the stock price being below the target price at expiry.
/// - `prob_above`: The probability of the stock price being above the target price at expiry.
///
/// # Errors
///
/// Returns an error if:
/// - `current_price` is zero, which leaves the log price ratio undefined
///   ([`ProbabilityError::PriceError`] with
///   [`PriceErrorKind::InvalidUnderlyingPrice`]).
/// - `time_to_expiry` is not positive, indicating the expiration date has passed or is invalid.
/// - `volatility_adj.base_volatility` is non-positive.
/// - `trend.confidence` is not between 0 and 1.
/// - the price ratio, its logarithm, or the volatility scaling leaves the
///   representable range ([`ProbabilityError::PositiveError`]).
///
pub fn calculate_single_point_probability(
    current_price: &Positive,
    target_price: &Positive,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: &ExpirationDate,
    risk_free_rate: Option<Decimal>,
) -> Result<(Positive, Positive), ProbabilityError> {
    if *target_price == Positive::ZERO {
        return Ok((Positive::ZERO, Positive::ONE));
    }
    // The log-normal model is built on the ratio `target / current`. A spot of
    // zero has no ratio and no return distribution around it, so it is
    // rejected here rather than dividing by it below.
    if *current_price == Positive::ZERO {
        return Err(ProbabilityError::PriceError(
            PriceErrorKind::InvalidUnderlyingPrice {
                price: 0.0,
                reason: "current price must be strictly positive to form the log price ratio"
                    .to_string(),
            },
        ));
    }
    let time_to_expiry = expiration_date.get_years()?;
    if time_to_expiry <= 0.0 {
        return Err(ProbabilityError::ExpirationError(
            ExpirationErrorKind::InvalidExpiration {
                reason: "Time to expiry must be positive".to_string(),
            },
        ));
    }

    // Get base parameters
    let risk_free = risk_free_rate.unwrap_or(Decimal::ZERO);

    // Calculate adjusted volatility if provided
    let volatility = match volatility_adj {
        Some(adj) => {
            if adj.base_volatility <= Positive::ZERO {
                return Err(ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::VolatilityAdjustmentError {
                        reason: "Base volatility must be positive".to_string(),
                    },
                ));
            }
            adj.base_volatility
                .checked_mul_f64(1.0 + adj.std_dev_adjustment)?
        }
        None => Positive::new_decimal(dec!(0.2)).unwrap_or(Positive::ZERO), // Default volatility if not provided
    };

    // Adjust drift rate based on trend if provided
    let drift_rate = match trend {
        Some(t) => {
            if !(0.0..=1.0).contains(&t.confidence) {
                return Err(ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::TrendError {
                        reason: "Confidence must be between 0 and 1".to_string(),
                    },
                ));
            }
            let rf = risk_free.to_f64().ok_or_else(|| {
                ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::ExpectedValueError {
                        reason: format!(
                            "calculate_single_point_probability: risk_free Decimal {risk_free} not representable as f64"
                        ),
                    },
                )
            })?;
            rf + (t.drift_rate * t.confidence)
        }
        None => risk_free.to_f64().ok_or_else(|| {
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::ExpectedValueError {
                    reason: format!(
                        "calculate_single_point_probability: risk_free Decimal {risk_free} not representable as f64"
                    ),
                },
            )
        })?,
    };

    // Calculate parameters for the log-normal distribution. A spot far below
    // the target overflows the ratio, and a spot far above it rounds the ratio
    // to zero, where the logarithm is undefined; both are reported rather than
    // aborting.
    let log_ratio = target_price.checked_div(current_price)?.checked_ln()?;
    let std_dev = volatility.checked_mul(&p_sqrt(
        &time_to_expiry,
        "analytics::probability::calculate_single_point_probability",
    )?)?;

    // Calculate z-score considering drift
    // `Positive::ln` returns `Decimal` as of positive 0.6: the log of a
    // positive number is not necessarily positive.
    let log_ratio_f = log_ratio.to_f64().ok_or_else(|| {
        ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::ExpectedValueError {
            reason: format!(
                "calculate_single_point_probability: log ratio {log_ratio} not representable as f64"
            ),
        })
    })?;
    let z_score: Decimal = f2du!((log_ratio_f - drift_rate * time_to_expiry) / std_dev)?;

    // Calculate probabilities using the standard normal distribution
    let prob_below: Positive = Positive::new_decimal(big_n(z_score)?).unwrap_or(Positive::ZERO);
    let prob_above: Positive = Positive::new(1.0 - prob_below.to_f64()).unwrap_or(Positive::ZERO);

    Ok((prob_below, prob_above))
}

/// Calculate the probability of the underlying price being in different ranges at expiration
///
/// # Arguments
///
/// * `current_price` - Current price of the underlying asset
/// * `lower_bound` - Lower boundary of the target price range
/// * `upper_bound` - Upper boundary of the target price range
/// * `volatility_adj` - Optional volatility adjustment parameters
/// * `trend` - Optional price trend parameters
/// * `expiration_date` - Expiration date of the analysis
/// * `risk_free_rate` - Optional risk-free rate
///
/// # Returns
///
/// Returns a tuple containing:
/// * Probability of price being below the range
/// * Probability of price being within the range
/// * Probability of price being above the range
///
/// # Errors
///
/// Returns an error if:
/// * Lower bound is greater than upper bound
///   ([`ProbabilityError::PriceError`] with
///   [`PriceErrorKind::InvalidPriceRange`]).
/// * Time to expiry is not positive
/// * Volatility parameters are invalid
/// * Trend confidence is not between 0 and 1
/// * The probability below the upper bound is smaller than below the lower
///   bound ([`ProbabilityError::CalculationError`] with
///   [`ProbabilityCalculationErrorKind::InvalidProbability`]). A distribution
///   function is monotone, so this only happens when the inputs sit past the
///   precision of the price model (a spot near `Positive::MAX` with a
///   volatility around `1e-28`); it is reported rather than floored to zero,
///   which would return a triple that does not sum to one.
pub fn calculate_price_probability(
    current_price: &Positive,
    lower_bound: &Positive,
    upper_bound: &Positive,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: &ExpirationDate,
    risk_free_rate: Option<Decimal>,
) -> Result<(Positive, Positive, Positive), ProbabilityError> {
    if lower_bound > upper_bound {
        return Err(ProbabilityError::PriceError(
            PriceErrorKind::InvalidPriceRange {
                range: format!("lower_bound: {lower_bound} upper_bound: {upper_bound}"),
                reason: "Lower bound must be less than upper bound".to_string(),
            },
        ));
    }

    // Calculate probabilities for the lower bound
    let (prob_below_lower, _) = calculate_single_point_probability(
        current_price,
        lower_bound,
        volatility_adj.clone(),
        trend.clone(),
        expiration_date,
        risk_free_rate,
    )?;

    // Calculate probabilities for the upper bound
    let (prob_below_upper, prob_above_upper) = calculate_single_point_probability(
        current_price,
        upper_bound,
        volatility_adj,
        trend,
        expiration_date,
        risk_free_rate,
    )?;

    // A distribution function is monotone, so `upper >= lower` must give
    // `prob_below_upper >= prob_below_lower`; equality is a zero-width range
    // with probability zero, which `sub_or_none` returns. A smaller
    // probability at the upper bound is a result outside the model's
    // precision, not a property of the range: with a spot near `Positive::MAX`
    // and a volatility of `1e-28`, `(MAX - 4) / MAX` rounds to
    // `0.9999999999999999999999999999` and `Decimal::checked_ln` returns
    // `+9e-28` for it where the true value is `-1e-28`, so the lower bound
    // lands three standard deviations above the spot. Flooring that to zero
    // returned a triple summing to 1.5 instead of 1, so it is reported
    // (#570, same rule as `ProfitLossRange::calculate_probability`, #569).
    let prob_below_range = prob_below_lower;
    let prob_in_range = prob_below_upper
        .sub_or_none(prob_below_lower.to_dec_ref())
        .ok_or_else(|| {
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::InvalidProbability {
                    value: prob_below_upper.to_f64() - prob_below_lower.to_f64(),
                    reason: format!(
                        "probability below the upper bound {upper_bound}                          ({prob_below_upper}) is smaller than below the lower bound                          {lower_bound} ({prob_below_lower}); the inputs are outside                          the precision of the price model"
                    ),
                },
            )
        })?;
    let prob_above_range = prob_above_upper;

    Ok((prob_below_range, prob_in_range, prob_above_range))
}

#[cfg(test)]
mod tests_single_point_probability {
    use super::*;
    use approx::assert_relative_eq;
    use chrono::{Duration, Utc};
    use positive::constants::DAYS_IN_A_YEAR;
    use rust_decimal_macros::dec;

    // Helper function to create default volatility adjustment
    fn default_volatility_adj() -> VolatilityAdjustment {
        VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.2),
            std_dev_adjustment: pos_or_panic!(0.1),
        }
    }

    // Helper function to create default trend
    fn default_trend() -> PriceTrend {
        PriceTrend {
            drift_rate: 0.05,
            confidence: 0.8,
        }
    }

    #[test]
    fn test_basic_calculation_with_days() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);
        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            None,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_calculation_with_datetime() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);
        let expiration_date = Utc::now() + Duration::days(365);

        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            None,
            None,
            &ExpirationDate::DateTime(expiration_date),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_with_volatility_adjustment() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);
        let vol_adj = Some(default_volatility_adj());

        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            vol_adj,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_with_trend() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);
        let trend = Some(default_trend());

        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            None,
            trend,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_with_risk_free_rate() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);

        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            None,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_all_parameters() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);
        let vol_adj = Some(default_volatility_adj());
        let trend = Some(default_trend());

        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            vol_adj,
            trend,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            Some(dec!(0.05)),
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_target_equals_current() {
        let price = Positive::HUNDRED;

        let result = calculate_single_point_probability(
            &price,
            &price,
            Some({
                VolatilityAdjustment {
                    base_volatility: pos_or_panic!(0.8),
                    std_dev_adjustment: Positive::ZERO,
                }
            }),
            Some({
                PriceTrend {
                    drift_rate: 0.0,
                    confidence: 1.0,
                }
            }),
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert_relative_eq!((prob_above + prob_below).to_f64(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(prob_below.to_f64(), 0.5, epsilon = 1e-10);
        assert_relative_eq!(prob_above.to_f64(), 0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_zero_days_to_expiry() {
        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            None,
            None,
            &ExpirationDate::Days(Positive::ZERO),
            None,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ProbabilityError::ExpirationError(ExpirationErrorKind::InvalidExpiration {
                reason,
            }) => {
                assert_eq!(reason, "Time to expiry must be positive");
            }
            _ => panic!("Unexpected error type"),
        };
    }

    #[test]
    fn test_past_datetime() {
        let past_date = Utc::now() - Duration::days(1);

        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            None,
            None,
            &ExpirationDate::DateTime(past_date),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_volatility() {
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: Positive::ZERO,
            std_dev_adjustment: pos_or_panic!(0.1),
        });

        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            vol_adj,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::VolatilityAdjustmentError { reason },
            ) => {
                assert_eq!(reason, "Base volatility must be positive");
            }
            _ => panic!("Unexpected error type"),
        };
    }

    #[test]
    fn test_invalid_trend_confidence() {
        let trend = Some(PriceTrend {
            drift_rate: 0.05,
            confidence: 1.5, // Invalid
        });

        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            None,
            trend,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::TrendError {
                reason,
            }) => {
                assert_eq!(reason, "Confidence must be between 0 and 1");
            }
            _ => panic!("Unexpected error type"),
        };
    }

    #[test]
    fn test_extreme_target_prices() {
        // Test with very high target price
        let result_high = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(1000000.0),
            None,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result_high.is_ok());
        let (_, prob_above) = result_high.unwrap();
        assert!(prob_above < pos_or_panic!(0.01)); // Probability should be very low

        // Test with very low target price
        let result_low = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(0.1),
            None,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result_low.is_ok());
        let (prob_below, prob_above) = result_low.unwrap();
        assert!(prob_above > pos_or_panic!(0.99)); // Probability should be very high
        assert!(prob_below < pos_or_panic!(0.01)); // Probability should be very low
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_extreme_volatility() {
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: Positive::ONE,
            std_dev_adjustment: pos_or_panic!(5.0),
        });

        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            vol_adj,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_extreme_trend() {
        let trend = Some(PriceTrend {
            drift_rate: 2.0, // 200% annual drift
            confidence: 0.99,
        });

        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            None,
            trend,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!((prob_below + prob_above).to_f64(), 1.0, epsilon = 1e-10);
    }
}

#[cfg(test)]
mod tests_calculate_price_probability {
    use super::*;
    use approx::assert_relative_eq;
    use positive::constants::DAYS_IN_A_YEAR;

    #[test]
    fn test_price_probability_basic() {
        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(95.0),
            &pos_or_panic!(105.0),
            None,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_in_range, prob_above) = result.unwrap();
        assert!(prob_below >= Positive::ZERO && prob_above <= Positive::ONE);
        assert!(prob_in_range >= Positive::ZERO && prob_in_range <= Positive::ONE);
        assert!(prob_above >= Positive::ZERO && prob_above <= Positive::ONE);
        assert_relative_eq!(
            (prob_below + prob_in_range + prob_above).to_f64(),
            1.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn test_price_probability_invalid_bounds() {
        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0), // Lower bound higher than upper bound
            &pos_or_panic!(95.0),
            None,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ProbabilityError::PriceError(PriceErrorKind::InvalidPriceRange { range, reason }) => {
                assert_eq!(range, "lower_bound: 105 upper_bound: 95");
                assert_eq!(reason, "Lower bound must be less than upper bound");
            }
            _ => panic!("Unexpected error type"),
        };
    }

    #[test]
    fn test_price_probability_with_volatility() {
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.5),
            std_dev_adjustment: Positive::ZERO,
        });

        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(90.0),
            &pos_or_panic!(110.0),
            vol_adj,
            None,
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_in_range, prob_above) = result.unwrap();
        assert_relative_eq!(
            (prob_below + prob_in_range + prob_above).to_f64(),
            1.0,
            epsilon = 1e-10
        );
    }
}

#[cfg(test)]
mod tests_probability_inversion {
    use super::*;
    use crate::ExpirationDate;
    use rust_decimal::Decimal;

    /// The `(below, in, above)` triple is a partition of the outcome space, so
    /// it sums to one or the call fails. Before #570 the middle term was
    /// floored to zero on an inverted CDF difference and this input returned
    /// `(1, 0, 0.5)`, a triple summing to 1.5 that no distribution produces.
    ///
    /// Reaching it needs a spot near `Positive::MAX` with a volatility around
    /// `1e-28`: `(MAX - 4) / MAX` rounds to `0.9999999999999999999999999999`
    /// at `Decimal`'s twenty-eight places and `checked_ln` returns `+9e-28`
    /// for it where the true value is `-1e-28`, which puts the lower bound
    /// three standard deviations above the spot.
    #[test]
    fn test_inverted_cdf_difference_is_reported_not_floored() {
        let lower = Positive::MAX - Decimal::from(4);
        let result = calculate_price_probability(
            &Positive::MAX,
            &lower,
            &Positive::MAX,
            Some(VolatilityAdjustment {
                base_volatility: Positive::new(1e-28)
                    .expect("1e-28 is a representable positive volatility"),
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            &ExpirationDate::Days(pos_or_panic!(365.0)),
            None,
        );

        match result {
            Err(ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::InvalidProbability { .. },
            )) => {}
            other => panic!("expected InvalidProbability, got {other:?}"),
        }
    }

    /// The same shape at a volatility the model can still represent: the
    /// inversion is smaller (the triple used to sum to 1.0000007) but it is
    /// the same defect and gets the same answer.
    #[test]
    fn test_near_precision_inversion_is_reported() {
        let lower = Positive::MAX - Decimal::from(4);
        let result = calculate_price_probability(
            &Positive::MAX,
            &lower,
            &Positive::MAX,
            Some(VolatilityAdjustment {
                base_volatility: Positive::new(1e-20)
                    .expect("1e-20 is a representable positive volatility"),
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            &ExpirationDate::Days(Positive::ONE),
            None,
        );

        assert!(
            result.is_err(),
            "an inverted CDF difference must be reported, got {result:?}"
        );
    }

    /// Equality is a value, not an error: coincident bounds are a zero-width
    /// range whose probability is zero, and the triple still sums to one.
    #[test]
    fn test_zero_width_range_is_zero_and_the_triple_sums_to_one() {
        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &Positive::HUNDRED,
            &Positive::HUNDRED,
            None,
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        let (below, inside, above) = match result {
            Ok(triple) => triple,
            other => panic!("a zero-width range is a value, got {other:?}"),
        };
        assert_eq!(inside, Positive::ZERO);
        assert_eq!(
            below.to_dec() + inside.to_dec() + above.to_dec(),
            Decimal::ONE,
        );
    }

    /// An ordinary range: the partition property holds on the inputs callers
    /// actually pass, which is what makes the error path unreachable for them.
    #[test]
    fn test_ordinary_range_triple_sums_to_one() {
        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(95.0),
            &pos_or_panic!(105.0),
            Some(VolatilityAdjustment {
                base_volatility: pos_or_panic!(0.2),
                std_dev_adjustment: pos_or_panic!(0.1),
            }),
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        let (below, inside, above) = match result {
            Ok(triple) => triple,
            other => panic!("an ordinary range must succeed, got {other:?}"),
        };
        assert!(inside > Positive::ZERO);
        assert_eq!(
            below.to_dec() + inside.to_dec() + above.to_dec(),
            Decimal::ONE,
        );
    }
}
