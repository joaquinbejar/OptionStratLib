/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/3/25
******************************************************************************/
use crate::utils::TimeFrame;
use crate::utils::time::convert_time_frame;
use crate::{ExpirationDate, Positive};
use serde::{Serialize, Serializer};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use tracing::debug;

/// Represents a step in a time series with an indexed value at a specific time point.
///
/// This struct encapsulates a value at a specific point in a time series, tracking not only
/// the value itself but also its position (index), the time unit it belongs to, and its
/// associated datetime. It's designed to be used in financial and statistical analysis
/// where tracking values across time is essential.
///
/// The generic parameter `T` allows for different value types while ensuring they can be
/// incrementally accumulated (via `AddAssign`) and converted to a `Positive` type for
/// calculations that require non-negative values.
///
/// # Type Parameters
///
/// * `T` - The type of the value being stored, which must implement `AddAssign`, be convertible
///   to `Positive`, and be `Copy`.
///
/// # Fields
///
/// * `index` - An integer representing the position of this step in a sequence.
///
/// * `value` - The actual data value at this time step, of generic type `T`.
///
/// * `time_unit` - Defines the time granularity (e.g., day, hour, minute) for this step.
///
/// * `datetime` - The specific expiration date associated with this time step,
///   which can be either a relative number of days or an absolute datetime.
///
#[derive(Debug, Copy, Clone)]
pub struct Xstep<T>
where
    T: Copy + Into<Positive> + AddAssign + Display,
{
    index: i32,
    step_size_in_time: T,
    time_unit: TimeFrame,
    datetime: ExpirationDate,
}

/// Implementation for the `Xstep<T>` struct, which represents a time step in a simulation.
///
/// The `Xstep<T>` struct is designed to handle sequential time steps in financial simulations,
/// allowing for forward and backward movement in time. The struct maintains an index counter,
/// a value of type T, a time unit, and an expiration date.
///
/// # Type Parameters
///
/// * `T` - A type that implements `AddAssign`, can be converted into `Positive`, and is `Copy`.
///   This typically represents a numeric value used for calculating time differences.
///
/// # Examples
///
/// ```rust
///
/// // Create a step with 7 days as the value, using days as the time unit
/// use optionstratlib::{pos, ExpirationDate};
/// use optionstratlib::simulation::steps::Xstep;
/// use optionstratlib::utils::TimeFrame;
/// let step = Xstep::new(7, TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
///
/// // Move to the next step (forward in time)
/// let next_step = step.next();
///
/// // Move to the previous step (backward in time)
/// let prev_step = step.previous();
/// ```
impl<T> Xstep<T>
where
    T: Copy + Into<Positive> + AddAssign + Display,
{
    /// Creates a new `Xstep` with the specified value, time unit, and datetime.
    ///
    /// # Parameters
    ///
    /// * `value` - The step size value
    /// * `time_unit` - The time unit for this step (e.g., Day, Week, Month)
    /// * `datetime` - The expiration date, which must be of type `ExpirationDate::Days`
    ///
    /// # Panics
    ///
    /// This method will panic if `datetime` is of type `ExpirationDate::DateTime` as only
    /// the `ExpirationDate::Days` variant is currently supported.
    ///
    /// # Returns
    ///
    /// A new `Xstep<T>` instance initialized with the provided parameters and index set to 0.
    pub fn new(value: T, time_unit: TimeFrame, datetime: ExpirationDate) -> Self {
        let datetime = match datetime {
            ExpirationDate::Days(_) => datetime,
            ExpirationDate::DateTime(_) => panic!(
                "ExpirationDate::DateTime is not supported for Step yet. Please use ExpirationDate::Days instead."
            ),
        };
        Self {
            index: 0,
            step_size_in_time: value,
            time_unit,
            datetime,
        }
    }

    /// Returns a reference to the index of the time step.
    ///
    /// The index represents the position of this step in a sequence of time steps.
    ///
    /// # Returns
    ///
    /// * `&i32` - A reference to the index value.
    pub fn index(&self) -> &i32 {
        &self.index
    }

    /// Returns a reference to the step size in time units.
    ///
    /// This represents the magnitude of this time step in the context of its time frame,
    /// such as the number of days, hours, or other time units.
    ///
    /// # Returns
    ///
    /// * `&T` - A reference to the step size value.
    pub fn step_size_in_time(&self) -> &T {
        &self.step_size_in_time
    }

    /// Returns a reference to the time unit associated with this step.
    ///
    /// The time unit defines the granularity of the time measurement (e.g., day, hour, minute).
    ///
    /// # Returns
    ///
    /// * `&TimeFrame` - A reference to the time unit enumeration.
    pub fn time_unit(&self) -> &TimeFrame {
        &self.time_unit
    }

    /// Returns a reference to the datetime associated with this step.
    ///
    /// This represents the specific point in time (as an expiration date) that
    /// this step corresponds to, which can be either a relative value or an absolute datetime.
    ///
    /// # Returns
    ///
    /// * `&ExpirationDate` - A reference to the expiration date.
    pub fn datetime(&self) -> &ExpirationDate {
        &self.datetime
    }

    /// Calculates the number of days left until the expiry date associated with this step.
    ///
    /// This method delegates to the underlying `ExpirationDate` component to determine
    /// the days remaining until expiration. It provides a consistent way to access time-to-expiry
    /// regardless of how the date was originally specified (as days or as an absolute datetime).
    ///
    /// # Returns
    ///
    /// * `Result<Positive, Box<dyn Error>>` - A positive decimal value representing the number of days
    ///   until expiration, or an error if the calculation cannot be performed.
    ///
    pub fn days_left(&self) -> Result<Positive, Box<dyn Error>> {
        self.datetime.get_days()
    }

    /// Generates the next step by reducing the expiration days by the step value.
    ///
    /// This method calculates a new `Xstep` instance with its index incremented by 1,
    /// and the expiration date reduced by the equivalent of `value` in days.
    ///
    /// # Returns
    ///
    /// A new `Xstep<T>` instance with updated index and datetime values.
    pub fn next(&self) -> Result<Self, Box<dyn Error>> {
        let days = self.datetime.get_days().unwrap();
        if days == Positive::ZERO {
            return Err("Cannot generate next step. Expiration date is already reached.".into());
        }
        let days_to_rest = convert_time_frame(
            self.step_size_in_time.into(),
            &self.time_unit,
            &TimeFrame::Day,
        );
        let datetime = if days_to_rest <= days {
            ExpirationDate::Days(days - days_to_rest)
        } else {
            ExpirationDate::Days(Positive::ZERO)
        };
        debug!(
            "days_to_rest: {}, days: {}, datetime: {}",
            days_to_rest, days, datetime
        );
        Ok(Self {
            index: self.index + 1,
            step_size_in_time: self.step_size_in_time,
            time_unit: self.time_unit,
            datetime,
        })
    }

    /// Generates the previous step by increasing the expiration days by the step value.
    ///
    /// This method calculates a new `Xstep` instance with its index decremented by 1,
    /// and the expiration date increased by the equivalent of `value` in days.
    ///
    /// # Returns
    ///
    /// A new `Xstep<T>` instance with updated index and datetime values.
    pub fn previous(&self) -> Result<Self, Box<dyn Error>> {
        let days = self.datetime.get_days().unwrap();
        let days_to_rest = convert_time_frame(
            self.step_size_in_time.into(),
            &self.time_unit,
            &TimeFrame::Day,
        );
        let datetime = ExpirationDate::Days(days + days_to_rest);
        Ok(Self {
            index: self.index - 1,
            step_size_in_time: self.step_size_in_time,
            time_unit: self.time_unit,
            datetime,
        })
    }
}

