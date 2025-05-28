use crate::constants::{DAYS_IN_A_YEAR, EPSILON};
use crate::{Positive, pos};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Represents the expiration of an option contract or financial instrument.
///
/// This enum allows for two different ways to specify when something expires:
/// - As a number of days from the current date
/// - As a specific date and time
///
/// `ExpirationDate` is used throughout the options modeling system to handle
/// time-based calculations such as time decay (theta) and option valuation.
#[derive(Clone, Copy)]
pub enum ExpirationDate {
    /// Represents expiration as a positive number of days from the current date.
    /// This is typically used for relative time specifications.
    /// when converting between Days and DateTime variants.
    Days(Positive),

    /// Represents expiration as an absolute point in time using UTC datetime.
    /// This is used when a precise expiration moment is known.
    DateTime(DateTime<Utc>),
}

impl Hash for ExpirationDate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ExpirationDate::Days(days) => {
                0.hash(state); // Variant discriminant
                days.hash(state);
            }
            ExpirationDate::DateTime(datetime) => {
                1.hash(state); // Variant discriminant
                datetime.timestamp().hash(state);
                datetime.timestamp_subsec_nanos().hash(state);
            }
        }
    }
}

impl PartialEq for ExpirationDate {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpirationDate::Days(a), ExpirationDate::Days(b)) => (a.0 - b.0).abs() < EPSILON,
            (ExpirationDate::DateTime(a), ExpirationDate::DateTime(b)) => a == b,
            _ => false, // Different variants are never equal
        }
    }
}

impl Eq for ExpirationDate {}

impl PartialOrd for ExpirationDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExpirationDate {
    fn cmp(&self, other: &Self) -> Ordering {
        // We know get_days() should never fail for valid expiration dates,
        // so we can unwrap safely in this implementation.
        let self_days = self.get_days().unwrap_or(Positive::ZERO);
        let other_days = other.get_days().unwrap_or(Positive::ZERO);

        self_days.cmp(&other_days)
    }
}

impl ExpirationDate {
    /// Calculates the time to expiration in years.
    ///
    /// Returns a `Result<Positive, Box<dyn Error>>`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The `ExpirationDate` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the `DateTime` variant results in a negative duration
    /// indicating the expiration date is in the past.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::{assert_pos_relative_eq, pos, ExpirationDate};
    ///
    /// let days = pos!(365.0);
    /// let expiration_date_days = ExpirationDate::Days(days);
    /// let years = expiration_date_days.get_years().unwrap();
    /// assert_pos_relative_eq!(years, pos!(1.0), pos!(0.001));
    ///
    /// let datetime = Utc::now() + Duration::days(365);
    /// let expiration_date_datetime = ExpirationDate::DateTime(datetime);
    /// let years = expiration_date_datetime.get_years().unwrap();
    /// assert_pos_relative_eq!(years, pos!(1.0), pos!(0.001));
    /// ```
    pub fn get_years(&self) -> Result<Positive, Box<dyn Error>> {
        let days = self.get_days()?;
        Ok(pos!(days.to_f64() / DAYS_IN_A_YEAR))
    }

    /// Calculates the number of days until expiration for this `ExpirationDate` instance.
    ///
    /// This method converts both variants of `ExpirationDate` to a common representation:
    /// the number of days until expiration. This is useful for calculations that need
    /// time-to-expiry in a standardized format.
    ///
    /// # Returns
    ///
    /// * `Result<Positive, Box<dyn Error>>` - A `Positive` value representing the number of days
    ///   until expiration, or an error if the calculation fails.
    ///
    /// # Details
    ///
    /// * For `ExpirationDate::Days` variant: Returns the stored days value directly.
    /// * For `ExpirationDate::DateTime` variant: Calculates the difference between the stored
    ///   datetime and the current time, converting it to days.
    ///
    /// If the calculation results in zero or negative days (meaning the expiration date
    /// is in the past), the method returns `Positive::ZERO` to indicate immediate expiration.
    ///
    pub fn get_days(&self) -> Result<Positive, Box<dyn Error>> {
        match self {
            ExpirationDate::Days(days) => Ok(*days),
            ExpirationDate::DateTime(datetime) => {
                // Store the original datetime as reference for future use
                Self::set_reference_datetime(Some(*datetime));

                let now = Utc::now();
                let duration = datetime.signed_duration_since(now);
                let num_days = duration.num_seconds() as f64 / (24.0 * 60.0 * 60.0);
                if num_days <= 0.0 {
                    return Ok(Positive::ZERO);
                }
                Ok(pos!(num_days))
            }
        }
    }

