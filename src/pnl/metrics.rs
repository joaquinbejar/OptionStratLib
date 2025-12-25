use crate::pnl::PnL;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// # PnLMetricsStep
///
/// Represents a single step in a profit and loss (PnL) metrics analysis simulation.
///
/// This structure captures detailed financial performance data for a specific step in
/// a multi-step simulation of an options trading strategy. It records not only the
/// basic profit/loss information but also tracks performance metrics, price changes,
/// and position adjustments that occurred during this particular step.
///
///
/// This structure is serializable and deserializable through Serde, making it suitable
/// for persistence and data exchange operations in trading simulations and backtesting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PnLMetricsStep {
    /// `pnl`: The profit and loss calculation for this step, including realized and unrealized components.
    pub pnl: PnL,
    /// `win`: A boolean indicating whether this step resulted in a winning outcome.
    pub win: bool,
    /// `step_number`: The sequential number of this step within the simulation.
    pub step_number: u32,
    /// `step_duration`: The duration of this step.
    pub step_duration: Positive,
    /// `max_unrealized_pnl`: The maximum unrealized profit and loss observed during this step.
    pub max_unrealized_pnl: Positive,
    /// `min_unrealized_pnl`: The minimum unrealized profit and loss observed during this step.
    pub min_unrealized_pnl: Positive,
    /// `winning_steps`: The cumulative number of winning steps up to this point in the simulation.
    pub winning_steps: u32,
    /// `losing_steps`: The cumulative number of losing steps up to this point in the simulation.
    pub losing_steps: u32,
    /// `initial_price`: The initial price at the beginning of this step.
    pub initial_price: Positive,
    /// `final_price`: The final price at the end of this step.
    pub final_price: Positive,
    /// `strikes`: A vector of strike prices relevant to the options strategy in this step.
    pub strikes: Vec<Positive>,
    /// `initial_volumes`: A vector of initial volumes for each option in the strategy at the start of this step.
    pub initial_volumes: Vec<Positive>,
    /// `final_volumes`: A vector of final volumes for each option in the strategy at the end of this step.
    pub final_volumes: Vec<Positive>,
    /// `delta_adjustments`: The delta adjustments made during this step.
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

/// `PnLMetrics` struct holds various metrics related to Profit and Loss (PnL) analysis.
///
/// This struct captures a comprehensive set of metrics including total PnL, maximum profit and loss,
/// win/loss rates, average win/loss amounts, maximum drawdown, Sharpe and Sortino ratios,
/// profit and recovery factors, expected payoff, simulation duration, and start/end times.
///
/// It is designed to provide a detailed overview of the performance and risk characteristics
/// of a trading strategy or simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PnLMetrics {
    /// The total profit and loss.
    pub total_pnl: Decimal,
    /// The maximum profit achieved.
    pub max_profit: Positive,
    /// The maximum loss incurred.
    pub max_loss: Positive,
    /// The win rate, i.e., the proportion of winning trades.
    pub win_rate: Decimal,
    /// The loss rate, i.e., the proportion of losing trades.
    pub loss_rate: Decimal,
    /// The total number of steps in the simulation or trading period.
    pub total_steps: u32,
    /// The number of steps that resulted in a win.
    pub winning_steps: u32,
    /// The number of steps that resulted in a loss.
    pub losing_steps: u32,
    /// The average win amount.
    pub avg_win: Decimal,
    /// The average loss amount.
    pub avg_loss: Decimal,
    /// The maximum drawdown experienced.
    pub max_drawdown: Positive,
    /// The Sharpe ratio, a measure of risk-adjusted return.
    pub sharpe_ratio: Decimal,
    /// The Sortino ratio, a variation of the Sharpe ratio that only considers downside risk.
    pub sortino_ratio: Decimal,
    /// The profit factor, calculated as gross profit divided by gross loss.
    pub profit_factor: Decimal,
    /// The recovery factor, a measure of how quickly losses are recovered.
    pub recovery_factor: Decimal,
    /// The expected payoff per trade or step.
    pub expected_payoff: Decimal,
    /// The duration of the simulation or trading period.
    pub simulation_duration: Decimal,
    /// The start time of the simulation or trading period.
    pub start_time: DateTime<Utc>,
    /// The end time of the simulation or trading period.
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

/// Represents a document containing profit and loss (PnL) metrics for a specific asset over a period.
///
/// This struct encapsulates the PnL metrics, symbol, fees, delta and delta adjustments.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnLMetricsDocument {
    /// * `days`: The numbers of days considered in the metrics.
    pub days: Positive,
    /// * `symbol`: The trading symbol of the asset.
    pub symbol: String,
    /// * `fee`: The fee associated with the trading.
    pub fee: Positive,
    /// * `delta`: The delta value of the asset.
    pub delta: Decimal,
    /// * `delta_adjustment_at`: The delta adjustment applied to the asset.
    pub delta_adjustment_at: Decimal,
    /// * `metrics`: A vector of `PnLMetricsStep` structs, each representing a step in the PnL calculation.
    pub metrics: Vec<PnLMetricsStep>,
}