impl<T> Display for Xstep<T>
where
    T: Copy + Into<Positive> + AddAssign + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Xstep {{ index: {}, value: {}, time_unit: {:?}, datetime: {} }}",
            self.index, self.step_size_in_time, self.time_unit, self.datetime
        )
    }
}

impl<T> Serialize for Xstep<T>
where
    T: Copy + Into<Positive> + AddAssign + Display + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Use a struct with 4 fields to represent Xstep
        use serde::ser::SerializeStruct;
        let step_size_in_time: Positive = self.step_size_in_time.into();
        let mut state = serializer.serialize_struct("Xstep", 4)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("step_size_in_time", &step_size_in_time)?;
        state.serialize_field("time_unit", &self.time_unit)?;
        state.serialize_field("datetime", &self.datetime)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;

    #[test]
    fn test_days_left() {
        let mut step = Xstep::new(1.5f64, TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
        step.index = 42;

        assert!(step.next().is_ok());

        assert_eq!(step.days_left().unwrap(), pos!(30.0));
        let step1 = step.next().unwrap();
        assert_eq!(*step1.index(), 43);
        assert_eq!(step1.days_left().unwrap(), pos!(28.5));
        let step2 = step1.next().unwrap();
        assert_eq!(*step2.index(), 44);
        assert_eq!(step2.days_left().unwrap(), pos!(27.0));
        let step3 = step2.next().unwrap();
        assert_eq!(*step3.index(), 45);
        assert_eq!(step3.days_left().unwrap(), pos!(25.5));
        let step4 = step3.previous().unwrap();
        assert_eq!(*step4.index(), 44);
        assert_eq!(step4.days_left().unwrap(), pos!(27.0));
    }
}

#[cfg(test)]
mod tests_serialize {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use rust_decimal_macros::dec;
    use serde_json::{Value, json};

    #[test]
    fn test_serialized_structure() {
        // Create an Xstep with f64
        let mut step = Xstep::new(1.5f64, TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));
        step.index = 42;

        // Serialize to JSON
        let serialized = serde_json::to_string(&step).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        // Check all fields exist and have correct types
        assert!(parsed.is_object());
        assert!(parsed.get("index").unwrap().is_i64());
        assert!(parsed.get("step_size_in_time").unwrap().is_number());
        assert!(
            parsed.get("time_unit").unwrap().is_object()
                || parsed.get("time_unit").unwrap().is_string()
        );
        assert!(parsed.get("datetime").unwrap().is_object());

