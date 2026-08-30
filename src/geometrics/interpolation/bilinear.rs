use crate::error::InterpolationError;

/// A trait for bilinear interpolation on 2D data grids.
///
/// # Purpose
/// The `BiLinearInterpolation` trait is designed to perform bilinear interpolation
/// to estimate intermediate values within a grid of 2D points. This method is often
/// used in numerical computation tasks, such as image processing, terrain modeling,
/// and scientific data visualization.
///
/// # Type Parameters
/// - `Point`: The output type, typically used to represent the interpolated 2D point.
/// - `Input`: The input type for the interpolation parameter, typically a scalar value.
///
/// # Associated Type
/// - `Error`: Defines the type returned in case of a failure during interpolation.
///
/// # Method
/// - [`bilinear_interpolate`](#method.bilinear_interpolate):
///   Computes the interpolated value at the given input, returning either the result
///   or an error if the operation cannot proceed.
///
/// # Errors
/// Any errors encountered during interpolation are encapsulated in the type `Error`.
/// This trait is expected to return meaningful errors in cases like:
/// - Insufficient or invalid data for computation.
/// - Inputs that are out of bounds for the given dataset.
/// - Issues specific to the interpolation logic.
///
/// # Example Usage
/// Below is an example demonstrating how an implementing struct might use this trait:
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Point2D;
/// use optionstratlib::error::InterpolationError;
/// use optionstratlib::geometrics::BiLinearInterpolation;
/// struct GridInterpolator {
///     // Implementation-specific fields like the grid or data.
/// }
///
/// impl BiLinearInterpolation<Point2D, Decimal> for GridInterpolator {
///
///     fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
///         Ok(Point2D::new(x, x)) // Placeholder implementation
///     }
/// }
/// ```
///
/// In this example:
/// - `GridInterpolator` implements the trait for bilinear interpolation.
/// - The `bilinear_interpolate` method calculates the interpolated `Point2D` for a given `x` value.
///
/// # Related Types
/// - [`Point2D`](crate::curves::Point2D): A struct representing a 2D point with `x` and `y` coordinates.
/// - [`CurvesError`](crate::error::CurveError): A recommended error type for detailed error categorization.
///
/// # See Also
/// - [`crate::geometrics::interpolation::InterpolationType`](crate::geometrics::InterpolationType):
///   A module defining different types of interpolation methods.
/// - [`crate::geometrics::interpolation::LinearInterpolation`](crate::geometrics::LinearInterpolation):
///   A simpler interpolation method for one-dimensional data.
///
/// # One point per abscissa
///
/// Every interpolator here reads its sample as a function of the abscissa:
/// at most one ordinate per `x`. See
/// [`Curve::new`](crate::curves::Curve::new) for the rule and for why no
/// constructor enforces it. A sample carrying several ordinates at one
/// abscissa is outside the contract: the grid cell around a repeated
/// abscissa has zero width and the slope across it is undefined, so the
/// answer is [`InterpolationError::DegenerateInterval`].
///
/// That holds on every branch, including the exact match: the
/// [`Curve`](crate::curves::Curve) implementation short-circuits an `x` that
/// lands on a sample, and on a stack of ordinates at that `x` it reports the
/// degeneracy rather than returning the lowest of them.
///
/// A projection of a surface is multi-valued by construction, which is why
/// [`Surface::project_onto`](crate::surfaces::Surface::project_onto) returns a
/// `Vec<Point2D>` and not a curve; aggregate it before interpolating.
///
/// # Cells at the edge of the sample
///
/// A cell wants samples on both sides of the query along every axis, which
/// the samples at the upper edge of the domain cannot supply. The
/// [`Curve`](crate::curves::Curve) implementation builds its cell out of two
/// consecutive segments and clamps the far one to the curve's last segment
/// where it would run off the end, so the segment holding the query is
/// always one edge of the cell and the answer stays a convex combination of
/// the four ordinates read; that implementation documents the rule and its
/// consequences in full. The [`Surface`](crate::surfaces::Surface) one picks
/// the four samples nearest the query, so its cell has no end to run off.
pub trait BiLinearInterpolation<Point, Input> {
    /// Performs bilinear interpolation to compute a value for the given input.
    ///
    /// # Parameters
    /// - `x`: The input value (e.g., an `x` coordinate in 2D space) for which the interpolation is performed.
    ///
    /// # Returns
    /// - `Ok(Point)`: The interpolated point (e.g., a `Point2D`) representing the computed values.
    /// - `Err(Self::Error)`: An error indicating why the interpolation could not be performed.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::BTreeSet;
    /// use rust_decimal::Decimal;
    /// use tracing::info;
    /// use optionstratlib::curves::{Curve, Point2D};
    /// use optionstratlib::geometrics::BiLinearInterpolation;
    /// let curve = Curve::new(BTreeSet::from_iter(vec![
    ///            Point2D::new(Decimal::ZERO, Decimal::ZERO),
    ///            Point2D::new(Decimal::ONE, Decimal::TWO),
    ///        ]));
    /// let result = curve.bilinear_interpolate(Decimal::from(2));
    ///
    /// match result {
    ///     Ok(point) => info!("Interpolated point: {:?}", point),
    ///     Err(e) => info!("Interpolation failed: {:?}", e),
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`InterpolationError::EmptyData`] when the grid has
    /// no samples, [`InterpolationError::OutOfRange`] when `x` falls
    /// outside the covered domain,
    /// [`InterpolationError::DegenerateInterval`] when the
    /// neighbouring grid cell has zero width on either axis, and
    /// [`InterpolationError::Bilinear`] when the sample is too small to build
    /// a cell from (fewer than four points on a
    /// [`Curve`](crate::curves::Curve)), or when a checked-arithmetic step
    /// leaves the representable range.
    fn bilinear_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
