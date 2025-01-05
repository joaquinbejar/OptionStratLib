/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/

//! # Probability Error Module
//!
//! This module provides error handling for probability analysis and calculations in option trading.
//! It defines a comprehensive error system to handle various scenarios in probability calculations,
//! profit/loss analysis, and option pricing.
//!
//! ## Main Error Types
//!
//! ### Probability Error (`ProbabilityError`)
//! Main error enum with four variants:
//! * `CalculationError` - For probability calculation failures
//! * `RangeError` - For profit/loss range analysis errors
//! * `ExpirationError` - For expiration date related errors
//! * `PriceError` - For price calculation and validation errors
//!
//! ### Calculation Errors (`ProbabilityCalculationErrorKind`)
//! Handles specific calculation failures:
//! * Invalid probability values
//! * Expected value calculation errors
//! * Volatility adjustment errors
//! * Price trend errors
//!
//! ### Range Errors (`ProfitLossRangeErrorKind`)
//! Manages profit/loss analysis errors:
//! * Invalid profit ranges
//! * Invalid loss ranges
//! * Break-even point errors
//!
//! ### Price Errors (`PriceErrorKind`)
//! Handles pricing-related errors:
//! * Invalid underlying prices
//! * Invalid price ranges
//!
//! ## Usage Example
//!
//! ```rust
//! use optionstratlib::error::probability::{ProbabilityError, ProbabilityResult};
//!
//! fn calculate_probability(value: f64) -> ProbabilityResult<f64> {
//!     if value < 0.0 || value > 1.0 {
//!         return Err(ProbabilityError::invalid_probability(
//!             value,
//!             "Probability must be between 0 and 1"
//!         ));
//!     }
//!     Ok(value)
//! }
//! ```
//!
//! ## Error Creation Helpers
//!
//! The module provides helper methods for creating common errors:
//! * `invalid_probability` - Creates an error for invalid probability values
//! * `invalid_profit_range` - Creates an error for invalid profit ranges
//! * `invalid_expiration` - Creates an error for invalid expiration dates
//!
//! ## Type Conversions
//!
//! Implements conversions from:
//! * `String` to `ProbabilityError`
//! * `&str` to `ProbabilityError`
//!
//! A type alias `ProbabilityResult<T>` is provided for convenience when working
//! with Results that may contain probability errors.

use crate::error::strategies::{BreakEvenErrorKind, OperationErrorKind, ProfitLossErrorKind};
use crate::error::StrategyError;
use std::error::Error;
use std::fmt;

/// Represents all possible errors that can occur during probability analysis calculations
#[derive(Debug)]
pub enum ProbabilityError {
    /// Errors related to probability calculations
    CalculationError(ProbabilityCalculationErrorKind),
    /// Errors related to profit/loss ranges
    RangeError(ProfitLossRangeErrorKind),
    /// Errors related to expiration dates
    ExpirationError(ExpirationErrorKind),
    /// Errors related to price parameters
    PriceError(PriceErrorKind),

    StdError(String),
}

/// Specific errors that can occur during probability calculations
#[derive(Debug)]
pub enum ProbabilityCalculationErrorKind {
    /// Error in probability calculation
    InvalidProbability { value: f64, reason: String },
    /// Error in expected value calculation
    ExpectedValueError { reason: String },
    /// Error in volatility adjustments
    VolatilityAdjustmentError { reason: String },
    /// Error in price trends
    TrendError { reason: String },
}

/// Errors related to profit and loss range calculations
#[derive(Debug)]
pub enum ProfitLossRangeErrorKind {
    /// Error in profit ranges
    InvalidProfitRange { range: String, reason: String },
    /// Error in loss ranges
    InvalidLossRange { range: String, reason: String },
    /// Error in break-even points
    InvalidBreakEvenPoints { reason: String },
}

/// Errors related to expiration dates and rates
#[derive(Debug)]
pub enum ExpirationErrorKind {
    /// Error in expiration date
    InvalidExpiration { reason: String },
    /// Error in risk-free rate
    InvalidRiskFreeRate { rate: Option<f64>, reason: String },
}

/// Errors related to price calculations and validations
#[derive(Debug)]
pub enum PriceErrorKind {
    /// Error in underlying price
    InvalidUnderlyingPrice { price: f64, reason: String },
    /// Error in price range
    InvalidPriceRange { range: String, reason: String },
}

impl fmt::Display for ProbabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbabilityError::CalculationError(err) => write!(f, "Calculation error: {}", err),
            ProbabilityError::RangeError(err) => write!(f, "Range error: {}", err),
            ProbabilityError::ExpirationError(err) => write!(f, "Expiration error: {}", err),
            ProbabilityError::PriceError(err) => write!(f, "Price error: {}", err),
            ProbabilityError::StdError(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl fmt::Display for ExpirationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpirationErrorKind::InvalidExpiration { reason } => {
                write!(f, "Invalid expiration: {}", reason)
            }
            ExpirationErrorKind::InvalidRiskFreeRate { rate, reason } => {
                write!(
                    f,
                    "Invalid risk-free rate {:?}: {}",
                    rate.unwrap_or(0.0),
                    reason
                )
            }
        }
    }
}

impl fmt::Display for PriceErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceErrorKind::InvalidUnderlyingPrice { price, reason } => {
                write!(f, "Invalid underlying price {}: {}", price, reason)
            }
            PriceErrorKind::InvalidPriceRange { range, reason } => {
                write!(f, "Invalid price range {}: {}", range, reason)
            }
        }
    }
}

