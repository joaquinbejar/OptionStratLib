/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/3/25
******************************************************************************/
use crate::utils::TimeFrame;
use crate::utils::time::convert_time_frame;
use crate::{ExpirationDate, Positive};

/// Represents a combined x-y step in a two-dimensional simulation or analysis.
///
/// This struct pairs an x-direction step (typically representing time) with a y-direction step
/// (typically representing price or value). It's designed for financial simulations, data
/// visualization, or any context requiring tracking of values across both a time dimension
/// and a value dimension.
///
/// The generic parameters allow for flexibility in the types of values stored in both
/// dimensions, while enforcing appropriate constraints to ensure mathematical operations
/// can be performed safely.
///
/// # Type Parameters
///
/// * `X` - The type for x-axis values, which must implement `AddAssign` (allowing values to be
///   accumulated), be convertible to `Positive`, and be `Copy`.
///
/// * `Y` - The type for y-axis values, which must be `Copy` and convertible to `Positive`.
///
/// # Fields
///
/// * `x` - An `Xstep<X>` instance representing the time dimension step with its associated
///   temporal information and value.
///
/// * `y` - A `Ystep<Y>` instance representing the value dimension step with its associated index
///   and numeric value.
///
/// # Usage
///
/// Typically used in financial modeling, time series analysis, and visualization contexts where
/// coordinated progression along both time and value axes is needed.
#[derive(Debug, Copy, Clone)]
pub struct Step<X, Y>
where
    X: std::ops::AddAssign + Into<Positive> + Copy,
    Y: Copy + Into<Positive>,
{
    /// The x-axis step containing temporal information and an associated value
    pub x: Xstep<X>,

    /// The y-axis step containing an index and an associated positive value
    pub y: Ystep<Y>,
}

/// Implementation of methods for the `Step` struct, which represents a point in a two-dimensional
/// sequential space with both X and Y coordinates.
///
/// `Step<X, Y>` is designed to track position in simulations or financial models where both
/// dimensions have meaningful values that must remain positive. Each step maintains its position
/// in a sequence along with associated values.
///
/// # Type Parameters
///
/// * `X` - The x-axis value type that must implement `AddAssign`, be convertible to `Positive`, and be `Copy`.
/// * `Y` - The y-axis value type that must be `Copy` and convertible to `Positive`.
///
impl<X, Y> Step<X, Y>
where
    X: std::ops::AddAssign + Into<Positive> + Copy,
    Y: Copy + Into<Positive>,
{
    /// Creates a new Step with the given X and Y coordinates
    ///
    /// # Parameters
    ///
    /// * `x_value` - The initial x-axis value
    /// * `time_unit` - The time frame unit to use for this step
    /// * `datetime` - The expiration date information
    /// * `y_value` - The initial y-axis value
    ///
    /// # Returns
    ///
    /// A new `Step<X, Y>` instance with initialized x and y components
    pub fn new(x_value: X, time_unit: TimeFrame, datetime: ExpirationDate, y_value: Y) -> Self {
        Self {
            x: Xstep::new(x_value, time_unit, datetime),
            y: Ystep::new(y_value),
        }
    }

    /// Move to the next step in the sequence
    ///
    /// Creates a new step with incremented x-position and index, using the provided y-value.
    /// This is typically used to advance forward in a simulation or calculation sequence.
    ///
    /// # Parameters
    ///
    /// * `new_y_value` - The y-axis value to use for the next step
    ///
    /// # Returns
    ///
    /// A new `Step<X, Y>` instance that represents the next step in the sequence
    pub fn next(&self, new_y_value: Y) -> Self {
        Self {
            x: self.x.next(),
            y: Ystep {
                index: self.y.index + 1,
                value: new_y_value,
            },
        }
    }

    /// Move to the previous step in the sequence
    ///
    /// Creates a new step with decremented x-position and index, using the provided y-value.
    /// This is typically used to move backward in a simulation or calculation sequence.
    ///
    /// # Parameters
    ///
    /// * `new_y_value` - The y-axis value to use for the previous step
    ///
    /// # Returns
    ///
    /// A new `Step<X, Y>` instance that represents the previous step in the sequence
    pub fn previous(&self, new_y_value: Y) -> Self {
        Self {
            x: self.x.previous(),
            y: Ystep {
                index: self.y.index - 1,
                value: new_y_value,
            },
        }
    }
}

