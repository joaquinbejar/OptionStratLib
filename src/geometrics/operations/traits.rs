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
/// # Usage Example
/// ```rust
/// use std::collections::BTreeSet;
/// use rust_decimal_macros::dec;
/// use optionstratlib::geometrics::{Arithmetic, MergeOperation};
/// use optionstratlib::surfaces::{Point3D, Surface};
///
/// // Assuming `Curve` is an implementation of `CurveArithmetic`
/// let points1 = BTreeSet::from_iter(vec![
///             Point3D::new(dec!(0.0), dec!(0.0), dec!(2.0)),
///             Point3D::new(dec!(0.5), dec!(0.0), dec!(2.0)),
///             Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
///             Point3D::new(dec!(0.0), dec!(0.5), dec!(2.0)),
///         ]);
/// let surface1 = Surface::new(points1);
/// let points2 = BTreeSet::from_iter(vec![
///             Point3D::new(dec!(0.0), dec!(0.0), dec!(2.0)),
///             Point3D::new(dec!(0.5), dec!(0.0), dec!(2.0)),
///             Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
///             Point3D::new(dec!(0.0), dec!(0.5), dec!(2.0)),
///         ]);
///  let surface2 = Surface::new(points2);
///
/// // Merge two geometries by adding values
/// let merged_curve = Arithmetic::merge(&[&surface1, &surface2], MergeOperation::Add);
///
/// // Alternatively, merge using an object instance
/// let merged_with = surface1.merge_with(&surface2, MergeOperation::Multiply);
/// ```
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
