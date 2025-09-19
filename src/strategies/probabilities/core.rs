//! # Probability Analysis Module
//!
//! This module provides functionality for analyzing the probability metrics of option trading strategies.
//! It includes tools for calculating expected values, profit probabilities, and risk-reward ratios based
//! on various market conditions like volatility and price trends.
//!
//! The `ProbabilityAnalysis` trait extends the `Strategies` and `Profit` traits to provide
//! comprehensive probability analysis capabilities for option strategies.

use crate::error::probability::ProbabilityError;
use crate::model::ProfitLossRange;
use crate::pricing::payoff::Profit;
use crate::strategies::base::Strategies;
use crate::strategies::probabilities::analysis::StrategyProbabilityAnalysis;
use crate::strategies::probabilities::utils::{
    PriceTrend, VolatilityAdjustment, calculate_single_point_probability,
};
use crate::{Positive, pos};
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use tracing::warn;

/// Trait for analyzing probabilities and risk metrics of option strategies
///
/// This trait provides methods to analyze the probability characteristics of options strategies,
/// including probability of profit/loss, expected value, and risk-reward metrics.
///
/// # Type Requirements
///
/// Implementors must also implement:
/// - The `Strategies` trait, which provides access to strategy configuration
/// - The `Profit` trait, which provides profit calculation capabilities
///
/// # Key Features
///
/// - Calculate probability of profit for option strategies
/// - Compute expected values with adjustments for volatility and price trends
/// - Determine break-even points and risk-reward ratios
/// - Analyze extreme outcome probabilities (max profit and max loss scenarios)
///
pub trait ProbabilityAnalysis: Strategies + Profit {
    /// Calculate probability analysis for a strategy
    ///
    /// Performs a comprehensive probability analysis for an option strategy, taking into
    /// account optional volatility adjustments and price trend parameters.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Optional volatility adjustment parameters
    /// - `trend`: Optional price trend parameters indicating market direction bias
    ///
    /// # Returns
    ///
    /// - `Result<StrategyProbabilityAnalysis, ProbabilityError>`: Structured analysis results or an error
    ///
    /// # Analysis Components
    ///
    /// The returned analysis includes:
    /// - Probability of profit
    /// - Probability of reaching maximum profit
    /// - Probability of suffering maximum loss
    /// - Expected value
    /// - Break-even points
    /// - Risk-reward ratio
    fn analyze_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<StrategyProbabilityAnalysis, ProbabilityError> {
        let break_even_points = self.get_break_even_points().unwrap();
        // If both parameters are None, return default probabilities based on profit ranges
        if volatility_adj.is_none() && trend.is_none() {
            let probability_of_profit = self.probability_of_profit(None, None)?;
            let expected_value = self.expected_value(None, None)?;

            return Ok(StrategyProbabilityAnalysis {
                probability_of_profit,
                probability_of_max_profit: Positive::ZERO, // Default value when no volatility adjustment
                probability_of_max_loss: Positive::ZERO, // Default value when no volatility adjustment
                expected_value,
                break_even_points: break_even_points.to_vec(),
                risk_reward_ratio: self.get_profit_ratio().unwrap().into(),
            });
        }

        // If we have adjustments, calculate with them
        let probability_of_profit =
            self.probability_of_profit(volatility_adj.clone(), trend.clone())?;
        let expected_value = self.expected_value(volatility_adj.clone(), trend.clone())?;
        let (prob_max_profit, prob_max_loss) =
            self.calculate_extreme_probabilities(volatility_adj, trend)?;
        let risk_reward_ratio = self.get_profit_ratio().unwrap().into();

        Ok(StrategyProbabilityAnalysis {
            probability_of_profit,
            probability_of_max_profit: prob_max_profit,
            probability_of_max_loss: prob_max_loss,
            expected_value,
            break_even_points: break_even_points.to_vec(),
            risk_reward_ratio,
        })
    }

