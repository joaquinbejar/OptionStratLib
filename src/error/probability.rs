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
///
/// This enum serves as the top-level error type for the probability analysis system,
/// encapsulating various categories of errors that might occur during financial calculations
/// related to probabilities, prices, ranges, and expirations.
///
/// # Variants
///
/// * `CalculationError` - Errors that occur during probability calculations themselves,
///   such as invalid probability values, expected value errors, or problems with
///   volatility adjustments.
///
/// * `RangeError` - Errors related to profit and loss ranges, such as invalid profit
///   ranges, loss ranges, or break-even point calculations.
///
/// * `ExpirationError` - Errors related to expiration dates and risk-free rates in
///   options calculations.
///
/// * `PriceError` - Errors related to price parameters, such as invalid underlying
///   prices or invalid price ranges.
///
/// * `StdError` - Standard errors from external systems or libraries, wrapped as strings.
///
/// * `NoPositions` - Error indicating that no positions were available for analysis.
///
/// # Usage
///
/// This error type is typically returned from functions that perform financial probability
/// analysis, risk assessment, or option pricing. It provides a structured way to handle
/// the various error conditions that might arise during these calculations.
#[derive(Debug)]
pub enum ProbabilityError {
    /// Errors related to probability calculations
    ///
    /// Wraps a `ProbabilityCalculationErrorKind` that provides specific details
    /// about what went wrong during probability calculations.
    CalculationError(ProbabilityCalculationErrorKind),

    /// Errors related to profit/loss ranges
    ///
    /// Wraps a `ProfitLossRangeErrorKind` that provides specific details
    /// about what went wrong with profit or loss range calculations.
    RangeError(ProfitLossRangeErrorKind),

    /// Errors related to expiration dates
    ///
    /// Wraps an `ExpirationErrorKind` that provides specific details
    /// about what went wrong with expiration dates or risk-free rates.
    ExpirationError(ExpirationErrorKind),

    /// Errors related to price parameters
    ///
    /// Wraps a `PriceErrorKind` that provides specific details
    /// about what went wrong with price-related calculations.
    PriceError(PriceErrorKind),

    /// Standard error from an external system or library
    ///
    /// Contains a string description of an error from a standard library
    /// or external dependency.
    StdError(String),

    /// Error indicating no positions are available for analysis
    ///
    /// Contains a string description explaining why positions are missing
    /// or why they cannot be analyzed.
    NoPositions(String),
}

/// Error types that can occur during financial probability calculations.
///
/// This enum represents specific error categories that may arise when performing
/// probability-related calculations for financial models, risk assessments, and
/// statistical analysis of market scenarios. It provides structured error information
/// with detailed context about what caused the calculation to fail.
///
/// # Variants
///
/// * `InvalidProbability` - Represents errors where a probability value falls outside
///   the valid range [0,1] or otherwise violates mathematical constraints of probability theory.
///
/// * `ExpectedValueError` - Represents errors in the calculation of expected values,
///   such as invalid inputs, numerical overflow/underflow, or unsupported distributions.
///
/// * `VolatilityAdjustmentError` - Represents errors that occur when attempting to
///   adjust calculations for volatility, including invalid volatility inputs or
///   mathematical errors in the adjustment process.
///
/// * `TrendError` - Represents errors related to price trend calculations, such as
///   insufficient historical data, invalid trend parameters, or statistical anomalies.
///
/// # Usage
///
/// This error type is typically used in financial modeling components that deal with
/// probabilistic outcomes, such as Monte Carlo simulations, risk models, and
/// probability-based trading strategies.
#[derive(Debug)]
pub enum ProbabilityCalculationErrorKind {
    /// Error indicating an invalid probability value
    ///
    /// This variant is used when a probability calculation produces a result that
    /// violates fundamental probability constraints (e.g., not in range [0,1])
    /// or other mathematical properties required for valid statistical analysis.
    InvalidProbability {
        /// The problematic probability value that caused the error
        value: f64,
        /// Detailed explanation of why the probability is invalid
        reason: String,
    },

    /// Error in the calculation of expected values
    ///
    /// This variant is used when expected value calculations fail due to
    /// mathematical constraints, invalid inputs, or problems with the
    /// underlying probability distribution.
    ExpectedValueError {
        /// Detailed explanation of the error in expected value calculation
        reason: String,
    },

    /// Error when adjusting calculations for volatility
    ///
    /// This variant is used when volatility-related adjustments to probability
    /// calculations encounter problems, such as extreme volatility values or
    /// mathematical errors in the adjustment algorithms.
    VolatilityAdjustmentError {
        /// Detailed explanation of the volatility adjustment error
        reason: String,
    },

    /// Error in price trend calculations or analysis
    ///
    /// This variant is used when trend-related probability calculations fail,
    /// such as issues with trend detection, invalid trend parameters, or
    /// insufficient data for meaningful trend analysis.
    TrendError {
        /// Detailed explanation of the trend calculation error
        reason: String,
    },
}

