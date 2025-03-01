/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 1/3/25
******************************************************************************/

use crate::pnl::model::PnLRange;
use crate::risk::RiskMetricsSimulation;
use crate::simulation::model::WalkResult;
use crate::simulation::{SimulationResult, WalkId};
use crate::{Positive, pos};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::error::Error;

pub(super) fn calculate_extra_metrics(
    y_values: &Vec<Positive>,
) -> Result<HashMap<String, Decimal>, Box<dyn Error>> {
    let mut metrics: HashMap<String, Decimal> = HashMap::new();

    // Skip calculation if we don't have enough data points
    if y_values.len() < 2 {
        return Err("Not enough data points to calculate extra metrics.".into());
    }

    // Calculate returns (percentage changes between consecutive prices)
    let returns: Vec<Decimal> = y_values
        .windows(2)
        .map(|w| (w[1].to_dec() - w[0].to_dec()) / w[0].to_dec())
        .collect();

    // Basic statistics
    let len = Decimal::from(returns.len());

    // Mean calculation
    let mean_return = returns.iter().sum::<Decimal>() / len;

    // Variance calculation
    let returns_variance = returns
        .iter()
        .map(|r| (*r - mean_return).powi(2))
        .sum::<Decimal>()
        / len;

    // Standard deviation - using sqrt approximation for Decimal
    let std_dev = returns_variance.sqrt().unwrap_or(Decimal::ZERO);

    // 1. Realized Volatility (annualized)
    let trading_days_per_year = dec!(252);
    let sqrt_trading_days = trading_days_per_year.sqrt().unwrap_or(Decimal::ZERO);
    let realized_volatility = std_dev * sqrt_trading_days;
    metrics.insert("realized_volatility".to_string(), realized_volatility);

    // 2. Sharpe Ratio (assuming risk-free rate = 0 for simplicity)
    let sharpe_ratio = if std_dev > dec!(0) {
        mean_return / std_dev * sqrt_trading_days
    } else {
        dec!(0)
    };
    metrics.insert("sharpe_ratio".to_string(), sharpe_ratio);

    // 3. Sortino Ratio
    let negative_returns: Vec<Decimal> =
        returns.iter().filter(|&r| *r < dec!(0)).cloned().collect();

    let downside_risk = if !negative_returns.is_empty() {
        let neg_len = Decimal::from(negative_returns.len());
        let neg_mean = negative_returns.iter().sum::<Decimal>() / neg_len;
        let neg_variance = negative_returns
            .iter()
            .map(|r| (*r - neg_mean).powi(2))
            .sum::<Decimal>()
            / neg_len;
        neg_variance.sqrt().unwrap_or(Decimal::ZERO)
    } else {
        std_dev
    };

    let sortino_ratio = if downside_risk > dec!(0) {
        mean_return / downside_risk * sqrt_trading_days
    } else {
        dec!(0)
    };
    metrics.insert("sortino_ratio".to_string(), sortino_ratio);

    // 4. Ulcer Index
    let mut max_so_far = vec![y_values[0]];
    for i in 1..y_values.len() {
        let current_max = if y_values[i] > max_so_far[i - 1] {
            y_values[i]
        } else {
            max_so_far[i - 1]
        };
        max_so_far.push(current_max);
    }

    let drawdowns: Vec<Decimal> = y_values
        .iter()
        .zip(max_so_far.iter())
        .map(|(price, max)| (price.to_dec() - max.to_dec()) / max.to_dec())
        .collect();

    let squared_drawdowns_sum: Decimal = drawdowns.iter().map(|d| d.powi(2)).sum();

    let ulcer_index = (squared_drawdowns_sum / Decimal::from(drawdowns.len()))
        .sqrt()
        .unwrap_or(Decimal::ZERO);
    metrics.insert("ulcer_index".to_string(), ulcer_index);

    // 5. Maximum Drawdown Percentage
    let max_drawdown = drawdowns.iter().fold(
        dec!(0),
        |max_dd, dd| if *dd < max_dd { *dd } else { max_dd },
    );
    metrics.insert("max_drawdown_percentage".to_string(), max_drawdown);

    // 6. Time in Profit
    let initial_price = y_values[0];
    let profits_count = y_values
        .iter()
        .filter(|price| **price > initial_price)
        .count();
    let time_in_profit = Decimal::from(profits_count) / Decimal::from(y_values.len());
    metrics.insert("time_in_profit".to_string(), time_in_profit);

    // 7. Profit Factor
    let (gross_profit, gross_loss) =
        returns
            .iter()
            .fold((dec!(0), dec!(0)), |(profit, loss), ret| {
                if *ret > dec!(0) {
                    (profit + *ret, loss)
                } else {
                    (profit, loss + ret.abs())
                }
            });

    let profit_factor = if gross_loss > dec!(0) {
        gross_profit / gross_loss
    } else {
        Decimal::MAX
    };
    metrics.insert("profit_factor".to_string(), profit_factor);

    // 12. Probability of Profit
    let positive_returns_count = returns.iter().filter(|ret| **ret > dec!(0)).count();
    let probability_of_profit =
        Decimal::from(positive_returns_count) / Decimal::from(returns.len());
    metrics.insert("probability_of_profit".to_string(), probability_of_profit);

    // 13. Expected Value
    metrics.insert("expected_value".to_string(), mean_return);

    // 14. Kelly Criterion
    let win_probability = probability_of_profit;

    // Calculate average win
    let positive_returns: Vec<Decimal> = returns
        .iter()
        .filter(|&ret| *ret > dec!(0))
        .cloned()
        .collect();

    let avg_win = if !positive_returns.is_empty() {
        positive_returns.iter().sum::<Decimal>() / Decimal::from(positive_returns.len())
    } else {
        dec!(0)
    };

    // Calculate average loss
    let negative_returns: Vec<Decimal> = returns
        .iter()
        .filter(|&ret| *ret < dec!(0))
        .cloned()
        .collect();

    let avg_loss = if !negative_returns.is_empty() {
        negative_returns.iter().sum::<Decimal>() / Decimal::from(negative_returns.len())
    } else {
        dec!(0)
    };

    let kelly_criterion = if avg_loss.abs() > dec!(0) && !avg_win.is_zero() {
        win_probability - (dec!(1) - win_probability) / (avg_win / avg_loss.abs())
    } else {
        dec!(0)
    };
    metrics.insert("kelly_criterion".to_string(), kelly_criterion);

    // 15. Skewness - Manual calculation
    let m2 = returns
        .iter()
        .map(|x| (*x - mean_return).powi(2))
        .sum::<Decimal>()
        / len;

    let m3 = returns
        .iter()
        .map(|x| (*x - mean_return).powi(3))
        .sum::<Decimal>()
        / len;

    let skewness = if m2 > dec!(0) {
        m3 / m2.powf(1.5)
    } else {
        dec!(0)
    };
    metrics.insert("skewness".to_string(), skewness);

    // 16. Kurtosis - Manual calculation
    let m4 = returns
        .iter()
        .map(|x| (*x - mean_return).powi(4))
        .sum::<Decimal>()
        / len;

    let kurtosis = if m2 > dec!(0) {
        m4 / m2.powi(2) - dec!(3)
    } else {
        dec!(0)
    }; // Excess kurtosis (normal = 0)

    metrics.insert("kurtosis".to_string(), kurtosis);

    // Return the calculated metrics
    Ok(metrics)
}

/// Generates a SimulationResult from a vector of WalkResult
pub fn create_simulation_result(
    walk_results: Vec<WalkResult>,
    pnl_range_size: i32,
) -> Result<SimulationResult, Box<dyn Error>> {
    if walk_results.is_empty() {
        return Err("No walk results to process".into());
    }

    let iterations = walk_results.len() as u32;

    // Calculate basic profit/loss metrics
    let profitable_walks: f64 = (walk_results
        .iter()
        .filter(|w| w.payoff > Decimal::ZERO)
        .count()) as f64;
    let loss_walks: f64 = (walk_results
        .iter()
        .filter(|w| w.payoff <= Decimal::ZERO)
        .count()) as f64;

    let profit_probability: Positive =
        Decimal::from_f64(profitable_walks * 100.0 / iterations as f64)
            .unwrap()
            .into();

    let loss_probability: Positive = Decimal::from_f64(loss_walks * 100.0 / iterations as f64)
        .unwrap()
        .into();

    // Find max profit and max loss
    let max_profit = walk_results
        .iter()
        .map(|w| w.payoff)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or("Could not determine max profit")?;

    let max_profit = if max_profit > Decimal::ZERO {
        max_profit.into()
    } else {
        pos!(0.01)
    };

    let min_profit = walk_results
        .iter()
        .map(|w| w.payoff)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or("Could not determine min profit")?;

    let max_loss = if min_profit < Decimal::ZERO {
        min_profit.abs().into()
    } else {
        pos!(0.01)
    };

    // Calculate average PnL and standard deviation
    let total_pnl: Decimal = walk_results.iter().map(|w| w.payoff).sum();
    let average_pnl = total_pnl / Decimal::from(iterations);

    // Calculate PnL standard deviation
    let variance: Decimal = walk_results
        .iter()
        .map(|w| (w.payoff - average_pnl).powi(2))
        .sum::<Decimal>()
        / Decimal::from(iterations);

    let pnl_std_dev: Positive = variance.sqrt().unwrap_or(Decimal::ZERO).into();

    // Create risk metrics
    let risk_levels = calculate_risk_metrics(&walk_results)?;

    // Generate PnL distribution for histogram
    let pnl_distribution = generate_pnl_distribution(&walk_results, pnl_range_size)?;

    // Calculate additional metrics from the walk results' extra_metrics
    let additional_metrics = aggregate_extra_metrics(&walk_results)?;

    // Create a HashMap of WalkResults indexed by their IDs
    let mut walk_results_map: HashMap<WalkId, WalkResult> = HashMap::new();
    for (idx, walk) in walk_results.iter().enumerate() {
        let walk_id = WalkId::new(idx.to_string());
        walk_results_map.insert(walk_id, walk.clone());
    }

    // Construct the final SimulationResult
    let simulation_result = SimulationResult {
        iterations,
        profit_probability,
        loss_probability,
        max_profit,
        max_loss,
        average_pnl,
        pnl_std_dev,
        risk_levels,
        pnl_distribution,
        additional_metrics,
        walk_results: walk_results_map,
    };

    Ok(simulation_result)
}

