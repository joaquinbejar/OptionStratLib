/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/10/24
******************************************************************************/

use crate::constants::*;
use crate::error::ClockError;
#[cfg(feature = "clock")]
use chrono::NaiveTime;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use expiration_date::ExpirationDate;
use expiration_date::error::ExpirationDateError;
use positive::constants::DAYS_IN_A_YEAR;
use positive::{Positive, PositiveError};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
use positive::pos_or_panic;

/// Reads the wall clock.
///
/// The only place in the library that does, together with the local-date
/// helper behind [`get_x_days_formatted`]. Exists only with the `clock`
/// feature; use [`now`] where the code has an error channel.
#[cfg(feature = "clock")]
#[must_use]
#[inline]
pub fn wall_clock() -> DateTime<Utc> {
    Utc::now()
}

/// The current instant, from the wall clock.
///
/// # Errors
///
/// Returns [`ClockError::Unavailable`] in a build without the `clock` feature.
/// There is no fallback to a made-up instant.
#[cfg(feature = "clock")]
#[inline]
pub fn now() -> Result<DateTime<Utc>, ClockError> {
    Ok(wall_clock())
}

/// The current instant, from the wall clock.
///
/// # Errors
///
/// Returns [`ClockError::Unavailable`] in a build without the `clock` feature,
/// as this one is. There is no fallback to a made-up instant.
#[cfg(not(feature = "clock"))]
#[inline]
pub fn now() -> Result<DateTime<Utc>, ClockError> {
    Err(ClockError::Unavailable)
}

/// The current instant when the build has a clock, `existing` otherwise.
///
/// For the code that re-stamps something that already carries a time, such as
/// a leg rebuilt into a strategy or the P&L of a dated transaction.
///
/// **The two builds stamp differently**: the same call gives the time of the call
/// with the `clock` feature and `existing` without it, and features unify, so
/// another crate in the build can change which one this is. That is acceptable
/// because the result only labels a record and goes into no price, Greek or P&L
/// figure, and because `existing` came from the caller, so it is never a made-up
/// time. Code whose result depends on the instant must not use this: take the time
/// as an argument, or use [`now`] and report [`ClockError`].
#[must_use]
#[inline]
pub fn now_or(existing: DateTime<Utc>) -> DateTime<Utc> {
    now().unwrap_or(existing)
}

/// The instant a time-to-expiry is measured from.
///
/// With the `clock` feature, the wall clock; the thread's reference datetime is
/// ignored, as `ExpirationDate::get_years` ignores it. Without it, the
/// thread's reference datetime.
///
/// **The two builds can evaluate differently**: with a reference datetime set, a
/// `DateTime` expiration is measured from the wall clock with the `clock` feature
/// and from the reference without it, so the same call can price differently, and
/// features unify. That is acceptable because the default build has to keep
/// measuring from the wall clock, and without a clock the reference datetime is the
/// only time the caller can have chosen: it is never made up, and with none this
/// fails. Code that needs the same answer in every build takes the time as an
/// argument: the `*_at` forms read neither.
///
/// # Errors
///
/// Returns [`ClockError::Unavailable`] when the build has no clock and the
/// thread has no reference datetime.
#[inline]
pub fn evaluation_time() -> Result<DateTime<Utc>, ClockError> {
    now().or_else(|error| ExpirationDate::get_reference_datetime().ok_or(error))
}

/// The instant a relative expiration is resolved from.
///
/// The thread's reference datetime when one is set, the wall clock otherwise,
/// as `ExpirationDate::get_date` does.
///
/// # Errors
///
/// Returns [`ClockError::Unavailable`] when the thread has no reference
/// datetime and the build has no clock.
#[inline]
pub fn resolution_base() -> Result<DateTime<Utc>, ClockError> {
    ExpirationDate::get_reference_datetime().map_or_else(now, Ok)
}

/// The years from now to `expiration`, on Actual/365 Fixed.
///
/// Gives what `ExpirationDate::get_years` gives and also exists without the
/// `clock` feature. A `Days` needs no clock: its count over 365 is the answer,
/// in every build. A `DateTime` is measured from [`evaluation_time`].
///
/// # Errors
///
/// Returns [`ExpirationDateError::InvalidDateTime`] for a `DateTime` when
/// [`evaluation_time`] fails, and the conversion errors of
/// [`ExpirationDate::get_years_at`].
#[inline]
pub fn years_to_expiration(expiration: &ExpirationDate) -> Result<Positive, ExpirationDateError> {
    match expiration {
        // The formula of `expiration_date` for a `Days`, which `get_years_at`
        // answers without reading its base; a test pins the two together.
        ExpirationDate::Days(days) => {
            Positive::new(days.to_f64() / DAYS_IN_A_YEAR.to_f64()) // scan-banned: allow -- the formula of `expiration_date`, kept bit for bit so a default build stays numerically identical
                .map_err(Into::into)
        }
        ExpirationDate::DateTime(_) => expiration.get_years_at(evaluation_time()?),
    }
}

/// The fractional days from now to `expiration`.
///
/// Gives what `ExpirationDate::get_days` gives and also exists without the
/// `clock` feature. A `Days` returns its own count, in every build. A `DateTime`
/// is measured from [`evaluation_time`], and is zero once it has passed.
///
/// # Errors
///
/// Returns [`ExpirationDateError::InvalidDateTime`] for a `DateTime` when
/// [`evaluation_time`] fails, and the conversion errors of
/// [`ExpirationDate::get_days_at`].
#[inline]
pub fn days_to_expiration(expiration: &ExpirationDate) -> Result<Positive, ExpirationDateError> {
    match expiration {
        ExpirationDate::Days(days) => Ok(*days),
        ExpirationDate::DateTime(_) => expiration.get_days_at(evaluation_time()?),
    }
}