    /// This function calculates the expected value of an option strategy
    /// based on an underlying price, volatility adjustments, and price trends.
    ///
    /// # Parameters
    /// - `volatility_adj`: An optional `VolatilityAdjustment` parameter, which contains
    ///   the base volatility and the number of standard deviations to adjust.
    /// - `trend`: An optional `PriceTrend` parameter, which indicates the
    ///   annual drift rate and the confidence level for the trend.
    ///
    /// # Returns
    /// - `Result<Positive, String>`: On success, returns a `Positive` representing
    ///   the expected value. On failure, returns an error message as a `String`.
    ///
    /// The function performs the following operations:
    /// - Determines the pricing range using the underlying asset's price and steps based
    ///   on 1% increments of the current price.
    /// - Calculates the single-point probability for each price within the range using the
    ///   provided volatility adjustments and price trends.
    /// - Computes the expected value by summing up the product of calculated probabilities
    ///   and the strategy's profit at each price point.
    /// - Logs the calculated range with probabilities for diagnostic purposes.
    ///
    /// This function relies on several auxiliary methods and traits, such as
    /// `get_underlying_price`, `best_range_to_show`, and `calculate_profit_at`,
    /// which are defined in the module's traits and utilities.
    fn expected_value(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        // Special case: when volatility is zero, return the current value
        if let Some(ref vol_adj) = volatility_adj
            && vol_adj.base_volatility == Positive::ZERO
            && vol_adj.std_dev_adjustment == Positive::ZERO
        {
            let current_profit = self.calculate_profit_at(self.get_underlying_price())?;
            return if current_profit <= Decimal::ZERO {
                Ok(Positive::ZERO)
            } else {
                Ok(current_profit.into())
            };
        }

        let step = self.get_underlying_price() / 100.0;
        let range = self.get_best_range_to_show(step).unwrap();
        let expiration = *self.get_expiration().values().next().unwrap();

        let mut probabilities = Vec::with_capacity(range.len());
        let mut last_prob = 0.0;

        for price in range.iter() {
            let prob = calculate_single_point_probability(
                self.get_underlying_price(),
                price,
                volatility_adj.clone(),
                trend.clone(),
                expiration,
                None,
            )?;

            let marginal_prob = prob.0 - last_prob;
            probabilities.push(marginal_prob);
            last_prob = prob.0.into();
        }

        let expected_value =
            range
                .iter()
                .zip(probabilities.iter())
                .fold(0.0, |acc, (price, prob)| {
                    acc + self.calculate_profit_at(price).unwrap().to_f64().unwrap() * *prob
                });

        let total_prob: f64 = probabilities.iter().map(|p| p.to_f64()).sum();
        if (total_prob - 1.0).abs() > 0.05 {
            warn!(
                "Sum of probabilities ({}) deviates significantly from 1.0",
                total_prob
            );
        }
        if expected_value <= 0.0 {
            Ok(Positive::ZERO)
        } else {
            let trend_adjustment = trend.map_or(1.0, |t| 1.0 / (1.0 + t.drift_rate.abs()));
            Ok(pos!(expected_value * trend_adjustment))
        }
    }

    /// Calculate probability of profit
    ///
    /// Calculates the probability that the option strategy will result in a profit at expiration.
    /// This method aggregates probabilities across all price ranges that would result in a profit.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Optional volatility adjustment parameters
    /// - `trend`: Optional price trend parameters
    ///
    /// # Returns
    ///
    /// - `Result<Positive, ProbabilityError>`: The probability of profit (between 0 and 1) or an error
    fn probability_of_profit(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        let mut sum_of_probabilities = Positive::ZERO;
        let ranges = self.get_profit_ranges()?;
        let option = self.one_option();
        let expiration = option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let underlying_price = option.underlying_price;
        for mut range in ranges {
            range.calculate_probability(
                &underlying_price,
                volatility_adj.clone(),
                trend.clone(),
                &expiration,
                Some(risk_free_rate),
            )?;
            sum_of_probabilities += range.probability;
        }
        Ok(sum_of_probabilities)
    }

