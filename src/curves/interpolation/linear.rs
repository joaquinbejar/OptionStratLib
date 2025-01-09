use rust_decimal::Decimal;
use crate::curves::Point2D;
use crate::error::CurvesError;

/// A trait for performing linear interpolation on a set of 2D points.
///
/// # Overview
/// The `LinearInterpolation` trait defines a method for calculating
/// a 2D point (`Point2D`) that corresponds to a given target x-coordinate (`x`),
/// based on linear interpolation. It is expected to be implemented by types
/// that provide relevant data or logic for interpolation, such as collections
/// of 2D points.
///
/// # Contract
/// Implementers of this trait must define how their specific type performs
/// linear interpolation. A successful interpolation will result in a `Point2D`.
/// Otherwise, errors are captured in the form of a `CurvesError` if the interpolation
/// cannot be performed.
///
/// # Method
/// - **`linear_interpolate`**: Calculates the interpolated 2D point for a given
///   `x` value using linear interpolation.
///
/// # Possible Errors
/// The method may return:
/// - **`CurvesError::InterpolationError`**:
///   Indicates that interpolation cannot be performed due to missing, invalid,
///   or insufficient data, or because the desired x-coordinate is outside
///   the valid range of points.
///
/// # Usage
/// This trait is generally part of a larger interpolation framework, designed to be
/// called via methods such as `interpolate` from a broader trait like `Interpolate`.
///
/// # Example Workflow
/// 1. The implementer provides access to the set of points used in interpolation
///    (often via a method like `get_points`).
/// 2. The user specifies an x-coordinate for which they seek the interpolated point.
/// 3. The `linear_interpolate` method computes the corresponding `Point2D` based on
///    the provided x-coordinate and the underlying data.
///
/// # Associated Types and Dependencies
/// - **`Decimal`**: Utilized for high-precision x-coordinate input.
/// - **`Point2D`**: Represents the x and y-coordinate result in 2D space.
/// - **`CurvesError`**: Represents errors encountered during the interpolation process.
///
/// # Example Implementation
/// Implementing this trait would involve defining `linear_interpolate` such that
/// it operates over the data being managed by the implementing type. The method
/// typically utilizes a linear interpolation formula using two points that bracket
/// the target x-coordinate.
///
/// # See Also
/// - [`Interpolate`](crate::curves::interpolation::traits::Interpolate): Higher-level trait
///   that integrates multiple interpolation implementations.
/// - [`InterpolationType`](crate::curves::interpolation::types::InterpolationType): Enum
///   that allows selecting interpolation methods, one of which is linear.
///
pub trait LinearInterpolation {
    /// Computes a 2D point (`Point2D`) corresponding to the given target x-coordinate
    /// (`x`) using linear interpolation.
    ///
    /// # Parameters
    /// - **`x`**: A `Decimal` value representing the x-coordinate for which the
    ///   interpolated `Point2D` is to be determined.
    ///
    /// # Returns
    /// - **`Ok(Point2D)`**: A `Point2D` representing the resulting point
    ///   of the linear interpolation.
    /// - **`Err(CurvesError)`**: An error if the interpolation fails for reasons
    ///   such as invalid input, insufficient data, or x being out of range.
    ///
    /// # Errors
    /// - **`CurvesError::InterpolationError`**: Typically returned if:
    ///   - There are fewer than two points available for interpolation.
    ///   - The requested x-coordinate lies outside the x-range of all available
    ///     points.
    ///
    fn linear_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError>;
}