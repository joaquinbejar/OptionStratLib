/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/3/25
******************************************************************************/
use crate::ExpirationDate;
use crate::error::SimulationError;
use crate::simulation::steps::{Xstep, Ystep};
use crate::utils::TimeFrame;
use num_traits::FromPrimitive;
use positive::Positive;
use rust_decimal::Decimal;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;

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
#[derive(Debug, Clone)]
pub struct Step<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
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
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
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
            y: Ystep::new(0, y_value),
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
    pub fn next(&self, new_y_value: Y) -> Result<Self, SimulationError> {
        let next_x = match self.x.next() {
            Ok(x_step) => x_step,
            Err(e) => {
                return Err(format!(
                    "Cannot generate next step. Expiration date is already reached: {e}"
                )
                .into());
            }
        };
        Ok(Self {
            x: next_x,
            y: Ystep::new(self.y.index() + 1, new_y_value),
        })
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
    pub fn previous(&self, new_y_value: Y) -> Result<Self, SimulationError> {
        let previous_x = match self.x.previous() {
            Ok(x_step) => x_step,
            Err(e) => {
                return Err(format!(
                    "Cannot generate previous step. Expiration date is already reached: {e}"
                )
                .into());
            }
        };
        Ok(Self {
            x: previous_x,
            y: Ystep::new(self.y.index() - 1, new_y_value),
        })
    }

    /// Returns the x-value prepared for graphing operations
    ///
    /// Converts the current x-axis index to a `Positive` value that can be used
    /// in graphing and visualization functions.
    ///
    /// # Returns
    ///
    /// A `Positive` representation of the x-axis index as a floating point value
    pub fn get_graph_x_value(&self) -> Result<Decimal, SimulationError> {
        match Decimal::from_i32(*self.x.index()) {
            Some(x) => Ok(x),
            None => Err("Cannot convert x-axis index to decimal".into()),
        }
    }

    /// Returns the number of days left until expiration for the x-axis component of this step.
    ///
    /// This method provides a convenient way to access the time-to-expiry value from the
    /// x-component of the step without having to handle the potential error result that
    /// the underlying `days_left()` method returns. It's useful for graphing and visualization
    /// contexts where the days remaining until expiration need to be accessed directly.
    ///
    /// # Returns
    ///
    /// * `Positive` - A guaranteed positive decimal value representing the number of days
    ///   until expiration. This value is extracted from the step's x-component's expiration date.
    ///
    pub fn get_graph_x_in_days_left(&self) -> Positive {
        self.x.days_left().unwrap()
    }

    /// Returns the y-value prepared for graphing operations
    ///
    /// Converts the current y-axis value to a `Positive` value that can be used
    /// in graphing and visualization functions.
    ///
    /// # Returns
    ///
    /// A `Positive` representation of the y-axis value
    pub fn get_graph_y_value(&self) -> Positive {
        self.y.positive()
    }

    /// Returns a reference to the `Ystep<Y>` instance associated with the current object.
    ///
    /// # Returns
    ///
    /// A reference to the `Ystep<Y>` contained within the struct.
    ///
    /// # Notes
    ///
    /// This method provides a non-mutable reference to the `Ystep<Y>`. If you need
    /// to modify the `Ystep<Y>`, consider using a different method or accessing it mutably.
    pub fn get_y_step(&self) -> &Ystep<Y> {
        &self.y
    }

    ///
    /// Returns a reference to the `Xstep<X>` held within the struct.
    ///
    /// # Return
    ///
    /// A reference of type `&Xstep<X>` representing the internal `x` field.
    pub fn get_x_step(&self) -> &Xstep<X> {
        &self.x
    }

    /// Returns a reference to the value of type `Y` held by the `self` instance.
    ///
    /// This method retrieves the value stored in the `y` field of the struct,
    /// by calling the `value()` method on `y`, and returns a reference to it.
    ///
    /// # Returns
    /// A reference to the value of type `Y`.
    ///
    pub fn get_value(&self) -> &Y {
        self.y.value()
    }

    /// Retrieves the index value associated with the instance.
    ///
    /// # Returns
    ///
    /// A reference to an object of type `X`, obtained by calling the
    /// `step_size_in_time` method on the field `x` of the instance.
    ///
    /// Note: The behavior of this method depends on the implementation of the
    /// `step_size_in_time` method for the type of `x`.
    pub fn get_index(&self) -> &X {
        self.x.step_size_in_time()
    }

    /// Retrieves the positive value associated with the current instance.
    ///
    /// # Returns
    ///
    /// * `Positive` - A positive value derived from `self.y`.
    ///
    /// This function internally calls the `positive` method on `self.y`
    /// and returns the resulting value.
    pub fn get_positive_value(&self) -> Positive {
        self.y.positive()
    }
}