        // Check field values
        assert_eq!(parsed["index"], json!(42));
        assert_eq!(parsed["step_size_in_time"], json!(1.5));
    }

    #[test]
    fn test_serialization_value_conversion() {
        // Create instances with different types but same value
        let step_f64 = Xstep::new(2.5f64, TimeFrame::Day, ExpirationDate::Days(pos!(1.0)));
        let step_decimal = Xstep::new(dec!(2.5), TimeFrame::Day, ExpirationDate::Days(pos!(1.0)));
        let step_positive = Xstep::new(pos!(2.5), TimeFrame::Day, ExpirationDate::Days(pos!(1.0)));

        // Serialize all three
        let json_f64 = serde_json::to_string(&step_f64).unwrap();
        let json_decimal = serde_json::to_string(&step_decimal).unwrap();
        let json_positive = serde_json::to_string(&step_positive).unwrap();

        // Parse to access and check the step_size_in_time values
        let parsed_f64: Value = serde_json::from_str(&json_f64).unwrap();
        let parsed_decimal: Value = serde_json::from_str(&json_decimal).unwrap();
        let parsed_positive: Value = serde_json::from_str(&json_positive).unwrap();

        // All should serialize to the same value
        assert_eq!(parsed_f64["step_size_in_time"], json!(2.5));
        assert_eq!(parsed_decimal["step_size_in_time"], json!(2.5));
        assert_eq!(parsed_positive["step_size_in_time"], json!(2.5));
    }

    #[test]
    fn test_serialization_format_identity() {
        // Create instances with different types
        let step_f64 = Xstep::new(3.1f64, TimeFrame::Hour, ExpirationDate::Days(pos!(1.0)));
        let step_decimal = Xstep::new(dec!(3.1), TimeFrame::Hour, ExpirationDate::Days(pos!(1.0)));
        let step_positive = Xstep::new(pos!(3.1), TimeFrame::Hour, ExpirationDate::Days(pos!(1.0)));

        // Serialize all three
        let json_f64 = serde_json::to_string(&step_f64).unwrap();
        let json_decimal = serde_json::to_string(&step_decimal).unwrap();
        let json_positive = serde_json::to_string(&step_positive).unwrap();

        // They should all serialize to identical JSON
        assert_eq!(json_f64, json_decimal);
        assert_eq!(json_decimal, json_positive);
    }

    #[test]
    fn test_serialization_edge_cases() {
        // Test with zero
        let step_zero = Xstep::new(0.01f64, TimeFrame::Minute, ExpirationDate::Days(pos!(0.0)));
        let json_zero = serde_json::to_string(&step_zero).unwrap();
        let parsed_zero: Value = serde_json::from_str(&json_zero).unwrap();
        assert_eq!(parsed_zero["step_size_in_time"], json!(0.01));

        // Test with very small number
        let step_small = Xstep::new(
            0.00001f64,
            TimeFrame::Minute,
            ExpirationDate::Days(pos!(1.0)),
        );
        let json_small = serde_json::to_string(&step_small).unwrap();
        let parsed_small: Value = serde_json::from_str(&json_small).unwrap();
        assert!(parsed_small["step_size_in_time"].as_f64().unwrap() > 0.0);
        assert!(parsed_small["step_size_in_time"].as_f64().unwrap() < 0.0001);

        // Test with very large number
        let step_large = Xstep::new(
            1_000_000.01f64,
            TimeFrame::Minute,
            ExpirationDate::Days(pos!(1.0)),
        );
        let json_large = serde_json::to_string(&step_large).unwrap();
        let parsed_large: Value = serde_json::from_str(&json_large).unwrap();
        assert_eq!(parsed_large["step_size_in_time"], json!(1_000_000.01));
    }

    #[test]
    fn test_serialization_precision() {
        // Create a step with a value that has many decimal places
        let step = Xstep::new(
            1.23456789f64,
            TimeFrame::Day,
            ExpirationDate::Days(pos!(1.0)),
        );

        // Serialize and parse
        let serialized = serde_json::to_string(&step).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        // Check precision is maintained (to reasonable float precision)
        let value = parsed["step_size_in_time"].as_f64().unwrap();
        assert!((value - 1.23456789).abs() < 0.0000001);
    }

    #[test]
    fn test_datetime_serialization() {
        // Test with Days expiration
        let step_days = Xstep::new(1.0f64, TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));

        let serialized_days = serde_json::to_string(&step_days).unwrap();
        let parsed_days: Value = serde_json::from_str(&serialized_days).unwrap();

        assert!(parsed_days["datetime"].is_object());
        assert!(parsed_days["datetime"].get("days").is_some());

        // Note: We don't test DateTime because the new constructor explicitly
        // panics for DateTime variant, which matches the implementation
    }

    #[test]
    #[should_panic(expected = "ExpirationDate::DateTime is not supported for Step yet")]
    fn test_datetime_constructor_panics() {
        // Test that the constructor panics with DateTime variant
        let date_time = chrono::Utc::now();
        let _step = Xstep::new(1.0f64, TimeFrame::Day, ExpirationDate::DateTime(date_time));
    }
}
