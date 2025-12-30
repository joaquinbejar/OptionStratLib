/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 31/12/25
******************************************************************************/

use thiserror::Error;

/// Represents errors that can occur during trade-related operations.
///
/// This enum encapsulates various error conditions that may arise during trade execution,
/// validation, or parameter processing. It provides detailed context about what went wrong
/// during trading operations.
///
/// `TradeError` is particularly useful for diagnosing problems in trade execution and validation.
#[derive(Error, Debug)]
pub enum TradeError {
    /// Error indicating that a trade is invalid.
    ///
    /// This variant is used when a trade doesn't meet the requirements
    /// for valid trading operations.
    #[error("Invalid trade: {reason}")]
    InvalidTrade {
        /// A description explaining why the trade is invalid.
        reason: String,
    },

    /// Error indicating that a trade status is invalid.
    ///
    /// This variant is used when a trade status doesn't meet the requirements
    /// for valid trading operations.
    #[error("Invalid trade status: {reason}")]
    InvalidTradeStatus {
        /// A description explaining why the trade status is invalid.
        reason: String,
    },
}

/// Helper methods for creating trading-related errors
///
/// This implementation provides a set of convenience helper methods for creating
/// different types of trading errors. These methods create properly structured
/// error instances with clear, descriptive information about what went wrong.
///
/// # Methods
///
/// These helper methods simplify error creation throughout the codebase and ensure
/// that errors have consistent formatting and information.
impl TradeError {
    /// Creates a generic invalid trade error
    ///
    /// # Parameters
    ///
    /// * `reason` - A description of why the trade is invalid
    ///
    /// # Returns
    ///
    /// A `TradeError::InvalidTrade` variant
    pub fn invalid_trade(reason: &str) -> Self {
        TradeError::InvalidTrade {
            reason: reason.to_string(),
        }
    }

    /// Creates a generic invalid trade status error
    ///
    /// # Parameters
    ///
    /// * `reason` - A description of why the trade status is invalid
    ///
    /// # Returns
    ///
    /// A `TradeError::InvalidTradeStatus` variant
    pub fn invalid_trade_status(reason: &str) -> Self {
        TradeError::InvalidTradeStatus {
            reason: reason.to_string(),
        }
    }
}

impl From<&str> for TradeError {
    fn from(s: &str) -> Self {
        TradeError::InvalidTrade {
            reason: s.to_string(),
        }
    }
}

impl From<String> for TradeError {
    fn from(s: String) -> Self {
        TradeError::InvalidTrade { reason: s }
    }
}

#[cfg(test)]
mod tests_trade_errors {
    use super::*;

    #[test]
    fn test_invalid_trade_error() {
        let error = TradeError::InvalidTrade {
            reason: "Trade is not valid".to_string(),
        };

        assert_eq!(error.to_string(), "Invalid trade: Trade is not valid");
    }

    #[test]
    fn test_invalid_trade_status_error() {
        let error = TradeError::InvalidTradeStatus {
            reason: "Trade status is not valid".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Invalid trade status: Trade status is not valid"
        );
    }

    #[test]
    fn test_invalid_trade_error_helper_method() {
        let error = TradeError::invalid_trade("Trade is not valid");
        assert_eq!(error.to_string(), "Invalid trade: Trade is not valid");
    }

    #[test]
    fn test_invalid_trade_status_error_helper_method() {
        let error = TradeError::invalid_trade_status("Trade status is not valid");
        assert_eq!(
            error.to_string(),
            "Invalid trade status: Trade status is not valid"
        );
    }
}
