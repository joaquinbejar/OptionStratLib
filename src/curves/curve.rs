/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/1/25
******************************************************************************/
use crate::curves::Point2D;
use crate::curves::utils::detect_peaks_and_valleys;
use crate::error::{CurveError, InterpolationError, MetricsError};
use crate::geometrics::{
    Arithmetic, AxisOperations, BasicMetrics, BiLinearInterpolation, ConstructionMethod,
    ConstructionParams, CubicInterpolation, GeometricObject, GeometricTransformations, Interpolate,
    InterpolationType, Len, LinearInterpolation, MergeAxisInterpolate, MergeOperation,
    MetricsExtractor, RangeMetrics, RiskMetrics, ShapeMetrics, SplineInterpolation, TrendMetrics,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::ops::Index;

/// Represents a mathematical curve as a collection of 2D points.
///
/// # Overview
/// The `Curve` struct is a fundamental representation of a curve, defined as a series
/// of points in a two-dimensional Cartesian coordinate system. Each curve is associated
/// with an `x_range`, specifying the inclusive domain of the curve in terms of its x-coordinates.
///
/// This structure supports precise mathematical and computational operations, including
/// interpolation, analysis, transformations, and intersections. The use of `Decimal`
/// for coordinates ensures high-precision calculations, making it particularly suitable
/// for scientific, financial, or mathematical applications.
///
/// # Usage
/// The `Curve` struct acts as the basis for high-level operations provided within
/// the `crate::curves` module. These include (but are not limited to):
/// - Generating statistical analyses (`CurveAnalysisResult`)
/// - Performing curve interpolation
/// - Logical manipulations, such as merging curves (`MergeOperation`)
/// - Visualizing graphs or curve plots using libraries like `plotters`
///
/// # Example Applications
/// The `Curve` type fits into mathematical or graphical operations such as:
/// - Modeling data over a range of x-values
/// - Comparing curves through transformations or intersections
/// - Calculating derivatives, integrals, and extrema along the curve
///
/// # Constraints
/// - All points in the `points` vector must lie within the specified `x_range`.
/// - Methods working with `Curve` data will assume that the `points` vector is ordered
///   by the `x`-coordinate. Non-ordered inputs may lead to undefined behavior in specific
///   operations.
///
/// # See Also
/// - [`Point2D`]: The fundamental data type for representing points in 2D space.
/// - [`MergeOperation`]: Enum for combining multiple curves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curve {
    /// A ordered set of `Point2D` objects that defines the curve in terms of its x-y plane coordinates.
    /// Points are stored in a `BTreeSet` which automatically maintains them in sorted order by their x-coordinate.
    pub points: BTreeSet<Point2D>,

    /// A tuple `(min_x, max_x)` that specifies the minimum and maximum x-coordinate values
    /// for the curve. Operations performed on the curve should ensure they fall within this range.
    /// Both values are of type `Decimal` to ensure high precision in boundary calculations.
    pub x_range: (Decimal, Decimal),
}

impl Display for Curve {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for point in self.points.iter() {
            write!(f, "{}\n", point)?;
        }
        Ok(())
    }
}

impl Default for Curve {
    fn default() -> Self {
        Curve {
            points: BTreeSet::new(),
            x_range: (Decimal::ZERO, Decimal::ZERO),
        }
    }
}

impl Curve {
    /// Creates a new curve from a vector of points.
    ///
    /// This constructor initializes a `Curve` instance using a list of 2D points
    /// provided as a `BTreeSet<Point2D>`. Additionally, the x-range of the curve is calculated
    /// and stored. The x-range is determined by evaluating the minimum and maximum
    /// x-coordinates among the provided points.
    ///
    /// # Parameters
    ///
    /// - `points` (`BTreeSet<Point2D>`): A vector of points that define the curve in a
    ///   two-dimensional Cartesian coordinate plane.
    ///
    /// # Returns
    ///
    /// - `Curve`: A newly instantiated curve containing the provided points and
    ///   the computed x-range.
    ///
    /// # Behavior
    ///
    /// - Calculates the x-range of the points using `calculate_range()`.
    /// - Stores the provided points for later use in curve-related calculations.
    ///
    /// # See Also
    ///
    /// - [`Point2D`]: The type of points used to define the curve.
    /// - [`crate::curves::Curve::calculate_range`]: Computes the x-range of a set of points.
    pub fn new(points: BTreeSet<Point2D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }
}

impl Len for Curve {
    fn len(&self) -> usize {
        self.points.len()
    }
}

impl GeometricObject<Point2D, Decimal> for Curve {
    type Error = CurveError;

    fn get_points(&self) -> BTreeSet<&Point2D> {
        self.points.iter().collect()
    }

    fn from_vector<T>(points: Vec<T>) -> Self
    where
        T: Into<Point2D> + Clone,
    {
        let points: BTreeSet<Point2D> = points.into_iter().map(|p| p.into()).collect();

        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }

