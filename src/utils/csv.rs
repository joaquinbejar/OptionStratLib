use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromStr;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use zip::ZipArchive;

/// Represents an OHLC+V candlestick with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvCandle {
    /// Date of the candle
    pub date: NaiveDate,
    /// Time of the candle in HH:MM:SS format
    pub time: String,
    /// Opening price
    pub open: Decimal,
    /// Highest price during the period
    pub high: Decimal,
    /// Lowest price during the period
    pub low: Decimal,
    /// Closing price
    pub close: Decimal,
    /// Volume traded during the period
    pub volume: u64,
}

/// Error type for OHLCV operations
#[derive(Debug)]
pub enum OhlcvError {
    /// IO errors
    IoError { reason: String },

    /// ZIP errors
    ZipError { reason: String },

    /// CSV parsing errors
    CsvError { reason: String },

    /// Date parsing errors
    DateParseError { reason: String },

    /// Decimal parsing errors
    DecimalParseError { reason: String },

    /// Invalid parameters error
    InvalidParameter { reason: String },

    /// General error
    OtherError { reason: String },
}

impl std::fmt::Display for OhlcvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError { reason, .. } => write!(f, "IO error: {}", reason),
            Self::ZipError { reason, .. } => write!(f, "ZIP error: {}", reason),
            Self::CsvError { reason } => write!(f, "CSV error: {}", reason),
            Self::DateParseError { reason, .. } => write!(f, "Date parse error: {}", reason),
            Self::DecimalParseError { reason, .. } => write!(f, "Decimal parse error: {}", reason),
            Self::InvalidParameter { reason } => write!(f, "Invalid parameter {}", reason),
            Self::OtherError { reason } => write!(f, "Error: {}", reason),
        }
    }
}

impl std::error::Error for OhlcvError {}

impl From<std::io::Error> for OhlcvError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError {
            reason: error.to_string(),
        }
    }
}

impl From<zip::result::ZipError> for OhlcvError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::ZipError {
            reason: format!("ZIP error: {:?}", error),
        }
    }
}

impl From<chrono::ParseError> for OhlcvError {
    fn from(error: chrono::ParseError) -> Self {
        Self::DateParseError {
            reason: format!("Date parse error: {}", error),
        }
    }
}

impl From<rust_decimal::Error> for OhlcvError {
    fn from(error: rust_decimal::Error) -> Self {
        Self::DecimalParseError {
            reason: format!("Decimal parse error: {}", error),
        }
    }
}

/// Reads OHLCV data from a zipped CSV file and filters it by date range
///
/// # Arguments
///
/// * `zip_path` - Path to the ZIP file containing the CSV
/// * `start_date` - Start date in DD/MM/YYYY format (inclusive)
/// * `end_date` - End date in DD/MM/YYYY format (inclusive)
///
/// # Returns
///
/// A vector of OhlcvCandle structs containing the filtered data
///
/// # Errors
///
/// Returns an OhlcvError if:
/// - The ZIP file cannot be opened
/// - The CSV file within the ZIP cannot be read
/// - The date range is invalid
/// - Data parsing fails
pub fn read_ohlcv_from_zip(
    zip_path: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<OhlcvCandle>, OhlcvError> {
    // Parse date range
    let start = NaiveDate::parse_from_str(start_date, "%d/%m/%Y")?;
    let end = NaiveDate::parse_from_str(end_date, "%d/%m/%Y")?;

    // Validate date range
    if start > end {
        return Err(OhlcvError::InvalidParameter {
            reason: format!("Start date {} is after end date {}", start_date, end_date),
        });
    }

    // Open the ZIP file
    let file = File::open(Path::new(zip_path))?;
    let mut archive = ZipArchive::new(file)?;

    // Find the first CSV file in the archive
    let mut csv_index = None;
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.name().ends_with(".csv") {
            csv_index = Some(i);
            break;
        }
    }

    let csv_index = csv_index.ok_or(OhlcvError::OtherError {
        reason: "No CSV file found in ZIP archive".to_string(),
    })?;
    let file = archive.by_index(csv_index)?;
    let reader = BufReader::new(file);

    let mut candles = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        // Skip header if present (line 0)
        if line_num == 0
            && line_result
                .as_ref().is_ok_and(|l| l.contains("date") || l.contains("Date"))
        {
            continue;
        }

        let line = line_result?;
        let parts: Vec<&str> = line.split(';').collect();

        // Ensure we have 7 parts: date, time, open, high, low, close, volume
        if parts.len() != 7 {
            return Err(OhlcvError::CsvError {
                reason: format!(
                    "Invalid CSV format at line {}: expected 7 fields, got {}",
                    line_num + 1,
                    parts.len()
                ),
            });
        }

        // Parse date
        let date = NaiveDate::parse_from_str(parts[0], "%d/%m/%Y")?;

        // Skip records outside our date range
        if date < start || date > end {
            continue;
        }

        // Parse other fields
        let candle = OhlcvCandle {
            date,
            time: parts[1].to_string(),
            open: Decimal::from_str(parts[2])?,
            high: Decimal::from_str(parts[3])?,
            low: Decimal::from_str(parts[4])?,
            close: Decimal::from_str(parts[5])?,
            volume: parts[6].parse::<u64>().map_err(|e| OhlcvError::CsvError {
                reason: format!("Invalid volume at line {}: {}", line_num + 1, e),
            })?,
        };

        candles.push(candle);
    }

    Ok(candles)
}


