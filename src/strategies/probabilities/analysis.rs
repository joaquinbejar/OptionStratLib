/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/

use positive::{Positive, pos_or_panic};

/// # StrategyProbabilityAnalysis
///
/// Represents the probabilistic analysis of an options trading strategy, providing key metrics
/// for risk assessment and expected outcomes.
///
/// This struct contains various probability-based metrics that help traders evaluate the
/// potential performance of a strategy under different market scenarios.
///
/// ## Fields
/// * `probability_of_profit`: The probability that the strategy will result in a profit at expiration.
///   This is a key metric for assessing the overall likelihood of success.
///
/// * `probability_of_max_profit`: The probability that the strategy will achieve its maximum possible profit.
///   This typically occurs when specific price conditions are met at expiration.
///
/// * `probability_of_max_loss`: The probability that the strategy will experience its maximum possible loss.
///   Understanding this risk metric is crucial for proper position sizing and risk management.
///
/// * `expected_value`: The mathematical expectation of the strategy's outcome, calculated by multiplying
///   possible profits and losses by their respective probabilities. A positive expected value suggests
///   the strategy is favorable in the long run.
///
/// * `break_even_points`: The price levels at which the strategy neither makes a profit nor incurs a loss.
///   These are important reference points for monitoring strategy performance.
///
/// * `risk_reward_ratio`: The ratio between the potential loss and the potential gain of the strategy.
///   Lower values generally indicate more favorable risk-adjusted opportunities.
///
/// This analysis is typically generated through statistical models or Monte Carlo simulations
/// that incorporate factors such as implied volatility, time to expiration, and price movement
/// probabilities.
#[allow(dead_code)]
#[derive(Debug)]
pub struct StrategyProbabilityAnalysis {
    /// The probability of profit (POP)
    pub probability_of_profit: Positive,
    /// The probability of maximum profit
    pub probability_of_max_profit: Positive,
    /// The probability of maximum loss
    pub probability_of_max_loss: Positive,
    /// Expected value of the strategy
    pub expected_value: Positive,
    /// Break-even points
    pub break_even_points: Vec<Positive>,
    /// Risk-reward ratio
    pub risk_reward_ratio: Positive,
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;

    #[test]
    fn test_strategy_analysis_creation() {
        // Create a valid StrategyProbabilityAnalysis instance
        let analysis = StrategyProbabilityAnalysis {
            probability_of_profit: pos_or_panic!(0.65),
            probability_of_max_profit: pos_or_panic!(0.30),
            probability_of_max_loss: pos_or_panic!(0.20),
            expected_value: pos_or_panic!(250.00),
            break_even_points: vec![pos_or_panic!(45.50), pos_or_panic!(55.50)],
            risk_reward_ratio: pos_or_panic!(0.75),
        };

        // Verify that the fields were set correctly
        assert_eq!(analysis.probability_of_profit.0, dec!(0.65));
        assert_eq!(analysis.probability_of_max_profit.0, dec!(0.30));
        assert_eq!(analysis.probability_of_max_loss.0, dec!(0.20));
        assert_eq!(analysis.expected_value.0, dec!(250.00));
        assert_eq!(analysis.break_even_points.len(), 2);
        assert_eq!(analysis.break_even_points[0].0, dec!(45.50));
        assert_eq!(analysis.break_even_points[1].0, dec!(55.50));
        assert_eq!(analysis.risk_reward_ratio.0, dec!(0.75));
    }

    #[test]
    fn test_debug_implementation() {
        // Create a simple StrategyProbabilityAnalysis instance
        let analysis = StrategyProbabilityAnalysis {
            probability_of_profit: pos_or_panic!(0.60),
            probability_of_max_profit: pos_or_panic!(0.25),
            probability_of_max_loss: pos_or_panic!(0.15),
            expected_value: pos_or_panic!(100.00),
            break_even_points: vec![pos_or_panic!(50.00)],
            risk_reward_ratio: pos_or_panic!(1.00),
        };

        // Verify that Debug trait is implemented by checking that debug string is not empty
        let debug_str = format!("{analysis:?}");
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("StrategyProbabilityAnalysis"));
    }

    #[test]
    fn test_with_empty_break_even_points() {
        // Create a StrategyProbabilityAnalysis with no break-even points
        let analysis = StrategyProbabilityAnalysis {
            probability_of_profit: pos_or_panic!(0.70),
            probability_of_max_profit: pos_or_panic!(0.40),
            probability_of_max_loss: pos_or_panic!(0.10),
            expected_value: pos_or_panic!(350.00),
            break_even_points: vec![],
            risk_reward_ratio: pos_or_panic!(0.50),
        };

        // Verify that empty vector is handled correctly
        assert_eq!(analysis.break_even_points.len(), 0);
    }

    #[test]
    fn test_with_multiple_break_even_points() {
        // Create a StrategyProbabilityAnalysis with multiple break-even points
        let analysis = StrategyProbabilityAnalysis {
            probability_of_profit: pos_or_panic!(0.55),
            probability_of_max_profit: pos_or_panic!(0.20),
            probability_of_max_loss: pos_or_panic!(0.25),
            expected_value: pos_or_panic!(150.00),
            break_even_points: vec![
                pos_or_panic!(40.00),
                pos_or_panic!(50.00),
                pos_or_panic!(60.00),
            ],
            risk_reward_ratio: pos_or_panic!(1.25),
        };

        // Verify that multiple break-even points are handled correctly
        assert_eq!(analysis.break_even_points.len(), 3);
        assert_eq!(analysis.break_even_points[0].0, dec!(40.00));
        assert_eq!(analysis.break_even_points[1].0, dec!(50.00));
        assert_eq!(analysis.break_even_points[2].0, dec!(60.00));
    }

    #[test]
    fn test_probability_values_within_range() {
        // Create an analysis with probabilities that should be between 0 and 1
        let analysis = StrategyProbabilityAnalysis {
            probability_of_profit: pos_or_panic!(0.65),
            probability_of_max_profit: pos_or_panic!(0.30),
            probability_of_max_loss: pos_or_panic!(0.20),
            expected_value: pos_or_panic!(250.00),
            break_even_points: vec![pos_or_panic!(45.50), pos_or_panic!(55.50)],
            risk_reward_ratio: pos_or_panic!(0.75),
        };

        // Verify that probabilities are within expected range
        assert!(
            analysis.probability_of_profit.0 >= dec!(0)
                && analysis.probability_of_profit.0 <= dec!(1)
        );
        assert!(
            analysis.probability_of_max_profit.0 >= dec!(0)
                && analysis.probability_of_max_profit.0 <= dec!(1)
        );
        assert!(
            analysis.probability_of_max_loss.0 >= dec!(0)
                && analysis.probability_of_max_loss.0 <= dec!(1)
        );
    }
}