/// The expiration as a `YYYY-MM-DD` string.
///
/// Gives what `ExpirationDate::get_date_string` gives and also exists without
/// the `clock` feature. A `DateTime` needs no clock. A `Days` counts from 18:30
/// UTC on the day of [`evaluation_time`].
///
/// # Errors
///
/// Returns [`ExpirationDateError::InvalidDateTime`] for a `Days` when
/// [`evaluation_time`] fails, and the overflow errors of
/// [`ExpirationDate::get_date_string_at`].
pub fn expiration_date_string(expiration: &ExpirationDate) -> Result<String, ExpirationDateError> {
    match expiration {
        ExpirationDate::DateTime(datetime) => expiration.get_date_string_at(*datetime),
        ExpirationDate::Days(_) => expiration.get_date_string_at(evaluation_time()?),
    }
}

/// Today's date in the local time zone.
#[cfg(feature = "clock")]
#[inline]
fn local_today() -> Result<NaiveDate, ClockError> {
    Ok(chrono::Local::now().date_naive())
}

/// Today's date in the local time zone: unavailable in a build with no clock.
#[cfg(not(feature = "clock"))]
#[inline]
fn local_today() -> Result<NaiveDate, ClockError> {
    Err(ClockError::Unavailable)
}

/// Represents different timeframes for volatility calculations.
///
/// This enum provides a standardized way to represent various time periods
/// used in financial calculations, including common periods like days, weeks,
/// months, and years, as well as custom periods defined by the user.
///
/// The `TimeFrame` enum is used throughout the library to specify the timeframe
/// for calculations like volatility, returns, and other time-dependent metrics.
///
/// # Examples
///
/// ```
/// use optionstratlib_core::utils::time::TimeFrame;
/// use positive::pos_or_panic;
///
/// // Using standard timeframes
/// let daily = TimeFrame::Day;
/// let weekly = TimeFrame::Week;
///
/// // Using custom timeframes
/// let custom_period = TimeFrame::Custom(pos_or_panic!(360.0));
///
/// // Accessing the number of periods per year
/// let periods_per_year = daily.periods_per_year(); // Returns 252.0
/// let custom_periods = custom_period.periods_per_year(); // Returns 360.0
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub enum TimeFrame {
    /// 1-microsecond data.
    Microsecond,
    /// 1-millisecond data.
    Millisecond,
    /// 1-second data.
    Second,
    /// 1-minute data.
    Minute,
    /// 1-hour data.
    Hour,
    /// Daily data.
    Day,
    /// Weekly data.
    Week,
    /// Monthly data.
    Month,
    /// Quarterly data.
    Quarter,
    /// Yearly data.
    Year,
    /// Custom periods per year.
    Custom(Positive),
}

impl TimeFrame {
    /// Returns the number of periods in a trading year for this timeframe.
    ///
    /// This function calculates the number of periods that occur within a trading year
    /// based on the chosen `TimeFrame`.  A trading year is assumed to have 252 days
    /// and 6.5 trading hours per day.
    ///
    /// For custom timeframes, the number of periods is directly specified by the user.
    ///
    /// # Examples
    ///
    /// ```
    /// use optionstratlib_core::utils::time::TimeFrame;
    /// use positive::pos_or_panic;
    ///
    /// let daily = TimeFrame::Day;
    /// let periods_per_year = daily.periods_per_year(); // Returns 252
    /// assert_eq!(periods_per_year, pos_or_panic!(252.0));
    ///
    /// let hourly = TimeFrame::Hour;
    /// let periods_per_year = hourly.periods_per_year(); // Returns 1638
    /// assert_eq!(periods_per_year, pos_or_panic!(1638.0));
    ///
    /// let custom = TimeFrame::Custom(pos_or_panic!(360.0));
    /// let periods_per_year = custom.periods_per_year(); // Returns 360
    /// assert_eq!(periods_per_year, pos_or_panic!(360.0));
    /// ```
    #[must_use]
    pub fn periods_per_year(&self) -> Positive {
        // The sub-day frames are products of `TRADING_DAYS` (252.0),
        // `TRADING_HOURS` (6.5), `SECONDS_PER_HOUR` (3600.0),
        // `MINUTES_PER_HOUR` (60), `MILLISECONDS_PER_SECOND` (1000) and
        // `MICROSECONDS_PER_SECOND` (1_000_000.0). They are spelled out as
        // literals, with the scale the `Positive` product carried, rather
        // than multiplied here: `Positive * Positive` aborts on overflow and
        // this method has no error channel, while the literals need no
        // arithmetic at all (#788). `test_periods_per_year_matches_products`
        // pins each literal to the product of the constants.
        match self {
            // 252.0 × 6.5 × 3600.0 × 1_000_000.0 microseconds in a trading year
            TimeFrame::Microsecond => pos_lit(dec!(5896800000000.0000)),
            // 252.0 × 6.5 × 3600.0 × 1000 milliseconds in a trading year
            TimeFrame::Millisecond => pos_lit(dec!(5896800000.000)),
            // 252.0 × 6.5 × 3600.0 seconds in a trading year
            TimeFrame::Second => pos_lit(dec!(5896800.000)),
            // 252.0 × 6.5 × 60 minutes in a trading year
            TimeFrame::Minute => pos_lit(dec!(98280.00)),
            // 252.0 × 6.5 hours in a trading year
            TimeFrame::Hour => pos_lit(dec!(1638.00)),
            TimeFrame::Day => *TRADING_DAYS, // Trading days in a year
            TimeFrame::Week => *WEEKS_PER_YEAR, // Weeks in a year
            TimeFrame::Month => *MONTHS_PER_YEAR, // Months in a year
            TimeFrame::Quarter => QUARTERS_PER_YEAR, // Quarters in a year
            TimeFrame::Year => Positive::ONE, // Base unit
            TimeFrame::Custom(periods) => *periods, // Custom periods per year
        }
    }
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeFrame::Microsecond => write!(f, "microsecond"),
            TimeFrame::Millisecond => write!(f, "millisecond"),
            TimeFrame::Second => write!(f, "second"),
            TimeFrame::Minute => write!(f, "minute"),
            TimeFrame::Hour => write!(f, "hour"),
            TimeFrame::Day => write!(f, "day"),
            TimeFrame::Week => write!(f, "week"),
            TimeFrame::Month => write!(f, "month"),
            TimeFrame::Quarter => write!(f, "quarter"),
            TimeFrame::Year => write!(f, "year"),
            TimeFrame::Custom(periods) => write!(f, "custom ({periods})"),
        }
    }
}

