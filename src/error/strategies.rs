/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 24/12/24
 ******************************************************************************/
use std::fmt;
use crate::error::probability::ProbabilityError;

use std::error::Error;

impl Error for StrategyError {}

impl Error for PriceErrorKind {}
impl Error for BreakEvenErrorKind {}
impl Error for ProfitLossErrorKind {}
impl Error for OperationErrorKind {}
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
}

#[derive(Debug)]
pub enum PriceErrorKind {
    /// Error when underlying price is not available or invalid
    InvalidUnderlyingPrice {
        reason: String,
    },
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
    CalculationError {
        reason: String,
    },
    /// No break-even points found
    NoBreakEvenPoints,
}

#[derive(Debug)]
pub enum ProfitLossErrorKind {
    /// Error calculating maximum profit
    MaxProfitError {
        reason: String,
    },
    /// Error calculating maximum loss
    MaxLossError {
        reason: String,
    },
    /// Error in profit range calculation
    ProfitRangeError {
        reason: String,
    },
}

#[derive(Debug)]
pub enum OperationErrorKind {
    /// Operation not supported for this strategy
    NotSupported {
        operation: String,
        strategy_type: String,
    },
    /// Invalid parameters for operation
    InvalidParameters {
        operation: String,
        reason: String,
    },
}

impl fmt::Display for StrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyError::PriceError(err) => write!(f, "Price error: {}", err),
            StrategyError::BreakEvenError(err) => write!(f, "Break-even error: {}", err),
            StrategyError::ProfitLossError(err) => write!(f, "Profit/Loss error: {}", err),
            StrategyError::OperationError(err) => write!(f, "Operation error: {}", err),
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

impl fmt::Display for OperationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationErrorKind::NotSupported { operation, strategy_type } => {
                write!(
                    f,
                    "Operation '{}' is not supported for strategy '{}'",
                    operation, strategy_type
                )
            }
            OperationErrorKind::InvalidParameters { operation, reason } => {
                write!(f, "Invalid parameters for operation '{}': {}", operation, reason)
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

impl From<StrategyError> for ProbabilityError {
    fn from(error: StrategyError) -> Self {
        match error {
            StrategyError::ProfitLossError(kind) => match kind {
                ProfitLossErrorKind::MaxProfitError { reason } |
                ProfitLossErrorKind::MaxLossError { reason } |
                ProfitLossErrorKind::ProfitRangeError { reason } => {
                    ProbabilityError::from(reason)
                }
            },
            StrategyError::PriceError(kind) => match kind {
                PriceErrorKind::InvalidUnderlyingPrice { reason } |
                PriceErrorKind::InvalidPriceRange { start: _, end: _, reason } => {
                    ProbabilityError::from(reason)
                }
            },
            StrategyError::BreakEvenError(kind) => match kind {
                BreakEvenErrorKind::CalculationError { reason } => {
                    ProbabilityError::from(reason)
                },
                BreakEvenErrorKind::NoBreakEvenPoints => {
                    ProbabilityError::from("No break-even points found".to_string())
                }
            },
            StrategyError::OperationError(kind) => match kind {
                OperationErrorKind::NotSupported { operation, strategy_type } => {
                    ProbabilityError::from(
                        format!("Operation '{}' not supported for strategy '{}'", operation, strategy_type)
                    )
                },
                OperationErrorKind::InvalidParameters { operation, reason } => {
                    ProbabilityError::from(
                        format!("Invalid parameters for operation '{}': {}", operation, reason)
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_from_str {
    use super::*;

    #[test]
    fn test_strategy_to_probability_error_conversion() {
        let strategy_error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        let probability_error = ProbabilityError::from(strategy_error);

        // Ajusta este assert según la estructura real de ProbabilityError
        assert!(probability_error.to_string().contains("max_profit"));
        assert!(probability_error.to_string().contains("TestStrategy"));
    }

    #[test]
    fn test_profit_loss_error_conversion() {
        let strategy_error = StrategyError::ProfitLossError(ProfitLossErrorKind::MaxProfitError {
            reason: "Test error".to_string()
        });
        let probability_error = ProbabilityError::from(strategy_error);

        // Ajusta este assert según la estructura real de ProbabilityError
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
        assert!(error.to_string().contains("Cannot calculate maximum profit"));
    }

    #[test]
    fn test_operation_error_display() {
        let error = StrategyError::operation_not_supported("max_profit", "TestStrategy");
        assert!(error.to_string().contains("max_profit"));
        assert!(error.to_string().contains("TestStrategy"));
    }
}