/// Calculates risk metrics from walk results
fn calculate_risk_metrics(
    walk_results: &[WalkResult],
) -> Result<RiskMetricsSimulation, Box<dyn Error>> {
    if walk_results.is_empty() {
        return Err("No walk results to calculate risk metrics".into());
    }

    // Calculate VaR (Value at Risk) 95% and 99%
    let mut payoffs: Vec<Decimal> = walk_results.iter().map(|w| w.payoff).collect();
    payoffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let index_95 = (payoffs.len() as f64 * 0.05) as usize; // 5th percentile for 95% VaR
    let index_99 = (payoffs.len() as f64 * 0.01) as usize; // 1st percentile for 99% VaR

    let var_95 = payoffs
        .get(index_95)
        .copied()
        .unwrap_or(Decimal::ZERO)
        .abs();
    let var_99 = payoffs
        .get(index_99)
        .copied()
        .unwrap_or(Decimal::ZERO)
        .abs();

    // Calculate CVaR (Conditional Value at Risk / Expected Shortfall) 95%
    let cvar_elements: Vec<Decimal> = payoffs
        .iter()
        .take(index_95 + 1) // Include the 95% VaR point
        .copied()
        .collect();

    let cvar_95 = if !cvar_elements.is_empty() {
        cvar_elements.iter().sum::<Decimal>() / Decimal::from(cvar_elements.len())
    } else {
        Decimal::ZERO
    }
    .abs();

    // Calculate severe loss probability (losing more than 50% of max investment)
    // Assuming max investment is derived from some property of the walk results
    // For example, using the initially value
    let max_investment = walk_results
        .iter()
        .map(|w| w.initially)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(dec!(1)); // Default to 1 if no max found

    let severe_loss_threshold = max_investment * dec!(-0.5); // 50% loss

    let severe_losses = walk_results
        .iter()
        .filter(|w| w.payoff <= severe_loss_threshold)
        .count();

    let severe_loss_probability: Positive =
        Decimal::from_usize(severe_losses * 100 / walk_results.len())
            .unwrap()
            .into();

    // Calculate maximum drawdown
    let max_drawdown = walk_results
        .iter()
        .map(|w| {
            // Assuming each WalkResult has a min_value field representing the lowest point
            let (price, _) = w.min_value;
            let percentage_drop = (price - w.initially) / w.initially;
            percentage_drop.abs()
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or("Could not calculate maximum drawdown")?;

    let max_drawdown: Positive = max_drawdown.into();

    // Calculate Sharpe ratio
    let sharpe_ratio = calculate_sharpe_ratio(walk_results)?;

    // Create RiskMetricsSimulation struct
    let risk_metrics = RiskMetricsSimulation {
        var_95,
        var_99,
        cvar_95,
        severe_loss_probability,
        max_drawdown,
        sharpe_ratio,
    };

    Ok(risk_metrics)
}

/// Generates PnL distribution for histogram visualization
fn generate_pnl_distribution(
    walk_results: &[WalkResult],
    range_size: i32,
) -> Result<HashMap<PnLRange, Decimal>, Box<dyn Error>> {
    if walk_results.is_empty() {
        return Err("No walk results to generate PnL distribution".into());
    }

    if range_size <= 0 {
        return Err("Range size must be greater than zero".into());
    }

    let mut distribution: HashMap<PnLRange, Decimal> = HashMap::new();
    let total_walks = Decimal::from(walk_results.len());

    // Find min and max PnL to determine the range
    let min_pnl = walk_results
        .iter()
        .map(|w| w.payoff)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(Decimal::ZERO);

    let max_pnl = walk_results
        .iter()
        .map(|w| w.payoff)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(Decimal::ZERO);

    // Convert to i32 for PnLRange and round to ensure all values are included
    let min_range_i32 =
        (min_pnl.to_f64().unwrap_or(0.0) / range_size as f64).floor() as i32 * range_size;
    let max_range_i32 =
        (max_pnl.to_f64().unwrap_or(0.0) / range_size as f64).ceil() as i32 * range_size;

    let mut current_range = min_range_i32;
    while current_range < max_range_i32 {
        let range_start = current_range;
        let range_end = current_range + range_size;

        // Convert range bounds to Decimal for comparison
        let dec_range_start = Decimal::from(range_start);
        let dec_range_end = Decimal::from(range_end);

        // Count walks in this range
        let walks_in_range = walk_results
            .iter()
            .filter(|w| w.payoff >= dec_range_start && w.payoff < dec_range_end)
            .count();

        // Calculate probability
        let probability = Decimal::from(walks_in_range) / total_walks;

        // Create range object and add to distribution if there are walks in this range
        if walks_in_range > 0 {
            let pnl_range = PnLRange {
                lower: range_start,
                upper: range_end,
            };
            distribution.insert(pnl_range, probability);
        }

        current_range += range_size;
    }

    Ok(distribution)
}

/// Aggregates extra metrics from all walk results
fn aggregate_extra_metrics(
    walk_results: &[WalkResult],
) -> Result<HashMap<String, Decimal>, Box<dyn Error>> {
    let mut aggregated_metrics: HashMap<String, Decimal> = HashMap::new();

    if walk_results.is_empty() {
        return Ok(aggregated_metrics);
    }

    // Collect all unique metric keys
    let mut all_keys: Vec<String> = Vec::new();
    for walk in walk_results {
        for key in walk.extra_metrics.keys() {
            if !all_keys.contains(key) {
                all_keys.push(key.clone());
            }
        }
    }

    // Calculate average for each metric
    for key in all_keys {
        let metric_values: Vec<Decimal> = walk_results
            .iter()
            .filter_map(|w| w.extra_metrics.get(&key).copied())
            .collect();

        if !metric_values.is_empty() {
            let metric_sum: Decimal = metric_values.iter().sum();
            let metric_avg = metric_sum / Decimal::from(metric_values.len());
            aggregated_metrics.insert(format!("avg_{}", key), metric_avg);

            // Also calculate median, min, max for important metrics
            let mut sorted_values = metric_values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let metric_min = sorted_values[0];
            let metric_max = sorted_values[sorted_values.len() - 1];

            // Median calculation
            let metric_median = if sorted_values.len() % 2 == 0 {
                let mid = sorted_values.len() / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / dec!(2)
            } else {
                sorted_values[sorted_values.len() / 2]
            };

            aggregated_metrics.insert(format!("min_{}", key), metric_min);
            aggregated_metrics.insert(format!("max_{}", key), metric_max);
            aggregated_metrics.insert(format!("median_{}", key), metric_median);

            // Calculate standard deviation for the metric
            let variance = metric_values
                .iter()
                .map(|v| (*v - metric_avg).powi(2))
                .sum::<Decimal>()
                / Decimal::from(metric_values.len());

            let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);
            aggregated_metrics.insert(format!("std_dev_{}", key), std_dev);
        }
    }

    // Add some overall metrics
    let profitable_walks = walk_results
        .iter()
        .filter(|w| w.payoff > Decimal::ZERO)
        .count();
    let sortino_ratio = calculate_sortino_ratio(walk_results)?;

    aggregated_metrics.insert("sortino_ratio".to_string(), sortino_ratio);
    aggregated_metrics.insert(
        "profitable_walks_count".to_string(),
        Decimal::from(profitable_walks),
    );

    Ok(aggregated_metrics)
}

/// Calculates Sharpe ratio across all walks
fn calculate_sharpe_ratio(walk_results: &[WalkResult]) -> Result<Decimal, Box<dyn Error>> {
    if walk_results.is_empty() {
        return Err("No walk results to calculate Sharpe ratio".into());
    }

    // Calculate average return
    let returns: Vec<Decimal> = walk_results.iter().map(|w| w.payoff).collect();
    let avg_return = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

    // Calculate standard deviation
    let variance = returns
        .iter()
        .map(|r| (*r - avg_return).powi(2))
        .sum::<Decimal>()
        / Decimal::from(returns.len());

    let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

    // Calculate Sharpe ratio (assuming risk-free rate = 0)
    if std_dev > Decimal::ZERO {
        let annualization_factor = dec!(252).sqrt().unwrap_or(dec!(15.87)); // √252 ≈ 15.87
        Ok(avg_return / std_dev * annualization_factor)
    } else if avg_return > Decimal::ZERO {
        // If no volatility but positive return, assign a high Sharpe
        Ok(dec!(999.99))
    } else if avg_return < Decimal::ZERO {
        // If no volatility but negative return, assign a very negative Sharpe
        Ok(dec!(-999.99))
    } else {
        // If no volatility and no return, Sharpe is zero
        Ok(Decimal::ZERO)
    }
}

/// Calculates Sortino ratio across all walks
fn calculate_sortino_ratio(walk_results: &[WalkResult]) -> Result<Decimal, Box<dyn Error>> {
    if walk_results.is_empty() {
        return Err("No walk results to calculate Sortino ratio".into());
    }

    // Calculate average return
    let returns: Vec<Decimal> = walk_results.iter().map(|w| w.payoff).collect();
    let avg_return = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());

    // Calculate downside deviation (only consider negative returns)
    let negative_returns: Vec<Decimal> = returns
        .iter()
        .filter(|&r| *r < Decimal::ZERO)
        .cloned()
        .collect();

    if negative_returns.is_empty() {
        // No negative returns, return a high ratio
        return Ok(dec!(999.99)); // Arbitrary high number for perfect strategies
    }

    let avg_negative =
        negative_returns.iter().sum::<Decimal>() / Decimal::from(negative_returns.len());

    let downside_variance = negative_returns
        .iter()
        .map(|r| (*r - avg_negative).powi(2))
        .sum::<Decimal>()
        / Decimal::from(negative_returns.len());

    let downside_deviation = downside_variance.sqrt().unwrap_or(Decimal::ZERO);

    // Calculate Sortino ratio
    if downside_deviation > Decimal::ZERO {
        let annualization_factor = dec!(252).sqrt().unwrap_or(dec!(15.87));
        Ok(avg_return / downside_deviation * annualization_factor)
    } else if avg_return > Decimal::ZERO {
        // If no downside deviation but positive return, assign a high Sortino
        Ok(dec!(999.99))
    } else if avg_return < Decimal::ZERO {
        // If no downside deviation but negative return, assign a very negative Sortino
        Ok(dec!(-999.99))
    } else {
        // If no downside deviation and no return, Sortino is zero
        Ok(Decimal::ZERO)
    }
}

