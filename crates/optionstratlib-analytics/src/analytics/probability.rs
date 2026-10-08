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
use num_traits::ToPrimitive;
use optionstratlib_core::f2du;
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{decimal_to_f64, p_sqrt};
#[cfg(test)]
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::greeks::big_n;
use rust_decimal::Decimal;

/// Volatility used by the probability kernels.
///
/// The kernels price at `base_volatility * (1 + std_dev_adjustment)`. They
/// have no default: a caller that wants the volatility of a strategy asks the
/// strategy for it (`ProbabilityAnalysis::reference_volatility`) rather than
/// letting the kernel guess one.
#[derive(Debug, Clone, Copy)]
pub struct VolatilityAdjustment {
    /// Annualized base volatility, as a fraction (`0.2` is 20%). Must be
    /// strictly positive.
    pub base_volatility: Positive,
    /// Relative widening applied on top of the base: `0` leaves it unchanged,
    /// `0.1` widens it by 10%.
    pub std_dev_adjustment: Positive,
}

/// The 0.2 volatility the kernels used to substitute when given `None`;
/// tests that relied on that default now ask for it explicitly, so the numbers
/// they pin are unchanged and the dependency is visible.
#[cfg(test)]
pub(crate) fn flat_volatility_0_2() -> VolatilityAdjustment {
    VolatilityAdjustment {
        base_volatility: Positive::new_decimal(rust_decimal::Decimal::new(2, 1))
            .unwrap_or(Positive::ONE),
        std_dev_adjustment: Positive::ZERO,
    }
}

/// Price trend applied on top of the risk-free drift by the probability
/// kernels.
///
/// The kernels drift the price at `risk_free_rate + drift_rate * confidence`.
/// That is the arithmetic drift `mu` (the expected return of the price), not
/// the drift of its logarithm: the kernels apply the lognormal convexity
/// correction themselves and drift `ln S` at `mu - sigma^2 / 2`, so a
/// `drift_rate` read off as an expected return is passed as is (#664).
///
/// Both fields are dimensionless and private: [`PriceTrend::new`] is the
/// only way to build one, so a `confidence` outside `[0, 1]` cannot reach a
/// kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceTrend {
    /// Annual drift rate, as a fraction (`0.1` is 10%); positive for an
    /// upward trend, negative for a downward one.
    drift_rate: Decimal,
    /// Weight given to the drift, in `[0, 1]`.
    confidence: Decimal,
}

