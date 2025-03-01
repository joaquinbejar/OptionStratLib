/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/10/24
******************************************************************************/
use crate::constants::*;
use crate::{Positive, pos};
use chrono::{Duration, Local, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents different timeframes for volatility calculations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum TimeFrame {
    Microsecond,      // 1-microsecond data
    Millisecond,      // 1-millisecond data
    Second,           // 1-second data
    Minute,           // 1-minute data
    Hour,             // 1-hour data
    Day,              // Daily data
    Week,             // Weekly data
    Month,            // Monthly data
    Quarter,          // Quarterly data
    Year,             // Yearly data
    Custom(Positive), // Custom periods per year
}

impl TimeFrame {
    /// Returns the number of periods in a year for this timeframe
    pub fn periods_per_year(&self) -> Positive {
        match self {
            TimeFrame::Microsecond => {
                TRADING_DAYS * TRADING_HOURS * SECONDS_PER_HOUR * MICROSECONDS_PER_SECOND
            } // Microseconds in trading year
            TimeFrame::Millisecond => {
                TRADING_DAYS * TRADING_HOURS * SECONDS_PER_HOUR * MILLISECONDS_PER_SECOND
            } // Milliseconds in trading year
            TimeFrame::Second => TRADING_DAYS * TRADING_HOURS * SECONDS_PER_HOUR, // Seconds in trading year
            TimeFrame::Minute => TRADING_DAYS * TRADING_HOURS * MINUTES_PER_HOUR, // Minutes in trading year
            TimeFrame::Hour => TRADING_DAYS * TRADING_HOURS, // Hours in trading year
            TimeFrame::Day => TRADING_DAYS,                  // Trading days in a year
            TimeFrame::Week => WEEKS_PER_YEAR,               // Weeks in a year
            TimeFrame::Month => MONTHS_PER_YEAR,             // Months in a year
            TimeFrame::Quarter => QUARTERS_PER_YEAR,         // Quarters in a year
            TimeFrame::Year => Positive::ONE,                // Base unit
            TimeFrame::Custom(periods) => *periods,          // Custom periods per year
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
/// A Decimal representing how many of the given time frame fit in a year
pub fn units_per_year(time_frame: &TimeFrame) -> Positive {
    match time_frame {
        TimeFrame::Microsecond => pos!(31536000000000.0), // 365 * 24 * 60 * 60 * 1_000_000
        TimeFrame::Millisecond => pos!(31536000000.0),    // 365 * 24 * 60 * 60 * 1_000
        TimeFrame::Second => pos!(31536000.0),            // 365 * 24 * 60 * 60
        TimeFrame::Minute => pos!(525600.0),              // 365 * 24 * 60
        TimeFrame::Hour => pos!(8760.0),                  // 365 * 24
        TimeFrame::Day => pos!(365.0),                    // 365
        TimeFrame::Week => pos!(52.14285714),             // 365 / 7
        TimeFrame::Month => pos!(12.0),                   // 12
        TimeFrame::Quarter => pos!(4.0),                  // 4
        TimeFrame::Year => pos!(1.0),                     // 1
        TimeFrame::Custom(periods) => *periods,           // Custom periods per year
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
/// # Examples
///
/// ```
/// use optionstratlib::{assert_pos_relative_eq, pos};
/// use optionstratlib::utils::time::convert_time_frame;
/// use optionstratlib::utils::TimeFrame;
///
/// // Convert 60 seconds to minutes
/// let result = convert_time_frame(pos!(60.0), &TimeFrame::Second, &TimeFrame::Minute);
/// assert_pos_relative_eq!(result, pos!(1.0), pos!(0.0000001));
///
/// // Convert 12 hours to days
/// let result = convert_time_frame(pos!(12.0), &TimeFrame::Hour, &TimeFrame::Day);
/// assert_pos_relative_eq!(result, pos!(0.5), pos!(0.0000001));
/// ```
pub fn convert_time_frame(
    value: Positive,
    from_time_frame: &TimeFrame,
    to_time_frame: &TimeFrame,
) -> Positive {
    // If the time frames are the same, return the original value
    if from_time_frame == to_time_frame {
        return value;
    }

    if value.is_zero() {
        return Positive::ZERO;
    }

    // Get the units per year for each time frame
    let from_units_per_year = units_per_year(from_time_frame);
    let to_units_per_year = units_per_year(to_time_frame);

    // Calculate the conversion factor
    // The conversion factor is the ratio of units per year
    // For example, to convert from seconds to minutes:
    // seconds per year / minutes per year = 31536000 / 525600 = 60
    // So 60 seconds = 1 minute
    let conversion_factor = to_units_per_year / from_units_per_year;

    // Apply the conversion
    value * conversion_factor
}

pub fn get_tomorrow_formatted() -> String {
    let tomorrow = Local::now().date_naive() + Duration::days(1);
    tomorrow.format("%d-%b-%Y").to_string().to_lowercase()
}

pub fn get_today_formatted() -> String {
    let today = Local::now().date_naive();
    today.format("%d-%b-%Y").to_string().to_lowercase()
}

pub fn get_today_or_tomorrow_formatted() -> String {
    let cutoff_time = NaiveTime::from_hms_opt(18, 30, 0).unwrap();
    let now = Utc::now();

    // Get the date we should use based on current UTC time
    let target_date = if now.time() > cutoff_time {
        now.date_naive().succ_opt().unwrap_or(now.date_naive()) // Get next day safely
    } else {
        now.date_naive()
    };

    target_date.format("%d-%b-%Y").to_string().to_lowercase()
}

#[cfg(test)]
mod tests_timeframe {
    use super::*;
    use crate::{assert_pos_relative_eq, pos};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_microsecond_periods() {
        let expected = TRADING_DAYS * TRADING_HOURS * SECONDS_PER_HOUR * MICROSECONDS_PER_SECOND;
        assert_eq!(TimeFrame::Microsecond.periods_per_year(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_millisecond_periods() {
        let expected = TRADING_DAYS * TRADING_HOURS * SECONDS_PER_HOUR * MILLISECONDS_PER_SECOND;
        assert_eq!(TimeFrame::Millisecond.periods_per_year(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_second_periods() {
        let expected = TRADING_DAYS * TRADING_HOURS * SECONDS_PER_HOUR;
        assert_eq!(TimeFrame::Second.periods_per_year(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_minute_periods() {
        let expected = TRADING_DAYS * TRADING_HOURS * MINUTES_PER_HOUR;
        assert_eq!(TimeFrame::Minute.periods_per_year(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_hour_periods() {
        let expected = TRADING_DAYS * TRADING_HOURS;
        assert_eq!(TimeFrame::Hour.periods_per_year(), expected);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_day_periods() {
        assert_eq!(TimeFrame::Day.periods_per_year(), TRADING_DAYS);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_week_periods() {
        assert_eq!(TimeFrame::Week.periods_per_year(), 52.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_month_periods() {
        assert_eq!(TimeFrame::Month.periods_per_year(), 12.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_quarter_periods() {
        assert_eq!(TimeFrame::Quarter.periods_per_year(), 4.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_year_periods() {
        assert_eq!(TimeFrame::Year.periods_per_year(), 1.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_custom_periods() {
        let custom_periods = pos!(123.45);
        assert_eq!(
            TimeFrame::Custom(custom_periods).periods_per_year(),
            custom_periods
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_specific_conversion_ratios() {
        // Test specific conversion ratios between timeframes
        assert_pos_relative_eq!(
            TimeFrame::Hour.periods_per_year() / TimeFrame::Day.periods_per_year(),
            TRADING_HOURS,
            pos!(1e-10)
        );

        assert_pos_relative_eq!(
            TimeFrame::Minute.periods_per_year() / TimeFrame::Hour.periods_per_year(),
            MINUTES_PER_HOUR,
            pos!(1e-10)
        );

        assert_pos_relative_eq!(
            TimeFrame::Second.periods_per_year() / TimeFrame::Minute.periods_per_year(),
            MINUTES_PER_HOUR,
            pos!(1e-10)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_trading_days_relationship() {
        // Verify relationships with trading days
        assert_pos_relative_eq!(TimeFrame::Day.periods_per_year(), TRADING_DAYS, pos!(1e-10));

        assert_pos_relative_eq!(
            TimeFrame::Hour.periods_per_year() / TRADING_HOURS,
            TRADING_DAYS,
            pos!(1e-10)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_custom_edge_cases() {
        // Test edge cases for custom periods
        assert_eq!(TimeFrame::Custom(Positive::ZERO).periods_per_year(), 0.0);
        assert_eq!(
            TimeFrame::Custom(Positive::INFINITY).periods_per_year(),
            Positive::INFINITY
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_timeframe_debug() {
        assert_eq!(format!("{:?}", TimeFrame::Day), "Day");
        assert_eq!(format!("{:?}", TimeFrame::Custom(pos!(1.5))), "Custom(1.5)");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_timeframe_clone() {
        let tf = TimeFrame::Day;
        let cloned = tf;
        assert_eq!(tf.periods_per_year(), cloned.periods_per_year());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_timeframe_copy() {
        let tf = TimeFrame::Day;
        let copied = tf;
        assert_eq!(tf.periods_per_year(), copied.periods_per_year());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_pos_relative_eq, pos};

    #[test]
    fn test_convert_seconds_to_minutes() {
        let result = convert_time_frame(pos!(60.0), &TimeFrame::Second, &TimeFrame::Minute);
        assert_pos_relative_eq!(result, pos!(1.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_hours_to_days() {
        let result = convert_time_frame(pos!(12.0), &TimeFrame::Hour, &TimeFrame::Day);
        assert_pos_relative_eq!(result, pos!(0.5), pos!(1e-10));
    }

    #[test]
    fn test_convert_days_to_weeks() {
        let result = convert_time_frame(pos!(7.0), &TimeFrame::Day, &TimeFrame::Week);
        assert_pos_relative_eq!(result, pos!(1.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_months_to_quarters() {
        let result = convert_time_frame(pos!(3.0), &TimeFrame::Month, &TimeFrame::Quarter);
        assert_pos_relative_eq!(result, pos!(1.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_minutes_to_hours() {
        let result = convert_time_frame(pos!(120.0), &TimeFrame::Minute, &TimeFrame::Hour);
        assert_pos_relative_eq!(result, pos!(2.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_custom_to_day() {
        let result =
            convert_time_frame(pos!(10.0), &TimeFrame::Custom(pos!(365.0)), &TimeFrame::Day);
        assert_pos_relative_eq!(result, pos!(10.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_day_to_custom() {
        let result =
            convert_time_frame(pos!(2.0), &TimeFrame::Day, &TimeFrame::Custom(pos!(365.0)));
        assert_pos_relative_eq!(result, pos!(2.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_same_timeframe() {
        let result = convert_time_frame(pos!(42.0), &TimeFrame::Hour, &TimeFrame::Hour);
        assert_pos_relative_eq!(result, pos!(42.0), pos!(1e-10));
    }

    #[test]
    fn test_convert_weeks_to_months() {
        let result = convert_time_frame(pos!(4.0), &TimeFrame::Week, &TimeFrame::Month);
        // Approximately 0.92 months (4 weeks / 4.33 weeks per month)
        assert_pos_relative_eq!(result, pos!(0.920_547_945_255_920_4), pos!(1e-10));
    }

    #[test]
    fn test_convert_milliseconds_to_seconds() {
        let result = convert_time_frame(pos!(1000.0), &TimeFrame::Millisecond, &TimeFrame::Second);
        assert_pos_relative_eq!(result, pos!(1.0), pos!(1e-10));
    }

    #[test]
    fn test_zero() {
        let result =
            convert_time_frame(Positive::ZERO, &TimeFrame::Millisecond, &TimeFrame::Second);
        assert_pos_relative_eq!(result, Positive::ZERO, pos!(1e-10));
    }
}
