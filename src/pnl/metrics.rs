use std::fs::File;
use std::io;
use std::io::Write;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::pnl::PnL;
use crate::Positive;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PnLMetricsStep {
    pub pnl: PnL,
    pub win: bool,
    pub step_number: u32,
    pub step_duration: Positive,
    pub max_unrealized_pnl: Positive,
    pub min_unrealized_pnl: Positive,
    pub winning_steps: u32,
    pub losing_steps: u32,
    pub initial_price: Positive,
    pub final_price: Positive,
    pub strikes: Vec<Positive>,
}

impl Default for PnLMetricsStep {
    fn default() -> Self {
        Self {
            pnl: PnL::default(),
            win: false,
            step_number: 0,
            step_duration: Positive::ZERO,
            max_unrealized_pnl: Positive::ZERO,
            min_unrealized_pnl: Positive::ZERO,
            winning_steps: 0,
            losing_steps: 0,
            initial_price: Positive::ZERO,
            final_price: Positive::ZERO,
            strikes: Vec::new(),
        }
    }
}
        
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PnLMetrics {
    pub total_pnl: Decimal,
    pub max_profit: Positive,
    pub max_loss: Positive,
    pub win_rate: Decimal,
    pub loss_rate: Decimal,
    pub total_steps: u32,
    pub winning_steps: u32,
    pub losing_steps: u32,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub max_drawdown: Positive,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub profit_factor: Decimal,
    pub recovery_factor: Decimal,
    pub expected_payoff: Decimal,
    pub simulation_duration: Decimal, 
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

}

impl Default for PnLMetrics {
    fn default() -> Self {
        Self {
            total_pnl: Decimal::ZERO,
            max_profit: Positive::ZERO,
            max_loss: Positive::ZERO,
            win_rate: Decimal::ZERO,
            loss_rate: Decimal::ZERO,
            total_steps: 0,
            winning_steps: 0,
            losing_steps: 0,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            max_drawdown: Positive::ZERO,
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            recovery_factor: Decimal::ZERO,
            expected_payoff: Decimal::ZERO,
            simulation_duration: Decimal::ZERO, 
            start_time: Utc::now(),
            end_time: Utc::now(),
        }
    }
}


/// Serializes a vector of PnLMetricsStep to compact JSON and saves it to a file
///
/// Similar to save_pnl_metrics_to_json but creates a compact representation
/// without extra whitespace, resulting in smaller file size
///
/// # Arguments
///
/// * `metrics` - Vector of PnLMetricsStep to serialize
/// * `file_path` - Path where the JSON file will be saved
///
/// # Returns
///
/// * `io::Result<()>` - Success or error result
pub fn save_pnl_metrics(metrics: &[PnLMetricsStep], file_path: &str) -> io::Result<()> {
    // Serialize to compact JSON without pretty formatting
    let json = serde_json::to_string(metrics)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Create or open the file for writing
    let mut file = File::create(file_path)?;

    // Write the JSON string to the file
    file.write_all(json.as_bytes())?;

    Ok(())
}

/// Loads a vector of PnLMetricsStep from a JSON file
///
/// # Arguments
///
/// * `file_path` - Path to the JSON file
///
/// # Returns
///
/// * `io::Result<Vec<PnLMetricsStep>>` - Vector of deserialized metrics or error
///
pub fn load_pnl_metrics(file_path: &str) -> io::Result<Vec<PnLMetricsStep>> {
    // Read the file content
    let file_content = std::fs::read_to_string(file_path)?;

    // Deserialize JSON to Vector of PnLMetricsStep
    let metrics: Vec<PnLMetricsStep> = serde_json::from_str(&file_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(metrics)
}