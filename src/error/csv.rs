/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2024
******************************************************************************/

//! Error types for OHLCV data operations.
//!
//! This module provides error types for handling failures in OHLCV (Open, High, Low, Close, Volume)
//! data operations, including CSV parsing, ZIP file handling, and data validation.

use thiserror::Error;

/// Error type for OHLCV operations
///
/// This enum represents various error conditions that can occur during OHLCV data
/// processing, including file I/O, ZIP archive handling, CSV parsing, and data validation.
///
/// # Variants
///
/// * `IoError` - File I/O errors
/// * `ZipError` - ZIP archive errors
/// * `CsvError` - CSV parsing errors
/// * `DateParseError` - Date/time parsing errors
/// * `DecimalParseError` - Decimal number parsing errors
/// * `InvalidParameter` - Invalid function parameters
/// * `OtherError` - General errors
///
/// # Examples
///
/// ```
/// use optionstratlib::error::OhlcvError;
///
/// fn read_data() -> Result<(), OhlcvError> {
///     Err(OhlcvError::IoError {
///         reason: "File not found".to_string(),
///     })
/// }
/// ```
#[derive(Error, Debug)]
pub enum OhlcvError {
    /// IO errors
    ///
    /// This variant is used when file I/O operations fail, such as reading from
    /// or writing to files.
    #[error("IO error: {reason}")]
    IoError {
        /// Reason for the error
        reason: String,
    },

    /// ZIP errors
    ///
    /// This variant is used when ZIP archive operations fail, such as opening
    /// or extracting files from ZIP archives.
    #[error("ZIP error: {reason}")]
    ZipError {
        /// Reason for the error
        reason: String,
    },

    /// CSV parsing errors
    ///
    /// This variant is used when CSV parsing fails due to malformed data,
    /// incorrect format, or missing fields.
    #[error("CSV error: {reason}")]
    CsvError {
        /// Reason for the error
        reason: String,
    },

    /// Date parsing errors
    ///
    /// This variant is used when date/time parsing fails due to invalid
    /// format or out-of-range values.
    #[error("Date parse error: {reason}")]
    DateParseError {
        /// Reason for the error
        reason: String,
    },

    /// Decimal parsing errors
    ///
    /// This variant is used when decimal number parsing fails due to
    /// invalid format or precision issues.
    #[error("Decimal parse error: {reason}")]
    DecimalParseError {
        /// Reason for the error
        reason: String,
    },

    /// Invalid parameters error
    ///
    /// This variant is used when function parameters are invalid or
    /// out of acceptable ranges.
    #[error("Invalid parameter: {reason}")]
    InvalidParameter {
        /// Reason for the error
        reason: String,
    },

    /// General error
    ///
    /// This variant is used for errors that don't fit into other categories.
    #[error("Error: {reason}")]
    OtherError {
        /// Reason for the error
        reason: String,
    },
}

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
            reason: format!("ZIP error: {error:?}"),
        }
    }
}

impl From<chrono::ParseError> for OhlcvError {
    fn from(error: chrono::ParseError) -> Self {
        Self::DateParseError {
            reason: format!("Date parse error: {error}"),
        }
    }
}

impl From<rust_decimal::Error> for OhlcvError {
    fn from(error: rust_decimal::Error) -> Self {
        Self::DecimalParseError {
            reason: format!("Decimal parse error: {error}"),
        }
    }
}