/// A step entity in a Y-axis progression with an associated numeric value.
///
/// `Ystep` represents a discrete point in a sequence, maintaining both an integer index
/// and an associated positive numeric value. This structure is designed to be used in
/// financial calculations, visualizations, or any context where tracking incremental
/// steps with corresponding values is needed.
///
/// The generic parameter `T` allows flexibility in the type of value stored, as long as
/// it can be converted into a `Positive` type, ensuring all values are valid non-negative numbers.
///
/// # Type Parameters
///
/// * `T` - A type that is `Copy` and can be converted into a `Positive` value.
///   This ensures that all values stored in `Ystep` are non-negative.
///
/// # Fields
///
/// * `index` - An integer representing the step's position in a sequence.
///
/// * `value` - A positive numeric value associated with this step.
///
#[derive(Debug, Copy, Clone)]
pub struct Ystep<T>
where
    T: Copy + Into<Positive>,
{
    /// An integer index representing the step's position in a sequence
    index: i32,

    /// The positive numeric value associated with this step
    value: T,
}

/// A step value holder for simulation values that must be positive.
///
/// `Ystep<T>` maintains an index counter and a value of type `T`, where `T`
/// must be copyable and convertible to a `Positive` value.
///
/// This struct is typically used in financial simulations where values need
/// to be tracked across simulation steps while ensuring they remain positive.
///
/// # Type Parameters
///
/// * `T` - The value type that must implement `Copy` and be convertible to `Positive`
///
/// # Examples
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::simulation::step::Ystep;
///
/// // Create a new step with initial value
/// let step = Ystep::new(dec!(10.5));
///
/// // Access the current value
/// assert_eq!(*step.value(), dec!(10.5));
/// ```
impl<T> Ystep<T>
where
    T: Copy + Into<Positive>,
{
    /// Creates a new `Ystep` instance with the specified value.
    ///
    /// The index is initialized to 0 and the provided value is stored.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial value to store
    ///
    /// # Returns
    ///
    /// A new `Ystep<T>` instance
    pub fn new(value: T) -> Self {
        Self { index: 0, value }
    }

    /// Returns an immutable reference to the stored value.
    ///
    /// # Returns
    ///
    /// A reference to the stored value of type `T`
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the stored value.
    ///
    /// This allows modifying the value while maintaining the `Ystep` structure.
    ///
    /// # Returns
    ///
    /// A mutable reference to the stored value of type `T`
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

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
    T: std::ops::AddAssign + Into<Positive> + Copy,
{
    index: i32,
    value: T,
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
    T: std::ops::AddAssign + Into<Positive> + Copy,
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
            value,
            time_unit,
            datetime,
        }
    }

    /// Generates the next step by reducing the expiration days by the step value.
    ///
    /// This method calculates a new `Xstep` instance with its index incremented by 1,
    /// and the expiration date reduced by the equivalent of `value` in days.
    ///
    /// # Returns
    ///
    /// A new `Xstep<T>` instance with updated index and datetime values.
    pub fn next(&self) -> Self {
        let days = self.datetime.get_days().unwrap();
        let days_to_rest = convert_time_frame(self.value.into(), &self.time_unit, &TimeFrame::Day);
        let datetime = ExpirationDate::Days(days - days_to_rest);
        Self {
            index: self.index + 1,
            value: self.value,
            time_unit: self.time_unit,
            datetime,
        }
    }

    /// Generates the previous step by increasing the expiration days by the step value.
    ///
    /// This method calculates a new `Xstep` instance with its index decremented by 1,
    /// and the expiration date increased by the equivalent of `value` in days.
    ///
    /// # Returns
    ///
    /// A new `Xstep<T>` instance with updated index and datetime values.
    pub fn previous(&self) -> Self {
        let days = self.datetime.get_days().unwrap();
        let days_to_rest = convert_time_frame(self.value.into(), &self.time_unit, &TimeFrame::Day);
        let datetime = ExpirationDate::Days(days + days_to_rest);
        Self {
            index: self.index - 1,
            value: self.value,
            time_unit: self.time_unit,
            datetime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Positive, pos};

    // Helper struct for testing
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct TestValue(u32);

    impl From<TestValue> for Positive {
        fn from(val: TestValue) -> Self {
            Positive::new(val.0 as f64).unwrap()
        }
    }

    impl std::ops::AddAssign for TestValue {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    #[test]
    fn test_step_new() {
        // Test creation with valid parameters
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);

        assert_eq!(step.index, 0);
        assert_eq!(step.value.0, 5);
        assert_eq!(step.time_unit, TimeFrame::Day);
        assert_eq!(step.datetime, ExpirationDate::Days(pos!(30.0)));
    }

    #[test]
    #[should_panic(expected = "ExpirationDate::DateTime is not supported")]
    fn test_step_new_with_datetime_should_panic() {
        // Using DateTime should panic
        use chrono::{Duration, Utc};

        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let dt = Utc::now() + Duration::days(30);
        let datetime = ExpirationDate::DateTime(dt);

        // This should panic
        let _step = Xstep::new(value, time_unit, datetime);
    }

    #[test]
    fn test_step_next() {
        // Test days calculation for next step
        // If we have a step with value 5 Day and 30 days expiration
        // Next step should have 25 days expiration (30 - 5)
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next();

        assert_eq!(next_step.index, 1);
        assert_eq!(next_step.value.0, 5);
        assert_eq!(next_step.time_unit, TimeFrame::Day);
        assert_eq!(next_step.datetime, ExpirationDate::Days(pos!(25.0)));
    }

    #[test]
    fn test_step_next_with_weeks() {
        // Test time conversion for weeks
        // If we have a step with value 2 Week and 30 days expiration
        // Next step should have 16 days expiration (30 - (2 * 7))
        let value = TestValue(2);
        let time_unit = TimeFrame::Week;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next();

        assert_eq!(next_step.index, 1);
        assert_eq!(next_step.value.0, 2);
        assert_eq!(next_step.time_unit, TimeFrame::Week);
        assert_eq!(next_step.datetime, ExpirationDate::Days(pos!(16.0))); // 30 - (2 * 7)
    }

    #[test]
    fn test_step_previous() {
        // Test days calculation for previous step
        // If we have a step with value 5 Day and 30 days expiration
        // Previous step should have 35 days expiration (30 + 5)
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous();

        assert_eq!(prev_step.index, -1);
        assert_eq!(prev_step.value.0, 5);
        assert_eq!(prev_step.time_unit, TimeFrame::Day);
        assert_eq!(prev_step.datetime, ExpirationDate::Days(pos!(35.0)));
    }

    #[test]
    fn test_step_previouse_with_months() {
        // Test time conversion for months
        // If we have a step with value 1 Month and 30 days expiration
        // Previous step should have 60 days expiration (30 + 30)
        // Assuming convert_time_frame correctly converts 1 Month to 30 days
        let value = TestValue(3);
        let time_unit = TimeFrame::Month;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous();

        assert_eq!(prev_step.index, -1);
        assert_eq!(prev_step.value.0, 3);
        assert_eq!(prev_step.time_unit, TimeFrame::Month);
        assert_eq!(prev_step.datetime, ExpirationDate::Days(pos!(121.25)));
    }

    #[test]
    fn test_multiple_steps() {
        // Test a sequence of steps
        let value = TestValue(10);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(100.0));

        let step = Xstep::new(value, time_unit, datetime);
        let step1 = step.next();
        let step2 = step1.next();
        let step3 = step2.next();

        assert_eq!(step1.index, 1);
        assert_eq!(step1.datetime, ExpirationDate::Days(pos!(90.0)));

        assert_eq!(step2.index, 2);
        assert_eq!(step2.datetime, ExpirationDate::Days(pos!(80.0)));

        assert_eq!(step3.index, 3);
        assert_eq!(step3.datetime, ExpirationDate::Days(pos!(70.0)));
    }

    #[test]
    fn test_forward_and_backward() {
        // Test going forward and then backward
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(50.0));

        let original = Xstep::new(value, time_unit, datetime);
        let forward = original.next();
        let back_to_original = forward.previous();

        assert_eq!(original.index, 0);
        assert_eq!(forward.index, 1);
        assert_eq!(back_to_original.index, 0);

        assert_eq!(original.datetime, ExpirationDate::Days(pos!(50.0)));
        assert_eq!(forward.datetime, ExpirationDate::Days(pos!(45.0)));
        assert_eq!(back_to_original.datetime, ExpirationDate::Days(pos!(50.0)));
    }
}

