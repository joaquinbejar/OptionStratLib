/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/
use crate::constants::ZERO;
use crate::model::types::{ExpirationDate, PositiveF64, PZERO};
use crate::model::ProfitRange;
use crate::pos;
use crate::strategies::base::Strategies;
use crate::strategies::probabilities::analysis::StrategyProbabilityAnalysis;
use crate::strategies::probabilities::utils::{
    calculate_single_point_probability, PriceTrend, VolatilityAdjustment,
};

/// Trait for analyzing probabilities and risk metrics of option strategies
#[allow(dead_code)]
pub trait ProbabilityAnalysis: Strategies {
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
                probability_of_max_profit: PZERO,  // Default value when no volatility adjustment
                probability_of_max_loss: PZERO,    // Default value when no volatility adjustment
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

    /// Calculate expected value of the strategy
    fn expected_value(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<PositiveF64, String>;

    /// Calculate probability of profit
    fn probability_of_profit(
        &self,
        _volatility_adj: Option<VolatilityAdjustment>,
        _trend: Option<PriceTrend>,
    ) -> Result<PositiveF64, String> {
        let break_even_points = self.get_break_even_points();
        if break_even_points.is_empty() {
            return Ok(PZERO);
        }

        // // Get the current price and expiration from the strategy
        // let current_price = self.get_reference_price()?;
        // let expiration = self.get_expiration()?;
        //
        // // Calculate probabilities for each break-even point
        // let bounds_probs = calculate_bounds_probability(
        //     current_price,
        //     break_even_points,
        //     volatility_adj,
        //     trend,
        //     expiration,
        //     None,
        // )?;

        // Sum probabilities for profitable ranges
        let mut total_prob = ZERO;
        let profit_ranges = self.get_profit_ranges()?;

        for range in profit_ranges {
            total_prob += range.probability;
        }

        Ok(pos!(total_prob))
    }

    /// Calculate extreme probabilities (max profit and max loss)
    fn calculate_extreme_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<(PositiveF64, PositiveF64), String> {
        let current_price = self.get_reference_price()?;
        let expiration = self.get_expiration()?;
        let profit_ranges = self.get_profit_ranges()?;

        // Find ranges for max profit and max loss
        let (max_profit_prob, max_loss_prob) = match profit_ranges.as_slice() {
            [] => return Ok((PZERO, PZERO)),
            [range] => {
                let (prob_below, prob_above) = match (range.lower_bound, range.upper_bound) {
                    (None, Some(upper)) => calculate_single_point_probability(
                        current_price,
                        upper,
                        volatility_adj,
                        trend,
                        expiration,
                        None,
                    )?,
                    (Some(lower), None) => calculate_single_point_probability(
                        current_price,
                        lower,
                        volatility_adj,
                        trend,
                        expiration,
                        None,
                    )?,
                    _ => return Err("Invalid profit range configuration".to_string()),
                };
                (prob_above, prob_below)
            }
            ranges => {
                let first = &ranges[0];
                let last = &ranges[ranges.len() - 1];

                
                match (first.lower_bound, last.upper_bound) {
                    (Some(lower), Some(upper)) => {
                        let (prob_below, _) = calculate_single_point_probability(
                            current_price,
                            upper,
                            volatility_adj.clone(),
                            trend.clone(),
                            expiration.clone(),
                            None,
                        )?;
                        let (prob_above_lower, _) = calculate_single_point_probability(
                            current_price,
                            lower,
                            volatility_adj,
                            trend,
                            expiration,
                            None,
                        )?;
                        (prob_below - prob_above_lower, prob_above_lower)
                    }
                    _ => return Err("Invalid range configuration for multiple ranges".to_string()),
                }
            }
        };

        Ok((max_profit_prob, max_loss_prob))
    }

    // Required helper methods
    fn get_reference_price(&self) -> Result<PositiveF64, String>;
    fn get_expiration(&self) -> Result<ExpirationDate, String>;
    fn get_profit_ranges(&self) -> Result<Vec<ProfitRange>, String>;
}


#[cfg(test)]
mod tests_probability_analysis {
    use super::*;
    use crate::model::types::{ExpirationDate, PositiveF64};
    use crate::model::ProfitRange;
    use crate::strategies::base::{Strategies, Validable};

    // Mock struct for testing
    struct MockStrategy {
        reference_price: PositiveF64,
        expiration: ExpirationDate,
        profit_ranges: Vec<ProfitRange>,
        break_even_points: Vec<PositiveF64>,
    }

    impl Validable for MockStrategy {}

