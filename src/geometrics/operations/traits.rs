use crate::geometrics::MergeOperation;

/// A trait that provides arithmetic operations for working with geometries,
/// enabling merging and combining of curve data based on different operations.
///
/// # Associated Types
/// - `Error`: Represents the type of error that may occur during curve operations.
///
/// # Required Methods
///
/// ## `merge_geometries`
/// Combines multiple geometries into a single curve based on a specified `MergeOperation`.
///
/// ### Parameters
/// - `geometries`: A slice of references to the input geometries that need to be merged.
/// - `operation`: The operation used to combine the geometries (e.g., addition, subtraction, etc.).
///
/// ### Returns
/// - `Result<Input, Self::Error>`: Returns the resulting merged curve if successful,
///   or an error of type `Self::Error` if the merge process fails.
///
/// ## `merge_with`
/// Merges the current curve with another curve based on a specified `MergeOperation`.
///
/// ### Parameters
/// - `other`: A reference to another curve to be merged.
/// - `operation`: The operation used to combine the geometries (e.g., addition, subtraction, etc.).
///
/// ### Returns
/// - `Result<Input, Self::Error>`: Returns the resulting merged curve if successful,
///   or an error of type `Self::Error` if the merge process fails.
///
/// # Notes
/// - This trait is designed to be implemented for specific curve types which define how
///   the merging will occur. The associated error type should capture and communicate
///   any issues encountered during operations.
pub trait Arithmetic<Input> {
    type Error;

    /// Combines multiple geometries into one using the specified merge operation.
    fn merge(geometries: &[&Input], operation: MergeOperation) -> Result<Input, Self::Error>;

    /// Merges the current curve with another curve using the specified merge operation.
    fn merge_with(&self, other: &Input, operation: MergeOperation) -> Result<Input, Self::Error>;
}
