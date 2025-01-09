use crate::curves::{Curve, MergeOperation};
use crate::error::CurvesError;


/// A trait for performing arithmetic operations between curves.
///
/// # Overview
/// The `CurveArithmetic` trait provides methods to perform various arithmetic 
/// operations between two or more curves. It supports operations like addition, 
/// subtraction, multiplication, division, finding maximum, and finding minimum 
/// values across multiple curves.
///
/// # Operations
/// - `merge_curves`: Combines multiple curves using a specified arithmetic operation.
/// - `merge_with`: Performs an arithmetic operation between two curves.
///
/// # Design Principles
///
/// ## Interpolation Handling
/// When performing arithmetic operations, these methods may need to:
/// 1. Interpolate points between curves with different x-coordinates
/// 2. Handle cases where curves have different lengths or x-ranges
///
/// ## Interpolation Strategy
/// The default implementation uses cubic interpolation to ensure smooth 
/// transitions and minimize artifacts when combining curves with different 
/// point distributions.
///
/// # Error Handling
/// Returns `CurvesError` to provide detailed context about potential 
/// failures during curve arithmetic operations.
///
pub trait CurveArithmetic {
    /// Merges multiple curves using a specified arithmetic operation.
    ///
    /// # Parameters
    /// - `curves`: A slice of references to curves to be merged
    /// - `operation`: The arithmetic operation to apply (`MergeOperation`)
    ///
    /// # Returns
    /// - `Ok(Curve)`: A new curve resulting from the merge operation
    /// - `Err(CurvesError)`: Error if merging fails
    fn merge_curves(
        curves: &[&Curve],
        operation: MergeOperation
    ) -> Result<Curve, CurvesError>;

    /// Performs an arithmetic operation between two curves.
    ///
    /// # Parameters
    /// - `other`: Another curve to merge with the current curve
    /// - `operation`: The arithmetic operation to apply (`MergeOperation`)
    ///
    /// # Returns
    /// - `Ok(Curve)`: A new curve resulting from the merge operation
    /// - `Err(CurvesError)`: Error if merging fails
    fn merge_with(
        &self,
        other: &Curve,
        operation: MergeOperation
    ) -> Result<Curve, CurvesError>;
}