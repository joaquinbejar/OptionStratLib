/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/1/25
******************************************************************************/

use rust_decimal::Decimal;
use crate::curves::Point2D;
use crate::error::SurfaceError;
use crate::surfaces::types::Point3D;

/// A trait for linear interpolation operations on 2D and 3D points.
///
/// # Overview
/// The `LinearInterpolation` trait provides methods for performing linear interpolation
/// in two-dimensional and three-dimensional space. It assumes inputs in the form of
/// either individual `x` and `y` coordinates or a `Point2D` object. The output is a
/// `Point3D` object, representing the interpolated position in 3D space.
///
/// ## Methods
///
/// ### `linear_interpolate`
/// Interpolates a point in three-dimensional space based on individual `x` and `y`
/// coordinates. Provides flexibility for use cases where coordinates are not already combined
/// into a `Point2D` structure.
///
/// - **Parameters**:
///     - `x` (`Decimal`): The `x` coordinate of the input point.
///     - `y` (`Decimal`): The `y` coordinate of the input point.
/// - **Returns**:
///     - `Result<Point3D, SurfaceError>`:
///         - `Ok(Point3D)`: The interpolated point in 3D space.
///         - `Err(SurfaceError)`: An error if the interpolation fails.
/// - **Errors**:
///     - `SurfaceError::InterpolationError`: Indicates failure due to invalid interpolation logic.
///     - `SurfaceError::Point3DError`: Indicates an issue creating the `Point3D`.
///
/// ### `linear_interpolate_point`
/// Interpolates a point in three-dimensional space based on a `Point2D` input. This method
/// is a convenience for cases where the data is already in point format.
///
/// - **Parameters**:
///     - `x` (`Point2D`): The input point in two-dimensional space.
/// - **Returns**:
///     - `Result<Point3D, SurfaceError>`:
///         - `Ok(Point3D)`: The interpolated point in 3D space.
///         - `Err(SurfaceError)`: An error if the interpolation fails.
/// - **Errors**:
///     - `SurfaceError::InterpolationError`: Indicates failure due to invalid interpolation logic.
///     - `SurfaceError::Point3DError`: Indicates an issue creating the `Point3D`.
///
/// ## Implementing the Trait
/// Any structure implementing this trait should define the specific algorithm for
/// linear interpolation between points in 3D space. The implementation should ensure
/// precision in calculations, as the trait is intended for high-precision geometry
/// and numerical analysis.
///
/// ## Error Handling
/// The associated `SurfaceError` enum is used to capture and describe various
/// error states. Implementations should document specific cases where errors may
/// occur and how they are handled internally.
///
/// ## Applications
/// This trait is well-suited for use in advanced geometry modeling, physical simulations,
/// or any application where 3D surface estimation and interpolation are required.
pub trait LinearInterpolation {

    /// Performs linear interpolation to estimate a `Point3D` on a 3D surface given the input coordinates.
    ///
    /// # Description
    /// The `linear_interpolate` function is used to compute an estimated point in 
    /// three-dimensional space based on two input parameters, `x` and `y`, using a 
    /// linear interpolation technique. The estimated point is returned as a 
    /// `Point3D` with high-precision `Decimal` coordinates.
    ///
    /// This function is generally used in applications requiring high-accuracy 
    /// numerical computations, such as 3D modeling, simulation, or surface analysis.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate of the input point, of type `Decimal`.
    /// - `y`: The y-coordinate of the input point, of type `Decimal`.
    ///
    /// # Returns
    /// This function returns a `Result`:
    /// - `Ok(Point3D)`: If the interpolation is successful, the function returns 
    ///   the estimated point in 3D space as a `Point3D` struct containing `x`, 
    ///   `y`, and `z` coordinates.
    /// - `Err(SurfaceError)`: If an error occurs during the interpolation process, 
    ///   the function returns a `SurfaceError` giving detailed information about the failure.
    ///
    /// # Errors
    /// The function may return one of the following `SurfaceError` variants:
    /// - `Point3DError`: Indicates that an error occurred when constructing or 
    ///   handling the `Point3D`.
    /// - `OperationError`: Reflects issues during underlying operations related to 
    ///   interpolation.
    /// - `InterpolationError`: Indicates a failure in the interpolation process, 
    ///   including invalid or insufficient input data.
    /// - Other `SurfaceError` variants: May signal additional issues, such as an 
    ///   analysis or construction error in the surface model.
    ///
    /// # Example Usage
    /// ```rust
    /// use std::collections::BTreeSet;
    /// use optionstratlib::surfaces::{Point3D, Surface};
    /// use optionstratlib::error::SurfaceError;
    /// use rust_decimal::Decimal;
    ///
    /// fn process_interpolation() -> Result<Point3D, SurfaceError> {
    ///     let x = Decimal::new(125, 2); // Example x-coordinate
    ///     let y = Decimal::new(50, 2);  // Example y-coordinate
    ///     let z = Decimal::new(75, 2);  // Example z-coordinate
    ///     let surface = Surface::new(BTreeSet::from_iter(vec![Point3D::new(x, y, z)]));
    ///     let interpolated_point = surface.linear_interpolate(x, y)?;
    ///     Ok(interpolated_point)
    /// }
    /// ```
    ///
    /// # Notes
    /// - Ensure that input parameters `x` and `y` are valid and within the domain 
    ///   of the surface being interpolated.
    /// - The precision of the result depends on the `Decimal` implementation and the 
    ///   accuracy of the interpolation algorithm.
    ///
    /// # See Also
    /// Refer to other interpolation techniques like bilinear or trilinear interpolation 
    /// for more complex estimation needs.
    fn linear_interpolate(
        &self,
        x: Decimal,
        y: Decimal,
    ) -> Result<Point3D, SurfaceError>;

    fn linear_interpolate_point(
        &self,
        x: Point2D,
    ) -> Result<Point3D, SurfaceError> {
        self.linear_interpolate(x.x, x.y)
    }
}