impl fmt::Display for ProbabilityCalculationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbabilityCalculationErrorKind::InvalidProbability { value, reason } => {
                write!(f, "Invalid probability {}: {}", value, reason)
            }
            ProbabilityCalculationErrorKind::ExpectedValueError { reason } => {
                write!(f, "Expected value error: {}", reason)
            }
            ProbabilityCalculationErrorKind::VolatilityAdjustmentError { reason } => {
                write!(f, "Volatility adjustment error: {}", reason)
            }
            ProbabilityCalculationErrorKind::TrendError { reason } => {
                write!(f, "Trend error: {}", reason)
            }
        }
    }
}

impl fmt::Display for ProfitLossRangeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfitLossRangeErrorKind::InvalidProfitRange { range, reason } => {
                write!(f, "Invalid profit range {}: {}", range, reason)
            }
            ProfitLossRangeErrorKind::InvalidLossRange { range, reason } => {
                write!(f, "Invalid loss range {}: {}", range, reason)
            }
            ProfitLossRangeErrorKind::InvalidBreakEvenPoints { reason } => {
                write!(f, "Invalid break-even points: {}", reason)
            }
        }
    }
}

impl Error for ProbabilityError {}

impl From<Box<dyn Error>> for ProbabilityError {
    fn from(error: Box<dyn Error>) -> Self {
        ProbabilityError::StdError(error.to_string())
    }
}

/// Convenient type alias for Results with ProbabilityError
pub type ProbabilityResult<T> = Result<T, ProbabilityError>;

// Implementation of From<String> for compatibility with existing code
impl From<String> for ProbabilityError {
    fn from(msg: String) -> Self {
        ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::ExpectedValueError {
            reason: msg,
        })
    }
}

impl From<&str> for ProbabilityError {
    fn from(msg: &str) -> Self {
        ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::ExpectedValueError {
            reason: msg.to_string(),
        })
    }
}

impl From<StrategyError> for ProbabilityError {
    fn from(error: StrategyError) -> Self {
        match error {
            StrategyError::ProfitLossError(kind) => match kind {
                ProfitLossErrorKind::MaxProfitError { reason }
                | ProfitLossErrorKind::MaxLossError { reason }
                | ProfitLossErrorKind::ProfitRangeError { reason } => {
                    ProbabilityError::from(reason)
                }
            },
            StrategyError::PriceError(kind) => match kind {
                crate::error::strategies::PriceErrorKind::InvalidUnderlyingPrice { reason }
                | crate::error::strategies::PriceErrorKind::InvalidPriceRange {
                    start: _,
                    end: _,
                    reason,
                } => ProbabilityError::from(reason),
            },
            StrategyError::BreakEvenError(kind) => match kind {
                BreakEvenErrorKind::CalculationError { reason } => ProbabilityError::from(reason),
                BreakEvenErrorKind::NoBreakEvenPoints => {
                    ProbabilityError::from("No break-even points found".to_string())
                }
            },
            StrategyError::OperationError(kind) => match kind {
                OperationErrorKind::NotSupported {
                    operation,
                    strategy_type,
                } => ProbabilityError::from(format!(
                    "Operation '{}' not supported for strategy '{}'",
                    operation, strategy_type
                )),
                OperationErrorKind::InvalidParameters { operation, reason } => {
                    ProbabilityError::from(format!(
                        "Invalid parameters for operation '{}': {}",
                        operation, reason
                    ))
                }
            },
            StrategyError::StdError { reason: msg } => ProbabilityError::StdError(msg),
        }
    }
}

// Helper functions to create common errors
impl ProbabilityError {
    /// Creates a new invalid probability error
    pub fn invalid_probability(value: f64, reason: &str) -> Self {
        ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::InvalidProbability {
            value,
            reason: reason.to_string(),
        })
    }

    /// Creates a new invalid profit range error
    pub fn invalid_profit_range(range: &str, reason: &str) -> Self {
        ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidProfitRange {
            range: range.to_string(),
            reason: reason.to_string(),
        })
    }

    /// Creates a new invalid expiration error
    pub fn invalid_expiration(reason: &str) -> Self {
        ProbabilityError::ExpirationError(ExpirationErrorKind::InvalidExpiration {
            reason: reason.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_probability_error() {
        let error = ProbabilityError::invalid_probability(1.2, "Probability cannot exceed 1.0");
        assert!(matches!(
            error,
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::InvalidProbability { .. }
            )
        ));
    }

    #[test]
    fn test_string_conversion() {
        let error = ProbabilityError::from("Test error message".to_string());
        assert!(matches!(
            error,
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::ExpectedValueError { .. }
            )
        ));
    }

    #[test]
    fn test_error_formatting() {
        let error = ProbabilityError::invalid_probability(1.2, "Probability cannot exceed 1.0");
        let error_string = error.to_string();
        assert!(error_string.contains("Invalid probability"));
        assert!(error_string.contains("1.2"));
        assert!(error_string.contains("Probability cannot exceed 1.0"));
    }

    #[test]
    fn test_expiration_error_display() {
        let error = ProbabilityError::ExpirationError(ExpirationErrorKind::InvalidExpiration {
            reason: "Cannot be in the past".to_string(),
        });
        assert!(error.to_string().contains("Cannot be in the past"));
    }

    #[test]
    fn test_price_error_display() {
        let error = ProbabilityError::PriceError(PriceErrorKind::InvalidUnderlyingPrice {
            price: -10.0,
            reason: "Price cannot be negative".to_string(),
        });
        assert!(error.to_string().contains("Price cannot be negative"));
        assert!(error.to_string().contains("-10"));
    }

    #[test]
    fn test_str_conversion() {
        let error = ProbabilityError::from("Test error message");
        assert!(matches!(
            error,
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::ExpectedValueError { .. }
            )
        ));
    }
}
