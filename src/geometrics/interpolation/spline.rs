use crate::error::InterpolationError;

/// A trait defining the functionality for spline-based interpolation over a dataset.
///
/// # Overview
/// The `SplineInterpolation` trait provides a framework for approximating unknown values
/// between known data points using spline interpolation techniques. A spline is a
/// piecewise polynomial function that ensures smooth and continuous transitions
/// across its entire range. This method is commonly utilized in numerical analysis,
/// computer graphics, and scientific computations.
///
/// ## Use Cases
/// - Filling in missing values in datasets.
/// - Generating smooth curves to approximate trends in data.
/// - Scenarios requiring continuity and smoothness across multiple points.
///
/// ## Key Features:
/// - Allows implementing custom spline interpolators.
/// - Provides error handling for boundary cases or dataset inconsistencies.
/// - Can be extended for different types of interpolation datasets.
///
/// # Associated Types
/// - `Point`: Represents the output type, typically a point in 2D space (e.g., `Point2D`).
/// - `Input`: The type of the input x-coordinate for which interpolation is performed.
/// - `Error`: Represents possible errors encountered during interpolation
///   (e.g., inadequate data, boundary conditions, or internal computation errors).
///
/// # Required Method
///
/// ### `spline_interpolate`
///
/// - **Purpose**: Computes the `y` value corresponding to a supplied `x` value
///   using spline interpolation techniques.
/// - **Parameters**:
///   - `x`: The x-coordinate (of type `Input`) for which interpolation is required.
/// - **Returns**:
///   - `Ok(Point)`: Represents the interpolated output, typically containing
///     both the x and calculated y coordinates.
///   - `Err(Self::Error)`: Represents the failure scenario during the interpolation.
///
/// # Error Handling
/// This trait defines an associated `Error` type to handle failures during interpolation.
/// Expected error cases include:
/// - Insufficient or invalid data points.
/// - Extrapolation requests (depending on implementation).
/// - Incorrect spline configurations or singularities in computation.
///
/// # Example Usage
///
/// ```rust
/// use optionstratlib::geometrics::SplineInterpolation;
/// use optionstratlib::error::InterpolationError;
/// use optionstratlib::curves::Point2D;
/// use rust_decimal::Decimal;///
///
/// use tracing::{error, info};
///
/// struct MySplineInterpolator {
///     data_points: Vec<Point2D>,
/// }
///
/// impl SplineInterpolation<Point2D, Decimal> for MySplineInterpolator {
///     fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
///         Ok(Point2D::new(x, x)) // Placeholder implementation
///     }
/// }
///
/// // Demonstration of usage (not runnable code)
/// fn example_usage() {
///     let interpolator = MySplineInterpolator {
///         data_points: vec![],
///     };
///
///     match interpolator.spline_interpolate(Decimal::new(10, 1)) {
///         Ok(point) => info!("Interpolated Point: {:?}", point),
///         Err(err) => error!("Interpolation failed: {:?}", err),
///     }
/// }
/// ```
///
/// # Related Concepts
/// - `LinearInterpolation`: Straight-line approximations between points.
/// - `CubicInterpolation`: For creating smooth curves with cubic polynomials.
/// - `BilinearInterpolation`: Interpolation techniques extended to two dimensions.
///
/// This trait is part of a broader framework supporting multiple interpolation techniques,
/// allowing developers to extend and choose specific methods as per the dataset requirements.
pub trait SplineInterpolation<Point, Input> {
    /// Interpolates a y-value for the provided x-coordinate using spline interpolation.
    ///
    /// - **Parameters:**
    ///   - `x`: The x-coordinate value of type `Input` for which interpolation is required.
    ///
    /// - **Returns:**
    ///   - `Ok(Point)`: The interpolated point, typically containing
    ///     both `x` and computed `y` values.
    ///   - `Err(Self::Error)`: Represents an error during the interpolation process.
    /// # Example
    /// ```rust
    /// use std::collections::BTreeSet;
    /// use rust_decimal::Decimal;
    /// use tracing::info;
    /// use optionstratlib::curves::{Curve, Point2D};
    /// use optionstratlib::geometrics::SplineInterpolation;
    /// let curve = Curve::new(BTreeSet::from_iter(vec![
    ///            Point2D::new(Decimal::ZERO, Decimal::ZERO),
    ///            Point2D::new(Decimal::ONE, Decimal::TWO),
    ///        ]));
    /// let result = curve.spline_interpolate(Decimal::from(2));
    ///
    /// match result {
    ///     Ok(point) => info!("Interpolated point: {:?}", point),
    ///     Err(e) => info!("Interpolation failed: {:?}", e),
    /// }
    /// ```
    fn spline_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
