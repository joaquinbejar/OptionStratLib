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

use crate::error::strategies::{BreakEvenErrorKind, ProfitLossErrorKind};
use crate::error::{OperationErrorKind, StrategyError};
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

    NoPositions(String),
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
            ProbabilityError::NoPositions(msg) => write!(f, "No positions: {}", msg),
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
                    reason: strategy_type,
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
            StrategyError::NotImplemented => {
                ProbabilityError::StdError("Strategy not implemented".to_string())
            }
        }
    }
}
impl From<OperationErrorKind> for ProbabilityError {
    fn from(error: OperationErrorKind) -> Self {
        match error {
            OperationErrorKind::InvalidParameters { operation, reason } => {
                ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::ExpectedValueError {
                        reason: format!(
                            "Invalid parameters for operation '{}': {}",
                            operation, reason
                        ),
                    },
                )
            }
            OperationErrorKind::NotSupported { operation, reason } => {
                ProbabilityError::CalculationError(
                    ProbabilityCalculationErrorKind::ExpectedValueError {
                        reason: format!("Operation '{}' not supported: {}", operation, reason),
                    },
                )
            }
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_formatting() {
        let error = ProbabilityError::invalid_probability(1.2, "Probability cannot exceed 1.0");
        let error_string = error.to_string();
        assert!(error_string.contains("Invalid probability"));
        assert!(error_string.contains("1.2"));
        assert!(error_string.contains("Probability cannot exceed 1.0"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_expiration_error_display() {
        let error = ProbabilityError::ExpirationError(ExpirationErrorKind::InvalidExpiration {
            reason: "Cannot be in the past".to_string(),
        });
        assert!(error.to_string().contains("Cannot be in the past"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_price_error_display() {
        let error = ProbabilityError::PriceError(PriceErrorKind::InvalidUnderlyingPrice {
            price: -10.0,
            reason: "Price cannot be negative".to_string(),
        });
        assert!(error.to_string().contains("Price cannot be negative"));
        assert!(error.to_string().contains("-10"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

#[cfg(test)]
mod tests_extended {
    use super::*;
    use crate::error::strategies;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_formatting() {
        let error = ProbabilityError::invalid_probability(1.2, "Probability cannot exceed 1.0");
        let error_string = error.to_string();
        assert!(error_string.contains("Invalid probability"));
        assert!(error_string.contains("1.2"));
        assert!(error_string.contains("Probability cannot exceed 1.0"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_loss_range_error_display() {
        let error = ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidProfitRange {
            range: "100-200".to_string(),
            reason: "Invalid range".to_string(),
        });
        assert!(error.to_string().contains("100-200"));
        assert!(error.to_string().contains("Invalid range"));

        let error =
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found".to_string(),
            });
        assert!(error.to_string().contains("No break-even points found"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculation_error_display() {
        let error = ProbabilityError::CalculationError(
            ProbabilityCalculationErrorKind::VolatilityAdjustmentError {
                reason: "Invalid volatility adjustment".to_string(),
            },
        );
        assert!(error.to_string().contains("Invalid volatility adjustment"));

        let error =
            ProbabilityError::CalculationError(ProbabilityCalculationErrorKind::TrendError {
                reason: "Invalid trend".to_string(),
            });
        assert!(error.to_string().contains("Invalid trend"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_expiration_error() {
        let error = ProbabilityError::ExpirationError(ExpirationErrorKind::InvalidRiskFreeRate {
            rate: Some(0.05),
            reason: "Rate out of bounds".to_string(),
        });
        assert!(error.to_string().contains("0.05"));
        assert!(error.to_string().contains("Rate out of bounds"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_error_conversion() {
        let strategy_error = StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
            reason: "Invalid max profit".to_string(),
        });
        let prob_error: ProbabilityError = strategy_error.into();
        assert!(matches!(
            prob_error,
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::ExpectedValueError { .. }
            )
        ));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_break_even_error_conversion() {
        let strategy_error = StrategyError::BreakEvenError(BreakEvenErrorKind::NoBreakEvenPoints);
        let prob_error: ProbabilityError = strategy_error.into();
        assert!(matches!(
            prob_error,
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::ExpectedValueError { .. }
            )
        ));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_operation_error_conversion() {
        let strategy_error = StrategyError::OperationError(OperationErrorKind::NotSupported {
            operation: "test".to_string(),
            reason: "TestStrategy".to_string(),
        });
        let prob_error: ProbabilityError = strategy_error.into();
        assert!(matches!(
            prob_error,
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::ExpectedValueError { .. }
            )
        ));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_box_dyn_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let boxed_error: Box<dyn Error> = Box::new(io_error);
        let prob_error = ProbabilityError::from(boxed_error);
        assert!(matches!(prob_error, ProbabilityError::StdError(..)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_no_positions_error() {
        let error = ProbabilityError::NoPositions("No positions found".to_string());
        assert!(error.to_string().contains("No positions found"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_result_type() {
        let success: ProbabilityResult<f64> = Ok(0.5);
        let failure: ProbabilityResult<f64> =
            Err(ProbabilityError::invalid_probability(1.5, "Value too high"));

        assert!(success.is_ok());
        assert!(failure.is_err());
    }

    #[test]
    fn test_probability_error_std_error() {
        let error = ProbabilityError::StdError("Calculation failed".to_string());
        assert_eq!(format!("{}", error), "Error: Calculation failed");
    }

    #[test]
    fn test_price_error_invalid_price_range() {
        let error = PriceErrorKind::InvalidPriceRange {
            range: "0-100".to_string(),
            reason: "Negative values are not allowed".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid price range 0-100: Negative values are not allowed"
        );
    }

    #[test]
    fn test_profit_loss_range_error_invalid_loss_range() {
        let error = ProfitLossRangeErrorKind::InvalidLossRange {
            range: "-50-0".to_string(),
            reason: "Range must be positive".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid loss range -50-0: Range must be positive"
        );
    }

    #[test]
    fn test_profit_loss_error_max_loss_error() {
        let error = ProfitLossErrorKind::MaxLossError {
            reason: "Maximum loss exceeded".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Maximum loss calculation error: Maximum loss exceeded"
        );
    }

    #[test]
    fn test_strategy_error_price_error_invalid_price_range() {
        let error = StrategyError::PriceError(strategies::PriceErrorKind::InvalidPriceRange {
            start: 0.0,
            end: 100.0,
            reason: "Out of bounds".to_string(),
        });
        assert!(matches!(error, StrategyError::PriceError(_)));
    }

    #[test]
    fn test_break_even_error_calculation_error() {
        let error = StrategyError::BreakEvenError(BreakEvenErrorKind::CalculationError {
            reason: "Failed to calculate break-even point".to_string(),
        });
        let converted_error: ProbabilityError = error.into();
        assert_eq!(
            format!("{}", converted_error),
            "Calculation error: Expected value error: Failed to calculate break-even point"
        );
    }

    #[test]
    fn test_operation_error_invalid_parameters() {
        let error = OperationErrorKind::InvalidParameters {
            operation: "Calculate P/L".to_string(),
            reason: "Invalid input values".to_string(),
        };
        let converted_error: ProbabilityError = error.into();
        assert_eq!(
            format!("{}", converted_error),
            "Calculation error: Expected value error: Invalid parameters for operation 'Calculate P/L': Invalid input values"
        );
    }

    #[test]
    fn test_strategy_error_std_error() {
        let error = StrategyError::StdError {
            reason: "General strategy failure".to_string(),
        };

        let converted_error: ProbabilityError = error.into();

        assert_eq!(
            format!("{}", converted_error),
            "Error: General strategy failure"
        );
    }

    #[test]
    fn test_invalid_profit_range_constructor() {
        let error = ProbabilityError::invalid_profit_range("0-100", "Range mismatch");
        assert_eq!(
            format!("{}", error),
            "Range error: Invalid profit range 0-100: Range mismatch"
        );
    }

    #[test]
    fn test_invalid_expiration_constructor() {
        let error = ProbabilityError::invalid_expiration("Expiration date invalid");
        assert_eq!(
            format!("{}", error),
            "Expiration error: Invalid expiration: Expiration date invalid"
        );
    }

    #[test]
    fn test_profit_loss_error_max_loss_error_bis() {
        let error = ProfitLossErrorKind::MaxLossError {
            reason: "Exceeded allowed loss".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Maximum loss calculation error: Exceeded allowed loss"
        );
    }

    #[test]
    fn test_profit_loss_error_profit_range_error() {
        let error = ProfitLossErrorKind::ProfitRangeError {
            reason: "Profit range mismatch".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Profit range calculation error: Profit range mismatch"
        );
    }

    #[test]
    fn test_strategy_error_price_error_invalid_underlying_price() {
        let error = StrategyError::PriceError(
            crate::error::strategies::PriceErrorKind::InvalidUnderlyingPrice {
                reason: "Underlying price is negative".to_string(),
            },
        );
        let converted_error: ProbabilityError = ProbabilityError::from(error);
        assert_eq!(
            format!("{}", converted_error),
            "Calculation error: Expected value error: Underlying price is negative"
        );
    }

    #[test]
    fn test_strategy_error_price_error_invalid_price_range_bis() {
        let error = StrategyError::PriceError(
            crate::error::strategies::PriceErrorKind::InvalidPriceRange {
                start: 0.0,
                end: 50.0,
                reason: "Start price is greater than end price".to_string(),
            },
        );
        let converted_error: ProbabilityError = ProbabilityError::from(error);
        assert_eq!(
            format!("{}", converted_error),
            "Calculation error: Expected value error: Start price is greater than end price"
        );
    }

    #[test]
    fn test_operation_error_invalid_parameters_bis() {
        let error = OperationErrorKind::InvalidParameters {
            operation: "Calculate P/L".to_string(),
            reason: "Invalid input values".to_string(),
        };
        let converted_error: ProbabilityError =
            ProbabilityError::from(format!("Invalid parameters for operation {}", error));
        assert_eq!(
            format!("{}", converted_error),
            "Calculation error: Expected value error: Invalid parameters for operation Invalid parameters for operation 'Calculate P/L': Invalid input values"
        );
    }

    #[test]
    fn test_operation_error_not_supported() {
        let error = OperationErrorKind::NotSupported {
            operation: "Hedging".to_string(),
            reason: "Operation not implemented".to_string(),
        };
        let converted_error = ProbabilityError::CalculationError(
            ProbabilityCalculationErrorKind::ExpectedValueError {
                reason: format!("Operation {}", error),
            },
        );
        assert_eq!(
            format!("{}", converted_error),
            "Calculation error: Expected value error: Operation Operation 'Hedging' is not supported for strategy 'Operation not implemented'"
        );
    }
}