impl PriceTrend {
    /// Builds a trend from an annual drift rate and the confidence in it.
    ///
    /// # Arguments
    ///
    /// * `drift_rate` - Annual drift rate as a fraction (`0.1` is 10%); any
    ///   sign.
    /// * `confidence` - Weight given to the drift, from `0` (ignored) to `1`
    ///   (applied in full), both ends included.
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::CalculationError`] with
    /// [`ProbabilityCalculationErrorKind::TrendError`] when `confidence` is
    /// below `0` or above `1`.
    pub fn new(drift_rate: Decimal, confidence: Decimal) -> Result<Self, ProbabilityError> {
        if !(Decimal::ZERO..=Decimal::ONE).contains(&confidence) {
            return Err(invalid_confidence(confidence));
        }
        Ok(Self {
            drift_rate,
            confidence,
        })
    }

    /// Annual drift rate, as a fraction; may be negative.
    #[must_use]
    #[inline]
    pub fn drift_rate(&self) -> Decimal {
        self.drift_rate
    }

    /// Weight given to the drift, in `[0, 1]`.
    #[must_use]
    #[inline]
    pub fn confidence(&self) -> Decimal {
        self.confidence
    }
}

#[cold]
#[inline(never)]
fn invalid_confidence(confidence: Decimal) -> ProbabilityError {
    ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::TrendError {
        reason: format!("Confidence must be between 0 and 1, got {confidence}"),
    })
}

/// Converts a trend field to the `f64` nearest to it for the kernel, so a
/// field written with the digits of a former `f64` literal gives back that
/// literal exactly; a value with no `f64` form is reported.
fn trend_field_to_f64(name: &str, value: Decimal) -> Result<f64, ProbabilityError> {
    decimal_to_f64(value).map_err(|e| {
        ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::TrendError {
            reason: format!("trend {name} {value} has no f64 form: {e}"),
        })
    })
}

/// Calculates the probability of a stock price reaching a target price within a given timeframe.
///
/// This function estimates the probability of a stock following a log-normal distribution
/// to reach a specified target price before expiration. It also provides the probability
/// of the stock price being below or above the target price at the expiration date.
///
/// With `sigma` the adjusted volatility, `T` the years to expiry and `mu` the
/// arithmetic drift (`risk_free_rate`, plus `drift_rate * confidence` when a
/// `trend` is given),
///
/// ```text
/// z = (ln(K / S) - (mu - sigma^2 / 2) * T) / (sigma * sqrt(T))
/// P(S_T < K) = N(z)        P(S_T >= K) = 1 - N(z)
/// ```
///
/// so with no trend `z = -d2` and `P(S_T < K) = N(-d2)`, the risk-neutral
/// probability that a call at `K` expires out of the money (Hull, *Options,
/// Futures, and Other Derivatives*, ch. 15).
///
/// # Parameters
///
/// - `current_price`: The current stock price, represented as a `Positive`.
/// - `target_price`: The target stock price to evaluate, represented as a `Positive`.
/// - `volatility`: The `VolatilityAdjustment` to price at. Required: the kernel
///   has no default volatility.
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
/// - `volatility.base_volatility` is non-positive
///   ([`ProbabilityCalculationErrorKind::VolatilityAdjustmentError`]).
/// - a `trend` field has no `f64` form
///   ([`ProbabilityCalculationErrorKind::TrendError`]); its confidence is
///   already in `[0, 1]`, checked by [`PriceTrend::new`].
/// - the price ratio, its logarithm, or the volatility scaling leaves the
///   representable range ([`ProbabilityError::PositiveError`]).
///
pub fn calculate_single_point_probability(
    current_price: &Positive,
    target_price: &Positive,
    volatility: VolatilityAdjustment,
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

    if volatility.base_volatility <= Positive::ZERO {
        return Err(ProbabilityError::CalculationError(
            ProbabilityCalculationErrorKind::VolatilityAdjustmentError {
                reason: "Base volatility must be positive".to_string(),
            },
        ));
    }
    let volatility = volatility
        .base_volatility
        .checked_mul_f64(1.0 + volatility.std_dev_adjustment.to_f64())?;

    // Adjust drift rate based on trend if provided
    let drift_rate = match trend {
        Some(t) => {
            // `PriceTrend::new` already holds the confidence in [0, 1]. Each
            // field converts to its nearest `f64` and the product is taken in
            // `f64`, as it always was, so a trend written with the digits of
            // the former `f64` fields drifts the distribution by the same
            // `f64`.
            let trend_drift = trend_field_to_f64("drift_rate", t.drift_rate)?;
            let confidence = trend_field_to_f64("confidence", t.confidence)?;
            let rf = risk_free.to_f64().ok_or_else(|| {
                ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::ExpectedValueError {
                        reason: format!(
                            "calculate_single_point_probability: risk_free Decimal {risk_free} not representable as f64"
                        ),
                    },
                )
            })?;
            rf + (trend_drift * confidence)
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
    // `drift_rate` is the arithmetic drift `mu` of the price, so the log price
    // drifts at `mu - sigma^2 / 2` (Ito). With `mu = r` the threshold is
    // `-d2` and `P(S_T < K) = N(-d2)` (Hull, ch. 15); omitting the convexity
    // term understated it for every `sigma > 0` (#664).
    let sigma = volatility.to_f64();
    let log_drift = drift_rate - sigma * sigma / 2.0;
    let z_score: Decimal =
        f2du!((log_ratio_f - log_drift * time_to_expiry.to_f64()) / std_dev.to_f64())?;

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
/// * `volatility` - Volatility to price at (required; see [`VolatilityAdjustment`])
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
/// * A `trend` field has no `f64` form
///   ([`ProbabilityCalculationErrorKind::TrendError`])
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
    volatility: VolatilityAdjustment,
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
        volatility,
        trend.clone(),
        expiration_date,
        risk_free_rate,
    )?;

    // Calculate probabilities for the upper bound
    let (prob_below_upper, prob_above_upper) = calculate_single_point_probability(
        current_price,
        upper_bound,
        volatility,
        trend,
        expiration_date,
        risk_free_rate,
    )?;

    // A distribution function is monotone, so `upper >= lower` must give
    // `prob_below_upper >= prob_below_lower`; equality carries zero mass,
    // which `sub_or_none` returns. Equality does not mean the bounds
    // coincide: two distinct bounds far into a tail can round to the same
    // `Decimal`, and zero is still the right mass for them. A smaller
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

    // Helper function to create default volatility adjustment
    fn default_volatility_adj() -> VolatilityAdjustment {
        VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.2),
            std_dev_adjustment: pos_or_panic!(0.1),
        }
    }

    // Helper function to create default trend
    fn default_trend() -> PriceTrend {
        price_trend(dec!(0.05), dec!(0.8))
    }

    #[test]
    fn test_basic_calculation_with_days() {
        let current_price = Positive::HUNDRED;
        let target_price = pos_or_panic!(105.0);
        let result = calculate_single_point_probability(
            &current_price,
            &target_price,
            flat_volatility_0_2(),
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
            flat_volatility_0_2(),
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
        let vol_adj = default_volatility_adj();

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
            flat_volatility_0_2(),
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
            flat_volatility_0_2(),
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
        let vol_adj = default_volatility_adj();
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
            {
                VolatilityAdjustment {
                    base_volatility: pos_or_panic!(0.8),
                    std_dev_adjustment: Positive::ZERO,
                }
            },
            Some(price_trend(dec!(0.0), dec!(1.0))),
            &ExpirationDate::Days(DAYS_IN_A_YEAR),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert_relative_eq!((prob_above + prob_below).to_f64(), 1.0, epsilon = 1e-10);
        // At the money with zero drift the log price drifts at
        // `-sigma^2 / 2`, so `P(S_T < S) = N(sigma sqrt(T) / 2) = N(0.4)`, not
        // one half: the median of a lognormal price sits below its mean.
        // #664: 0.5 -> 0.655421741610324 (below), 0.5 -> 0.344578258389676
        // (above).
        assert_relative_eq!(prob_below.to_f64(), 0.655_421_741_610_324, epsilon = 1e-10);
        assert_relative_eq!(prob_above.to_f64(), 0.344_578_258_389_676, epsilon = 1e-10);
    }

    #[test]
    fn test_zero_days_to_expiry() {
        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            flat_volatility_0_2(),
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
            flat_volatility_0_2(),
            None,
            &ExpirationDate::DateTime(past_date),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_volatility() {
        let vol_adj = VolatilityAdjustment {
            base_volatility: Positive::ZERO,
            std_dev_adjustment: pos_or_panic!(0.1),
        };

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
        // The confidence is checked where the trend is built, so an invalid
        // one never reaches the kernel.
        match PriceTrend::new(dec!(0.05), dec!(1.5)) {
            Err(ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::TrendError { reason },
            )) => {
                assert_eq!(reason, "Confidence must be between 0 and 1, got 1.5");
            }
            other => panic!("Unexpected result: {other:?}"),
        };
    }

    #[test]
    fn test_extreme_target_prices() {
        // Test with very high target price
        let result_high = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(1000000.0),
            flat_volatility_0_2(),
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
            flat_volatility_0_2(),
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
        let vol_adj = VolatilityAdjustment {
            base_volatility: Positive::ONE,
            std_dev_adjustment: pos_or_panic!(5.0),
        };

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
        let trend = Some(price_trend(
            dec!(2.0), // 200% annual drift
            dec!(0.99),
        ));

        let result = calculate_single_point_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(105.0),
            flat_volatility_0_2(),
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
    use optionstratlib_core::constants::DAYS_IN_A_YEAR;

    #[test]
    fn test_price_probability_basic() {
        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(95.0),
            &pos_or_panic!(105.0),
            flat_volatility_0_2(),
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
            flat_volatility_0_2(),
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
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.5),
            std_dev_adjustment: Positive::ZERO,
        };

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
    use optionstratlib_core::model::ExpirationDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

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
            VolatilityAdjustment {
                base_volatility: Positive::new(1e-28)
                    .expect("1e-28 is a representable positive volatility"),
                std_dev_adjustment: Positive::ZERO,
            },
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
            VolatilityAdjustment {
                base_volatility: Positive::new(1e-20)
                    .expect("1e-20 is a representable positive volatility"),
                std_dev_adjustment: Positive::ZERO,
            },
            None,
            &ExpirationDate::Days(Positive::ONE),
            None,
        );

        assert!(
            result.is_err(),
            "an inverted CDF difference must be reported, got {result:?}"
        );
    }

    /// Equal CDF values are a value, not an error: the mass between the
    /// bounds is zero and the triple still sums to one. Coincident bounds are
    /// the clearest way to produce that, but not the only one, since two
    /// distinct bounds far into a tail can round to the same `Decimal`.
    #[test]
    fn test_equal_cdf_values_give_zero_mass_and_the_triple_sums_to_one() {
        let result = calculate_price_probability(
            &Positive::HUNDRED,
            &Positive::HUNDRED,
            &Positive::HUNDRED,
            flat_volatility_0_2(),
            None,
            &ExpirationDate::Days(pos_or_panic!(30.0)),
            Some(dec!(0.05)),
        );

        let (below, inside, above) = match result {
            Ok(triple) => triple,
            other => panic!("equal CDF values are a value, got {other:?}"),
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
            VolatilityAdjustment {
                base_volatility: pos_or_panic!(0.2),
                std_dev_adjustment: pos_or_panic!(0.1),
            },
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

#[cfg(test)]
mod tests_price_trend {
    use super::*;
    use rust_decimal_macros::dec;

    fn price_trend(drift_rate: Decimal, confidence: Decimal) -> PriceTrend {
        match PriceTrend::new(drift_rate, confidence) {
            Ok(trend) => trend,
            Err(e) => panic!("valid trend: {e}"),
        }
    }

    fn trend_reason(result: Result<PriceTrend, ProbabilityError>) -> String {
        match result {
            Err(ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::TrendError { reason },
            )) => reason,
            other => panic!("expected a trend error, got {other:?}"),
        }
    }

    fn below_above(trend: Option<PriceTrend>, target: Positive) -> (Positive, Positive) {
        let volatility = VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.1),
        };
        match calculate_single_point_probability(
            &Positive::HUNDRED,
            &target,
            volatility,
            trend,
            &ExpirationDate::Days(pos_or_panic!(45.0)),
            Some(dec!(0.03)),
        ) {
            Ok(pair) => pair,
            Err(e) => panic!("kernel evaluates an ordinary input: {e}"),
        }
    }

    fn in_range(trend: Option<PriceTrend>) -> (Positive, Positive, Positive) {
        let volatility = VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.1),
        };
        match calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(92.5),
            &pos_or_panic!(110.0),
            volatility,
            trend,
            &ExpirationDate::Days(pos_or_panic!(45.0)),
            Some(dec!(0.03)),
        ) {
            Ok(triple) => triple,
            Err(e) => panic!("kernel evaluates an ordinary input: {e}"),
        }
    }

    #[test]
    fn test_price_trend_new_confidence_bounds_accepted() {
        let none = price_trend(dec!(-0.25), Decimal::ZERO);
        assert_eq!(none.drift_rate(), dec!(-0.25));
        assert_eq!(none.confidence(), Decimal::ZERO);

        let full = price_trend(dec!(0.10), Decimal::ONE);
        assert_eq!(full.drift_rate(), dec!(0.10));
        assert_eq!(full.confidence(), Decimal::ONE);
    }

    #[test]
    fn test_price_trend_new_confidence_below_zero_rejected() {
        assert_eq!(
            trend_reason(PriceTrend::new(dec!(0.10), dec!(-0.01))),
            "Confidence must be between 0 and 1, got -0.01"
        );
    }

    #[test]
    fn test_price_trend_new_confidence_above_one_rejected() {
        assert_eq!(
            trend_reason(PriceTrend::new(dec!(0.10), dec!(1.01))),
            "Confidence must be between 0 and 1, got 1.01"
        );
    }

    #[test]
    fn test_price_trend_new_any_drift_sign_accepted() {
        for drift in [Decimal::MIN, dec!(-2), Decimal::ZERO, dec!(2), Decimal::MAX] {
            assert_eq!(price_trend(drift, dec!(0.5)).drift_rate(), drift);
        }
    }

    /// Pinned from the `f64`-field kernel (`PriceTrend { drift_rate: -0.37,
    /// confidence: 0.65 }`) before the fields became `Decimal` (#656), then
    /// re-baselined when the threshold gained the lognormal `-sigma^2 / 2`
    /// term (#664); each value agrees with the closed-form `N(z)` to `1e-11`.
    #[test]
    fn test_price_trend_kernel_output_unchanged_matches_f64_fields() {
        let trend = || Some(price_trend(dec!(-0.37), dec!(0.65)));

        // #664: 0.895412344777716 -> 0.903902952229174 and
        // 0.104587655222284 -> 0.096097047770826.
        let (below, above) = below_above(trend(), pos_or_panic!(110.0));
        assert_eq!(below.to_dec(), dec!(0.903902952229174));
        assert_eq!(above.to_dec(), dec!(0.096097047770826));

        // #664: 0.2950713010903496 -> 0.3119431542059403,
        // 0.6003410436873664 -> 0.5919597980232337 and
        // 0.104587655222284 -> 0.096097047770826.
        let (below, inside, above) = in_range(trend());
        assert_eq!(below.to_dec(), dec!(0.3119431542059403));
        assert_eq!(inside.to_dec(), dec!(0.5919597980232337));
        assert_eq!(above.to_dec(), dec!(0.096097047770826));

        // The trend moves the result: without it the same inputs give a
        // different probability, so the pin above exercises the drift.
        // #664: 0.82862940208696 -> 0.840628037745865.
        let (below_no_trend, _) = below_above(None, pos_or_panic!(110.0));
        assert_eq!(below_no_trend.to_dec(), dec!(0.840628037745865));
    }

    /// Fields with 15 and 28 decimal places, where `Decimal::to_f64` lands one
    /// ULP away from the nearest `f64`. Pinned from the `f64`-field kernel fed
    /// the nearest `f64`s (`drift_rate: 2.999789999999902`, `confidence:
    /// 0.12345678901234568`), then re-baselined when the threshold gained the
    /// lognormal `-sigma^2 / 2` term (#664).
    #[test]
    fn test_price_trend_kernel_many_decimal_places_matches_f64_fields() {
        let trend = || {
            Some(price_trend(
                dec!(2.999789999999902),
                dec!(0.1234567890123456789012345678),
            ))
        };

        // #664: 0.682928086717717 -> 0.699924018145762 and
        // 0.317071913282283 -> 0.300075981854238.
        let (below, above) = below_above(trend(), pos_or_panic!(110.0));
        assert_eq!(below.to_dec(), dec!(0.699924018145762));
        assert_eq!(above.to_dec(), dec!(0.300075981854238));

        // #664: 0.0936575883560122 -> 0.10199178784555,
        // 0.5892704983617048 -> 0.597932230300212 and
        // 0.317071913282283 -> 0.300075981854238.
        let (below, inside, above) = in_range(trend());
        assert_eq!(below.to_dec(), dec!(0.10199178784555));
        assert_eq!(inside.to_dec(), dec!(0.597932230300212));
        assert_eq!(above.to_dec(), dec!(0.300075981854238));
    }
}

