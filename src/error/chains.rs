/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/

//! # Chain Error Module
//!
//! This module provides error handling for operations related to option chains and their data.
//! It defines a comprehensive error hierarchy to handle various failure scenarios in option chain
//! operations, data validation, and file handling.
//!
//! ## Error Types
//!
//! * `ChainError` - The main error enum that encompasses all possible chain-related errors
//! * `OptionDataErrorKind` - Specific errors related to option data validation
//! * `ChainBuildErrorKind` - Errors that can occur during chain construction
//! * `FileErrorKind` - File operation related errors
//! * `StrategyErrorKind` - Strategy-specific validation errors
//!
//! ## Usage Example
//!
//! ```rust
//! use optionstratlib::error::chains::ChainError;
//!
//! fn validate_strike_price(strike: f64) -> Result<(), ChainError> {
//!     if strike <= 0.0 {
//!         return Err(ChainError::invalid_strike(
//!             strike,
//!             "Strike price must be positive"
//!         ));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Error Creation Helpers
//!
//! The module provides several helper methods for creating common errors:
//!
//! * `invalid_strike` - Creates an error for invalid strike prices
//! * `invalid_volatility` - Creates an error for invalid volatility values
//! * `invalid_prices` - Creates an error for invalid bid/ask prices
//! * `invalid_legs` - Creates an error for invalid strategy legs
//! * `invalid_parameters` - Creates an error for invalid chain building parameters
//!
//! ## Conversions
//!
//! The module implements various conversion traits:
//!
//! * `From<io::Error>` - Converts IO errors to chain errors
//! * `From<String>` - Converts string messages to price calculation errors
//!
//! All error types implement `std::error::Error` and `std::fmt::Display` for proper error
//! handling and formatting.

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ChainError {
    /// Errors related to option data validation
    OptionDataError(OptionDataErrorKind),
    /// Errors related to chain building
    ChainBuildError(ChainBuildErrorKind),
    /// Errors related to file operations
    FileError(FileErrorKind),
    /// Errors related to strategies
    StrategyError(StrategyErrorKind),
    
    DynError { message: String },
}

/// Specific errors for option data
#[derive(Debug)]
pub enum OptionDataErrorKind {
    /// Invalid strike price
    InvalidStrike { strike: f64, reason: String },
    /// Invalid implied volatility
    InvalidVolatility {
        volatility: Option<f64>,
        reason: String,
    },
    /// Invalid bid/ask prices
    InvalidPrices {
        bid: Option<f64>,
        ask: Option<f64>,
        reason: String,
    },
    /// Invalid delta
    InvalidDelta { delta: Option<f64>, reason: String },
    /// Error in price calculation
    PriceCalculationError(String),
}

/// Specific errors for chain building
#[derive(Debug)]
pub enum ChainBuildErrorKind {
    /// Invalid parameters for building
    InvalidParameters { parameter: String, reason: String },
    /// Error in volatility adjustment
    VolatilityAdjustmentError { skew_factor: f64, reason: String },
    /// Error in strike generation
    StrikeGenerationError {
        reference_price: f64,
        interval: f64,
        reason: String,
    },
}

/// Errors related to file operations
#[derive(Debug)]
pub enum FileErrorKind {
    /// Error when reading/writing file
    IOError(io::Error),
    /// Error in file format
    InvalidFormat { format: String, reason: String },
    /// Error in data parsing
    ParseError {
        line: usize,
        content: String,
        reason: String,
    },
}

// Specific errors for strategies
#[derive(Debug, PartialEq)]
pub enum StrategyErrorKind {
    /// Error in legs validation
    InvalidLegs {
        expected: usize,
        found: usize,
        reason: String,
    },
    /// Error in options combination
    InvalidCombination {
        strategy_type: String,
        reason: String,
    },
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainError::OptionDataError(err) => write!(f, "Option data error: {}", err),
            ChainError::ChainBuildError(err) => write!(f, "Chain build error: {}", err),
            ChainError::FileError(err) => write!(f, "File error: {}", err),
            ChainError::StrategyError(err) => write!(f, "Strategy error: {}", err),
            ChainError::DynError { message } => write!(f, "Error: {}", message),
        }
    }
}