    /// Returns the expiration date as a `DateTime<Utc>`.
    ///
    /// For the `Days` variant, it calculates the date by adding the specified number of days to the current date and time.
    /// For the `DateTime` variant, it returns the stored `DateTime<Utc>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::{pos, ExpirationDate};
    ///
    /// let days = pos!(30.0);
    /// let expiration_date_days = ExpirationDate::Days(days);
    /// let future_date = Utc::now() + Duration::days(30);
    /// let calculated_date = expiration_date_days.get_date().unwrap();
    /// // Check if dates are within a small tolerance (due to potential time differences during test)
    /// assert_eq!(calculated_date.date_naive(), future_date.date_naive());
    ///
    /// let datetime = Utc::now() + Duration::days(365);
    /// let expiration_date_datetime = ExpirationDate::DateTime(datetime);
    /// let stored_date = expiration_date_datetime.get_date().unwrap();
    /// assert_eq!(stored_date, datetime);
    /// ```
    pub fn get_date(&self) -> Result<DateTime<Utc>, Box<dyn Error>> {
        self.get_date_with_options(false)
    }

    // Thread-local storage to store reference datetime for Days variant
    thread_local! {
        static REFERENCE_DATETIME: std::cell::RefCell<Option<DateTime<Utc>>> = const { std::cell::RefCell::new(None) };
    }

    /// Retrieves the reference `DateTime` stored in a thread-local storage.
    ///
    /// This function accesses a thread-local variable `REFERENCE_DATETIME` using a provided
    /// closure to borrow its value. The method returns the value as an `Option<DateTime<Utc>>`,
    /// which may contain a valid `DateTime<Utc>` or `None` if no reference datetime is set.
    ///
    /// # Returns
    /// - `Some(DateTime<Utc>)`: If a reference datetime is currently stored in the thread-local storage.
    /// - `None`: If no reference datetime is set in the storage.
    ///
    ///
    /// # Note
    /// This function operates on thread-local storage (TLS), ensuring that the state is specific
    /// to the thread which invokes it. Changes to the `REFERENCE_DATETIME` in one thread will not
    /// affect its value in another thread.
    pub(crate) fn get_reference_datetime() -> Option<DateTime<Utc>> {
        let mut result = None;
        Self::REFERENCE_DATETIME.with(|cell| {
            result = *cell.borrow();
        });
        result
    }

    /// Sets the reference datetime for the current context.
    ///
    /// This function updates an internal thread-local storage with the given optional `DateTime<Utc>`.
    /// The `dt` parameter can either be a `Some(DateTime<Utc>)` to set a specific datetime
    /// or `None` to clear the reference datetime.
    ///
    /// # Parameters
    /// - `dt`: An `Option<DateTime<Utc>>` representing the datetime to set. If `Some(dt)` is provided,
    ///   it will overwrite the current reference datetime. If `None` is provided, the reference datetime
    ///   will be cleared.
    ///
    /// # Panics
    /// This function will panic if the thread-local storage cannot be borrowed mutably or if there are
    /// existing immutable references to it at the time of this call.
    ///
    pub(crate) fn set_reference_datetime(dt: Option<DateTime<Utc>>) {
        Self::REFERENCE_DATETIME.with(|cell| {
            *cell.borrow_mut() = dt;
        });
    }

