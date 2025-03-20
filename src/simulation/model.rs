/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/
use crate::pnl::model::PnLRange;
use crate::risk::{RiskCategory, RiskMetricsSimulation};
use crate::simulation::WalkId;
use crate::{Positive, pos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the results of a trading strategy simulation.
///
/// This structure contains comprehensive information about the performance of a trading strategy
/// across multiple simulation iterations. It captures key statistics related to profitability,
/// risk metrics, distribution of outcomes, and individual walk results.
///
/// The data in this structure can be used for strategy evaluation, risk assessment,
/// performance visualization, and comparison between different trading approaches.
///
/// ## Key Features:
/// - Probability analysis of profitable vs. losing outcomes
/// - Statistical measures of profit/loss distribution
/// - Detailed risk metrics at various confidence levels
/// - Profit/loss distribution data suitable for histogram visualization
/// - Results from individual simulation walks for granular analysis
///
/// ## Usage:
/// This structure is typically generated as the output of a strategy backtesting or
/// Monte Carlo simulation process, providing a comprehensive view of expected performance
/// characteristics and risk profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Number of simulation iterations run.
    pub iterations: u32,

    /// Overall probability of profit (percentage from 0.0 to 100.0).
    pub profit_probability: Positive,

    /// Overall probability of loss (percentage from 0.0 to 100.0).
    pub loss_probability: Positive,

    /// Maximum profit observed across all simulations.
    pub max_profit: Positive,

    /// Minimum profit (maximum loss) observed across all simulations.
    pub max_loss: Positive,

    /// Average profit/loss across all simulations.
    pub average_pnl: Decimal,

    /// Standard deviation of profit/loss.
    pub pnl_std_dev: Positive,

    /// Risk metrics categorized by severity.
    pub risk_levels: RiskMetricsSimulation,

    /// Probability distribution of PnL outcomes (can be used for histograms).
    /// The keys represent profit/loss ranges, and the values represent the probability
    /// of the outcome falling within that range.
    pub pnl_distribution: HashMap<PnLRange, Decimal>,

    /// Additional strategy-specific metrics.
    pub additional_metrics: HashMap<String, Decimal>,

    /// Detailed results for each individual simulation walk.
    /// Maps walk identifiers to their corresponding results.
    pub walk_results: HashMap<WalkId, WalkResult>,
}

impl Default for SimulationResult {
    fn default() -> Self {
        Self::new(0)
    }
}

impl SimulationResult {
    /// Creates a new, empty result container.
    ///
    /// # Arguments
    ///
    /// * `iterations` - The number of iterations the simulation will run.
    pub fn new(iterations: u32) -> Self {
        Self {
            iterations,
            profit_probability: pos!(0.01),
            loss_probability: pos!(0.01),
            max_profit: pos!(0.01),
            max_loss: pos!(0.01),
            average_pnl: dec!(0.0),
            pnl_std_dev: pos!(0.01), // Minimum positive value
            risk_levels: RiskMetricsSimulation::default(),
            pnl_distribution: HashMap::new(),
            additional_metrics: HashMap::new(),
            walk_results: HashMap::new(),
        }
    }

    /// Returns whether the strategy is generally favorable based on risk-adjusted metrics.
    /// This considers both average profit/loss and the Sharpe ratio.
    pub fn is_favorable(&self) -> bool {
        let is_positive_pnl = self.average_pnl.is_sign_positive();
        let sharpe_threshold = dec!(1.0);
        is_positive_pnl && self.risk_levels.sharpe_ratio > sharpe_threshold
    }

    /// Categorizes the risk level of this strategy based on simulated outcomes.
    /// Considers severe loss probability and Value at Risk (VaR).
    pub fn risk_category(&self) -> RiskCategory {
        // Thresholds for determining the risk category
        let severe_loss_threshold = pos!(0.25);
        let var_multiplier = dec!(2.0);

        // Determine risk category based on various metrics
        if self.risk_levels.severe_loss_probability > severe_loss_threshold {
            RiskCategory::High
        } else {
            let var_threshold = self.average_pnl * var_multiplier;
            if self.risk_levels.var_95.abs() > var_threshold {
                RiskCategory::Medium
            } else {
                RiskCategory::Low
            }
        }
    }

    /// Returns the probability of the simulated profit/loss falling within a given range.
    ///
    /// # Arguments
    ///
    /// * `range` - The PnL range to query.
    pub fn probability_in_range(&self, range: &PnLRange) -> Decimal {
        self.pnl_distribution
            .get(range)
            .cloned()
            .unwrap_or(Decimal::ZERO)
    }
}

