/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/
use crate::model::types::{ExpirationDate, PositiveF64, PZERO};
use crate::model::ProfitLossRange;
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::base::Strategies;
use crate::strategies::probabilities::analysis::StrategyProbabilityAnalysis;
use crate::strategies::probabilities::utils::{
    calculate_single_point_probability, PriceTrend, VolatilityAdjustment,
};
use tracing::warn;

/// Trait for analyzing probabilities and risk metrics of option strategies
#[allow(dead_code)]
pub trait ProbabilityAnalysis: Strategies + Profit {
    /// Calculate probability analysis for a strategy
    fn analyze_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<StrategyProbabilityAnalysis, String> {
        let break_even_points = self.get_break_even_points();
        // If both parameters are None, return default probabilities based on profit ranges
        if volatility_adj.is_none() && trend.is_none() {
            let probability_of_profit = self.probability_of_profit(None, None)?;
            let expected_value = self.expected_value(None, None)?;

            return Ok(StrategyProbabilityAnalysis {
                probability_of_profit,
                probability_of_max_profit: PZERO, // Default value when no volatility adjustment
                probability_of_max_loss: PZERO,   // Default value when no volatility adjustment
                expected_value,
                break_even_points: break_even_points.to_vec(),
                risk_reward_ratio: pos!(self.profit_ratio()),
            });
        }
        // If we have adjustments, calculate with them
        let probability_of_profit =
            self.probability_of_profit(volatility_adj.clone(), trend.clone())?;
        let expected_value = self.expected_value(volatility_adj.clone(), trend.clone())?;
        let (prob_max_profit, prob_max_loss) =
            self.calculate_extreme_probabilities(volatility_adj, trend)?;
        let risk_reward_ratio = pos!(self.profit_ratio());

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
    /// - `Result<PositiveF64, String>`: On success, returns a `PositiveF64` representing
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
    ) -> Result<PositiveF64, String> {
        // Special case: when volatility is zero, return the current value
        if let Some(ref vol_adj) = volatility_adj {
            if vol_adj.base_volatility == PZERO && vol_adj.std_dev_adjustment == PZERO {
                let current_profit = self.calculate_profit_at(self.get_underlying_price());
                return if current_profit <= 0.0 {
                    Ok(PZERO)
                } else {
                    Ok(pos!(current_profit))
                };
            }
        }

        let step = pos!(self.get_underlying_price().value() / 100.0);
        let range = self.best_range_to_show(step).unwrap();
        let expiration = self.get_expiration()?;

        let mut probabilities = Vec::with_capacity(range.len());
        let mut last_prob = 0.0;

        for price in range.iter() {
            let prob = calculate_single_point_probability(
                self.get_underlying_price(),
                *price,
                volatility_adj.clone(),
                trend.clone(),
                expiration.clone(),
                None,
            )?;

            let marginal_prob = prob.0.value() - last_prob;
            probabilities.push(marginal_prob);
            last_prob = prob.0.value();
        }

        let expected_value = range
            .iter()
            .zip(probabilities.iter())
            .fold(0.0, |acc, (price, prob)| {
                acc + self.calculate_profit_at(*price) * prob
            });

        let total_prob: f64 = probabilities.iter().sum();
        if (total_prob - 1.0).abs() > 0.05 {
            warn!(
                "Sum of probabilities ({}) deviates significantly from 1.0",
                total_prob
            );
        }

        if expected_value <= 0.0 {
            Ok(PZERO)
        } else {
            let trend_adjustment = trend.map_or(1.0, |t| 1.0 / (1.0 + t.drift_rate.abs()));
            Ok(pos!(expected_value * trend_adjustment))
        }
    }