    /// Calculates and returns a `DateTime<Utc>` based on the specified options and expiration criteria.
    ///
    /// # Parameters
    /// - `use_fixed_time`:
    ///   - If `true`, a fixed daily time of 18:30 UTC is used as the base time for calculations.
    ///   - If `false`, the base time for calculations depends on a reference datetime if available.
    ///     If no reference datetime exists, the calculation will use the current time.
    ///
    /// # Returns
    /// - `Ok(DateTime<Utc>)`:
    ///   - If the expiration date can be successfully calculated based on the provided options and
    ///     stored expiration criteria (`Days` or `DateTime`).
    /// - `Err(Box<dyn Error>)`:
    ///   - If there is an invalid time conversion or inconsistency in the configuration.
    ///
    /// # Behavior
    /// This function handles two expiration types:
    ///
    /// 1. **`ExpirationDate::Days`**:
    ///    - If `use_fixed_time` is `true`:
    ///      - Takes today's date and sets the time to 18:30 UTC as the base datetime.
    ///      - Adds the specified number of days from the `Days` variant to this fixed datetime.
    ///    - If `use_fixed_time` is `false`:
    ///      - Uses a stored reference datetime (`get_reference_datetime`, if available) as the base datetime,
    ///        and adds the number of days from the `Days` variant.
    ///      - If no reference datetime is found, uses the current time as the base datetime.
    /// 2. **`ExpirationDate::DateTime`**:
    ///    - Directly returns the pre-stored datetime associated with this variant.
    ///
    /// # Errors
    /// - Returns an error if a fixed time (18:30 UTC) cannot be correctly configured.
    /// - Returns an error if any internal inconsistency occurs (e.g., invalid conversions).
    pub fn get_date_with_options(
        &self,
        use_fixed_time: bool,
    ) -> Result<DateTime<Utc>, Box<dyn Error>> {
        match self {
            ExpirationDate::Days(days) => {
                if use_fixed_time {
                    // Get today's date at 18:30 UTC (original behavior)
                    let today = Utc::now().date_naive();
                    let fixed_time = today.and_hms_opt(18, 30, 0).ok_or("Invalid time")?;
                    let fixed_datetime =
                        DateTime::<Utc>::from_naive_utc_and_offset(fixed_time, Utc);
                    Ok(fixed_datetime + Duration::days((*days).to_i64()))
                } else {
                    // Check if we have a reference datetime stored
                    if let Some(ref_dt) = Self::get_reference_datetime() {
                        // Use the reference datetime and add the days
                        Ok(ref_dt + Duration::days((*days).to_i64()))
                    } else {
                        // Fallback to current time if no reference is stored
                        let now = Utc::now();
                        Ok(now + Duration::days((*days).to_i64()))
                    }
                }
            }
            ExpirationDate::DateTime(datetime) => Ok(*datetime),
        }
    }

    /// Returns the expiration date as a formatted string in `YYYY-MM-DD` format.
    ///
    /// This method calls `get_date()` to retrieve the `DateTime<Utc>` and then formats it into the specified string format.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::{pos, ExpirationDate};
    ///
    /// let days = pos!(30.0);
    /// let expiration_date = ExpirationDate::Days(days);
    /// let date_string = expiration_date.get_date_string().unwrap();
    /// assert!(date_string.len() == 10); // YYYY-MM-DD format
    /// ```
    pub fn get_date_string(&self) -> Result<String, Box<dyn Error>> {
        // Use fixed time for backward compatibility with existing tests
        let date = self.get_date_with_options(true)?;
        Ok(date.format("%Y-%m-%d").to_string())
    }

    /// Creates an `ExpirationDate` from a string.
    ///
    /// This function attempts to parse the input string `s` into an `ExpirationDate`. It supports various formats, including:
    ///
    /// 1. **Positive number of days:** Parses the string as a `Positive` number, representing days from now.
    /// 2. **RFC3339 DateTime:** Parses the string as an RFC3339 compliant date and time string.
    /// 3. **Numeric Date (YYYYMMDD):** Parses an 8-digit numeric string as year, month, and day. Sets the time to 23:59:59.
    /// 4. **Common Date formats:** Parses various common date formats (e.g., DD-MM-YYYY, DD MMM YYYY, etc.). Sets the time to 23:59:59.
    ///
    /// If none of the above formats can be parsed successfully, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `s` - The input string to parse.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use chrono::{DateTime, Utc};
    /// use rust_decimal_macros::dec;
    /// use tracing::info;
    /// use optionstratlib::{pos, ExpirationDate};
    ///
    /// let expiration_date_days = ExpirationDate::from_string("365").unwrap();
    /// assert_eq!(expiration_date_days, ExpirationDate::Days(pos!(365.0)));
    ///
    /// let rfc3339_string = "2025-01-01T12:00:00Z";
    /// let expiration_date_rfc3339 = ExpirationDate::from_string(rfc3339_string).unwrap();
    /// let datetime = DateTime::parse_from_rfc3339(rfc3339_string).unwrap();
    /// assert_eq!(expiration_date_rfc3339, ExpirationDate::DateTime(DateTime::from(datetime)));
    ///
    /// let numeric_date_string = "20250101";
    /// let expiration_date_numeric = ExpirationDate::from_string(numeric_date_string).unwrap();
    /// if let ExpirationDate::DateTime(dt) = expiration_date_numeric {
    ///     assert_eq!(dt.format("%Y%m%d").to_string(), numeric_date_string);
    /// } else {
    ///     info!("Expected ExpirationDate::DateTime");
    /// }
    ///
    ///
    /// let common_date_string = "01-01-2025";
    /// let expiration_date_common = ExpirationDate::from_string(common_date_string).unwrap();
    /// if let ExpirationDate::DateTime(dt) = expiration_date_common {
    ///     assert_eq!(dt.format("%d-%m-%Y").to_string(), common_date_string);
    /// } else {
    ///     info!("Expected ExpirationDate::DateTime");
    /// }
    /// ```
    pub fn from_string(s: &str) -> Result<Self, Box<dyn Error>> {
        // First try parsing as Positive (days)
        if let Ok(days) = s.parse::<Positive>() {
            return Ok(ExpirationDate::Days(days));
        }

        // Try to parse as a date only
        if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(18, 30, 0).ok_or("Invalid time")?;
            let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
            // Store the datetime as reference
            Self::set_reference_datetime(Some(utc_dt));
            return Ok(ExpirationDate::DateTime(utc_dt));
        }