impl fmt::Display for OptionDataErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionDataErrorKind::InvalidStrike { strike, reason } => {
                write!(f, "Invalid strike price {}: {}", strike, reason)
            }
            OptionDataErrorKind::InvalidVolatility { volatility, reason } => {
                write!(
                    f,
                    "Invalid volatility {:?}: {}",
                    volatility.unwrap_or(0.0),
                    reason
                )
            }
            OptionDataErrorKind::InvalidPrices { bid, ask, reason } => {
                write!(
                    f,
                    "Invalid prices (bid: {:?}, ask: {:?}): {}",
                    bid, ask, reason
                )
            }
            OptionDataErrorKind::InvalidDelta { delta, reason } => {
                write!(f, "Invalid delta {:?}: {}", delta, reason)
            }
            OptionDataErrorKind::PriceCalculationError(msg) => {
                write!(f, "Price calculation error: {}", msg)
            }
        }
    }
}

impl fmt::Display for ChainBuildErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainBuildErrorKind::InvalidParameters { parameter, reason } => {
                write!(f, "Invalid parameter '{}': {}", parameter, reason)
            }
            ChainBuildErrorKind::VolatilityAdjustmentError {
                skew_factor,
                reason,
            } => {
                write!(
                    f,
                    "Volatility adjustment error (skew factor: {}): {}",
                    skew_factor, reason
                )
            }
            ChainBuildErrorKind::StrikeGenerationError {
                reference_price,
                interval,
                reason,
            } => {
                write!(
                    f,
                    "Strike generation error (reference: {}, interval: {}): {}",
                    reference_price, interval, reason
                )
            }
        }
    }
}

impl fmt::Display for FileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileErrorKind::IOError(err) => write!(f, "IO error: {}", err),
            FileErrorKind::InvalidFormat { format, reason } => {
                write!(f, "Invalid {} format: {}", format, reason)
            }
            FileErrorKind::ParseError {
                line,
                content,
                reason,
            } => {
                write!(
                    f,
                    "Parse error at line {}, content '{}': {}",
                    line, content, reason
                )
            }
        }
    }
}

impl fmt::Display for StrategyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyErrorKind::InvalidLegs {
                expected,
                found,
                reason,
            } => {
                write!(
                    f,
                    "Invalid number of legs (expected: {}, found: {}): {}",
                    expected, found, reason
                )
            }
            StrategyErrorKind::InvalidCombination {
                strategy_type,
                reason,
            } => {
                write!(
                    f,
                    "Invalid combination for strategy '{}': {}",
                    strategy_type, reason
                )
            }
        }
    }
}

impl Error for ChainError {}

impl From<io::Error> for ChainError {
    fn from(error: io::Error) -> Self {
        ChainError::FileError(FileErrorKind::IOError(error))
    }
}

impl ChainError {
    pub fn invalid_strike(strike: f64, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidStrike {
            strike,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_volatility(volatility: Option<f64>, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
            volatility,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_prices(bid: Option<f64>, ask: Option<f64>, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidPrices {
            bid,
            ask,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_legs(expected: usize, found: usize, reason: &str) -> Self {
        ChainError::StrategyError(StrategyErrorKind::InvalidLegs {
            expected,
            found,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_parameters(parameter: &str, reason: &str) -> Self {
        ChainError::ChainBuildError(ChainBuildErrorKind::InvalidParameters {
            parameter: parameter.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl From<String> for ChainError {
    fn from(msg: String) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::PriceCalculationError(msg))
    }
}

impl From<Box<dyn Error>> for ChainError {
    fn from(error: Box<dyn Error>) -> Self {
        ChainError::DynError{ message: error.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_data_errors() {
        let error = ChainError::invalid_strike(-10.0, "Strike cannot be negative");
        assert!(matches!(
            error,
            ChainError::OptionDataError(OptionDataErrorKind::InvalidStrike { .. })
        ));

        let error = ChainError::invalid_volatility(Some(-0.5), "Volatility must be positive");
        assert!(matches!(
            error,
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility { .. })
        ));
    }

    #[test]
    fn test_error_messages() {
        let error = ChainError::invalid_strike(0.0, "Strike must be positive");
        assert!(error.to_string().contains("Strike must be positive"));
    }

    #[test]
    fn test_chain_build_errors() {
        let error = ChainError::ChainBuildError(ChainBuildErrorKind::InvalidParameters {
            parameter: "chain_size".to_string(),
            reason: "Must be greater than 0".to_string(),
        });
        assert!(error.to_string().contains("chain_size"));
        assert!(error.to_string().contains("Must be greater than 0"));
    }

    #[test]
    fn test_strategy_errors() {
        let error = ChainError::invalid_legs(4, 3, "Iron Condor requires exactly 4 legs");
        assert!(error.to_string().contains("4"));
        assert!(error.to_string().contains("3"));
        assert!(error.to_string().contains("Iron Condor"));
    }

    #[test]
    fn test_file_errors() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = ChainError::from(io_error);
        assert!(matches!(
            error,
            ChainError::FileError(FileErrorKind::IOError(..))
        ));
    }
}