impl<X, Y> Display for Step<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Step {{ x: {}, y: {} }}", self.x, self.y)
    }
}

impl<X, Y> Serialize for Step<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display + Serialize,
    Y: Into<Positive> + Display + Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Step", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;

    // Helper struct for testing
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct TestValue(u32);

    impl From<TestValue> for Positive {
        fn from(val: TestValue) -> Self {
            Positive::new(val.0 as f64).unwrap()
        }
    }

    impl AddAssign for TestValue {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Display for TestValue {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[test]
    fn test_step_new() {
        // Test creation with valid parameters
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);

        assert_eq!(*step.index(), 0);
        assert_eq!(step.step_size_in_time(), &TestValue(5));
        assert_eq!(*step.time_unit(), TimeFrame::Day);
        assert_eq!(*step.datetime(), ExpirationDate::Days(pos_or_panic!(30.0)));
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
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next().unwrap();

        assert_eq!(*step.index(), 0);
        assert_eq!(step.step_size_in_time(), &TestValue(5));
        assert_eq!(*step.time_unit(), TimeFrame::Day);
        assert_eq!(*step.datetime(), ExpirationDate::Days(pos_or_panic!(30.0)));

        assert_eq!(*next_step.index(), 1);
        assert_eq!(next_step.step_size_in_time(), &TestValue(5));
        assert_eq!(*next_step.time_unit(), TimeFrame::Day);
        assert_eq!(
            *next_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(25.0))
        );
    }

    #[test]
    fn test_step_next_with_weeks() {
        // Test time conversion for weeks
        // If we have a step with value 2 Week and 30 days expiration
        // Next step should have 16 days expiration (30 - (2 * 7))
        let value = TestValue(2);
        let time_unit = TimeFrame::Week;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next().unwrap();

        assert_eq!(*step.index(), 0);
        assert_eq!(step.step_size_in_time(), &TestValue(2));
        assert_eq!(*step.time_unit(), TimeFrame::Week);
        assert_eq!(*step.datetime(), ExpirationDate::Days(pos_or_panic!(30.0)));

        assert_eq!(*next_step.index(), 1);
        assert_eq!(next_step.step_size_in_time(), &TestValue(2));
        assert_eq!(*next_step.time_unit(), TimeFrame::Week);
        assert_eq!(
            *next_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(16.0))
        );
    }

    #[test]
    fn test_step_previous() {
        // Test days calculation for previous step
        // If we have a step with value 5 Day and 30 days expiration
        // Previous step should have 35 days expiration (30 + 5)
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous().unwrap();

        assert_eq!(*step.index(), 0);
        assert_eq!(step.step_size_in_time(), &TestValue(5));
        assert_eq!(*step.time_unit(), TimeFrame::Day);
        assert_eq!(*step.datetime(), ExpirationDate::Days(pos_or_panic!(30.0)));

        assert_eq!(*prev_step.index(), -1);
        assert_eq!(prev_step.step_size_in_time(), &TestValue(5));
        assert_eq!(*prev_step.time_unit(), TimeFrame::Day);
        assert_eq!(
            *prev_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(35.0))
        );
    }

    #[test]
    fn test_step_previouse_with_months() {
        // Test time conversion for months
        // If we have a step with value 1 Month and 30 days expiration
        // Previous step should have 60 days expiration (30 + 30)
        // Assuming convert_time_frame correctly converts 1 Month to 30 days
        let value = TestValue(3);
        let time_unit = TimeFrame::Month;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous().unwrap();

        assert_eq!(*step.index(), 0);
        assert_eq!(step.step_size_in_time(), &TestValue(3));
        assert_eq!(*step.time_unit(), TimeFrame::Month);
        assert_eq!(*step.datetime(), ExpirationDate::Days(pos_or_panic!(30.0)));

        assert_eq!(*prev_step.index(), -1);
        assert_eq!(prev_step.step_size_in_time(), &TestValue(3));
        assert_eq!(*prev_step.time_unit(), TimeFrame::Month);
        assert_eq!(
            *prev_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(121.25))
        );
    }

    #[test]
    fn test_multiple_steps() {
        // Test a sequence of steps
        let value = TestValue(10);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(Positive::HUNDRED);

        let step = Xstep::new(value, time_unit, datetime);
        let step1 = step.next().unwrap();
        let step2 = step1.next().unwrap();
        let step3 = step2.next().unwrap();

        assert_eq!(*step1.index(), 1);
        assert_eq!(*step1.datetime(), ExpirationDate::Days(pos_or_panic!(90.0)));
        assert_eq!(*step2.index(), 2);
        assert_eq!(*step2.datetime(), ExpirationDate::Days(pos_or_panic!(80.0)));
        assert_eq!(*step3.index(), 3);
        assert_eq!(*step3.datetime(), ExpirationDate::Days(pos_or_panic!(70.0)));
    }

    #[test]
    fn test_forward_and_backward() {
        // Test going forward and then backward
        let value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(50.0));

        let original = Xstep::new(value, time_unit, datetime);
        let forward = original.next().unwrap();
        let back_to_original = forward.previous().unwrap();

        assert_eq!(*original.index(), 0);
        assert_eq!(*forward.index(), 1);
        assert_eq!(*back_to_original.index(), 0);
        assert_eq!(
            *original.datetime(),
            ExpirationDate::Days(pos_or_panic!(50.0))
        );
        assert_eq!(
            *forward.datetime(),
            ExpirationDate::Days(pos_or_panic!(45.0))
        );
        assert_eq!(
            *back_to_original.datetime(),
            ExpirationDate::Days(pos_or_panic!(50.0))
        );
    }
}