        // Try to parse as a date with time and timezone
        if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %Z") {
            let utc_dt = dt.with_timezone(&Utc);
            // Store the datetime as reference
            Self::set_reference_datetime(Some(utc_dt));
            return Ok(ExpirationDate::DateTime(utc_dt));
        }

        // Try parsing format "2025-05-23 12:03:18 UTC"
        if s.contains(" UTC") && s.contains(":") {
            // Intentar varios formatos para el patrón con UTC
            for format in ["%Y-%m-%d %H:%M:%S %Z", "%Y-%m-%d %H:%M:%S UTC"] {
                if let Ok(datetime) = DateTime::parse_from_str(s, format) {
                    let utc_dt = DateTime::from(datetime);
                    // Store the datetime as reference
                    Self::set_reference_datetime(Some(utc_dt));
                    return Ok(ExpirationDate::DateTime(utc_dt));
                }
            }

            // Si los formatos anteriores fallan, intentar construir manualmente
            if s.contains(" UTC") {
                // Extract the date and time part without the UTC suffix
                let date_time_part = s.trim_end_matches(" UTC").trim();

                // Try to parse as a date with time
                if let Ok(dt) = NaiveDateTime::parse_from_str(date_time_part, "%Y-%m-%d %H:%M:%S") {
                    let utc_dt = DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc);
                    // Store the datetime as reference
                    Self::set_reference_datetime(Some(utc_dt));
                    return Ok(ExpirationDate::DateTime(utc_dt));
                }
            }
        }

        // Try parsing format "2025-05-23T15:29" (without seconds)
        if s.contains('T') && s.matches(':').count() == 1 {
            // Añadir segundos y zona horaria si no están presentes
            let datetime_str = format!("{s}:00Z");
            if let Ok(datetime) = DateTime::parse_from_rfc3339(&datetime_str) {
                let utc_dt = DateTime::from(datetime);
                // Store the datetime as reference
                Self::set_reference_datetime(Some(utc_dt));
                return Ok(ExpirationDate::DateTime(utc_dt));
            }
        }

        // Try numeric date formats first
        if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            // Format: YYYYMMDD
            let year = s[0..4].parse::<i32>()?;
            let month = s[4..6].parse::<u32>()?;
            let day = s[6..8].parse::<u32>()?;

            if let Some(naive_datetime) = NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|date| date.and_hms_opt(23, 59, 59))
            {
                let datetime = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                return Ok(ExpirationDate::DateTime(datetime));
            }
        }

        // Try parsing common date formats, including ISO format
        let formats = [
            "%Y-%m-%d", // "2024-01-01"
            "%d-%m-%Y", // "01-01-2025"
            "%d %b %Y", // "30 jan 2025"
            "%d-%b-%Y", // "30-jan-2025"
            "%d %B %Y", // "30 january 2025"
            "%d-%B-%Y", // "30-january-2025"
        ];

        for format in formats {
            if let Ok(naive_date) = NaiveDate::parse_from_str(s.to_lowercase().as_str(), format) {
                // Convert NaiveDate to DateTime<Utc> by setting time to end of day
                let naive_datetime = naive_date
                    .and_hms_opt(18, 30, 00)
                    .ok_or_else(|| format!("Invalid time conversion for date: {s}"))?;

                let datetime = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                return Ok(ExpirationDate::DateTime(datetime));
            }
        }

        // If none of the above worked, return error
        Err(format!("Failed to parse ExpirationDate from string: {s}").into())
    }

    /// Converts a string representation of an expiration date into a `Days` variant of `ExpirationDate`.
    ///
    /// # Arguments
    /// * `s` - A string slice representing the expiration date. The string should adhere
    ///   to a format that `ExpirationDate::from_string` can parse.
    ///
    /// # Returns
    /// * `Ok(Self)` - If the string is successfully parsed and converted into days.
    ///   The result is an `ExpirationDate::Days` variant containing the
    ///   floored number of days.
    /// * `Err(Box<dyn Error>)` - If parsing or conversion fails, an error wrapped in a
    ///   `Box<dyn Error>` is returned.
    ///
    /// # Errors
    /// This function may return an error in the following cases:
    /// * The provided string `s` is not in a valid or expected format.
    /// * The computed number of days could not be retrieved from the parsed expiration date.
    ///
    /// # Note
    /// The function assumes that `floor()` truncates any remaining fractional days.
    pub fn from_string_to_days(s: &str) -> Result<Self, Box<dyn Error>> {
        // Try to parse as a date
        let date_result = Self::from_string(s);
        if let Ok(expiration_date) = date_result {
            // Convert to days
            let days = expiration_date.get_days()?;
            // The get_days method will have stored the reference datetime if it was a DateTime variant
            return Ok(ExpirationDate::Days(days));
        }

        // If parsing as a date fails, try parsing as a number of days directly
        if let Ok(days) = s.parse::<Positive>() {
            // Clear any stored reference datetime since we're creating a Days variant directly
            Self::set_reference_datetime(None);
            return Ok(ExpirationDate::Days(days));
        }

        // If all parsing attempts fail, return an error
        Err("Failed to parse expiration date".into())
    }
}