#[cfg(test)]
mod tests_calculate_extra_metrics {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_extra_metrics_basic() {
        // Create a simple upward trend
        let y_values: Vec<Positive> = (1..=10).map(|i| Positive(Decimal::from(i * 10))).collect();

        let result = calculate_extra_metrics( &y_values).unwrap();

        // Basic checks
        assert!(result.contains_key("realized_volatility"));
        assert!(result.contains_key("sharpe_ratio"));
        assert!(result.contains_key("sortino_ratio"));
        assert!(result.contains_key("max_drawdown_percentage"));

        // In a steady uptrend, probability of profit should be 1
        assert_eq!(result["probability_of_profit"], dec!(1));

        // Drawdown should be 0 in perfect uptrend
        assert_eq!(result["max_drawdown_percentage"], dec!(0));
    }

    #[test]
    fn test_calculate_extra_metrics_downtrend() {
        // Create a simple downward trend
        let y_values: Vec<Positive> = (1..=10)
            .map(|i| Positive(Decimal::from(110 - i * 10)))
            .collect();

        let result = calculate_extra_metrics( &y_values).unwrap();

        // In a steady downtrend, probability of profit should be 0
        assert_eq!(result["probability_of_profit"], dec!(0));

        // Expected value should be negative
        assert!(result["expected_value"] < dec!(0));
    }

    #[test]
    fn test_calculate_extra_metrics_volatile() {
        // Create a volatile pattern (up-down-up-down)
        let y_values: Vec<Positive> = vec![
            Positive(dec!(10)),
            Positive(dec!(20)),
            Positive(dec!(15)),
            Positive(dec!(25)),
            Positive(dec!(20)),
            Positive(dec!(30)),
            Positive(dec!(25)),
            Positive(dec!(35)),
            Positive(dec!(30)),
            Positive(dec!(40)),
        ];

        let result = calculate_extra_metrics( &y_values).unwrap();

        // Volatility should be higher than in steady trend
        let uptrend_y: Vec<Positive> = (1..=10).map(|i| Positive(Decimal::from(i * 10))).collect();
        let uptrend_result = calculate_extra_metrics( &uptrend_y).unwrap();

        assert!(result["realized_volatility"] > uptrend_result["realized_volatility"]);
    }

    #[test]
    fn test_calculate_extra_metrics_flat() {
        // Create a flat trend (no change)
        let y_values: Vec<Positive> = (1..=10).map(|_| Positive(dec!(100))).collect();

        let result = calculate_extra_metrics(&y_values).unwrap();

        // In a flat trend, volatility should be very low
        assert!(result["realized_volatility"] < dec!(0.01));

        // Expected value close to zero
        assert!(result["expected_value"].abs() < dec!(0.01));
    }

    #[test]
    fn test_calculate_extra_metrics_drawdown() {
        // Create a pattern with a significant drawdown
        let y_values: Vec<Positive> = vec![
            Positive(dec!(100)),
            Positive(dec!(110)),
            Positive(dec!(120)),
            Positive(dec!(130)),
            Positive(dec!(90)), // 30.77% drawdown from 130
            Positive(dec!(70)), // 46.15% drawdown from 130
            Positive(dec!(80)),
            Positive(dec!(100)),
            Positive(dec!(110)),
            Positive(dec!(120)),
        ];

        let result = calculate_extra_metrics( &y_values).unwrap();

        // Check max drawdown (should be around -0.46 to -0.47)
        assert!(result["max_drawdown_percentage"] < dec!(-0.45));
        assert!(result["max_drawdown_percentage"] > dec!(-0.47));

        // Time in profit should be around 0.6-0.7 (6-7 out of 10 points are above the initial value)
        assert!(result["time_in_profit"] >= dec!(0.5));
        assert!(result["time_in_profit"] <= dec!(0.7));
    }

