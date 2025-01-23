/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/1/25
******************************************************************************/
use std::collections::BTreeSet;
use crate::curves::{Curve, Point2D};
use crate::geometrics::GeometricObject;
use rust_decimal::Decimal;

/// Creates a linear curve defined by a starting point, an ending point, and a slope.
///
/// This function generates a 2-dimensional curve by calculating evenly spaced points
/// (10 intervals by default) between the `start` and `end` x-coordinates. For each
/// generated x-coordinate, the corresponding y-coordinate is computed using the provided
/// slope, following the equation:
///
/// ```text
/// y = slope * x
/// ```
///
/// The generated points are then used to construct a `Curve` instance.
///
/// # Parameters
/// - `start`: The starting x-coordinate of the curve (as a `Decimal`).
/// - `end`: The ending x-coordinate of the curve (as a `Decimal`).
///   - Must be greater than the `start` value for the function to work as intended.
/// - `slope`: The slope of the linear curve, which determines the relationship between
///   x and y values.
///
/// # Returns
/// A `Curve` instance containing evenly spaced points along the linear curve determined
/// by the specified parameters.
///
/// # Behavior
/// - The x-coordinates are computed as evenly spaced values between `start` and `end`
///   across 10 steps. Each x-coordinate includes its corresponding `y` value determined
///   by the slope.
/// - Internally uses `Point2D::new` to construct points based on the computed x- and
///   y-coordinate values.
/// - Constructs the final curve using `Curve::from_vector`, with the computed points
///   forming the curve.
///
/// # Constraints
/// - The `end` value must be greater than the `start` value; otherwise, the generated
///   points will result in an incorrect or potentially invalid curve.
/// - The function uses a fixed number (10) of steps to divide the range between `start`
///   and `end`. This ensures uniform spacing between points but limits flexibility
///   for other resolutions.
///
/// # Example Workflow (Internal Overview)
/// 1. Divide the range `[start, end]` into 10 equal steps (`step_size`).
/// 2. Iteratively compute `(x, y)` points using the formula `y = slope * x`.
/// 3. Accumulate these points into a `Vec<Point2D>`.
/// 4. Construct the final `Curve` using `Curve::from_vector`.
///
/// # Usage Notes
/// - This function is best suited for applications requiring a simple linear curve
///   representation between two bounds.
/// - For higher resolution or adaptive step generation, consider modifying the function
///   or implementing a similar utility.
///
/// # Panics
/// This function will panic if the calculated `step_size` results in a division by zero,
/// which could occur if `end` is equal to `start`. The caller should ensure that `end`
/// is greater than `start` to avoid this scenario.
///
/// # See Also
/// - [`Point2D::new`]: Utility used to construct individual points for the curve.
/// - [`Curve::from_vector`]: Used to generate the resulting curve from the constructed points.
///
/// # Example (High-Level Usage Concept)
/// While examples are omitted as requested, the general idea is to pass desired
/// values for `start`, `end`, and `slope` into this function in a practical implementation
/// scenario.
///
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::create_linear_curve;
/// let curve = create_linear_curve(
///     Decimal::new(0, 1),   // start = 0.0
///     Decimal::new(100, 1), // end = 10.0
///     Decimal::new(1, 0)    // slope = 1.0
/// );
/// ```
///
/// would result in a curve defined by the points:
/// `(0.0, 0.0)`, `(1.0, 1.0)`, ..., `(10.0, 10.0)`.
///
/// From the above, it demonstrates how linearly spaced and
pub fn create_linear_curve(start: Decimal, end: Decimal, slope: Decimal) -> Curve {
    let steps = 10;
    let step_size = (end - start) / Decimal::from(steps);

    let points: Vec<Point2D> = (0..=steps)
        .map(|i| {
            let x = start + step_size * Decimal::from(i);
            let y = slope * x;
            Point2D::new(x, y)
        })
        .collect();

    Curve::from_vector(points.iter().collect())
}

/// Creates a constant curve with equidistant points along the x-axis and the same constant value for the y-axis.
///
/// This function generates a simple mathematical curve defined over a fixed range of x-values with an equal spacing
/// between points, where each y-coordinate is set to a constant value specified by the `value` parameter. The curve
/// is represented as a collection of `Point2D` points, which are then used to create a `Curve` object.
///
/// # Parameters
/// - `start`: The starting x-coordinate for the curve, represented as a `Decimal`.
/// - `end`: The ending x-coordinate for the curve, represented as a `Decimal`.
/// - `value`: The constant y-coordinate value applied to all points in the curve, represented as a `Decimal`.
///
/// # Returns
/// A `Curve` instance that represents the constant curve. The returned curve consists of equidistant `Point2D`
/// points between the `start` and `end` x-coordinates, all having the same y-coordinate defined by `value`.
///
/// # Behavior
/// - The function divides the range `[start, end]` into a fixed number of equally spaced steps.
/// - The x-coordinate of each point is calculated based on this step size.
/// - The `value` is used as the y-coordinate for all points.
/// - A `Curve` is created using the generated `Point2D` points via the `Curve::from_vector` method.
///
/// # Details
/// - Internally, this function assumes 10 steps (`steps = 10`) for dividing the x-range. This creates 11 points
///   including both the `start` and `end` x-coordinates.
/// - The calculation of intermediate x-coordinates uses a constant `step_size`, computed as `(end - start) / steps`.
/// - The function ensures that both the `start` and `end` values are included in the resulting curve.
///
/// # Example
/// While this is designed to remain usage-agnostic, in practice, it results in a horizontal line in Cartesian
/// space that is constant in the y-dimension and spans the x-range.
///
/// # Panics
/// - The function will panic if `steps` is set to zero or if the provided `start` and `end` values result in
///   invalid arithmetic operations, such as division by zero or overflow of Decimal values.
///
/// # See Also
/// - [`Point2D::new`]: Used to create individual points in the resulting curve.
/// - [`Curve::from_vector`]: Used internally to convert the set of constant points into a `Curve` object.
pub fn create_constant_curve(start: Decimal, end: Decimal, value: Decimal) -> Curve {
    let steps = 10;
    let step_size = (end - start) / Decimal::from(steps);

    let point_values: Vec<Point2D> = (0..=steps)
        .map(|i| {
            let x = start + step_size * Decimal::from(i);
            Point2D::new(x, value)
        })
        .collect();

    let points: Vec<&Point2D> = point_values.iter().collect();

    Curve::from_vector(points)
}

/// Detects peaks and valleys in a set of points
///
/// # Arguments
///
/// * `points` - A reference to a BTreeSet of Point2D
///
/// # Returns
///
/// A tuple containing two vectors:
/// - The first vector contains the peaks (local maxima)
/// - The second vector contains the valleys (local minima)
pub fn detect_peaks_and_valleys(points: &BTreeSet<Point2D>) -> (Vec<Point2D>, Vec<Point2D>) {
    let points_vec: Vec<Point2D> = points.iter().cloned().collect();

    let mut peaks = Vec::new();
    let mut valleys = Vec::new();

    // Need at least 3 points to detect peaks and valleys
    if points_vec.len() < 3 {
        return (peaks, valleys);
    }

    for i in 1..points_vec.len()-1 {
        let prev = &points_vec[i-1];
        let current = &points_vec[i];
        let next = &points_vec[i+1];

        // Peak: y value is higher than its immediate neighbors
        if current.y > prev.y && current.y > next.y {
            peaks.push(*current);
        }

        // Valley: y value is lower than its immediate neighbors
        if current.y < prev.y && current.y < next.y {
            valleys.push(*current);
        }
    }

    (peaks, valleys)
}