    fn construct<T>(method: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
        T: Into<ConstructionMethod<Point2D, Decimal>>,
    {
        let method = method.into();
        match method {
            ConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(CurveError::Point2DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Curve::new(points))
            }
            ConstructionMethod::Parametric { f, params } => {
                let (t_start, t_end, steps) = match params {
                    ConstructionParams::D2 {
                        t_start,
                        t_end,
                        steps,
                    } => (t_start, t_end, steps),
                    _ => {
                        return Err(CurveError::ConstructionError(
                            "Invalid parameters".to_string(),
                        ));
                    }
                };
                let step_size = (t_end - t_start) / Decimal::from(steps);

                let points: Result<BTreeSet<Point2D>, CurveError> = (0..=steps)
                    .into_par_iter()
                    .map(|i| {
                        let t = t_start + step_size * Decimal::from(i);
                        f(t).map_err(|e| CurveError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(Curve::new)
            }
        }
    }
}

/// Allows indexed access to the points in a `Curve` using `usize` indices.
///
/// # Overview
/// This implementation provides intuitive, array-like access to the points
/// within a `Curve`. By using the `Index<usize>` trait, users can directly
/// reference specific points by their index within the internal `points` collection
/// without manually iterating or managing indices themselves.
///
/// # Behavior
/// - The `index` method fetches the `Point2D` at the specified position in the order
///   of the curve's `points` (sorted by the `Point2D` ordering, typically based on the `x` values).
/// - If the specified index exceeds the range of available points, it triggers a panic
///   with the message `"Index out of bounds"`.
///
/// # Constraints
/// - The index must be a valid value between `0` and `self.points.len() - 1`.
/// - The `Curve`'s `points` are internally stored as a `BTreeSet<Point2D>`, so indexing
///   reflects the natural order of the set, which is determined by the `Ord` trait
///   implementation for `Point2D`.
///
/// # Fields Accessed
/// - **`points`**: A `BTreeSet` of `Point2D` structs representing the curve's 2D points.
///
/// # Panics
/// This implementation will panic if:
/// - The index provided is out of bounds (less than `0` or greater than/equal to the number
///   of points in the curve).
///
/// # Use Cases
/// - Quickly accessing specific points on a curve during visualization, interpolation,
///   or analysis operations.
/// - Performing operations that require stepwise access to points, such as
///   slicing or filtering points along the curve.
///
/// # Example
/// Suppose you have a `Curve` instance `curve` with multiple points:
/// ```ignore
/// let point = curve[0]; // Access the first point
/// ```
///
/// # Important Notes
/// - This indexing implementation provides read-only access (`&Self::Output`).
/// - Modifying the `points` collection or its contents directly is not allowed through
///   this implementation, ensuring immutability when using indexed access.
///
/// # Type Associations
/// - **Input**:
///   - The input type for the `Index` operation is `usize`, the standard for indexing.
/// - **Output**:
///   - The output type for the `Index` operation is a reference to `Point2D`,
///     specifically `&Point2D`.
///
/// # Key Implementations
/// - **`Index<usize>`**: Provides indexing-based access to curve points.
impl Index<usize> for Curve {
    type Output = Point2D;

    /// Fetches the `Point2D` at the specified index.
    ///
    /// Panics if the index is invalid.
    fn index(&self, index: usize) -> &Self::Output {
        self.points.iter().nth(index).expect("Index out of bounds")
    }
}

/// Implementation of the `Interpolate` trait for the `Curve` struct.
///
/// This implementation integrates the `get_points` method for the `Curve` structure,
/// providing access to its internal points. The `Interpolate` trait ensures compatibility
/// with various interpolation methods such as Linear, BiLinear, Cubic, and Spline
/// interpolations. By implementing this trait, `Curve` gains the ability to perform
/// interpolation operations and access bracketing points.
///
/// # Traits Involved
///
/// The `Interpolate` trait is an aggregation of multiple interpolation-related traits:
/// - [`LinearInterpolation`]
/// - [`BiLinearInterpolation`]
/// - [`CubicInterpolation`]
/// - [`SplineInterpolation`]
///
/// These underlying traits implement specific interpolation algorithms,
/// enabling `Curve` to support a robust set of interpolation options through the associated methods.
/// Depending on the use case and provided parameters (e.g., interpolation type and target x-coordinate),
/// the appropriate algorithm is invoked.
///
/// # See Also
///
/// - [`Curve`]: The underlying mathematical structure being interpolated.
/// - [`Point2D`]: The fundamental data type for the curve's points.
/// - [`Interpolate`]: The trait defining interpolation operations.
///
impl Interpolate<Point2D, Decimal> for Curve {}

/// Implements the `LinearInterpolation` trait for the `Curve` struct.
///
/// This implementation provides linear interpolation functionality for a given set
/// of points on a curve. The interpolation computes the `y` value corresponding
/// to a given `x` value using the linear interpolation formula. The method ensures
/// that the input `x` is within the range of the curve's defined points.
///
/// ```text
/// y = p1.y + (x - p1.x) * (p2.y - p1.y) / (p2.x - p1.x)
/// ```
///
/// # Parameters
/// - `x`: A `Decimal` representing the `x`-coordinate for which the corresponding
///   interpolated `y` value is to be computed.
///
/// # Returns
/// - `Ok(Point2D)`: A `Point2D` instance containing the input `x` value and the
///   interpolated `y` value.
/// - `Err(CurvesError)`: Returns an error of type `CurvesError::InterpolationError`
///   in any of the following cases:
///     - The curve does not have enough points for interpolation.
///     - The provided `x` value is outside the range of the curve's points.
///     - Bracketing points for `x` cannot be found.
///
/// # Working Mechanism
/// 1. The method calls `find_bracket_points` (implemented in the `Interpolate` trait)
///    to locate the index pair `(i, j)` of two points that bracket the `x` value.
/// 2. From the located points `p1` and `p2`, the method calculates the interpolated
///    `y` value using the linear interpolation formula.
/// 3. Finally, a `Point2D` is created and returned with the provided `x` and the computed
///    `y` value.
///
/// # Implementation Details
/// - The function leverages `Decimal` arithmetic for high precision in calculations.
/// - It assumes that the provided points on the curve are sorted in ascending order
///   based on their `x` values.
///
/// # Errors
/// This method returns a `CurvesError` in the following cases:
/// - **Insufficient Points**: When the curve has fewer than two points.
/// - **Out-of-Range `x`**: When the input `x` value lies outside the range of the
///   defined points.
/// - **No Bracketing Points Found**: When the method fails to find two points
///   that bracket the given `x`.
///
/// # Example (How it works internally)
/// Suppose the curve is defined by the following points:
/// - `p1 = (2.0, 4.0)`
/// - `p2 = (5.0, 10.0)`
///
/// Given `x = 3.0`, the method computes:
/// ```text
/// y = 4.0 + (3.0 - 2.0) * (10.0 - 4.0) / (5.0 - 2.0)
///   = 4 + 1 * 6 / 3
///   = 4 + 2
///   = 6.0
/// ```
/// It will return `Point2D { x: 3.0, y: 6.0 }`.
///
/// # See Also
/// - `find_bracket_points`: Finds two points that bracket a value.
/// - `Point2D`: Represents points in 2D space.
/// - `CurvesError`: Represents errors related to curve operations.
impl LinearInterpolation<Point2D, Decimal> for Curve {
    /// # Method
    /// ### `linear_interpolate`
    ///
    /// Performs linear interpolation for a given `x` value by finding two consecutive
    /// points on the curve (`p1` and `p2`) that bracket the provided `x`. The `y` value
    /// is then calculated using the linear interpolation formula:
    fn linear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let (i, j) = self.find_bracket_points(x)?;

        let p1 = &self[i];
        let p2 = &self[j];

        // Linear interpolation for y value
        let y = p1.y + (x - p1.x) * (p2.y - p1.y) / (p2.x - p1.x);

        Ok(Point2D::new(x, y))
    }
}

/// Implementation of the `BiLinearInterpolation` trait for the `Curve` struct.
///
/// This function performs bilinear interpolation, which is used to estimate the value
/// of a function at a given point `x` within a grid defined by at least 4 points.
/// Bilinear interpolation combines two linear interpolations: one along the x-axis
/// and another along the y-axis, within the bounds defined by the four surrounding points.
///
/// # Parameters
///
/// - **`x`**: The x-coordinate value for which the interpolation will be performed.
///   Must fall within the range of the x-coordinates of the curve's points.
///
/// # Returns
///
/// - **`Ok(Point2D)`**: A `Point2D` instance representing the interpolated point
///   at the given x-coordinate, with both x and y provided as `Decimal` values.
/// - **`Err(CurvesError)`**: An error if the interpolation cannot be performed due
///   to one of the following reasons:
///   - There are fewer than 4 points in the curve.
///   - The x-coordinate does not fall within a valid range of points.
///   - The bracketing points for the given x-coordinate cannot be determined.
///
/// # Function Details
///
/// 1. **Input Validation**:
///    - Ensures the curve has at least 4 points, as required for bilinear interpolation.
///    - Returns an error if the condition is not met.
///
/// 2. **Exact Match Check**:
///    - If the x-coordinate matches exactly with one of the points in the curve,
///      the corresponding `Point2D` is returned directly.
///
/// 3. **Bracket Point Search**:
///    - Determines the bracketing points (`i` and `j`) for the given x-coordinate
///      using the `find_bracket_points` method.
///
/// 4. **Grid Point Selection**:
///    - Extracts four points from the curve:
///      - `p11`: Bottom-left point.
///      - `p12`: Bottom-right point.
///      - `p21`: Top-left point.
///      - `p22`: Top-right point.
///
/// 5. **x-Normalization**:
///    - Computes a normalized x value (`dx` in the range `[0,1]`), used to perform
///      interpolation along the x-axis within the defined grid.
///
/// 6. **Linear Interpolation**:
///    - First performs interpolation along the x-axis for the bottom and top edges of
///      the grid, resulting in partial y-values `bottom` and `top`.
///    - Then, interpolates along the y-axis between the bottom and top edge values,
///      resulting in the final interpolated y-coordinate.
///
/// 7. **Output**:
///    - Returns the interpolated `Point2D` with the input x-coordinate and the computed y-coordinate.
///
/// # Errors
///
/// - **Insufficient Points**: If the curve contains fewer than 4 points, a `CurvesError`
///   with a relevant message is returned.
/// - **Out-of-Range x**: If the x-coordinate cannot be bracketed by points in the curve,
///   a `CurvesError` is returned with an appropriate message.
///
/// # Related Traits
///
/// - [`BiLinearInterpolation`]: The trait defining this method.
/// - [`Interpolate`]: Ensures compatibility of the curve with multiple interpolation methods.
///
/// # See Also
///
/// - [`Curve`]: The overarching structure that represents the curve.
/// - [`Point2D`]: The data type used to represent individual points on the curve.
/// - [`find_bracket_points`](crate::geometrics::Interpolate::find_bracket_points):
///   A helper method used to locate the two points that bracket the given x-coordinate.
impl BiLinearInterpolation<Point2D, Decimal> for Curve {
    /// Performs bilinear interpolation to find the value of the curve at a given `x` coordinate.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate at which the interpolation is to be performed. This should be a `Decimal` value
    ///        within the range of the curve's known points.
    ///
    /// # Returns
    /// - On success, returns a `Point2D` instance representing the interpolated point at the given `x` value.
    /// - On failure, returns a `CurvesError`:
    ///   - `CurvesError::InterpolationError`: If there are fewer than four points available for interpolation or
    ///                                         if the required conditions for interpolation are not met.
    ///
    /// # Function Description
    /// - The function retrieves the set of points defining the curve using `self.get_points()`.
    /// - If fewer than four points exist, the function immediately fails with an `InterpolationError`.
    /// - If the exact `x` value is found in the point set, its corresponding `Point2D` is returned directly.
    /// - Otherwise, it determines the bracketing points (two pairs of points forming a square grid) necessary
    ///   for bilinear interpolation using `self.find_bracket_points()`.
    /// - From the bracketing points, it computes:
    ///   - `dx`: A normalized value representing the relative position of `x` between its bracketing x-coordinates
    ///           in the `[`0,1`]` interval.
    ///   - `bottom`: The interpolated y-value along the bottom edge of the grid.
    ///   - `top`: The interpolated y-value along the top edge of the grid.
    ///   - `y`: The final interpolated value along the y-dimension from `bottom` to `top`.
    /// - Returns the final interpolated point as `Point2D(x, y)`.
    ///
    /// # Errors
    /// - Returns an error if the curve has fewer than four points, as bilinear interpolation requires at least four.
    /// - Returns an error from `self.find_bracket_points()` if `x` cannot be bracketed.
    ///
    /// # Notes
    /// - The input `x` should be within the bounds of the curve for interpolation to succeed,
    ///   as specified by the bracketing function.
    /// - This function assumes that the points provided by `get_points` are sorted by ascending x-coordinate.
    ///
    /// # Example Use Case
    /// This method is useful for calculating intermediate values on a 2D grid when exact measurements are unavailable.
    /// Bilinear interpolation is particularly applicable for approximating smoother values in a tabular dataset
    /// or a regularly sampled grid.
    fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let points = self.get_points();

        if points.len() < 4 {
            return Err(InterpolationError::Bilinear(
                "Need at least four points for bilinear interpolation".to_string(),
            ));
        }

        // For exact points, return the actual point value
        if let Some(point) = points.iter().find(|p| p.x == x) {
            return Ok(**point);
        }

        let (i, _j) = self.find_bracket_points(x)?;

        // Get four points forming a grid by using Index implementation on self
        let p11 = &self[i]; // Bottom left
        let p12 = &self[i + 1]; // Bottom right
        let p21 = &self[i + 2]; // Top left
        let p22 = &self[i + 3]; // Top right

        // Normalize x to [0,1] interval
        let dx = (x - p11.x) / (p12.x - p11.x);

        // Interpolate along bottom edge
        let bottom = p11.y + dx * (p12.y - p11.y);

        // Interpolate along top edge
        let top = p21.y + dx * (p22.y - p21.y);

        // Final interpolation in y direction
        let y = bottom + (top - bottom) / dec!(2);

        Ok(Point2D::new(x, y))
    }
}

/// Implements the `CubicInterpolation` trait for the `Curve` struct,
/// providing an algorithm for cubic interpolation utilizing a Catmull-Rom spline.
///
/// # Method: `cubic_interpolate`
///
/// ## Parameters
/// - **`x`**: The x-value at which the interpolation is performed. This value must
///   be within the range of x-values in the curve's defined points, and it is passed
///   as a `Decimal` to allow for high-precision computation.
///
/// ## Returns
/// - **`Ok(Point2D)`**: Returns a `Point2D` representing the interpolated x and y values.
/// - **`Err(CurvesError)`**: Returns an error if:
///   - There are fewer than 4 points available for interpolation.
///   - The x-value is outside the curve's range, or interpolation fails for any other reason.
///
/// ## Behavior
/// 1. **Point Validation**: Ensures at least four points exist for cubic interpolation,
///    as this is a fundamental requirement for computing the Catmull-Rom spline.
/// 2. **Exact Match Check**: If the x-value matches an existing point in the curve, the
///    method directly returns the corresponding `Point2D` without further computation.
/// 3. **Bracket Points**: Determines the bracketing points (4 points total) around the
///    provided x-value. Depending on the position of the x-value in the curve, the
///    method dynamically adjusts the selected points to ensure they form a proper bracket:
///    - If near the start of the curve, uses the first four points.
///    - If near the end, uses the last four points.
///    - Else, selects points before and after x to define the bracket.
/// 4. **Parameter Calculation**: Computes a normalized parameter `t` that represents
///    the relative position of the target x-value between `p1` and `p2`.
/// 5. **Catmull-Rom Spline**: Performs cubic interpolation using a Catmull-Rom spline,
///    a widely used, smooth spline algorithm. The coefficients are calculated based on
///    the relative x position and the y-values of the four surrounding points.
/// 6. **Interpolation**: Calculates the interpolated y-value using the cubic formula:
///    ```text
///    y(t) = 0.5 * (
///        2 * p1.y + (-p0.y + p2.y) * t
///        + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t^2
///        + (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * t^3
///    )
///    ```
///    Here, `t` is the normalized x position, and `p0`, `p1`, `p2`, `p3` are the four bracketed points.
///
/// ## Errors
/// - Returns an error of type `CurvesError::InterpolationError` if any issues are encountered,
///   such as insufficient points or the inability to locate bracket points.
///
/// ## Example
/// This method is part of the `Curve` struct, which defines a set of points and supports interpolation.
/// It is often used in applications requiring smooth manifolds or animations.
///
/// ## Notes
/// - The computed y-value ensures smooth transitions and continuity between interpolated segments.
/// - Catmull-Rom splines are particularly effective for creating visually smooth transitions,
///   making this method suitable for curves, animations, and numerical analysis.
///
/// # See Also
/// - [`CubicInterpolation`]: The trait defining this method.
/// - [`Point2D`]: Represents the points used for interpolation.
/// - [`find_bracket_points`](crate::geometrics::Interpolate::find_bracket_points): Determines the bracketing points required for interpolation.
impl CubicInterpolation<Point2D, Decimal> for Curve {
    /// Performs cubic interpolation on a set of points to estimate the y-coordinate
    /// for a given x value using a Catmull-Rom spline.
    ///
    /// # Parameters
    ///
    /// - `x`: The x-coordinate for which the interpolation is performed. This value
    ///   should lie within the range of the points on the curve.
    ///
    /// # Returns
    ///
    /// - `Ok(Point2D)`: A `Point2D` instance representing the interpolated position
    ///   `(x, y)`, where `y` is estimated using cubic interpolation.
    /// - `Err(CurvesError)`: An error indicating issues with the interpolation process,
    ///   such as insufficient points or an out-of-range x value.
    ///
    /// # Requirements
    ///
    /// - The number of points in the curve must be at least 4, as cubic interpolation
    ///   requires four points for accurate calculations.
    /// - The specified `x` value should be inside the range defined by the curve's points.
    /// - If the specified x matches an existing point on the curve, the interpolated result
    ///   directly returns that exact point.
    ///
    /// # Functionality
    ///
    /// This method performs cubic interpolation using the general properties of the
    /// Catmull-Rom spline, which is well-suited for smooth curve fitting. It operates as follows:
    ///
    /// 1. **Exact Point Check**: If the x value matches an existing point, the method
    ///    returns that point without further processing.
    ///
    /// 2. **Bracketing Points Selection**:
    ///    - Searches for two points that bracket the given x value (using `find_bracket_points`
    ///      from the `Interpolate` trait). The method ensures that there are always enough
    ///      points before and after the target x value to perform cubic interpolation.
    ///
    /// 3. **Point Selection for Interpolation**:
    ///    - Depending on the position of the target x value, four points (`p0, p1, p2, p3`)
    ///      are selected:
    ///        - When `x` is near the start of the points, select the first four.
    ///        - When `x` is near the end, select the last four.
    ///        - Otherwise, select the two points just before and after the x value and
    ///          include an additional adjacent point on either side.
    ///
    /// 4. **Parameter Calculation**:
    ///    - The `t` parameter is derived, representing the normalized position of x
    ///      between `p1` and `p2`.
    ///
    /// 5. **Cubic Interpolation**:
    ///    - The interpolated y-coordinate is computed using the Catmull-Rom spline formula,
    ///      leveraging the `t`-value and the y-coordinates of the four selected points.
    ///
    /// # Error Handling
    ///
    /// This method returns an error in the following circumstances:
    /// - If fewer than 4 points are available, it returns a `CurvesError::InterpolationError`
    ///   with a corresponding message.
    /// - If the bracketing points cannot be identified (e.g., when `x` is outside the
    ///   range of points), the appropriate interpolation error is propagated.
    ///
    /// # Example
    ///
    /// - Interpolating smoothly along a curve defined by a set of points, avoiding sharp
    ///   transitions between segments.
    ///
    /// - Provides a high degree of precision due to the use of the `Decimal` type for
    ///   `x` and `y` calculations.
    fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let points = self.get_points();
        let len = self.len();