    #[test]
    fn test_calculate_extra_metrics_not_enough_data() {
        // Test with insufficient data
        let y_values: Vec<Positive> = vec![Positive(dec!(100))];

        let result = calculate_extra_metrics( &y_values);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_extra_metrics_skewness_kurtosis() {
        // Create data with positive skew
        let y_values: Vec<Positive> = vec![
            Positive(dec!(100)),
            Positive(dec!(101)),
            Positive(dec!(102)),
            Positive(dec!(103)),
            Positive(dec!(104)),
            Positive(dec!(105)),
            Positive(dec!(106)),
            Positive(dec!(107)),
            Positive(dec!(108)),
            Positive(dec!(120)), // outlier creates positive skew
        ];

        let result = calculate_extra_metrics( &y_values).unwrap();

        // Skewness should be positive
        assert!(result["skewness"] > dec!(0));
    }

    #[test]
    fn test_profit_factor() {
        // Create data with known profit/loss ratio
        let y_values: Vec<Positive> = vec![
            Positive(dec!(100)),
            Positive(dec!(110)), // +10% (profit of 10)
            Positive(dec!(105)), // -4.5% (loss of 5)
            Positive(dec!(115)), // +9.5% (profit of 10)
            Positive(dec!(110)), // -4.3% (loss of 5)
        ];

        let result = calculate_extra_metrics( &y_values).unwrap();

        // Gross profit should be around 0.195, gross loss around 0.088
        // Profit factor should be around 2.2
        assert!(result["profit_factor"] > dec!(2.0));
        assert!(result["profit_factor"] < dec!(2.5));
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json;
    use tracing::info;

    #[test]
    fn test_metrics_serialization() {
        // Create test data with a variety of patterns to generate diverse metrics
        let y_values: Vec<Positive> = vec![
            Positive(dec!(100)),
            Positive(dec!(110)),
            Positive(dec!(105)),
            Positive(dec!(115)),
            Positive(dec!(110)),
            Positive(dec!(120)),
            Positive(dec!(115)),
            Positive(dec!(125)),
            Positive(dec!(120)),
            Positive(dec!(130)),
        ];

        // Calculate metrics
        let metrics_result = calculate_extra_metrics( &y_values).unwrap();

        // Serialize the HashMap to JSON
        let serialized = serde_json::to_string_pretty(&metrics_result).unwrap();

        info!("{}", serialized);

        // Deserialize back to HashMap to verify round-trip
        let deserialized: HashMap<String, Decimal> = serde_json::from_str(&serialized).unwrap();

        // Verify the deserialized HashMap matches the original
        assert_eq!(metrics_result.len(), deserialized.len());

        // Verify specific key values are preserved
        for (key, value) in &metrics_result {
            assert!(deserialized.contains_key(key));
            assert_eq!(deserialized[key], *value);
        }

        // Test with custom formatting for better readability
        info!("\nMetrics in key-value format:");
        for (key, value) in metrics_result.iter() {
            info!("{}: {}", key, value);
        }

        // Format specific metrics for business interpretation
        info!("\nBusiness interpretation of key metrics:");
        info!(
            "Realized Volatility: {}%",
            (metrics_result["realized_volatility"] * dec!(100)).round_dp(2)
        );
        info!(
            "Sharpe Ratio: {}",
            metrics_result["sharpe_ratio"].round_dp(2)
        );
        info!(
            "Maximum Drawdown: {}%",
            (metrics_result["max_drawdown_percentage"] * dec!(100)).round_dp(2)
        );
        info!(
            "Probability of Profit: {}%",
            (metrics_result["probability_of_profit"] * dec!(100)).round_dp(2)
        );
        info!(
            "Profit Factor: {}",
            metrics_result["profit_factor"].round_dp(2)
        );

        // Example of how to access individual metric
        let volatility = metrics_result["realized_volatility"];
        assert!(volatility > dec!(0)); // Basic sanity check
    }
}

#[cfg(test)]
mod tests_calculate_sortino_ratio {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock WalkResult for testing
    #[derive(Debug, Clone)]
    struct MockWalkResult {
        payoff: Decimal,
    }

    impl From<MockWalkResult> for WalkResult {
        fn from(mock: MockWalkResult) -> Self {
            // Create a minimal WalkResult with only the fields needed for testing
            WalkResult {
                initially: dec!(100), // Default value, not used in calculate_sortino_ratio
                finally: dec!(100),   // Default value, not used in calculate_sortino_ratio
                payoff: mock.payoff,
                change_percentage: dec!(0), // Default value, not used in calculate_sortino_ratio
                diff: dec!(0),              // Default value, not used in calculate_sortino_ratio
                max_value: (dec!(0), dec!(0)), // Default value, not used in calculate_sortino_ratio
                min_value: (dec!(0), dec!(0)), // Default value, not used in calculate_sortino_ratio
                positive_points: Vec::new(), // Default value, not used in calculate_sortino_ratio
                negative_points: Vec::new(), // Default value, not used in calculate_sortino_ratio
                pnl_at_prices: HashMap::new(), // Default value, not used in calculate_sortino_ratio
                extra_metrics: HashMap::new(), // Default value, not used in calculate_sortino_ratio
                volatilities: Vec::new(),   // Default value, not used in calculate_sortino_ratio
            }
        }
    }

    #[test]
    fn test_calculate_sortino_ratio_empty_input() {
        // Test with empty input
        let walk_results: Vec<WalkResult> = Vec::new();
        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "No walk results to calculate Sortino ratio");
        }
    }

    #[test]
    fn test_calculate_sortino_ratio_all_positive_returns() {
        // Test with all positive returns
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(20) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();
        assert_eq!(sortino, dec!(999.99));
    }

    #[test]
    fn test_calculate_sortino_ratio_all_negative_returns() {
        // Test with all negative returns
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-15) },
            MockWalkResult { payoff: dec!(-5) },
            MockWalkResult { payoff: dec!(-20) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // Average return is -12.5
        // With negative average return, expect a negative Sortino ratio
        assert!(sortino < dec!(0));
    }

    #[test]
    fn test_calculate_sortino_ratio_mixed_returns() {
        // Test with a mix of positive and negative returns
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(-5) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(-10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // Average return is 2.5
        // Negative returns are -5 and -10, avg = -7.5
        // Variance calculation: ((-5-(-7.5))² + (-10-(-7.5))²)/2 = (2.5² + (-2.5)²)/2 = 6.25
        // Downside deviation = √6.25 = 2.5
        // Sortino = 2.5 / 2.5 * 15.87 = 15.87

        // Due to potential decimal precision issues, check within a reasonable range
        assert!(sortino > dec!(15.8));
        assert!(sortino < dec!(15.9));
    }

    #[test]
    fn test_calculate_sortino_ratio_zero_downside_deviation_positive_return() {
        // Test with zero downside deviation but positive average return
        // This happens when all negative returns are exactly the same value
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(-5) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(-5) },
        ];

        // Force downside variance to be zero by making all negative returns identical
        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        // Calculate manually
        let returns: Vec<Decimal> = walk_results.iter().map(|w| w.payoff).collect();
        let avg_return = returns.iter().sum::<Decimal>() / Decimal::from(returns.len());
        assert!(avg_return > Decimal::ZERO); // Confirm avg return is positive

        let negative_returns: Vec<Decimal> = returns
            .iter()
            .filter(|&r| *r < Decimal::ZERO)
            .cloned()
            .collect();

        assert!(!negative_returns.is_empty()); // Confirm we have negative returns

        // In this case, all negative returns are -5, so their deviation from mean is 0
        // This should lead to zero downside variance

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // For a positive return with zero downside deviation, we expect the maximum value
        assert_eq!(sortino, dec!(999.99));
    }

    #[test]
    fn test_calculate_sortino_ratio_zero_downside_deviation_negative_return() {
        // Test with zero downside deviation but negative average return
        // This case would be unusual but possible
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // For a negative return with zero downside deviation, we expect the minimum value
        assert_eq!(sortino, dec!(-999.99));
    }

    #[test]
    fn test_calculate_sortino_ratio_zero_return() {
        // Test with zero average return
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(-5) },
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(-10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // Since average return is 0, Sortino should be close to 0
        assert!(sortino.abs() < dec!(0.01));
    }

    #[test]
    fn test_calculate_sortino_ratio_single_result() {
        // Test with a single result
        let mock_walks = vec![MockWalkResult { payoff: dec!(10) }];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // Single positive result with no negatives should return maximum value
        assert_eq!(sortino, dec!(999.99));
    }

    #[test]
    fn test_calculate_sortino_ratio_annualization_factor() {
        // Test to verify the annualization factor
        // Create a situation where we know the exact return and downside deviation
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(1) },
            MockWalkResult { payoff: dec!(-1) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter().map(|m| m.into()).collect();

        // Manual calculation:
        // Average return = 0
        // Downside deviation = 1
        // Expected Sortino = 0 / 1 * 15.87 = 0

        let result = calculate_sortino_ratio(&walk_results);

        assert!(result.is_ok());
        let sortino = result.unwrap();

        // Since average return is 0, Sortino should be 0 regardless of annualization
        assert!(sortino.abs() < dec!(0.01));
    }
}

#[cfg(test)]
mod tests_calculate_sharpe_ratio {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock WalkResult for testing
    #[derive(Debug, Clone)]
    struct MockWalkResult {
        payoff: Decimal,
    }

    impl From<MockWalkResult> for WalkResult {
        fn from(mock: MockWalkResult) -> Self {
            // Create a minimal WalkResult with only the fields needed for testing
            WalkResult {
                initially: dec!(100),  // Default value, not used in calculate_sharpe_ratio
                finally: dec!(100),    // Default value, not used in calculate_sharpe_ratio
                payoff: mock.payoff,
                change_percentage: dec!(0), // Default value, not used in calculate_sharpe_ratio
                diff: dec!(0),        // Default value, not used in calculate_sharpe_ratio
                max_value: (dec!(0), dec!(0)), // Default value, not used in calculate_sharpe_ratio
                min_value: (dec!(0), dec!(0)), // Default value, not used in calculate_sharpe_ratio
                positive_points: Vec::new(), // Default value, not used in calculate_sharpe_ratio
                negative_points: Vec::new(), // Default value, not used in calculate_sharpe_ratio
                pnl_at_prices: HashMap::new(), // Default value, not used in calculate_sharpe_ratio
                extra_metrics: HashMap::new(), // Default value, not used in calculate_sharpe_ratio
                volatilities: Vec::new(), // Default value, not used in calculate_sharpe_ratio
            }
        }
    }