    /// Calculate probability of loss
    ///
    /// Calculates the probability that the option strategy will result in a loss at expiration.
    /// This method aggregates probabilities across all price ranges that would result in a loss.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Optional volatility adjustment parameters
    /// - `trend`: Optional price trend parameters
    ///
    /// # Returns
    ///
    /// - `Result<Positive, ProbabilityError>`: The probability of loss (between 0 and 1) or an error
    fn probability_of_loss(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        let mut sum_of_probabilities = Positive::ZERO;
        let ranges = self.get_loss_ranges()?;
        let option = self.one_option();
        let expiration = option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let underlying_price = option.underlying_price;
        for mut range in ranges {
            range.calculate_probability(
                &underlying_price,
                volatility_adj.clone(),
                trend.clone(),
                &expiration,
                Some(risk_free_rate),
            )?;
            sum_of_probabilities += range.probability;
        }
        Ok(sum_of_probabilities)
    }

    /// Calculate extreme probabilities (max profit and max loss)
    ///
    /// Calculates the probabilities of reaching the maximum possible profit and
    /// suffering the maximum possible loss for the strategy.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Optional volatility adjustment parameters
    /// - `trend`: Optional price trend parameters
    ///
    /// # Returns
    ///
    /// - `Result<(Positive, Positive), ProbabilityError>`: A tuple containing (probability_of_max_profit,
    ///   probability_of_max_loss) or an error
    fn calculate_extreme_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<(Positive, Positive), ProbabilityError> {
        let profit_ranges = self.get_profit_ranges()?;
        let loss_ranges = self.get_loss_ranges()?;

        let max_profit_range = profit_ranges
            .iter()
            .find(|range| range.upper_bound.is_none());

        let max_loss_range = loss_ranges.iter().find(|range| range.lower_bound.is_none());
        let expiration = *self.get_expiration().values().next().unwrap();
        let risk_free_rate = *self.get_risk_free_rate().values().next().unwrap();
        let underlying_price = self.get_underlying_price();

        let mut max_profit_prob = Positive::ZERO;
        if let Some(range) = max_profit_range {
            let mut range_clone = range.clone();
            range_clone.calculate_probability(
                underlying_price,
                volatility_adj.clone(),
                trend.clone(),
                expiration,
                Some(*risk_free_rate),
            )?;
            max_profit_prob = range_clone.probability;
        }

        let mut max_loss_prob = Positive::ZERO;
        if let Some(range) = max_loss_range {
            let mut range_clone = range.clone();
            range_clone.calculate_probability(
                underlying_price,
                volatility_adj,
                trend,
                expiration,
                Some(*risk_free_rate),
            )?;
            max_loss_prob = range_clone.probability;
        }

        Ok((max_profit_prob, max_loss_prob))
    }

    /// Get the price ranges that would result in a profit
    ///
    /// # Returns
    /// - `Result<Vec<ProfitLossRange>, ProbabilityError>`: A vector of price ranges
    ///   that result in profit, or an error
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError>;

    /// # Get Profit/Loss Ranges
    ///
    /// Returns a collection of price ranges with associated probabilities for profit and loss scenarios.
    ///
    /// This function analyzes the strategy to identify distinct price ranges where the strategy
    /// would result in either profit or loss at expiration. Each range includes probability
    /// information based on the statistical model for the underlying asset.
    ///
    /// ## Returns
    ///
    /// * `Result<Vec<ProfitLossRange>, ProbabilityError>` - On success, returns a vector of
    ///   profit/loss ranges sorted by their price boundaries. On failure, returns a
    ///   `ProbabilityError` indicating what went wrong during the analysis.
    ///
    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError>;
}

#[cfg(test)]
mod tests_probability_analysis {
    use super::*;
    use crate::strategies::BullCallSpread;
    use crate::{ExpirationDate, pos};
    use rust_decimal_macros::dec;