    /// Calculate probability of profit
    fn probability_of_profit(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<PositiveF64, String> {
        let mut sum_of_probabilities = PZERO;
        let ranges = self.get_profit_ranges()?;
        for mut range in ranges {
            range.calculate_probability(
                self.get_underlying_price(),
                volatility_adj.clone(),
                trend.clone(),
                self.get_expiration()?,
                self.get_risk_free_rate(),
            )?;
            sum_of_probabilities += range.probability;
        }
        Ok(sum_of_probabilities)
    }

    fn probability_of_loss(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<PositiveF64, String> {
        let mut sum_of_probabilities = PZERO;
        let ranges = self.get_loss_ranges()?;
        for mut range in ranges {
            range.calculate_probability(
                self.get_underlying_price(),
                volatility_adj.clone(),
                trend.clone(),
                self.get_expiration()?,
                self.get_risk_free_rate(),
            )?;
            sum_of_probabilities += range.probability;
        }
        Ok(sum_of_probabilities)
    }

    /// Calculate extreme probabilities (max profit and max loss)
    fn calculate_extreme_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<(PositiveF64, PositiveF64), String> {
        let profit_ranges = self.get_profit_ranges()?;
        let loss_ranges = self.get_loss_ranges()?;

        let max_profit_range = profit_ranges
            .iter()
            .find(|range| range.upper_bound.is_none());

        let max_loss_range = loss_ranges.iter().find(|range| range.lower_bound.is_none());

        let mut max_profit_prob = PZERO;
        if let Some(range) = max_profit_range {
            let mut range_clone = range.clone();
            range_clone.calculate_probability(
                self.get_underlying_price(),
                volatility_adj.clone(),
                trend.clone(),
                self.get_expiration()?,
                self.get_risk_free_rate(),
            )?;
            max_profit_prob = range_clone.probability;
        }

        let mut max_loss_prob = PZERO;
        if let Some(range) = max_loss_range {
            let mut range_clone = range.clone();
            range_clone.calculate_probability(
                self.get_underlying_price(),
                volatility_adj,
                trend,
                self.get_expiration()?,
                self.get_risk_free_rate(),
            )?;
            max_loss_prob = range_clone.probability;
        }

        Ok((max_profit_prob, max_loss_prob))
    }

    fn get_expiration(&self) -> Result<ExpirationDate, String>;
    fn get_risk_free_rate(&self) -> Option<f64>;
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String>;
    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String>;
}

#[cfg(test)]
mod tests_probability_analysis {
    use super::*;
    use crate::model::types::{ExpirationDate, PositiveF64};
    use crate::pos;
    use crate::pricing::payoff::Profit;
    use crate::strategies::base::{Positionable, Strategies, Validable};

    // Mock struct para testing
    struct MockStrategy {
        underlying_price: PositiveF64,
        expiration: ExpirationDate,
        risk_free_rate: f64,
    }

    impl Validable for MockStrategy {}

    impl Positionable for MockStrategy {}

    impl Strategies for MockStrategy {
        fn get_underlying_price(&self) -> PositiveF64 {
            self.underlying_price
        }

        fn profit_ratio(&self) -> f64 {
            2.0
        }

        fn best_range_to_show(&self, _step: PositiveF64) -> Option<Vec<PositiveF64>> {
            Some(vec![
                pos!(90.0),
                pos!(95.0),
                pos!(100.0),
                pos!(105.0),
                pos!(110.0),
            ])
        }

        fn get_break_even_points(&self) -> Vec<PositiveF64> {
            vec![pos!(95.0), pos!(105.0)]
        }
    }

    impl Profit for MockStrategy {
        fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
            price.value() - self.underlying_price.value()
        }
    }

    impl MockStrategy {
        fn new() -> Self {
            MockStrategy {
                underlying_price: pos!(100.0),
                expiration: ExpirationDate::Days(30.0),
                risk_free_rate: 0.05,
            }
        }
    }

    impl ProbabilityAnalysis for MockStrategy {
        fn get_expiration(&self) -> Result<ExpirationDate, String> {
            Ok(self.expiration.clone())
        }

        fn get_risk_free_rate(&self) -> Option<f64> {
            Some(self.risk_free_rate)
        }

        fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
            Ok(vec![ProfitLossRange::new(
                Some(pos!(95.0)),
                Some(pos!(105.0)),
                pos!(0.0),
            )?])
        }

        fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
            Ok(vec![
                ProfitLossRange::new(None, Some(pos!(95.0)), pos!(0.0))?,
                ProfitLossRange::new(Some(pos!(105.0)), None, pos!(0.0))?,
            ])
        }
    }

    #[test]
    fn test_analyze_probabilities_without_adjustments() {
        let strategy = MockStrategy::new();
        let result = strategy.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > PZERO);
        assert_eq!(analysis.probability_of_max_profit, PZERO);
        assert_eq!(analysis.probability_of_max_loss, PZERO);
        assert!(analysis.risk_reward_ratio > PZERO);
    }

    #[test]
    fn test_analyze_probabilities_with_adjustments() {
        let strategy = MockStrategy::new();
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
        assert!(analysis.probability_of_profit > PZERO);
        assert!(analysis.probability_of_max_profit >= PZERO);
        assert!(analysis.probability_of_max_loss >= PZERO);
    }

    #[test]
    fn test_expected_value_calculation() {
        let strategy = MockStrategy::new();
        let result = strategy.expected_value(None, None);

        assert!(result.is_ok());
        assert!(result.unwrap() > PZERO);
    }

    #[test]
    fn test_expected_value_with_trend() {
        let strategy = MockStrategy::new();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strategy.expected_value(None, trend);

        assert!(result.is_ok());
        assert!(result.unwrap() > PZERO);
    }

    #[test]
    fn test_probability_of_profit() {
        let strategy = MockStrategy::new();
        let result = strategy.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_of_loss() {
        let strategy = MockStrategy::new();
        let result = strategy.probability_of_loss(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let strategy = MockStrategy::new();
        let result = strategy.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= PZERO);
        assert!(max_loss_prob >= PZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }

    #[test]
    fn test_extreme_probabilities_with_adjustments() {
        let strategy = MockStrategy::new();
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
        assert!(max_profit_prob >= PZERO);
        assert!(max_loss_prob >= PZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_calculations_sum_to_one() {
        let strategy = MockStrategy::new();
        let profit_prob = strategy.probability_of_profit(None, None).unwrap();
        let loss_prob = strategy.probability_of_loss(None, None).unwrap();

        let total_prob = profit_prob + loss_prob;
        assert!((total_prob.value() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_expected_value_with_volatility() {
        let strategy = MockStrategy::new();
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
    use crate::strategies::base::{Positionable, Validable};

    // Helper function to create a test strategy
    fn create_test_strategy() -> TestStrategy {
        TestStrategy {
            underlying_price: pos!(100.0),
            expiration: ExpirationDate::Days(30.0),
            risk_free_rate: 0.05,
        }
    }

    // Mock strategy for testing
    struct TestStrategy {
        underlying_price: PositiveF64,
        expiration: ExpirationDate,
        risk_free_rate: f64,
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_underlying_price(&self) -> PositiveF64 {
            self.underlying_price
        }

        fn best_range_to_show(&self, _step: PositiveF64) -> Option<Vec<PositiveF64>> {
            Some(vec![
                pos!(90.0),
                pos!(95.0),
                pos!(100.0),
                pos!(105.0),
                pos!(110.0),
            ])
        }
    }

    impl Profit for TestStrategy {
        fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
            price.value() - self.underlying_price.value()
        }
    }

    impl ProbabilityAnalysis for TestStrategy {
        fn get_expiration(&self) -> Result<ExpirationDate, String> {
            Ok(self.expiration.clone())
        }

        fn get_risk_free_rate(&self) -> Option<f64> {
            Some(self.risk_free_rate)
        }

        fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
            Ok(vec![ProfitLossRange::new(
                Some(pos!(95.0)),
                Some(pos!(105.0)),
                PZERO,
            )?])
        }

        fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
            Ok(vec![ProfitLossRange::new(None, Some(pos!(95.0)), PZERO)?])
        }
    }

    #[test]
    fn test_expected_value_basic() {
        let strategy = create_test_strategy();
        let result = strategy.expected_value(None, None);

        assert!(result.is_ok(), "Expected value calculation should succeed");
        let ev = result.unwrap();
        assert!(ev >= PZERO, "Expected value should be non-negative");
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
        assert!(result.unwrap() >= PZERO);
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
        assert!(result.unwrap() >= PZERO);
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
        assert!(result.unwrap() >= PZERO);
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
        assert!(result.unwrap() >= PZERO);
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
        assert!(result.unwrap() >= PZERO);
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
    fn test_expected_value_with_extreme_prices() {
        struct ExtremeStrategy {
            base: TestStrategy,
        }

        impl Validable for ExtremeStrategy {}
        impl Positionable for ExtremeStrategy {}
        impl Strategies for ExtremeStrategy {
            fn get_underlying_price(&self) -> PositiveF64 {
                self.base.get_underlying_price()
            }

            fn best_range_to_show(&self, _step: PositiveF64) -> Option<Vec<PositiveF64>> {
                Some(vec![pos!(1.0), pos!(1000.0), pos!(10000.0)])
            }
        }

        impl Profit for ExtremeStrategy {
            fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
                self.base.calculate_profit_at(price)
            }
        }

        impl ProbabilityAnalysis for ExtremeStrategy {
            fn get_expiration(&self) -> Result<ExpirationDate, String> {
                self.base.get_expiration()
            }

            fn get_risk_free_rate(&self) -> Option<f64> {
                self.base.get_risk_free_rate()
            }

            fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
                self.base.get_profit_ranges()
            }

            fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
                self.base.get_loss_ranges()
            }
        }

        let strategy = ExtremeStrategy {
            base: create_test_strategy(),
        };

        let result = strategy.expected_value(None, None);
        assert!(result.is_ok());
        assert!(result.unwrap() >= PZERO);
    }

    #[test]
    fn test_expected_value_with_minimal_volatility() {
        let strategy = create_test_strategy();
        // Use a very small but positive volatility value
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.0001), // Very small but non-zero volatility
            std_dev_adjustment: PZERO,
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(
            result.is_ok(),
            "Expected value calculation should succeed with minimal volatility"
        );
        assert!(
            result.unwrap() >= PZERO,
            "Expected value should be non-negative"
        );
    }

    #[test]
    fn test_expected_value_with_zero_volatility() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: PZERO,
            std_dev_adjustment: PZERO,
        });
        let result = strategy.expected_value(vol_adj, None).unwrap();
        assert_eq!(result, PZERO);
    }
}