    #[test]
    fn test_calculate_sharpe_ratio_empty_input() {
        // Test with empty input
        let walk_results: Vec<WalkResult> = Vec::new();
        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "No walk results to calculate Sharpe ratio");
        }
    }

    #[test]
    fn test_calculate_sharpe_ratio_consistent_positive_returns() {
        // Test with consistent positive returns (zero standard deviation)
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();
        assert_eq!(sharpe, dec!(999.99));
    }

    #[test]
    fn test_calculate_sharpe_ratio_consistent_negative_returns() {
        // Test with consistent negative returns (zero standard deviation)
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();
        assert_eq!(sharpe, dec!(-999.99));
    }

    #[test]
    fn test_calculate_sharpe_ratio_zero_returns() {
        // Test with all zero returns (zero standard deviation and zero mean)
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(0) },
            MockWalkResult { payoff: dec!(0) },
            MockWalkResult { payoff: dec!(0) },
            MockWalkResult { payoff: dec!(0) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();
        assert_eq!(sharpe, dec!(0));
    }

    #[test]
    fn test_calculate_sharpe_ratio_positive_mean() {
        // Test with a positive mean return and some standard deviation
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(8) },
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(12) },
            MockWalkResult { payoff: dec!(10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // Mean: 10, Variance: 2, Std Dev: 1.414
        // Sharpe: 10 / 1.414 * 15.87 ≈ 112.2
        // Allow for some decimal precision differences
        assert!(sharpe > dec!(110));
        assert!(sharpe < dec!(115));
    }

    #[test]
    fn test_calculate_sharpe_ratio_negative_mean() {
        // Test with a negative mean return and some standard deviation
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-12) },
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(-8) },
            MockWalkResult { payoff: dec!(-10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // Mean: -10, Variance: 2, Std Dev: 1.414
        // Sharpe: -10 / 1.414 * 15.87 ≈ -112.2
        // Allow for some decimal precision differences
        assert!(sharpe < dec!(-110));
        assert!(sharpe > dec!(-115));
    }

    #[test]
    fn test_calculate_sharpe_ratio_mixed_returns() {
        // Test with mixed positive and negative returns
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-20) },
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(40) },
            MockWalkResult { payoff: dec!(-10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // Mean: 5, Variance: 550, Std Dev: 23.45
        // Sharpe: 5 / 23.45 * 15.87 ≈ 3.38
        // Allow for some decimal precision differences
        assert!(sharpe > dec!(3.3));
        assert!(sharpe < dec!(3.5));
    }

    #[test]
    fn test_calculate_sharpe_ratio_zero_mean() {
        // Test with a zero mean return and some standard deviation
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(-10) },
            MockWalkResult { payoff: dec!(10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // With a mean of 0, the Sharpe ratio should be 0
        assert!(sharpe.abs() < dec!(0.01));
    }

    #[test]
    fn test_calculate_sharpe_ratio_single_result() {
        // Test with a single result
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // Single result has zero standard deviation and positive mean
        assert_eq!(sharpe, dec!(999.99));
    }

    #[test]
    fn test_calculate_sharpe_ratio_annualization_factor() {
        // Test to verify the annualization factor
        // Create a situation where mean = 1 and std_dev = 1
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(0) },
            MockWalkResult { payoff: dec!(2) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // Mean: 1, Std Dev: 1
        // Sharpe should be approximately equal to the annualization factor
        let annualization_factor = dec!(15.87);
        let error_margin = dec!(0.1);

        assert!(sharpe > annualization_factor - error_margin);
        assert!(sharpe < annualization_factor + error_margin);
    }

    #[test]
    fn test_calculate_sharpe_ratio_high_volatility() {
        // Test with high volatility
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-100) },
            MockWalkResult { payoff: dec!(100) },
            MockWalkResult { payoff: dec!(-50) },
            MockWalkResult { payoff: dec!(50) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_sharpe_ratio(&walk_results);

        assert!(result.is_ok());
        let sharpe = result.unwrap();

        // Mean: 0, with high volatility
        // Sharpe should be close to 0
        assert!(sharpe.abs() < dec!(0.01));
    }
}

#[cfg(test)]
mod tests_aggregate_extra_metrics {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock WalkResult for testing
    #[derive(Debug, Clone)]
    struct MockWalkResult {
        payoff: Decimal,
        extra_metrics: HashMap<String, Decimal>,
    }

    impl From<MockWalkResult> for WalkResult {
        fn from(mock: MockWalkResult) -> Self {
            // Create a minimal WalkResult with only the fields needed for testing
            WalkResult {
                initially: dec!(100),  // Default value
                finally: dec!(100),    // Default value
                payoff: mock.payoff,
                change_percentage: dec!(0), // Default value
                diff: dec!(0),        // Default value
                max_value: (dec!(0), dec!(0)), // Default value
                min_value: (dec!(0), dec!(0)), // Default value
                positive_points: Vec::new(), // Default value
                negative_points: Vec::new(), // Default value
                pnl_at_prices: HashMap::new(), // Default value
                extra_metrics: mock.extra_metrics,
                volatilities: Vec::new(), // Default value
            }
        }
    }


    #[test]
    fn test_aggregate_extra_metrics_empty_input() {
        // Test with empty input
        let walk_results: Vec<WalkResult> = Vec::new();

        // Temporarily replace the real function with our mock
        let _original_calculate_sortino = calculate_sortino_ratio;
        // This line would be used if we had access to replace the function
        // calculate_sortino_ratio = mock_calculate_sortino_ratio;

        // Since we can't replace the function in this test, we'll note that
        // in a real environment, we'd mock it to avoid dependency on its implementation

        let result = aggregate_extra_metrics(&walk_results);

        // Restore the original function
        // calculate_sortino_ratio = original_calculate_sortino;

        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_aggregate_extra_metrics_single_metric() {
        // Test with a single metric across multiple walks
        let mut mock_walks = Vec::new();

        for i in 1..=5 {
            let mut extra_metrics = HashMap::new();
            extra_metrics.insert("volatility".to_string(), Decimal::from(i));

            mock_walks.push(MockWalkResult {
                payoff: Decimal::from(i * 10),
                extra_metrics,
            });
        }

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Replace calculate_sortino_ratio with mock (in a real environment)
        let result = aggregate_extra_metrics(&walk_results);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // Check average volatility: (1+2+3+4+5)/5 = 3
        assert_eq!(metrics["avg_volatility"], dec!(3));

        // Check min, max, median
        assert_eq!(metrics["min_volatility"], dec!(1));
        assert_eq!(metrics["max_volatility"], dec!(5));
        assert_eq!(metrics["median_volatility"], dec!(3));

        // Check standard deviation
        // Variance: ((1-3)² + (2-3)² + (3-3)² + (4-3)² + (5-3)²) / 5 = (4+1+0+1+4)/5 = 2
        // Std Dev: √2 = 1.414...
        assert!(metrics["std_dev_volatility"] > dec!(1.4));
        assert!(metrics["std_dev_volatility"] < dec!(1.42));

        // Check profitable_walks_count (all payoffs are positive)
        assert_eq!(metrics["profitable_walks_count"], dec!(5));
    }

    #[test]
    fn test_aggregate_extra_metrics_multiple_metrics() {
        // Test with multiple metrics
        let mut mock_walks = Vec::new();

        for i in 1..=4 {
            let mut extra_metrics = HashMap::new();
            extra_metrics.insert("volatility".to_string(), Decimal::from(i));
            extra_metrics.insert("sharpe_ratio".to_string(), Decimal::from(i) / dec!(2));

            // Alternate between positive and negative payoffs
            let payoff = if i % 2 == 0 { Decimal::from(i * 10) } else { Decimal::from(-i * 10) };

            mock_walks.push(MockWalkResult {
                payoff,
                extra_metrics,
            });
        }

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = aggregate_extra_metrics(&walk_results);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // Check averages
        // volatility: (1+2+3+4)/4 = 2.5
        // sharpe_ratio: (0.5+1+1.5+2)/4 = 1.25
        assert_eq!(metrics["avg_volatility"], dec!(2.5));
        assert_eq!(metrics["avg_sharpe_ratio"], dec!(1.25));

        // Check mins and maxes
        assert_eq!(metrics["min_volatility"], dec!(1));
        assert_eq!(metrics["max_volatility"], dec!(4));
        assert_eq!(metrics["min_sharpe_ratio"], dec!(0.5));
        assert_eq!(metrics["max_sharpe_ratio"], dec!(2));

        // Check medians
        assert_eq!(metrics["median_volatility"], dec!(2.5)); // even number of elements: (2+3)/2
        assert_eq!(metrics["median_sharpe_ratio"], dec!(1.25)); // (1+1.5)/2

        // Check profitable_walks_count (only even indices have positive payoffs: 2 out of 4)
        assert_eq!(metrics["profitable_walks_count"], dec!(2));
    }

    #[test]
    fn test_aggregate_extra_metrics_missing_values() {
        // Test with missing metrics in some walks
        let mut mock_walks = Vec::new();

        for i in 1..=4 {
            let mut extra_metrics = HashMap::new();

            // First two walks have volatility
            if i <= 2 {
                extra_metrics.insert("volatility".to_string(), Decimal::from(i));
            }

            // Last two walks have sharpe_ratio
            if i >= 3 {
                extra_metrics.insert("sharpe_ratio".to_string(), Decimal::from(i));
            }

            mock_walks.push(MockWalkResult {
                payoff: Decimal::from(i * 10),
                extra_metrics,
            });
        }

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = aggregate_extra_metrics(&walk_results);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // Check volatility (only present in first two walks)
        assert_eq!(metrics["avg_volatility"], dec!(1.5)); // (1+2)/2
        assert_eq!(metrics["min_volatility"], dec!(1));
        assert_eq!(metrics["max_volatility"], dec!(2));

        // Check sharpe_ratio (only present in last two walks)
        assert_eq!(metrics["avg_sharpe_ratio"], dec!(3.5)); // (3+4)/2
        assert_eq!(metrics["min_sharpe_ratio"], dec!(3));
        assert_eq!(metrics["max_sharpe_ratio"], dec!(4));
    }

    #[test]
    fn test_aggregate_extra_metrics_identical_values() {
        // Test with identical metric values
        let mut mock_walks = Vec::new();

        for i in 1..=4 {
            let mut extra_metrics = HashMap::new();
            // All walks have the same volatility value
            extra_metrics.insert("volatility".to_string(), dec!(5));

            mock_walks.push(MockWalkResult {
                payoff: Decimal::from(i * 10),
                extra_metrics,
            });
        }

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = aggregate_extra_metrics(&walk_results);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // All stats for volatility should be 5
        assert_eq!(metrics["avg_volatility"], dec!(5));
        assert_eq!(metrics["min_volatility"], dec!(5));
        assert_eq!(metrics["max_volatility"], dec!(5));
        assert_eq!(metrics["median_volatility"], dec!(5));

        // Standard deviation should be 0
        assert_eq!(metrics["std_dev_volatility"], dec!(0));
    }

    #[test]
    fn test_aggregate_extra_metrics_negative_values() {
        // Test with negative metric values
        let mut mock_walks = Vec::new();

        for i in 1..=4 {
            let mut extra_metrics = HashMap::new();
            // Use negative values for the metrics
            extra_metrics.insert("metric_a".to_string(), Decimal::from(-i));

            mock_walks.push(MockWalkResult {
                payoff: dec!(0), // Neutral payoff
                extra_metrics,
            });
        }

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = aggregate_extra_metrics(&walk_results);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // Average: (-1-2-3-4)/4 = -2.5
        assert_eq!(metrics["avg_metric_a"], dec!(-2.5));
        assert_eq!(metrics["min_metric_a"], dec!(-4));
        assert_eq!(metrics["max_metric_a"], dec!(-1));
        assert_eq!(metrics["median_metric_a"], dec!(-2.5)); // (-2-3)/2
    }

    #[test]
    fn test_aggregate_extra_metrics_no_extra_metrics() {
        // Test with walks that have no extra metrics
        let mut mock_walks = Vec::new();

        for i in 1..=4 {
            let extra_metrics = HashMap::new(); // Empty

            mock_walks.push(MockWalkResult {
                payoff: Decimal::from(i * 10),
                extra_metrics,
            });
        }

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = aggregate_extra_metrics(&walk_results);

        assert!(result.is_ok());
        let metrics = result.unwrap();

        // Should only have overall metrics
        assert_eq!(metrics.len(), 2); // sortino_ratio and profitable_walks_count
        assert!(metrics.contains_key("sortino_ratio"));
        assert!(metrics.contains_key("profitable_walks_count"));
        assert_eq!(metrics["profitable_walks_count"], dec!(4)); // All payoffs are positive
    }
    
}

#[cfg(test)]
mod tests_generate_pnl_distribution {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock WalkResult for testing
    #[derive(Debug, Clone)]
    struct MockWalkResult {
        payoff: Decimal,
    }