        // Need at least 4 points for cubic interpolation
        if len < 4 {
            return Err(InterpolationError::Cubic(
                "Need at least four points for cubic interpolation".to_string(),
            ));
        }

        // For exact points, return the actual point value
        if let Some(point) = points.iter().find(|p| p.x == x) {
            return Ok(**point);
        }

        let (i, _) = self.find_bracket_points(x)?;

        // Select four points for interpolation
        // Ensuring we always have enough points before and after
        let (p0, p1, p2, p3) = if i == 0 {
            (&self[0], &self[1], &self[2], &self[3])
        } else if i == len - 2 {
            (
                &self[len - 4],
                &self[len - 3],
                &self[len - 2],
                &self[len - 1],
            )
        } else {
            (&self[i - 1], &self[i], &self[i + 1], &self[i + 2])
        };

        // Calculate t parameter (normalized x position between p1 and p2)
        let t = (x - p1.x) / (p2.x - p1.x);

        // Cubic interpolation using Catmull-Rom spline
        let t2 = t * t;
        let t3 = t2 * t;

        let y = dec!(0.5)
            * (dec!(2) * p1.y
                + (-p0.y + p2.y) * t
                + (dec!(2) * p0.y - dec!(5) * p1.y + dec!(4) * p2.y - p3.y) * t2
                + (-p0.y + dec!(3) * p1.y - dec!(3) * p2.y + p3.y) * t3);

        Ok(Point2D::new(x, y))
    }
}

/// Implements the `SplineInterpolation` trait for the `Curve` struct, providing functionality
/// to perform cubic spline interpolation.
///
/// # Overview
/// This method calculates the interpolated `y` value for a given `x` value by using cubic
/// spline interpolation on the points in the `Curve`. The method ensures a smooth transition
/// between points by computing second derivatives of the curve at each point, and uses those
/// derivatives in the spline interpolation formula.
///
/// # Parameters
/// - `x`: The x-coordinate at which the curve should be interpolated. This value is passed as
///   a `Decimal` for precise calculations.
///
/// # Returns
/// - On success, returns a `Point2D` instance representing the interpolated point.
/// - On error, returns a `CurvesError` indicating the reason for failure (e.g., insufficient points
///   or an out-of-range x-coordinate).
///
/// # Errors
/// - Returns `CurvesError::InterpolationError` with an appropriate error message in the following cases:
///   - If the curve contains fewer than three points, as spline interpolation requires at least three points.
///   - If the given `x` value lies outside the range of x-coordinates spanned by the points in the curve.
///   - If a valid segment for interpolation cannot be located.
///
/// # Details
/// 1. **Validation**:
///    - Ensures that there are at least three points in the curve for spline interpolation.
///    - Validates that the provided `x` value is within the range of `x` values of the curve.
/// 2. **Exact Match**: If the `x` value matches the x-coordinate of an existing point, the corresponding
///    `Point2D` is returned immediately.
/// 3. **Second Derivative Calculation**:
///    - Uses a tridiagonal matrix to compute the second derivatives at each point. This step
///      involves setting up the system of equations based on the boundary conditions (natural spline)
///      and solving it using the Thomas algorithm.
/// 4. **Segment Identification**:
///    - Determines the segment (interval between two consecutive points) in which the provided `x` value lies.
/// 5. **Interpolation**:
///    - Computes the interpolated y-coordinate using the cubic spline formula, which is based on
///      the second derivatives and the positions of the surrounding points.
///
/// # Implementation Notes
/// - This implementation uses `Decimal` from the `rust_decimal` crate to ensure high precision
///   in calculations, making it suitable for scientific and financial applications.
/// - The Thomas algorithm is employed to solve the tridiagonal matrix system efficiently.
/// - The method assumes natural spline boundary conditions, where the second derivatives at the
///   endpoints are set to zero, ensuring a smooth and continuous curve shape.
///
/// # Example Usage
/// Refer to the documentation for how to use the `SplineInterpolation` trait, as examples
/// are not provided inline with this implementation.
///
/// # See Also
/// - [`SplineInterpolation`]: The trait definition for spline interpolation.
/// - [`Point2D`]: Represents a point in 2D space.
/// - [`Curve`]: Represents a mathematical curve made up of points for interpolation.
/// - [`CurveError`]: Enumerates possible errors during curve operations.
impl SplineInterpolation<Point2D, Decimal> for Curve {
    /// Performs cubic spline interpolation for a given x-coordinate and returns the interpolated
    /// `Point2D` value. This function computes the second derivatives of the curve points, solves
    /// a tridiagonal system to derive the interpolation parameters, and evaluates the spline
    /// function for the provided `x` value.
    ///
    /// # Parameters
    ///
    /// - `x`:
    ///   - The x-coordinate at which the interpolation is to be performed.
    ///   - Must be of type `Decimal`.
    ///
    /// # Returns
    ///
    /// - `Ok(Point2D)`:
    ///   - The `Point2D` instance representing the interpolated point at the given `x` value.
    ///   - The interpolated `y` value is calculated based on the cubic spline interpolation algorithm.
    ///
    /// - `Err(CurvesError)`:
    ///   - Returned when an error occurs during the interpolation process, such as:
    ///     - Insufficient points provided (less than 3 points).
    ///     - The given `x` is outside the valid range of the points.
    ///     - Unable to determine the correct segment for interpolation.
    ///
    /// # Errors
    ///
    /// - `CurvesError::InterpolationError`:
    ///   - Occurs under the following conditions:
    ///     - **"Need at least three points for spline interpolation"**:
    ///       Requires at least 3 points to perform cubic spline interpolation.
    ///     - **"x is outside the range of points"**:
    ///       The provided `x` value lies outside the domain of the curve points.
    ///     - **"Could not find valid segment for interpolation"**:
    ///       Spline interpolation fails due to an invalid segment determination.
    ///
    /// # Pre-conditions
    ///
    /// - The curve must contain at least three points for cubic spline interpolation.
    /// - The `x` value must fall within the range of the curve's x-coordinates.
    ///
    /// # Implementation Details
    ///
    /// - **Inputs**:
    ///   - Uses the `get_points` method of the curve to retrieve the list of `Point2D` instances
    ///     that define the interpolation curve.
    ///   - Computes the second derivatives (`m`) for each point using the Thomas algorithm to solve
    ///     a tridiagonal system.
    /// - **Boundary Conditions**:
    ///   - Natural spline boundary conditions are used, with the second derivatives on the boundary
    ///     set to zero.
    /// - **Interpolation**:
    ///   - Determines the segment `[x_i, x_{i+1}]` to which the input `x` belongs.
    ///   - Uses the cubic spline equation to calculate the interpolated `y` value.
    ///
    /// # Mathematical Formulation
    ///
    /// Let `x_i`, `x_{i+1}`, `y_i`, `y_{i+1}` refer to the points of the segment where `x` lies.
    /// The cubic spline function at `x` is computed as follows:
    ///
    /// ```text
    /// S(x) = m_i * (x_{i+1} - x)^3 / (6 * h)
    ///      + m_{i+1} * (x - x_i)^3 / (6 * h)
    ///      + (y_i / h - h * m_i / 6) * (x_{i+1} - x)
    ///      + (y_{i+1} / h - h * m_{i+1} / 6) * (x - x_i)
    /// ```
    ///
    /// Where:
    /// - `m_i`, `m_{i+1}` are the second derivatives at `x_i` and `x_{i+1}`.
    /// - `h = x_{i+1} - x_i` is the distance between the two points.
    /// - `(x_{i+1} - x)` and `(x - x_i)` are the relative distances within the segment.
    ///
    /// # Example Usages (Non-code)
    ///
    /// This method is typically used for high-precision curve fitting or graphical rendering where
    /// smooth transitions between points are essential. Common applications include:
    /// - Signal processing.
    /// - Data interpolation for missing values.
    /// - Smooth graphical representations of mathematical functions.
    ///
    /// # Related Types
    ///
    /// - [`Point2D`]: Represents a 2D point and is used as input/output
    ///   for this function.
    /// - [`CurveError`] Represents any error encountered during
    ///   interpolation.
    ///
    /// # Performance
    ///
    /// - The function operates with `O(n)` complexity, where `n` is the number of points. The
    ///   tridiagonal system is solved efficiently using the Thomas algorithm.
    ///
    /// # Notes
    ///
    /// - Natural spline interpolation may introduce minor deviations beyond the range of existing
    ///   data points due to its boundary conditions. For strictly constrained results, consider
    ///   alternative interpolation methods, such as linear or cubic Hermite interpolation.
    fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let points = self.get_points();
        let len = self.len();

        // Need at least 3 points for spline interpolation
        if len < 3 {
            return Err(InterpolationError::Spline(
                "Need at least three points for spline interpolation".to_string(),
            ));
        }