#[cfg(test)]
mod tests_positive {
    use super::*;
    use positive::pos_or_panic;

    #[test]
    fn test_step_new() {
        // Test creation with valid parameters
        let value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);

        assert_eq!(*step.index(), 0);
        assert_eq!(*step.step_size_in_time(), pos_or_panic!(5.0));
        assert_eq!(*step.time_unit(), TimeFrame::Day);
        assert_eq!(*step.datetime(), ExpirationDate::Days(pos_or_panic!(30.0)));
    }

    #[test]
    #[should_panic(expected = "ExpirationDate::DateTime is not supported")]
    fn test_step_new_with_datetime_should_panic() {
        // Using DateTime should panic
        use chrono::{Duration, Utc};

        let value = pos_or_panic!(5.0);
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
        let value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next().unwrap();

        assert_eq!(*next_step.index(), 1);
        assert_eq!(*next_step.step_size_in_time(), pos_or_panic!(5.0));
        assert_eq!(*next_step.time_unit(), TimeFrame::Day);
        assert_eq!(
            *next_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(25.0))
        );
    }

    #[test]
    fn test_step_next_with_weeks() {
        // Test time conversion for weeks
        // If we have a step with value 2 Week and 30 days expiration
        // Next step should have 16 days expiration (30 - (2 * 7))
        let value = Positive::TWO;
        let time_unit = TimeFrame::Week;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let next_step = step.next().unwrap();

        assert_eq!(*next_step.index(), 1);
        assert_eq!(*next_step.step_size_in_time(), Positive::TWO);
        assert_eq!(*next_step.time_unit(), TimeFrame::Week);
        assert_eq!(
            *next_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(16.0))
        );
    }

    #[test]
    fn test_step_previous() {
        // Test days calculation for previous step
        // If we have a step with value 5 Day and 30 days expiration
        // Previous step should have 35 days expiration (30 + 5)
        let value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous().unwrap();

        assert_eq!(*prev_step.index(), -1);
        assert_eq!(*prev_step.step_size_in_time(), pos_or_panic!(5.0));
        assert_eq!(*prev_step.time_unit(), TimeFrame::Day);
        assert_eq!(
            *prev_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(35.0))
        );
    }

    #[test]
    fn test_step_previouse_with_months() {
        // Test time conversion for months
        // If we have a step with value 1 Month and 30 days expiration
        // Previous step should have 60 days expiration (30 + 30)
        // Assuming convert_time_frame correctly converts 1 Month to 30 days
        let value = pos_or_panic!(3.0);
        let time_unit = TimeFrame::Month;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));

        let step = Xstep::new(value, time_unit, datetime);
        let prev_step = step.previous().unwrap();

        assert_eq!(*prev_step.index(), -1);
        assert_eq!(*prev_step.step_size_in_time(), pos_or_panic!(3.0));
        assert_eq!(*prev_step.time_unit(), TimeFrame::Month);
        assert_eq!(
            *prev_step.datetime(),
            ExpirationDate::Days(pos_or_panic!(121.25))
        );
    }

    #[test]
    fn test_multiple_steps() {
        // Test a sequence of steps
        let value = Positive::TEN;
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(Positive::HUNDRED);

        let step = Xstep::new(value, time_unit, datetime);
        let step1 = step.next().unwrap();
        let step2 = step1.next().unwrap();
        let step3 = step2.next().unwrap();

        assert_eq!(*step1.index(), 1);
        assert_eq!(*step1.datetime(), ExpirationDate::Days(pos_or_panic!(90.0)));
        assert_eq!(*step2.index(), 2);
        assert_eq!(*step2.datetime(), ExpirationDate::Days(pos_or_panic!(80.0)));
        assert_eq!(*step3.index(), 3);
        assert_eq!(*step3.datetime(), ExpirationDate::Days(pos_or_panic!(70.0)));
    }

    #[test]
    fn test_forward_and_backward() {
        // Test going forward and then backward
        let value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(50.0));

        let original = Xstep::new(value, time_unit, datetime);
        let forward = original.next().unwrap();
        let back_to_original = forward.previous().unwrap();

        assert_eq!(*original.index(), 0);
        assert_eq!(*forward.index(), 1);
        assert_eq!(*back_to_original.index(), 0);
        assert_eq!(
            *original.datetime(),
            ExpirationDate::Days(pos_or_panic!(50.0))
        );
        assert_eq!(
            *forward.datetime(),
            ExpirationDate::Days(pos_or_panic!(45.0))
        );
        assert_eq!(
            *back_to_original.datetime(),
            ExpirationDate::Days(pos_or_panic!(50.0))
        );
    }
}

