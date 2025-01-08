/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com
    Date: 24/12/24
******************************************************************************/

//! # Strategy Error Module
//!
//! This module provides error handling for option trading strategies and their operations.
//! It defines error types for strategy calculations, validations, and operations,
//! with integration with the probability analysis system.
//!
//! ## Error Types
//!
//! ### Strategy Error (`StrategyError`)
//! The main error enum with four categories:
//! * `PriceError` - For price calculation failures
//! * `BreakEvenError` - For break-even point calculation errors
//! * `ProfitLossError` - For profit/loss calculation failures
//! * `OperationError` - For strategy operation errors
//!
//! ### Price Errors (`PriceErrorKind`)
//! Specific errors for price-related operations:
//! * Invalid underlying prices
//! * Invalid price ranges with start and end points
//!
//! ### Break-Even Errors (`BreakEvenErrorKind`)
//! Handles break-even point calculations:
//! * Calculation failures
//! * Missing break-even points
//!
//! ### Profit/Loss Errors (`ProfitLossErrorKind`)
//! Manages profit and loss calculations:
//! * Maximum profit calculation errors
//! * Maximum loss calculation errors
//! * Profit range calculation errors
//!
//! ## Integration with Probability Analysis
//!
//! Implements conversion from `StrategyError` to `ProbabilityError` for seamless
//! error handling between strategy and probability calculations.
//!
//! ## Usage Example
//!
//! ```rust
//!
//! use optionstratlib::error::strategies::{StrategyError, StrategyResult};
//!
//! fn validate_strategy_operation(operation: &str, strategy: &str) -> StrategyResult<()> {
//!     if !is_supported_operation(operation) {
//!         return Err(StrategyError::operation_not_supported(operation, strategy));
//!     }
//!     Ok(())
//! }
//!
//! fn is_supported_operation(p0: &str) -> bool  {
//!     false
//! }
//! ```
//!
//! ## Helper Methods
//!
//! The module provides convenient methods for creating common errors:
//! * `operation_not_supported` - Creates an error for unsupported operations
//! * `invalid_parameters` - Creates an error for invalid operation parameters
//!
//! ## Type Alias
//!
//! Provides `StrategyResult<T>` for convenient error handling in strategy operations.
use crate::error::PositionError;
use std::error::Error;
use std::fmt;
use crate::error::common::OperationErrorKind;

impl Error for StrategyError {}
impl Error for PriceErrorKind {}
impl Error for BreakEvenErrorKind {}
impl Error for ProfitLossErrorKind {}

#[derive(Debug)]
pub enum StrategyError {
    /// Errors related to price calculations
    PriceError(PriceErrorKind),
    /// Errors related to break-even points
    BreakEvenError(BreakEvenErrorKind),
    /// Errors related to profit/loss calculations
    ProfitLossError(ProfitLossErrorKind),
    /// Errors related to strategy operations
    OperationError(OperationErrorKind),

    StdError {
        reason: String,
    },
}

#[derive(Debug)]
pub enum PriceErrorKind {
    /// Error when underlying price is not available or invalid
    InvalidUnderlyingPrice { reason: String },
    /// Error in price range calculation
    InvalidPriceRange {
        start: f64,
        end: f64,
        reason: String,
    },
}

#[derive(Debug)]
pub enum BreakEvenErrorKind {
    /// Error calculating break-even points
    CalculationError { reason: String },
    /// No break-even points found
    NoBreakEvenPoints,
}

#[derive(Debug)]
pub enum ProfitLossErrorKind {
    /// Error calculating maximum profit
    MaxProfitError { reason: String },
    /// Error calculating maximum loss
    MaxLossError { reason: String },
    /// Error in profit range calculation
    ProfitRangeError { reason: String },
}


impl fmt::Display for StrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyError::PriceError(err) => write!(f, "Price error: {}", err),
            StrategyError::BreakEvenError(err) => write!(f, "Break-even error: {}", err),
            StrategyError::ProfitLossError(err) => write!(f, "Profit/Loss error: {}", err),
            StrategyError::OperationError(err) => write!(f, "Operation error: {}", err),
            StrategyError::StdError { reason } => write!(f, "Error: {}", reason),
        }
    }
}

