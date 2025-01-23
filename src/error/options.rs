//! # Options Error Module
//!
//! This module provides error handling for financial options operations.
//! It defines a custom error type `OptionsError` that encapsulates various error conditions
//! that can occur during options trading and calculations.
//!
//! ## Error Types
//!
//! - `ValidationError`: For errors in option parameter validation
//! - `PricingError`: For errors during price calculation
//! - `GreeksCalculationError`: For errors when calculating Greeks
//! - `TimeError`: For errors related to time calculations
//! - `PayoffError`: For errors in payoff calculations
//! - `UpdateError`: For errors during option data updates
//!
//! ## Usage
//!
//! ```rust
//! use optionstratlib::error::OptionsError;
//!
//! // Create a validation error
//! let error = OptionsError::validation_error("strike_price", "must be positive");
//!
//! // Create a pricing error
//! let error = OptionsError::pricing_error("black_scholes", "invalid volatility");
//!
//! // Use with Result
//! type Result<T> = std::result::Result<T, OptionsError>;
//! ```
//!
//! ## Features
//!
//! - Implements standard Error trait
//! - Provides detailed error context
//! - Supports conversion from standard error types
//! - Includes helper methods for error creation
//!
//! ## Error Conversion
//!
//! The module supports conversion from:
//! - `String`
//! - `&str`
//! - `Box<dyn Error>`
//!
//! And conversion to:
//! - `Box<dyn Error>`
//!
//! ## Examples
//!
//! ```rust
//! use optionstratlib::error::{OptionsError, OptionsResult};
//!
//! fn validate_strike_price(price: f64) -> OptionsResult<f64> {
//!     if price <= 0.0 {
//!         return Err(OptionsError::validation_error(
//!             "strike_price",
//!             "must be positive"
//!         ));
//!     }
//!     Ok(price)
//! }
//! ```

use std::error::Error;
use std::fmt;

/// Custom errors that can occur during Options operations
#[derive(Debug)]
pub enum OptionsError {
    /// Error when validating option parameters
    ValidationError { field: String, reason: String },
    /// Error during price calculation
    PricingError { method: String, reason: String },
    /// Error when calculating greeks
    GreeksCalculationError { greek: String, reason: String },
    /// Error when dealing with time calculations
    TimeError { operation: String, reason: String },
    /// Error when performing payoff calculations
    PayoffError { reason: String },
    /// Error during option data updates
    UpdateError { field: String, reason: String },

    /// Error when performing other operations
    OtherError { reason: String },
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionsError::ValidationError { field, reason } => {
                write!(f, "Validation error for field '{}': {}", field, reason)
            }
            OptionsError::PricingError { method, reason } => {
                write!(f, "Pricing error using method '{}': {}", method, reason)
            }
            OptionsError::GreeksCalculationError { greek, reason } => {
                write!(f, "Error calculating greek '{}': {}", greek, reason)
            }
            OptionsError::TimeError { operation, reason } => {
                write!(f, "Time calculation error in '{}': {}", operation, reason)
            }
            OptionsError::PayoffError { reason } => {
                write!(f, "Payoff calculation error: {}", reason)
            }
            OptionsError::UpdateError { field, reason } => {
                write!(f, "Update error for field '{}': {}", field, reason)
            }
            OptionsError::OtherError { reason } => {
                write!(f, "Other error: {}", reason)
            }
        }
    }
}

impl Error for OptionsError {}

// Convenient type alias for Results with OptionsError
pub type OptionsResult<T> = Result<T, OptionsError>;