/// Creates a `PnLMetricsDocument` instance.
///
/// This function constructs a `PnLMetricsDocument` from provided profit and loss metrics
/// and associated parameters. It encapsulates the simulation results and configuration
/// details, providing a structured representation of the analysis.
///
/// # Arguments
///
/// * `metrics`: A vector of `PnLMetricsStep` representing the individual steps in the
///   profit and loss simulation.
/// * `days`: A `Positive` value indicating the number of days over which the simulation was run.
/// * `symbol`: A `String` representing the financial symbol (e.g., stock ticker) used in the simulation.
/// * `fee`: A `Positive` value representing the transaction fee applied during the simulation.
/// * `delta`: A `Decimal` representing the delta value used in the simulation.
/// * `delta_adjustment_at`: A `Decimal` representing the delta adjustment threshold used in the simulation.
///
/// # Returns
///
/// A `PnLMetricsDocument` instance containing the provided metrics and parameters.
pub fn create_pnl_metrics_document(
    metrics: Vec<PnLMetricsStep>,
    days: Positive,
    symbol: String,
    fee: Positive,
    delta: Decimal,
    delta_adjustment_at: Decimal,
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

/// Saves PnL metrics to a JSON file, handling concurrent access and file existence.
///
/// This function either appends the provided `PnLMetricsDocument` to an existing JSON file
/// or creates a new file containing the document within a JSON array. It uses file locking
/// to prevent race conditions when multiple processes or threads attempt to write to the same
/// file simultaneously.
///
/// # Arguments
///
/// * `document` - A reference to the `PnLMetricsDocument` to be saved.  The document is cloned before saving.
/// * `file_path` - A string slice representing the path to the JSON file.
///
/// # Errors
///
/// Returns an `io::Error` if:
///
/// * The file cannot be opened for reading or writing.
/// * The existing file content cannot be read.
/// * The existing file content cannot be parsed as a JSON array of `PnLMetricsDocument`.
/// * The updated JSON array cannot be serialized.
/// * The file cannot be truncated.
/// * The data cannot be written to the file.
///
/// # Thread Safety
///
/// This function is thread-safe.  It uses a file-based lock to prevent concurrent writes to the file.
///
/// # Example
///
/// ```no_run
/// use rust_decimal::Decimal;
/// use tracing::{error, info};
/// use optionstratlib::pnl::{save_pnl_metrics_with_document, PnLMetricsDocument};
/// use optionstratlib::pos_or_panic;
///
/// // Assume 'document' is a valid PnLMetricsDocument instance
/// let document = PnLMetricsDocument {
///     days: pos_or_panic!(10.0),
///     symbol: "AAPL".to_string(),
///     fee: pos_or_panic!(0.01),
///     delta: Decimal::new(5, 1),
///     delta_adjustment_at: Decimal::new(0, 0),
///     metrics: vec![],
/// };
/// let file_path = "pnl_metrics.json";
///
/// match save_pnl_metrics_with_document(&document, file_path) {
///     Ok(_) => info!("PnL metrics saved successfully."),
///     Err(e) => error!("Error saving PnL metrics: {}", e),
/// }
/// ```
pub fn save_pnl_metrics_with_document(
    document: &PnLMetricsDocument,
    file_path: &str,
) -> io::Result<()> {
    // Get a lock for this specific file
    let file_lock = get_file_lock(file_path);
    let _guard = file_lock.lock().unwrap();

    // Check if file exists
    let file_exists = Path::new(file_path).exists();

    if file_exists {
        // Read existing content
        let mut file = OpenOptions::new().read(true).open(file_path)?;

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

#[cfg(test)]
mod tests_pnl_metrics {
    use super::*;
    use crate::pnl::PnL;

    use chrono::Utc;
    use num_traits::FromPrimitive;
    use positive::pos_or_panic;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_pnl_metrics_step_default() {
        let metrics_step = PnLMetricsStep::default();

        assert_eq!(metrics_step.pnl, PnL::default());
        assert!(!metrics_step.win);
        assert_eq!(metrics_step.step_number, 0);
        assert_eq!(metrics_step.step_duration, Positive::ZERO);
        assert_eq!(metrics_step.max_unrealized_pnl, Positive::ZERO);
        assert_eq!(metrics_step.min_unrealized_pnl, Positive::ZERO);
        assert_eq!(metrics_step.winning_steps, 0);
        assert_eq!(metrics_step.losing_steps, 0);
        assert_eq!(metrics_step.initial_price, Positive::ZERO);
        assert_eq!(metrics_step.final_price, Positive::ZERO);
        assert!(metrics_step.strikes.is_empty());
        assert!(metrics_step.initial_volumes.is_empty());
        assert!(metrics_step.final_volumes.is_empty());
        assert_eq!(metrics_step.delta_adjustments, Positive::ZERO);
    }

    #[test]
    fn test_pnl_metrics_default() {
        let metrics = PnLMetrics::default();

        assert_eq!(metrics.total_pnl, Decimal::ZERO);
        assert_eq!(metrics.max_profit, Positive::ZERO);
        assert_eq!(metrics.max_loss, Positive::ZERO);
        assert_eq!(metrics.win_rate, Decimal::ZERO);
        assert_eq!(metrics.loss_rate, Decimal::ZERO);
        assert_eq!(metrics.total_steps, 0);
        assert_eq!(metrics.winning_steps, 0);
        assert_eq!(metrics.losing_steps, 0);
        assert_eq!(metrics.avg_win, Decimal::ZERO);
        assert_eq!(metrics.avg_loss, Decimal::ZERO);
        assert_eq!(metrics.max_drawdown, Positive::ZERO);
        assert_eq!(metrics.sharpe_ratio, Decimal::ZERO);
        assert_eq!(metrics.sortino_ratio, Decimal::ZERO);
        assert_eq!(metrics.profit_factor, Decimal::ZERO);
        assert_eq!(metrics.recovery_factor, Decimal::ZERO);
        assert_eq!(metrics.expected_payoff, Decimal::ZERO);
        assert_eq!(metrics.simulation_duration, Decimal::ZERO);
        // Note: We can't directly compare DateTime instances for exact equality
        // but we can check they're reasonably close to now
        let now = Utc::now();
        assert!(metrics.start_time <= now);
        assert!(metrics.end_time <= now);
        assert!((metrics.end_time - metrics.start_time).num_seconds() <= 1);
    }

    #[test]
    fn test_pnl_metrics_step_display() {
        // Create a sample metrics step
        let metrics_step = PnLMetricsStep {
            win: true,
            step_number: 5,
            step_duration: pos_or_panic!(1.5),
            max_unrealized_pnl: Positive::HUNDRED,
            min_unrealized_pnl: pos_or_panic!(50.0),
            winning_steps: 3,
            losing_steps: 2,
            initial_price: pos_or_panic!(95.0),
            final_price: pos_or_panic!(105.0),
            strikes: vec![pos_or_panic!(90.0), Positive::HUNDRED, pos_or_panic!(110.0)],
            initial_volumes: vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)],
            final_volumes: vec![pos_or_panic!(0.5), pos_or_panic!(1.5), pos_or_panic!(2.5)],
            delta_adjustments: Positive::TWO,
            ..Default::default()
        };

        // Test display output
        let display = format!("{metrics_step}");

        // Check that all fields are present in the output
        assert!(display.contains("win: true"));
        assert!(display.contains("step_number: 5"));
        assert!(display.contains("step_duration: 1.5"));
        assert!(display.contains("max_unrealized_pnl: 100"));
        assert!(display.contains("min_unrealized_pnl: 50"));
        assert!(display.contains("winning_steps: 3"));
        assert!(display.contains("losing_steps: 2"));
        assert!(display.contains("initial_price: 95"));
        assert!(display.contains("final_price: 105"));
        assert!(display.contains("strikes: [90, 100, 110]"));
        assert!(display.contains("initial_volumes: [1, 2, 3]"));
        assert!(display.contains("final_volumes: [0.5, 1.5, 2.5]"));
        assert!(display.contains("delta_adjustments: 2"));
    }

    #[test]
    fn test_save_and_load_pnl_metrics() {
        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test_metrics.json");
        let file_path_str = file_path.to_str().unwrap();

        // Create test metrics data
        let metrics_step1 = PnLMetricsStep {
            step_number: 1,
            win: true,
            initial_price: Positive::HUNDRED,
            final_price: pos_or_panic!(105.0),
            ..Default::default()
        };
        let metrics_step2 = PnLMetricsStep {
            step_number: 2,
            win: false,
            initial_price: pos_or_panic!(105.0),
            final_price: pos_or_panic!(95.0),
            ..Default::default()
        };

        let metrics = vec![metrics_step1.clone(), metrics_step2.clone()];

        // Save the metrics
        let result = save_pnl_metrics(&metrics, file_path_str);
        assert!(result.is_ok(), "Failed to save metrics: {:?}", result.err());

        // Verify the file exists
        assert!(file_path.exists(), "File was not created");

        // Load the metrics
        let loaded_metrics = load_pnl_metrics(file_path_str).expect("Failed to load metrics");

        // Verify loaded data matches the original
        assert_eq!(loaded_metrics.len(), 2);
        assert_eq!(loaded_metrics[0].step_number, 1);
        assert!(loaded_metrics[0].win);
        assert_eq!(loaded_metrics[0].initial_price, Positive::HUNDRED);
        assert_eq!(loaded_metrics[0].final_price, pos_or_panic!(105.0));
        assert_eq!(loaded_metrics[1].step_number, 2);
        assert!(!loaded_metrics[1].win);
        assert_eq!(loaded_metrics[1].initial_price, pos_or_panic!(105.0));
        assert_eq!(loaded_metrics[1].final_price, pos_or_panic!(95.0));

        // Clean up
        temp_dir.close().expect("Failed to remove temp directory");
    }

    #[test]
    fn test_pnl_metrics_document_creation() {
        // Create test metrics
        let metrics_step = PnLMetricsStep {
            step_number: 1,
            win: true,
            initial_price: Positive::HUNDRED,
            final_price: pos_or_panic!(110.0),
            ..Default::default()
        };

        let metrics = vec![metrics_step];

        // Create the document
        let document = create_pnl_metrics_document(
            metrics.clone(),
            pos_or_panic!(30.0),
            "AAPL".to_string(),
            pos_or_panic!(0.65),
            Decimal::from_f64(0.5).unwrap(),
            Decimal::from_f64(0.1).unwrap(),
        );

        // Verify document properties
        assert_eq!(document.days, pos_or_panic!(30.0));
        assert_eq!(document.symbol, "AAPL");
        assert_eq!(document.fee, pos_or_panic!(0.65));
        assert_eq!(document.delta, Decimal::from_f64(0.5).unwrap());
        assert_eq!(
            document.delta_adjustment_at,
            Decimal::from_f64(0.1).unwrap()
        );
        assert_eq!(document.metrics.len(), 1);
        assert_eq!(document.metrics[0].step_number, 1);
        assert!(document.metrics[0].win);
        assert_eq!(document.metrics[0].initial_price, Positive::HUNDRED);
        assert_eq!(document.metrics[0].final_price, pos_or_panic!(110.0));
    }

    #[test]
    fn test_save_pnl_metrics_with_document() {
        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test_document.json");
        let file_path_str = file_path.to_str().unwrap();

        // Create test metrics
        let metrics_step = PnLMetricsStep {
            step_number: 1,
            win: true,
            ..Default::default()
        };

        let metrics = vec![metrics_step];

        // Create the document
        let document = create_pnl_metrics_document(
            metrics,
            pos_or_panic!(30.0),
            "AAPL".to_string(),
            pos_or_panic!(0.65),
            Decimal::from_f64(0.5).unwrap(),
            Decimal::from_f64(0.1).unwrap(),
        );

        // Test saving when file doesn't exist yet
        let result = save_pnl_metrics_with_document(&document, file_path_str);
        assert!(
            result.is_ok(),
            "Failed to save document: {:?}",
            result.err()
        );

        // Verify the file exists
        assert!(file_path.exists(), "File was not created");

        // Read file contents
        let content = fs::read_to_string(&file_path).expect("Failed to read file");
        let documents: Vec<PnLMetricsDocument> =
            serde_json::from_str(&content).expect("Failed to parse JSON");

        // Verify content
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].symbol, "AAPL");
        assert_eq!(documents[0].days, pos_or_panic!(30.0));

        // Create a second document
        let metrics_step2 = PnLMetricsStep {
            step_number: 2,
            win: false,
            ..Default::default()
        };

        let metrics2 = vec![metrics_step2];

        let document2 = create_pnl_metrics_document(
            metrics2,
            pos_or_panic!(60.0),
            "MSFT".to_string(),
            pos_or_panic!(0.75),
            Decimal::from_f64(0.6).unwrap(),
            Decimal::from_f64(0.2).unwrap(),
        );

        // Test saving when file already exists
        let result = save_pnl_metrics_with_document(&document2, file_path_str);
        assert!(
            result.is_ok(),
            "Failed to save second document: {:?}",
            result.err()
        );

        // Read file contents again
        let content = fs::read_to_string(&file_path).expect("Failed to read file after update");
        let documents: Vec<PnLMetricsDocument> =
            serde_json::from_str(&content).expect("Failed to parse updated JSON");

        // Verify content
        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0].symbol, "AAPL");
        assert_eq!(documents[0].days, pos_or_panic!(30.0));
        assert_eq!(documents[1].symbol, "MSFT");
        assert_eq!(documents[1].days, pos_or_panic!(60.0));

        // Clean up
        temp_dir.close().expect("Failed to remove temp directory");
    }

    #[test]
    fn test_load_pnl_metrics_empty_file() {
        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("empty.json");
        let file_path_str = file_path.to_str().unwrap();

        // Create an empty file
        fs::write(&file_path, "").expect("Failed to write empty file");

        // Attempt to load from empty file
        let result = load_pnl_metrics(file_path_str);
        assert!(result.is_err(), "Loading from empty file should fail");

        // Clean up
        temp_dir.close().expect("Failed to remove temp directory");
    }

    #[test]
    fn test_load_pnl_metrics_invalid_json() {
        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("invalid.json");
        let file_path_str = file_path.to_str().unwrap();

        // Create a file with invalid JSON
        fs::write(&file_path, "{invalid_json: this is not valid}")
            .expect("Failed to write invalid file");

        // Attempt to load from invalid file
        let result = load_pnl_metrics(file_path_str);
        assert!(result.is_err(), "Loading from invalid JSON should fail");

        // Clean up
        temp_dir.close().expect("Failed to remove temp directory");
    }

    #[test]
    fn test_save_pnl_metrics_with_document_concurrent() {
        // This is a basic test of concurrency - for more thorough testing, you might
        // need to use actual threads, but that's beyond the scope of a simple unit test

        // Create a temporary directory for test files
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("concurrent.json");
        let file_path_str = file_path.to_str().unwrap();

        // Create multiple documents
        let mut documents = Vec::new();

        for i in 1..=5 {
            let metrics_step = PnLMetricsStep {
                step_number: i,
                ..Default::default()
            };
            let metrics = vec![metrics_step];

            let document = create_pnl_metrics_document(
                metrics,
                pos_or_panic!(30.0),
                format!("SYMBOL{i}"),
                pos_or_panic!(0.65),
                Decimal::from_f64(0.5).unwrap(),
                Decimal::from_f64(0.1).unwrap(),
            );

            documents.push(document);
        }

        // Save all documents sequentially (simulating concurrent access)
        for doc in &documents {
            let result = save_pnl_metrics_with_document(doc, file_path_str);
            assert!(
                result.is_ok(),
                "Failed to save document: {:?}",
                result.err()
            );
        }

        // Read file contents
        let content = fs::read_to_string(&file_path).expect("Failed to read file");
        let loaded_docs: Vec<PnLMetricsDocument> =
            serde_json::from_str(&content).expect("Failed to parse JSON");

        // Verify all documents were saved
        assert_eq!(loaded_docs.len(), 5);

        // Verify the order is correct
        for (i, _) in loaded_docs.iter().enumerate().take(5) {
            assert_eq!(loaded_docs[i].symbol, format!("SYMBOL{}", i + 1));
        }

        // Clean up
        temp_dir.close().expect("Failed to remove temp directory");
    }
}