impl fmt::Display for PriceErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceErrorKind::InvalidUnderlyingPrice { reason } => {
                write!(f, "Invalid underlying price: {}", reason)
            }
            PriceErrorKind::InvalidPriceRange { start, end, reason } => {
                write!(f, "Invalid price range [{}, {}]: {}", start, end, reason)
            }
        }
    }
}

impl fmt::Display for BreakEvenErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakEvenErrorKind::CalculationError { reason } => {
                write!(f, "Break-even calculation error: {}", reason)
            }
            BreakEvenErrorKind::NoBreakEvenPoints => {
                write!(f, "No break-even points found")
            }
        }
    }
}

impl fmt::Display for ProfitLossErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfitLossErrorKind::MaxProfitError { reason } => {
                write!(f, "Maximum profit calculation error: {}", reason)
            }
            ProfitLossErrorKind::MaxLossError { reason } => {
                write!(f, "Maximum loss calculation error: {}", reason)
            }
            ProfitLossErrorKind::ProfitRangeError { reason } => {
                write!(f, "Profit range calculation error: {}", reason)
            }
        }
    }
}



// Type alias for convenience
pub type StrategyResult<T> = Result<T, StrategyError>;

// Implementation helpers
impl StrategyError {
    pub fn operation_not_supported(operation: &str, strategy_type: &str) -> Self {
        StrategyError::OperationError(OperationErrorKind::NotSupported {
            operation: operation.to_string(),
            strategy_type: strategy_type.to_string(),
        })
    }

    pub fn invalid_parameters(operation: &str, reason: &str) -> Self {
        StrategyError::OperationError(OperationErrorKind::InvalidParameters {
            operation: operation.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl From<PositionError> for StrategyError {
    fn from(err: PositionError) -> Self {
        StrategyError::OperationError(OperationErrorKind::InvalidParameters {
            operation: "Position".to_string(),
            reason: err.to_string(),
        })
    }
}

impl From<Box<dyn Error>> for StrategyError {
    fn from(err: Box<dyn Error>) -> Self {
        StrategyError::StdError {
            reason: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests_from_str {
    use super::*;
    use crate::error::ProbabilityError;

    #[test]
    fn test_strategy_to_probability_error_conversion() {
        let strategy_error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        let probability_error = ProbabilityError::from(strategy_error);

        assert!(probability_error.to_string().contains("max_profit"));
        assert!(probability_error.to_string().contains("TestStrategy"));
    }

    #[test]
    fn test_profit_loss_error_conversion() {
        let strategy_error = StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
            reason: "Test error".to_string(),
        });
        let probability_error = ProbabilityError::from(strategy_error);

        assert!(probability_error.to_string().contains("Test error"));
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_error_creation() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        assert!(matches!(
            error,
            StrategyError::OperationError(OperationErrorKind::NotSupported { .. })
        ));
    }

    #[test]
    fn test_error_messages() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        let error_string = error.to_string();
        assert!(error_string.contains("max_profit"));
        assert!(error_string.contains("TestStrategy"));
    }
}

#[cfg(test)]
mod tests_display {
    use super::*;

    #[test]
    fn test_price_error_display() {
        let error = StrategyError::PriceError(PriceErrorKind::InvalidUnderlyingPrice {
            reason: "Price cannot be negative".to_string(),
        });
        assert!(error.to_string().contains("Price cannot be negative"));
    }

    #[test]
    fn test_break_even_error_display() {
        let error = StrategyError::BreakEvenError(BreakEvenErrorKind::CalculationError {
            reason: "Invalid input parameters".to_string(),
        });
        assert!(error.to_string().contains("Invalid input parameters"));
    }

    #[test]
    fn test_profit_loss_error_display() {
        let error = StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
            reason: "Cannot calculate maximum profit".to_string(),
        });
        assert!(error
            .to_string()
            .contains("Cannot calculate maximum profit"));
    }

    #[test]
    fn test_operation_error_display() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        assert!(error.to_string().contains("max_profit"));
        assert!(error.to_string().contains("TestStrategy"));
    }
}