impl Default for ExpirationDate {
    fn default() -> Self {
        ExpirationDate::Days(pos!(365.0))
    }
}

impl Serialize for ExpirationDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ExpirationDate::Days(days) => {
                let mut state = serializer.serialize_map(Some(1))?;
                state.serialize_entry("days", &days.to_f64())?;
                state.end()
            }
            ExpirationDate::DateTime(dt) => {
                let mut state = serializer.serialize_map(Some(1))?;
                state.serialize_entry("datetime", &dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ExpirationDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        enum Field {
            days,
            datetime,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl Visitor<'_> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`days` or `datetime`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "days" => Ok(Field::days),
                            "datetime" => Ok(Field::datetime),
                            _ => Err(serde::de::Error::unknown_field(
                                value,
                                &["days", "datetime"],
                            )),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ExpirationDateVisitor;

        impl<'de> Visitor<'de> for ExpirationDateVisitor {
            type Value = ExpirationDate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ExpirationDate")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ExpirationDate, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut days: Option<Positive> = None;
                let mut datetime: Option<DateTime<Utc>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::days => {
                            if days.is_some() {
                                return Err(serde::de::Error::duplicate_field("days"));
                            }
                            let value: f64 = map.next_value()?;
                            days = Some(pos!(value));
                        }
                        Field::datetime => {
                            if datetime.is_some() {
                                return Err(serde::de::Error::duplicate_field("datetime"));
                            }
                            let value: String = map.next_value()?;
                            datetime = Some(
                                DateTime::parse_from_rfc3339(&value)
                                    .map_err(serde::de::Error::custom)?
                                    .with_timezone(&Utc),
                            );
                        }
                    }
                }

                if let Some(days) = days {
                    Ok(ExpirationDate::Days(days))
                } else if let Some(datetime) = datetime {
                    Ok(ExpirationDate::DateTime(datetime))
                } else {
                    Err(serde::de::Error::missing_field("either days or datetime"))
                }
            }
        }

        const FIELDS: &[&str] = &["days", "datetime"];
        deserializer.deserialize_struct("ExpirationDate", FIELDS, ExpirationDateVisitor)
    }
}

#[cfg(test)]
mod tests_expiration_date {
    use super::*;
    use crate::constants::{DAYS_IN_A_YEAR, ZERO};
    use chrono::Duration;

    #[test]
    fn test_expiration_date_days() {
        let expiration = ExpirationDate::Days(DAYS_IN_A_YEAR);
        assert_eq!(expiration.get_years().unwrap(), 1.0);

        let expiration = ExpirationDate::Days(pos!(182.5));
        assert_eq!(expiration.get_years().unwrap(), 0.5);

        let expiration = ExpirationDate::Days(Positive::ZERO);
        assert_eq!(expiration.get_years().unwrap(), ZERO);
    }