#[cfg(test)]
mod tests_pnl_metrics_serialization {
    use super::*;
    use chrono::{TimeZone, Utc};
    use num_traits::FromPrimitive;
    use positive::pos_or_panic;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::pnl::PnL;

    #[test]
    fn test_pnl_metrics_step_serialization() {
        // Create a sample PnLMetricsStep with all fields populated
        let mut metrics_step = PnLMetricsStep {
            win: true,
            step_number: 42,
            step_duration: pos_or_panic!(1.5),
            max_unrealized_pnl: pos_or_panic!(100.5),
            min_unrealized_pnl: pos_or_panic!(50.25),
            winning_steps: 30,
            losing_steps: 12,
            initial_price: pos_or_panic!(95.75),
            final_price: pos_or_panic!(105.25),
            strikes: vec![pos_or_panic!(90.0), Positive::HUNDRED, pos_or_panic!(110.0)],
            initial_volumes: vec![pos_or_panic!(1.5), pos_or_panic!(2.5), pos_or_panic!(3.5)],
            final_volumes: vec![pos_or_panic!(0.5), pos_or_panic!(1.5), pos_or_panic!(2.5)],
            delta_adjustments: pos_or_panic!(2.25),
            ..Default::default()
        };

        // Set PnL fields with meaningful values
        let pnl = PnL::new(
            Some(dec!(123.45)),
            Some(dec!(67.89)),
            pos_or_panic!(500.0),
            pos_or_panic!(250.0),
            Utc::now(),
        );
        metrics_step.pnl = pnl;

        // Serialize to JSON
        let serialized = serde_json::to_string(&metrics_step).expect("Failed to serialize");

        // Verify it contains expected fields
        assert!(serialized.contains("\"win\":true"));
        assert!(serialized.contains("\"step_number\":42"));
        assert!(serialized.contains("\"winning_steps\":30"));
        assert!(serialized.contains("\"losing_steps\":12"));

        // Deserialize back to struct
        let deserialized: PnLMetricsStep =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify values are preserved
        assert!(deserialized.win);
        assert_eq!(deserialized.step_number, 42);
        assert_eq!(deserialized.step_duration, pos_or_panic!(1.5));
        assert_eq!(deserialized.max_unrealized_pnl, pos_or_panic!(100.5));
        assert_eq!(deserialized.min_unrealized_pnl, pos_or_panic!(50.25));
        assert_eq!(deserialized.winning_steps, 30);
        assert_eq!(deserialized.losing_steps, 12);
        assert_eq!(deserialized.initial_price, pos_or_panic!(95.75));
        assert_eq!(deserialized.final_price, pos_or_panic!(105.25));
        assert_eq!(deserialized.delta_adjustments, pos_or_panic!(2.25));

        // Check arrays are preserved correctly
        assert_eq!(deserialized.strikes.len(), 3);
        assert_eq!(deserialized.strikes[0], pos_or_panic!(90.0));
        assert_eq!(deserialized.strikes[1], Positive::HUNDRED);
        assert_eq!(deserialized.strikes[2], pos_or_panic!(110.0));

        assert_eq!(deserialized.initial_volumes.len(), 3);
        assert_eq!(deserialized.initial_volumes[0], pos_or_panic!(1.5));
        assert_eq!(deserialized.initial_volumes[1], pos_or_panic!(2.5));
        assert_eq!(deserialized.initial_volumes[2], pos_or_panic!(3.5));

        assert_eq!(deserialized.final_volumes.len(), 3);
        assert_eq!(deserialized.final_volumes[0], pos_or_panic!(0.5));
        assert_eq!(deserialized.final_volumes[1], pos_or_panic!(1.5));
        assert_eq!(deserialized.final_volumes[2], pos_or_panic!(2.5));

        // Check PnL values
        assert_eq!(deserialized.pnl.realized, Some(dec!(123.45)));
        assert_eq!(deserialized.pnl.unrealized, Some(dec!(67.89)));
        assert_eq!(deserialized.pnl.initial_costs, pos_or_panic!(500.0));
        assert_eq!(deserialized.pnl.initial_income, pos_or_panic!(250.0));
    }

