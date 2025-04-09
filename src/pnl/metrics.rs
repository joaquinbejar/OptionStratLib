use crate::Positive;
use crate::pnl::PnL;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;

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
    pub initial_volumes: Vec<Positive>,
    pub final_volumes: Vec<Positive>,
    pub delta_adjustments: Positive,
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
            initial_volumes: Vec::new(),
            final_volumes: Vec::new(),
            delta_adjustments: Positive::ZERO,
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

impl fmt::Display for PnLMetricsStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format vectors with rounded values
        fn format_vec(values: &[Positive]) -> String {
            let formatted: Vec<String> = values
                .iter()
                .map(|v| v.to_dec().round_dp(3).to_string())
                .collect();
            format!("[{}]", formatted.join(", "))
        }

        write!(
            f,
            "PnLMetricsStep: {{\
             {}, \
             win: {}, \
             step_number: {}, \
             step_duration: {}, \
             max_unrealized_pnl: {}, \
             min_unrealized_pnl: {}, \
             winning_steps: {}, \
             losing_steps: {}, \
             initial_price: {}, \
             final_price: {}, \
             strikes: {}, \
             initial_volumes: {}, \
             final_volumes: {}, \
             delta_adjustments: {}\
             }}",
            self.pnl,
            self.win,
            self.step_number,
            self.step_duration.round_to(3),
            self.max_unrealized_pnl.round_to(3),
            self.min_unrealized_pnl.round_to(3),
            self.winning_steps,
            self.losing_steps,
            self.initial_price.round_to(3),
            self.final_price.round_to(3),
            format_vec(&self.strikes),
            format_vec(&self.initial_volumes),
            format_vec(&self.final_volumes),
            self.delta_adjustments
        )
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnLMetricsDocument {
    pub days: Positive,
    pub symbol: String,
    pub fee: Positive,
    pub delta: Decimal,
    pub delta_adjustment_at: Decimal,
    pub metrics: Vec<PnLMetricsStep>,
}

pub fn create_pnl_metrics_document(
    metrics: Vec<PnLMetricsStep>,
    days: Positive,
    symbol: String,
    fee: Positive,
    delta: Decimal,
    delta_adjustment_at: Decimal
) -> PnLMetricsDocument {
    PnLMetricsDocument {
        days,
        symbol,
        fee,
        delta,
        delta_adjustment_at,
        metrics,
    }
}


// Global file locks map
lazy_static! {
    static ref FILE_LOCKS: Mutex<HashMap<String, Arc<Mutex<()>>>> = Mutex::new(HashMap::new());
}

// Helper function to get or create a lock for a specific file
fn get_file_lock(file_path: &str) -> Arc<Mutex<()>> {
    let mut locks = FILE_LOCKS.lock().unwrap();
    locks
        .entry(file_path.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

pub fn save_pnl_metrics_with_document(
    document: &PnLMetricsDocument,
    file_path: &str
) -> io::Result<()> {
    // Get a lock for this specific file
    let file_lock = get_file_lock(file_path);
    let _guard = file_lock.lock().unwrap();

    // Check if file exists
    let file_exists = Path::new(file_path).exists();

    if file_exists {
        // Read existing content
        let mut file = OpenOptions::new()
            .read(true)
            .open(file_path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Parse existing content
        let mut documents: Vec<PnLMetricsDocument> = if content.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        };

        // Add new document
        documents.push(document.clone());

        // Write back all documents
        let json = serde_json::to_string(&documents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(file_path)?;

        file.write_all(json.as_bytes())?;
    } else {
        // Create new file with single document in an array
        let documents = vec![document.clone()];
        let json = serde_json::to_string(&documents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut file = File::create(file_path)?;
        file.write_all(json.as_bytes())?;
    }

    // Lock is automatically released when _guard goes out of scope
    Ok(())
}

// pub fn save_pnl_metrics_with_document(
//     document: &PnLMetricsDocument,
//     file_path: &str
// ) -> io::Result<()> {
//     // Check if file exists
//     let file_exists = Path::new(file_path).exists();
// 
//     if file_exists {
//         // Read existing content
//         let mut file = OpenOptions::new()
//             .read(true)
//             .open(file_path)?;
// 
//         let mut content = String::new();
//         file.read_to_string(&mut content)?;
// 
//         // Parse existing content
//         let mut documents: Vec<PnLMetricsDocument> = if content.trim().is_empty() {
//             Vec::new()
//         } else {
//             serde_json::from_str(&content)
//                 .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
//         };
// 
//         // Add new document - clone it to convert from &PnLMetricsDocument to PnLMetricsDocument
//         documents.push(document.clone());
// 
//         // Write back all documents
//         let json = serde_json::to_string(&documents)
//             .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
// 
//         let mut file = OpenOptions::new()
//             .write(true)
//             .truncate(true)
//             .open(file_path)?;
// 
//         file.write_all(json.as_bytes())?;
//     } else {
//         // Create new file with single document in an array
//         let documents = vec![document.clone()];
//         let json = serde_json::to_string(&documents)
//             .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
// 
//         let mut file = File::create(file_path)?;
//         file.write_all(json.as_bytes())?;
//     }
// 
//     Ok(())
// }