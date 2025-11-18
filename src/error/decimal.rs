/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/
use thiserror::Error;

/// # Decimal Error Management
///
/// Represents errors that can occur during decimal operations in financial calculations.
/// This enum provides a structured way to handle various error conditions that may arise
/// when working with decimal values, including validation, arithmetic operations,
/// conversions, and precision issues.
///
/// # Use Cases
///
/// * Financial calculations requiring strict decimal precision
/// * Currency and monetary value operations
/// * Option pricing models where precision is critical
/// * Risk management calculations
///
/// # Error Propagation
///
/// These errors are typically wrapped in `DecimalResult` and propagated through
/// the application's calculation pipeline.
///
/// # Variants
///
/// * `InvalidValue` - Handles errors when a value cannot be represented as a valid decimal
/// * `ArithmeticError` - Handles errors during mathematical operations
/// * `ConversionError` - Handles errors when converting between different decimal representations
/// * `OutOfBounds` - Handles errors when a value exceeds defined limits
/// * `InvalidPrecision` - Handles errors related to decimal precision settings
///
/// # Example Usage
///
/// ```rust
/// use optionstratlib::error::DecimalError;
///
/// fn validate_decimal(value: f64) -> Result<(), DecimalError> {
///     if value.is_nan() {
///         return Err(DecimalError::InvalidValue {
///             value,
///             reason: "Value cannot be NaN".to_string(),
///         });
///     }
///     
///     if value < 0.0 || value > 100.0 {
///         return Err(DecimalError::OutOfBounds {
///             value,
///             min: 0.0,
///             max: 100.0,
///         });
///     }
///     
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum DecimalError {
    /// Error when attempting to create a decimal from an invalid value
    ///
    /// Occurs when a value cannot be properly represented as a decimal,
    /// such as when it's NaN, infinity, or otherwise unsuitable for
    /// financial calculations.
    #[error("Invalid decimal value {value}: {reason}")]
    InvalidValue {
        /// The problematic value that caused the error
        value: f64,
        /// Detailed explanation of why the value is invalid
        reason: String,
    },

    /// Error when performing decimal arithmetic operations
    ///
    /// Occurs during mathematical operations such as addition, subtraction,
    /// multiplication, or division when the operation cannot be completed
    /// correctly (e.g., division by zero, overflow).
    #[error("Decimal arithmetic error during {operation}: {reason}")]
    ArithmeticError {
        /// The operation that failed (e.g., "addition", "division")
        operation: String,
        /// Detailed explanation of why the operation failed
        reason: String,
    },

    /// Error when converting between decimal types
    ///
    /// Occurs when a decimal value cannot be correctly converted from one
    /// representation to another, such as between different precision levels
    /// or between different decimal formats.
    #[error("Failed to convert decimal from {from_type} to {to_type}: {reason}")]
    ConversionError {
        /// The source type being converted from
        from_type: String,
        /// The destination type being converted to
        to_type: String,
        /// Detailed explanation of why the conversion failed
        reason: String,
    },

    /// Error when a decimal value exceeds its bounds
    ///
    /// Occurs when a decimal value falls outside of acceptable minimum
    /// or maximum values for a specific calculation context.
    #[error("Decimal value {value} is out of bounds (min: {min}, max: {max})")]
    OutOfBounds {
        /// The value that is out of bounds
        value: f64,
        /// The minimum acceptable value
        min: f64,
        /// The maximum acceptable value
        max: f64,
    },

    /// Error when decimal precision is invalid
    ///
    /// Occurs when an operation specifies or results in an invalid precision
    /// level that cannot be properly handled.
    #[error("Invalid decimal precision {precision}: {reason}")]
    InvalidPrecision {
        /// The problematic precision value
        precision: i32,
        /// Detailed explanation of why the precision is invalid
        reason: String,
    },
}

/// A specialized `Result` type for decimal calculation operations.
///
/// This type alias provides a convenient shorthand for operations that can result in a
/// `DecimalError`. It helps improve code readability and reduces boilerplate when working
/// with decimal calculations throughout the library.
///
/// # Type Parameters
///
/// * `T` - The successful result type of the operation
///
/// # Examples
///
/// ```rust
/// use optionstratlib::error::{DecimalError, DecimalResult};
///
/// fn divide(a: f64, b: f64) -> DecimalResult<f64> {
///     if b == 0.0 {
///         Err(DecimalError::ArithmeticError {
///             operation: "division".to_string(),
///             reason: "division by zero".to_string(),
///         })
///     } else {
///         Ok(a / b)
///     }
/// }
/// ```
///
/// # Usage Context
///
/// This type is primarily used in the financial calculations and decimal handling
/// components of the library, where precise decimal operations are critical and
/// error handling needs to be consistent and well-structured.
///
/// # Related Types
///
/// * `DecimalError` - The error type representing various decimal operation failures
pub type DecimalResult<T> = Result<T, DecimalError>;

