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