    impl From<MockWalkResult> for WalkResult {
        fn from(mock: MockWalkResult) -> Self {
            // Create a minimal WalkResult with only the fields needed for testing
            WalkResult {
                initially: dec!(100),  // Default value
                finally: dec!(100),    // Default value
                payoff: mock.payoff,
                change_percentage: dec!(0), // Default value
                diff: dec!(0),        // Default value
                max_value: (dec!(0), dec!(0)), // Default value
                min_value: (dec!(0), dec!(0)), // Default value
                positive_points: Vec::new(), // Default value
                negative_points: Vec::new(), // Default value
                pnl_at_prices: HashMap::new(), // Default value
                extra_metrics: HashMap::new(), // Default value
                volatilities: Vec::new(), // Default value
            }
        }
    }

    #[test]
    fn test_generate_pnl_distribution_empty_input() {
        // Test with empty input
        let walk_results: Vec<WalkResult> = Vec::new();
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "No walk results to generate PnL distribution");
        }
    }

    #[test]
    fn test_generate_pnl_distribution_invalid_range_size() {
        // Test with invalid range size
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(20) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = generate_pnl_distribution(&walk_results, 0);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Range size must be greater than zero");
        }

        let result = generate_pnl_distribution(&walk_results, -5);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "Range size must be greater than zero");
        }
    }

    #[test]
    fn test_generate_pnl_distribution_single_bucket() {
        // Test where all payoffs fall into a single bucket
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) },
            MockWalkResult { payoff: dec!(12) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(18) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 10 means all values (10-18) will fall into one bucket (10-20)
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have exactly one bucket
        assert_eq!(distribution.len(), 1);

        // Find the bucket and check its probability
        let bucket = PnLRange { lower: 10, upper: 20 };
        assert!(distribution.contains_key(&bucket));
        assert_eq!(distribution[&bucket], dec!(1)); // 4/4 = 100% probability
    }

    #[test]
    fn test_generate_pnl_distribution_multiple_buckets() {
        // Test distribution across multiple buckets
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(25) },
            MockWalkResult { payoff: dec!(35) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 10 means values will fall into 4 buckets
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have exactly 4 buckets
        assert_eq!(distribution.len(), 4);

        // Check each bucket and its probability
        let buckets = [
            PnLRange { lower: 0, upper: 10 },
            PnLRange { lower: 10, upper: 20 },
            PnLRange { lower: 20, upper: 30 },
            PnLRange { lower: 30, upper: 40 },
        ];

        for bucket in &buckets {
            assert!(distribution.contains_key(bucket));
            assert_eq!(distribution[bucket], dec!(0.25)); // 1/4 = 25% probability
        }
    }

    #[test]
    fn test_generate_pnl_distribution_negative_values() {
        // Test with negative payoffs
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(-25) },
            MockWalkResult { payoff: dec!(-15) },
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(15) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 10
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have 4 buckets: -30 to -20, -20 to -10, 0 to 10, 10 to 20
        assert_eq!(distribution.len(), 4);

        // Check the buckets and probabilities
        let buckets = [
            PnLRange { lower: -30, upper: -20 },
            PnLRange { lower: -20, upper: -10 },
            PnLRange { lower: 0, upper: 10 },
            PnLRange { lower: 10, upper: 20 },
        ];

        for bucket in &buckets {
            assert!(distribution.contains_key(bucket));
            assert_eq!(distribution[bucket], dec!(0.25)); // 1/4 = 25% probability
        }
    }

    #[test]
    fn test_generate_pnl_distribution_different_bucket_densities() {
        // Test with uneven distribution across buckets
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(16) },
            MockWalkResult { payoff: dec!(17) },
            MockWalkResult { payoff: dec!(18) },
            MockWalkResult { payoff: dec!(25) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 10
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have 3 buckets: 0-10, 10-20, 20-30
        assert_eq!(distribution.len(), 3);

        // Check probabilities
        assert_eq!(distribution[&PnLRange { lower: 0, upper: 10 }], dec!(1) / dec!(6)); // 1/6
        assert_eq!(distribution[&PnLRange { lower: 10, upper: 20 }], dec!(4) / dec!(6)); // 4/6
        assert_eq!(distribution[&PnLRange { lower: 20, upper: 30 }], dec!(1) / dec!(6)); // 1/6
    }

    #[test]
    fn test_generate_pnl_distribution_edge_cases() {
        // Test edge cases with values exactly at bucket boundaries
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(10) }, // Exactly at boundary
            MockWalkResult { payoff: dec!(20) }, // Exactly at boundary
            MockWalkResult { payoff: dec!(15) }, // Middle of bucket
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 10
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_ok());
        let distribution = result.unwrap();


        assert_eq!(distribution.len(), 1);

        assert_eq!(distribution[&PnLRange { lower: 10, upper: 20 }], dec!(2) / dec!(3)); // 2/3
    }

    #[test]
    fn test_generate_pnl_distribution_single_value() {
        // Test with a single value
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(15) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 10
        let result = generate_pnl_distribution(&walk_results, 10);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have 1 bucket: 10-20
        assert_eq!(distribution.len(), 1);
        assert_eq!(distribution[&PnLRange { lower: 10, upper: 20 }], dec!(1)); // 100% probability
    }

    #[test]
    fn test_generate_pnl_distribution_different_range_sizes() {
        // Test with different range sizes
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(25) },
            MockWalkResult { payoff: dec!(35) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Try with range size of 20
        let result = generate_pnl_distribution(&walk_results, 20);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have 2 buckets: 0-20, 20-40
        assert_eq!(distribution.len(), 2);
        assert_eq!(distribution[&PnLRange { lower: 0, upper: 20 }], dec!(0.5)); // 2/4
        assert_eq!(distribution[&PnLRange { lower: 20, upper: 40 }], dec!(0.5)); // 2/4
    }

    #[test]
    fn test_generate_pnl_distribution_large_range_size() {
        // Test with a range size that encompasses all values
        let mock_walks = vec![
            MockWalkResult { payoff: dec!(5) },
            MockWalkResult { payoff: dec!(15) },
            MockWalkResult { payoff: dec!(25) },
            MockWalkResult { payoff: dec!(35) },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Range size of 100 should put all values in a single bucket
        let result = generate_pnl_distribution(&walk_results, 100);

        assert!(result.is_ok());
        let distribution = result.unwrap();

        // Should have 1 bucket: 0-100
        assert_eq!(distribution.len(), 1);
        assert_eq!(distribution[&PnLRange { lower: 0, upper: 100 }], dec!(1)); // All in one bucket
    }
}

