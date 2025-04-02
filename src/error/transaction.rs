use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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