/// Contains the results of a strategy on a single random walk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkResult {
    /// Initial price at the start of the walk
    pub initially: Decimal,

    /// Final price at the end of the walk
    pub finally: Decimal,

    /// Option payoff at last price
    pub payoff: Decimal,

    /// Percentage change in price from start to end
    pub change_percentage: Decimal,

    /// Prices diffs
    pub diff: Decimal,

    pub max_value: (Decimal, Decimal),

    /// Maximum drawdown observed during the walk
    pub min_value: (Decimal, Decimal),

    /// Collection of profit points throughout the walk
    pub positive_points: Vec<(Decimal, Decimal)>,

    /// Collection of drawdown points throughout the walk
    pub negative_points: Vec<(Decimal, Decimal)>,

    /// Map of hypothetical PnL at different price points
    pub pnl_at_prices: HashMap<Decimal, Decimal>,

    pub extra_metrics: HashMap<String, Decimal>,

    pub volatilities: Vec<Positive>,
}

#[cfg(test)]
mod tests_simulation_result {
    use super::*;
    use crate::pnl::model::PnLRange;
    use crate::pos;
    use crate::risk::{RiskCategory, RiskMetricsSimulation};
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    /// Helper function to create a sample WalkResult for testing
    fn create_test_walk_result() -> WalkResult {
        WalkResult {
            initially: dec!(100.0),
            finally: dec!(120.0),
            payoff: dec!(20.0),
            change_percentage: dec!(20.0),
            diff: dec!(20.0),
            max_value: (dec!(130.0), dec!(30.0)),
            min_value: (dec!(95.0), dec!(-5.0)),
            positive_points: vec![(dec!(105.0), dec!(5.0)), (dec!(120.0), dec!(20.0))],
            negative_points: vec![(dec!(95.0), dec!(-5.0))],
            pnl_at_prices: {
                let mut map = HashMap::new();
                map.insert(dec!(90.0), dec!(-10.0));
                map.insert(dec!(100.0), dec!(0.0));
                map.insert(dec!(110.0), dec!(10.0));
                map
            },
            extra_metrics: {
                let mut map = HashMap::new();
                map.insert("max_drawdown".to_string(), dec!(5.0));
                map
            },
            volatilities: vec![pos!(0.15), pos!(0.18)],
        }
    }

    #[test]
    fn test_simulation_result_new() {
        let result = SimulationResult::new(100);

        assert_eq!(result.iterations, 100);
        assert_eq!(result.profit_probability, pos!(0.01));
        assert_eq!(result.loss_probability, pos!(0.01));
        assert_eq!(result.max_profit, pos!(0.01));
        assert_eq!(result.max_loss, pos!(0.01));
        assert_eq!(result.average_pnl, dec!(0.0));
        assert_eq!(result.pnl_std_dev, pos!(0.01));
        assert!(result.pnl_distribution.is_empty());
        assert!(result.additional_metrics.is_empty());
        assert!(result.walk_results.is_empty());
    }

    #[test]
    fn test_simulation_result_default() {
        let result = SimulationResult::default();

        assert_eq!(result.iterations, 0);
        assert_eq!(result.profit_probability, pos!(0.01));
        assert_eq!(result.loss_probability, pos!(0.01));
    }

    #[test]
    fn test_is_favorable_positive() {
        let mut result = SimulationResult::new(100);
        result.average_pnl = dec!(150.0);

        // Set Sharpe ratio above threshold
        let risk_levels = RiskMetricsSimulation {
            var_95: Default::default(),
            var_99: Default::default(),
            cvar_95: Default::default(),
            severe_loss_probability: Default::default(),
            max_drawdown: Default::default(),
            sharpe_ratio: dec!(1.5),
        };
        result.risk_levels = risk_levels;
        assert!(result.is_favorable());
    }

    #[test]
    fn test_is_favorable_negative() {
        // Test case 1: Negative average PnL
        let mut result = SimulationResult::new(100);
        result.average_pnl = dec!(-50.0);

        let risk_levels = RiskMetricsSimulation {
            var_95: Default::default(),
            var_99: Default::default(),
            cvar_95: Default::default(),
            severe_loss_probability: Default::default(),
            max_drawdown: Default::default(),
            sharpe_ratio: dec!(1.5),
        };
        result.risk_levels = risk_levels;

        assert!(!result.is_favorable());

        // Test case 2: Low Sharpe ratio
        let mut result = SimulationResult::new(100);
        result.average_pnl = dec!(150.0); // Positive average PnL

        let risk_levels = RiskMetricsSimulation {
            var_95: Default::default(),
            var_99: Default::default(),
            cvar_95: Default::default(),
            severe_loss_probability: Default::default(),
            max_drawdown: Default::default(),
            sharpe_ratio: dec!(0.5),
        };
        result.risk_levels = risk_levels;

        assert!(!result.is_favorable());
    }

