/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/3/25
******************************************************************************/
use crate::Positive;
use rust_decimal::Decimal;

/// A trait for traversing and extracting values from various data structures.
///
/// The `Walktypable` trait defines a standard interface for accessing sequential or calculated
/// values from implementing types. It provides methods to retrieve the next value in a sequence,
/// the maximum value, a decimal representation, or a guaranteed positive value. This trait is
/// particularly useful in simulation contexts, financial calculations, or other scenarios where
/// data needs to be accessed in a consistent manner regardless of the underlying data structure.
///
/// # Type Parameters
///
/// * `Ytype` - The output type that the implementing structure produces when traversed.
///
/// # Default Implementations
///
/// All methods have default implementations that return an `unimplemented!()` error,
/// allowing implementors to only override the methods relevant to their type.
///
/// # Methods
///
/// Each method returns a `Result` type which either contains the requested value
/// or a boxed error if the operation cannot be completed.
///
pub trait Walktypable: Sized {
    /// Generates or retrieves the next value in a sequence.
    ///
    /// This method is used to iterate through values or generate the next
    /// value based on a mathematical formula or algorithm.
    ///
    /// # Parameters
    ///
    /// * `_exp` - An exponent or other parameter that may influence the next value calculation.
    ///
    /// # Returns
    ///
    /// * `Ok(Ytype)` - The next value in the sequence or calculation.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the next value cannot be determined.
    fn walk_next(&self, _exp: f64) -> Result<Self, Box<dyn std::error::Error>>;

    /// Retrieves the maximum value accessible in the implementing type.
    ///
    /// This method is useful for finding upper bounds or peak values within data structures.
    ///
    /// # Returns
    ///
    /// * `Ok(Ytype)` - The maximum value found.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the maximum cannot be determined.
    fn walk_max(&self) -> Result<Self, Box<dyn std::error::Error>>;

    /// Converts or extracts a decimal representation from the implementing type.
    ///
    /// This method provides a way to get a precise decimal value, which is particularly
    /// useful in financial calculations where floating-point precision issues must be avoided.
    ///
    /// # Returns
    ///
    /// * `Ok(Decimal)` - The decimal representation of a value from the implementing type.
    /// * `Err(Box<dyn std::error::Error>)` - An error if the decimal conversion fails.
    fn walk_dec(&self) -> Result<Decimal, Box<dyn std::error::Error>>;

    /// Retrieves a guaranteed positive value from the implementing type.
    ///
    /// This method is useful in contexts where negative values would be invalid,
    /// such as prices, rates, or other strictly positive quantities.
    ///
    /// # Returns
    ///
    /// * `Ok(Positive)` - A wrapper type containing a guaranteed positive value.
    /// * `Err(Box<dyn std::error::Error>)` - An error if a positive value cannot be determined.
    fn walk_positive(&self) -> Result<Positive, Box<dyn std::error::Error>>;

    /// Retrieves the current value in the walk sequence without advancing the iterator.
    ///
    /// This method provides access to the current value in the random walk sequence based on
    /// the internal state, allowing inspection without modifying position or state.
    ///
    /// # Type Parameters
    ///
    /// * `Ytype` - The type representing values in the walk sequence, typically a numeric
    ///   type such as `f64`, `Positive`, or other suitable value type.
    ///
    /// # Returns
    ///
    /// * `Result<Ytype, Box<dyn std::error::Error>>` - The current value if available,
    ///   or an error if the operation cannot be completed.
    ///
    /// # Errors
    ///
    /// This method may return an error in the following situations:
    /// * The walk has not been initialized properly
    /// * The iterator is in an invalid state (e.g., before first element or past the end)
    /// * The current position cannot be accessed due to implementation constraints
    ///
    fn walk_current(&self) -> Result<&Self, Box<dyn std::error::Error>> {
        Ok(self)
    }
}
