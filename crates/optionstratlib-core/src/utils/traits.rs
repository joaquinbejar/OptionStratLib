/// A trait for types that have a notion of length or size.
///
/// This trait provides a standard interface for determining the number of elements
/// in a collection or the size of an object. It defines both a required `len()` method
/// and a default implementation of `is_empty()` which relies on `len()`.
///
/// Types implementing this trait can be checked for emptiness using the `is_empty()`
/// method without requiring a separate implementation, as long as they provide
/// a way to determine their length.
///
pub trait Len {
    /// Returns the number of elements in the collection or the size of the object.
    ///
    /// # Returns
    ///
    /// A `usize` representing the length or size.
    fn len(&self) -> usize;

    /// Returns `true` if the collection contains no elements or the object has zero size.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the object is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