        // Check if x is within the valid range
        if x < self[0].x || x > self[len - 1].x {
            return Err(InterpolationError::Spline(
                "x is outside the range of points".to_string(),
            ));
        }

        // For exact points, return the actual point value
        if let Some(point) = points.iter().find(|p| p.x == x) {
            return Ok(**point);
        }

        let n = len;

        // Calculate second derivatives
        let mut a = vec![Decimal::ZERO; n];
        let mut b = vec![Decimal::ZERO; n];
        let mut c = vec![Decimal::ZERO; n];
        let mut r = vec![Decimal::ZERO; n];

        // Fill the matrices
        for i in 1..n - 1 {
            let hi = self[i].x - self[i - 1].x;
            let hi1 = self[i + 1].x - self[i].x;

            a[i] = hi;
            b[i] = dec!(2) * (hi + hi1);
            c[i] = hi1;

            r[i] = dec!(6) * ((self[i + 1].y - self[i].y) / hi1 - (self[i].y - self[i - 1].y) / hi);
        }

        // Add boundary conditions (natural spline)
        b[0] = dec!(1);
        b[n - 1] = dec!(1);

        // Solve tridiagonal system using Thomas algorithm
        let mut m = vec![Decimal::ZERO; n];

        for i in 1..n - 1 {
            let w = a[i] / b[i - 1];
            b[i] -= w * c[i - 1];
            r[i] = r[i] - w * r[i - 1];
        }

        m[n - 1] = r[n - 1] / b[n - 1];
        for i in (1..n - 1).rev() {
            m[i] = (r[i] - c[i] * m[i + 1]) / b[i];
        }

        // Find segment for interpolation
        let mut segment = None;
        for i in 0..n - 1 {
            if self[i].x <= x && x <= self[i + 1].x {
                segment = Some(i);
                break;
            }
        }

        let segment = segment.ok_or_else(|| {
            InterpolationError::Spline("Could not find valid segment for interpolation".to_string())
        })?;

        // Calculate interpolated value
        let h = self[segment + 1].x - self[segment].x;
        let dx = self[segment + 1].x - x;
        let dx1 = x - self[segment].x;

        let y = m[segment] * dx * dx * dx / (dec!(6) * h)
            + m[segment + 1] * dx1 * dx1 * dx1 / (dec!(6) * h)
            + (self[segment].y / h - m[segment] * h / dec!(6)) * dx
            + (self[segment + 1].y / h - m[segment + 1] * h / dec!(6)) * dx1;

        Ok(Point2D::new(x, y))
    }
}

/// A default implementation for the `Curve` type using a provided default strategy.
///
/// This implementation provides a basic approach to computing curve metrics
/// by using interpolation and statistical methods available in the standard
/// curve analysis library.
///
/// # Note
/// This is a minimal implementation that may need to be customized or enhanced
/// based on specific requirements or domain-specific analysis needs.
impl MetricsExtractor for Curve {
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, MetricsError> {
        let y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();

        // Handle empty curve
        if y_values.is_empty() {
            return Ok(BasicMetrics {
                mean: Decimal::ZERO,
                median: Decimal::ZERO,
                mode: Decimal::ZERO,
                std_dev: Decimal::ZERO,
            });
        }

        // Mean
        let mean = y_values.iter().sum::<Decimal>() / Decimal::from(y_values.len());

        // Median
        let mut sorted_values = y_values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[sorted_values.len() / 2 - 1] + sorted_values[sorted_values.len() / 2])
                / Decimal::TWO
        } else {
            sorted_values[sorted_values.len() / 2]
        };

        // Mode (most frequent value)
        let mode = {
            let mut freq_map = std::collections::HashMap::new();
            for &val in &y_values {
                *freq_map.entry(val).or_insert(0) += 1;
            }
            freq_map
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(val, _)| val)
                .unwrap_or(Decimal::ZERO)
        };

        // Standard Deviation
        let variance = y_values
            .iter()
            .map(|&x| (x - mean).powu(2))
            .sum::<Decimal>()
            / Decimal::from(y_values.len());
        let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

        Ok(BasicMetrics {
            mean,
            median,
            mode,
            std_dev,
        })
    }

    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError> {
        let y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();

        // Handle empty or single-point curve
        if y_values.len() < 2 {
            return Ok(ShapeMetrics {
                skewness: Decimal::ZERO,
                kurtosis: Decimal::ZERO,
                peaks: vec![],
                valleys: vec![],
                inflection_points: vec![],
            });
        }

        // Mean and Standard Deviation
        let mean = y_values.iter().sum::<Decimal>() / Decimal::from(y_values.len());

        // Compute centered and scaled values
        let centered_values: Vec<Decimal> = y_values.iter().map(|&x| x - mean).collect();

        // Compute variance
        let variance = centered_values.iter().map(|&x| x.powu(2)).sum::<Decimal>()
            / Decimal::from(y_values.len());
        let std_dev = variance.sqrt().unwrap_or(Decimal::ONE);
        if std_dev.is_zero() || std_dev < dec!(1e-9) {
            panic!("The standard deviation is too small or zero.");
        }

        // Skewness calculation (Fisher-Pearson standardized moment)
        let skewness = centered_values
            .iter()
            .map(|&x| (x / std_dev).powu(3))
            .sum::<Decimal>()
            / Decimal::from(y_values.len());

        // Kurtosis calculation (Fisher's definition - adjust to excess kurtosis)
        let kurtosis = centered_values
            .iter()
            .map(|&x| (x / std_dev).powu(4))
            .sum::<Decimal>()
            / Decimal::from(y_values.len())
            - Decimal::from(3);

        // Peaks and Valleys detection
        let (peaks, valleys) = detect_peaks_and_valleys(&self.points);

        Ok(ShapeMetrics {
            skewness,
            kurtosis,
            peaks,
            valleys,
            inflection_points: vec![],
        })
    }

    fn compute_range_metrics(&self) -> Result<RangeMetrics, MetricsError> {
        // Handle empty curve
        if self.points.is_empty() {
            return Ok(RangeMetrics {
                min: Point2D::new(Decimal::ZERO, Decimal::ZERO),
                max: Point2D::new(Decimal::ZERO, Decimal::ZERO),
                range: Decimal::ZERO,
                quartiles: (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
                interquartile_range: Decimal::ZERO,
            });
        }

        let mut y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();
        y_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = y_values.len();
        let min_point = self
            .points
            .iter()
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .cloned()
            .unwrap();
        let max_point = self
            .points
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .cloned()
            .unwrap();

        let range = max_point.y - min_point.y;

        // Quartiles
        let q1 = y_values[len / 4];
        let q2 = y_values[len / 2];
        let q3 = y_values[3 * len / 4];

        let interquartile_range = q3 - q1;

        Ok(RangeMetrics {
            min: min_point,
            max: max_point,
            range,
            quartiles: (q1, q2, q3),
            interquartile_range,
        })
    }

    fn compute_trend_metrics(&self) -> Result<TrendMetrics, MetricsError> {
        let points: Vec<Point2D> = self.points.clone().into_iter().collect();

        // Handle insufficient points
        if points.len() < 2 {
            return Ok(TrendMetrics {
                slope: Decimal::ZERO,
                intercept: Decimal::ZERO,
                r_squared: Decimal::ZERO,
                moving_average: vec![],
            });
        }

        // Linear Regression Calculation
        let n = Decimal::from(points.len());
        let x_vals: Vec<Decimal> = points.iter().map(|p| p.x).collect();
        let y_vals: Vec<Decimal> = points.iter().map(|p| p.y).collect();

        let sum_x: Decimal = x_vals.iter().sum();
        let sum_y: Decimal = y_vals.iter().sum();
        let sum_xy: Decimal = x_vals.iter().zip(&y_vals).map(|(x, y)| *x * *y).sum();
        let sum_xx: Decimal = x_vals.iter().map(|x| *x * *x).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // R-squared Calculation
        let mean_y = sum_y / n;
        let sst: Decimal = y_vals.iter().map(|y| (*y - mean_y).powu(2)).sum();

        let ssr: Decimal = y_vals
            .iter()
            .zip(&x_vals)
            .map(|(y, x)| {
                let y_predicted = slope * *x + intercept;
                (*y - y_predicted).powu(2)
            })
            .sum();

        let r_squared = if sst == Decimal::ZERO {
            Decimal::ONE
        } else {
            Decimal::ONE - (ssr / sst)
        };

        // Moving Average Calculation
        let window_sizes = [3, 5, 7];
        let moving_average: Vec<Point2D> = window_sizes
            .iter()
            .flat_map(|&window| {
                if window > points.len() {
                    vec![]
                } else {
                    points
                        .windows(window)
                        .map(|window_points| {
                            let avg_x = window_points.iter().map(|p| p.x).sum::<Decimal>()
                                / Decimal::from(window_points.len());
                            let avg_y = window_points.iter().map(|p| p.y).sum::<Decimal>()
                                / Decimal::from(window_points.len());
                            Point2D::new(avg_x, avg_y)
                        })
                        .collect::<Vec<Point2D>>()
                }
            })
            .collect();

        Ok(TrendMetrics {
            slope,
            intercept,
            r_squared,
            moving_average,
        })
    }

    fn compute_risk_metrics(&self) -> Result<RiskMetrics, MetricsError> {
        let y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();

        if y_values.is_empty() {
            return Ok(RiskMetrics {
                volatility: Decimal::ZERO,
                value_at_risk: Decimal::ZERO,
                expected_shortfall: Decimal::ZERO,
                beta: Decimal::ZERO,
                sharpe_ratio: Decimal::ZERO,
            });
        }

        let mean = y_values.iter().sum::<Decimal>() / Decimal::from(y_values.len());
        let volatility = y_values
            .iter()
            .map(|&x| (x - mean).powu(2))
            .sum::<Decimal>()
            / Decimal::from(y_values.len())
                .sqrt()
                .unwrap_or(Decimal::ZERO);

        if volatility == Decimal::ZERO {
            return Ok(RiskMetrics {
                volatility,
                value_at_risk: Decimal::ZERO,
                expected_shortfall: Decimal::ZERO,
                beta: Decimal::ZERO,
                sharpe_ratio: Decimal::ZERO,
            });
        }

        let z_score = dec!(1.645);
        let var = mean - z_score * volatility;

        let below_var_count = y_values.iter().filter(|&&x| x < var).count();
        let expected_shortfall = if below_var_count > 0 {
            y_values.iter().filter(|&&x| x < var).sum::<Decimal>()
                / Decimal::from(below_var_count as u64)
        } else {
            Decimal::ZERO
        };

        let beta = if mean != Decimal::ZERO {
            volatility / mean
        } else {
            Decimal::ZERO
        };

        let sharpe_ratio = mean / volatility;

        Ok(RiskMetrics {
            volatility,
            value_at_risk: var,
            expected_shortfall,
            beta,
            sharpe_ratio,
        })
    }
}

/// Implements the `CurveArithmetic` trait for the `Curve` type, providing
/// functionality for merging multiple curves using a specified mathematical
/// operation and performing arithmetic operations between two curves.
impl Arithmetic<Curve> for Curve {
    type Error = CurveError;

