//! Re-export of `ExpirationDate` from the standalone `expiration_date` crate.
//!
//! This module re-exports the `ExpirationDate` enum and its error type from the
//! external `expiration_date` crate, which provides all the core functionality
//! for handling financial instrument expiration dates.
//!
//! It also owns the single calendar-overflow guard the rest of the crate uses.
//! The `expiration_date` crate resolves a relative `Days(n)` with the `+`
//! operator on `DateTime<Utc>` and with `TimeDelta::days`, both of which abort
//! the process on overflow instead of reporting it, so every call site in this
//! library goes through [`resolve_expiration_date`] or
//! [`reject_unrepresentable_expiration`] rather than calling `get_date` and
//! friends directly.

use chrono::{DateTime, TimeDelta, Utc};
pub use expiration_date::ExpirationDate;
pub use expiration_date::error::ExpirationDateError;
use positive::Positive;

/// Hour of the fixed resolution base used by
/// `ExpirationDate::get_date_with_options(true)` and therefore by
/// `ExpirationDate::get_date_string`.
const FIXED_BASE_HOUR: u32 = 18;

/// Minute of the fixed resolution base. See [`FIXED_BASE_HOUR`].
const FIXED_BASE_MINUTE: u32 = 30;

/// Reports a day count that no calendar instant can represent.
#[cold]
#[inline(never)]
fn unrepresentable(days: Positive) -> ExpirationDateError {
    ExpirationDateError::ArithmeticOverflow(format!(
        "expiration of {days} days is outside the representable calendar range"
    ))
}

/// Adds a whole number of days to `base` without aborting on overflow.
///
/// Mirrors `base + Duration::days(i64::try_from(days)?)`, the expression the
/// `expiration_date` crate uses, step by step: the same `Positive` to `i64`
/// conversion, the same day-to-span conversion and the same addition, each
/// through its checked counterpart. The instant returned is therefore
/// identical for every input the panicking form would have accepted.
fn checked_offset_days(
    base: DateTime<Utc>,
    days: Positive,
) -> Result<DateTime<Utc>, ExpirationDateError> {
    let whole_days = i64::try_from(days)?;
    let span = TimeDelta::try_days(whole_days).ok_or_else(|| unrepresentable(days))?;
    base.checked_add_signed(span)
        .ok_or_else(|| unrepresentable(days))
}

/// The base a relative expiration is resolved from: the reference datetime
/// when one has been installed, the current instant otherwise.
fn relative_base() -> DateTime<Utc> {
    ExpirationDate::get_reference_datetime().unwrap_or_else(Utc::now)
}