    impl Strategies for MockStrategy {
        fn get_break_even_points(&self) -> Vec<PositiveF64> {
            self.break_even_points.clone()
        }

        fn profit_ratio(&self) -> f64 {
            2.0 // Mock value for testing
        }
    }

    impl ProbabilityAnalysis for MockStrategy {
        fn expected_value(
            &self,
            _volatility_adj: Option<VolatilityAdjustment>,
            _trend: Option<PriceTrend>,
        ) -> Result<PositiveF64, String> {
            Ok(pos!(100.0))
        }

        fn get_reference_price(&self) -> Result<PositiveF64, String> {
            Ok(self.reference_price)
        }

        fn get_expiration(&self) -> Result<ExpirationDate, String> {
            Ok(self.expiration.clone())
        }

        fn get_profit_ranges(&self) -> Result<Vec<ProfitRange>, String> {
            Ok(self.profit_ranges.clone())
        }
    }

    // Helper function to create a basic mock strategy
    fn create_mock_strategy() -> MockStrategy {
        MockStrategy {
            reference_price: pos!(100.0),
            expiration: ExpirationDate::Days(30.0),
            profit_ranges: vec![
                // Valid range configuration for a single range
                ProfitRange {
                    lower_bound: Some(pos!(90.0)),
                    upper_bound: Some(pos!(110.0)),
                    probability: pos!(0.6),
                }
            ],
            break_even_points: vec![pos!(95.0), pos!(105.0)],
        }
    }

    #[test]
    fn test_analyze_probabilities_basic() {
        let strategy = create_mock_strategy();
        let result = strategy.analyze_probabilities(None, None);
        match &result {
            Ok(_) => (),
            Err(e) => println!("Error in analyze_probabilities: {}", e),
        }
        assert!(result.is_ok(), "analyze_probabilities failed");

        let analysis = result.unwrap();
        assert!(
            analysis.probability_of_profit > PZERO,
            "probability_of_profit should be greater than zero"
        );
        assert!(
            analysis.risk_reward_ratio > PZERO,
            "risk_reward_ratio should be greater than zero"
        );
        assert_eq!(
            analysis.break_even_points.len(),
            2,
            "Should have exactly 2 break_even_points"
        );
    }

    #[test]
    fn test_calculate_extreme_probabilities_single_range() {
        let mut strategy = create_mock_strategy();
        // For single range case, we need only one bound defined
        strategy.profit_ranges = vec![ProfitRange {
            lower_bound: Some(pos!(90.0)),
            upper_bound: None,  // Single range should have only one bound defined
            probability: pos!(0.7),
        }];

        let result = strategy.calculate_extreme_probabilities(None, None);
        match &result {
            Ok(_) => (),
            Err(e) => println!("Error in single_range test: {}", e),
        }
        assert!(
            result.is_ok(),
            "calculate_extreme_probabilities with single range failed"
        );
    }

    #[test]
    fn test_calculate_extreme_probabilities_empty_ranges() {
        let mut strategy = create_mock_strategy();
        strategy.profit_ranges = vec![];

        let result = strategy.calculate_extreme_probabilities(None, None);
        assert!(
            result.is_ok(),
            "calculate_extreme_probabilities with empty ranges failed"
        );
        let (max_profit, max_loss) = result.unwrap();
        assert_eq!(max_profit, PZERO);
        assert_eq!(max_loss, PZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities_multiple_ranges() {
        let mut strategy = create_mock_strategy();
        strategy.profit_ranges = vec![
            ProfitRange {
                lower_bound: Some(pos!(90.0)),
                upper_bound: Some(pos!(95.0)),
                probability: pos!(0.3),
            },
            ProfitRange {
                lower_bound: Some(pos!(105.0)),
                upper_bound: Some(pos!(110.0)),
                probability: pos!(0.4),
            },
        ];

        let result = strategy.calculate_extreme_probabilities(None, None);
        match &result {
            Ok(_) => (),
            Err(e) => println!("Error in multiple_ranges test: {}", e),
        }
        assert!(
            result.is_ok(),
            "calculate_extreme_probabilities with multiple ranges failed"
        );
    }

    #[test]
    fn test_invalid_profit_range_configuration() {
        let mut strategy = create_mock_strategy();
        strategy.profit_ranges = vec![ProfitRange {
            lower_bound: None,
            upper_bound: None,
            probability: pos!(0.5),
        }];

        let result = strategy.calculate_extreme_probabilities(None, None);
        assert!(
            result.is_err(),
            "Should fail with invalid range configuration"
        );
    }
}