#[cfg(test)]
mod tests_positive {
    use super::*;
    use crate::pos;

    #[test]
    fn test_step_new() {
        // Test creation with valid parameters
        let value = pos!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);

        assert_eq!(step.index, 0);
        assert_eq!(step.value.0, pos!(5.0));
        assert_eq!(step.time_unit, TimeFrame::Day);
        assert_eq!(step.datetime, ExpirationDate::Days(pos!(30.0)));
    }

    #[test]
    #[should_panic(expected = "ExpirationDate::DateTime is not supported")]
    fn test_step_new_with_datetime_should_panic() {
        // Using DateTime should panic
        use chrono::{Duration, Utc};

        let value = pos!(5.0);
        let time_unit = TimeFrame::Day;
        let dt = Utc::now() + Duration::days(30);
        let datetime = ExpirationDate::DateTime(dt);

        // This should panic
        let _step = Xstep::new(value, time_unit, datetime);
    }

    #[test]
    fn test_step_next() {
        // Test days calculation for next step
        // If we have a step with value 5 Day and 30 days expiration
        // Next step should have 25 days expiration (30 - 5)
        let value = pos!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next();

        assert_eq!(next_step.index, 1);
        assert_eq!(next_step.value.0, pos!(5.0));
        assert_eq!(next_step.time_unit, TimeFrame::Day);
        assert_eq!(next_step.datetime, ExpirationDate::Days(pos!(25.0)));
    }

    #[test]
    fn test_step_next_with_weeks() {
        // Test time conversion for weeks
        // If we have a step with value 2 Week and 30 days expiration
        // Next step should have 16 days expiration (30 - (2 * 7))
        let value = pos!(2.0);
        let time_unit = TimeFrame::Week;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next();

        assert_eq!(next_step.index, 1);
        assert_eq!(next_step.value.0, pos!(2.0));
        assert_eq!(next_step.time_unit, TimeFrame::Week);
        assert_eq!(next_step.datetime, ExpirationDate::Days(pos!(16.0))); // 30 - (2 * 7)
    }

    #[test]
    fn test_step_previous() {
        // Test days calculation for previous step
        // If we have a step with value 5 Day and 30 days expiration
        // Previous step should have 35 days expiration (30 + 5)
        let value = pos!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous();

        assert_eq!(prev_step.index, -1);
        assert_eq!(prev_step.value.0, pos!(5.0));
        assert_eq!(prev_step.time_unit, TimeFrame::Day);
        assert_eq!(prev_step.datetime, ExpirationDate::Days(pos!(35.0)));
    }

    #[test]
    fn test_step_previouse_with_months() {
        // Test time conversion for months
        // If we have a step with value 1 Month and 30 days expiration
        // Previous step should have 60 days expiration (30 + 30)
        // Assuming convert_time_frame correctly converts 1 Month to 30 days
        let value = pos!(3.0);
        let time_unit = TimeFrame::Month;
        let datetime = ExpirationDate::Days(pos!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous();

        assert_eq!(prev_step.index, -1);
        assert_eq!(prev_step.value.0, pos!(3.0));
        assert_eq!(prev_step.time_unit, TimeFrame::Month);
        assert_eq!(prev_step.datetime, ExpirationDate::Days(pos!(121.25)));
    }

    #[test]
    fn test_multiple_steps() {
        // Test a sequence of steps
        let value = Positive::TEN;
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(100.0));

        let step = Xstep::new(value, time_unit, datetime);
        let step1 = step.next();
        let step2 = step1.next();
        let step3 = step2.next();

        assert_eq!(step1.index, 1);
        assert_eq!(step1.datetime, ExpirationDate::Days(pos!(90.0)));

        assert_eq!(step2.index, 2);
        assert_eq!(step2.datetime, ExpirationDate::Days(pos!(80.0)));

        assert_eq!(step3.index, 3);
        assert_eq!(step3.datetime, ExpirationDate::Days(pos!(70.0)));
    }

    #[test]
    fn test_forward_and_backward() {
        // Test going forward and then backward
        let value = pos!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(50.0));

        let original = Xstep::new(value, time_unit, datetime);
        let forward = original.next();
        let back_to_original = forward.previous();

        assert_eq!(original.index, 0);
        assert_eq!(forward.index, 1);
        assert_eq!(back_to_original.index, 0);

        assert_eq!(original.datetime, ExpirationDate::Days(pos!(50.0)));
        assert_eq!(forward.datetime, ExpirationDate::Days(pos!(45.0)));
        assert_eq!(back_to_original.datetime, ExpirationDate::Days(pos!(50.0)));
    }
}