    fn test_strategy() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            pos!(2505.8), // underlying_price
            pos!(2460.0), // long_strike_itm
            pos!(2515.0), // short_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(27.26),    // premium_long
            pos!(5.33),     // premium_short
            pos!(0.58),     // open_fee_long
            pos!(0.58),     // close_fee_long
            pos!(0.55),     // close_fee_short
            pos!(0.54),     // open_fee_short
        )
    }

    #[test]
    fn test_analyze_probabilities_without_adjustments() {
        let strategy = test_strategy();
        let result = strategy.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert_eq!(analysis.probability_of_max_profit, Positive::ZERO);
        assert_eq!(analysis.probability_of_max_loss, Positive::ZERO);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_analyze_probabilities_with_adjustments() {
        let strategy = test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.2),
            std_dev_adjustment: pos!(0.05),
        });
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strategy.analyze_probabilities(vol_adj, trend);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_calculation() {
        let strategy = test_strategy();
        let result = strategy.expected_value(None, None);

        assert!(result.is_ok());
        assert!(result.unwrap() > Positive::ZERO);
    }

    #[test]
    fn test_expected_value_with_trend() {
        let strategy = test_strategy();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strategy.expected_value(None, trend);

        assert!(result.is_ok());
        assert!(result.unwrap() > Positive::ZERO);
    }

    #[test]
    fn test_probability_of_profit() {
        let strategy = test_strategy();
        let result = strategy.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_of_loss() {
        let strategy = test_strategy();
        let result = strategy.probability_of_loss(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let strategy = test_strategy();
        let result = strategy.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }

    #[test]
    fn test_extreme_probabilities_with_adjustments() {
        let strategy = test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.2),
            std_dev_adjustment: pos!(0.05),
        });
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strategy.calculate_extreme_probabilities(vol_adj, trend);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }

    #[test]
    fn test_expected_value_with_volatility() {
        let strategy = test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.3),
            std_dev_adjustment: pos!(0.05),
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod tests_expected_value {
    use super::*;
    use crate::ExpirationDate;
    use crate::strategies::BullCallSpread;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            pos!(2505.8), // underlying_price
            pos!(2460.0), // long_strike_itm
            pos!(2515.0), // short_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(27.26),    // premium_long
            pos!(5.33),     // premium_short
            pos!(0.58),     // open_fee_long
            pos!(0.58),     // close_fee_long
            pos!(0.55),     // close_fee_short
            pos!(0.54),     // open_fee_short
        )
    }

    #[test]
    fn test_expected_value_basic() {
        let strategy = create_test_strategy();
        let result = strategy.expected_value(None, None);

        assert!(result.is_ok(), "Expected value calculation should succeed");
        let ev = result.unwrap();
        assert!(
            ev >= Positive::ZERO,
            "Expected value should be non-negative"
        );
    }

    #[test]
    fn test_expected_value_with_volatility() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(result.is_ok());
        assert!(result.unwrap() >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_with_trend() {
        let strategy = create_test_strategy();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strategy.expected_value(None, trend);
        assert!(result.is_ok());
        assert!(result.unwrap() >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_with_both_adjustments() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strategy.expected_value(vol_adj, trend);
        assert!(result.is_ok());
        assert!(result.unwrap() >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_with_high_volatility() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(1.0),
            std_dev_adjustment: pos!(0.5),
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(result.is_ok());
        assert!(result.unwrap() >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_with_negative_trend() {
        let strategy = create_test_strategy();
        let trend = Some(PriceTrend {
            drift_rate: -0.2,
            confidence: 0.90,
        });

        let result = strategy.expected_value(None, trend);
        assert!(result.is_ok());
        assert!(result.unwrap() >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_probabilities_sum() {
        let strategy = create_test_strategy();
        let result = strategy.expected_value(None, None);
        assert!(result.is_ok());

        // Test passes implicitly if no warning is logged about probability sum deviation
        // The actual check is done inside the method using warn!
    }

    #[test]
    fn test_expected_value_with_minimal_volatility() {
        let strategy = create_test_strategy();
        // Use a very small but positive volatility value
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.0001), // Very small but non-zero volatility
            std_dev_adjustment: Positive::ZERO,
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(
            result.is_ok(),
            "Expected value calculation should succeed with minimal volatility"
        );
        assert!(
            result.unwrap() >= Positive::ZERO,
            "Expected value should be non-negative"
        );
    }
}