/// The single-point kernel against the closed-form lognormal threshold
/// (#664): with no trend `P(S_T < K) = N(-d2)`, with
/// `d2 = (ln(S / K) + (r - sigma^2 / 2) T) / (sigma sqrt(T))` (Hull,
/// *Options, Futures, and Other Derivatives*, ch. 15). The references are
/// `0.5 * erfc(d2 / sqrt(2))` evaluated in double precision outside the
/// library.
#[cfg(test)]
mod tests_lognormal_threshold {
    use super::*;
    use rust_decimal_macros::dec;

    /// Absolute agreement with the reference. The kernel's normal CDF
    /// (`big_n`, through `statrs`) is accurate to about `2e-11` in this range:
    /// `big_n(0.807915)` returns `0.790430242158675` where the CDF is
    /// `0.790430242178701`. The threshold itself is exact to the last digits
    /// of the `Decimal` inputs, so `1e-10` still separates the corrected
    /// threshold from the uncorrected one by seven orders of magnitude (the
    /// smallest gap in the table below is `7.4e-4`).
    const REFERENCE_TOLERANCE: f64 = 1e-10;

    fn volatility(sigma: f64) -> VolatilityAdjustment {
        VolatilityAdjustment {
            base_volatility: pos_or_panic!(sigma),
            std_dev_adjustment: Positive::ZERO,
        }
    }

