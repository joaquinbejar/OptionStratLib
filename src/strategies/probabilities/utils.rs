/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 30/11/24
 ******************************************************************************/
use crate::greeks::utils::{big_n};
use crate::model::types::{PositiveF64, PZERO, ExpirationDate};
use crate::pos;

/// Struct to hold volatility adjustment parameters
#[derive(Debug, Clone)]
pub struct VolatilityAdjustment {
    /// Base volatility
    pub base_volatility: PositiveF64,
    /// Number of standard deviations to adjust
    pub std_dev_adjustment: PositiveF64,
}

/// Struct to hold price trend parameters
#[derive(Debug, Clone)]
pub struct PriceTrend {
    /// Annual drift rate (positive for upward trend, negative for downward)
    pub drift_rate: f64,
    /// Confidence level for the trend (0 to 1)
    pub confidence: f64,
}


/// Calculate the probability of the underlying price being within a specific range at expiration
/// using log-normal transformation for more symmetric distribution
pub fn calculate_single_point_probability(
    current_price: PositiveF64,
    target_price: PositiveF64,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: ExpirationDate,
    risk_free_rate: Option<f64>,
) -> Result<(PositiveF64, PositiveF64), String> {
    let time_to_expiry = expiration_date.get_years();
    if time_to_expiry <= 0.0 {
        return Err("Time to expiry must be positive".to_string());
    }

    // Get base parameters
    let risk_free = risk_free_rate.unwrap_or(0.0);

    // Calculate adjusted volatility if provided
    let volatility = match volatility_adj {
        Some(adj) => {
            if adj.base_volatility <= PZERO {
                return Err("Base volatility must be positive".to_string());
            }
            adj.base_volatility * (1.0 + adj.std_dev_adjustment)
        }
        None => pos!(0.2), // Default volatility if not provided
    };

    // Adjust drift rate based on trend if provided
    let drift_rate = match trend {
        Some(t) => {
            if !(0.0..=1.0).contains(&t.confidence) {
                return Err("Confidence must be between 0 and 1".to_string());
            }
            risk_free + (t.drift_rate * t.confidence)
        }
        None => risk_free,
    };
    
    // Calculate parameters for the log-normal distribution
    let log_ratio = (target_price / current_price).value().ln();
    let std_dev = volatility.value() * time_to_expiry.sqrt();

    // Calculate z-score considering drift
    let z_score = (log_ratio - drift_rate * time_to_expiry) / std_dev;

    // Calculate probabilities using the standard normal distribution
    let prob_below = pos!(big_n(z_score));
    let prob_above = pos!(1.0 - prob_below);

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
/// * Time to expiry is not positive
/// * Volatility parameters are invalid
/// * Trend confidence is not between 0 and 1
pub fn calculate_price_probability(
    current_price: PositiveF64,
    lower_bound: PositiveF64,
    upper_bound: PositiveF64,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: ExpirationDate,
    risk_free_rate: Option<f64>,
) -> Result<(PositiveF64, PositiveF64, PositiveF64), String> {
    if lower_bound > upper_bound {
        return Err("Lower bound must be less than upper bound".to_string());
    }

    // Calculate probabilities for the lower bound
    let (prob_below_lower, _) = calculate_single_point_probability(
        current_price,
        lower_bound,
        volatility_adj.clone(),
        trend.clone(),
        expiration_date.clone(),
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

    // Calculate the three required probabilities
    let prob_below_range = prob_below_lower;
    let prob_in_range = prob_below_upper - prob_below_lower;
    let prob_above_range = prob_above_upper;

    Ok((prob_below_range, prob_in_range, prob_above_range))
}


/// Calculate probabilities for multiple price ranges at expiration
///
/// # Arguments
///
/// * `current_price` - Current price of the underlying asset
/// * `bounds` - Vector of price boundaries in ascending order
/// * `volatility_adj` - Optional volatility adjustment parameters
/// * `trend` - Optional price trend parameters
/// * `expiration_date` - Expiration date of the analysis
/// * `risk_free_rate` - Optional risk-free rate
///
/// # Returns
///
/// Returns a vector of probabilities where:
/// * First element is the probability of price being below the first bound
/// * Middle elements are probabilities of price being between consecutive bounds
/// * Last element is the probability of price being above the last bound
///
/// The returned vector will have length = bounds.length + 1
///
/// # Errors
///
/// Returns an error if:
/// * Bounds vector is empty
/// * Bounds are not in ascending order
/// * Time to expiry is not positive
/// * Volatility parameters are invalid
/// * Trend confidence is not between 0 and 1
pub fn calculate_bounds_probability(
    current_price: PositiveF64,
    bounds: Vec<PositiveF64>,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: ExpirationDate,
    risk_free_rate: Option<f64>,
) -> Result<Vec<PositiveF64>, String> {
    // Check if bounds vector is empty
    if bounds.is_empty() {
        return Err("Bounds vector cannot be empty".to_string());
    }

    // Check if bounds are in ascending order
    for i in 1..bounds.len() {
        if bounds[i] <= bounds[i-1] {
            return Err("Bounds must be in ascending order".to_string());
        }
    }

    // Calculate probabilities for each bound
    let mut bound_probs = Vec::with_capacity(bounds.len());
    for bound in &bounds {
        let (prob_below, _) = calculate_single_point_probability(
            current_price,
            *bound,
            volatility_adj.clone(),
            trend.clone(),
            expiration_date.clone(),
            risk_free_rate,
        )?;
        bound_probs.push(prob_below);
    }

    // Calculate range probabilities
    let mut range_probs = Vec::with_capacity(bounds.len() + 1);

    // Probability below first bound
    range_probs.push(bound_probs[0]);

    // Probabilities between bounds
    for i in 1..bound_probs.len() {
        range_probs.push(bound_probs[i] - bound_probs[i-1]);
    }

    // Probability above last bound
    range_probs.push(pos!(1.0 - bound_probs[bound_probs.len() - 1]));

    Ok(range_probs)
}

#[cfg(test)]
mod tests_calculate_bounds_probability {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_bounds_probability_basic() {
        let bounds = vec![pos!(95.0), pos!(100.0), pos!(105.0)];
        let result = calculate_bounds_probability(
            pos!(100.0),
            bounds,
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_ok());
        let probs = result.unwrap();
        assert_eq!(probs.len(), 4); // N+1 probabilities

        // Verify probabilities sum to 1
        let sum: PositiveF64 = probs.iter().sum();
        assert_relative_eq!(sum.value(), 1.0, epsilon = 1e-10);

        // Verify all probabilities are between 0 and 1
        for prob in probs {
            assert!(prob >= PZERO && prob <= pos!(1.0));
        }
    }

    #[test]
    fn test_bounds_probability_empty_bounds() {
        let bounds = vec![];
        let result = calculate_bounds_probability(
            pos!(100.0),
            bounds,
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Bounds vector cannot be empty");
    }

    #[test]
    fn test_bounds_probability_unordered_bounds() {
        let bounds = vec![pos!(100.0), pos!(95.0), pos!(105.0)];
        let result = calculate_bounds_probability(
            pos!(100.0),
            bounds,
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Bounds must be in ascending order");
    }

    #[test]
    fn test_bounds_probability_single_bound() {
        let bounds = vec![pos!(100.0)];
        let result = calculate_bounds_probability(
            pos!(100.0),
            bounds,
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_ok());
        let probs = result.unwrap();
        assert_eq!(probs.len(), 2); // Two probabilities for single bound
        assert_relative_eq!((probs[0] + probs[1]).value(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_bounds_probability_with_volatility() {
        let bounds = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.5),
            std_dev_adjustment: PZERO,
        });

        let result = calculate_bounds_probability(
            pos!(100.0),
            bounds,
            vol_adj,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_ok());
        let probs = result.unwrap();
        assert_eq!(probs.len(), 4);
        assert_relative_eq!(probs.iter().sum::<PositiveF64>().value(), 1.0, epsilon = 1e-10);
    }
}

#[cfg(test)]
mod tests_single_point_probability {
    use super::*;
    use approx::assert_relative_eq;
    use chrono::{Duration, Utc};

    // Helper function to create default volatility adjustment
    fn default_volatility_adj() -> VolatilityAdjustment {
        VolatilityAdjustment {
            base_volatility: pos!(0.2),
            std_dev_adjustment: pos!(0.1),
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
        let current_price = pos!(100.0);
        let target_price = pos!(105.0);
        let result = calculate_single_point_probability(
            current_price,
            target_price,
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_calculation_with_datetime() {
        let current_price = pos!(100.0);
        let target_price = pos!(105.0);
        let expiration_date = Utc::now() + Duration::days(365);
    
        let result = calculate_single_point_probability(
            current_price,
            target_price,
            None,
            None,
            ExpirationDate::DateTime(expiration_date),
            None,
        );
    
        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_with_volatility_adjustment() {
        let current_price = pos!(100.0);
        let target_price = pos!(105.0);
        let vol_adj = Some(default_volatility_adj());
    
        let result = calculate_single_point_probability(
            current_price,
            target_price,
            vol_adj,
            None,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_ok());
        let ( prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_with_trend() {
        let current_price = pos!(100.0);
        let target_price = pos!(105.0);
        let trend = Some(default_trend());
    
        let result = calculate_single_point_probability(
            current_price,
            target_price,
            None,
            trend,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_with_risk_free_rate() {
        let current_price = pos!(100.0);
        let target_price = pos!(105.0);
    
        let result = calculate_single_point_probability(
            current_price,
            target_price,
            None,
            None,
            ExpirationDate::Days(365.0),
            Some(0.05),
        );
    
        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_all_parameters() {
        let current_price = pos!(100.0);
        let target_price = pos!(105.0);
        let vol_adj = Some(default_volatility_adj());
        let trend = Some(default_trend());
    
        let result = calculate_single_point_probability(
            current_price,
            target_price,
            vol_adj,
            trend,
            ExpirationDate::Days(365.0),
            Some(0.05),
        );
    
        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_target_equals_current() {
        let price = pos!(100.0);
    
        let result = calculate_single_point_probability(
            price,
            price,
            Some({
                VolatilityAdjustment {
                    base_volatility: pos!(0.8),
                    std_dev_adjustment: PZERO,
                }
            }),
            Some({
                PriceTrend {
                    drift_rate: 0.0,
                    confidence: 1.0,
                }
            }),
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_ok());
        let ( prob_below, prob_above) = result.unwrap();
        assert_relative_eq!((prob_above + prob_below).value(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(prob_below.value(), 0.5, epsilon = 1e-10);
        assert_relative_eq!(prob_above.value(), 0.5, epsilon = 1e-10);
    }
    
    #[test]
    fn test_zero_days_to_expiry() {
        let result = calculate_single_point_probability(
            pos!(100.0),
            pos!(105.0),
            None,
            None,
            ExpirationDate::Days(0.0),
            None,
        );
    
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Time to expiry must be positive".to_string()
        );
    }
    
    #[test]
    #[should_panic]
    fn test_past_datetime() {
        let past_date = Utc::now() - Duration::days(1);
    
        let result = calculate_single_point_probability(
            pos!(100.0),
            pos!(105.0),
            None,
            None,
            ExpirationDate::DateTime(past_date),
            None,
        );
    
        assert!(result.is_err());
    }
    
    #[test]
    fn test_invalid_volatility() {
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: PZERO,
            std_dev_adjustment: pos!(0.1),
        });
    
        let result = calculate_single_point_probability(
            pos!(100.0),
            pos!(105.0),
            vol_adj,
            None,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Base volatility must be positive".to_string()
        );
    }
    
    #[test]
    fn test_invalid_trend_confidence() {
        let trend = Some(PriceTrend {
            drift_rate: 0.05,
            confidence: 1.5, // Invalid confidence > 1.0
        });
    
        let result = calculate_single_point_probability(
            pos!(100.0),
            pos!(105.0),
            None,
            trend,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Confidence must be between 0 and 1".to_string()
        );
    }
    
    #[test]
    fn test_extreme_target_prices() {
        // Test with very high target price
        let result_high = calculate_single_point_probability(
            pos!(100.0),
            pos!(1000000.0),
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result_high.is_ok());
        let ( _, prob_above) = result_high.clone().unwrap();
        assert!(prob_above < pos!(0.01)); // Probability should be very low
    
        // Test with very low target price
        let result_low = calculate_single_point_probability(
            pos!(100.0),
            pos!(0.1),
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result_low.is_ok());
        let (prob_below ,prob_above) = result_low.unwrap();
        assert!(prob_above > pos!(0.99)); // Probability should be very high
        assert!(prob_below < pos!(0.01)); // Probability should be very low
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_extreme_volatility() {
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(1.0),
            std_dev_adjustment: pos!(5.0),
        });
    
        let result = calculate_single_point_probability(
            pos!(100.0),
            pos!(105.0),
            vol_adj,
            None,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_ok());
        let (prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_extreme_trend() {
        let trend = Some(PriceTrend {
            drift_rate: 2.0, // 200% annual drift
            confidence: 0.99,
        });
    
        let result = calculate_single_point_probability(
            pos!(100.0),
            pos!(105.0),
            None,
            trend,
            ExpirationDate::Days(365.0),
            None,
        );
    
        assert!(result.is_ok());
        let ( prob_below, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_above).value(), 1.0, epsilon = 1e-10);
    }
}


#[cfg(test)]
mod tests_calculate_price_probability {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_price_probability_basic() {
        let result = calculate_price_probability(
            pos!(100.0),
            pos!(95.0),
            pos!(105.0),
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_ok());
        println!("{:?}", result);
        let (prob_below, prob_in_range, prob_above) = result.unwrap();
        assert!(prob_below >= PZERO && prob_above <= pos!(1.0));
        assert!(prob_in_range >= PZERO && prob_in_range <= pos!(1.0));
        assert!(prob_above >= PZERO && prob_above <= pos!(1.0));
        assert_relative_eq!((prob_below + prob_in_range + prob_above).value(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_price_probability_invalid_bounds() {
        let result = calculate_price_probability(
            pos!(100.0),
            pos!(105.0), // Lower bound higher than upper bound
            pos!(95.0),
            None,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Lower bound must be less than upper bound".to_string()
        );
    }

    #[test]
    fn test_price_probability_with_volatility() {
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.5),
            std_dev_adjustment: PZERO,
        });

        let result = calculate_price_probability(
            pos!(100.0),
            pos!(90.0),
            pos!(110.0),
            vol_adj,
            None,
            ExpirationDate::Days(365.0),
            None,
        );

        assert!(result.is_ok());
        let (prob_below, prob_in_range, prob_above) = result.unwrap();
        assert_relative_eq!((prob_below + prob_in_range + prob_above).value(), 1.0, epsilon = 1e-10);
    }
}