    #[test]
    fn test_risk_category_high() {
        let mut result = SimulationResult::new(100);

        // High risk: severe_loss_probability > 0.25
        let risk_levels = RiskMetricsSimulation {
            var_95: Default::default(),
            var_99: Default::default(),
            cvar_95: Default::default(),
            severe_loss_probability: pos!(0.3),
            max_drawdown: Default::default(),
            sharpe_ratio: Default::default(),
        };
        result.risk_levels = risk_levels;

        assert_eq!(result.risk_category(), RiskCategory::High);
    }

    #[test]
    fn test_risk_category_medium() {
        let mut result = SimulationResult::new(100);
        result.average_pnl = dec!(100.0);

        // Medium risk: VaR > average_pnl * 2
        let risk_levels = RiskMetricsSimulation {
            var_95: Default::default(),
            var_99: Default::default(),
            cvar_95: dec!(-250.0),
            severe_loss_probability: pos!(0.2),
            max_drawdown: Default::default(),
            sharpe_ratio: Default::default(),
        };
        result.risk_levels = risk_levels;

        assert_eq!(result.risk_category(), RiskCategory::Medium);
    }

    #[test]
    fn test_risk_category_low() {
        let mut result = SimulationResult::new(100);
        result.average_pnl = dec!(100.0);

        // Low risk: All metrics below thresholds
        let risk_levels = RiskMetricsSimulation {
            var_95: Default::default(),
            var_99: Default::default(),
            cvar_95: dec!(-150.0),
            severe_loss_probability: pos!(0.2),
            max_drawdown: Default::default(),
            sharpe_ratio: Default::default(),
        };

        result.risk_levels = risk_levels;

        assert_eq!(result.risk_category(), RiskCategory::Low);
    }

    #[test]
    fn test_probability_in_range() {
        let mut result = SimulationResult::new(100);

        // Add some PnL distribution data
        let mut pnl_distribution = HashMap::new();
        pnl_distribution.insert(PnLRange::new_decimal(dec!(-100.0), dec!(0.0)), dec!(0.4));
        pnl_distribution.insert(PnLRange::new_decimal(dec!(0.0), dec!(100.0)), dec!(0.3));
        result.pnl_distribution = pnl_distribution;

        // Test existing range
        let range1 = PnLRange::new_decimal(dec!(-100.0), dec!(0.0));
        assert_eq!(result.probability_in_range(&range1), dec!(0.4));

        // Test another existing range
        let range2 = PnLRange::new_decimal(dec!(0.0), dec!(100.0));
        assert_eq!(result.probability_in_range(&range2), dec!(0.3));

        // Test non-existing range
        let range3 = PnLRange::new_decimal(dec!(100.0), dec!(200.0));
        assert_eq!(result.probability_in_range(&range3), dec!(0.0));
    }

    #[test]
    fn test_walk_result_fields() {
        let walk_result = create_test_walk_result();

        assert_eq!(walk_result.initially, dec!(100.0));
        assert_eq!(walk_result.finally, dec!(120.0));
        assert_eq!(walk_result.payoff, dec!(20.0));
        assert_eq!(walk_result.change_percentage, dec!(20.0));
        assert_eq!(walk_result.diff, dec!(20.0));

        // Test max and min values
        assert_eq!(walk_result.max_value, (dec!(130.0), dec!(30.0)));
        assert_eq!(walk_result.min_value, (dec!(95.0), dec!(-5.0)));

        // Test points collections
        assert_eq!(walk_result.positive_points.len(), 2);
        assert_eq!(walk_result.negative_points.len(), 1);

        // Test PnL at prices
        assert_eq!(
            walk_result.pnl_at_prices.get(&dec!(100.0)),
            Some(&dec!(0.0))
        );
        assert_eq!(
            walk_result.pnl_at_prices.get(&dec!(110.0)),
            Some(&dec!(10.0))
        );

        // Test extra metrics
        assert_eq!(
            walk_result.extra_metrics.get("max_drawdown"),
            Some(&dec!(5.0))
        );

        // Test volatilities
        assert_eq!(walk_result.volatilities.len(), 2);
        assert_eq!(walk_result.volatilities[0], pos!(0.15));
    }
}