/// The fixed base `ExpirationDate::get_date_string` resolves from: today at
/// [`FIXED_BASE_HOUR`]:[`FIXED_BASE_MINUTE`] UTC.
fn fixed_base() -> Option<DateTime<Utc>> {
    Utc::now()
        .date_naive()
        .and_hms_opt(FIXED_BASE_HOUR, FIXED_BASE_MINUTE, 0)
        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

/// Resolves an expiration into a calendar instant without aborting on overflow.
///
/// Drop-in replacement for [`ExpirationDate::get_date`]: it returns the same
/// instant for every representable input, and a typed error instead of a
/// process abort for the rest. `Days(1_000_000_000)` overflows the
/// `DateTime + TimeDelta` addition and `Days(1e15)` overflows `TimeDelta`
/// itself; both come back as [`ExpirationDateError::ArithmeticOverflow`].
///
/// # Errors
///
/// * [`ExpirationDateError::ArithmeticOverflow`] - the day count is outside
///   the range a `DateTime<Utc>` can hold.
/// * [`ExpirationDateError::ConversionError`] - the day count does not fit an
///   `i64`, propagated from the `Positive` conversion.
#[must_use = "the resolved instant carries the failure the panicking form would have aborted on"]
pub fn resolve_expiration_date(
    expiration_date: &ExpirationDate,
) -> Result<DateTime<Utc>, ExpirationDateError> {
    match expiration_date {
        ExpirationDate::Days(days) => checked_offset_days(relative_base(), *days),
        ExpirationDate::DateTime(datetime) => Ok(*datetime),
    }
}

/// Rejects a relative expiration that no calendar instant can represent.
///
/// Validates the day count against both bases the `expiration_date` crate
/// resolves from — the relative one used by `get_date` and the fixed 18:30 UTC
/// one used by `get_date_string` — so a caller that goes on to invoke either
/// of them cannot reach their panicking arithmetic. Prefer
/// [`resolve_expiration_date`] where the resolved instant is what is wanted;
/// this guard exists for the call sites that need the formatted string or the
/// day count instead.
///
/// # Errors
///
/// * [`ExpirationDateError::ArithmeticOverflow`] - the day count is outside
///   the range a `DateTime<Utc>` can hold, from either base.
/// * [`ExpirationDateError::ConversionError`] - the day count does not fit an
///   `i64`, propagated from the `Positive` conversion.
pub fn reject_unrepresentable_expiration(
    expiration_date: &ExpirationDate,
) -> Result<(), ExpirationDateError> {
    let ExpirationDate::Days(days) = expiration_date else {
        return Ok(());
    };
    checked_offset_days(relative_base(), *days)?;
    let fixed = fixed_base().ok_or_else(|| unrepresentable(*days))?;
    checked_offset_days(fixed, *days)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn pos(value: rust_decimal::Decimal) -> Positive {
        Positive::new_decimal(value).unwrap_or(Positive::ZERO)
    }

    #[test]
    fn test_resolve_expiration_date_days_matches_get_date() {
        let expiration = ExpirationDate::Days(pos(dec!(30.0)));
        let resolved = resolve_expiration_date(&expiration).expect("30 days is representable");
        let reference = expiration.get_date().expect("30 days is representable");
        assert_eq!(resolved.date_naive(), reference.date_naive());
    }

    #[test]
    fn test_resolve_expiration_date_datetime_is_returned_unchanged() {
        let instant = Utc::now();
        let resolved = resolve_expiration_date(&ExpirationDate::DateTime(instant))
            .expect("an explicit instant is always representable");
        assert_eq!(resolved, instant);
    }

    #[test]
    fn test_resolve_expiration_date_datetime_overflow_returns_error() {
        let error = resolve_expiration_date(&ExpirationDate::Days(pos(dec!(1000000000.0))))
            .expect_err("a billion days overflows the calendar");
        assert!(matches!(error, ExpirationDateError::ArithmeticOverflow(_)));
    }

    #[test]
    fn test_resolve_expiration_date_span_overflow_returns_error() {
        let error = resolve_expiration_date(&ExpirationDate::Days(pos(dec!(1000000000000000.0))))
            .expect_err("1e15 days overflows TimeDelta itself");
        assert!(matches!(error, ExpirationDateError::ArithmeticOverflow(_)));
    }

    #[test]
    fn test_resolve_expiration_date_beyond_i64_returns_error() {
        let error = resolve_expiration_date(&ExpirationDate::Days(Positive::MAX))
            .expect_err("Positive::MAX days does not fit an i64");
        assert!(matches!(
            error,
            ExpirationDateError::ConversionError { .. } | ExpirationDateError::PositiveError(_)
        ));
    }

    #[test]
    fn test_reject_unrepresentable_expiration_accepts_ordinary_horizons() {
        for days in [dec!(0.0), dec!(1.0), dec!(30.0), dec!(3650.0)] {
            reject_unrepresentable_expiration(&ExpirationDate::Days(pos(days)))
                .expect("ordinary horizons are representable");
        }
    }

    #[test]
    fn test_reject_unrepresentable_expiration_rejects_billion_days() {
        let error =
            reject_unrepresentable_expiration(&ExpirationDate::Days(pos(dec!(1000000000.0))))
                .expect_err("a billion days overflows the calendar");
        assert!(matches!(error, ExpirationDateError::ArithmeticOverflow(_)));
    }

    #[test]
    fn test_reject_unrepresentable_expiration_accepts_any_datetime() {
        reject_unrepresentable_expiration(&ExpirationDate::DateTime(Utc::now()))
            .expect("an explicit instant needs no calendar arithmetic");
    }
}
