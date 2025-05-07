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
///
/// This enum provides a structured error system for handling various failure scenarios
/// that may arise during option trading operations, calculations, and data management.
/// Each variant represents a specific category of error with contextual information
/// to help with debugging and error handling.
///
/// # Variants
///
/// * `ValidationError` - Errors that occur when validating option parameters
///   such as strike prices, expiration dates, or option styles.
///
/// * `PricingError` - Errors that occur during option price calculation
///   using various pricing models like Black-Scholes, Binomial, etc.
///
/// * `GreeksCalculationError` - Errors that occur when calculating option Greeks
///   (delta, gamma, theta, vega, rho) which measure option price sensitivities.
///
/// * `TimeError` - Errors related to time calculations, such as determining
///   days to expiration, time decay, or handling calendar adjustments.
///
/// * `PayoffError` - Errors that occur when calculating potential payoffs
///   for options at different price points or expiration scenarios.
///
/// * `UpdateError` - Errors that occur when attempting to update option data
///   or parameters in an existing option object.
///
/// * `OtherError` - A catch-all for errors that don't fit into other categories
///   but still need to be represented in the options domain.
///
/// # Usage
///
/// This error type is typically returned in Result objects from functions that
/// perform operations on option contracts, pricing calculations, or option strategy
/// analysis where various error conditions need to be handled.
#[derive(Debug)]
pub enum OptionsError {
    /// Error when validating option parameters
    ///
    /// Used when input parameters for option contracts fail validation.
    ValidationError {
        /// The field name that failed validation
        field: String,
        /// Detailed explanation of the validation failure
        reason: String,
    },

    /// Error during price calculation
    ///
    /// Used when an option pricing algorithm encounters problems.
    PricingError {
        /// The pricing method that failed (e.g., "Black-Scholes", "Binomial")
        method: String,
        /// Detailed explanation of the pricing calculation failure
        reason: String,
    },

    /// Error when calculating greeks
    ///
    /// Used when calculations for option sensitivities (Greeks) fail.
    GreeksCalculationError {
        /// The specific Greek that failed to calculate (delta, gamma, theta, etc.)
        greek: String,
        /// Detailed explanation of the Greek calculation failure
        reason: String,
    },

    /// Error when dealing with time calculations
    ///
    /// Used for failures in time-related calculations like time to expiry.
    TimeError {
        /// The time-related operation that failed
        operation: String,
        /// Detailed explanation of the time calculation failure
        reason: String,
    },

    /// Error when performing payoff calculations
    ///
    /// Used when potential profit/loss calculations for options fail.
    PayoffError {
        /// Detailed explanation of the payoff calculation failure
        reason: String,
    },

    /// Error during option data updates
    ///
    /// Used when attempts to update option parameters or data fail.
    UpdateError {
        /// The field that failed to update
        field: String,
        /// Detailed explanation of the update failure
        reason: String,
    },

