//! Target crate (ADR-0001 D6): **core**. Owns `ClockError`.
//!
//! A build without the `clock` feature has no wall clock, and everything that
//! asks for the current time gets this error rather than a made-up instant: a
//! wrong time would silently price an option wrongly.

use expiration_date::error::ExpirationDateError;
use thiserror::Error;

/// The current time was asked for and this build cannot provide it.
///
/// The exhaustive error enums of this library carry it through their existing
/// variants. In particular [`ExpirationDateError::InvalidDateTime`] holds its
/// message, so a time-to-expiry that fails for lack of a clock arrives as
/// `PricingError::ExpirationDate`, `GreeksError::ExpirationDate` and so on.
///
/// Only [`crate::utils::time::now`], [`crate::utils::time::evaluation_time`]
/// and [`crate::utils::time::resolution_base`] return it as such.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ClockError {
    /// The crate was built without the `clock` feature, and the operation has
    /// no explicit time and no reference datetime to use instead.
    #[error(
        "the wall clock is unavailable: this build has no `clock` feature, so pass the time \
         explicitly or set the thread's reference datetime"
    )]
    Unavailable,
}

impl From<ClockError> for ExpirationDateError {
    #[cold]
    #[inline(never)]
    fn from(err: ClockError) -> Self {
        Self::InvalidDateTime(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_error_converts_to_invalid_datetime_keeping_its_message() {
        let message = ClockError::Unavailable.to_string();
        let converted = ExpirationDateError::from(ClockError::Unavailable);
        assert!(
            matches!(converted, ExpirationDateError::InvalidDateTime(ref text) if *text == message)
        );
        assert!(message.contains("wall clock is unavailable"));
    }
}