    fn single_point(
        spot: f64,
        strike: f64,
        sigma: f64,
        days: f64,
        rate: Decimal,
        trend: Option<PriceTrend>,
    ) -> (Positive, Positive) {
        match calculate_single_point_probability(
            &pos_or_panic!(spot),
            &pos_or_panic!(strike),
            volatility(sigma),
            trend,
            &ExpirationDate::Days(pos_or_panic!(days)),
            Some(rate),
        ) {
            Ok(pair) => pair,
            Err(e) => panic!("kernel evaluates an ordinary input: {e}"),
        }
    }

    fn assert_close(actual: Positive, expected: f64, case: &str) {
        let diff = (actual.to_f64() - expected).abs();
        assert!(
            diff < REFERENCE_TOLERANCE,
            "{case}: got {actual}, N(-d2) = {expected}, diff {diff:e}"
        );
    }

    /// `(spot, strike, sigma, days, rate, N(-d2))`; the first row is Hull's
    /// worked example (S = 42, K = 40, r = 10%, sigma = 20%, T = 0.5,
    /// d2 = 0.6278).
    const CASES: [(f64, f64, f64, f64, Decimal, f64); 6] = [
        (42.0, 40.0, 0.2, 182.5, dec!(0.1), 0.265_053_963_154_091_47),
        (100.0, 105.0, 0.2, 30.0, dec!(0.05), 0.790_430_264_386_178_2),
        (
            100.0,
            100.0,
            0.3,
            365.0,
            dec!(0.05),
            0.493_351_269_806_317_44,
        ),
        (100.0, 90.0, 0.5, 730.0, dec!(0.02), 0.558_821_741_169_794_4),
        (50.0, 60.0, 0.15, 91.25, dec!(0.0), 0.993_215_097_131_548_5),
        (100.0, 105.0, 0.8, 365.0, dec!(0.0), 0.677_596_286_987_641_4),
    ];