    /// Error when performing other operations
    ///
    /// A general-purpose error for cases not covered by other variants.
    OtherError {
        /// Detailed explanation of the error
        reason: String,
    },
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

/// A specialized result type for operations related to Options calculations and processing.
///
/// This type alias simplifies error handling for functions that can fail with various
/// options-specific errors. It uses the [`OptionsError`] enum to provide structured
/// error information about validation failures, pricing issues, Greeks calculations,
/// time-related problems, and other option-specific errors.
///
/// # Type Parameters
///
/// * `T` - The success type that will be returned if the operation succeeds.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::error::{OptionsResult, OptionsError};
///
/// fn calculate_call_price(strike: f64, spot: f64) -> OptionsResult<f64> {
///     if strike <= 0.0 {
///         return Err(OptionsError::ValidationError {
///             field: "strike".to_string(),
///             reason: "Strike price must be positive".to_string()
///         });
///     }
///     
///     // Calculation logic would go here
///     let price = 0.0; // Placeholder
///     Ok(price)
/// }
/// ```
///
/// # Usage Context
///
/// This result type is commonly used throughout the library for:
///
/// * Option pricing calculations
/// * Parameter validation
/// * Greeks calculations
/// * Expiration and time value calculations
/// * Option payoff analysis
///
/// [`OptionsError`]: enum.OptionsError.html
pub type OptionsResult<T> = Result<T, OptionsError>;

/// Helper methods for creating common options errors.
///
/// This implementation provides convenient factory methods for creating different
/// variants of `OptionsError` without having to manually construct the enum variants.
/// Each method corresponds to a specific error type and properly formats the error fields.
///
/// # Methods
///
/// * `validation_error` - Creates an error for parameter validation failures
/// * `pricing_error` - Creates an error for pricing calculation issues
/// * `greeks_error` - Creates an error for problems with Greeks calculations
/// * `time_error` - Creates an error for time-related calculations
/// * `payoff_error` - Creates an error for payoff calculation problems
/// * `update_error` - Creates an error for option data update issues
///
/// # Examples
///
/// ```
/// use optionstratlib::error::OptionsError;
/// let error = OptionsError::validation_error("strike_price", "must be positive");
///
/// // Create a pricing error
/// let error = OptionsError::pricing_error("black_scholes", "invalid volatility input");
/// ```
impl OptionsError {
    /// Creates a validation error with the specified field name and reason.
    ///
    /// This method is used when option parameters fail validation checks.
    ///
    /// # Parameters
    ///
    /// * `field` - The name of the field that failed validation
    /// * `reason` - The reason why validation failed
    ///
    /// # Returns
    ///
    /// An `OptionsError::ValidationError` variant with formatted fields
    pub fn validation_error(field: &str, reason: &str) -> Self {
        OptionsError::ValidationError {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a pricing error with the specified pricing method and reason.
    ///
    /// This method is used when an error occurs during option price calculation.
    ///
    /// # Parameters
    ///
    /// * `method` - The name of the pricing method that encountered an error
    /// * `reason` - The description of what went wrong
    ///
    /// # Returns
    ///
    /// An `OptionsError::PricingError` variant with formatted fields
    pub fn pricing_error(method: &str, reason: &str) -> Self {
        OptionsError::PricingError {
            method: method.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a Greeks calculation error with the specified Greek name and reason.
    ///
    /// This method is used when an error occurs during the calculation of option Greeks
    /// (delta, gamma, theta, vega, etc.).
    ///
    /// # Parameters
    ///
    /// * `greek` - The name of the Greek calculation that failed
    /// * `reason` - The description of what went wrong
    ///
    /// # Returns
    ///
    /// An `OptionsError::GreeksCalculationError` variant with formatted fields
    pub fn greeks_error(greek: &str, reason: &str) -> Self {
        OptionsError::GreeksCalculationError {
            greek: greek.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a time calculation error with the specified operation and reason.
    ///
    /// This method is used when an error occurs during time-related calculations,
    /// such as time to expiration, day count conventions, or calendar adjustments.
    ///
    /// # Parameters
    ///
    /// * `operation` - The name of the time operation that failed
    /// * `reason` - The description of what went wrong
    ///
    /// # Returns
    ///
    /// An `OptionsError::TimeError` variant with formatted fields
    pub fn time_error(operation: &str, reason: &str) -> Self {
        OptionsError::TimeError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a payoff calculation error with the specified reason.
    ///
    /// This method is used when an error occurs during the calculation of option payoffs.
    ///
    /// # Parameters
    ///
    /// * `reason` - The description of what went wrong
    ///
    /// # Returns
    ///
    /// An `OptionsError::PayoffError` variant with formatted reason
    pub fn payoff_error(reason: &str) -> Self {
        OptionsError::PayoffError {
            reason: reason.to_string(),
        }
    }

    /// Creates an update error with the specified field and reason.
    ///
    /// This method is used when an error occurs during the update of option parameters
    /// or other option data.
    ///
    /// # Parameters
    ///
    /// * `field` - The name of the field that failed to update
    /// * `reason` - The description of what went wrong
    ///
    /// # Returns
    ///
    /// An `OptionsError::UpdateError` variant with formatted fields
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
    fn test_to_box_dyn_error_conversion() {
        let error = OptionsError::validation_error("price", "must be positive");
        let boxed: Box<dyn Error> = error.into();
        assert_eq!(
            boxed.to_string(),
            "Validation error for field 'price': must be positive"
        );
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OptionsError>();
    }

    #[test]
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
    fn test_error_conversion_preservation() {
        let original = "preserve this message";
        let error1: OptionsError = original.into();
        let error2: OptionsError = error1.to_string().into();

        assert!(error2.to_string().contains(original));
    }

    #[test]
    fn test_option_result_operations() {
        let success: OptionsResult<i32> = Ok(42);
        let failure: OptionsResult<i32> = Err(OptionsError::validation_error("test", "error"));

        let mapped_success = success.map(|x| x * 2);
        let mapped_failure = failure.map(|x| x * 2);

        assert_eq!(mapped_success.unwrap(), 84);
        assert!(mapped_failure.is_err());
    }

    #[test]
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