#[cfg(test)]
mod tests_step {
    use super::*;
    use positive::pos_or_panic;

    // Helper struct for testing
    #[derive(Debug, Copy, Clone, PartialEq)]
    struct TestValue(u32);

    impl From<TestValue> for Positive {
        fn from(val: TestValue) -> Self {
            Positive::new(val.0 as f64).unwrap()
        }
    }

    impl AddAssign for TestValue {
        fn add_assign(&mut self, other: Self) {
            self.0 += other.0;
        }
    }

    impl Display for TestValue {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[test]
    fn test_step_new() {
        // Test creation with valid parameters
        let x_value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = 42.5;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Check X properties
        assert_eq!(*step.x.index(), 0);
        assert_eq!(step.x.step_size_in_time(), &TestValue(5));
        assert_eq!(*step.x.time_unit(), TimeFrame::Day);
        assert_eq!(
            *step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(30.0))
        );

        // Check Y properties
        assert_eq!(*step.y.index(), 0);
        assert_eq!(*step.y.value(), 42.5);

        let result_next = step.next(100.0);
        assert!(result_next.is_ok());
        let next = result_next.unwrap();

        assert!(
            next.to_string()
                .contains("Step { x: Xstep { index: 1, value: 5, time_unit: Day, datetime:")
        );

        let previous_next = step.previous(100.0);
        assert!(previous_next.is_ok());
        let previous = previous_next.unwrap();

        assert!(
            previous
                .to_string()
                .contains("Step { x: Xstep { index: -1, value: 5, time_unit: Day, datetime:")
        );

