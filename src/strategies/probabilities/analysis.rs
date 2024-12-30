/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/11/24
******************************************************************************/
use crate::Positive;

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