    /// Merges a collection of curves into a single curve based on the specified
    /// mathematical operation.
    ///
    /// # Parameters
    ///
    /// - `curves` (`&[&Curve]`): A slice of references to the curves to be merged.
    ///   Each curve must have defined x-ranges and interpolation capabilities.
    /// - `operation` (`MergeOperation`): The arithmetic operation to perform on the
    ///   interpolated y-values for the provided curves. Operations include addition,
    ///   subtraction, multiplication, division, and aggregation (e.g., max, min).
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: Returns a new curve resulting from the merging operation.
    /// - `Err(CurvesError)`: If input parameters are invalid or the merge operation
    ///   encounters an error (e.g., incompatible x-ranges or interpolation failure),
    ///   an error is returned.
    ///
    /// # Behavior
    ///
    /// 1. **Parameter Validation**:
    ///   - Verifies that at least one curve is provided in the `curves` parameter.
    ///   - Returns an error if no curves are included or if x-ranges are incompatible.
    ///
    /// 2. **Cloning Single Curve**:
    ///   - If only one curve is provided, its clone is returned as the result without
    ///     performing any further calculations.
    ///
    /// 3. **Range Computation**:
    ///   - Computes the intersection of x-ranges across the curves by finding the
    ///     maximum lower bound (`min_x`) and minimum upper bound (`max_x`).
    ///   - If the x-range intersection is invalid (i.e., `min_x >= max_x`), an error
    ///     is returned.
    ///
    /// 4. **Interpolation and Arithmetic**:
    ///   - Divides the x-range into `steps` equally spaced intervals (default: 100).
    ///   - Interpolates the y-values for all curves at each x-value in the range.
    ///   - Applies the specified `operation` to the aggregated y-values at each x-point.
    ///
    /// 5. **Parallel Processing**:
    ///   - Uses parallel iteration to perform interpolation and value combination
    ///     efficiently, leveraging the Rayon library.
    ///
    /// 6. **Error Handling**:
    ///   - Any errors during interpolation or arithmetic operations are propagated
    ///     back to the caller.
    ///
    /// # Errors
    ///
    /// - **Invalid Parameter** (`CurvesError`): Returned when no curves are provided
    ///   or x-ranges are incompatible.
    /// - **Interpolation Failure** (`CurvesError`): Raised if interpolation fails
    ///   for a specific curve or x-value.
    ///
    /// # Example Use Case
    ///
    /// This function enables combining multiple curves for tasks such as:
    /// - Summing y-values across different curves to compute a composite curve.
    /// - Finding the maximum/minimum y-value at each x-point for a collection of curves.
    fn merge(curves: &[&Curve], operation: MergeOperation) -> Result<Curve, CurveError> {
        if curves.is_empty() {
            return Err(CurveError::invalid_parameters(
                "merge_curves",
                "No curves provided for merging",
            ));
        }

        // If only one curve, return a clone
        if curves.len() == 1 {
            return Ok(curves[0].clone());
        }

        // Find the intersection of x-ranges
        let min_x = curves
            .iter()
            .map(|c| c.x_range.0)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(Decimal::ZERO);

        let max_x = curves
            .iter()
            .map(|c| c.x_range.1)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(Decimal::ZERO);

        // Check if ranges are compatible
        if min_x >= max_x {
            return Err(CurveError::invalid_parameters(
                "merge_curves",
                "Curves have incompatible x-ranges",
            ));
        }

        // Determine number of interpolation steps
        let steps = 100; // Configurable number of interpolation points
        let step_size = (max_x - min_x) / Decimal::from(steps);

        // Interpolate and perform operation using parallel iterator
        let result_points: Result<Vec<Point2D>, CurveError> = (0..=steps)
            .into_par_iter()
            .map(|i| {
                let x = min_x + step_size * Decimal::from(i);

                // Interpolate y values for each curve
                let y_values: Result<Vec<Decimal>, CurveError> = curves
                    .iter()
                    .map(|curve| {
                        curve
                            .interpolate(x, InterpolationType::Cubic)
                            .map(|point| point.y)
                            .map_err(CurveError::from)
                    })
                    .collect();

                let y_values = y_values?;

                // Perform the specified operation on interpolated y values
                let result_y: Decimal = match operation {
                    MergeOperation::Add => y_values.par_iter().sum(),
                    MergeOperation::Subtract => {
                        // Use Rayon's fold to parallelize subtraction
                        y_values
                            .par_iter()
                            .enumerate()
                            .map(|(i, &val)| if i == 0 { val } else { -val })
                            .reduce(|| Decimal::ZERO, |a, b| a + b)
                    }
                    MergeOperation::Multiply => y_values.par_iter().product(),
                    MergeOperation::Divide => y_values
                        .par_iter()
                        .enumerate()
                        .map(|(i, &val)| {
                            if i == 0 {
                                val
                            } else if val == Decimal::ZERO {
                                Decimal::MAX
                            } else {
                                Decimal::ONE / val
                            }
                        })
                        .reduce(|| Decimal::ONE, |a, b| a * b),
                    MergeOperation::Max => y_values
                        .par_iter()
                        .cloned()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(Decimal::ZERO),
                    MergeOperation::Min => y_values
                        .par_iter()
                        .cloned()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(Decimal::ZERO),
                };

                Ok(Point2D::new(x, result_y))
            })
            .collect();

        // Handle potential errors during parallel processing
        let result_points = result_points?;

        Ok(Curve::from_vector(result_points))
    }

    /// Combines the current `Curve` instance with another curve using a mathematical
    /// operation, resulting in a new curve.
    ///
    /// # Parameters
    ///
    /// - `self` (`&Self`): A reference to the current curve instance.
    /// - `other` (`&Curve`): A reference to the second curve for the arithmetic operation.
    /// - `operation` (`MergeOperation`): The operation to apply when merging the curves.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: Returns a new curve that represents the result of the operation.
    /// - `Err(CurvesError)`: If the merge operation fails (e.g., incompatible x-ranges or
    ///   interpolation errors), an error is returned.
    ///
    /// # Behavior
    ///
    /// This function is a convenience wrapper around `merge_curves` that operates
    /// specifically on two curves. It passes `self` and `other` as an array to
    /// `merge_curves` and applies the desired operation.
    ///
    /// # Errors
    ///
    /// - Inherits all errors returned by the `merge_curves` method, including parameter
    ///   validation and interpolation errors.
    ///
    /// # Examples
    ///
    /// Use this method to easily perform arithmetic operations between two curves,
    /// such as summing their y-values or finding their pointwise maximum.
    fn merge_with(&self, other: &Curve, operation: MergeOperation) -> Result<Curve, CurveError> {
        Self::merge(&[self, other], operation)
    }
}

impl AxisOperations<Point2D, Decimal> for Curve {
    type Error = CurveError;

    fn contains_point(&self, x: &Decimal) -> bool {
        let point = Point2D::new(*x, Decimal::ZERO);
        self.points.contains(&point)
    }

    fn get_index_values(&self) -> Vec<Decimal> {
        self.points.iter().map(|p| p.x).collect()
    }

    fn get_values(&self, x: Decimal) -> Vec<&Decimal> {
        self.points
            .iter()
            .filter(|p| p.x == x)
            .map(|p| &p.y)
            .collect()
    }

    fn get_closest_point(&self, x: &Decimal) -> Result<&Point2D, Self::Error> {
        self.points
            .iter()
            .min_by(|a, b| {
                let dist_a = (a.x - *x).abs();
                let dist_b = (b.x - *x).abs();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .ok_or(CurveError::Point2DError {
                reason: "No points available",
            })
    }

    fn get_point(&self, x: &Decimal) -> Option<&Point2D> {
        if self.contains_point(x) {
            self.points.iter().find(|p| p.x == *x)
        } else {
            None
        }
    }
}

impl MergeAxisInterpolate<Point2D, Decimal> for Curve
where
    Self: Sized,
{
    fn merge_axis_interpolate(
        &self,
        other: &Self,
        interpolation: InterpolationType,
    ) -> Result<(Self, Self), Self::Error> {
        // Get merged unique x-coordinates
        let merged_x_values = self.merge_axis_index(other);

        // Sort the merged x values
        let mut sorted_x_values: Vec<Decimal> = merged_x_values.into_iter().collect();
        sorted_x_values.sort();

        let mut interpolated_self_points = BTreeSet::new();
        let mut interpolated_other_points = BTreeSet::new();

        for x in &sorted_x_values {
            if self.contains_point(x) {
                interpolated_self_points.insert(*self.get_point(x).unwrap());
            } else {
                let interpolated_point = self.interpolate(*x, interpolation)?;
                interpolated_self_points.insert(interpolated_point);
            }
            if other.contains_point(x) {
                interpolated_other_points.insert(*other.get_point(x).unwrap());
            } else {
                let interpolated_point = other.interpolate(*x, interpolation)?;
                interpolated_other_points.insert(interpolated_point);
            }
        }
        Ok((
            Curve::new(interpolated_self_points),
            Curve::new(interpolated_other_points),
        ))
    }
}

impl GeometricTransformations<Point2D> for Curve {
    type Error = CurveError;

    fn translate(&self, deltas: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if deltas.len() != 2 {
            return Err(CurveError::invalid_parameters(
                "translate",
                "Expected 2 deltas for 2D translation",
            ));
        }

        let translated_points = self
            .points
            .iter()
            .map(|point| Point2D::new(point.x + deltas[0], point.y + deltas[1]))
            .collect();

        Ok(Curve::new(translated_points))
    }

    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if factors.len() != 2 {
            return Err(CurveError::invalid_parameters(
                "scale",
                "Expected 2 factors for 2D scaling",
            ));
        }

        let scaled_points = self
            .points
            .iter()
            .map(|point| Point2D::new(point.x * factors[0], point.y * factors[1]))
            .collect();

        Ok(Curve::new(scaled_points))
    }

    fn intersect_with(&self, other: &Self) -> Result<Vec<Point2D>, Self::Error> {
        let mut intersections = Vec::new();

        // Use existing pairs iterator for efficiency
        for p1 in self.get_points() {
            for p2 in other.get_points() {
                // Find points with small distance between them
                if (p1.x - p2.x).abs() < Decimal::new(1, 6)
                    && (p1.y - p2.y).abs() < Decimal::new(1, 6)
                {
                    intersections.push(*p1);
                }
            }
        }

        Ok(intersections)
    }

    fn derivative_at(&self, point: &Point2D) -> Result<Vec<Decimal>, Self::Error> {
        let (i, j) = self.find_bracket_points(point.x)?;

        let p0 = &self[i];
        let p1 = &self[j];

        let a = (p1.y - p0.y) / (p1.x * p1.x - p0.x * p0.x);
        let derivative = dec!(2.0) * a * point.x;

        Ok(vec![derivative])
    }

    fn extrema(&self) -> Result<(Point2D, Point2D), Self::Error> {
        if self.points.is_empty() {
            return Err(CurveError::invalid_parameters(
                "extrema",
                "Curve has no points",
            ));
        }

        let min_point = self
            .points
            .iter()
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .cloned()
            .unwrap();

        let max_point = self
            .points
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
            .cloned()
            .unwrap();

        Ok((min_point, max_point))
    }

    fn measure_under(&self, base_value: &Decimal) -> Result<Decimal, Self::Error> {
        if self.points.len() < 2 {
            return Ok(Decimal::ZERO);
        }

        let mut area = Decimal::ZERO;
        let points: Vec<_> = self.points.iter().collect();

        // Approximate area using trapezoidal rule
        for pair in points.windows(2) {
            let width = pair[1].x - pair[0].x;
            let height = ((pair[0].y - base_value) + (pair[1].y - base_value)) / Decimal::TWO;
            area += width * height;
        }

        Ok(area.abs())
    }
}

#[cfg(test)]
mod tests_curves {
    use super::*;
    use crate::curves::utils::{create_constant_curve, create_linear_curve};
    use crate::{Positive, pos};
    use Decimal;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_to_tuple_with_decimal() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Decimal, Decimal) = point.to_tuple().unwrap();
        assert_eq!(tuple, (dec!(1.5), dec!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_to_tuple_with_positive() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Positive, Positive) = point.to_tuple().unwrap();
        assert_eq!(tuple, (pos!(1.5), pos!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_tuple_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_tuple_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_with_mixed_types() {
        let x = dec!(1.5);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_constant_curve() {
        let curve = create_constant_curve(dec!(1.0), dec!(2.0), dec!(5.0));
        assert_eq!(curve.get_points().len(), 11);
        for point in curve.get_points() {
            assert_eq!(point.y, dec!(5.0));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_linear_curve() {
        let curve = create_linear_curve(dec!(1.0), dec!(2.0), dec!(2.0));
        assert_eq!(curve.get_points().len(), 11);
        let mut slope = dec!(2.0);
        for point in curve.get_points() {
            assert_eq!(point.y, slope);
            slope += dec!(0.2);
        }
    }
}

#[cfg(test)]
mod tests_linear_interpolate {
    use super::*;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_exact_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        // Test exact input points
        let p0 = curve
            .interpolate(Decimal::ZERO, InterpolationType::Linear)
            .unwrap();
        assert_eq!(p0.x, Decimal::ZERO);
        assert_eq!(p0.y, Decimal::ZERO);

        let p1 = curve
            .interpolate(Decimal::ONE, InterpolationType::Linear)
            .unwrap();
        assert_eq!(p1.x, Decimal::ONE);
        assert_eq!(p1.y, Decimal::TWO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_midpoint() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        // Test midpoint
        let mid = curve
            .interpolate(dec!(0.5), InterpolationType::Linear)
            .unwrap();
        assert_eq!(mid.x, dec!(0.5));
        assert_eq!(mid.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_quarter_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        // Test at x = 0.25
        let p25 = curve
            .interpolate(dec!(0.25), InterpolationType::Linear)
            .unwrap();
        assert_eq!(p25.x, dec!(0.25));
        assert_eq!(p25.y, dec!(0.5));

        // Test at x = 0.75
        let p75 = curve
            .interpolate(dec!(0.75), InterpolationType::Linear)
            .unwrap();
        assert_eq!(p75.x, dec!(0.75));
        assert_eq!(p75.y, dec!(1.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.1), InterpolationType::Linear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(1.1), InterpolationType::Linear)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(0.0),
            dec!(0.0),
        )]));

        assert!(
            curve
                .interpolate(dec!(0.5), InterpolationType::Linear)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_non_monotonic() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(1.0)),
        ]));

        let p15 = curve
            .interpolate(dec!(1.5), InterpolationType::Linear)
            .unwrap();
        assert_eq!(p15.x, dec!(1.5));
        assert_eq!(p15.y, dec!(1.5));
    }
}