/// Returns the number of units per year for each TimeFrame.
///
/// # Arguments
///
/// * `time_frame` - The TimeFrame to get the units per year for
///
/// # Returns
///
/// A `Positive` representing how many of the given time frame fit in a year
fn pos_lit(d: rust_decimal::Decimal) -> Positive {
    Positive::new_decimal(d).unwrap_or(Positive::ZERO)
}

/// Returns how many units of the given `TimeFrame` fit into a calendar year
/// as a `Positive`. Used by annualisation helpers (volatility, yield
/// curves) to scale per-period values to a standardised annual basis.
#[must_use]
pub fn units_per_year(time_frame: &TimeFrame) -> Positive {
    match time_frame {
        TimeFrame::Microsecond => pos_lit(dec!(31536000000000.0)), // 365 * 24 * 60 * 60 * 1_000_000
        TimeFrame::Millisecond => pos_lit(dec!(31536000000.0)),    // 365 * 24 * 60 * 60 * 1_000
        TimeFrame::Second => pos_lit(dec!(31536000.0)),            // 365 * 24 * 60 * 60
        TimeFrame::Minute => pos_lit(dec!(525600.0)),              // 365 * 24 * 60
        TimeFrame::Hour => pos_lit(dec!(8760.0)),                  // 365 * 24
        TimeFrame::Day => pos_lit(dec!(365.0)),                    // 365
        // 365 / 7: the exact `Decimal` quotient `dec!(365.0) / dec!(7.0)`
        // (28 significant digits, scale 27), spelled out so that no division
        // runs here (#788). Exact `Decimal` keeps the round-trip identity
        // Week→Day→Week (an f64 literal would accumulate ~1 ulp of error and
        // break the strict assertion in tests like
        // `test_step_next_with_weeks`). `test_units_per_year_week_is_365_over_7`
        // pins the literal to the quotient.
        TimeFrame::Week => pos_lit(dec!(52.142857142857142857142857143)),
        TimeFrame::Month => pos_lit(dec!(12.0)),  // 12
        TimeFrame::Quarter => pos_lit(dec!(4.0)), // 4
        TimeFrame::Year => Positive::ONE,         // 1
        TimeFrame::Custom(periods) => *periods,   // Custom periods per year
    }
}

/// Converts a value from one TimeFrame to another.
///
/// # Arguments
///
/// * `value` - The value to convert
/// * `from_time_frame` - The source TimeFrame
/// * `to_time_frame` - The target TimeFrame
///
/// # Returns
///
/// A Decimal representing the converted value
///
/// # Errors
///
/// Returns [`PositiveError`] when the source frame has no units in a year
/// (`TimeFrame::Custom(Positive::ZERO)`), so the conversion factor is
/// undefined, and when the factor or the converted value leaves the
/// `Positive` range. Both aborted inside the `Positive` operators before
/// #788.
///
/// # Examples
///
/// ```
///
/// use optionstratlib_core::utils::time::convert_time_frame;
/// use optionstratlib_core::utils::TimeFrame;
/// use positive::{pos_or_panic, Positive, assert_pos_relative_eq};
///
/// // Convert 60 seconds to minutes
/// let result = convert_time_frame(pos_or_panic!(60.0), &TimeFrame::Second, &TimeFrame::Minute)?;
/// assert_pos_relative_eq!(result, Positive::ONE, pos_or_panic!(0.0000001));
///
/// // Convert 12 hours to days
/// let result = convert_time_frame(pos_or_panic!(12.0), &TimeFrame::Hour, &TimeFrame::Day)?;
/// assert_pos_relative_eq!(result, pos_or_panic!(0.5), pos_or_panic!(0.0000001));
/// # Ok::<(), positive::PositiveError>(())
/// ```
pub fn convert_time_frame(
    value: Positive,
    from_time_frame: &TimeFrame,
    to_time_frame: &TimeFrame,
) -> Result<Positive, PositiveError> {
    // If the time frames are the same, return the original value
    if from_time_frame == to_time_frame {
        return Ok(value);
    }

    if value.is_zero() {
        return Ok(Positive::ZERO);
    }

    // Get the units per year for each time frame
    let from_units_per_year = units_per_year(from_time_frame);
    let to_units_per_year = units_per_year(to_time_frame);

    // Calculate the conversion factor
    // The conversion factor is the ratio of units per year
    // For example, to convert from seconds to minutes:
    // seconds per year / minutes per year = 31536000 / 525600 = 60
    // So 60 seconds = 1 minute
    let conversion_factor = to_units_per_year.checked_div(&from_units_per_year)?;
    // Apply the conversion
    value.checked_mul(&conversion_factor)
}

