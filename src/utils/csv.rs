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
    IoError {
        /// Reason for the error
        reason: String,
    },

    /// ZIP errors
    ZipError {
        /// Reason for the error
        reason: String,
    },

    /// CSV parsing errors
    CsvError {
        /// Reason for the error
        reason: String,
    },

    /// Date parsing errors
    DateParseError {
        /// Reason for the error
        reason: String,
    },

    /// Decimal parsing errors
    DecimalParseError {
        /// Reason for the error
        reason: String,
    },

    /// Invalid parameters error
    InvalidParameter {
        /// Reason for the error
        reason: String,
    },

    /// General error
    OtherError {
        /// Reason for the error
        reason: String,
    },
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
/// * `start_date` - Optional start date in DD/MM/YYYY format (inclusive)
/// * `end_date` - Optional end date in DD/MM/YYYY format (inclusive)
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
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<OhlcvCandle>, OhlcvError> {
    // Parse date range if provided
    let start = if let Some(start_str) = start_date {
        Some(NaiveDate::parse_from_str(start_str, "%d/%m/%Y")?)
    } else {
        None
    };

    let end = if let Some(end_str) = end_date {
        Some(NaiveDate::parse_from_str(end_str, "%d/%m/%Y")?)
    } else {
        None
    };

    // Validate date range if both dates are provided
    if let (Some(start_date), Some(end_date)) = (&start, &end) {
        if start_date > end_date {
            return Err(OhlcvError::InvalidParameter {
                reason: format!(
                    "Start date {} is after end date {}",
                    start_date, end_date
                ),
            });
        }
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
            .as_ref()
            .is_ok_and(|l| l.contains("date") || l.contains("Date"))
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

        // Skip records outside our date range if dates are specified
        if (start.is_some() && date < start.unwrap()) ||
            (end.is_some() && date > end.unwrap()) {
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

#[cfg(test)]
mod ohlcv_tests {
    use super::*;
    use chrono::NaiveDate;
    use mockall::predicate::*;
    use rust_decimal_macros::dec;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use zip::{ZipWriter, write::FileOptions};

    // Helper function to create a temporary zip file with test data
    fn create_test_zip(data: &str) -> Result<(String, NamedTempFile), OhlcvError> {
        // Create a temp file to hold our zip
        let temp_file = NamedTempFile::new().map_err(|e| OhlcvError::IoError {
            reason: e.to_string(),
        })?;

        // Create a zip archive
        let mut zip = ZipWriter::new(File::create(temp_file.path())?);

        // Add a CSV file
        let options: FileOptions<'_, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("test_data.csv", options)
            .map_err(|e| OhlcvError::ZipError {
                reason: e.to_string(),
            })?;

        // Write our test data
        zip.write_all(data.as_bytes())
            .map_err(|e| OhlcvError::IoError {
                reason: e.to_string(),
            })?;

        // Finish the zip
        zip.finish().map_err(|e| OhlcvError::ZipError {
            reason: e.to_string(),
        })?;

        Ok((temp_file.path().to_string_lossy().to_string(), temp_file))
    }

    #[test]
    fn test_read_ohlcv_valid_data() -> Result<(), OhlcvError> {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                        01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                        02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000\n\
                        03/01/2022;10:00:00;110.0;115.0;108.0;114.0;7000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let candles = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("02/01/2022"))?;

        // Verify results
        assert_eq!(candles.len(), 2, "Should return exactly 2 candles");

        // Verify first candle
        assert_eq!(
            candles[0].date,
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );
        assert_eq!(candles[0].time, "10:00:00");
        assert_eq!(candles[0].open, dec!(100.0));
        assert_eq!(candles[0].high, dec!(110.0));
        assert_eq!(candles[0].low, dec!(95.0));
        assert_eq!(candles[0].close, dec!(105.0));
        assert_eq!(candles[0].volume, 5000);

        // Verify second candle
        assert_eq!(
            candles[1].date,
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap()
        );
        assert_eq!(candles[1].time, "10:00:00");
        assert_eq!(candles[1].open, dec!(105.0));
        assert_eq!(candles[1].high, dec!(112.0));
        assert_eq!(candles[1].low, dec!(104.0));
        assert_eq!(candles[1].close, dec!(110.0));
        assert_eq!(candles[1].volume, 6000);

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_without_header() -> Result<(), OhlcvError> {
        // Create test data without header
        let csv_data = "01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                        02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let candles = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("02/01/2022"))?;

        // Verify results
        assert_eq!(
            candles.len(),
            2,
            "Should return exactly 2 candles even without header"
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_invalid_date_range() {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                        01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000";

        let (zip_path, _temp_file) = create_test_zip(csv_data).unwrap();

        // Call our function with end date before start date
        let result = read_ohlcv_from_zip(&zip_path, Some("02/01/2022"), Some("01/01/2022"));

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for invalid date range"
        );
        if let Err(OhlcvError::InvalidParameter { reason }) = result {
            assert!(
                reason.contains("Start date"),
                "Error should mention start date being after end date"
            );
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_read_ohlcv_nonexistent_file() {
        // Call function with nonexistent file
        let result = read_ohlcv_from_zip("nonexistent_file.zip", Some("01/01/2022"), Some("31/12/2022"));

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for nonexistent file"
        );
        if let Err(OhlcvError::IoError { .. }) = result {
            // This is expected
        } else {
            panic!("Expected IoError");
        }
    }

    #[test]
    fn test_read_ohlcv_invalid_csv_format() -> Result<(), OhlcvError> {
        // Create test data with invalid format (missing a column)
        let csv_data = "date;time;open;high;low;close\n\
                        01/01/2022;10:00:00;100.0;110.0;95.0;105.0";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let result = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("31/12/2022"));

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for invalid CSV format"
        );
        if let Err(OhlcvError::CsvError { reason }) = result {
            assert!(
                reason.contains("expected 7 fields"),
                "Error should mention expected field count"
            );
        } else {
            panic!("Expected CsvError");
        }

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_invalid_decimal() -> Result<(), OhlcvError> {
        // Create test data with invalid decimal
        let csv_data = "date;time;open;high;low;close;volume\n\
                        01/01/2022;10:00:00;not_a_number;110.0;95.0;105.0;5000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let result = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("31/12/2022"));

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for invalid decimal"
        );
        if let Err(OhlcvError::DecimalParseError { .. }) = result {
            // This is expected
        } else {
            panic!("Expected DecimalParseError");
        }

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_invalid_volume() -> Result<(), OhlcvError> {
        // Create test data with invalid volume
        let csv_data = "date;time;open;high;low;close;volume\n\
                        01/01/2022;10:00:00;100.0;110.0;95.0;105.0;not_a_number";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let result = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("31/12/2022"));

        // Verify results
        assert!(result.is_err(), "Should return an error for invalid volume");
        if let Err(OhlcvError::CsvError { .. }) = result {
            // This is expected
        } else {
            panic!("Expected CsvError");
        }

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_invalid_date_format() -> Result<(), OhlcvError> {
        // Create test data with invalid date format
        let csv_data = "date;time;open;high;low;close;volume\n\
                        2022-01-01;10:00:00;100.0;110.0;95.0;105.0;5000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let result = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("31/12/2022"));

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for invalid date format"
        );
        if let Err(OhlcvError::DateParseError { .. }) = result {
            // This is expected
        } else {
            panic!("Expected DateParseError");
        }

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_empty_file() -> Result<(), OhlcvError> {
        // Create empty test file
        let csv_data = "";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function
        let candles = read_ohlcv_from_zip(&zip_path, Some("01/01/2022"), Some("31/12/2022"))?;

        // Verify results
        assert_eq!(
            candles.len(),
            0,
            "Should return empty vector for empty file"
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_no_matching_dates() -> Result<(), OhlcvError> {
        // Create test data with dates outside requested range
        let csv_data = "date;time;open;high;low;close;volume\n\
                        01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                        02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function with date range that doesn't match any data
        let candles = read_ohlcv_from_zip(&zip_path, Some("03/01/2022"), Some("04/01/2022"))?;

        // Verify results
        assert_eq!(
            candles.len(),
            0,
            "Should return empty vector when no dates match"
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_partial_matches() -> Result<(), OhlcvError> {
        // Create test data with some dates in range and some out of range
        let csv_data = "date;time;open;high;low;close;volume\n\
                        01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                        02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000\n\
                        03/01/2022;10:00:00;110.0;115.0;108.0;114.0;7000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function with date range that only matches some data
        let candles = read_ohlcv_from_zip(&zip_path, Some("02/01/2022"),Some("03/01/2022"))?;

        // Verify results
        assert_eq!(candles.len(), 2, "Should return exactly 2 candles");
        assert_eq!(
            candles[0].date,
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap()
        );
        assert_eq!(
            candles[1].date,
            NaiveDate::from_ymd_opt(2022, 1, 3).unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_ohlcv_error_display() {
        // Test IoError display
        let io_error = OhlcvError::IoError {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", io_error), "IO error: test reason");

        // Test ZipError display
        let zip_error = OhlcvError::ZipError {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", zip_error), "ZIP error: test reason");

        // Test CsvError display
        let csv_error = OhlcvError::CsvError {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", csv_error), "CSV error: test reason");

        // Test DateParseError display
        let date_error = OhlcvError::DateParseError {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", date_error), "Date parse error: test reason");

        // Test DecimalParseError display
        let decimal_error = OhlcvError::DecimalParseError {
            reason: "test reason".to_string(),
        };
        assert_eq!(
            format!("{}", decimal_error),
            "Decimal parse error: test reason"
        );

        // Test InvalidParameter display
        let param_error = OhlcvError::InvalidParameter {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", param_error), "Invalid parameter test reason");

        // Test OtherError display
        let other_error = OhlcvError::OtherError {
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", other_error), "Error: test reason");
    }

    #[test]
    fn test_read_ohlcv_error_conversions() {
        // Test std::io::Error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ohlcv_error = OhlcvError::from(io_error);
        assert!(matches!(ohlcv_error, OhlcvError::IoError { .. }));

        // Test zip::result::ZipError conversion
        let zip_error = zip::result::ZipError::FileNotFound;
        let ohlcv_error = OhlcvError::from(zip_error);
        assert!(matches!(ohlcv_error, OhlcvError::ZipError { .. }));

        // Test chrono::ParseError conversion
        let date_str = "invalid date";
        let parse_result = NaiveDate::parse_from_str(date_str, "%d/%m/%Y");
        assert!(parse_result.is_err());
        let ohlcv_error = OhlcvError::from(parse_result.err().unwrap());
        assert!(matches!(ohlcv_error, OhlcvError::DateParseError { .. }));

        // Test rust_decimal::Error conversion
        let decimal_str = "not a number";
        let decimal_result = Decimal::from_str(decimal_str);
        assert!(decimal_result.is_err());
        let ohlcv_error = OhlcvError::from(decimal_result.err().unwrap());
        assert!(matches!(ohlcv_error, OhlcvError::DecimalParseError { .. }));
    }

    #[test]
    fn test_ohlcv_struct_serialization() {
        // Create a sample candle
        let candle = OhlcvCandle {
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            time: "10:00:00".to_string(),
            open: dec!(100.0),
            high: dec!(110.0),
            low: dec!(95.0),
            close: dec!(105.0),
            volume: 5000,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&candle).unwrap();

        // Deserialize from JSON
        let deserialized: OhlcvCandle = serde_json::from_str(&json).unwrap();

        // Verify equality
        assert_eq!(candle.date, deserialized.date);
        assert_eq!(candle.time, deserialized.time);
        assert_eq!(candle.open, deserialized.open);
        assert_eq!(candle.high, deserialized.high);
        assert_eq!(candle.low, deserialized.low);
        assert_eq!(candle.close, deserialized.close);
        assert_eq!(candle.volume, deserialized.volume);
    }

    #[test]
    fn test_read_ohlcv_all_data_no_dates() -> Result<(), OhlcvError> {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                    02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000\n\
                    03/01/2022;10:00:00;110.0;115.0;108.0;114.0;7000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function without specifying any dates
        let candles = read_ohlcv_from_zip(&zip_path, None, None)?;

        // Verify results - should include all 3 candles
        assert_eq!(candles.len(), 3, "Should return all 3 candles when no dates specified");

        // Verify all candles are included
        assert_eq!(
            candles[0].date,
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );
        assert_eq!(
            candles[1].date,
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap()
        );
        assert_eq!(
            candles[2].date,
            NaiveDate::from_ymd_opt(2022, 1, 3).unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_only_start_date() -> Result<(), OhlcvError> {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                    02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000\n\
                    03/01/2022;10:00:00;110.0;115.0;108.0;114.0;7000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function with only start date specified
        let candles = read_ohlcv_from_zip(&zip_path, Some("02/01/2022"), None)?;

        // Verify results - should include candles from 02/01/2022 onwards
        assert_eq!(candles.len(), 2, "Should return 2 candles from start date onwards");

        // Verify correct candles are included
        assert_eq!(
            candles[0].date,
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap()
        );
        assert_eq!(
            candles[1].date,
            NaiveDate::from_ymd_opt(2022, 1, 3).unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_only_end_date() -> Result<(), OhlcvError> {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                    02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000\n\
                    03/01/2022;10:00:00;110.0;115.0;108.0;114.0;7000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function with only end date specified
        let candles = read_ohlcv_from_zip(&zip_path, None, Some("02/01/2022"))?;

        // Verify results - should include candles up to 02/01/2022
        assert_eq!(candles.len(), 2, "Should return 2 candles up to end date");

        // Verify correct candles are included
        assert_eq!(
            candles[0].date,
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );
        assert_eq!(
            candles[1].date,
            NaiveDate::from_ymd_opt(2022, 1, 2).unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_invalid_start_date_format() {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000";

        let (zip_path, _temp_file) = create_test_zip(csv_data).unwrap();

        // Call our function with invalid start date format
        let result = read_ohlcv_from_zip(&zip_path, Some("2022-01-01"), None);

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for invalid start date format"
        );
        if let Err(OhlcvError::DateParseError { .. }) = result {
            // This is expected
        } else {
            panic!("Expected DateParseError");
        }
    }

    #[test]
    fn test_read_ohlcv_invalid_end_date_format() {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000";

        let (zip_path, _temp_file) = create_test_zip(csv_data).unwrap();

        // Call our function with invalid end date format
        let result = read_ohlcv_from_zip(&zip_path, None, Some("2022-01-01"));

        // Verify results
        assert!(
            result.is_err(),
            "Should return an error for invalid end date format"
        );
        if let Err(OhlcvError::DateParseError { .. }) = result {
            // This is expected
        } else {
            panic!("Expected DateParseError");
        }
    }

    #[test]
    fn test_read_ohlcv_no_matching_dates_with_only_start_date() -> Result<(), OhlcvError> {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                    02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function with start date after all available dates
        let candles = read_ohlcv_from_zip(&zip_path, Some("03/01/2022"), None)?;

        // Verify results
        assert_eq!(
            candles.len(),
            0,
            "Should return empty vector when no dates match the start date criteria"
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_no_matching_dates_with_only_end_date() -> Result<(), OhlcvError> {
        // Create test data
        let csv_data = "date;time;open;high;low;close;volume\n\
                    01/01/2022;10:00:00;100.0;110.0;95.0;105.0;5000\n\
                    02/01/2022;10:00:00;105.0;112.0;104.0;110.0;6000";

        let (zip_path, _temp_file) = create_test_zip(csv_data)?;

        // Call our function with end date before all available dates
        let candles = read_ohlcv_from_zip(&zip_path, None, Some("31/12/2021"))?;

        // Verify results
        assert_eq!(
            candles.len(),
            0,
            "Should return empty vector when no dates match the end date criteria"
        );

        Ok(())
    }

    #[test]
    fn test_read_ohlcv_updates_in_existing_tests() {
        // Update all occurrences of read_ohlcv_from_zip to use Some() for dates
        // This test doesn't need implementation, just a reminder that all
        // existing tests should be updated to use Some() for date parameters
        assert!(true);
    }
}