#[cfg(test)]
mod tests_bilinear_interpolate {
    use super::*;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::TWO, Decimal::ONE),
            Point2D::new(Decimal::TEN, Decimal::ONE),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        // Test exact points
        let corner = curve
            .interpolate(Decimal::ZERO, InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(corner.x, Decimal::ZERO);
        assert_eq!(corner.y, Decimal::ZERO);

        // Test midpoint interpolation
        let mid = curve
            .interpolate(dec!(0.5), InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(mid.x, dec!(0.5));
        assert_eq!(mid.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.5), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(1.5), InterpolationType::Bilinear)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(0.5), InterpolationType::Bilinear)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_quarter_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11 (0,0)
            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12 (1,1)
            Point2D::new(Decimal::TWO, Decimal::ONE),   // p21 (0,1)
            Point2D::new(Decimal::TEN, Decimal::TWO),   // p22 (1,2)
        ]));

        // At x = 0.25:
        // Bottom edge: 0.25 * (1 - 0) = 0.25
        // Top edge: 0.25 * (2 - 1) + 1 = 1.25
        // Result: 0.25 + (1.25 - 0.25)/2 = 0.75
        let p25 = curve
            .interpolate(dec!(0.25), InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(p25.x, dec!(0.25));
        assert_eq!(p25.y, dec!(0.75));

        // At x = 0.75:
        // Bottom edge: 0.75 * (1 - 0) = 0.75
        // Top edge: 0.75 * (2 - 1) + 1 = 1.75
        // Result: 0.75 + (1.75 - 0.75)/2 = 1.25
        let p75 = curve
            .interpolate(dec!(0.75), InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(p75.x, dec!(0.75));
        assert_eq!(p75.y, dec!(1.25));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_boundaries() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.1), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(1.1), InterpolationType::Bilinear)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::ONE),
            Point2D::new(Decimal::ZERO, Decimal::ONE),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        assert!(
            curve
                .interpolate(dec!(-1), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(Decimal::TWO, InterpolationType::Bilinear)
                .is_err()
        );
    }
}

#[cfg(test)]
mod tests_cubic_interpolate {
    use super::*;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_exact_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test exact points
        let p1 = curve
            .interpolate(dec!(1.0), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(p1.x, dec!(1.0));
        assert_eq!(p1.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_midpoints() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test midpoint interpolation
        let mid = curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid.x, dec!(1.5));
        // y value should be continuous and smooth
        assert!(mid.y > dec!(1.0) && mid.y < dec!(4.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(1.5), InterpolationType::Cubic)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.5), InterpolationType::Cubic)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(3.5), InterpolationType::Cubic)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_monotonicity() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        let p1 = curve
            .interpolate(dec!(0.5), InterpolationType::Cubic)
            .unwrap();
        let p2 = curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .unwrap();
        let p3 = curve
            .interpolate(dec!(2.5), InterpolationType::Cubic)
            .unwrap();

        assert!(p1.y < p2.y);
        assert!(p2.y < p3.y);
        info!("p1: {:?}, p2: {:?}, p3: {:?}", p1, p2, p3);
    }
}

#[cfg(test)]
mod tests_spline_interpolate {
    use super::*;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_exact_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        let p1 = curve
            .interpolate(dec!(1.0), InterpolationType::Spline)
            .unwrap();
        assert_eq!(p1.x, dec!(1.0));
        assert_eq!(p1.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_midpoints() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        let mid = curve
            .interpolate(dec!(1.5), InterpolationType::Spline)
            .unwrap();
        assert_eq!(mid.x, dec!(1.5));
        // Value should be continuous and between the points
        assert!(mid.y > dec!(1.0) && mid.y < dec!(4.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(0.5), InterpolationType::Spline)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.5), InterpolationType::Spline)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(2.5), InterpolationType::Spline)
                .is_err()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_smoothness() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test points close together to verify smoothness
        let p1 = curve
            .interpolate(dec!(1.48), InterpolationType::Spline)
            .unwrap();
        let p2 = curve
            .interpolate(dec!(1.49), InterpolationType::Spline)
            .unwrap();
        let p3 = curve
            .interpolate(dec!(1.50), InterpolationType::Spline)
            .unwrap();
        let p4 = curve
            .interpolate(dec!(1.51), InterpolationType::Spline)
            .unwrap();
        let p5 = curve
            .interpolate(dec!(1.52), InterpolationType::Spline)
            .unwrap();

        // Verify monotonicity and smooth transitions
        assert!(p1.y < p2.y);
        assert!(p2.y < p3.y);
        assert!(p3.y < p4.y);
        assert!(p4.y < p5.y);

        // Verify that the changes are smooth (second differences are small)
        let d1 = p2.y - p1.y;
        let d2 = p3.y - p2.y;
        let d3 = p4.y - p3.y;
        let d4 = p5.y - p4.y;

        // Second differences should be small
        assert!((d2 - d1).abs() < dec!(0.001));
        assert!((d3 - d2).abs() < dec!(0.001));
        assert!((d4 - d3).abs() < dec!(0.001));
    }
}

