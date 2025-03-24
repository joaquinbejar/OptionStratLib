/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 24/3/25
 ******************************************************************************/
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use serde::{Serialize, Serializer};
use tracing::debug;
use crate::{ExpirationDate, Positive};
use crate::utils::time::convert_time_frame;
use crate::utils::TimeFrame;

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
/// use optionstratlib::simulation::step::Xstep;
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
    
    pub fn index(&self) -> &i32 {
        &self.index
    }
    
    pub fn step_size_in_time(&self) -> &T {
        &self.step_size_in_time
    }
    
    pub fn time_unit(&self) -> &TimeFrame {
        &self.time_unit
    }
    pub fn datetime(&self) -> &ExpirationDate {
        &self.datetime
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
            return Err(
                "Cannot generate next step. Expiration date is already reached.".into());
        }
        let days_to_rest = convert_time_frame(self.step_size_in_time.into(), &self.time_unit, &TimeFrame::Day);
        let datetime = if days_to_rest <= days {
            ExpirationDate::Days(days - days_to_rest)
        } else {
            ExpirationDate::Days(Positive::ZERO)
        };
        debug!(
            "days_to_rest: {}, days: {}, datetime: {}",
            days_to_rest, days, datetime);
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
        let days_to_rest = convert_time_frame(self.step_size_in_time.into(), &self.time_unit, &TimeFrame::Day);
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