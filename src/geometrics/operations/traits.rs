use crate::geometrics::MergeOperation;

/// A trait that provides arithmetic operations for working with geometries,
/// enabling merging and combining of curve data based on different operations.
///
/// This trait allows performing mathematical operations between geometric objects,
/// particularly curves, to create new derived geometries. It supports various
/// operations like addition, subtraction, multiplication, and more through the
/// `MergeOperation` enum.
///
/// # Type Parameters
/// - `Input`: The type of geometric object that can be merged and operated upon.
///
/// # Associated Types
/// - `Error`: Represents the type of error that may occur during curve operations.
///
/// # Required Methods
///
/// ## `merge`
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
/// # Examples
///
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib::geometrics::{Arithmetic, GeometricObject, MergeOperation};
/// use optionstratlib::curves::{Curve, Point2D};
///
/// let curve1 = Curve::from_vector(vec![
///     Point2D::new(Decimal::ZERO, Decimal::ZERO),
///     Point2D::new(Decimal::ONE, Decimal::ONE),
/// ]);
/// let curve2 = Curve::from_vector(vec![
///     Point2D::new(Decimal::ZERO, Decimal::ONE),
///     Point2D::new(Decimal::ONE, Decimal::TWO),
/// ]);
///
/// // Merge two curves by adding their values
/// let result_curve = Curve::merge(&[&curve1, &curve2], MergeOperation::Add);
/// ```
///
/// # Notes
/// - This trait is designed to be implemented for specific curve types which define how
///   the merging will occur. The associated error type should capture and communicate
///   any issues encountered during operations.
/// - The implementation may need to handle cases where curves have different domains
///   or sampling points.
pub trait Arithmetic<Input> {
    /// The error type returned when merging operations fail
    type Error;

    /// Combines multiple geometries into one using the specified merge operation.
    fn merge(geometries: &[&Input], operation: MergeOperation) -> Result<Input, Self::Error>;

    /// Merges the current curve with another curve using the specified merge operation.
    fn merge_with(&self, other: &Input, operation: MergeOperation) -> Result<Input, Self::Error>;
}
