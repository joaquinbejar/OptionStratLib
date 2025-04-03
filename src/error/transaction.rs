use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// # Transaction Error
///
/// Defines the error type used for transaction-related operations in the financial library.
/// This error is typically raised when operations involving trades, orders, or financial
/// transactions encounter issues such as validation failures, execution problems,
/// or data inconsistencies.
/// Represents an error that occurred during a financial transaction operation.
///
/// This struct encapsulates error information for transaction-related operations,
/// providing a descriptive message that explains the nature of the error.
///
#[derive(Debug)]
pub struct TransactionError {
    /// The error message
    pub message: String,
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TransactionError: {}", self.message)
    }
}

impl Error for TransactionError {}