    #[test]
    fn test_pnl_metrics_serialization() {
        // Create a sample PnLMetrics with all fields populated
        let start_time = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2023, 1, 31, 23, 59, 59).unwrap();

        let metrics = PnLMetrics {
            total_pnl: dec!(1234.56),
            max_profit: pos_or_panic!(500.75),
            max_loss: pos_or_panic!(200.25),
            win_rate: dec!(0.65),
            loss_rate: dec!(0.35),
            total_steps: 100,
            winning_steps: 65,
            losing_steps: 35,
            avg_win: dec!(20.5),
            avg_loss: dec!(15.75),
            max_drawdown: pos_or_panic!(150.0),
            sharpe_ratio: dec!(1.75),
            sortino_ratio: dec!(2.5),
            profit_factor: dec!(2.25),
            recovery_factor: dec!(3.5),
            expected_payoff: dec!(12.5),
            simulation_duration: dec!(30.0),
            start_time,
            end_time,
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&metrics).expect("Failed to serialize");

        // Verify it contains expected fields
        assert!(serialized.contains("\"total_pnl\":\"1234.56\""));
        assert!(serialized.contains("\"win_rate\":\"0.65\""));
        assert!(serialized.contains("\"total_steps\":100"));
        assert!(serialized.contains("\"winning_steps\":65"));
        assert!(serialized.contains("\"2023-01-01T00:00:00Z\""));
        assert!(serialized.contains("\"2023-01-31T23:59:59Z\""));