// Helper methods for creating common options errors
impl OptionsError {
    pub fn validation_error(field: &str, reason: &str) -> Self {
        OptionsError::ValidationError {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn pricing_error(method: &str, reason: &str) -> Self {
        OptionsError::PricingError {
            method: method.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn greeks_error(greek: &str, reason: &str) -> Self {
        OptionsError::GreeksCalculationError {
            greek: greek.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn time_error(operation: &str, reason: &str) -> Self {
        OptionsError::TimeError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn payoff_error(reason: &str) -> Self {
        OptionsError::PayoffError {
            reason: reason.to_string(),
        }
    }

    pub fn update_error(field: &str, reason: &str) -> Self {
        OptionsError::UpdateError {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }
}

impl From<Box<dyn Error>> for OptionsError {
    fn from(err: Box<dyn Error>) -> Self {
        OptionsError::ValidationError {
            field: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<&str> for OptionsError {
    fn from(err: &str) -> Self {
        OptionsError::ValidationError {
            field: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<String> for OptionsError {
    fn from(err: String) -> Self {
        OptionsError::ValidationError {
            field: "unknown".to_string(),
            reason: err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validation_error_creation() {
        let error = OptionsError::validation_error("price", "must be positive");
        match error {
            OptionsError::ValidationError { field, reason } => {
                assert_eq!(field, "price");
                assert_eq!(reason, "must be positive");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_pricing_error_creation() {
        let error = OptionsError::pricing_error("black_scholes", "invalid parameters");
        match error {
            OptionsError::PricingError { method, reason } => {
                assert_eq!(method, "black_scholes");
                assert_eq!(reason, "invalid parameters");
            }
            _ => panic!("Expected PricingError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_error_creation() {
        let error = OptionsError::greeks_error("delta", "calculation failed");
        match error {
            OptionsError::GreeksCalculationError { greek, reason } => {
                assert_eq!(greek, "delta");
                assert_eq!(reason, "calculation failed");
            }
            _ => panic!("Expected GreeksCalculationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_time_error_creation() {
        let error = OptionsError::time_error("expiry", "invalid date");
        match error {
            OptionsError::TimeError { operation, reason } => {
                assert_eq!(operation, "expiry");
                assert_eq!(reason, "invalid date");
            }
            _ => panic!("Expected TimeError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_error_creation() {
        let error = OptionsError::payoff_error("invalid strike price");
        match error {
            OptionsError::PayoffError { reason } => {
                assert_eq!(reason, "invalid strike price");
            }
            _ => panic!("Expected PayoffError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_update_error_creation() {
        let error = OptionsError::update_error("volatility", "out of bounds");
        match error {
            OptionsError::UpdateError { field, reason } => {
                assert_eq!(field, "volatility");
                assert_eq!(reason, "out of bounds");
            }
            _ => panic!("Expected UpdateError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_display() {
        let error = OptionsError::validation_error("price", "must be positive");
        assert_eq!(
            error.to_string(),
            "Validation error for field 'price': must be positive"
        );

        let error = OptionsError::pricing_error("black_scholes", "invalid parameters");
        assert_eq!(
            error.to_string(),
            "Pricing error using method 'black_scholes': invalid parameters"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_str_conversion() {
        let error: OptionsError = "test error".into();
        match error {
            OptionsError::ValidationError { field, reason } => {
                assert_eq!(field, "unknown");
                assert_eq!(reason, "test error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_conversion() {
        let error: OptionsError = String::from("test error").into();
        match error {
            OptionsError::ValidationError { field, reason } => {
                assert_eq!(field, "unknown");
                assert_eq!(reason, "test error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_box_dyn_error_conversion() {
        struct TestError(String);

        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl fmt::Debug for TestError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "TestError({})", self.0)
            }
        }

        impl Error for TestError {}

        let original_error: Box<dyn Error> = Box::new(TestError("test error".to_string()));
        let error: OptionsError = original_error.into();

        match error {
            OptionsError::ValidationError { field, reason } => {
                assert_eq!(field, "unknown");
                assert_eq!(reason, "test error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_to_box_dyn_error_conversion() {
        let error = OptionsError::validation_error("price", "must be positive");
        let boxed: Box<dyn Error> = error.into();
        assert_eq!(
            boxed.to_string(),
            "Validation error for field 'price': must be positive"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OptionsError>();
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_options_result_type() {
        let success: OptionsResult<i32> = Ok(42);
        let failure: OptionsResult<i32> = Err(OptionsError::validation_error("test", "error"));

        assert!(success.is_ok());
        assert!(failure.is_err());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_chaining() {
        let error1 = OptionsError::validation_error("strike", "invalid value");
        let error2: OptionsError = error1.to_string().into();

        match error2 {
            OptionsError::ValidationError { field, reason } => {
                assert!(reason.contains("invalid value"));
                assert_eq!(field, "unknown");
            }
            _ => panic!("Expected ValidationError"),
        }

        // Segunda forma: usando From<&str>
        let error3 = OptionsError::validation_error("price", "must be positive");
        let error4: OptionsError = error3.to_string().as_str().into();

        match error4 {
            OptionsError::ValidationError { field, reason } => {
                assert!(reason.contains("must be positive"));
                assert_eq!(field, "unknown");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_multiple_conversions() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let boxed: Box<dyn Error> = Box::new(io_error);
        let error: OptionsError = boxed.into();

        assert!(matches!(error, OptionsError::ValidationError { .. }));

        match error {
            OptionsError::ValidationError { field: _, reason } => {
                assert!(reason.contains("test error"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_complex_error_scenario() {
        fn nested_function() -> OptionsResult<()> {
            Err(OptionsError::validation_error("nested", "inner error"))
        }

        fn outer_function() -> OptionsResult<()> {
            nested_function().map_err(|e| OptionsError::time_error("outer", &e.to_string()))
        }

        let result = outer_function();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, OptionsError::TimeError { .. }));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validation_combinations() {
        let errors = vec![
            OptionsError::validation_error("price", "negative value"),
            OptionsError::validation_error("strike", "too high"),
            OptionsError::validation_error("expiry", "past date"),
        ];

        for error in errors {
            match error {
                OptionsError::ValidationError { field, reason } => {
                    assert!(!field.is_empty());
                    assert!(!reason.is_empty());
                }
                _ => panic!("Expected ValidationError"),
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_pricing_error_variants() {
        let methods = ["black_scholes", "binomial", "monte_carlo"];
        let reasons = ["invalid vol", "negative rate", "bad params"];

        for (method, reason) in methods.iter().zip(reasons.iter()) {
            let error = OptionsError::pricing_error(method, reason);
            let error_str = error.to_string();
            assert!(error_str.contains(method));
            assert!(error_str.contains(reason));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_error_conversion_preservation() {
        let original = "preserve this message";
        let error1: OptionsError = original.into();
        let error2: OptionsError = error1.to_string().into();

        assert!(error2.to_string().contains(original));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_option_result_operations() {
        let success: OptionsResult<i32> = Ok(42);
        let failure: OptionsResult<i32> = Err(OptionsError::validation_error("test", "error"));

        let mapped_success = success.map(|x| x * 2);
        let mapped_failure = failure.map(|x| x * 2);

        assert_eq!(mapped_success.unwrap(), 84);
        assert!(mapped_failure.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_nested_error_handling() {
        fn process_value(value: i32) -> OptionsResult<i32> {
            if value < 0 {
                Err(OptionsError::validation_error("value", "must be positive"))
            } else {
                Ok(value)
            }
        }

        let results: Vec<OptionsResult<i32>> =
            vec![-1, 0, 1].into_iter().map(process_value).collect();

        assert_eq!(results.len(), 3);
        assert!(results[0].is_err());
        assert!(results[1].is_ok());
        assert!(results[2].is_ok());
    }

    #[test]
    fn test_options_error_greeks_calculation_error() {
        let error = OptionsError::GreeksCalculationError {
            greek: "Delta".to_string(),
            reason: "Division by zero".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Error calculating greek 'Delta': Division by zero"
        );
    }

    #[test]
    fn test_options_error_time_error() {
        let error = OptionsError::TimeError {
            operation: "Option maturity".to_string(),
            reason: "Negative time value".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Time calculation error in 'Option maturity': Negative time value"
        );
    }

    #[test]
    fn test_options_error_payoff_error() {
        let error = OptionsError::PayoffError {
            reason: "Payoff cannot be negative".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Payoff calculation error: Payoff cannot be negative"
        );
    }

    #[test]
    fn test_options_error_update_error() {
        let error = OptionsError::UpdateError {
            field: "Volatility".to_string(),
            reason: "Invalid update value".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Update error for field 'Volatility': Invalid update value"
        );
    }
}
