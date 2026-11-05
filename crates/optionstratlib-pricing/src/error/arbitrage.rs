/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/11/26
******************************************************************************/

//! Target crate (ADR-0001 D6): **pricing**. Owns `StaticArbitrageError`.

use crate::error::SviError;
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::{Positive, PositiveError};
use rust_decimal::Decimal;
use thiserror::Error;

/// Reasons why a static arbitrage check could not run.
///
/// Raised by [`check_butterfly`](crate::volatility::check_butterfly),
/// [`check_calendar`](crate::volatility::check_calendar) and
/// [`check_static_arbitrage`](crate::volatility::check_static_arbitrage). A
/// violation found is not an error: it goes in the
/// [`StaticArbitrageReport`](crate::volatility::StaticArbitrageReport). This is
/// for a check that cannot be evaluated, which must not read as one that passed.
///
/// The aggregate `optionstratlib::error::Error` has no variant for it yet:
/// adding one to that exhaustive enum is breaking, and #944 adds it with the
/// 0.24.0 bump.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StaticArbitrageError {
    /// The grid of log moneyness has no point, so nothing would be tested.
    #[error("static arbitrage grid is empty, so nothing would be tested")]
    EmptyGrid,

    /// Too few slices to compare one expiry with another.
    #[error("calendar check needs at least {required} slices, got {got}")]
    TooFewSlices {
        /// The slices supplied.
        got: usize,
        /// The minimum: two.
        required: usize,
    },

    /// Two slices share an expiry, so they have no order to compare them in,
    /// and they may disagree on the total variance of that expiry.
    #[error("two slices share the expiry {time_to_expiry}, in years")]
    DuplicateExpiry {
        /// The shared time to expiry, in years.
        time_to_expiry: Positive,
    },

    /// A slice has no forward or no time to expiry. `SviSlice` cannot reach
    /// this; another implementor of the trait can.
    #[error("invalid slice with expiry {time_to_expiry} and forward {forward}: {reason}")]
    InvalidSlice {
        /// The time to expiry of the slice, in years.
        time_to_expiry: Positive,
        /// The forward of the slice.
        forward: Positive,
        /// The condition the slice breaks.
        reason: &'static str,
    },

    /// `forward * exp(k)` is outside the range of a `Positive`: it overflows or
    /// rounds to zero, so a finding at this point cannot name its strike.
    #[error("no strike for forward {forward} at log moneyness {log_moneyness}")]
    StrikeOutOfRange {
        /// The forward of the slice.
        forward: Positive,
        /// The log moneyness of the grid point.
        log_moneyness: Decimal,
    },

    /// A slice other than `SviSlice` could not evaluate itself at a grid point,
    /// for a reason none of the other variants describes.
    #[error("slice could not be evaluated at log moneyness {log_moneyness}: {reason}")]
    SliceEvaluation {
        /// The log moneyness of the grid point.
        log_moneyness: Decimal,
        /// What went wrong.
        reason: String,
    },

    /// An `SviSlice` could not be evaluated at a grid point.
    #[error(transparent)]
    Svi(#[from] SviError),

    /// A checked `Decimal` operation failed.
    #[error(transparent)]
    Decimal(#[from] DecimalError),

    /// A value could not be represented as a `Positive`.
    #[error(transparent)]
    Positive(#[from] PositiveError),
}

#[cfg(test)]
mod tests_static_arbitrage_error {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_static_arbitrage_error_too_few_slices_names_both_counts() {
        let message = StaticArbitrageError::TooFewSlices {
            got: 1,
            required: 2,
        }
        .to_string();
        assert!(message.contains("at least 2"), "{message}");
        assert!(message.contains("got 1"), "{message}");
    }

    #[test]
    fn test_static_arbitrage_error_duplicate_expiry_names_the_expiry() {
        let message = StaticArbitrageError::DuplicateExpiry {
            time_to_expiry: Positive::new_decimal(dec!(0.25)).unwrap(),
        }
        .to_string();
        assert!(message.contains("0.25"), "{message}");
    }

    #[test]
    fn test_static_arbitrage_error_strike_out_of_range_names_both_values() {
        let message = StaticArbitrageError::StrikeOutOfRange {
            forward: Positive::HUNDRED,
            log_moneyness: dec!(1000),
        }
        .to_string();
        assert!(message.contains("100"), "{message}");
        assert!(message.contains("1000"), "{message}");
    }

    #[test]
    fn test_static_arbitrage_error_lower_errors_convert_with_from() {
        let decimal: StaticArbitrageError = DecimalError::arithmetic_error("arb", "boom").into();
        assert!(matches!(decimal, StaticArbitrageError::Decimal(_)));
        let svi: StaticArbitrageError = SviError::ZeroForward.into();
        assert!(matches!(svi, StaticArbitrageError::Svi(_)));
    }
}
