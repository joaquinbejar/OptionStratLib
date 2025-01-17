/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/
use std::error::Error;
use std::fmt;

/// Represents errors that can occur during decimal operations
#[derive(Debug)]
pub enum DecimalError {
    /// Error when attempting to create a decimal from an invalid value
    InvalidValue { value: f64, reason: String },
    /// Error when performing decimal arithmetic operations
    ArithmeticError { operation: String, reason: String },
    /// Error when converting between decimal types
    ConversionError {
        from_type: String,
        to_type: String,
        reason: String,
    },
    /// Error when a decimal value exceeds its bounds
    OutOfBounds { value: f64, min: f64, max: f64 },
    /// Error when decimal precision is invalid
    InvalidPrecision { precision: i32, reason: String },
}

impl fmt::Display for DecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecimalError::InvalidValue { value, reason } => {
                write!(f, "Invalid decimal value {}: {}", value, reason)
            }
            DecimalError::ArithmeticError { operation, reason } => {
                write!(
                    f,
                    "Decimal arithmetic error during {}: {}",
                    operation, reason
                )
            }
            DecimalError::ConversionError {
                from_type,
                to_type,
                reason,
            } => {
                write!(
                    f,
                    "Failed to convert decimal from {} to {}: {}",
                    from_type, to_type, reason
                )
            }
            DecimalError::OutOfBounds { value, min, max } => {
                write!(
                    f,
                    "Decimal value {} is out of bounds (min: {}, max: {})",
                    value, min, max
                )
            }
            DecimalError::InvalidPrecision { precision, reason } => {
                write!(f, "Invalid decimal precision {}: {}", precision, reason)
            }
        }
    }
}

impl Error for DecimalError {}

// Convenient type alias for Results with DecimalError
pub type DecimalResult<T> = Result<T, DecimalError>;

// Helper methods for creating common decimal errors
impl DecimalError {
    pub fn invalid_value(value: f64, reason: &str) -> Self {
        DecimalError::InvalidValue {
            value,
            reason: reason.to_string(),
        }
    }

    pub fn arithmetic_error(operation: &str, reason: &str) -> Self {
        DecimalError::ArithmeticError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn conversion_error(from_type: &str, to_type: &str, reason: &str) -> Self {
        DecimalError::ConversionError {
            from_type: from_type.to_string(),
            to_type: to_type.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn out_of_bounds(value: f64, min: f64, max: f64) -> Self {
        DecimalError::OutOfBounds { value, min, max }
    }

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_value_error() {
        let error = DecimalError::invalid_value(-1.0, "Value cannot be negative");
        assert!(matches!(error, DecimalError::InvalidValue { .. }));
        assert!(error.to_string().contains("cannot be negative"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_arithmetic_error() {
        let error = DecimalError::arithmetic_error("division", "Division by zero");
        assert!(matches!(error, DecimalError::ArithmeticError { .. }));
        assert!(error.to_string().contains("Division by zero"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_conversion_error() {
        let error = DecimalError::conversion_error("f64", "Decimal", "Value out of range");
        assert!(matches!(error, DecimalError::ConversionError { .. }));
        assert!(error.to_string().contains("out of range"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_out_of_bounds_error() {
        let error = DecimalError::out_of_bounds(150.0, 0.0, 100.0);
        assert!(matches!(error, DecimalError::OutOfBounds { .. }));
        assert!(error.to_string().contains("150"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_precision_error() {
        let error = DecimalError::invalid_precision(-1, "Precision must be non-negative");
        assert!(matches!(error, DecimalError::InvalidPrecision { .. }));
        assert!(error.to_string().contains("non-negative"));
    }
}
