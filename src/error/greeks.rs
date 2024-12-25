/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 25/12/24
 ******************************************************************************/

//! # Greeks Error Module
//!
//! This module provides error handling for Greek calculations and equations in option pricing.
//! It defines error types for various mathematical calculations and validations used in
//! financial derivatives analysis.
//!
//! ## Error Types
//!
//! ### Greeks Error (`GreeksError`)
//! Main error enum that encompasses:
//! * Calculation errors in Greek values
//! * Input validation errors
//! * Mathematical operation errors
//! * Boundary condition errors
//!
//! ### Mathematical Error (`MathErrorKind`)
//! Handles specific mathematical errors:
//! * Division by zero
//! * Overflow conditions
//! * Invalid domain errors
//! * Convergence failures
//!
//! ### Input Validation Error (`InputErrorKind`)
//! Manages validation of input parameters:
//! * Invalid volatility values
//! * Invalid time values
//! * Invalid price values
//! * Invalid rate values

use std::error::Error;
use std::fmt;
use crate::error::decimal::DecimalError;

#[derive(Debug)]
pub enum GreeksError {
    /// Errors related to mathematical calculations
    MathError(MathErrorKind),
    /// Errors related to input validation
    InputError(InputErrorKind),
    /// Errors related to Greek calculations
    CalculationError(CalculationErrorKind),
}

#[derive(Debug)]
pub enum MathErrorKind {
    /// Division by zero error
    DivisionByZero,
    /// Numerical overflow
    Overflow,
    /// Invalid mathematical domain
    InvalidDomain { value: f64, reason: String },
    /// Convergence failure
    ConvergenceFailure { iterations: usize, tolerance: f64 },
}

#[derive(Debug)]
pub enum InputErrorKind {
    /// Invalid volatility value
    InvalidVolatility {
        value: f64,
        reason: String,
    },
    /// Invalid time value
    InvalidTime {
        value: f64,
        reason: String,
    },
    /// Invalid price value
    InvalidPrice {
        value: f64,
        reason: String,
    },
    /// Invalid rate value
    InvalidRate {
        value: f64,
        reason: String,
    },
    InvalidStrike {
        value: f64,
        reason: String,
    },
}

#[derive(Debug)]
pub enum CalculationErrorKind {
    /// Error in delta calculation
    DeltaError { reason: String },
    /// Error in gamma calculation
    GammaError { reason: String },
    /// Error in theta calculation
    ThetaError { reason: String },
    /// Error in vega calculation
    VegaError { reason: String },
    /// Error in rho calculation
    RhoError { reason: String },
    DecimalError { error: DecimalError },
}

impl fmt::Display for GreeksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GreeksError::MathError(err) => write!(f, "Mathematical error: {}", err),
            GreeksError::InputError(err) => write!(f, "Input validation error: {}", err),
            GreeksError::CalculationError(err) => write!(f, "Greek calculation error: {}", err),
        }
    }
}

impl fmt::Display for MathErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathErrorKind::DivisionByZero => write!(f, "Division by zero"),
            MathErrorKind::Overflow => write!(f, "Numerical overflow"),
            MathErrorKind::InvalidDomain { value, reason } => {
                write!(f, "Invalid domain value {}: {}", value, reason)
            }
            MathErrorKind::ConvergenceFailure { iterations, tolerance } => {
                write!(
                    f,
                    "Failed to converge after {} iterations (tolerance: {})",
                    iterations, tolerance
                )
            }
        }
    }
}

impl fmt::Display for InputErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputErrorKind::InvalidVolatility { value, reason } => {
                write!(f, "Invalid volatility {}: {}", value, reason)
            }
            InputErrorKind::InvalidTime { value, reason } => {
                write!(f, "Invalid time value {}: {}", value, reason)
            }
            InputErrorKind::InvalidPrice { value, reason } => {
                write!(f, "Invalid price {}: {}", value, reason)
            }
            InputErrorKind::InvalidRate { value, reason } => {
                write!(f, "Invalid rate {}: {}", value, reason)
            }
            InputErrorKind::InvalidStrike { value, reason } => {
                write!(f, "Invalid strike price {}: {}", value, reason)
            }
        }
    }
}

impl fmt::Display for CalculationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculationErrorKind::DeltaError { reason } => write!(f, "Delta calculation error: {}", reason),
            CalculationErrorKind::GammaError { reason } => write!(f, "Gamma calculation error: {}", reason),
            CalculationErrorKind::ThetaError { reason } => write!(f, "Theta calculation error: {}", reason),
            CalculationErrorKind::VegaError { reason } => write!(f, "Vega calculation error: {}", reason),
            CalculationErrorKind::RhoError { reason } => write!(f, "Rho calculation error: {}", reason),
            CalculationErrorKind::DecimalError { error } => write!(f, "Decimal error: {}", error),
        }
    }
}