    #[test]
    fn test_single_point_no_trend_matches_n_minus_d2() {
        for (spot, strike, sigma, days, rate, expected) in CASES {
            let (below, _) = single_point(spot, strike, sigma, days, rate, None);
            let case = format!("S={spot} K={strike} sigma={sigma} days={days} r={rate}");
            assert_close(below, expected, &case);
        }
    }

    /// The two tails partition the outcome space: `P(S_T < K) +
    /// P(S_T >= K) = 1`, and the upper tail is `N(d2)`.
    #[test]
    fn test_single_point_tails_sum_to_one_and_upper_is_n_d2() {
        for (spot, strike, sigma, days, rate, expected) in CASES {
            let (below, above) = single_point(spot, strike, sigma, days, rate, None);
            let case = format!("S={spot} K={strike} sigma={sigma} days={days} r={rate}");
            assert!(
                (below.to_f64() + above.to_f64() - 1.0).abs() < REFERENCE_TOLERANCE,
                "{case}: {below} + {above} != 1"
            );
            assert_close(above, 1.0 - expected, &case);
        }
    }

    /// The range kernel is the difference of the two single-point
    /// probabilities, so it inherits the corrected threshold: the mass in
    /// `[90, 110)` is `N(-d2(110)) - N(-d2(90))`.
    #[test]
    fn test_price_probability_range_matches_n_minus_d2_difference() {
        // S = 100, sigma = 0.25, T = 0.5, r = 0.03.
        let below_90 = 0.276_766_764_798_688_95;
        let below_110 = 0.706_328_827_515_829_9;
        let (below, inside, above) = match calculate_price_probability(
            &Positive::HUNDRED,
            &pos_or_panic!(90.0),
            &pos_or_panic!(110.0),
            volatility(0.25),
            None,
            &ExpirationDate::Days(pos_or_panic!(182.5)),
            Some(dec!(0.03)),
        ) {
            Ok(triple) => triple,
            Err(e) => panic!("kernel evaluates an ordinary input: {e}"),
        };
        assert_close(below, below_90, "below 90");
        assert_close(inside, below_110 - below_90, "in [90, 110)");
        assert_close(above, 1.0 - below_110, "above 110");
        assert_eq!(
            below.to_dec() + inside.to_dec() + above.to_dec(),
            Decimal::ONE
        );
    }

