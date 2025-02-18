/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/10/24
******************************************************************************/
use crate::constants::*;
use chrono::{Duration, Local, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use crate::Positive;

/// Represents different timeframes for volatility calculations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum TimeFrame {
    Microsecond, // 1-microsecond data
    Millisecond, // 1-millisecond data
    Second,      // 1-second data
    Minute,      // 1-minute data
    Hour,        // 1-hour data
    Day,         // Daily data
    Week,        // Weekly data
    Month,       // Monthly data
    Quarter,     // Quarterly data
    Year,        // Yearly data
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
            TimeFrame::Year => Positive::ONE,                          // Base unit
            TimeFrame::Custom(periods) => *periods,          // Custom periods per year
        }
    }
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
        assert_pos_relative_eq!(
            TimeFrame::Day.periods_per_year(),
            TRADING_DAYS,
            pos!(1e-10)
        );

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