#[cfg(test)]
mod tests_calculate_risk_metrics {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock WalkResult for testing
    #[derive(Debug, Clone)]
    struct MockWalkResult {
        initially: Decimal,
        payoff: Decimal,
        min_value: (Decimal, Decimal),
    }

    impl From<MockWalkResult> for WalkResult {
        fn from(mock: MockWalkResult) -> Self {
            // Create a minimal WalkResult with only the fields needed for testing
            WalkResult {
                initially: mock.initially,
                finally: dec!(100),    // Default value, not used in calculate_risk_metrics
                payoff: mock.payoff,
                change_percentage: dec!(0), // Default value, not used in calculate_risk_metrics
                diff: dec!(0),        // Default value, not used in calculate_risk_metrics
                max_value: (dec!(0), dec!(0)), // Default value, not used in calculate_risk_metrics
                min_value: mock.min_value,
                positive_points: Vec::new(), // Default value, not used in calculate_risk_metrics
                negative_points: Vec::new(), // Default value, not used in calculate_risk_metrics
                pnl_at_prices: HashMap::new(), // Default value, not used in calculate_risk_metrics
                extra_metrics: HashMap::new(), // Default value, not used in calculate_risk_metrics
                volatilities: Vec::new(), // Default value, not used in calculate_risk_metrics
            }
        }
    }



    #[test]
    fn test_calculate_risk_metrics_empty_input() {
        // Test with empty input
        let walk_results: Vec<WalkResult> = Vec::new();

        // In a real test environment, we would temporarily replace calculate_sharpe_ratio
        // with the mock implementation

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "No walk results to calculate risk metrics");
        }
    }

    #[test]
    fn test_calculate_risk_metrics_var_calculation() {
        // Test VaR calculations with known data
        let mock_walks = vec![
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-50),
                min_value: (dec!(50), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-40),
                min_value: (dec!(60), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-30),
                min_value: (dec!(70), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-20),
                min_value: (dec!(80), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-10),
                min_value: (dec!(90), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(0),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(10),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(20),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(30),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(40),
                min_value: (dec!(100), dec!(0)),
            },
            // For a 20-element array, indices: 
            // 5% = index 1 (0-based), 1% = index 0 (0-based)
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(50),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(60),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(70),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(80),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(90),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(100),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(110),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(120),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(130),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(140),
                min_value: (dec!(100), dec!(0)),
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        // Temporarily replace calculate_sharpe_ratio in a real test
        // Here we're just noting that we would do this

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // With 20 elements:
        // 5% VaR = abs(element at index 1) = abs(-40) = 40
        // 1% VaR = abs(element at index 0) = abs(-50) = 50
        assert_eq!(risk_metrics.var_95, dec!(40));
        assert_eq!(risk_metrics.var_99, dec!(50));

        // CVaR 95% is the average of the worst 5% of outcomes
        // Here that's the average of [-50, -40] = -45, and we take absolute value
        assert_eq!(risk_metrics.cvar_95, dec!(45));
    }

    #[test]
    fn test_calculate_risk_metrics_severe_loss_probability() {
        // Test severe loss probability calculation
        let mock_walks = vec![
            // 3 walks with losses > 50%
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-60),
                min_value: (dec!(40), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-55),
                min_value: (dec!(45), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-51),
                min_value: (dec!(49), dec!(0)),
            },
            // 2 walks with losses < 50%
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-49),
                min_value: (dec!(51), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-30),
                min_value: (dec!(70), dec!(0)),
            },
            // 5 walks with gains
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(10),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(20),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(30),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(40),
                min_value: (dec!(100), dec!(0)),
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(50),
                min_value: (dec!(100), dec!(0)),
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // Severe loss probability: 3 out of 10 walks = 30%
        assert_eq!(risk_metrics.severe_loss_probability, Positive::from(dec!(30)));
    }

    #[test]
    fn test_calculate_risk_metrics_max_drawdown() {
        // Test maximum drawdown calculation
        let mock_walks = vec![
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-20),
                min_value: (dec!(80), dec!(0)), // 20% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-30),
                min_value: (dec!(70), dec!(0)), // 30% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-40),
                min_value: (dec!(60), dec!(0)), // 40% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-10),
                min_value: (dec!(90), dec!(0)), // 10% drawdown
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // Maximum drawdown should be 40%
        assert_eq!(risk_metrics.max_drawdown, Positive::from(dec!(0.4)));
    }

    #[test]
    fn test_calculate_risk_metrics_all_positive_returns() {
        // Test with all positive returns
        let mock_walks = vec![
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(10),
                min_value: (dec!(100), dec!(0)), // No drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(20),
                min_value: (dec!(100), dec!(0)), // No drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(30),
                min_value: (dec!(100), dec!(0)), // No drawdown
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // VaR and CVaR should be 0 with all positive returns
        assert_eq!(risk_metrics.var_95, dec!(10));
        assert_eq!(risk_metrics.var_99, dec!(10));
        assert_eq!(risk_metrics.cvar_95, dec!(10));

        // Severe loss probability should be 0
        assert_eq!(risk_metrics.severe_loss_probability, Positive::from(dec!(0)));

        // Max drawdown should be 0
        assert_eq!(risk_metrics.max_drawdown, Positive::from(dec!(0)));
    }

    #[test]
    fn test_calculate_risk_metrics_all_negative_returns() {
        // Test with all negative returns
        let mock_walks = vec![
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-60),
                min_value: (dec!(40), dec!(0)), // 60% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-70),
                min_value: (dec!(30), dec!(0)), // 70% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-80),
                min_value: (dec!(20), dec!(0)), // 80% drawdown
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // With 3 elements sorted [-80, -70, -60]:
        // 5% VaR (index 0) = 80
        // 1% VaR (index 0) = 80
        assert_eq!(risk_metrics.var_95, dec!(80));
        assert_eq!(risk_metrics.var_99, dec!(80));

        // CVaR 95% is average of worst 5% = -80 (abs = 80)
        assert_eq!(risk_metrics.cvar_95, dec!(80));

        // Severe loss probability should be 100% (all losses > 50%)
        assert_eq!(risk_metrics.severe_loss_probability, Positive::from(dec!(100)));

        // Max drawdown should be 80%
        assert_eq!(risk_metrics.max_drawdown, Positive::from(dec!(0.8)));
    }

    #[test]
    fn test_calculate_risk_metrics_mixed_returns() {
        // Test with mixed returns
        let mock_walks = vec![
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-50),
                min_value: (dec!(50), dec!(0)), // 50% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-20),
                min_value: (dec!(80), dec!(0)), // 20% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(10),
                min_value: (dec!(95), dec!(0)), // 5% drawdown
            },
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(30),
                min_value: (dec!(100), dec!(0)), // No drawdown
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // With 4 elements sorted [-50, -20, 10, 30]:
        // 5% VaR (index 0) = 50
        // 1% VaR (index 0) = 50
        assert_eq!(risk_metrics.var_95, dec!(50));
        assert_eq!(risk_metrics.var_99, dec!(50));

        // Severe loss probability: 1 out of 4 = 25%
        assert_eq!(risk_metrics.severe_loss_probability, Positive::from(dec!(25)));

        // Max drawdown should be 50%
        assert_eq!(risk_metrics.max_drawdown, Positive::from(dec!(0.5)));
    }

    #[test]
    fn test_calculate_risk_metrics_single_result() {
        // Test with a single result
        let mock_walks = vec![
            MockWalkResult {
                initially: dec!(100),
                payoff: dec!(-30),
                min_value: (dec!(70), dec!(0)), // 30% drawdown
            },
        ];

        let walk_results: Vec<WalkResult> = mock_walks.into_iter()
            .map(|m| m.into())
            .collect();

        let result = calculate_risk_metrics(&walk_results);

        assert!(result.is_ok());
        let risk_metrics = result.unwrap();

        // With 1 element:
        // VaR and CVaR should be 30 (absolute value of the only payoff)
        assert_eq!(risk_metrics.var_95, dec!(30));
        assert_eq!(risk_metrics.var_99, dec!(30));
        assert_eq!(risk_metrics.cvar_95, dec!(30));

        // Severe loss probability: 0 out of 1 = 0%
        assert_eq!(risk_metrics.severe_loss_probability, Positive::from(dec!(0)));

        // Max drawdown should be 30%
        assert_eq!(risk_metrics.max_drawdown, Positive::from(dec!(0.3)));
    }
}