        assert_eq!(next.get_graph_x_value().unwrap(), Decimal::ONE);
        assert_eq!(previous.get_graph_x_value().unwrap(), Decimal::NEGATIVE_ONE);
    }

    #[test]
    fn test_step_next() {
        // Setup initial step
        let x_value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = 42.5;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Create next step with a new Y value
        let new_y_value = 45.0;
        let next_step = step.next(new_y_value).unwrap();

        // Check X properties
        assert_eq!(*step.x.index(), 0);
        assert_eq!(step.x.step_size_in_time(), &TestValue(5));
        assert_eq!(*step.x.time_unit(), TimeFrame::Day);
        assert_eq!(
            *step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(30.0))
        );

        // Check Y properties
        assert_eq!(*step.y.index(), 0);
        assert_eq!(*step.y.value(), 42.5);

        assert_eq!(*next_step.x.index(), 1);
        assert_eq!(next_step.x.step_size_in_time(), &TestValue(5));
        assert_eq!(*next_step.x.time_unit(), TimeFrame::Day);
        assert_eq!(
            *next_step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(25.0))
        );

        // Check Y properties
        assert_eq!(*next_step.y.index(), 1);
        assert_eq!(*next_step.y.value(), 45.0);
    }

    #[test]
    fn test_step_previous() {
        // Setup initial step
        let x_value = TestValue(5);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = 42.5;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Create previous step with a new Y value
        let new_y_value = 38.0;
        let prev_step = step.previous(new_y_value).unwrap();

        // Check X properties
        assert_eq!(*step.x.index(), 0);
        assert_eq!(step.x.step_size_in_time(), &TestValue(5));
        assert_eq!(*step.x.time_unit(), TimeFrame::Day);
        assert_eq!(
            *step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(30.0))
        );

        // Check Y properties
        assert_eq!(*prev_step.y.value(), 38.0);
    }

    #[test]
    fn test_step_with_timeframe_conversion() {
        // Test step creation and manipulation with time frame conversion
        let x_value = TestValue(2);
        let time_unit = TimeFrame::Week;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = 100.0;

        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Next step should decrease days by 2 weeks (14 days)
        let next_y = 105.0;
        let next_step = step.next(next_y).unwrap();

        assert_eq!(
            *next_step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(16.0))
        );
        assert_eq!(*next_step.y.value(), 105.0);

        // Previous step from initial should increase days by 2 weeks
        let prev_y = 95.0;
        let prev_step = step.previous(prev_y).unwrap();

        assert_eq!(
            *prev_step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(44.0))
        );
        assert_eq!(*prev_step.y.value(), 95.0);
    }

    #[test]
    fn test_step_chain() {
        // Test a chain of steps with different Y values
        let x_value = TestValue(10);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(Positive::HUNDRED);
        let y_value = 50.0;

        let initial = Step::new(x_value, time_unit, datetime, y_value);

        // Chain: initial -> step1 -> step2 -> step3
        let step1 = initial.next(55.0).unwrap();
        let step2 = step1.next(60.0).unwrap();
        let step3 = step2.next(65.0).unwrap();

        // Check progression of x and y values
        assert_eq!(*step1.x.index(), 1);
        assert_eq!(
            *step1.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(90.0))
        );
        assert_eq!(*step1.y.index(), 1);
        assert_eq!(*step1.y.value(), 55.0);
        assert_eq!(*step2.x.index(), 2);
        assert_eq!(
            *step2.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(80.0))
        );
        assert_eq!(*step2.y.index(), 2);
        assert_eq!(*step2.y.value(), 60.0);
        assert_eq!(*step3.x.index(), 3);
        assert_eq!(
            *step3.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(70.0))
        );
        assert_eq!(*step3.y.index(), 3);
        assert_eq!(*step3.y.value(), 65.0);
    }

    #[test]
    fn test_with_positive_type() {
        // Test using Positive as both X and Y types
        let x_value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = pos_or_panic!(50.0);

        let step = Step::new(x_value, time_unit, datetime, y_value);

        let next_step = step.next(pos_or_panic!(55.0)).unwrap();

        assert_eq!(
            *next_step.x.datetime(),
            ExpirationDate::Days(pos_or_panic!(25.0))
        );
        assert_eq!(*next_step.y.value(), pos_or_panic!(55.0));
    }

    #[test]
    fn test_with_zero_days() {
        // Test using Positive as both X and Y types
        let x_value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(Positive::ZERO);
        let y_value = pos_or_panic!(50.0);
        let step = Step::new(x_value, time_unit, datetime, y_value);
        let result = step.next(pos_or_panic!(55.0));
        assert!(result.is_err());
        let step_x = step.get_x_step();
        assert_eq!(*step_x.index(), 0);
        assert_eq!(*step_x.step_size_in_time(), pos_or_panic!(5.0));
        let result = step_x.next();
        assert!(result.is_err());
    }

    #[test]
    fn next_ok_increments_indices_and_builds_self() {
        let x_value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(Positive::TWO);
        let y_value = pos_or_panic!(50.0);
        let step = Step::new(x_value, time_unit, datetime, y_value);

        let new_y = pos_or_panic!(55.0);
        let next = step.next(new_y).unwrap();

        assert_eq!(next.get_value(), &new_y);
        assert_eq!(*next.x.index(), 1);
        assert_eq!(*next.y.index(), step.y.index() + 1);
    }
}

