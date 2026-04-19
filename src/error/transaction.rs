use crate::OptionType;
use thiserror::Error;

/// # Transaction Error
///
/// Typed errors for operations on financial transactions. Replaces the
/// previous single-field string carrier so callers can match on the
/// specific failure mode and include the offending value.
///
/// # Variants
///
/// - `NotImplemented` — a trait default implementation was invoked on a
///   concrete type that does not support the operation.
/// - `UnsupportedOptionType` — the requested PnL / settlement path does
///   not support the supplied `OptionType` (typically any non-`European`).
/// - `Other` — catch-all for ad-hoc failures that have not yet earned a
///   dedicated variant.
#[derive(Error, Debug)]
pub enum TransactionError {
    /// A trait method lacks an override on the concrete implementer.
    #[error("{method} not implemented for {type_name}")]
    NotImplemented {
        /// Trait method that was invoked without an override (e.g.
        /// `add_transaction`, `get_transactions`).
        method: &'static str,
        /// `std::any::type_name` of the implementer that reached the
        /// default body.
        type_name: &'static str,
    },

    /// A downstream calculation does not support the supplied option type.
    #[error("unsupported option type in transaction: {option_type:?}")]
    UnsupportedOptionType {
        /// The offending `OptionType` passed through the transaction.
        option_type: OptionType,
    },

    /// Catch-all variant for ad-hoc transaction errors.
    #[error("transaction error: {reason}")]
    Other {
        /// Human-readable description of the failure.
        reason: String,
    },
}

impl TransactionError {
    /// Builds a `NotImplemented` error for a trait default that has no
    /// override on the concrete implementer.
    ///
    /// # Errors
    ///
    /// This is an error constructor — it always returns the variant.
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn not_implemented(method: &'static str, type_name: &'static str) -> Self {
        TransactionError::NotImplemented { method, type_name }
    }

    /// Builds an `UnsupportedOptionType` error for a transaction carrying
    /// an option type the current path does not support.
    ///
    /// # Errors
    ///
    /// This is an error constructor — it always returns the variant.
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn unsupported_option_type(option_type: OptionType) -> Self {
        TransactionError::UnsupportedOptionType { option_type }
    }

    /// Builds a catch-all `Other` variant.
    ///
    /// # Errors
    ///
    /// This is an error constructor — it always returns the variant.
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn other(reason: impl Into<String>) -> Self {
        TransactionError::Other {
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_implemented_display() {
        let err = TransactionError::not_implemented("add_transaction", "Position");
        assert!(err.to_string().contains("add_transaction"));
        assert!(err.to_string().contains("Position"));
    }

    #[test]
    fn test_not_implemented_match_fields() {
        let err = TransactionError::not_implemented("get_transactions", "Position");
        match err {
            TransactionError::NotImplemented { method, type_name } => {
                assert_eq!(method, "get_transactions");
                assert_eq!(type_name, "Position");
            }
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }

    #[test]
    fn test_unsupported_option_type_display() {
        let err = TransactionError::unsupported_option_type(OptionType::American);
        assert!(err.to_string().contains("American"));
    }

    #[test]
    fn test_other_constructor_accepts_str_and_string() {
        let from_str = TransactionError::other("boom");
        let from_string = TransactionError::other(String::from("kaboom"));
        assert!(matches!(from_str, TransactionError::Other { .. }));
        assert!(matches!(from_string, TransactionError::Other { .. }));
        assert!(from_str.to_string().contains("boom"));
        assert!(from_string.to_string().contains("kaboom"));
    }
}