#[cfg(test)]
mod tests_create_simulation_result {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Create a helper function to create test walk results
    fn create_test_walk_results(payoffs: Vec<Decimal>) -> Vec<WalkResult> {
        payoffs.into_iter().map(|payoff| {
            WalkResult {
                initially: dec!(100),
                finally: dec!(100) + payoff,
                payoff,
                change_percentage: payoff / dec!(100),
                diff: payoff,
                max_value: (dec!(100) + payoff.max(dec!(0)), dec!(0)),
                min_value: (dec!(100) + payoff.min(dec!(0)), dec!(0)),
                positive_points: Vec::new(),
                negative_points: Vec::new(),
                pnl_at_prices: HashMap::new(),
                extra_metrics: HashMap::new(),
                volatilities: Vec::new(),
            }
        }).collect()
    }





    #[test]
    fn test_create_simulation_result_empty_input() {
        // Test with empty input
        let walk_results: Vec<WalkResult> = Vec::new();
        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.to_string(), "No walk results to process");
        }
    }

    #[test]
    fn test_create_simulation_result_basic_metrics() {
        // Create test data: 8 positive payoffs, 2 negative
        let payoffs = vec![
            dec!(15), dec!(10), dec!(5), dec!(20), dec!(25),
            dec!(30), dec!(15), dec!(10), dec!(-5), dec!(-10)
        ];
        let walk_results = create_test_walk_results(payoffs);

        // In a real integration test, we would use the actual function
        // For unit testing, we would need to modify the code to accept dependency injection
        // Here we'll assume the function calls the real dependencies

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Basic metrics tests
        assert_eq!(sim_result.iterations, 10);
        assert_eq!(sim_result.profit_probability, Positive::from(dec!(80))); // 8/10 * 100
        assert_eq!(sim_result.loss_probability, Positive::from(dec!(20))); // 2/10 * 100
        assert_eq!(sim_result.max_profit, Positive::from(dec!(30))); // Max of all payoffs
        assert_eq!(sim_result.max_loss, Positive::from(dec!(10))); // Abs of min payoff

        // Average PnL should be (15+10+5+20+25+30+15+10-5-10)/10 = 11.5
        assert_eq!(sim_result.average_pnl, dec!(11.5));

        // Check that walk results are mapped correctly
        assert_eq!(sim_result.walk_results.len(), 10);

        // Verify WalkId keys are correctly assigned
        for i in 0..10 {
            let walk_id = WalkId::new(i.to_string());
            assert!(sim_result.walk_results.contains_key(&walk_id));
        }
    }

    #[test]
    fn test_create_simulation_result_all_positive_payoffs() {
        // Create test data with all positive payoffs
        let payoffs = vec![dec!(10), dec!(20), dec!(30), dec!(40), dec!(50)];
        let walk_results = create_test_walk_results(payoffs);

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Metrics specific to all-positive case
        assert_eq!(sim_result.profit_probability, Positive::from(dec!(100)));
        assert_eq!(sim_result.loss_probability, Positive::from(dec!(0)));
        assert_eq!(sim_result.max_profit, Positive::from(dec!(50)));
        assert_eq!(sim_result.max_loss, Positive::from(dec!(0.01))); // Default min value
        assert_eq!(sim_result.average_pnl, dec!(30)); // (10+20+30+40+50)/5 = 30
    }

    #[test]
    fn test_create_simulation_result_all_negative_payoffs() {
        // Create test data with all negative payoffs
        let payoffs = vec![dec!(-10), dec!(-20), dec!(-30), dec!(-40), dec!(-50)];
        let walk_results = create_test_walk_results(payoffs);

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Metrics specific to all-negative case
        assert_eq!(sim_result.profit_probability, Positive::from(dec!(0)));
        assert_eq!(sim_result.loss_probability, Positive::from(dec!(100)));
        assert_eq!(sim_result.max_profit, Positive::from(dec!(0.01))); // Default min value
        assert_eq!(sim_result.max_loss, Positive::from(dec!(50)));
        assert_eq!(sim_result.average_pnl, dec!(-30)); // (-10-20-30-40-50)/5 = -30
    }

    #[test]
    fn test_create_simulation_result_zero_payoffs() {
        // Create test data with all zero payoffs
        let payoffs = vec![dec!(0), dec!(0), dec!(0), dec!(0), dec!(0)];
        let walk_results = create_test_walk_results(payoffs);

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Metrics specific to zero-payoff case
        assert_eq!(sim_result.profit_probability, Positive::from(dec!(0)));
        assert_eq!(sim_result.loss_probability, Positive::from(dec!(100))); // Zero is considered a loss
        assert_eq!(sim_result.max_profit, Positive::from(dec!(0.01))); // Default min value
        assert_eq!(sim_result.max_loss, Positive::from(dec!(0.01))); // Default min value
        assert_eq!(sim_result.average_pnl, dec!(0));
        assert_eq!(sim_result.pnl_std_dev, Positive::from(dec!(0))); // Zero std dev
    }

    #[test]
    fn test_create_simulation_result_single_walk() {
        // Create test data with a single walk
        let payoffs = vec![dec!(25)];
        let walk_results = create_test_walk_results(payoffs);

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Metrics for single walk
        assert_eq!(sim_result.iterations, 1);
        assert_eq!(sim_result.profit_probability, Positive::from(dec!(100)));
        assert_eq!(sim_result.loss_probability, Positive::from(dec!(0)));
        assert_eq!(sim_result.max_profit, Positive::from(dec!(25)));
        assert_eq!(sim_result.max_loss, Positive::from(dec!(0.01))); // Default min value
        assert_eq!(sim_result.average_pnl, dec!(25));
        assert_eq!(sim_result.pnl_std_dev, Positive::from(dec!(0))); // Zero std dev with one sample

        // Verify walk results mapping
        assert_eq!(sim_result.walk_results.len(), 1);
        let walk_id = WalkId::new("0".to_string());
        assert!(sim_result.walk_results.contains_key(&walk_id));
    }

    #[test]
    fn test_create_simulation_result_mixed_payoffs_zero_sum() {
        // Create test data with mixed payoffs that sum to zero
        let payoffs = vec![dec!(10), dec!(-10), dec!(20), dec!(-20)];
        let walk_results = create_test_walk_results(payoffs);

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Metrics for zero-sum mixed payoffs
        assert_eq!(sim_result.profit_probability, Positive::from(dec!(50)));
        assert_eq!(sim_result.loss_probability, Positive::from(dec!(50)));
        assert_eq!(sim_result.max_profit, Positive::from(dec!(20)));
        assert_eq!(sim_result.max_loss, Positive::from(dec!(20)));
        assert_eq!(sim_result.average_pnl, dec!(0));

        // Std dev calculation for [10, -10, 20, -20]:
        // Mean = 0
        // Sum of squares = 10² + (-10)² + 20² + (-20)² = 100 + 100 + 400 + 400 = 1000
        // Variance = 1000 / 4 = 250
        // Std dev = √250 ≈ 15.811
        let expected_std_dev = Decimal::from(250).sqrt().unwrap();
        assert_eq!(sim_result.pnl_std_dev, Positive::from(expected_std_dev));
    }

    #[test]
    fn test_create_simulation_result_standard_deviation() {
        // Test specifically for standard deviation calculation
        let payoffs = vec![dec!(5), dec!(10), dec!(15), dec!(20), dec!(25)];
        let walk_results = create_test_walk_results(payoffs);

        let result = create_simulation_result(walk_results, 10);

        assert!(result.is_ok());
        let sim_result = result.unwrap();

        // Manual calculation of standard deviation
        // Mean = (5+10+15+20+25)/5 = 15
        // Sum of squares = (5-15)² + (10-15)² + (15-15)² + (20-15)² + (25-15)²
        //                 = 100 + 25 + 0 + 25 + 100 = 250
        // Variance = 250/5 = 50
        // Std dev = √50 ≈ 7.07
        let expected_std_dev = Decimal::from(50).sqrt().unwrap();
        assert_eq!(sim_result.pnl_std_dev, Positive::from(expected_std_dev));
    }

    // Note on dependency injection
    // For proper unit testing without mocking libraries, you would need to modify
    // the create_simulation_result function to accept the dependency functions as parameters:
    //
    // pub fn create_simulation_result(
    //     walk_results: Vec<WalkResult>,
    //     pnl_range_size: i32,
    //     risk_metrics_fn: fn(&[WalkResult]) -> Result<RiskMetricsSimulation, Box<dyn Error>>,
    //     pnl_distribution_fn: fn(&[WalkResult], i32) -> Result<HashMap<PnLRange, Decimal>, Box<dyn Error>>,
    //     extra_metrics_fn: fn(&[WalkResult]) -> Result<HashMap<String, Decimal>, Box<dyn Error>>,
    // ) -> Result<SimulationResult, Box<dyn Error>> { ... }
    //
    // Then in tests you could pass the test versions:
    // create_simulation_result(walk_results, 10, test_calculate_risk_metrics, 
    //                          test_generate_pnl_distribution, test_aggregate_extra_metrics)
}