        // Deserialize back to struct
        let deserialized: PnLMetrics =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify values are preserved
        assert_eq!(deserialized.total_pnl, dec!(1234.56));
        assert_eq!(deserialized.max_profit, pos_or_panic!(500.75));
        assert_eq!(deserialized.max_loss, pos_or_panic!(200.25));
        assert_eq!(deserialized.win_rate, dec!(0.65));
        assert_eq!(deserialized.loss_rate, dec!(0.35));
        assert_eq!(deserialized.total_steps, 100);
        assert_eq!(deserialized.winning_steps, 65);
        assert_eq!(deserialized.losing_steps, 35);
        assert_eq!(deserialized.avg_win, dec!(20.5));
        assert_eq!(deserialized.avg_loss, dec!(15.75));
        assert_eq!(deserialized.max_drawdown, pos_or_panic!(150.0));
        assert_eq!(deserialized.sharpe_ratio, dec!(1.75));
        assert_eq!(deserialized.sortino_ratio, dec!(2.5));
        assert_eq!(deserialized.profit_factor, dec!(2.25));
        assert_eq!(deserialized.recovery_factor, dec!(3.5));
        assert_eq!(deserialized.expected_payoff, dec!(12.5));
        assert_eq!(deserialized.simulation_duration, dec!(30.0));
        assert_eq!(deserialized.start_time, start_time);
        assert_eq!(deserialized.end_time, end_time);
    }

    #[test]
    fn test_pnl_metrics_document_serialization() {
        // Create sample metrics for the document
        let metrics_step1 = PnLMetricsStep {
            step_number: 1,
            win: true,
            initial_price: Positive::HUNDRED,
            final_price: pos_or_panic!(105.0),
            ..Default::default()
        };

        let metrics_step2 = PnLMetricsStep {
            step_number: 2,
            win: false,
            initial_price: pos_or_panic!(105.0),
            final_price: pos_or_panic!(95.0),
            ..Default::default()
        };

        let metrics = vec![metrics_step1, metrics_step2];

        // Create the document
        let document = PnLMetricsDocument {
            days: pos_or_panic!(30.0),
            symbol: "AAPL".to_string(),
            fee: pos_or_panic!(0.65),
            delta: dec!(0.5),
            delta_adjustment_at: dec!(0.1),
            metrics,
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&document).expect("Failed to serialize");

        // Verify it contains expected fields
        assert!(serialized.contains("\"days\":30"));
        assert!(serialized.contains("\"symbol\":\"AAPL\""));
        assert!(serialized.contains("\"fee\":0.65"));
        assert!(serialized.contains("\"delta\":\"0.5\""));
        assert!(serialized.contains("\"delta_adjustment_at\":\"0.1\""));
        assert!(serialized.contains("\"step_number\":1"));
        assert!(serialized.contains("\"step_number\":2"));
        assert!(serialized.contains("\"win\":true"));
        assert!(serialized.contains("\"win\":false"));

        // Deserialize back to struct
        let deserialized: PnLMetricsDocument =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify values are preserved
        assert_eq!(deserialized.days, pos_or_panic!(30.0));
        assert_eq!(deserialized.symbol, "AAPL");
        assert_eq!(deserialized.fee, pos_or_panic!(0.65));
        assert_eq!(deserialized.delta, dec!(0.5));
        assert_eq!(deserialized.delta_adjustment_at, dec!(0.1));

        // Verify metrics array
        assert_eq!(deserialized.metrics.len(), 2);
        assert_eq!(deserialized.metrics[0].step_number, 1);
        assert!(deserialized.metrics[0].win);
        assert_eq!(deserialized.metrics[0].initial_price, Positive::HUNDRED);
        assert_eq!(deserialized.metrics[0].final_price, pos_or_panic!(105.0));
        assert_eq!(deserialized.metrics[1].step_number, 2);
        assert!(!deserialized.metrics[1].win);
        assert_eq!(deserialized.metrics[1].initial_price, pos_or_panic!(105.0));
        assert_eq!(deserialized.metrics[1].final_price, pos_or_panic!(95.0));
    }

    #[test]
    fn test_pnl_serialization() {
        // Create a sample PnL object with all fields populated
        let date_time = Utc.with_ymd_and_hms(2023, 3, 15, 14, 30, 0).unwrap();
        let pnl = PnL::new(
            Some(dec!(123.45)),
            Some(dec!(-67.89)),
            pos_or_panic!(500.0),
            pos_or_panic!(250.0),
            date_time,
        );

        // Serialize to JSON
        let serialized = serde_json::to_string(&pnl).expect("Failed to serialize");

        // Verify it contains expected fields
        assert!(serialized.contains("\"realized\":\"123.45\""));
        assert!(serialized.contains("\"unrealized\":\"-67.89\""));
        assert!(serialized.contains("\"initial_costs\":500"));
        assert!(serialized.contains("\"initial_income\":250"));
        assert!(serialized.contains("\"2023-03-15T14:30:00Z\""));

        // Deserialize back to struct
        let deserialized: PnL = serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify values are preserved
        assert_eq!(deserialized.realized, Some(dec!(123.45)));
        assert_eq!(deserialized.unrealized, Some(dec!(-67.89)));
        assert_eq!(deserialized.initial_costs, pos_or_panic!(500.0));
        assert_eq!(deserialized.initial_income, pos_or_panic!(250.0));
        assert_eq!(deserialized.date_time, date_time);
    }

    #[test]
    fn test_pnl_metrics_step_null_fields() {
        // Create a sample with null fields
        let metrics_step = PnLMetricsStep::default();

        // Serialize to JSON
        let serialized = serde_json::to_string(&metrics_step).expect("Failed to serialize");

        // Deserialize back to struct
        let deserialized: PnLMetricsStep =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify values are preserved
        assert!(!deserialized.win);
        assert_eq!(deserialized.step_number, 0);
        assert_eq!(deserialized.winning_steps, 0);
        assert_eq!(deserialized.strikes.len(), 0);
        assert_eq!(deserialized.initial_volumes.len(), 0);
        assert_eq!(deserialized.final_volumes.len(), 0);
    }

    #[test]
    fn test_pnl_with_null_fields() {
        // Create a PnL with null fields
        let date_time = Utc.with_ymd_and_hms(2023, 3, 15, 14, 30, 0).unwrap();
        let pnl = PnL::new(
            None,
            None,
            pos_or_panic!(500.0),
            pos_or_panic!(250.0),
            date_time,
        );

        // Serialize to JSON
        let serialized = serde_json::to_string(&pnl).expect("Failed to serialize");

        // Verify nulls are correctly represented
        assert!(serialized.contains("\"realized\":null"));
        assert!(serialized.contains("\"unrealized\":null"));

        // Deserialize back to struct
        let deserialized: PnL = serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify null values are preserved
        assert_eq!(deserialized.realized, None);
        assert_eq!(deserialized.unrealized, None);
        assert_eq!(deserialized.initial_costs, pos_or_panic!(500.0));
        assert_eq!(deserialized.initial_income, pos_or_panic!(250.0));
        assert_eq!(deserialized.date_time, date_time);
    }

    #[test]
    fn test_array_of_pnl_metrics_documents_serialization() {
        // Create multiple documents
        let mut documents = Vec::new();

        for i in 1..=3 {
            let metrics_step = PnLMetricsStep {
                step_number: i as u32,
                win: i % 2 == 0,
                ..Default::default()
            };

            let document = PnLMetricsDocument {
                days: pos_or_panic!(30.0 * i as f64),
                symbol: format!("SYMBOL{i}"),
                fee: pos_or_panic!(0.5 + (i as f64 * 0.1)),
                delta: Decimal::from_f64(0.3 + (i as f64 * 0.1)).unwrap(),
                delta_adjustment_at: Decimal::from_f64(0.05 * i as f64).unwrap(),
                metrics: vec![metrics_step],
            };

            documents.push(document);
        }

        // Serialize array to JSON
        let serialized = serde_json::to_string(&documents).expect("Failed to serialize");

        // Verify it contains data from all documents
        assert!(serialized.contains("\"days\":30"));
        assert!(serialized.contains("\"days\":60"));
        assert!(serialized.contains("\"days\":90"));
        assert!(serialized.contains("\"SYMBOL1\""));
        assert!(serialized.contains("\"SYMBOL2\""));
        assert!(serialized.contains("\"SYMBOL3\""));

        // Deserialize back to struct
        let deserialized: Vec<PnLMetricsDocument> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify all documents were preserved
        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0].symbol, "SYMBOL1");
        assert_eq!(deserialized[1].symbol, "SYMBOL2");
        assert_eq!(deserialized[2].symbol, "SYMBOL3");

        assert_eq!(deserialized[0].days, pos_or_panic!(30.0));
        assert_eq!(deserialized[1].days, pos_or_panic!(60.0));
        assert_eq!(deserialized[2].days, pos_or_panic!(90.0));

        assert_eq!(deserialized[0].metrics[0].step_number, 1);
        assert_eq!(deserialized[1].metrics[0].step_number, 2);
        assert_eq!(deserialized[2].metrics[0].step_number, 3);

        assert!(!deserialized[0].metrics[0].win);
        assert!(deserialized[1].metrics[0].win);
        assert!(!deserialized[2].metrics[0].win);
    }

    #[test]
    fn test_serialization_json_format() {
        // Create a simple metrics step
        let metrics_step = PnLMetricsStep {
            step_number: 42,
            win: true,
            ..Default::default()
        };

        // Get compact vs pretty serialization
        let compact = serde_json::to_string(&metrics_step).expect("Failed to serialize");
        let pretty =
            serde_json::to_string_pretty(&metrics_step).expect("Failed to serialize pretty");

        // Verify both serialize to valid, different formats
        assert!(compact.len() < pretty.len());

        // Both should deserialize back to the same object
        let from_compact: PnLMetricsStep =
            serde_json::from_str(&compact).expect("Failed to deserialize compact");
        let from_pretty: PnLMetricsStep =
            serde_json::from_str(&pretty).expect("Failed to deserialize pretty");

        assert_eq!(from_compact.step_number, 42);
        assert_eq!(from_pretty.step_number, 42);
        assert!(from_compact.win);
        assert!(from_pretty.win);
    }
}