/// Enum representing errors that occur during profit and loss range calculations in options strategies.
///
/// This enum captures three main categories of errors that can occur when calculating, validating,
/// or processing profit and loss ranges and break-even points in financial calculations. It provides
/// structured error information with detailed descriptions of why the ranges are invalid.
///
/// # Variants
///
/// * `InvalidProfitRange` - Represents errors related to profit range calculations,
///   such as inconsistent boundaries, mathematically impossible ranges, or other
///   logical errors in profit projections.
///
/// * `InvalidLossRange` - Represents errors related to loss range calculations,
///   including boundary issues, logical inconsistencies, or other problems that
///   prevent accurate loss estimation.
///
/// * `InvalidBreakEvenPoints` - Represents errors in break-even point calculations,
///   which can include missing break-even points, multiple unexpected points, or
///   other calculation anomalies.
///
/// # Usage
///
/// This error type is typically used in financial strategy analysis, particularly in
/// options trading calculations where understanding the ranges of potential profit and loss
/// is critical for risk assessment and decision making.
#[derive(Debug)]
pub enum ProfitLossRangeErrorKind {
    /// Error indicating an invalid or problematic profit range calculation
    ///
    /// This variant is used when the calculated profit range contains errors,
    /// inconsistencies, or values that don't align with the underlying strategy.
    InvalidProfitRange {
        /// String representation of the problematic range
        range: String,
        /// Detailed explanation of why the profit range is invalid
        reason: String,
    },

    /// Error indicating an invalid or problematic loss range calculation
    ///
    /// This variant is used when the calculated loss range contains errors,
    /// inconsistencies, or values that don't make mathematical sense.
    InvalidLossRange {
        /// String representation of the problematic range
        range: String,
        /// Detailed explanation of why the loss range is invalid
        reason: String,
    },

    /// Error indicating problems with break-even point calculations
    ///
    /// This variant is used when break-even points couldn't be properly calculated,
    /// are missing, or contain unexpected values.
    InvalidBreakEvenPoints {
        /// Detailed explanation of the issue with break-even calculations
        reason: String,
    },
}

/// Enum representing errors related to expiration dates and interest rates in options calculations.
///
/// This enum captures two main categories of errors that can occur when validating or
/// processing expiration dates and risk-free rates for financial calculations. It provides
/// structured error information that includes detailed reasons for the error occurrence.
///
/// # Variants
///
/// * `InvalidExpiration` - Represents errors related to option expiration dates,
///   such as dates in the past, invalid formats, or dates that don't align with
///   market standards.
///
/// * `InvalidRiskFreeRate` - Represents errors related to risk-free rate values,
///   such as negative rates, unreasonably high rates, or other invalid inputs
///   that would cause calculation issues.
///
/// # Usage
///
/// This error type is typically used in option pricing models, risk assessment,
/// and other financial calculations where expiration dates and risk-free rates
/// are critical input parameters.
#[derive(Debug)]
pub enum ExpirationErrorKind {
    /// Error indicating an invalid or problematic expiration date
    ///
    /// This variant is used when the provided expiration date doesn't meet
    /// the required criteria for option calculations.
    InvalidExpiration {
        /// Detailed explanation of why the expiration date is invalid
        reason: String,
    },

    /// Error indicating an invalid or problematic risk-free rate
    ///
    /// This variant is used when the provided risk-free rate is outside
    /// acceptable bounds or otherwise unsuitable for financial calculations.
    InvalidRiskFreeRate {
        /// The problematic rate value that caused the error, if available
        rate: Option<f64>,
        /// Detailed explanation of why the risk-free rate is invalid
        reason: String,
    },
}

/// Enum that represents various errors that can occur during price calculations and validations.
///
/// This enum provides specific error variants for different types of pricing issues that may
/// arise in financial calculations, particularly in options pricing contexts. Each variant
/// contains detailed information about the error condition to aid in debugging and error handling.
///
/// ## Error Types
///
/// * `InvalidUnderlyingPrice` - Errors related to the underlying asset price
/// * `InvalidPriceRange` - Errors related to price range validations
///
#[derive(Debug)]
pub enum PriceErrorKind {
    /// Error indicating that the underlying asset price is invalid.
    ///
    /// This error occurs when the price of the underlying asset does not meet
    /// required validation criteria (e.g., non-negative, within expected bounds).
    ///
    InvalidUnderlyingPrice {
        /// * `price` - The invalid price value that triggered the error       
        price: f64,
        /// * `reason` - A detailed explanation of why the price is considered invalid
        reason: String,
    },
    /// Error indicating that a price range specification is invalid.
    ///
    /// This error occurs when a specified price range is inconsistent, malformed,
    /// or does not meet required validation criteria for financial calculations.
    ///
    InvalidPriceRange {
        /// * `range` - String representation of the invalid price range      
        range: String,
        /// * `reason` - A detailed explanation of why the price range is considered invalid
        reason: String,
    },
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
