use thiserror::Error;

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
#[derive(Error, Debug)]
#[error("TransactionError: {message}")]
pub struct TransactionError {
    /// The error message
    pub message: String,
}