impl Error for GreeksError {}

// Type alias for Results
pub type GreeksResult<T> = Result<T, GreeksError>;

// Helper methods for creating common errors
impl GreeksError {
    pub fn invalid_volatility(value: f64, reason: &str) -> Self {
        GreeksError::InputError(InputErrorKind::InvalidVolatility {
            value,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_time(value: f64, reason: &str) -> Self {
        GreeksError::InputError(InputErrorKind::InvalidTime {
            value,
            reason: reason.to_string(),
        })
    }

    pub fn delta_error(reason: &str) -> Self {
        GreeksError::CalculationError(CalculationErrorKind::DeltaError {
            reason: reason.to_string(),
        })
    }

    // Add more helper methods as needed...
}

impl From<DecimalError> for GreeksError {
    fn from(error: DecimalError) -> Self {
        GreeksError::CalculationError(CalculationErrorKind::DecimalError { error })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_volatility_error_creation() {
        let error = GreeksError::invalid_volatility(-0.5, "Volatility cannot be negative");
        match error {
            GreeksError::InputError(InputErrorKind::InvalidVolatility { value, reason }) => {
                assert_eq!(value, -0.5);
                assert_eq!(reason, "Volatility cannot be negative");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_invalid_time_error_creation() {
        let error = GreeksError::invalid_time(-1.0, "Time must be positive");
        match error {
            GreeksError::InputError(InputErrorKind::InvalidTime { value, reason }) => {
                assert_eq!(value, -1.0);
                assert_eq!(reason, "Time must be positive");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_delta_error_creation() {
        let error = GreeksError::delta_error("Failed to calculate delta");
        match error {
            GreeksError::CalculationError(CalculationErrorKind::DeltaError { reason }) => {
                assert_eq!(reason, "Failed to calculate delta");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_math_error_display() {
        let error = GreeksError::MathError(MathErrorKind::DivisionByZero);
        assert_eq!(error.to_string(), "Mathematical error: Division by zero");

        let error = GreeksError::MathError(MathErrorKind::InvalidDomain {
            value: 1.5,
            reason: "Value out of range".to_string(),
        });
        assert_eq!(
            error.to_string(),
            "Mathematical error: Invalid domain value 1.5: Value out of range"
        );
    }

    #[test]
    fn test_input_error_display() {
        let error = GreeksError::InputError(InputErrorKind::InvalidPrice {
            value: -100.0,
            reason: "Price cannot be negative".to_string(),
        });
        assert_eq!(
            error.to_string(),
            "Input validation error: Invalid price -100: Price cannot be negative"
        );

        let error = GreeksError::InputError(InputErrorKind::InvalidRate {
            value: 2.5,
            reason: "Rate must be between 0 and 1".to_string(),
        });
        assert_eq!(
            error.to_string(),
            "Input validation error: Invalid rate 2.5: Rate must be between 0 and 1"
        );
    }

    #[test]
    fn test_calculation_error_display() {
        let error = GreeksError::CalculationError(CalculationErrorKind::GammaError {
            reason: "Invalid input parameters".to_string(),
        });
        assert_eq!(
            error.to_string(),
            "Greek calculation error: Gamma calculation error: Invalid input parameters"
        );

        let error = GreeksError::CalculationError(CalculationErrorKind::VegaError {
            reason: "Calculation overflow".to_string(),
        });
        assert_eq!(
            error.to_string(),
            "Greek calculation error: Vega calculation error: Calculation overflow"
        );
    }

    #[test]
    fn test_convergence_failure_display() {
        let error = GreeksError::MathError(MathErrorKind::ConvergenceFailure {
            iterations: 1000,
            tolerance: 0.0001,
        });
        assert_eq!(
            error.to_string(),
            "Mathematical error: Failed to converge after 1000 iterations (tolerance: 0.0001)"
        );
    }

    #[test]
    fn test_result_type() {
        fn test_function() -> GreeksResult<f64> {
            Err(GreeksError::delta_error("Test error"))
        }

        let result = test_function();
        assert!(result.is_err());
        match result {
            Err(GreeksError::CalculationError(CalculationErrorKind::DeltaError { reason })) => {
                assert_eq!(reason, "Test error");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = GreeksError::delta_error("Test error");
        let _error_trait_object: &dyn Error = &error;
        // If this compiles, it means Error trait is implemented correctly
    }

    #[test]
    fn test_debug_implementation() {
        let error = GreeksError::delta_error("Test error");
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("DeltaError"));
        assert!(debug_string.contains("Test error"));
    }
}