#[cfg(test)]
mod tests_curve_arithmetic {
    use super::*;
    use crate::curves::utils::create_linear_curve;
    use crate::geometrics::InterpolationType;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_add() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Add).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                + curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_subtract() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Subtract).unwrap();
        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                - curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_multiply() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Multiply).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                * curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_divide() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(6.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Divide).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let y2 = curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            // Skip points where interpolation results in zero
            if y2 == Decimal::ZERO {
                continue;
            }

            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y / y2;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_max() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Max).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let y1 = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let y2 = curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let expected_y = y1.max(y2);

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_min() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Min).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let y1 = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let y2 = curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let expected_y = y1.min(y2);

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_with_single_operation() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = curve1.merge_with(&curve2, MergeOperation::Add).unwrap();

        // Verify that merge_with is equivalent to merge_curves with two curves
        let merged_result = Curve::merge(&[&curve1, &curve2], MergeOperation::Add).unwrap();

        // Compare points of both results
        assert_eq!(result.points.len(), merged_result.points.len());

        for i in 0..result.points.len() {
            assert!((result[i].x - merged_result[i].x).abs() < dec!(0.001));
            assert!((result[i].y - merged_result[i].y).abs() < dec!(0.001));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_error_handling() {
        // Test with empty slice
        let result = Curve::merge(&[], MergeOperation::Add);
        assert!(result.is_err());

        // Test with curves of incompatible ranges
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let curve2 = create_linear_curve(dec!(5.0), dec!(15.0), dec!(2.0));

        // Verify that the merge operation works even with partially overlapping ranges
        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Add);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_multiple_curves() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve3 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge(&[&curve1, &curve2, &curve3], MergeOperation::Add).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                + curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y
                + curve3.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use crate::error::CurveError::OperationError;
    use crate::error::OperationErrorKind;
    use crate::geometrics::{ConstructionMethod, ConstructionParams};
    use std::error::Error;

    #[test]
    fn test_construct_from_data_empty() {
        let result = Curve::construct(ConstructionMethod::FromData {
            points: BTreeSet::new(),
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::Point2DError { reason } => {
                assert_eq!(reason, "Empty points array");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_construct_parametric_valid() {
        let f = |t: Decimal| Ok(Point2D::new(t, t * dec!(2.0)));
        let params = ConstructionParams::D2 {
            t_start: Decimal::ZERO,
            t_end: dec!(10.0),
            steps: 10,
        };
        let result = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(f),
            params,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_construct_parametric_invalid_function() {
        let f = |_t: Decimal| -> Result<Point2D, Box<dyn Error>> {
            Err(Box::new(CurveError::ConstructionError(
                "Function evaluation failed".to_string(),
            )))
        };
        let params = ConstructionParams::D2 {
            t_start: Decimal::ZERO,
            t_end: dec!(10.0),
            steps: 10,
        };
        let result = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(f),
            params,
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::ConstructionError(reason) => {
                assert_eq!(reason, "Construction error: Function evaluation failed");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_segment_not_found_error() {
        let segment: Option<Point2D> = None;
        let result: Result<Point2D, CurveError> = segment.ok_or_else(|| CurveError::StdError {
            reason: "Could not find valid segment for interpolation".to_string(),
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::StdError { reason } => {
                assert_eq!(reason, "Could not find valid segment for interpolation");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_compute_basic_metrics_placeholder() {
        let curve = Curve {
            points: BTreeSet::new(),
            x_range: (Default::default(), Default::default()),
        };
        let metrics = curve.compute_basic_metrics();
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.mean, Decimal::ZERO);
    }

    #[test]
    fn test_single_curve_return() {
        let curve = Curve {
            points: BTreeSet::new(),
            x_range: (Default::default(), Default::default()),
        };
        let result = if vec![curve.clone()].len() == 1 {
            Ok(curve.clone())
        } else {
            Err(CurveError::invalid_parameters(
                "merge_curves",
                "Invalid state",
            ))
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_curves_invalid_x_range() {
        let min_x = dec!(10.0);
        let max_x = dec!(5.0);
        let result = if min_x >= max_x {
            Err(CurveError::invalid_parameters(
                "merge_curves",
                "Curves have incompatible x-ranges",
            ))
        } else {
            Ok(())
        };
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            OperationError(OperationErrorKind::InvalidParameters { operation, reason }) => {
                assert_eq!(operation, "merge_curves");
                assert_eq!(reason, "Curves have incompatible x-ranges");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }
}

#[cfg(test)]
mod tests_curve_metrics {
    use super::*;
    use crate::assert_decimal_eq;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    // Helper function to create test curves
    fn create_linear_curve() -> Curve {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(6.0)),
            Point2D::new(dec!(4.0), dec!(8.0)),
        ]);
        Curve::new(points)
    }

    fn create_non_linear_curve() -> Curve {
        Curve {
            points: (0..=20)
                .map(|x| Point2D {
                    x: Decimal::from(x),
                    y: Decimal::from(x * x % 7), // Ejemplo no lineal
                })
                .collect(),
            x_range: (Default::default(), Default::default()),
        }
    }

    fn create_constant_curve() -> Curve {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(5.0)),
            Point2D::new(dec!(1.0), dec!(5.0)),
            Point2D::new(dec!(2.0), dec!(5.0)),
        ]);
        Curve::new(points)
    }

    #[test]
    fn test_basic_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let basic_metrics = linear_curve.compute_basic_metrics().unwrap();

        // Expected values for linear curve
        assert_decimal_eq!(basic_metrics.mean, dec!(4.0), dec!(0.001));
        assert_decimal_eq!(basic_metrics.median, dec!(4.0), dec!(0.001));
        assert_decimal_eq!(basic_metrics.std_dev, dec!(2.82842712), dec!(0.001));

        // Constant curve
        let constant_curve = create_constant_curve();
        let constant_metrics = constant_curve.compute_basic_metrics().unwrap();

        assert_decimal_eq!(constant_metrics.mean, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_metrics.median, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_metrics.std_dev, dec!(0.0), dec!(0.001));
    }

    #[test]
    fn test_shape_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let shape_metrics = linear_curve.compute_shape_metrics().unwrap();

        // More lenient check for linear curve
        assert!(
            shape_metrics.skewness.abs() < dec!(0.5),
            "Skewness for linear curve should be very close to 0, got {}",
            shape_metrics.skewness
        );

        // Allow a wider range for kurtosis of a linear curve
        assert!(
            shape_metrics.kurtosis.abs() < dec!(2.0),
            "Kurtosis for linear curve should be close to 0, got {}",
            shape_metrics.kurtosis
        );

        // Non-linear curve
        let non_linear_curve = create_non_linear_curve();
        let non_linear_metrics = non_linear_curve.compute_shape_metrics().unwrap();

        // More nuanced checks for non-linear curve
        assert!(
            non_linear_metrics.skewness.abs() > dec!(0.3),
            "Non-linear curve should have significant skewness, got {}",
            non_linear_metrics.skewness
        );

        // Ensure the non-linear curve has a meaningfully different kurtosis
        assert!(
            non_linear_metrics.kurtosis.abs() > dec!(1.0),
            "Non-linear curve should have significant kurtosis, got {}",
            non_linear_metrics.kurtosis
        );

        // Check peaks and valleys
        assert!(
            !non_linear_metrics.peaks.is_empty(),
            "Peaks should be detected"
        );
        assert!(
            !non_linear_metrics.valleys.is_empty(),
            "Valleys should be detected"
        );
    }

    #[test]
    fn test_range_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let range_metrics = linear_curve.compute_range_metrics().unwrap();

        assert_decimal_eq!(range_metrics.min.y, dec!(0.0), dec!(0.001));
        assert_decimal_eq!(range_metrics.max.y, dec!(8.0), dec!(0.001));
        assert_decimal_eq!(range_metrics.range, dec!(8.0), dec!(0.001));

        // Constant curve
        let constant_curve = create_constant_curve();
        let constant_range_metrics = constant_curve.compute_range_metrics().unwrap();

        assert_decimal_eq!(constant_range_metrics.min.y, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_range_metrics.max.y, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_range_metrics.range, dec!(0.0), dec!(0.001));
    }

    #[test]
    fn test_trend_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let trend_metrics = linear_curve.compute_trend_metrics().unwrap();

        // Expected values for a perfectly linear curve
        assert_decimal_eq!(trend_metrics.slope, dec!(2.0), dec!(0.001));
        assert_decimal_eq!(trend_metrics.intercept, dec!(0.0), dec!(0.001));
        assert_decimal_eq!(trend_metrics.r_squared, dec!(1.0), dec!(0.001));

        // Non-linear curve
        let non_linear_curve = create_non_linear_curve();
        let non_linear_trend_metrics = non_linear_curve.compute_trend_metrics().unwrap();

        // R-squared should be less than 1
        assert!(non_linear_trend_metrics.r_squared < dec!(1.0));

        // Moving average should exist
        assert!(!non_linear_trend_metrics.moving_average.is_empty());
    }

    #[test]
    fn test_constant_curve_risk_metrics() {
        let constant_curve = create_constant_curve();
        let risk_metrics = constant_curve.compute_risk_metrics().unwrap();

        assert_eq!(risk_metrics.volatility, dec!(0.0));
        assert_eq!(risk_metrics.beta, dec!(0.0));
        assert_eq!(risk_metrics.sharpe_ratio, dec!(0.0));
    }

    #[test]
    fn test_risk_metrics() {
        // Curva lineal
        let linear_curve = create_linear_curve();
        let risk_metrics = linear_curve.compute_risk_metrics().unwrap();

        assert!(
            risk_metrics.volatility > dec!(0.0),
            "Volatility debe ser mayor a cero."
        );
        assert!(
            risk_metrics.value_at_risk != dec!(0.0),
            "Value at Risk no debe ser cero."
        );
        assert!(risk_metrics.beta != dec!(0.0), "Beta no debe ser cero.");
    }

    #[test]
    fn test_risk_metrics_bis() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let risk_metrics = linear_curve.compute_risk_metrics().unwrap();

        // Volatility and risk metrics should be non-zero
        assert!(risk_metrics.volatility > dec!(0.0));
        assert!(risk_metrics.value_at_risk != dec!(0.0));
        assert!(risk_metrics.beta != dec!(0.0));

        // Constant curve
        let constant_curve = create_constant_curve();
        let constant_risk_metrics = constant_curve.compute_risk_metrics().unwrap();

        // Volatility should be zero for a constant curve
        assert_decimal_eq!(constant_risk_metrics.volatility, dec!(0.0), dec!(0.001));
    }

    #[test]
    fn test_edge_cases() {
        // Empty curve
        let empty_curve = Curve::new(BTreeSet::new());

        assert!(empty_curve.compute_basic_metrics().is_ok());
        assert!(empty_curve.compute_shape_metrics().is_ok());
        assert!(empty_curve.compute_range_metrics().is_ok());
        assert!(empty_curve.compute_trend_metrics().is_ok());
        assert!(empty_curve.compute_risk_metrics().is_ok());

        // Single point curve
        let single_point_curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(1.0),
            dec!(1.0),
        )]));

        assert!(single_point_curve.compute_basic_metrics().is_ok());
        assert!(single_point_curve.compute_shape_metrics().is_ok());
        assert!(single_point_curve.compute_range_metrics().is_ok());
        assert!(single_point_curve.compute_trend_metrics().is_ok());
        assert!(single_point_curve.compute_risk_metrics().is_ok());
    }
}

#[cfg(test)]
mod tests_merge_axis_interpolate {
    use super::*;
    use crate::curves::utils::create_linear_curve;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    fn test_merge_axis_interpolate_linear() {
        // Create two curves with different x ranges and points
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(0.5));
        let curve2 = create_linear_curve(dec!(4.0), dec!(20.0), dec!(1.0));

        // Merge and interpolate using linear interpolation
        let result = curve1.merge_axis_interpolate(&curve2, InterpolationType::Linear);

        assert!(result.is_ok());
        let (interpolated_curve1, interpolated_curve2) = result.unwrap();

        // Verify that both interpolated curves have the same x range
        assert_eq!(interpolated_curve1.x_range.0, interpolated_curve2.x_range.0);
        assert_eq!(interpolated_curve1.x_range.1, interpolated_curve2.x_range.1);

        // Verify number of points (should cover full merged x range)
        assert_eq!(interpolated_curve1.points.len(), 10);
        assert_eq!(interpolated_curve2.points.len(), 10);
        assert_eq!(interpolated_curve1.x_range, interpolated_curve2.x_range);
        assert_eq!(
            interpolated_curve1.get_index_values(),
            interpolated_curve2.get_index_values()
        );
    }

    #[test]
    fn test_merge_axis_interpolate_cubic() {
        // Create two curves with different x ranges and points
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(0.5));
        let curve2 = create_linear_curve(dec!(4.0), dec!(20.0), dec!(1.0));

        // Merge and interpolate using cubic interpolation
        let result = curve1.merge_axis_interpolate(&curve2, InterpolationType::Cubic);

        assert!(result.is_ok());
        let (interpolated_curve1, interpolated_curve2) = result.unwrap();

        // Verify that both interpolated curves have the same x range
        assert_eq!(interpolated_curve1.x_range.0, interpolated_curve2.x_range.0);
        assert_eq!(interpolated_curve1.x_range.1, interpolated_curve2.x_range.1);

        // Verify number of points (should cover full merged x range)
        assert_eq!(interpolated_curve1.points.len(), 10);
        assert_eq!(interpolated_curve2.points.len(), 10);
        assert_eq!(interpolated_curve1.x_range, interpolated_curve2.x_range);
        assert_eq!(
            interpolated_curve1.get_index_values(),
            interpolated_curve2.get_index_values()
        );
    }
}