    #[test]
    fn test_expiration_date_datetime() {
        // Test for a date exactly one year in the future
        let one_year_future = Utc::now() + Duration::days(365);
        let expiration = ExpirationDate::DateTime(one_year_future);
        assert!((expiration.get_years().unwrap().to_f64() - 1.0).abs() < 0.01); // Allow small deviation due to leap years

        // Test for a date 6 months in the future
        let six_months_future = Utc::now() + Duration::days(182);
        let expiration = ExpirationDate::DateTime(six_months_future);
        assert!((expiration.get_years().unwrap().to_f64() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_expiration_date_datetime_specific() {
        // Test with a specific date
        // let specific_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let specific_date = Utc::now() + Duration::days(1);
        let expiration = ExpirationDate::DateTime(specific_date);
        assert!(expiration.get_years().unwrap() > Positive::ZERO);
    }

    #[test]
    fn test_get_date_from_datetime() {
        let future_date = Utc::now() + Duration::days(60);
        let expiration = ExpirationDate::DateTime(future_date);
        let result = expiration.get_date().unwrap();

        assert_eq!(result, future_date);
    }

    #[test]
    fn test_get_date_from_past_datetime() {
        let past_date = Utc::now() - Duration::days(30);
        let expiration = ExpirationDate::DateTime(past_date);
        let result = expiration.get_date().unwrap();
        assert_eq!(result, past_date);
    }

    #[test]
    fn test_positive_days() {
        let expiration = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let years = expiration.get_years().unwrap();
        assert_eq!(years, 1.0);
    }

    #[test]
    fn test_comparisons() {
        let one_day = ExpirationDate::Days(Positive::ONE);
        let less_than_one_day = ExpirationDate::Days(pos!(0.99));

        assert!(less_than_one_day < one_day);

        let now = Utc::now();
        let future = now + Duration::days(1);
        let past = now - Duration::days(1);
        let future_date = ExpirationDate::DateTime(future);
        let past_date = ExpirationDate::DateTime(past);

        assert!(future_date > past_date);

        let ten_days = ExpirationDate::Days(Positive::TEN);
        let tomorrow = Utc::now() + Duration::days(1);
        let tomorrow_date = ExpirationDate::DateTime(tomorrow);
        assert!(tomorrow_date < ten_days);
    }

    #[cfg(test)]
    mod tests_expiration_date_formatting {
        use super::*;
        use chrono::TimeZone;

        #[test]
        fn test_get_date_string_days() {
            let today = Utc::now();
            let expiration = ExpirationDate::Days(pos!(30.0));
            let date_str = expiration.get_date_string().unwrap();
            let expected_date = (today + Duration::days(30)).format("%Y-%m-%d").to_string();
            assert_eq!(date_str, expected_date);
        }

        #[test]
        fn test_get_date_string_datetime() {
            let specific_date = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
            let expiration = ExpirationDate::DateTime(specific_date);
            assert_eq!(expiration.get_date_string().unwrap(), "2024-12-31");
        }
    }
}

#[cfg(test)]
mod test_expiration_date {
    use crate::model::ExpirationDate;
    use crate::utils::time::get_today_formatted;
    use crate::{assert_pos_relative_eq, pos};
    use chrono::{Local, Timelike, Utc};

    #[test]
    fn test_from_string_valid_days() {
        let result = ExpirationDate::from_string("30.0");
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected Days variant"),
        }
    }

    #[test]
    fn test_from_string_passed_datetime() {
        let result = ExpirationDate::from_string("2024-12-31T00:00:00Z");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_string_format_one() {
        let result = ExpirationDate::from_string("30 jan 2025");
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_string_format_two() {
        let result = ExpirationDate::from_string("30-jan-2025");
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_string_format_three() {
        let result = ExpirationDate::from_string("20250101");
        assert!(result.is_ok());
    }

    #[test]
    fn four() {
        let result = ExpirationDate::from_string("30-01-2025");
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_string_invalid_format() {
        let result = ExpirationDate::from_string("invalid date");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_string_format_today() {
        let today = get_today_formatted();
        let result = ExpirationDate::from_string(&today);
        assert!(result.is_ok());
        let expiration = result.unwrap();
        assert!(expiration.get_date_string().is_ok());

        let today = Local::now().date_naive();
        assert_eq!(
            expiration.get_date_string().unwrap(),
            today.format("%Y-%m-%d").to_string()
        );

        // Get current UTC time
        let current_utc_time = Utc::now().time();
        let years = expiration.get_years().unwrap();

        // Check years based on current UTC time
        if current_utc_time
            < Utc::now()
                .date_naive()
                .and_hms_opt(18, 30, 0)
                .unwrap()
                .time()
        {
            // Before 18:30 UTC
            assert!(
                years > 0.0,
                "Years should be greater than 0 before 18:30 UTC"
            );
        } else {
            // After 18:30 UTC
            assert_eq!(years.to_f64(), 0.0, "Years should be 0 after 18:30 UTC");
        }

        assert!(expiration.get_date_string().is_ok());

        // Get the date
        let date = expiration.get_date().unwrap();

        // Additional checks for the date components
        assert_eq!(date.hour(), 18, "Hour should be 18");
        assert_eq!(date.minute(), 30);
        assert_eq!(date.second(), 0);

        assert!(expiration.get_date_string().is_ok());
    }

    #[test]
    fn test_from_expiration_date_zero() {
        let zero_expiration_date = ExpirationDate::Days(pos!(0.0));

        let today = Local::now().date_naive();
        assert_eq!(
            zero_expiration_date.get_date_string().unwrap(),
            today.format("%Y-%m-%d").to_string()
        );
        let years = zero_expiration_date.get_years().unwrap();
        assert_pos_relative_eq!(years, pos!(0.0), pos!(1e-3));

        assert!(zero_expiration_date.get_date_string().is_ok());

        // Get the date with fixed time (18:30 UTC)
        let date = zero_expiration_date.get_date_with_options(true).unwrap();

        // Additional checks for the date components
        assert_eq!(date.hour(), 18, "Hour should be 18");
        assert_eq!(date.minute(), 30);
        assert_eq!(date.second(), 0);

        assert!(zero_expiration_date.get_date_string().is_ok());
    }

    #[test]
    fn test_from_expiration_date_almost_zero() {
        let zero_expiration_date = ExpirationDate::Days(pos!(0.5));
        let today = Local::now().date_naive();
        assert_eq!(
            zero_expiration_date.get_date_string().unwrap(),
            today.format("%Y-%m-%d").to_string()
        );
        let years = zero_expiration_date.get_years().unwrap();
        assert_pos_relative_eq!(years, pos!(0.001369), pos!(1e-3));

        assert!(zero_expiration_date.get_date_string().is_ok());

        // Get the date with fixed time (18:30 UTC)
        let date = zero_expiration_date.get_date_with_options(true).unwrap();

        // Additional checks for the date components
        assert_eq!(date.hour(), 18, "Hour should be 18");
        assert_eq!(date.minute(), 30);
        assert_eq!(date.second(), 0);

        assert!(zero_expiration_date.get_date_string().is_ok());
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_expiration_date_days_serialization() {
        let days = pos!(30.0);
        let expiration = ExpirationDate::Days(days);
        let serialized = serde_json::to_string(&expiration).unwrap();
        assert_eq!(serialized, r#"{"days":30.0}"#);
    }

    #[test]
    fn test_expiration_date_days_deserialization() {
        let json = r#"{"days": 30.0}"#;
        let deserialized: ExpirationDate = serde_json::from_str(json).unwrap();
        match deserialized {
            ExpirationDate::Days(days) => assert_eq!(days, pos!(30.0)),
            _ => panic!("Expected Days variant"),
        }
    }

    #[test]
    fn test_expiration_date_datetime_serialization() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let expiration = ExpirationDate::DateTime(dt);
        let serialized = serde_json::to_string(&expiration).unwrap();
        assert_eq!(serialized, r#"{"datetime":"2025-01-01T00:00:00Z"}"#);
    }

    #[test]
    fn test_expiration_date_datetime_deserialization() {
        let json = r#"{"datetime": "2025-01-01T00:00:00Z"}"#;
        let deserialized: ExpirationDate = serde_json::from_str(json).unwrap();
        match deserialized {
            ExpirationDate::DateTime(dt) => {
                assert_eq!(dt, Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap());
            }
            _ => panic!("Expected DateTime variant"),
        }
    }

    #[test]
    fn test_expiration_date_roundtrip_days() {
        let original = ExpirationDate::Days(pos!(365.0));
        let serialized = serde_json::to_string(&original).unwrap();
        let modified_serialized = serialized.replace("Days", "days");
        let deserialized: ExpirationDate = serde_json::from_str(&modified_serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_expiration_date_roundtrip_datetime() {
        let dt = Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap();
        let original = ExpirationDate::DateTime(dt);
        let serialized = serde_json::to_string(&original).unwrap();
        let modified_serialized = serialized.replace("DateTime", "datetime");
        let deserialized: ExpirationDate = serde_json::from_str(&modified_serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_invalid_datetime_deserialization() {
        let json = r#"{"datetime":{"0":"invalid-date"}}"#;
        let result = serde_json::from_str::<ExpirationDate>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_days_deserialization() {
        let json = r#"{"days":{"0":-30.0}}"#;
        let result = serde_json::from_str::<ExpirationDate>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_variant_deserialization() {
        let json = r#"{"invalid":{"0":30}}"#;
        let result = serde_json::from_str::<ExpirationDate>(json);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_hash {
    use super::*;
    use chrono::{Duration, TimeZone};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Helper function to calculate hash value for any hashable type
    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn test_same_days_expiration_same_hash() {
        let exp1 = ExpirationDate::Days(Positive::new(30.0).unwrap());
        let exp2 = ExpirationDate::Days(Positive::new(30.0).unwrap());

        assert_eq!(calculate_hash(&exp1), calculate_hash(&exp2));
    }

    #[test]
    fn test_different_days_expiration_different_hash() {
        let exp1 = ExpirationDate::Days(Positive::new(30.0).unwrap());
        let exp2 = ExpirationDate::Days(Positive::new(45.0).unwrap());

        assert_ne!(calculate_hash(&exp1), calculate_hash(&exp2));
    }

    #[test]
    fn test_same_datetime_expiration_same_hash() {
        let date1 = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date2 = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let exp1 = ExpirationDate::DateTime(date1);
        let exp2 = ExpirationDate::DateTime(date2);

        assert_eq!(calculate_hash(&exp1), calculate_hash(&exp2));
    }

    #[test]
    fn test_different_datetime_expiration_different_hash() {
        let date1 = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let date2 = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();

        let exp1 = ExpirationDate::DateTime(date1);
        let exp2 = ExpirationDate::DateTime(date2);

        assert_ne!(calculate_hash(&exp1), calculate_hash(&exp2));
    }

    #[test]
    fn test_different_variants_different_hash() {
        let date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let exp1 = ExpirationDate::Days(Positive::new(30.0).unwrap());
        let exp2 = ExpirationDate::DateTime(date);

        assert_ne!(calculate_hash(&exp1), calculate_hash(&exp2));
    }

    #[test]
    fn test_hash_consistency_over_time() {
        let date = Utc::now();
        let exp = ExpirationDate::DateTime(date);

        let hash1 = calculate_hash(&exp);

        // Sleep for a short time to ensure the test remains stable
        std::thread::sleep(std::time::Duration::from_millis(10));

        let hash2 = calculate_hash(&exp);

        assert_eq!(hash1, hash2, "Hash should be consistent over time");
    }

    #[test]
    fn test_different_but_equivalent_dates_different_hash() {
        // Two dates that are 30 days apart
        let now = Utc::now();
        let thirty_days_later = now + Duration::days(30);

        // One as Days variant, one as DateTime variant
        let exp1 = ExpirationDate::Days(Positive::new(30.0).unwrap());
        let exp2 = ExpirationDate::DateTime(thirty_days_later);

        // Even though they might represent the same expiration in practice,
        // they should hash differently because they're different variants
        assert_ne!(calculate_hash(&exp1), calculate_hash(&exp2));
    }
}

#[cfg(test)]
mod tests_from_string {
    use super::*;
    use crate::assert_pos_relative_eq;
    use chrono::{Datelike, TimeZone};

    #[test]
    fn test_from_string_with_time_and_timezone() {
        let today = Utc::now();
        let tomorrow = today + Duration::days(1);
        let tomorrow_1529 = Utc
            .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), 15, 29, 0)
            .unwrap();

        let date_str = format!("{} UTC", tomorrow_1529.format("%Y-%m-%d %H:%M:%S"));

        // Parse the date string to get an ExpirationDate
        let result = ExpirationDate::from_string(&date_str).unwrap();

        // Verify it's a DateTime variant with the correct value
        if let ExpirationDate::DateTime(dt) = result {
            assert_eq!(
                dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                tomorrow_1529.format("%Y-%m-%d %H:%M:%S").to_string()
            );
        } else {
            panic!("Expected DateTime variant");
        }

        // Get days from the DateTime variant
        let days = result.get_days().unwrap();
        assert_pos_relative_eq!(days, pos!(1.0), pos!(0.2));

        // Instead of creating a Days variant, use the original DateTime variant
        // This ensures the time information is preserved
        let date_as_str = result.to_string();

        // The date (without time) should match tomorrow's date
        let date_str_parts: Vec<&str> = date_as_str.split_whitespace().collect();
        let date_part = date_str_parts[0];
        assert_eq!(date_part, tomorrow.format("%Y-%m-%d").to_string());

        // Verify that the string representation matches the original date string
        assert_eq!(date_as_str, date_str);
    }

    #[test]
    fn test_from_string_with_time_no_seconds() {
        // Test format "2025-05-23T15:29"
        let date_str = "2025-05-23T15:29";
        let result = ExpirationDate::from_string(date_str).unwrap();
        if let ExpirationDate::DateTime(dt) = result {
            assert_eq!(dt.format("%Y-%m-%dT%H:%M").to_string(), "2025-05-23T15:29");
        } else {
            panic!("Expected DateTime variant");
        }
    }
}