#[cfg(test)]
mod tests_step_serialization {
    use super::*;

    use chrono::{TimeZone, Utc};
    use positive::pos_or_panic;
    use serde_json::{self, Value};

    // Helper function to create a test step with f64 values
    fn create_test_step() -> Step<f64, f64> {
        let x_value = 1.5;
        let time_unit = TimeFrame::Day;
        let expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = 100.0;

        Step::new(x_value, time_unit, expiration_date, y_value)
    }

    #[test]
    fn test_step_serialization_with_days() {
        let step = create_test_step();

        // Serialize to JSON string
        let serialized = serde_json::to_string(&step).unwrap();

        // Parse the JSON string back to Value for easier assertions
        let json_value: Value = serde_json::from_str(&serialized).unwrap();

        // Verify the structure is correct
        assert!(json_value.is_object());
        assert!(json_value.get("x").is_some());
        assert!(json_value.get("y").is_some());

        // Verify x fields
        let x = &json_value["x"];
        assert_eq!(x["index"], 0);
        assert_eq!(x["step_size_in_time"], 1.5);

        // Verify y fields
        let y = &json_value["y"];
        assert_eq!(y["index"], 0);
        assert_eq!(y["value"], 100.0);
    }

    #[test]
    #[should_panic(expected = "ExpirationDate::DateTime is not supported for Step yet")]
    fn test_step_with_datetime_panics() {
        // Based on the test in Xstep serialization, the constructor panics with DateTime
        let x_value = 2.5;
        let time_unit = TimeFrame::Hour;
        let expiration_date =
            ExpirationDate::DateTime(Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap());
        let y_value = 200.0;

        Step::new(x_value, time_unit, expiration_date, y_value);
    }

    #[test]
    fn test_step_serialize() {
        let x_value = pos_or_panic!(5.0);
        let time_unit = TimeFrame::Day;
        let datetime = ExpirationDate::Days(pos_or_panic!(30.0));
        let y_value = 42.5;
        let step = Step::new(x_value, time_unit, datetime, y_value);

        // Serialize to JSON string
        let serialized = serde_json::to_string(&step).unwrap();
        assert_eq!(
            serialized,
            r#"{"x":{"index":0,"step_size_in_time":5,"time_unit":"Day","datetime":{"days":30.0}},"y":{"index":0,"value":42.5}}"#
        );
    }

    #[test]
    fn test_step_pretty_serialization() {
        let step = create_test_step();

        // Serialize to pretty JSON string
        let serialized = serde_json::to_string_pretty(&step).unwrap();

        // Verify the serialized string contains expected formatting
        assert!(serialized.contains("{\n"));
        assert!(serialized.contains("  \"x\": {"));
        assert!(serialized.contains("  \"y\": {"));

        // Make sure the content is still correct by deserializing
        let deserialized: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized["x"]["step_size_in_time"], 1.5);
        assert_eq!(deserialized["y"]["value"], 100.0);
    }
}