/// Returns tomorrow's date in "dd-mmm-yyyy" format (lowercase).
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "clock")] {
/// use tracing::info;
/// use optionstratlib_core::utils::time::get_tomorrow_formatted;
/// let tomorrow = get_tomorrow_formatted()?;
/// info!("{}", tomorrow); // Output will vary depending on the current date.
/// # }
/// # Ok::<(), optionstratlib_core::model::ExpirationDateError>(())
/// ```
///
/// # Errors
///
/// Returns [`ExpirationDateError::ArithmeticOverflow`] when today is the last
/// date the calendar can hold, and [`ExpirationDateError::InvalidDateTime`] in a
/// build without the `clock` feature; see [`get_x_days_formatted`].
pub fn get_tomorrow_formatted() -> Result<String, ExpirationDateError> {
    get_x_days_formatted(1)
}

/// Formats a date a specified number of days from the current date.
///
/// This function calculates the date that is `days` days from the current date and
/// formats it as a lowercase string in the format "dd-mmm-yyyy".  For example,
/// if the current date is 2024-11-20 and `days` is 1, the returned string will be
/// "21-nov-2024".
///
/// # Arguments
///
/// * `days`: The number of days to offset from the current date.  This can be
///   positive or negative.
///
/// # Returns
///
/// A lowercase string representing the calculated date in "dd-mmm-yyyy" format.
///
/// # Errors
///
/// Returns [`ExpirationDateError::ArithmeticOverflow`] when `days` is beyond
/// the range of a day count (`TimeDelta::days` aborted on it) or the date
/// it lands on is outside the calendar (`NaiveDate + TimeDelta` aborted on
/// it), both before #788, and [`ExpirationDateError::InvalidDateTime`] in a
/// build without the `clock` feature.
pub fn get_x_days_formatted(days: i64) -> Result<String, ExpirationDateError> {
    let today = local_today()?;
    let target = Duration::try_days(days)
        .and_then(|delta| today.checked_add_signed(delta))
        .ok_or_else(|| {
            ExpirationDateError::ArithmeticOverflow(format!(
                "a date {days} days from today is outside the representable calendar range"
            ))
        })?;
    Ok(target.format("%d-%b-%Y").to_string().to_lowercase())
}

/// Returns a formatted date string representing the date `x` days in the future.
///
/// This function takes a `Positive` number of days, calculates the ceiling value
/// as an integer, adds that many days to the current local date, and returns the
/// resulting date formatted in the "dd-MMM-yyyy" format in lowercase.
///
/// # Arguments
///
/// * `days` - A `Positive` value representing a positive number of days.
///
/// # Returns
///
/// A `String` containing the formatted date in lowercase. The format of the date
/// is "dd-MMM-yyyy", where:
/// - `dd` is the day of the month, zero-padded to 2 digits.
/// - `MMM` is the three-letter abbreviated name of the month.
/// - `yyyy` is the 4-digit year.
///
/// # Errors
///
/// Returns [`ExpirationDateError::ArithmeticOverflow`] when the rounded-up day
/// count does not fit an `i64`, does not fit a `chrono` day span, or moves the
/// date outside the calendar range `chrono` can represent. The date is never
/// replaced by today's: a caller asking for a date `days` ahead either gets
/// that date or an error. Returns [`ExpirationDateError::InvalidDateTime`] in a
/// build without the `clock` feature.
///
/// # Note
/// - The function uses the local time zone and the `chrono` crate for date manipulation.
/// - The `Positive` type is expected to provide a `.ceiling()` method that converts it to an integer-compatible representation.
pub fn get_x_days_formatted_pos(days: Positive) -> Result<String, ExpirationDateError> {
    // `checked_ceiling` rather than `ceiling`, which aborts when the result
    // leaves the `Positive` range (#788).
    let ceiling = days
        .checked_ceiling()
        .ok()
        .and_then(|whole| whole.to_i64_checked())
        .ok_or_else(|| unrepresentable_day_offset(days))?;
    let today = local_today()?;
    let target = Duration::try_days(ceiling)
        .and_then(|delta| today.checked_add_signed(delta))
        .ok_or_else(|| unrepresentable_day_offset(days))?;
    Ok(target.format("%d-%b-%Y").to_string().to_lowercase())
}

/// Reports a day offset no calendar date can represent.
#[cold]
#[inline(never)]
fn unrepresentable_day_offset(days: Positive) -> ExpirationDateError {
    ExpirationDateError::ArithmeticOverflow(format!(
        "a date {days} days from today is outside the representable calendar range"
    ))
}

/// Returns the current date formatted as "dd-mmm-yyyy" in lowercase.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "clock")] {
/// use chrono::Local;
/// use optionstratlib_core::utils::time::get_today_formatted;
///
/// let today_formatted = get_today_formatted();
/// let expected_format = Local::now().date_naive().format("%d-%b-%Y").to_string().to_lowercase();
/// assert_eq!(today_formatted, expected_format);
/// # }
/// ```
#[cfg(feature = "clock")]
#[must_use]
pub fn get_today_formatted() -> String {
    let today = chrono::Local::now().date_naive();
    today.format("%d-%b-%Y").to_string().to_lowercase()
}

/// UTC time of day after which [`get_today_or_tomorrow_formatted`] rolls
/// over to the next date.
///
/// `NaiveTime::from_hms_opt` is a `const fn`, so the `match` is evaluated by
/// the compiler: a literal outside chrono's ranges would fail the build with
/// `evaluation of constant value failed`, and no run time reaches the `None`
/// arm (#788), as with `DEFAULT_BINOMIAL_STEPS`.
#[cfg(feature = "clock")]
const TODAY_CUTOFF: NaiveTime = match NaiveTime::from_hms_opt(18, 30, 0) {
    Some(t) => t,
    None => unreachable!(), // scan-banned: allow -- `const` context: an unreachable arm here fails compilation, it cannot abort at run time
};

/// Formats the current date or the next day's date based on the current UTC time.
///
/// The function checks the current UTC time against a cutoff time of 18:30:00.
/// If the current time is past the cutoff, the date for the next day is returned.
/// Otherwise, the current date is returned.  The returned date is formatted
/// as `dd-mmm-yyyy` in lowercase. Note that getting the next day is done safely,
/// handling potential overflow (e.g. the last day of the year).
///
/// Returns:
///
/// A lowercase String representing the formatted date.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "clock")] {
/// use chrono::{Utc, NaiveTime, Timelike};
/// use tracing::info;
/// use optionstratlib_core::utils::time::get_today_or_tomorrow_formatted;
///
/// info!("{}", get_today_or_tomorrow_formatted());
/// # }
/// ```
#[cfg(feature = "clock")]
#[must_use]
pub fn get_today_or_tomorrow_formatted() -> String {
    let cutoff_time = TODAY_CUTOFF;
    let now = wall_clock();
    // Get the date we should use based on current UTC time
    let target_date = if now.time() > cutoff_time {
        now.date_naive()
            .succ_opt()
            .unwrap_or_else(|| now.date_naive()) // Get next day safely
    } else {
        now.date_naive()
    };
    target_date.format("%d-%b-%Y").to_string().to_lowercase()
}

#[cfg(test)]
mod tests_timeframe {
    use super::*;
    use positive::assert_pos_relative_eq;

    #[test]
    fn test_microsecond_periods() {
        let expected =
            *TRADING_DAYS * *TRADING_HOURS * *SECONDS_PER_HOUR * *MICROSECONDS_PER_SECOND;
        assert_eq!(TimeFrame::Microsecond.periods_per_year(), expected);
    }

    #[test]
    fn test_millisecond_periods() {
        let expected = *TRADING_DAYS * *TRADING_HOURS * *SECONDS_PER_HOUR * MILLISECONDS_PER_SECOND;
        assert_eq!(TimeFrame::Millisecond.periods_per_year(), expected);
    }

    #[test]
    fn test_second_periods() {
        let expected = *TRADING_DAYS * *TRADING_HOURS * *SECONDS_PER_HOUR;
        assert_eq!(TimeFrame::Second.periods_per_year(), expected);
    }

    #[test]
    fn test_minute_periods() {
        let expected = *TRADING_DAYS * *TRADING_HOURS * MINUTES_PER_HOUR;
        assert_eq!(TimeFrame::Minute.periods_per_year(), expected);
    }

    #[test]
    fn test_hour_periods() {
        let expected = *TRADING_DAYS * *TRADING_HOURS;
        assert_eq!(TimeFrame::Hour.periods_per_year(), expected);
    }

    #[test]
    fn test_day_periods() {
        assert_eq!(TimeFrame::Day.periods_per_year(), *TRADING_DAYS);
    }

    // The literals replaced `Positive` products (#788); they must carry the
    // same value and the same scale, so the comparison is on the decimal
    // representation, not only on numeric equality.
    #[test]
    fn test_periods_per_year_matches_products() {
        let hours = *TRADING_DAYS * *TRADING_HOURS;
        let seconds = hours * *SECONDS_PER_HOUR;
        let cases = [
            (TimeFrame::Microsecond, seconds * *MICROSECONDS_PER_SECOND),
            (TimeFrame::Millisecond, seconds * MILLISECONDS_PER_SECOND),
            (TimeFrame::Second, seconds),
            (TimeFrame::Minute, hours * MINUTES_PER_HOUR),
            (TimeFrame::Hour, hours),
        ];
        for (frame, product) in cases {
            assert_eq!(
                frame.periods_per_year().to_dec().to_string(),
                product.to_dec().to_string(),
                "{frame}"
            );
        }
    }

    #[test]
    fn test_units_per_year_week_is_365_over_7() {
        assert_eq!(
            units_per_year(&TimeFrame::Week).to_dec().to_string(),
            (dec!(365.0) / dec!(7.0)).to_string()
        );
    }

    #[test]
    fn test_week_periods() {
        assert_eq!(TimeFrame::Week.periods_per_year(), 52.0);
    }

    #[test]
    fn test_month_periods() {
        assert_eq!(TimeFrame::Month.periods_per_year(), 12.0);
    }

    #[test]
    fn test_quarter_periods() {
        assert_eq!(TimeFrame::Quarter.periods_per_year(), 4.0);
    }

    #[test]
    fn test_year_periods() {
        assert_eq!(TimeFrame::Year.periods_per_year(), 1.0);
    }

    #[test]
    fn test_custom_periods() {
        let custom_periods = pos_or_panic!(123.45);
        assert_eq!(
            TimeFrame::Custom(custom_periods).periods_per_year(),
            custom_periods
        );
    }

    #[test]
    fn test_relative_period_relationships() {
        // Test that higher timeframes have fewer periods
        assert!(
            TimeFrame::Microsecond.periods_per_year() > TimeFrame::Millisecond.periods_per_year()
        );
        assert!(TimeFrame::Millisecond.periods_per_year() > TimeFrame::Second.periods_per_year());
        assert!(TimeFrame::Second.periods_per_year() > TimeFrame::Minute.periods_per_year());
        assert!(TimeFrame::Minute.periods_per_year() > TimeFrame::Hour.periods_per_year());
        assert!(TimeFrame::Hour.periods_per_year() > TimeFrame::Day.periods_per_year());
        assert!(TimeFrame::Day.periods_per_year() > TimeFrame::Week.periods_per_year());
        assert!(TimeFrame::Week.periods_per_year() > TimeFrame::Month.periods_per_year());
        assert!(TimeFrame::Month.periods_per_year() > TimeFrame::Quarter.periods_per_year());
        assert!(TimeFrame::Quarter.periods_per_year() > TimeFrame::Year.periods_per_year());
    }

    #[test]
    fn test_specific_conversion_ratios() {
        // Test specific conversion ratios between timeframes
        assert_pos_relative_eq!(
            TimeFrame::Hour.periods_per_year() / TimeFrame::Day.periods_per_year(),
            *TRADING_HOURS,
            pos_or_panic!(1e-10)
        );

        assert_pos_relative_eq!(
            TimeFrame::Minute.periods_per_year() / TimeFrame::Hour.periods_per_year(),
            MINUTES_PER_HOUR,
            pos_or_panic!(1e-10)
        );

        assert_pos_relative_eq!(
            TimeFrame::Second.periods_per_year() / TimeFrame::Minute.periods_per_year(),
            MINUTES_PER_HOUR,
            pos_or_panic!(1e-10)
        );
    }

    #[test]
    fn test_trading_days_relationship() {
        // Verify relationships with trading days
        assert_pos_relative_eq!(
            TimeFrame::Day.periods_per_year(),
            *TRADING_DAYS,
            pos_or_panic!(1e-10)
        );

        assert_pos_relative_eq!(
            TimeFrame::Hour.periods_per_year() / *TRADING_HOURS,
            *TRADING_DAYS,
            pos_or_panic!(1e-10)
        );
    }

    #[test]
    fn test_custom_edge_cases() {
        // Test edge cases for custom periods
        assert_eq!(TimeFrame::Custom(Positive::ZERO).periods_per_year(), 0.0);
        assert_eq!(
            TimeFrame::Custom(Positive::MAX).periods_per_year(),
            Positive::MAX
        );
    }

    #[test]
    fn test_timeframe_debug() {
        assert_eq!(format!("{:?}", TimeFrame::Day), "Day");
        assert_eq!(
            format!("{:?}", TimeFrame::Custom(pos_or_panic!(1.5))),
            "Custom(1.5)"
        );
    }

    #[test]
    fn test_timeframe_clone() {
        let tf = TimeFrame::Day;
        let cloned = tf;
        assert_eq!(tf.periods_per_year(), cloned.periods_per_year());
    }

    #[test]
    fn test_timeframe_copy() {
        let tf = TimeFrame::Day;
        let copied = tf;
        assert_eq!(tf.periods_per_year(), copied.periods_per_year());
    }
}

#[cfg(test)]
mod tests_timeframe_convert {
    use super::*;
    use positive::assert_pos_relative_eq;

    #[test]
    fn test_convert_seconds_to_minutes() {
        let result =
            convert_time_frame(pos_or_panic!(60.0), &TimeFrame::Second, &TimeFrame::Minute)
                .unwrap();
        assert_pos_relative_eq!(result, Positive::ONE, pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_hours_to_days() {
        let result =
            convert_time_frame(pos_or_panic!(12.0), &TimeFrame::Hour, &TimeFrame::Day).unwrap();
        assert_pos_relative_eq!(result, pos_or_panic!(0.5), pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_days_to_weeks() {
        let result =
            convert_time_frame(pos_or_panic!(7.0), &TimeFrame::Day, &TimeFrame::Week).unwrap();
        assert_pos_relative_eq!(result, Positive::ONE, pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_weeks_to_days() {
        let result = convert_time_frame(Positive::TWO, &TimeFrame::Week, &TimeFrame::Day).unwrap();
        assert_pos_relative_eq!(result, pos_or_panic!(14.0), pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_months_to_quarters() {
        let result =
            convert_time_frame(pos_or_panic!(3.0), &TimeFrame::Month, &TimeFrame::Quarter).unwrap();
        assert_pos_relative_eq!(result, Positive::ONE, pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_minutes_to_hours() {
        let result =
            convert_time_frame(pos_or_panic!(120.0), &TimeFrame::Minute, &TimeFrame::Hour).unwrap();
        assert_pos_relative_eq!(result, Positive::TWO, pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_custom_to_day() {
        let result = convert_time_frame(
            pos_or_panic!(10.0),
            &TimeFrame::Custom(pos_or_panic!(365.0)),
            &TimeFrame::Day,
        )
        .unwrap();
        assert_pos_relative_eq!(result, pos_or_panic!(10.0), pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_day_to_custom() {
        let result = convert_time_frame(
            Positive::TWO,
            &TimeFrame::Day,
            &TimeFrame::Custom(pos_or_panic!(365.0)),
        )
        .unwrap();
        assert_pos_relative_eq!(result, Positive::TWO, pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_same_timeframe() {
        let result =
            convert_time_frame(pos_or_panic!(42.0), &TimeFrame::Hour, &TimeFrame::Hour).unwrap();
        assert_pos_relative_eq!(result, pos_or_panic!(42.0), pos_or_panic!(1e-10));
    }

    #[test]
    fn test_convert_weeks_to_months() {
        let result =
            convert_time_frame(pos_or_panic!(4.0), &TimeFrame::Week, &TimeFrame::Month).unwrap();
        // Approximately 0.92 months (4 weeks / 4.33 weeks per month)
        assert_pos_relative_eq!(
            result,
            pos_or_panic!(0.920_547_945_255_920_4),
            pos_or_panic!(1e-10)
        );
    }

    #[test]
    fn test_convert_milliseconds_to_seconds() {
        let result = convert_time_frame(
            pos_or_panic!(1000.0),
            &TimeFrame::Millisecond,
            &TimeFrame::Second,
        )
        .unwrap();
        assert_pos_relative_eq!(result, Positive::ONE, pos_or_panic!(1e-10));
    }

    #[test]
    fn test_zero() {
        let result =
            convert_time_frame(Positive::ZERO, &TimeFrame::Millisecond, &TimeFrame::Second)
                .unwrap();
        assert_pos_relative_eq!(result, Positive::ZERO, pos_or_panic!(1e-10));
    }
}

#[cfg(all(test, feature = "clock"))]
mod tests_x_days_formatted_pos {
    use super::*;

    #[test]
    fn test_get_x_days_formatted_pos_rounds_up_and_matches_the_integer_form() {
        let date = get_x_days_formatted_pos(pos_or_panic!(6.2)).expect("seven days ahead");
        assert_eq!(Some(date), get_x_days_formatted(7).ok());
    }

    #[test]
    fn test_get_x_days_formatted_pos_zero_days_is_today() {
        let date = get_x_days_formatted_pos(Positive::ZERO).expect("today");
        assert_eq!(date, get_today_formatted());
    }

    #[test]
    fn test_get_x_days_formatted_pos_beyond_the_calendar_reports_overflow() {
        // A billion days is a valid `chrono` span but no `NaiveDate` holds it.
        let result = get_x_days_formatted_pos(pos_or_panic!(1_000_000_000.0));
        assert!(matches!(
            result,
            Err(ExpirationDateError::ArithmeticOverflow(_))
        ));
    }

    #[test]
    fn test_get_x_days_formatted_pos_beyond_i64_reports_overflow() {
        let result = get_x_days_formatted_pos(Positive::MAX);
        assert!(matches!(
            result,
            Err(ExpirationDateError::ArithmeticOverflow(_))
        ));
    }
}

#[cfg(all(test, feature = "clock"))]
mod tests_panic_paths {
    use super::*;
    use chrono::{Duration, Local};

    // `to / from` with a zero `from` aborted with `Positive invariant broken
    // in div: result would be non-positive` (#788).
    #[test]
    fn test_convert_time_frame_from_custom_zero_is_error() {
        let result = convert_time_frame(
            Positive::ONE,
            &TimeFrame::Custom(Positive::ZERO),
            &TimeFrame::Day,
        );
        assert!(result.is_err());
    }

    // `value * factor` aborted with `Positive arithmetic overflow in mul`
    // (#788).
    #[test]
    fn test_convert_time_frame_overflow_is_error() {
        let result = convert_time_frame(Positive::MAX, &TimeFrame::Year, &TimeFrame::Day);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_time_frame_values_unchanged() {
        let result = convert_time_frame(Positive::TWO, &TimeFrame::Week, &TimeFrame::Day);
        assert_eq!(
            result.ok(),
            Some(
                Positive::TWO * units_per_year(&TimeFrame::Day) / units_per_year(&TimeFrame::Week)
            )
        );
    }

    // `Duration::days(i64::MAX)` aborted with `TimeDelta::days out of
    // bounds` (#788).
    #[test]
    fn test_get_x_days_formatted_i64_max_is_error() {
        assert!(matches!(
            get_x_days_formatted(i64::MAX),
            Err(ExpirationDateError::ArithmeticOverflow(_))
        ));
        assert!(get_x_days_formatted(i64::MIN).is_err());
    }

    // `NaiveDate + TimeDelta` aborted with `` `NaiveDate + TimeDelta`
    // overflowed `` a billion days out (#788).
    #[test]
    fn test_get_x_days_formatted_billion_days_is_error() {
        assert!(matches!(
            get_x_days_formatted(1_000_000_000),
            Err(ExpirationDateError::ArithmeticOverflow(_))
        ));
    }

    #[test]
    fn test_get_x_days_formatted_values_unchanged() {
        let expected = (Local::now().date_naive() + Duration::days(2))
            .format("%d-%b-%Y")
            .to_string()
            .to_lowercase();
        assert_eq!(get_x_days_formatted(2).ok(), Some(expected));
        let tomorrow = (Local::now().date_naive() + Duration::days(1))
            .format("%d-%b-%Y")
            .to_string()
            .to_lowercase();
        assert_eq!(get_tomorrow_formatted().ok(), Some(tomorrow));
    }
}

#[cfg(test)]
mod tests_clock_helpers {
    use super::*;
    use chrono::TimeZone;

    fn base() -> DateTime<Utc> {
        match Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).single() {
            Some(base) => base,
            None => panic!("2030-01-01 is one instant"),
        }
    }

    /// `years_to_expiration` answers a `Days` without a clock: the answer must
    /// be what `get_years_at` gives, whatever the base, in every build.
    #[test]
    fn test_years_to_expiration_days_matches_get_years_at_at_any_base() {
        for days in [0.0, 0.5, 1.0, 7.0, 30.0, 30.25, 365.0, 3650.0, 1e9] {
            let expiration = ExpirationDate::Days(pos_or_panic!(days));
            let years = years_to_expiration(&expiration).unwrap();
            assert_eq!(years, expiration.get_years_at(base()).unwrap());
            assert_eq!(
                years,
                expiration
                    .get_years_at(base() + Duration::days(777))
                    .unwrap()
            );
        }
    }

    #[test]
    fn test_days_to_expiration_days_is_its_own_count() {
        let expiration = ExpirationDate::Days(pos_or_panic!(30.25));
        assert_eq!(
            days_to_expiration(&expiration).unwrap(),
            pos_or_panic!(30.25)
        );
    }

    #[test]
    fn test_expiration_date_string_datetime_needs_no_clock() {
        let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
        assert_eq!(expiration_date_string(&expiration).unwrap(), "2030-01-31");
    }

    #[test]
    fn test_resolution_base_prefers_the_reference_datetime() {
        ExpirationDate::set_reference_datetime(Some(base()));
        let resolved = resolution_base();
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(resolved, Ok(base()));
    }

    /// With the clock on the helpers give exactly what the `expiration_date`
    /// methods give, so a default build is numerically unchanged.
    #[cfg(feature = "clock")]
    mod with_clock {
        use super::*;

        #[test]
        fn test_days_answers_match_the_clock_reading_methods() {
            for days in [0.0, 0.5, 1.0, 30.0, 30.25, 365.0] {
                let expiration = ExpirationDate::Days(pos_or_panic!(days));
                assert_eq!(
                    years_to_expiration(&expiration).unwrap(),
                    expiration.get_years().unwrap()
                );
                assert_eq!(
                    days_to_expiration(&expiration).unwrap(),
                    expiration.get_days().unwrap()
                );
                assert_eq!(
                    expiration_date_string(&expiration).unwrap(),
                    expiration.get_date_string().unwrap()
                );
            }
        }

        #[test]
        fn test_datetime_answers_match_the_clock_reading_methods() {
            // The two reads are moments apart, so the comparison leaves room for
            // them, and the half day keeps a whole-day count from flipping between.
            let tolerance = 1e-6;
            let expiration =
                ExpirationDate::DateTime(wall_clock() + Duration::days(30) + Duration::hours(12));
            let years = years_to_expiration(&expiration).unwrap().to_f64();
            assert!((years - expiration.get_years().unwrap().to_f64()).abs() < tolerance);
            let days = days_to_expiration(&expiration).unwrap().to_f64();
            assert!((days - expiration.get_days().unwrap().to_f64()).abs() < 1e-3);
            assert_eq!(
                expiration_date_string(&expiration).unwrap(),
                expiration.get_date_string().unwrap()
            );
        }

        #[test]
        fn test_the_reference_datetime_does_not_move_the_evaluation_time() {
            let long_ago = base() - Duration::days(36_500);
            ExpirationDate::set_reference_datetime(Some(long_ago));
            let evaluated = evaluation_time();
            ExpirationDate::set_reference_datetime(None);
            assert!(evaluated.unwrap() > long_ago);
        }

        #[test]
        fn test_now_or_reads_the_clock() {
            let long_ago = base() - Duration::days(365_000);
            assert!(now_or(long_ago) > long_ago);
            assert!(now().is_ok());
        }
    }

    #[cfg(not(feature = "clock"))]
    mod without_clock {
        use super::*;

        #[test]
        fn test_now_is_the_typed_error() {
            assert_eq!(now(), Err(ClockError::Unavailable));
        }

        #[test]
        fn test_now_or_keeps_the_given_instant() {
            assert_eq!(now_or(base()), base());
        }

        /// The caller's instant wins over the reference datetime: `now_or` only
        /// ever gives back what it was handed.
        #[test]
        fn test_now_or_ignores_the_reference_datetime() {
            ExpirationDate::set_reference_datetime(Some(base() + Duration::days(1)));
            let stamped = now_or(base());
            ExpirationDate::set_reference_datetime(None);
            assert_eq!(stamped, base());
        }

        /// A `DateTime` is measured from the reference, so moving the reference
        /// moves the answer: the difference from a build with the clock.
        #[test]
        fn test_evaluation_time_moves_a_datetime_with_the_reference() {
            let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
            let at = |reference: DateTime<Utc>| {
                ExpirationDate::set_reference_datetime(Some(reference));
                let days = days_to_expiration(&expiration);
                ExpirationDate::set_reference_datetime(None);
                days.unwrap()
            };
            assert_eq!(at(base()), pos_or_panic!(30.0));
            assert_eq!(at(base() + Duration::days(10)), pos_or_panic!(20.0));
        }

        #[test]
        fn test_evaluation_time_falls_back_only_to_the_reference_datetime() {
            assert_eq!(evaluation_time(), Err(ClockError::Unavailable));
            assert_eq!(resolution_base(), Err(ClockError::Unavailable));
            ExpirationDate::set_reference_datetime(Some(base()));
            let evaluated = evaluation_time();
            ExpirationDate::set_reference_datetime(None);
            assert_eq!(evaluated, Ok(base()));
        }

        #[test]
        fn test_datetime_without_a_reference_is_invalid_datetime() {
            let expiration = ExpirationDate::DateTime(base());
            let expected = ClockError::Unavailable.to_string();
            for error in [
                years_to_expiration(&expiration).unwrap_err(),
                days_to_expiration(&expiration).unwrap_err(),
            ] {
                assert!(
                    matches!(error, ExpirationDateError::InvalidDateTime(ref text) if *text == expected)
                );
            }
        }

        #[test]
        fn test_datetime_with_a_reference_matches_the_pure_forms() {
            let expiration = ExpirationDate::DateTime(base() + Duration::days(30));
            ExpirationDate::set_reference_datetime(Some(base()));
            let years = years_to_expiration(&expiration);
            let days = days_to_expiration(&expiration);
            ExpirationDate::set_reference_datetime(None);
            assert_eq!(years.unwrap(), expiration.get_years_at(base()).unwrap());
            assert_eq!(days.unwrap(), expiration.get_days_at(base()).unwrap());
        }

        #[test]
        fn test_days_string_needs_a_reference_datetime() {
            let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
            assert!(expiration_date_string(&expiration).is_err());
            ExpirationDate::set_reference_datetime(Some(base()));
            let text = expiration_date_string(&expiration);
            ExpirationDate::set_reference_datetime(None);
            assert_eq!(text.unwrap(), "2030-01-31");
        }

        #[test]
        fn test_local_date_helpers_report_the_missing_clock() {
            assert!(matches!(
                get_tomorrow_formatted(),
                Err(ExpirationDateError::InvalidDateTime(_))
            ));
            assert!(matches!(
                get_x_days_formatted_pos(Positive::ONE),
                Err(ExpirationDateError::InvalidDateTime(_))
            ));
        }
    }
}
