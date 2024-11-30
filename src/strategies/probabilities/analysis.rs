/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 30/11/24
 ******************************************************************************/
use crate::model::types::PositiveF64;

#[allow(dead_code)]
#[derive(Debug)]
pub struct StrategyProbabilityAnalysis {
    /// The probability of profit (POP)
    pub probability_of_profit: PositiveF64,
    /// The probability of maximum profit
    pub probability_of_max_profit: PositiveF64,
    /// The probability of maximum loss
    pub probability_of_max_loss: PositiveF64,
    /// Expected value of the strategy
    pub expected_value: PositiveF64,
    /// Break-even points
    pub break_even_points: Vec<PositiveF64>,
    /// Risk-reward ratio
    pub risk_reward_ratio: PositiveF64,
}