    /// The trend drift is an arithmetic drift `mu`, corrected by
    /// `-sigma^2 / 2` like the risk-free rate: a full-confidence trend of
    /// `mu` over a zero rate gives the same threshold as a rate of `mu` and no
    /// trend, `N(-d2)` evaluated at `r = mu`.
    #[test]
    fn test_single_point_trend_drift_is_arithmetic_and_convexity_corrected() {
        let trend =
            PriceTrend::new(dec!(0.1), Decimal::ONE).unwrap_or_else(|e| panic!("valid trend: {e}"));
        let (with_trend, _) = single_point(42.0, 40.0, 0.2, 182.5, dec!(0.0), Some(trend));
        let (with_rate, _) = single_point(42.0, 40.0, 0.2, 182.5, dec!(0.1), None);
        assert_eq!(with_trend, with_rate);
        assert_close(with_trend, 0.265_053_963_154_091_47, "trend mu = 0.1");

        // A trend held with zero confidence leaves the risk-neutral threshold.
        let ignored = PriceTrend::new(dec!(0.4), Decimal::ZERO)
            .unwrap_or_else(|e| panic!("valid trend: {e}"));
        let (with_ignored, _) = single_point(42.0, 40.0, 0.2, 182.5, dec!(0.1), Some(ignored));
        assert_eq!(with_ignored, with_rate);
    }

    /// At the money with `r = sigma^2 / 2` the log drift is zero, so the
    /// median of `S_T` is the spot and the probability is exactly one half;
    /// the uncorrected threshold put it at `N(-sigma sqrt(T) / 2)`.
    #[test]
    fn test_single_point_zero_log_drift_at_the_money_is_one_half() {
        let (below, above) = single_point(100.0, 100.0, 0.2, 365.0, dec!(0.02), None);
        assert_close(below, 0.5, "below");
        assert_close(above, 0.5, "above");
    }
}
