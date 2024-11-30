/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 30/11/24
 ******************************************************************************/
use crate::constants::ZERO;
use crate::model::ProfitRange;
use crate::model::types::{ExpirationDate, PositiveF64, PZERO};
use crate::pos;
use crate::strategies::base::Strategies;
use crate::strategies::probabilities::analysis::StrategyProbabilityAnalysis;
use crate::strategies::probabilities::utils::{ calculate_single_point_probability, PriceTrend, VolatilityAdjustment};

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

        let probability_of_profit = self.probability_of_profit(volatility_adj.clone(), trend.clone())?;
        let expected_value = self.expected_value(volatility_adj.clone(), trend.clone())?;

        // Calculate probability of max profit and max loss
        let (prob_max_profit, prob_max_loss) = self.calculate_extreme_probabilities(
            volatility_adj,
            trend,
        )?;

        // Calculate risk-reward ratio
        let risk_reward_ratio = pos!(self.profit_ratio());

        Ok(StrategyProbabilityAnalysis {
            probability_of_profit,
            probability_of_max_profit: prob_max_profit,
            probability_of_max_loss: prob_max_loss,
            expected_value,
            break_even_points: break_even_points.iter().copied().collect(),
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
                    (None, Some(upper)) => {
                        calculate_single_point_probability(
                            current_price,
                            upper,
                            volatility_adj,
                            trend,
                            expiration,
                            None,
                        )?
                    },
                    (Some(lower), None) => {
                        calculate_single_point_probability(
                            current_price,
                            lower,
                            volatility_adj,
                            trend,
                            expiration,
                            None,
                        )?
                    },
                    _ => return Err("Invalid profit range configuration".to_string()),
                };
                (prob_above, prob_below)
            },
            ranges => {
                let first = &ranges[0];
                let last = &ranges[ranges.len() - 1];

                let max_prob = match (first.lower_bound, last.upper_bound) {
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
                    },
                    _ => return Err("Invalid range configuration for multiple ranges".to_string()),
                };
                max_prob
            }
        };

        Ok((max_profit_prob, max_loss_prob))
    }

    // Required helper methods
    fn get_reference_price(&self) -> Result<PositiveF64, String>;
    fn get_expiration(&self) -> Result<ExpirationDate, String>;
    fn get_profit_ranges(&self) -> Result<Vec<ProfitRange>, String>;
}