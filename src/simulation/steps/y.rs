/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/3/25
******************************************************************************/
use crate::Positive;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};

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
    T: Copy + Into<Positive> + Display,
{
    /// An integer index representing the step's position in a sequence
    index: i32,

    /// The positive numeric value associated with this step
    value: T,
}

/// A step value holder for simulation values that must be positive.
///
/// `Ystep<T>` maintains an index counter and a value of type `T`, where `T`
/// must be copCopy + `Into<Positive>` + Display and convertible to a `Positive` value.
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
/// use optionstratlib::simulation::steps::Ystep;
///
/// // Create a new step with initial value
/// let step = Ystep::new(1, dec!(10.5));
///
/// // Access the current value
/// assert_eq!(*step.value(), dec!(10.5));
/// ```
impl<T> Ystep<T>
where
    T: Copy + Into<Positive> + Display,
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
    pub fn new(index: i32, value: T) -> Self {
        Self { index, value }
    }

    /// Creates a new step with an incremented index and the provided value.
    ///
    /// This method produces a new step instance that represents the next step in a sequence,
    /// with an index one higher than the current step and storing the provided value.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to store in the new step
    ///
    /// # Returns
    ///
    /// A new `Ystep<T>` instance with incremented index
    pub fn next(&self, value: T) -> Self {
        let index = self.index + 1;
        Self { index, value }
    }

    /// Returns an immutable reference to the stored value.
    ///
    /// # Returns
    ///
    /// A reference to the stored value of type `T`
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns an immutable reference to the index of this step.
    ///
    /// The index represents the step's position in a sequence, typically
    /// indicating how many steps have occurred since initialization.
    ///
    /// # Returns
    ///
    /// A reference to the index as an `i32` value
    pub fn index(&self) -> &i32 {
        &self.index
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

impl<T> Display for Ystep<T>
where
    T: Copy + Into<Positive> + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let positive_value: Positive = self.value.into();
        write!(
            f,
            "Ystep {{ index: {}, value: {} }}",
            self.index,
            positive_value.round_to(3)
        )
    }
}

impl<T> Serialize for Ystep<T>
where
    T: Copy + Into<Positive> + Display + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert value to Positive for consistent serialization
        let value: Positive = self.value.into();

        // Use a struct with 2 fields to represent Ystep
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Ystep", 2)?;
        state.serialize_field("index", &self.index)?;
        state.serialize_field("value", &value)?;
        state.end()
    }
}

#[cfg(test)]
mod tests_ystep {
    use super::*;

    #[test]
    fn test_ystep_new() {
        // Test creation with a simple value
        let value = 42.5;
        let step = Ystep::new(0, value);

        assert_eq!(step.index, 0);
        assert_eq!(step.value, 42.5);
    }
}

#[cfg(test)]
mod tests_serialize {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;
    use serde_json::{Value, json};

    #[test]
    fn test_basic_serialization() {
        let step = Ystep::new(5, 10.5f64);
        let serialized = serde_json::to_string(&step).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(parsed["index"], 5);
        assert_eq!(parsed["value"], 10.5);
    }

    #[test]
    fn test_serialized_structure() {
        let step = Ystep::new(42, 15.75f64);
        let serialized = serde_json::to_string(&step).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        // Check structure
        assert!(parsed.is_object());
        assert_eq!(parsed.as_object().unwrap().len(), 2);
        assert!(parsed.get("index").unwrap().is_i64());
        assert!(parsed.get("value").unwrap().is_number());

        // Check values
        assert_eq!(parsed["index"], json!(42));
        assert_eq!(parsed["value"], json!(15.75));
    }

    #[test]
    fn test_type_conversion() {
        // Test with different types but same values
        let step_f64 = Ystep::new(1, 2.5f64);
        let step_decimal = Ystep::new(1, dec!(2.5));
        let step_positive = Ystep::new(1, pos!(2.5));

        // Serialize all
        let json_f64 = serde_json::to_string(&step_f64).unwrap();
        let json_decimal = serde_json::to_string(&step_decimal).unwrap();
        let json_positive = serde_json::to_string(&step_positive).unwrap();

        // Parse to compare values
        let parsed_f64: Value = serde_json::from_str(&json_f64).unwrap();
        let parsed_decimal: Value = serde_json::from_str(&json_decimal).unwrap();
        let parsed_positive: Value = serde_json::from_str(&json_positive).unwrap();

        // All should have the same value representation
        assert_eq!(parsed_f64["value"], json!(2.5));
        assert_eq!(parsed_decimal["value"], json!(2.5));
        assert_eq!(parsed_positive["value"], json!(2.5));
    }

    #[test]
    fn test_json_format_identity() {
        // Test with different types but same values
        let step_f64 = Ystep::new(3, 4.01f64);
        let step_decimal = Ystep::new(3, dec!(4.01));
        let step_positive = Ystep::new(3, pos!(4.01));

        // Serialize all
        let json_f64 = serde_json::to_string(&step_f64).unwrap();
        let json_decimal = serde_json::to_string(&step_decimal).unwrap();
        let json_positive = serde_json::to_string(&step_positive).unwrap();

        // All should serialize to identical JSON
        assert_eq!(json_f64, json_decimal);
        assert_eq!(json_decimal, json_positive);
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero
        let step_zero = Ystep::new(0, 0.1f64);
        let json_zero = serde_json::to_string(&step_zero).unwrap();
        let parsed_zero: Value = serde_json::from_str(&json_zero).unwrap();
        assert_eq!(parsed_zero["value"], json!(0.1));

        // Test with very small value
        let step_small = Ystep::new(1, 0.000001f64);
        let json_small = serde_json::to_string(&step_small).unwrap();
        let parsed_small: Value = serde_json::from_str(&json_small).unwrap();
        assert!(parsed_small["value"].as_f64().unwrap() > 0.0);
        assert!(parsed_small["value"].as_f64().unwrap() < 0.0001);

        // Test with large value
        let step_large = Ystep::new(2, 1_000_000.01f64);
        let json_large = serde_json::to_string(&step_large).unwrap();
        let parsed_large: Value = serde_json::from_str(&json_large).unwrap();
        assert_eq!(parsed_large["value"], json!(1_000_000.01));
    }

    #[test]
    fn test_decimal_precision() {
        let step = Ystep::new(1, 1.23456789f64);
        let serialized = serde_json::to_string(&step).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        // Check precision is maintained (to reasonable float precision)
        let value = parsed["value"].as_f64().unwrap();
        assert!((value - 1.23456789).abs() < 0.0000001);
    }

    #[test]
    fn test_next_serialization() {
        let step = Ystep::new(1, 5.0f64);
        let next_step = step.next(10.0f64);

        let serialized = serde_json::to_string(&next_step).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        // Check next step has incremented index
        assert_eq!(parsed["index"], 2);
        assert_eq!(parsed["value"], 10.0);
    }

    #[test]
    fn test_pretty_serialization() {
        let step = Ystep::new(7, 15.25f64);
        let serialized = serde_json::to_string_pretty(&step).unwrap();

        // Check it contains appropriate formatting
        assert!(serialized.contains("\n"));
        assert!(serialized.contains("  "));

        // Ensure it can be parsed back
        let parsed: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed["index"], 7);
        assert_eq!(parsed["value"], 15.25);
    }
}
