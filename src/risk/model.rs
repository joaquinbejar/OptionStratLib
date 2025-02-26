/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/
use crate::{pos, Positive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Represents various risk metrics for the options strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetricsSimulation {
    /// Value at Risk (VaR) at 95% confidence level
    pub var_95: Decimal,

    /// Value at Risk (VaR) at 99% confidence level
    pub var_99: Decimal,

    /// Conditional Value at Risk (CVaR) at 95% confidence level (Expected Shortfall)
    pub cvar_95: Decimal,

    /// Probability of losing more than 50% of maximum investment
    pub severe_loss_probability: Positive,

    /// Maximum drawdown observed in simulations
    pub max_drawdown: Positive,

    /// Sharpe ratio (risk-adjusted return)
    pub sharpe_ratio: Decimal,
}

impl Default for RiskMetricsSimulation {
    fn default() -> Self {
        Self {
            var_95: dec!(0.0),
            var_99: dec!(0.0),
            cvar_95: dec!(0.0),
            severe_loss_probability: pos!(0.01),
            max_drawdown: pos!(0.01),
            sharpe_ratio: dec!(0.0),
        }
    }
}

/// Risk categories for options strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Low risk strategies (e.g., covered calls with strong underlying)
    Low,

    /// Medium risk strategies (e.g., credit spreads)
    Medium,

    /// High risk strategies (e.g., naked options, speculative positions)
    High,
}