/// Helper methods for creating common decimal errors
///
/// This implementation provides convenient factory methods to create
/// standardized instances of `DecimalError` for common error scenarios
/// in decimal operations. These methods help maintain consistency in
/// error creation across the codebase and simplify the construction of
/// descriptive error instances.
///
/// # Example
///
/// ```rust
/// use optionstratlib::error::DecimalError;
/// // Creating an invalid value error
/// let err = DecimalError::invalid_value(12.34, "Value exceeds maximum allowed");
///
/// // Creating an arithmetic error
/// let div_err = DecimalError::arithmetic_error("division", "Division by zero");
/// ```
impl DecimalError {
    /// Creates a new `InvalidValue` error
    ///
    /// Used when a decimal value fails validation due to being outside
    /// accepted ranges or otherwise inappropriate for the context.
    ///
    /// # Parameters
    ///
    /// * `value` - The problematic floating-point value
    /// * `reason` - Explanation of why the value is invalid
    ///
    /// # Returns
    ///
    /// A new `DecimalError::InvalidValue` instance
    pub fn invalid_value(value: f64, reason: &str) -> Self {
        DecimalError::InvalidValue {
            value,
            reason: reason.to_string(),
        }
    }

    /// Creates a new `ArithmeticError` error
    ///
    /// Used when a mathematical operation on decimal values fails, such as
    /// division by zero, overflow, or underflow.
    ///
    /// # Parameters
    ///
    /// * `operation` - The name of the operation that failed (e.g., "addition", "division")
    /// * `reason` - Explanation of why the operation failed
    ///
    /// # Returns
    ///
    /// A new `DecimalError::ArithmeticError` instance
    pub fn arithmetic_error(operation: &str, reason: &str) -> Self {
        DecimalError::ArithmeticError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `ConversionError` error
    ///
    /// Used when conversion between decimal types or from/to other number
    /// types fails due to compatibility or range issues.
    ///
    /// # Parameters
    ///
    /// * `from_type` - The source type being converted from
    /// * `to_type` - The destination type being converted to
    /// * `reason` - Explanation of why the conversion failed
    ///
    /// # Returns
    ///
    /// A new `DecimalError::ConversionError` instance
    pub fn conversion_error(from_type: &str, to_type: &str, reason: &str) -> Self {
        DecimalError::ConversionError {
            from_type: from_type.to_string(),
            to_type: to_type.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Creates a new `OutOfBounds` error
    ///
    /// Used when a decimal value falls outside of specified minimum and maximum bounds.
    ///
    /// # Parameters
    ///
    /// * `value` - The out-of-bounds floating-point value
    /// * `min` - The lower bound (inclusive) of the valid range
    /// * `max` - The upper bound (inclusive) of the valid range
    ///
    /// # Returns
    ///
    /// A new `DecimalError::OutOfBounds` instance
    pub fn out_of_bounds(value: f64, min: f64, max: f64) -> Self {
        DecimalError::OutOfBounds { value, min, max }
    }

    /// Creates a new `InvalidPrecision` error
    ///
    /// Used when a specified decimal precision is invalid, such as being negative,
    /// too large, or otherwise inappropriate for the context.
    ///
    /// # Parameters
    ///
    /// * `precision` - The problematic precision value
    /// * `reason` - Explanation of why the precision is invalid
    ///
    /// # Returns
    ///
    /// A new `DecimalError::InvalidPrecision` instance
    pub fn invalid_precision(precision: i32, reason: &str) -> Self {
        DecimalError::InvalidPrecision {
            precision,
            reason: reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_value_error() {
        let error = DecimalError::invalid_value(-1.0, "Value cannot be negative");
        assert!(matches!(error, DecimalError::InvalidValue { .. }));
        assert!(error.to_string().contains("cannot be negative"));
    }

    #[test]
    fn test_arithmetic_error() {
        let error = DecimalError::arithmetic_error("division", "Division by zero");
        assert!(matches!(error, DecimalError::ArithmeticError { .. }));
        assert!(error.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_conversion_error() {
        let error = DecimalError::conversion_error("f64", "Decimal", "Value out of range");
        assert!(matches!(error, DecimalError::ConversionError { .. }));
        assert!(error.to_string().contains("out of range"));
    }

    #[test]
    fn test_out_of_bounds_error() {
        let error = DecimalError::out_of_bounds(150.0, 0.0, 100.0);
        assert!(matches!(error, DecimalError::OutOfBounds { .. }));
        assert!(error.to_string().contains("150"));
    }

    #[test]
    fn test_invalid_precision_error() {
        let error = DecimalError::invalid_precision(-1, "Precision must be non-negative");
        assert!(matches!(error, DecimalError::InvalidPrecision { .. }));
        assert!(error.to_string().contains("non-negative"));
    }
}
