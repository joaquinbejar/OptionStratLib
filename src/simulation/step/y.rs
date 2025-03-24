/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 24/3/25
 ******************************************************************************/
use std::fmt::{Display, Formatter};
use serde::{Serialize, Serializer};
use crate::Positive;

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
/// must be copCopy + Into<Positive> + Display and convertible to a `Positive` value.
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
            self.index, positive_value.round_to(3)
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