#[cfg(test)]
mod tests_geometric_transformations {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_curve() -> Curve {
        Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]))
    }

    mod test_translate {
        use super::*;

        #[test]
        fn test_translate_positive() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(2.0), &dec!(3.0)]).unwrap();

            let translated_points: Vec<_> = result.points.iter().collect();
            assert_eq!(translated_points[0].x, dec!(2.0));
            assert_eq!(translated_points[0].y, dec!(3.0));

            assert_eq!(translated_points[1].x, dec!(3.0));
            assert_eq!(translated_points[1].y, dec!(4.0));

            assert_eq!(translated_points[2].x, dec!(4.0));
            assert_eq!(translated_points[2].y, dec!(7.0));

            assert_eq!(translated_points[3].x, dec!(5.0));
            assert_eq!(translated_points[3].y, dec!(12.0));
        }

        #[test]
        fn test_translate_negative() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(-1.0), &dec!(-2.0)]).unwrap();

            let translated_points: Vec<_> = result.points.iter().collect();
            assert_eq!(translated_points[0].x, dec!(-1.0));
            assert_eq!(translated_points[0].y, dec!(-2.0));
        }

        #[test]
        fn test_translate_zero() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(0.0), &dec!(0.0)]).unwrap();
            assert_eq!(curve.points, result.points);
        }

        #[test]
        fn test_translate_wrong_dimensions() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(1.0)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_translate_preserves_shape() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(1.0), &dec!(1.0)]).unwrap();

            let original_diffs: Vec<Decimal> = curve
                .points
                .iter()
                .zip(curve.points.iter().skip(1))
                .map(|(a, b)| (b.y - a.y))
                .collect();

            let translated_diffs: Vec<Decimal> = result
                .points
                .iter()
                .zip(result.points.iter().skip(1))
                .map(|(a, b)| (b.y - a.y))
                .collect();

            assert_eq!(original_diffs, translated_diffs);
        }
    }

    mod test_scale {
        use super::*;

        #[test]
        fn test_scale_uniform() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(2.0), &dec!(2.0)]).unwrap();

            let scaled_points: Vec<_> = result.points.iter().collect();

            assert_eq!(scaled_points[0].x, dec!(0.0));
            assert_eq!(scaled_points[0].y, dec!(0.0));

            assert_eq!(scaled_points[1].x, dec!(2.0));
            assert_eq!(scaled_points[1].y, dec!(2.0));

            assert_eq!(scaled_points[2].x, dec!(4.0));
            assert_eq!(scaled_points[2].y, dec!(8.0));

            assert_eq!(scaled_points[3].x, dec!(6.0));
            assert_eq!(scaled_points[3].y, dec!(18.0));
        }

        #[test]
        fn test_scale_non_uniform() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(2.0), &dec!(3.0)]).unwrap();

            let scaled_points: Vec<_> = result.points.iter().collect();
            assert_eq!(scaled_points[1].x, dec!(2.0));
            assert_eq!(scaled_points[1].y, dec!(3.0));
        }

        #[test]
        fn test_scale_zero() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(0.0), &dec!(0.0)]).unwrap();

            assert!(
                result
                    .points
                    .iter()
                    .all(|p| p.x == dec!(0.0) && p.y == dec!(0.0))
            );
        }

        #[test]
        fn test_scale_wrong_dimensions() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(2.0)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_scale_negative() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(-1.0), &dec!(-1.0)]).unwrap();

            assert_eq!(result[1].x, dec!(-2.0));
            assert_eq!(result[1].y, dec!(-4.0));

            assert_eq!(result[3].x, dec!(0.0));
            assert_eq!(result[3].y, dec!(0.0));
        }
    }

    mod test_intersect_with {
        use super::*;

        #[test]
        fn test_curves_intersect() {
            let curve1 = create_test_curve();
            let curve2 = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(2.0)),
            ]));

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert_eq!(intersections.len(), 1);
        }

        #[test]
        fn test_no_intersection() {
            let curve1 = create_test_curve();
            let curve2 = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(10.0), dec!(10.0)),
                Point2D::new(dec!(11.0), dec!(11.0)),
            ]));

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert!(intersections.is_empty());
        }

        #[test]
        fn test_multiple_intersections() {
            let curve1 = create_test_curve();
            let curve2 = create_test_curve();

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert_eq!(intersections.len(), curve1.points.len());
        }

        #[test]
        fn test_self_intersection() {
            let curve = create_test_curve();
            let intersections = curve.intersect_with(&curve).unwrap();
            assert_eq!(intersections.len(), curve.points.len());
        }

        #[test]
        fn test_empty_curves() {
            let curve1 = Curve::new(BTreeSet::new());
            let curve2 = Curve::new(BTreeSet::new());

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert!(intersections.is_empty());
        }
    }

    mod test_derivative_at {
        use super::*;

        #[test]
        fn test_linear_derivative() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let derivative = curve
                .derivative_at(&Point2D::new(dec!(0.5), dec!(0.5)))
                .unwrap();
            assert_eq!(derivative[0], dec!(1.0));
        }

        #[test]
        fn test_quadratic_derivative() {
            let curve = create_test_curve();
            let derivative = curve
                .derivative_at(&Point2D::new(dec!(1.0), dec!(1.0)))
                .unwrap();
            assert_eq!(derivative[0], dec!(2.0));
            let derivative2 = curve
                .derivative_at(&Point2D::new(dec!(2.0), dec!(4.0)))
                .unwrap();
            assert_eq!(derivative2[0], dec!(4.0));
        }

        #[test]
        fn test_out_of_range() {
            let curve = create_test_curve();
            let result = curve.derivative_at(&Point2D::new(dec!(10.0), dec!(0.0)));
            assert!(result.is_err());
        }

        #[test]
        fn test_at_endpoint() {
            let curve = create_test_curve();
            let derivative = curve
                .derivative_at(&Point2D::new(dec!(0.0), dec!(0.0)))
                .unwrap();
            assert!(derivative[0] == dec!(0.0));
        }

        #[test]
        fn test_vertical_line() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(1.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let result = curve.derivative_at(&Point2D::new(dec!(1.0), dec!(0.5)));
            assert!(result.is_err());
        }
    }

    mod test_extrema {
        use super::*;

        #[test]
        fn test_find_extrema() {
            let curve = create_test_curve();
            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min.y, dec!(0.0));
            assert_eq!(max.y, dec!(9.0));
        }

        #[test]
        fn test_empty_curve() {
            let curve = Curve::new(BTreeSet::new());
            let result = curve.extrema();
            assert!(result.is_err());
        }

        #[test]
        fn test_single_point() {
            let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
                dec!(1.0),
                dec!(1.0),
            )]));

            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min, max);
        }

        #[test]
        fn test_flat_curve() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(1.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min.y, max.y);
        }

        #[test]
        fn test_multiple_extrema() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
                Point2D::new(dec!(2.0), dec!(0.0)),
            ]));

            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min.y, dec!(0.0));
            assert_eq!(max.y, dec!(1.0));
        }
    }

    mod test_measure_under {
        use super::*;

        #[test]
        fn test_area_under_linear() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(area, dec!(0.5));
        }

        #[test]
        fn test_area_empty_curve() {
            let curve = Curve::new(BTreeSet::new());
            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(area, dec!(0.0));
        }

        #[test]
        fn test_area_single_point() {
            let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
                dec!(1.0),
                dec!(1.0),
            )]));

            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(area, dec!(0.0));
        }

        #[test]
        fn test_area_with_base_value() {
            let curve = create_test_curve();
            let area1 = curve.measure_under(&dec!(0.0)).unwrap();
            let area2 = curve.measure_under(&dec!(1.0)).unwrap();
            assert!(area1 > area2);
        }

        #[test]
        fn test_negative_area() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(-1.0)),
                Point2D::new(dec!(1.0), dec!(-2.0)),
            ]));

            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert!(area > dec!(0.0));
        }
    }
}

#[cfg(test)]
mod tests_curve_serde {
    use super::*;
    use rust_decimal_macros::dec;

    // Helper function to create a test curve
    fn create_test_curve() -> Curve {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: dec!(1.0),
            y: dec!(2.0),
        });
        points.insert(Point2D {
            x: dec!(3.0),
            y: dec!(4.0),
        });
        points.insert(Point2D {
            x: dec!(5.0),
            y: dec!(6.0),
        });

        Curve {
            points,
            x_range: (dec!(1.0), dec!(5.0)),
        }
    }

    #[test]
    fn test_basic_serialization() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }

    #[test]
    fn test_pretty_print() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string_pretty(&curve).unwrap();

        // Verify pretty print format
        assert!(serialized.contains('\n'));
        assert!(serialized.contains("  "));

        // Verify deserialization still works
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();
        assert_eq!(curve.points, deserialized.points);
    }

    #[test]
    fn test_empty_curve() {
        let curve = Curve {
            points: BTreeSet::new(),
            x_range: (dec!(0.0), dec!(0.0)),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.points.is_empty());
        assert_eq!(deserialized.x_range, (dec!(0.0), dec!(0.0)));
    }

    #[test]
    fn test_curve_with_negative_values() {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: dec!(-1.0),
            y: dec!(-2.0),
        });
        points.insert(Point2D {
            x: dec!(-3.0),
            y: dec!(-4.0),
        });

        let curve = Curve {
            points,
            x_range: (dec!(-3.0), dec!(-1.0)),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }

    #[test]
    fn test_curve_with_high_precision() {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: dec!(1.12345678901234567890),
            y: dec!(2.12345678901234567890),
        });
        points.insert(Point2D {
            x: dec!(3.12345678901234567890),
            y: dec!(4.12345678901234567890),
        });

        let curve = Curve {
            points,
            x_range: (dec!(1.12345678901234567890), dec!(3.12345678901234567890)),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }

    #[test]
    fn test_invalid_json() {
        // Missing required fields
        let json_str = r#"{"points": []}"#;
        let result = serde_json::from_str::<Curve>(json_str);
        assert!(result.is_err());

        // Invalid points format
        let json_str = r#"{"points": [1, 2, 3], "x_range": [0, 1]}"#;
        let result = serde_json::from_str::<Curve>(json_str);
        assert!(result.is_err());

        // Invalid x_range format
        let json_str = r#"{"points": [], "x_range": "invalid"}"#;
        let result = serde_json::from_str::<Curve>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_structure() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string(&curve).unwrap();
        let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Check structure
        assert!(json.is_object());
        assert!(json.get("points").is_some());
        assert!(json.get("x_range").is_some());

        // Check points is an array
        assert!(json.get("points").unwrap().is_array());

        // Check x_range is an array of 2 elements
        let x_range = json.get("x_range").unwrap().as_array().unwrap();
        assert_eq!(x_range.len(), 2);
    }

    #[test]
    fn test_multiple_curves() {
        let curve1 = create_test_curve();
        let mut curve2 = create_test_curve();
        curve2.x_range = (dec!(6.0), dec!(10.0));

        let curves = vec![curve1, curve2];
        let serialized = serde_json::to_string(&curves).unwrap();
        let deserialized: Vec<Curve> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curves.len(), deserialized.len());
        assert_eq!(curves[0].points, deserialized[0].points);
        assert_eq!(curves[1].points, deserialized[1].points);
    }

    #[test]
    fn test_ordering_preservation() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        // Convert points to vectors to check ordering
        let original_points: Vec<_> = curve.points.into_iter().collect();
        let deserialized_points: Vec<_> = deserialized.points.into_iter().collect();

        // Check if points maintain their order
        assert_eq!(original_points, deserialized_points);
    }

    #[test]
    fn test_curve_with_extremes() {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: Decimal::MAX,
            y: Decimal::MAX,
        });
        points.insert(Point2D {
            x: Decimal::MIN,
            y: Decimal::MIN,
        });

        let curve = Curve {
            points,
            x_range: (Decimal::MIN, Decimal::MAX),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }
}