#[cfg(test)]
mod tests_ystep {
    use super::*;

    #[test]
    fn test_ystep_new() {
        // Test creation with a simple value
        let value = 42.5;
        let step = Ystep::new(value);

        assert_eq!(step.index, 0);
        assert_eq!(step.value, 42.5);
    }
}

#[cfg(test)]
mod tests_step {
    use super::*;
    use crate::{Positive, pos};

    // Helper struct for testing
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct TestValue(u32);

    impl From<TestValue> for Positive {
        fn from(val: TestValue) -> Self {
            Positive::new(val.0 as f64).unwrap()
        }
    }

    impl std::ops::AddAssign for TestValue {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    #[test]
    fn test_step_new() {
        // Test creation with valid parameters
        let x_value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));
        let y_value = 42.5;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Check X properties
        assert_eq!(step.x.index, 0);
        assert_eq!(step.x.value.0, 5);
        assert_eq!(step.x.time_unit, TimeFrame::Day);
        assert_eq!(step.x.datetime, ExpirationDate::Days(pos!(30.0)));

        // Check Y properties
        assert_eq!(step.y.index, 0);
        assert_eq!(step.y.value, 42.5);
    }

    #[test]
    fn test_step_next() {
        // Setup initial step
        let x_value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));
        let y_value = 42.5;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Create next step with a new Y value
        let new_y_value = 45.0;
        let next_step = step.next(new_y_value);

        // Check X properties
        assert_eq!(next_step.x.index, 1);
        assert_eq!(next_step.x.value.0, 5);
        assert_eq!(next_step.x.time_unit, TimeFrame::Day);
        assert_eq!(next_step.x.datetime, ExpirationDate::Days(pos!(25.0)));

        // Check Y properties
        assert_eq!(next_step.y.index, 1);
        assert_eq!(next_step.y.value, 45.0);
    }

    #[test]
    fn test_step_previous() {
        // Setup initial step
        let x_value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));
        let y_value = 42.5;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Create previous step with a new Y value
        let new_y_value = 38.0;
        let prev_step = step.previous(new_y_value);

        // Check X properties
        assert_eq!(prev_step.x.index, -1);
        assert_eq!(prev_step.x.value.0, 5);
        assert_eq!(prev_step.x.time_unit, TimeFrame::Day);
        assert_eq!(prev_step.x.datetime, ExpirationDate::Days(pos!(35.0)));

        // Check Y properties
        assert_eq!(prev_step.y.index, -1);
        assert_eq!(prev_step.y.value, 38.0);
    }

    #[test]
    fn test_step_with_timeframe_conversion() {
        // Test step creation and manipulation with time frame conversion
        let x_value = TestValue(2);
        let time_unit = TimeFrame::Week;
        let datetime = ExpirationDate::Days(pos!(30.0));
        let y_value = 100.0;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Next step should decrease days by 2 weeks (14 days)
        let next_y = 105.0;
        let next_step = step.next(next_y);

        assert_eq!(next_step.x.datetime, ExpirationDate::Days(pos!(16.0)));
        assert_eq!(next_step.y.value, 105.0);

        // Previous step from initial should increase days by 2 weeks
        let prev_y = 95.0;
        let prev_step = step.previous(prev_y);

        assert_eq!(prev_step.x.datetime, ExpirationDate::Days(pos!(44.0)));
        assert_eq!(prev_step.y.value, 95.0);
    }

    #[test]
    fn test_step_chain() {
        // Test a chain of steps with different Y values
        let x_value = TestValue(10);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(100.0));
        let y_value = 50.0;

        let initial = Step::new(x_value, time_unit, datetime, y_value);

        // Chain: initial -> step1 -> step2 -> step3
        let step1 = initial.next(55.0);
        let step2 = step1.next(60.0);
        let step3 = step2.next(65.0);

        // Check progression of x and y values
        assert_eq!(step1.x.index, 1);
        assert_eq!(step1.x.datetime, ExpirationDate::Days(pos!(90.0)));
        assert_eq!(step1.y.index, 1);
        assert_eq!(step1.y.value, 55.0);

        assert_eq!(step2.x.index, 2);
        assert_eq!(step2.x.datetime, ExpirationDate::Days(pos!(80.0)));
        assert_eq!(step2.y.index, 2);
        assert_eq!(step2.y.value, 60.0);

        assert_eq!(step3.x.index, 3);
        assert_eq!(step3.x.datetime, ExpirationDate::Days(pos!(70.0)));
        assert_eq!(step3.y.index, 3);
        assert_eq!(step3.y.value, 65.0);
    }

    #[test]
    fn test_with_positive_type() {
        // Test using Positive as both X and Y types
        let x_value = pos!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos!(30.0));
        let y_value = pos!(50.0);

        let step = Step::new(x_value, time_unit, datetime, y_value);

        let next_step = step.next(pos!(55.0));

        assert_eq!(next_step.x.datetime, ExpirationDate::Days(pos!(25.0)));
        assert_eq!(next_step.y.value, pos!(55.0));
    }
}
