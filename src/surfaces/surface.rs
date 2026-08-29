//! This module provides functionality for visualizing and plotting 3D surfaces.
//! It leverages the `plotters` crate for rendering and offers a flexible API for
//! customizing plot appearance and saving outputs.  It supports plotting single
//! surfaces as well as collections of surfaces.
//!
//! # Key Features
//!
//! * **Surface Representation:** The `Surface` struct represents a 3D surface
//!   defined by a collection of 3D points.
//! * **Plotting:** The `Plottable` trait provides a common interface for generating plots.
//!   It's implemented for both `Surface` and `Vec<Surface>`, allowing single or multiple
//!   surfaces to be plotted easily.
//! * **Customization:** The `PlotBuilder` struct allows extensive customization of the plot's
//!   appearance, including titles, labels, dimensions, colors, and more.  It provides
//!   a builder pattern for configuring the plot.
//! * **Platform Compatibility:** Handles platform-specific differences for saving plots.
//!   Provides a no-op implementation for WASM targets where direct file saving is not
//!   supported.
//! * **Shading:** Utility functions are included to apply shading to surface points,
//!   enhancing 3D visualization.
//! * **Error Handling:** Uses the `SurfaceError` type for robust error management.
//!

use crate::curves::{Curve, Point2D};
use crate::error::decimal::DecimalError;
use crate::error::{InterpolationError, MetricsError, SurfaceError};
use crate::geometrics::{
    Arithmetic, AxisOperations, BasicMetrics, BiLinearInterpolation, ConstructionMethod,
    ConstructionParams, CubicInterpolation, GeometricObject, GeometricTransformations, Interpolate,
    InterpolationType, LinearInterpolation, MergeAxisInterpolate, MergeOperation, MetricsExtractor,
    RangeMetrics, RiskMetrics, ShapeMetrics, SplineInterpolation, TrendMetrics, powu_checked,
};
use crate::model::decimal::{d_add, d_div, d_mul, d_sub, d_sum_iter};
use crate::surfaces::Point3D;
use crate::surfaces::types::Axis;
use crate::utils::Len;

use crate::visualization::{Graph, GraphData, Surface3D};
use num_traits::ToPrimitive;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ops::Index;
use std::sync::Arc;
use tracing::warn;
use utoipa::ToSchema;

/// Represents a mathematical surface in 3D space.
///
/// # Overview
/// The `Surface` struct defines a three-dimensional surface composed of a collection
/// of 3D points. It tracks the range of coordinates in the X and Y dimensions to
/// establish the boundaries of the surface.
///
/// # Fields
/// - **points**: A sorted collection of `Point3D` objects that define the surface
///   geometry. Using `BTreeSet` ensures points are uniquely stored and ordered.
/// - **x_range**: A tuple containing the minimum and maximum x-coordinates of the surface
///   as `Decimal` values, representing the surface's width boundaries.
/// - **y_range**: A tuple containing the minimum and maximum y-coordinates of the surface
///   as `Decimal` values, representing the surface's depth boundaries.
///
/// # Examples
/// ```rust
/// use rust_decimal_macros::dec;
/// use std::collections::BTreeSet;
/// use optionstratlib::surfaces::{Surface, Point3D};
///
/// // Create some 3D points
/// let mut points = BTreeSet::new();
/// points.insert(Point3D { x: dec!(0.0), y: dec!(0.0), z: dec!(1.0) });
/// points.insert(Point3D { x: dec!(1.0), y: dec!(0.0), z: dec!(2.0) });
/// points.insert(Point3D { x: dec!(0.0), y: dec!(1.0), z: dec!(1.5) });
/// points.insert(Point3D { x: dec!(1.0), y: dec!(1.0), z: dec!(2.5) });
///
/// // Create a surface with these points
/// let surface = Surface {
///     points,
///     x_range: (dec!(0.0), dec!(1.0)),
///     y_range: (dec!(0.0), dec!(1.0)),
/// };
/// ```
///
/// # Usage
/// `Surface` is primarily used for mathematical modeling, data visualization,
/// and numerical analysis. It can represent various 3D structures such as
/// option pricing surfaces, terrain models, or any other data that can be
/// plotted in three dimensions.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    /// Collection of 3D points defining the surface
    pub points: BTreeSet<Point3D>,
    /// The minimum and maximum x-coordinates of the surface (min_x, max_x)
    pub x_range: (Decimal, Decimal),
    /// The minimum and maximum y-coordinates of the surface (min_y, max_y)
    pub y_range: (Decimal, Decimal),
}

impl Surface {
    /// Creates a new instance from a set of 3D points.
    ///
    /// # Parameters
    /// - `points`: A sorted set of `Point3D` objects that will form this geometric object.
    ///
    /// # Returns
    /// A new instance of the implementing structure with computed x and y ranges.
    ///
    /// # Details
    /// This constructor initializes a geometric object by:
    /// 1. Computing the minimum and maximum x-coordinate values
    /// 2. Computing the minimum and maximum y-coordinate values
    /// 3. Storing the provided points and calculated ranges
    ///
    /// The ranges are calculated using the `calculate_range` utility method
    /// defined in the `GeometricObject` trait.
    ///
    /// # Examples
    /// ```rust
    /// use std::collections::BTreeSet;
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::surfaces::{Point3D, Surface};
    ///
    /// let mut points = BTreeSet::new();
    /// points.insert(Point3D { x: dec!(1.0), y: dec!(2.0), z: dec!(3.0) });
    /// points.insert(Point3D { x: dec!(4.0), y: dec!(5.0), z: dec!(6.0) });
    ///
    /// let object = Surface::new(points);
    /// ```
    ///
    /// # Duplicate coordinates
    ///
    /// [`Point3D`] orders on the full `(x, y, z)` triple, so `points` may
    /// hold several heights above one xy-coordinate and `new` stores them
    /// all, even though [`Surface::get_point`] and
    /// [`Surface::contains_point`] return only the first match. Whether such
    /// a surface should be rejected or normalized is unresolved; see the
    /// issue #466.
    #[must_use]
    pub fn new(points: BTreeSet<Point3D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        Self {
            points,
            x_range,
            y_range,
        }
    }

    /// Projects a 3D surface onto a 2D plane based on the specified axis.
    ///
    /// This method creates a 2D curve by projecting the points of the surface onto a plane
    /// perpendicular to the specified axis. The projection is achieved by omitting the coordinate
    /// that corresponds to the specified axis.
    ///
    /// # Parameters
    /// - `&self`: Reference to the Surface instance
    /// - `axis` (`Axis`): The axis perpendicular to the projection plane:
    ///   - `Axis::X`: Projects onto the YZ plane (x-coordinate is omitted)
    ///   - `Axis::Y`: Projects onto the XZ plane (y-coordinate is omitted)
    ///   - `Axis::Z`: Projects onto the XY plane (z-coordinate is omitted)
    ///
    /// # Returns
    /// - `Curve`: A new 2D curve containing the projected points
    ///
    /// # Behavior
    /// - For `Axis::X`, the returned curve contains points with (y, z) coordinates
    /// - For `Axis::Y`, the returned curve contains points with (x, z) coordinates
    /// - For `Axis::Z`, the returned curve contains points with (x, y) coordinates
    ///
    /// # Multi-valued projections
    ///
    /// Dropping a coordinate from a grid is multi-valued: every row of the
    /// grid contributes a different height above the same projected abscissa.
    /// A projection of an `n`-by-`m` grid therefore yields up to `n * m`
    /// points, several of which share an abscissa.
    ///
    /// Every point is kept. The returned [`Curve`] is consequently **not**
    /// single-valued in `x`, unlike a curve built from a series, so
    /// interpolating it returns
    /// [`InterpolationError::DegenerateInterval`](crate::error::InterpolationError::DegenerateInterval)
    /// wherever two projected points share an abscissa. Callers that need a
    /// function of the projected abscissa must aggregate the points
    /// themselves — this method does not choose an aggregation for them.
    #[must_use]
    pub fn get_curve(&self, axis: Axis) -> Curve {
        let points = self
            .points
            .iter()
            .map(|p| match axis {
                Axis::X => Point2D::new(p.y, p.z),
                Axis::Y => Point2D::new(p.x, p.z),
                Axis::Z => Point2D::new(p.x, p.y),
            })
            .collect();
        Curve::new(points)
    }

    /// Performs one-dimensional spline interpolation on a collection of points.
    ///
    /// This function interpolates a value along a one-dimensional curve defined by a collection
    /// of points. It uses linear interpolation between adjacent points to estimate the value
    /// at the target position.
    ///
    /// # Parameters
    /// * `points` - A slice of points of type T
    /// * `target` - The x-coordinate at which to interpolate
    /// * `x_selector` - A function that extracts the x-coordinate from a point
    /// * `z_selector` - A function that extracts the z-coordinate (value) from a point
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The interpolated value at the target position
    /// * `Err(InterpolationError)` - If interpolation fails (e.g., insufficient points)
    ///
    /// # Type Parameters
    /// * `T` - The type of points, which must implement Clone
    ///
    /// # Behavior
    /// - Points are sorted by their x-coordinate
    /// - If fewer than 2 points are provided, returns an error
    /// - If the target is outside the range of x-coordinates, returns the value at the nearest endpoint
    /// - Otherwise performs linear interpolation between the two points that bracket the target
    fn one_dimensional_spline_interpolation<T>(
        &self,
        points: &[T],
        target: Decimal,
        x_selector: fn(&T) -> Decimal,
        z_selector: fn(&T) -> Decimal,
    ) -> Result<Decimal, InterpolationError>
    where
        T: Clone,
    {
        // Sort points by x coordinate
        let mut sorted_points = points.to_vec();
        sorted_points.sort_by(|a, b| {
            x_selector(a)
                .partial_cmp(&x_selector(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Ensure we have at least 2 points
        if sorted_points.len() < 2 {
            return Err(InterpolationError::Spline(
                "Insufficient points for interpolation".to_string(),
            ));
        }

        let missing = |index: usize| {
            InterpolationError::Spline(format!(
                "spline knot {index} is out of bounds for {} samples",
                sorted_points.len()
            ))
        };

        // Handle out-of-range cases
        let first = sorted_points.first().ok_or_else(|| missing(0))?;
        if target <= x_selector(first) {
            return Ok(z_selector(first));
        }

        let last_index = sorted_points.len() - 1;
        let last = sorted_points.last().ok_or_else(|| missing(last_index))?;
        if target >= x_selector(last) {
            return Ok(z_selector(last));
        }

        // Find the segment where the target falls. The `target <= first`
        // branch above already returned, so the first match is never at index
        // 0 and `index - 1` cannot underflow; the `checked_sub` states it.
        let (left_index, right_index) = match sorted_points
            .iter()
            .enumerate()
            .find(|(_, p)| x_selector(p) > target)
        {
            Some((index, _)) => (
                index.checked_sub(1).ok_or_else(|| {
                    InterpolationError::Spline(
                        "spline bracket starts before the first knot".to_string(),
                    )
                })?,
                index,
            ),
            None => (
                last_index.checked_sub(1).ok_or_else(|| {
                    InterpolationError::Spline(
                        "spline bracket needs at least two knots".to_string(),
                    )
                })?,
                last_index,
            ),
        };

        // Get the points for interpolation
        let left = sorted_points
            .get(left_index)
            .ok_or_else(|| missing(left_index))?;
        let right = sorted_points
            .get(right_index)
            .ok_or_else(|| missing(right_index))?;
        let x0 = x_selector(left);
        let x1 = x_selector(right);
        let z0 = z_selector(left);
        let z1 = z_selector(right);

        // Linear interpolation
        let op = "Surface::one_dimensional_spline_interpolation";
        let run = d_sub(x1, x0, op).map_err(interp_err(InterpolationError::Spline))?;
        if run.is_zero() {
            return Err(InterpolationError::DegenerateInterval);
        }
        let rise = d_sub(z1, z0, op).map_err(interp_err(InterpolationError::Spline))?;
        let offset = d_sub(target, x0, op).map_err(interp_err(InterpolationError::Spline))?;
        let ratio = d_div(offset, run, op).map_err(interp_err(InterpolationError::Spline))?;
        let step = d_mul(rise, ratio, op).map_err(interp_err(InterpolationError::Spline))?;
        let interpolated_z = d_add(z0, step, op).map_err(interp_err(InterpolationError::Spline))?;

        Ok(interpolated_z)
    }

    /// Fetches the point at `index` without going through the panicking
    /// [`Index`] contract.
    fn point_at(&self, index: usize) -> Result<&Point3D, SurfaceError> {
        self.points.iter().nth(index).ok_or_else(|| {
            SurfaceError::AnalysisError(format!(
                "point index {index} is out of bounds for a surface of {} points",
                self.points.len()
            ))
        })
    }

    /// Converts the surface points from Decimal to f64 format, with swapped y and z coordinates.
    ///
    /// # Returns
    /// A vector of tuples containing the coordinates of each point in the surface as `(x, z, y)`
    /// where each coordinate is converted to an `f64` value.
    ///
    /// # Details
    /// - This function is only available on non-WebAssembly targets.
    /// - The coordinates are returned as `(x, z, y)` tuples, with y and z swapped.
    /// - If the conversion from `Decimal` to `f64` fails for any coordinate, that value
    ///   will be replaced with 0.0.
    ///
    /// # Example
    /// ```rust,no_run
    /// use rust_decimal_macros::dec;
    /// use std::collections::BTreeSet;
    /// use optionstratlib::surfaces::{Point3D, Surface};
    ///
    /// let mut points = BTreeSet::new();
    /// points.insert(Point3D { x: dec!(1.5), y: dec!(3.0), z: dec!(2.0) });
    /// points.insert(Point3D { x: dec!(2.5), y: dec!(4.0), z: dec!(3.0) });
    ///
    /// let surface = Surface {
    ///     points,
    ///     x_range: (dec!(1.0), dec!(3.0)),
    ///     y_range: (dec!(3.0), dec!(4.0)),
    /// };
    ///
    /// // Will produce: [(1.5, 2.0, 3.0), (2.5, 3.0, 4.0)]
    /// let points = surface.get_f64_points();
    /// ```
    #[must_use]
    pub fn get_f64_points(&self) -> Vec<(f64, f64, f64)> {
        self.points
            .iter()
            .map(|p| {
                (
                    p.x.to_f64().unwrap_or(0.0),
                    p.z.to_f64().unwrap_or(0.0),
                    p.y.to_f64().unwrap_or(0.0),
                )
            })
            .collect()
    }
}

/// Wraps a checked-arithmetic failure in the interpolation variant of the
/// algorithm that raised it.
fn interp_err(
    kind: fn(String) -> InterpolationError,
) -> impl Fn(DecimalError) -> InterpolationError {
    move |err| kind(err.to_string())
}

/// Wraps a checked-arithmetic failure raised while building a surface.
fn construction_err(err: DecimalError) -> SurfaceError {
    SurfaceError::ConstructionError(err.to_string())
}

/// Wraps a checked-arithmetic failure raised while analysing a surface.
fn analysis_err(err: DecimalError) -> SurfaceError {
    SurfaceError::AnalysisError(err.to_string())
}

/// Wraps a checked-arithmetic failure raised while fitting a surface trend.
fn trend_err(err: DecimalError) -> MetricsError {
    MetricsError::TrendError(err.to_string())
}

/// Wraps a checked-arithmetic failure raised while computing surface risk.
fn risk_err(err: DecimalError) -> MetricsError {
    MetricsError::RiskError(err.to_string())
}

/// Squared euclidean distance from `point` to `(x, y)`, computed with checked
/// arithmetic so a coordinate at the edge of the `Decimal` range reports
/// instead of aborting.
fn squared_distance(
    point: &Point3D,
    x: Decimal,
    y: Decimal,
    op: &'static str,
) -> Result<Decimal, DecimalError> {
    let dx = d_sub(point.x, x, op)?;
    let dy = d_sub(point.y, y, op)?;
    d_add(powu_checked(dx, 2, op)?, powu_checked(dy, 2, op)?, op)
}

/// Orders `points` by their squared distance to `(x, y)`, keeping the stable
/// ordering `sort_by` produced while moving the arithmetic out of the
/// comparator, which has no channel for an overflow.
fn sort_by_distance<'a>(
    points: &mut [&'a Point3D],
    x: Decimal,
    y: Decimal,
    op: &'static str,
) -> Result<(), DecimalError> {
    let mut keyed: Vec<(Decimal, &'a Point3D)> = Vec::with_capacity(points.len());
    for point in points.iter() {
        keyed.push((squared_distance(point, x, y, op)?, point));
    }
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (slot, (_, point)) in points.iter_mut().zip(keyed) {
        *slot = point;
    }
    Ok(())
}

/// Reads one entry of a nearest-neighbour window, naming the slot on the
/// out-of-bounds path instead of panicking.
fn nth<'a>(
    points: &[&'a Point3D],
    index: usize,
    kind: fn(String) -> InterpolationError,
) -> Result<&'a Point3D, InterpolationError> {
    points.get(index).copied().ok_or_else(|| {
        kind(format!(
            "neighbour {index} is out of bounds for a window of {} points",
            points.len()
        ))
    })
}

/// Reads one sample of a sorted metric series, naming the statistic on the
/// out-of-bounds path instead of panicking.
fn sample_at(
    values: &[Decimal],
    index: usize,
    what: &'static str,
) -> Result<Decimal, MetricsError> {
    values.get(index).copied().ok_or_else(|| {
        MetricsError::BasicError(format!(
            "{what}: sample {index} is out of bounds for {} values",
            values.len()
        ))
    })
}

/// Arithmetic mean of a sample. Errors on an empty sample, where the mean is
/// undefined, and on a sum that leaves the representable range.
fn mean_of(values: &[Decimal], op: &'static str) -> Result<Decimal, DecimalError> {
    let sum = d_sum_iter(values.iter().copied(), op)?;
    d_div(sum, Decimal::from(values.len()), op)
}

/// Sum of `(value - mean)^order` over a sample: the raw central moment behind
/// the variance (`order = 2`), skewness (`3`) and kurtosis (`4`).
fn central_moment(
    values: &[Decimal],
    mean: Decimal,
    order: u64,
    op: &'static str,
) -> Result<Decimal, DecimalError> {
    let mut acc = Decimal::ZERO;
    for &value in values {
        let centered = d_sub(value, mean, op)?;
        acc = d_add(acc, powu_checked(centered, order, op)?, op)?;
    }
    Ok(acc)
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            points: BTreeSet::new(),
            x_range: (Decimal::ZERO, Decimal::ZERO),
            y_range: (Decimal::ZERO, Decimal::ZERO),
        }
    }
}

impl Graph for Surface {
    fn graph_data(&self) -> GraphData {
        GraphData::GraphSurface(Surface3D {
            x: self.points.iter().map(|p| p.x).collect(),
            y: self.points.iter().map(|p| p.y).collect(),
            z: self.points.iter().map(|p| p.z).collect(),
            name: "Surface".to_string(),
        })
    }
}

/// Implementation of the `GeometricObject` trait for the `Surface` struct.
///
/// This implementation provides functionality to create and manipulate 3D surfaces using points
/// in three-dimensional space. It supports construction from explicit point collections or
/// through parametric functions.
///
/// # Type Parameters
/// - Uses `Point3D` as the points that form the surface
/// - Uses `Point2D` as the parametric input for surface generation
///
/// # Methods
/// - `get_points()`: Retrieves all points in the surface
/// - `from_vector()`: Constructs a surface from a vector of points
/// - `construct()`: Creates a surface using different construction methods
///
/// # Error Handling
/// Uses `SurfaceError` for various error conditions, including:
/// - Empty point collections
/// - Invalid construction parameters
/// - Errors during parametric function evaluation
impl GeometricObject<Point3D, Point2D> for Surface {
    type Error = SurfaceError;

    /// Returns a borrowed reference to all points in the surface as an ordered set
    ///
    /// # Returns
    /// * `BTreeSet<&Point3D>` - A sorted set containing references to all points
    ///   that define the surface, maintaining the natural ordering of points
    ///
    /// # Example
    /// ```rust
    /// use optionstratlib::surfaces::{Surface, Point3D};
    /// use std::collections::BTreeSet;
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::geometrics::GeometricObject;
    ///
    /// // Create a surface with some points
    /// let mut surface = Surface {
    ///     points: BTreeSet::new(),
    ///     x_range: (dec!(0), dec!(10)),
    ///     y_range: (dec!(0), dec!(10)),
    /// };
    ///
    /// // Add points to the surface
    /// surface.points.insert(Point3D { x: dec!(1.0), y: dec!(2.0), z: dec!(3.0) });
    /// surface.points.insert(Point3D { x: dec!(4.0), y: dec!(5.0), z: dec!(6.0) });
    ///
    /// // Get references to all points in the surface
    /// let points = surface.get_points();
    /// assert_eq!(points.len(), 2);
    /// ```
    fn get_points(&self) -> BTreeSet<&Point3D> {
        self.points.iter().collect()
    }

    /// Creates a new Surface from a vector of points that can be converted into Point3D objects.
    ///
    /// This method constructs a Surface by converting each point in the input vector to a Point3D
    /// and collecting them into an ordered set. It also calculates the x and y coordinate ranges
    /// of the points to define the surface's boundaries.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type that can be converted into Point3D via the Into trait and can be cloned.
    ///
    /// # Parameters
    ///
    /// * `points`: A vector of objects that can be converted to Point3D.
    ///
    /// # Returns
    ///
    /// A new Surface instance containing the converted points and their coordinate ranges.
    ///
    /// # Example
    ///
    /// ```rust
    /// use optionstratlib::surfaces::{Surface, Point3D};
    /// use optionstratlib::geometrics::GeometricObject;
    /// use rust_decimal_macros::dec;
    ///
    /// // Create points data
    /// let points = vec![
    ///     Point3D { x: dec!(1.0), y: dec!(2.0), z: dec!(3.0) },
    ///     Point3D { x: dec!(4.0), y: dec!(5.0), z: dec!(6.0) }
    /// ];
    ///
    /// // Create a surface from the points
    /// let surface = Surface::from_vector(points);
    ///
    /// // The surface will contain both points and have x_range and y_range calculated automatically
    /// assert_eq!(surface.points.len(), 2);
    /// assert_eq!(surface.x_range, (dec!(1.0), dec!(4.0)));
    /// assert_eq!(surface.y_range, (dec!(2.0), dec!(5.0)));
    /// ```
    fn from_vector<T>(points: Vec<T>) -> Self
    where
        T: Into<Point3D> + Clone,
    {
        let points: BTreeSet<Point3D> = points.into_iter().map(|p| p.into()).collect();
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        Surface {
            points,
            x_range,
            y_range,
        }
    }

    /// Constructs a Surface from a given construction method.
    ///
    /// This function creates a Surface object from either a set of 3D points or a parametric function.
    ///
    /// # Parameters
    /// * `method` - A construction method that can be converted into a `ConstructionMethod<Point3D, Point2D>`
    ///
    /// # Type Parameters
    /// * `T` - Type that can be converted into a `ConstructionMethod<Point3D, Point2D>`
    ///
    /// # Returns
    /// * `Result<Self, Self::Error>` - Either a successfully constructed Surface or an error
    ///
    /// # Errors
    /// * `SurfaceError::Point3DError` - If an empty points array is provided
    /// * `SurfaceError::ConstructionError` - If invalid parameters are provided or the parametric function fails
    ///
    /// # Examples
    ///
    /// ## Creating from existing points
    /// ```rust
    /// # fn run() -> Result<(), optionstratlib::error::Error> {
    /// use std::collections::BTreeSet;
    /// use optionstratlib::geometrics::{ConstructionMethod, GeometricObject};
    /// use optionstratlib::surfaces::{Point3D, Surface};
    /// let points = BTreeSet::from_iter(vec![
    ///     Point3D::new(0, 0, 0),
    ///     Point3D::new(1, 0, 1),
    ///     Point3D::new(0, 1, 1),
    /// ]);
    /// let surface = Surface::construct(ConstructionMethod::FromData { points })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Creating from a parametric function
    /// ```rust,no_run
    /// # fn run() -> Result<(), optionstratlib::error::Error> {
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::curves::Point2D;
    /// use optionstratlib::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject, ResultPoint};
    /// use optionstratlib::surfaces::{Point3D, Surface};
    /// let params = ConstructionParams::D3 {
    ///     x_start: dec!(-1.0),
    ///     x_end: dec!(1.0),
    ///     y_start: dec!(-1.0),
    ///     y_end: dec!(1.0),
    ///     x_steps: 20,
    ///     y_steps: 20,
    /// };
    ///
    /// // Parametric function defining a paraboloid: z = x² + y²
    /// let f = Box::new(|p: Point2D| -> ResultPoint<Point3D> {
    ///     Ok(Point3D {
    ///         x: p.x,
    ///         y: p.y,
    ///         z: p.x * p.x + p.y * p.y,
    ///     })
    /// });
    ///
    /// let surface = Surface::construct(ConstructionMethod::Parametric { f, params })?;
    /// # Ok(())
    /// # }
    /// ```
    fn construct<T>(method: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
        T: Into<ConstructionMethod<Point3D, Point2D>>,
    {
        let method = method.into();
        match method {
            ConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(SurfaceError::Point3DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Surface::new(points))
            }
            ConstructionMethod::Parametric { f, params } => {
                let (x_start, x_end, y_start, y_end, x_steps, y_steps) = match params {
                    ConstructionParams::D3 {
                        x_start,
                        x_end,
                        y_start,
                        y_end,
                        x_steps,
                        y_steps,
                    } => (x_start, x_end, y_start, y_end, x_steps, y_steps),
                    _ => {
                        return Err(SurfaceError::ConstructionError(
                            "Invalid parameters".to_string(),
                        ));
                    }
                };
                if x_steps == 0 || y_steps == 0 {
                    return Err(SurfaceError::ConstructionError(
                        "Parametric construction needs at least one step on each axis".to_string(),
                    ));
                }
                let op = "Surface::construct::step";
                let x_span = d_sub(x_end, x_start, op).map_err(construction_err)?;
                let x_step = d_div(x_span, Decimal::from(x_steps), op).map_err(construction_err)?;
                let y_span = d_sub(y_end, y_start, op).map_err(construction_err)?;
                let y_step = d_div(y_span, Decimal::from(y_steps), op).map_err(construction_err)?;

                // Wrap f in an Arc so it can be shared across threads
                let f = Arc::new(f);

                let points: Result<BTreeSet<Point3D>, SurfaceError> = (0..=x_steps)
                    .into_par_iter()
                    .flat_map(|i| {
                        let f = Arc::clone(&f);
                        (0..=y_steps).into_par_iter().map(move |j| {
                            let x_offset =
                                d_mul(x_step, Decimal::from(i), op).map_err(construction_err)?;
                            let x = d_add(x_start, x_offset, op).map_err(construction_err)?;
                            let y_offset =
                                d_mul(y_step, Decimal::from(j), op).map_err(construction_err)?;
                            let y = d_add(y_start, y_offset, op).map_err(construction_err)?;
                            let t = Point2D::new(x, y);
                            f(t).map_err(|e| SurfaceError::ConstructionError(e.to_string()))
                        })
                    })
                    .collect();

                points.map(Surface::new)
            }
        }
    }
}

/// Implementation of the `Index` trait for `Surface`, allowing direct indexing access to surface points.
///
/// # Overview
/// This implementation allows you to access individual points in a `Surface` using array-like
/// indexing notation (e.g., `surface[0]`, `surface[1]`). Points are retrieved in the order they
/// appear in the underlying `BTreeSet`.
///
/// # Panics
///
/// Panics if `index >= self.points.len()`. This matches the documented
/// contract of [`std::ops::Index`].
///
/// # Performance
/// Note that this implementation uses `iter().nth(index)` which has O(n) time complexity
/// for `BTreeSet`. For frequent access to points by index, consider using a data structure
/// with O(1) indexing performance.
impl Index<usize> for Surface {
    type Output = Point3D;

    /// Retrieves a reference to a point on the surface at the specified index.
    ///
    /// This implementation allows using indexing syntax (e.g., `surface[i]`) to access
    /// individual points that make up the surface.
    ///
    /// # Panics
    ///
    /// Panics if `index >= self.points.len()`.
    fn index(&self, index: usize) -> &Self::Output {
        // INVARIANT: same as `Curve::index` above — `std::ops::Index`
        // requires returning `&Self::Output` with no signalling channel,
        // so panic on out-of-bounds is the documented contract.
        match self.points.iter().nth(index) {
            Some(p) => p,
            // scan-banned: allow -- `std::ops::Index` returns `&Self::Output`
            // and has no fallible channel; the contract mirrors `Vec::index`.
            // No library code reaches this arm: every internal lookup goes
            // through `Surface::point_at`.
            None => panic!(
                "Surface::index: out of bounds (index = {index}, len = {})",
                self.points.len()
            ),
        }
    }
}

/// Implementation of the `Interpolate` trait for the `Surface` type, enabling
/// interpolation from 3D surface points to 2D points.
///
/// # Overview
/// This implementation allows a `Surface` object to perform various types of interpolation
/// (linear, bilinear, cubic, and spline) by projecting 3D points from the surface to 2D points.
///
/// # Functionality
/// By implementing the `Interpolate` trait, `Surface` gains the following capabilities:
/// - Interpolating between 3D surface points to produce 2D projections
/// - Finding bracket points for interpolation operations
/// - Supporting multiple interpolation algorithms through the trait's methods
///
/// # Usage Example
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::surfaces::{Surface, Point3D};
/// use optionstratlib::curves::Point2D;
/// use optionstratlib::geometrics::{Interpolate, InterpolationType};
///
/// let surface = Surface::new(Default::default());
///
/// // Interpolate a 2D point at a specific position using linear interpolation
/// let input_point = Point2D { x: dec!(1.5), y: dec!(2.0) };
/// let result = surface.interpolate(input_point, InterpolationType::Linear);
/// ```
///
/// # Related Traits
/// This implementation relies on the surface also implementing:
/// - `LinearInterpolation<Point3D, Point2D>`
/// - `BiLinearInterpolation<Point3D, Point2D>`
/// - `CubicInterpolation<Point3D, Point2D>`
/// - `SplineInterpolation<Point3D, Point2D>`
/// - `GeometricObject<Point3D, Point2D>`
impl Interpolate<Point3D, Point2D> for Surface {}

/// # Linear Interpolation for Surfaces
///
/// Implementation of the `LinearInterpolation` trait for `Surface` structures, enabling
/// interpolation from 2D points to 3D points using barycentric coordinates.
///
/// ## Overview
///
/// This implementation allows calculating the height (z-coordinate) of any point within
/// the surface's x-y range by using linear interpolation based on the three nearest points
/// in the surface. The method employs barycentric coordinate interpolation with triangulation
/// of the nearest points.
///
/// ## Algorithm
///
/// The interpolation process follows these steps:
/// 1. Validate that the input point is within the surface's range
/// 2. Check for degenerate cases (all points at same location)
/// 3. Check for exact matches with existing points
/// 4. Find the three nearest points to the query point
/// 5. Calculate barycentric coordinates for the triangle formed by these points
/// 6. Interpolate the z-value using the barycentric weights
impl LinearInterpolation<Point3D, Point2D> for Surface {
    /// ## Parameters
    ///
    /// * `xy` - A `Point2D` representing the x and y coordinates where interpolation is needed
    ///
    /// ## Returns
    ///
    /// * `Result<Point3D, InterpolationError>` - The interpolated 3D point if successful, or an
    ///   appropriate error if interpolation cannot be performed
    ///
    /// ## Errors
    ///
    /// Returns `InterpolationError::Linear` in the following cases:
    /// * When the surface contains only coincident points forming a degenerate triangle
    /// * When the query point is outside the surface's x-y range
    fn linear_interpolate(&self, xy: Point2D) -> Result<Point3D, InterpolationError> {
        let first = match self.points.iter().next() {
            Some(p) => p,
            None => {
                return Err(InterpolationError::Linear(
                    "No points in the surface".to_string(),
                ));
            }
        };
        let all_same_xy = self.points.iter().all(|p| p.x == first.x && p.y == first.y);

        if all_same_xy && (first.x == xy.x && first.y == xy.y) {
            return Err(InterpolationError::Linear(
                "Degenerate triangle detected".to_string(),
            ));
        }

        if xy.x < self.x_range.0
            || xy.x > self.x_range.1
            || xy.y < self.y_range.0
            || xy.y > self.y_range.1
        {
            return Err(InterpolationError::Linear(
                "Point is outside the surface's range".to_string(),
            ));
        }

        // Check for degenerate triangle before exact match
        let unique_coords = self
            .points
            .iter()
            .map(|p| (p.x, p.y))
            .collect::<BTreeSet<_>>();

        if unique_coords.len() == 1 {
            return Err(InterpolationError::Linear(
                "Degenerate triangle detected".to_string(),
            ));
        }

        // Check for exact match
        if let Some(point) = self.points.iter().find(|p| p.x == xy.x && p.y == xy.y) {
            return Ok(*point);
        }

        // Barycentric interpolation needs a triangle, so a surface with fewer
        // than three points has no answer to give.
        if self.points.len() < 3 {
            return Err(InterpolationError::Linear(
                "Need at least three points for linear interpolation".to_string(),
            ));
        }

        let op = "Surface::linear_interpolate";
        let mut nearest_points: Vec<&Point3D> = self.points.iter().collect();
        sort_by_distance(&mut nearest_points, xy.x, xy.y, op)
            .map_err(interp_err(InterpolationError::Linear))?;

        let p1 = nth(&nearest_points, 0, InterpolationError::Linear)?;
        let p2 = nth(&nearest_points, 1, InterpolationError::Linear)?;
        let p3 = nth(&nearest_points, 2, InterpolationError::Linear)?;

        let y23 = d_sub(p2.y, p3.y, op).map_err(interp_err(InterpolationError::Linear))?;
        let x13 = d_sub(p1.x, p3.x, op).map_err(interp_err(InterpolationError::Linear))?;
        let x32 = d_sub(p3.x, p2.x, op).map_err(interp_err(InterpolationError::Linear))?;
        let y13 = d_sub(p1.y, p3.y, op).map_err(interp_err(InterpolationError::Linear))?;
        let denominator = d_add(
            d_mul(y23, x13, op).map_err(interp_err(InterpolationError::Linear))?,
            d_mul(x32, y13, op).map_err(interp_err(InterpolationError::Linear))?,
            op,
        )
        .map_err(interp_err(InterpolationError::Linear))?;
        if denominator.is_zero() {
            // Three collinear (or coincident) neighbours span no area, so the
            // barycentric weights are undefined rather than infinite.
            return Err(InterpolationError::Linear(
                "Degenerate triangle detected: the three nearest points are collinear".to_string(),
            ));
        }

        let qx3 = d_sub(xy.x, p3.x, op).map_err(interp_err(InterpolationError::Linear))?;
        let qy3 = d_sub(xy.y, p3.y, op).map_err(interp_err(InterpolationError::Linear))?;
        let y31 = d_sub(p3.y, p1.y, op).map_err(interp_err(InterpolationError::Linear))?;

        let w1_num = d_add(
            d_mul(y23, qx3, op).map_err(interp_err(InterpolationError::Linear))?,
            d_mul(x32, qy3, op).map_err(interp_err(InterpolationError::Linear))?,
            op,
        )
        .map_err(interp_err(InterpolationError::Linear))?;
        let w1 = d_div(w1_num, denominator, op).map_err(interp_err(InterpolationError::Linear))?;

        let w2_num = d_add(
            d_mul(y31, qx3, op).map_err(interp_err(InterpolationError::Linear))?,
            d_mul(x13, qy3, op).map_err(interp_err(InterpolationError::Linear))?,
            op,
        )
        .map_err(interp_err(InterpolationError::Linear))?;
        let w2 = d_div(w2_num, denominator, op).map_err(interp_err(InterpolationError::Linear))?;

        let w3 = d_sub(
            d_sub(Decimal::ONE, w1, op).map_err(interp_err(InterpolationError::Linear))?,
            w2,
            op,
        )
        .map_err(interp_err(InterpolationError::Linear))?;

        let z = d_sum_iter(
            [
                d_mul(w1, p1.z, op).map_err(interp_err(InterpolationError::Linear))?,
                d_mul(w2, p2.z, op).map_err(interp_err(InterpolationError::Linear))?,
                d_mul(w3, p3.z, op).map_err(interp_err(InterpolationError::Linear))?,
            ],
            op,
        )
        .map_err(interp_err(InterpolationError::Linear))?;

        Ok(Point3D::new(xy.x, xy.y, z))
    }
}

impl BiLinearInterpolation<Point3D, Point2D> for Surface {
    fn bilinear_interpolate(&self, xy: Point2D) -> Result<Point3D, InterpolationError> {
        // Check if we have enough points
        if self.points.len() < 4 {
            return Err(InterpolationError::Bilinear(
                "Need at least four points for bilinear interpolation".to_string(),
            ));
        }

        // Check if the point is within range
        if xy.x < self.x_range.0
            || xy.x > self.x_range.1
            || xy.y < self.y_range.0
            || xy.y > self.y_range.1
        {
            return Err(InterpolationError::Bilinear(
                "Point is outside the surface's range".to_string(),
            ));
        }

        // Check for invalid quadrilateral: all points have the same x and y but different z
        let xy_points: Vec<&Point3D> = self
            .points
            .iter()
            .filter(|p| p.x == xy.x && p.y == xy.y)
            .collect();

        if xy_points.len() == 4 {
            let z_values: Vec<Decimal> = xy_points.iter().map(|p| p.z).collect();
            let unique_z_values: Vec<Decimal> = z_values.clone();

            if unique_z_values.len() > 1 {
                return Err(InterpolationError::Bilinear(
                    "Invalid quadrilateral".to_string(),
                ));
            }
        }

        // For exact matches, return the actual point
        if let Some(point) = self.points.iter().find(|p| p.x == xy.x && p.y == xy.y) {
            return Ok(*point);
        }

        // Find the four closest points
        let op = "Surface::bilinear_interpolate";
        let mut sorted_points: Vec<&Point3D> = self.points.iter().collect();
        sort_by_distance(&mut sorted_points, xy.x, xy.y, op)
            .map_err(interp_err(InterpolationError::Bilinear))?;

        let closest_points = sorted_points.get(0..4).ok_or_else(|| {
            InterpolationError::Bilinear(format!(
                "need four neighbours, found {}",
                sorted_points.len()
            ))
        })?;

        // Sort points to create a quadrilateral
        let mut quad_points: Vec<&Point3D> = closest_points.to_vec();
        quad_points.sort_by(|a, b| {
            let a_key = (a.y, a.x);
            let b_key = (b.y, b.x);
            a_key
                .partial_cmp(&b_key)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Get the four points for interpolation
        let q11 = nth(&quad_points, 0, InterpolationError::Bilinear)?; // Bottom-left point
        let q12 = nth(&quad_points, 1, InterpolationError::Bilinear)?; // Bottom-right point
        let q21 = nth(&quad_points, 2, InterpolationError::Bilinear)?; // Top-left point
        let q22 = nth(&quad_points, 3, InterpolationError::Bilinear)?; // Top-right point

        // Calculate normalized coordinates
        let x_span = d_sub(q12.x, q11.x, op).map_err(interp_err(InterpolationError::Bilinear))?;
        let y_span = d_sub(q21.y, q11.y, op).map_err(interp_err(InterpolationError::Bilinear))?;
        if x_span.is_zero() || y_span.is_zero() {
            // The quadrilateral has collapsed onto a line, so the normalized
            // coordinates are undefined rather than infinite.
            return Err(InterpolationError::DegenerateInterval);
        }
        let x_offset = d_sub(xy.x, q11.x, op).map_err(interp_err(InterpolationError::Bilinear))?;
        let x_ratio =
            d_div(x_offset, x_span, op).map_err(interp_err(InterpolationError::Bilinear))?;
        let y_offset = d_sub(xy.y, q11.y, op).map_err(interp_err(InterpolationError::Bilinear))?;
        let y_ratio =
            d_div(y_offset, y_span, op).map_err(interp_err(InterpolationError::Bilinear))?;

        // Perform bilinear interpolation
        let inv_x =
            d_sub(Decimal::ONE, x_ratio, op).map_err(interp_err(InterpolationError::Bilinear))?;
        let inv_y =
            d_sub(Decimal::ONE, y_ratio, op).map_err(interp_err(InterpolationError::Bilinear))?;
        let corner = |a: Decimal, b: Decimal, z: Decimal| -> Result<Decimal, InterpolationError> {
            let weight = d_mul(a, b, op).map_err(interp_err(InterpolationError::Bilinear))?;
            d_mul(weight, z, op).map_err(interp_err(InterpolationError::Bilinear))
        };
        let z = d_sum_iter(
            [
                corner(inv_x, inv_y, q11.z)?,
                corner(x_ratio, inv_y, q12.z)?,
                corner(inv_x, y_ratio, q21.z)?,
                corner(x_ratio, y_ratio, q22.z)?,
            ],
            op,
        )
        .map_err(interp_err(InterpolationError::Bilinear))?;

        Ok(Point3D::new(xy.x, xy.y, z))
    }
}

impl CubicInterpolation<Point3D, Point2D> for Surface {
    fn cubic_interpolate(&self, xy: Point2D) -> Result<Point3D, InterpolationError> {
        // Check if we have enough points for cubic interpolation
        if self.points.len() < 9 {
            return Err(InterpolationError::Cubic(
                "Need at least nine points for cubic interpolation".to_string(),
            ));
        }

        // Check if the point is within range
        if xy.x < self.x_range.0
            || xy.x > self.x_range.1
            || xy.y < self.y_range.0
            || xy.y > self.y_range.1
        {
            return Err(InterpolationError::Cubic(
                "Point is outside the surface's range".to_string(),
            ));
        }

        // Check for exact point match
        if let Some(point) = self.points.iter().find(|p| p.x == xy.x && p.y == xy.y) {
            return Ok(*point);
        }

        // Find the 9 closest points for cubic interpolation
        let op = "Surface::cubic_interpolate";
        let mut sorted_points: Vec<&Point3D> = self.points.iter().collect();
        sort_by_distance(&mut sorted_points, xy.x, xy.y, op)
            .map_err(interp_err(InterpolationError::Cubic))?;

        let closest_points = sorted_points.get(0..9).ok_or_else(|| {
            InterpolationError::Cubic(format!(
                "need nine neighbours, found {}",
                sorted_points.len()
            ))
        })?;

        // Cubic interpolation requires solving a system of equations
        // We'll use a weighted cubic interpolation approach

        // Calculate weights based on distance
        let mut weights: Vec<Decimal> = Vec::with_capacity(closest_points.len());
        for &point in closest_points {
            let sq = squared_distance(point, xy.x, xy.y, op)
                .map_err(interp_err(InterpolationError::Cubic))?;
            let dist = match sq.sqrt() {
                Some(d) => d,
                None => {
                    // sqrt only fails for negative input or a result that
                    // cannot be represented as Decimal (overflow). Squared
                    // distance is always >= 0, so this is the overflow
                    // case — drop the point's contribution by giving it
                    // zero weight rather than a misleadingly small one.
                    warn!(
                        "cubic_interpolate: sqrt failed for operand ({sq}); dropping point from weighting"
                    );
                    weights.push(Decimal::ZERO);
                    continue;
                }
            };
            // The 1e-6 floor keeps a coincident neighbour out of a zero divisor.
            let shifted = d_add(dist, Decimal::new(1, 6), op)
                .map_err(interp_err(InterpolationError::Cubic))?;
            weights.push(
                d_div(Decimal::ONE, shifted, op).map_err(interp_err(InterpolationError::Cubic))?,
            );
        }

        // Weighted cubic interpolation
        let mut numerator_z = Decimal::ZERO;
        let mut denominator = Decimal::ZERO;

        for (&point, &weight) in closest_points.iter().zip(weights.iter()) {
            // Cubic weight function
            let cubic_weight =
                powu_checked(weight, 3, op).map_err(interp_err(InterpolationError::Cubic))?;
            let contribution =
                d_mul(point.z, cubic_weight, op).map_err(interp_err(InterpolationError::Cubic))?;
            numerator_z = d_add(numerator_z, contribution, op)
                .map_err(interp_err(InterpolationError::Cubic))?;
            denominator = d_add(denominator, cubic_weight, op)
                .map_err(interp_err(InterpolationError::Cubic))?;
        }

        // Prevent division by zero
        let interpolated_z = if denominator != Decimal::ZERO {
            d_div(numerator_z, denominator, op).map_err(interp_err(InterpolationError::Cubic))?
        } else {
            // Fallback to average if weights are problematic
            let zs: Vec<Decimal> = closest_points.iter().map(|p| p.z).collect();
            let sum = d_sum_iter(zs.iter().copied(), op)
                .map_err(interp_err(InterpolationError::Cubic))?;
            d_div(sum, Decimal::from(closest_points.len()), op)
                .map_err(interp_err(InterpolationError::Cubic))?
        };

        Ok(Point3D::new(xy.x, xy.y, interpolated_z))
    }
}

impl SplineInterpolation<Point3D, Point2D> for Surface {
    fn spline_interpolate(&self, xy: Point2D) -> Result<Point3D, InterpolationError> {
        // Check if we have enough points for spline interpolation
        if self.points.len() < 9 {
            return Err(InterpolationError::Spline(
                "Need at least nine points for spline interpolation".to_string(),
            ));
        }

        // Check if the point is within range
        if xy.x < self.x_range.0
            || xy.x > self.x_range.1
            || xy.y < self.y_range.0
            || xy.y > self.y_range.1
        {
            return Err(InterpolationError::Spline(
                "Point is outside the surface's range".to_string(),
            ));
        }

        // Check for exact point match
        if let Some(point) = self.points.iter().find(|p| p.x == xy.x && p.y == xy.y) {
            return Ok(*point);
        }

        // Sort points to create a grid-like structure
        let mut sorted_points: Vec<&Point3D> = self.points.iter().collect();
        sorted_points.sort_by(|a, b| {
            let a_key = (a.x, a.y);
            let b_key = (b.x, b.y);
            a_key
                .partial_cmp(&b_key)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Group points by x and y coordinates
        let mut x_groups: std::collections::HashMap<Decimal, Vec<&Point3D>> =
            std::collections::HashMap::new();
        let mut y_groups: std::collections::HashMap<Decimal, Vec<&Point3D>> =
            std::collections::HashMap::new();

        for &point in &sorted_points {
            x_groups.entry(point.x).or_default().push(point);
            y_groups.entry(point.y).or_default().push(point);
        }

        // Prepare data for interpolation
        let y_values: Vec<Decimal> = y_groups.keys().cloned().collect();

        // Natural cubic spline interpolation
        // We'll interpolate in two steps: first along x, then along y

        // Interpolate along x for each unique y value
        let mut interpolated_x_points: Vec<Point3D> = Vec::new();
        for &y in &y_values {
            let y_points: Vec<&Point3D> = sorted_points
                .iter()
                .filter(|&&p| p.y == y)
                .cloned()
                .collect();

            if y_points.len() < 2 {
                continue;
            }

            // Perform cubic spline interpolation along x for this y
            let x_interpolated =
                self.one_dimensional_spline_interpolation(&y_points, xy.x, |p| p.x, |p| p.z);

            if let Ok(z) = x_interpolated {
                interpolated_x_points.push(Point3D::new(xy.x, y, z));
            }
        }

        // If no x interpolation points, return error
        if interpolated_x_points.is_empty() {
            return Err(InterpolationError::Spline(
                "Could not interpolate along x-axis".to_string(),
            ));
        }

        // Now interpolate these points along y
        let y_interpolated = self.one_dimensional_spline_interpolation(
            &interpolated_x_points,
            xy.y,
            |p| p.y,
            |p| p.z,
        );

        // Return the final interpolated point
        y_interpolated.map(|z| Point3D::new(xy.x, xy.y, z))
    }
}

impl Len for Surface {
    fn len(&self) -> usize {
        self.points.len()
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl MetricsExtractor for Surface {
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, MetricsError> {
        let z_values: Vec<Decimal> = self.points.iter().map(|p| p.z).collect();

        // An empty surface has no statistics; report the same zeroed set
        // `Curve::compute_basic_metrics` already returns for an empty curve.
        if z_values.is_empty() {
            return Ok(BasicMetrics {
                mean: Decimal::ZERO,
                median: Decimal::ZERO,
                mode: Decimal::ZERO,
                std_dev: Decimal::ZERO,
            });
        }

        let op = "Surface::compute_basic_metrics";
        let mean = mean_of(&z_values, op).map_err(|e| MetricsError::BasicError(e.to_string()))?;

        let mut sorted = z_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sample_at(&sorted, sorted.len() / 2, "median")?;

        // Mode calculation using HashMap to count occurrences
        let mode = {
            let mut freq_map = std::collections::HashMap::new();
            for &val in &z_values {
                *freq_map.entry(val).or_insert(0) += 1;
            }
            freq_map
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(val, _)| val)
                .unwrap_or(Decimal::ZERO)
        };

        let sum_sq = central_moment(&z_values, mean, 2, op)
            .map_err(|e| MetricsError::BasicError(e.to_string()))?;
        let variance = d_div(sum_sq, Decimal::from(z_values.len()), op)
            .map_err(|e| MetricsError::BasicError(e.to_string()))?;
        let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

        Ok(BasicMetrics {
            mean,
            median,
            mode,
            std_dev,
        })
    }

    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError> {
        let z_values: Vec<Decimal> = self.points.iter().map(|p| p.z).collect();

        // Skewness and kurtosis need a spread to standardise by; mirror the
        // zeroed answer `Curve::compute_shape_metrics` returns below two
        // samples.
        if z_values.len() < 2 {
            return Ok(ShapeMetrics {
                skewness: Decimal::ZERO,
                kurtosis: Decimal::ZERO,
                peaks: vec![],
                valleys: vec![],
                inflection_points: vec![],
            });
        }

        let op = "Surface::compute_shape_metrics";
        let mean = mean_of(&z_values, op).map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let sum_sq = central_moment(&z_values, mean, 2, op)
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let variance = d_div(sum_sq, Decimal::from(z_values.len()), op)
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let std_dev = variance.sqrt().unwrap_or(Decimal::ONE);
        if std_dev.is_zero() {
            return Err(MetricsError::ShapeError(format!(
                "standard deviation ({std_dev}) is too small to compute skewness/kurtosis; the surface is degenerate"
            )));
        }

        let n = Decimal::from(z_values.len());

        let skew_num = central_moment(&z_values, mean, 3, op)
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let skew_den = d_mul(
            n,
            powu_checked(std_dev, 3, op).map_err(|e| MetricsError::ShapeError(e.to_string()))?,
            op,
        )
        .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let skewness =
            d_div(skew_num, skew_den, op).map_err(|e| MetricsError::ShapeError(e.to_string()))?;

        let kurt_num = central_moment(&z_values, mean, 4, op)
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let kurt_den = d_mul(
            n,
            powu_checked(std_dev, 4, op).map_err(|e| MetricsError::ShapeError(e.to_string()))?,
            op,
        )
        .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let kurtosis =
            d_div(kurt_num, kurt_den, op).map_err(|e| MetricsError::ShapeError(e.to_string()))?;

        Ok(ShapeMetrics {
            skewness,
            kurtosis,
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        })
    }

    fn compute_range_metrics(&self) -> Result<RangeMetrics, MetricsError> {
        let z_values: Vec<Decimal> = self.points.iter().map(|p| p.z).collect();

        // An empty surface has no quantiles; mirror the zeroed answer
        // `Curve::compute_range_metrics` returns for an empty curve.
        if z_values.is_empty() {
            return Ok(RangeMetrics {
                min: Point2D::new(Decimal::ZERO, Decimal::ZERO),
                max: Point2D::new(Decimal::ZERO, Decimal::ZERO),
                range: Decimal::ZERO,
                quartiles: (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
                interquartile_range: Decimal::ZERO,
            });
        }

        let mut sorted = z_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted.first().copied().unwrap_or(Decimal::ZERO);
        let max = sorted.last().copied().unwrap_or(Decimal::ZERO);

        let op = "Surface::compute_range_metrics";
        let len = sorted.len();
        let q1 = sample_at(&sorted, len / 4, "first quartile")?;
        let q2 = sample_at(&sorted, len / 2, "median")?;
        let q3 = sample_at(&sorted, 3 * len / 4, "third quartile")?;

        let range = d_sub(max, min, op).map_err(|e| MetricsError::RangeError(e.to_string()))?;
        let iqr = d_sub(q3, q1, op).map_err(|e| MetricsError::RangeError(e.to_string()))?;

        Ok(RangeMetrics {
            min: Point2D::new(Decimal::ZERO, min),
            max: Point2D::new(Decimal::ZERO, max),
            range,
            quartiles: (q1, q2, q3),
            interquartile_range: iqr,
        })
    }

    fn compute_trend_metrics(&self) -> Result<TrendMetrics, MetricsError> {
        let points: Vec<Point2D> = self.points.iter().map(|p| Point2D::new(p.x, p.z)).collect();

        // Handle surfaces with insufficient points
        if points.len() < 2 {
            return Ok(TrendMetrics {
                slope: Decimal::ZERO,
                intercept: Decimal::ZERO,
                r_squared: Decimal::ONE,
                moving_average: vec![],
            });
        }

        // Linear Regression Calculation
        let n = Decimal::from(points.len());
        let x_vals: Vec<Decimal> = points.iter().map(|p| p.x).collect();
        let z_vals: Vec<Decimal> = points.iter().map(|p| p.y).collect();

        let op = "Surface::compute_trend_metrics";
        let sum_x = d_sum_iter(x_vals.iter().copied(), op).map_err(trend_err)?;
        let sum_z = d_sum_iter(z_vals.iter().copied(), op).map_err(trend_err)?;

        // Check for identical points to avoid division by zero
        let first_z = sample_at(&z_vals, 0, "trend baseline")?;
        let is_identical_points = z_vals.iter().all(|&z| z == first_z);

        let (slope, intercept, r_squared) = if is_identical_points {
            // All points are the same
            (Decimal::ZERO, first_z, Decimal::ONE)
        } else {
            let regression = || -> Result<(Decimal, Decimal, Decimal), DecimalError> {
                let mut sum_xz = Decimal::ZERO;
                let mut sum_xx = Decimal::ZERO;
                for (x, z) in x_vals.iter().zip(&z_vals) {
                    sum_xz = d_add(sum_xz, d_mul(*x, *z, op)?, op)?;
                    sum_xx = d_add(sum_xx, d_mul(*x, *x, op)?, op)?;
                }

                let numerator = d_sub(d_mul(n, sum_xz, op)?, d_mul(sum_x, sum_z, op)?, op)?;
                let denominator = d_sub(d_mul(n, sum_xx, op)?, d_mul(sum_x, sum_x, op)?, op)?;
                let slope = d_div(numerator, denominator, op)?;
                let intercept = d_div(d_sub(sum_z, d_mul(slope, sum_x, op)?, op)?, n, op)?;

                // R-squared Calculation
                let mean_z = d_div(sum_z, n, op)?;
                let sst = central_moment(&z_vals, mean_z, 2, op)?;

                let mut ssr = Decimal::ZERO;
                for (z, x) in z_vals.iter().zip(&x_vals) {
                    let z_predicted = d_add(d_mul(slope, *x, op)?, intercept, op)?;
                    let residual = d_sub(*z, z_predicted, op)?;
                    ssr = d_add(ssr, powu_checked(residual, 2, op)?, op)?;
                }

                let r_squared = if sst == Decimal::ZERO {
                    Decimal::ONE
                } else {
                    d_sub(Decimal::ONE, d_div(ssr, sst, op)?, op)?
                };

                Ok((slope, intercept, r_squared))
            };

            // A surface whose abscissas all collapse to one value (a single
            // column) leaves the ordinary-least-squares denominator at zero,
            // where the slope is undefined rather than infinite.
            regression().map_err(trend_err)?
        };

        // Moving Average Calculation
        let window_sizes = [3, 5, 7];
        let mut moving_average: Vec<Point2D> = Vec::new();
        for window in window_sizes {
            if window > points.len() {
                continue;
            }
            for window_points in points.windows(window) {
                let xs: Vec<Decimal> = window_points.iter().map(|p| p.x).collect();
                let ys: Vec<Decimal> = window_points.iter().map(|p| p.y).collect();
                let avg_x = mean_of(&xs, op).map_err(trend_err)?;
                let avg_y = mean_of(&ys, op).map_err(trend_err)?;
                moving_average.push(Point2D::new(avg_x, avg_y));
            }
        }

        Ok(TrendMetrics {
            slope,
            intercept,
            r_squared,
            moving_average,
        })
    }

    fn compute_risk_metrics(&self) -> Result<RiskMetrics, MetricsError> {
        let z_values: Vec<Decimal> = self.points.iter().map(|p| p.z).collect();

        // An empty surface carries no risk; mirror the zeroed answer
        // `Curve::compute_risk_metrics` returns for an empty curve.
        if z_values.is_empty() {
            return Ok(RiskMetrics {
                volatility: Decimal::ZERO,
                value_at_risk: Decimal::ZERO,
                expected_shortfall: Decimal::ZERO,
                beta: Decimal::ZERO,
                sharpe_ratio: Decimal::ZERO,
            });
        }

        let op = "Surface::compute_risk_metrics";
        let mean = mean_of(&z_values, op).map_err(risk_err)?;
        let sum_sq = central_moment(&z_values, mean, 2, op).map_err(risk_err)?;
        let variance = d_div(sum_sq, Decimal::from(z_values.len()), op).map_err(risk_err)?;
        let volatility = variance.sqrt().unwrap_or(Decimal::ZERO);

        // Value at Risk (95% confidence) using parametric method. At zero
        // dispersion this is `mean - 1.645 * 0 = mean`, a deterministic level
        // rather than an absence of value, so it is computed from the formula
        // on every path instead of being short-circuited to zero.
        let z_score = dec!(1.645); // 95% confidence interval
        let scaled_vol = d_mul(z_score, volatility, op).map_err(risk_err)?;
        let var = d_sub(mean, scaled_vol, op).map_err(risk_err)?;

        // Expected Shortfall (Conditional VaR) calculation. An empty tail has
        // no conditional mean; report zero as `Curve` does.
        let tail: Vec<Decimal> = z_values.iter().copied().filter(|&x| x < var).collect();
        let expected_shortfall = if tail.is_empty() {
            Decimal::ZERO
        } else {
            mean_of(&tail, op).map_err(risk_err)?
        };

        // Beta calculation with optional market volatility
        let beta = Decimal::ZERO; // TODO: Implement beta calculation

        // Sharpe Ratio (assuming risk-free rate of 0). A flat surface has no
        // dispersion to divide by, which makes this the one field that is
        // genuinely undefined at `volatility == 0`; the others keep their
        // deterministic limits.
        let sharpe_ratio = if volatility.is_zero() {
            Decimal::ZERO
        } else {
            d_div(mean, volatility, op).map_err(risk_err)?
        };

        Ok(RiskMetrics {
            volatility,
            value_at_risk: var,
            expected_shortfall,
            beta,
            sharpe_ratio,
        })
    }
}

impl Arithmetic<Surface> for Surface {
    type Error = SurfaceError;

    fn merge(surfaces: &[&Surface], operation: MergeOperation) -> Result<Surface, Self::Error> {
        if surfaces.is_empty() {
            return Err(SurfaceError::invalid_parameters(
                "merge_surfaces",
                "No surfaces provided for merging",
            ));
        }

        if let [only] = surfaces {
            return Ok((*only).clone());
        }

        // Find intersection of x,y ranges
        let min_x = surfaces
            .iter()
            .map(|s| s.x_range.0)
            .max()
            .unwrap_or(Decimal::ZERO);
        let max_x = surfaces
            .iter()
            .map(|s| s.x_range.1)
            .min()
            .unwrap_or(Decimal::ZERO);
        let min_y = surfaces
            .iter()
            .map(|s| s.y_range.0)
            .max()
            .unwrap_or(Decimal::ZERO);
        let max_y = surfaces
            .iter()
            .map(|s| s.y_range.1)
            .min()
            .unwrap_or(Decimal::ZERO);

        // Validate ranges
        if min_x >= max_x || min_y >= max_y {
            return Err(SurfaceError::invalid_parameters(
                "merge_surfaces",
                "Surfaces have incompatible ranges",
            ));
        }

        // Create interpolation grid
        let steps = 50;
        let op = "Surface::merge";
        let x_span = d_sub(max_x, min_x, op).map_err(construction_err)?;
        let x_step = d_div(x_span, Decimal::from(steps), op).map_err(construction_err)?;
        let y_span = d_sub(max_y, min_y, op).map_err(construction_err)?;
        let y_step = d_div(y_span, Decimal::from(steps), op).map_err(construction_err)?;

        let result_points: Result<Vec<Point3D>, SurfaceError> = (0..=steps)
            .into_par_iter()
            .flat_map(|i| {
                (0..=steps).into_par_iter().map(move |j| {
                    let x_offset = d_mul(x_step, Decimal::from(i), op).map_err(construction_err)?;
                    let x = d_add(min_x, x_offset, op).map_err(construction_err)?;
                    let y_offset = d_mul(y_step, Decimal::from(j), op).map_err(construction_err)?;
                    let y = d_add(min_y, y_offset, op).map_err(construction_err)?;
                    let point = Point2D::new(x, y);

                    // Interpolate z values
                    let z_values: Result<Vec<Decimal>, SurfaceError> = surfaces
                        .iter()
                        .map(|surface| {
                            surface
                                .interpolate(point, InterpolationType::Cubic)
                                .map(|point3d| point3d.z)
                                .map_err(SurfaceError::from)
                        })
                        .collect();

                    let z_values = z_values?;

                    // Apply operation
                    let result_z = match operation {
                        MergeOperation::Add => {
                            d_sum_iter(z_values.iter().copied(), op).map_err(construction_err)?
                        }
                        MergeOperation::Subtract => {
                            let first = z_values.first().cloned().unwrap_or(Decimal::ZERO);
                            let remaining_sum = d_sum_iter(z_values.iter().skip(1).copied(), op)
                                .map_err(construction_err)?;
                            d_sub(first, remaining_sum, op).map_err(construction_err)?
                        }
                        MergeOperation::Multiply => z_values.par_iter().copied().map(Ok).reduce(
                            || Ok(Decimal::ONE),
                            |a, b| d_mul(a?, b?, op).map_err(construction_err),
                        )?,
                        MergeOperation::Divide => {
                            // Division is neither associative nor commutative,
                            // so the divisors are folded in sequence from the
                            // first value. A parallel fold produces one partial
                            // per chunk and its reducer would have to discard
                            // all but one of them, dropping both the trailing
                            // divisors and any checked-arithmetic error raised
                            // inside a discarded chunk.
                            let mut divisors = z_values.iter().copied();
                            let first = divisors.next().unwrap_or(Decimal::ONE);
                            divisors.try_fold(first, |acc, val| {
                                if val == Decimal::ZERO {
                                    Ok(acc)
                                } else {
                                    d_div(acc, val, op).map_err(construction_err)
                                }
                            })?
                        }
                        MergeOperation::Max => z_values
                            .par_iter()
                            .cloned()
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                            .unwrap_or(Decimal::ZERO),
                        MergeOperation::Min => z_values
                            .par_iter()
                            .cloned()
                            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                            .unwrap_or(Decimal::ZERO),
                    };

                    Ok(Point3D::new(x, y, result_z))
                })
            })
            .collect();

        let result_points = result_points?;
        Ok(Surface::from_vector(result_points))
    }

    fn merge_with(
        &self,
        other: &Surface,
        operation: MergeOperation,
    ) -> Result<Surface, Self::Error> {
        Self::merge(&[self, other], operation)
    }
}

impl AxisOperations<Point3D, Point2D> for Surface {
    type Error = SurfaceError;

    fn contains_point(&self, x: &Point2D) -> bool {
        self.points.iter().any(|p| p.x == x.x && p.y == x.y)
    }

    fn get_index_values(&self) -> Vec<Point2D> {
        self.points.iter().map(|p| Point2D::new(p.x, p.y)).collect()
    }

    fn get_values(&self, x: Point2D) -> Vec<&Decimal> {
        self.points
            .iter()
            .filter(|p| p.x == x.x && p.y == x.y)
            .map(|p| &p.z)
            .collect()
    }

    fn get_closest_point(&self, x: &Point2D) -> Result<&Point3D, Self::Error> {
        // Compare squared distances directly: ordering is monotonic on
        // non-negative inputs, so the sqrt is unnecessary and would
        // introduce a fallback that could otherwise distort ordering. The
        // distance is folded explicitly because a coordinate at the edge of
        // the `Decimal` range overflows, and `min_by` has no channel for that.
        let mut closest: Option<(&Point3D, Decimal)> = None;
        for point in &self.points {
            let squared = squared_distance(point, x.x, x.y, "Surface::get_closest_point")
                .map_err(analysis_err)?;
            // `min_by` keeps the first of several equal minima; `<=` here
            // preserves that.
            match closest {
                Some((_, best)) if best <= squared => {}
                _ => closest = Some((point, squared)),
            }
        }

        closest
            .map(|(point, _)| point)
            .ok_or(SurfaceError::Point3DError {
                reason: "No points found",
            })
    }

    fn get_point(&self, x: &Point2D) -> Option<&Point3D> {
        self.points.iter().find(|p| p.x == x.x && p.y == x.y)
    }
}

impl MergeAxisInterpolate<Point3D, Point2D> for Surface
where
    Self: Sized,
{
    fn merge_axis_interpolate(
        &self,
        other: &Self,
        interpolation: InterpolationType,
    ) -> Result<(Self, Self), Self::Error> {
        // Get merged unique xy-coordinates
        let merged_xy_values = self.merge_axis_index(other);

        let mut interpolated_self_points = BTreeSet::new();
        let mut interpolated_other_points = BTreeSet::new();

        for xy in &merged_xy_values {
            if self.contains_point(xy) {
                let pt = self
                    .points
                    .iter()
                    .find(|p| p.x == xy.x && p.y == xy.y)
                    .ok_or_else(|| {
                        SurfaceError::AnalysisError(format!(
                            "merge_axis_interpolate: missing self point at ({},{}) despite contains_point()",
                            xy.x, xy.y
                        ))
                    })?;
                interpolated_self_points.insert(*pt);
            } else {
                let interpolated_point = self.interpolate(*xy, interpolation)?;
                interpolated_self_points.insert(interpolated_point);
            }

            if other.contains_point(xy) {
                let pt = other
                    .points
                    .iter()
                    .find(|p| p.x == xy.x && p.y == xy.y)
                    .ok_or_else(|| {
                        SurfaceError::AnalysisError(format!(
                            "merge_axis_interpolate: missing other point at ({},{}) despite contains_point()",
                            xy.x, xy.y
                        ))
                    })?;
                interpolated_other_points.insert(*pt);
            } else {
                let interpolated_point = other.interpolate(*xy, interpolation)?;
                interpolated_other_points.insert(interpolated_point);
            }
        }

        Ok((
            Surface::new(interpolated_self_points),
            Surface::new(interpolated_other_points),
        ))
    }
}

impl GeometricTransformations<Point3D> for Surface {
    type Error = SurfaceError;

    fn translate(&self, deltas: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if deltas.len() != 3 {
            return Err(SurfaceError::invalid_parameters(
                "translate",
                "Expected 3 deltas for 3D translation",
            ));
        }

        let (Some(dx), Some(dy), Some(dz)) = (deltas.first(), deltas.get(1), deltas.get(2)) else {
            return Err(SurfaceError::invalid_parameters(
                "translate",
                "Expected 3 deltas for 3D translation",
            ));
        };

        let translated_points = self
            .points
            .iter()
            .map(|point| {
                let x = d_add(point.x, **dx, "Surface::translate::x")?;
                let y = d_add(point.y, **dy, "Surface::translate::y")?;
                let z = d_add(point.z, **dz, "Surface::translate::z")?;
                Ok(Point3D::new(x, y, z))
            })
            .collect::<Result<BTreeSet<Point3D>, DecimalError>>()
            .map_err(construction_err)?;

        Ok(Surface::new(translated_points))
    }

    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if factors.len() != 3 {
            return Err(SurfaceError::invalid_parameters(
                "scale",
                "Expected 3 factors for 3D scaling",
            ));
        }

        let (Some(fx), Some(fy), Some(fz)) = (factors.first(), factors.get(1), factors.get(2))
        else {
            return Err(SurfaceError::invalid_parameters(
                "scale",
                "Expected 3 factors for 3D scaling",
            ));
        };

        let scaled_points = self
            .points
            .iter()
            .map(|point| {
                let x = d_mul(point.x, **fx, "Surface::scale::x")?;
                let y = d_mul(point.y, **fy, "Surface::scale::y")?;
                let z = d_mul(point.z, **fz, "Surface::scale::z")?;
                Ok(Point3D::new(x, y, z))
            })
            .collect::<Result<BTreeSet<Point3D>, DecimalError>>()
            .map_err(construction_err)?;

        Ok(Surface::new(scaled_points))
    }

    fn intersect_with(&self, other: &Self) -> Result<Vec<Point3D>, Self::Error> {
        let mut intersections = Vec::new();
        let epsilon = Decimal::new(1, 6); // 0.000001 tolerance
        let op = "Surface::intersect_with";

        for p1 in self.points.iter() {
            for p2 in other.points.iter() {
                let dx = d_sub(p1.x, p2.x, op).map_err(analysis_err)?.abs();
                if dx >= epsilon {
                    continue;
                }
                let dy = d_sub(p1.y, p2.y, op).map_err(analysis_err)?.abs();
                if dy >= epsilon {
                    continue;
                }
                let dz = d_sub(p1.z, p2.z, op).map_err(analysis_err)?.abs();
                if dz < epsilon {
                    intersections.push(*p1);
                }
            }
        }

        Ok(intersections)
    }

    fn derivative_at(&self, point: &Point3D) -> Result<Vec<Decimal>, Self::Error> {
        // Handle surfaces with insufficient points
        if self.points.len() < 2 {
            return Err(SurfaceError::invalid_parameters(
                "derivative_at",
                "Surface needs at least 2 points for derivative calculation",
            ));
        }

        let op = "Surface::derivative_at";

        // For surfaces with exactly 2 or 3 points, use a simple approach
        if self.points.len() <= 3 {
            let p0 = self.point_at(0)?;
            let p1 = self.point_at(1)?;

            // Ensure points are not identical
            if p0 == p1 {
                return Err(SurfaceError::invalid_parameters(
                    "derivative_at",
                    "Points are identical, cannot calculate derivatives",
                ));
            }

            // Calculate derivatives using the first two points. `Decimal::MAX`
            // is the pre-existing sentinel for a collapsed axis.
            let rise = d_sub(p1.z, p0.z, op).map_err(analysis_err)?;
            let run_x = d_sub(p1.x, p0.x, op).map_err(analysis_err)?;
            let dx = if run_x == Decimal::ZERO {
                Decimal::MAX
            } else {
                d_div(rise, run_x, op).map_err(analysis_err)?
            };

            let run_y = d_sub(p1.y, p0.y, op).map_err(analysis_err)?;
            let dy = if run_y == Decimal::ZERO {
                Decimal::MAX
            } else {
                d_div(rise, run_y, op).map_err(analysis_err)?
            };

            return Ok(vec![dx, dy]);
        }

        if !(self.x_range.0..=self.x_range.1).contains(&point.x)
            || !(self.y_range.0..=self.y_range.1).contains(&point.y)
        {
            return Err(SurfaceError::invalid_parameters(
                "derivative_at",
                "Point is outside the surface's range",
            ));
        }

        // For more complex surfaces, find nearby points
        let tolerance = dec!(0.5);

        let mut x_points: BTreeSet<Point3D> = BTreeSet::new();
        let mut y_points: BTreeSet<Point3D> = BTreeSet::new();
        for candidate in self.get_points() {
            if d_sub(candidate.x, point.x, op).map_err(analysis_err)?.abs() < tolerance {
                x_points.insert(*candidate);
            }
            if d_sub(candidate.y, point.y, op).map_err(analysis_err)?.abs() < tolerance {
                y_points.insert(*candidate);
            }
        }

        // If not enough nearby points, use the entire surface
        let x_candidates = if x_points.len() < 2 {
            &self.points
        } else {
            &x_points
        };
        let y_candidates = if y_points.len() < 2 {
            &self.points
        } else {
            &y_points
        };

        // Ensure we have at least 2 points
        if x_candidates.len() < 2 || y_candidates.len() < 2 {
            return Err(SurfaceError::invalid_parameters(
                "derivative_at",
                "Could not find suitable points for derivative calculation",
            ));
        }

        // Sort and find derivatives
        let mut x_sorted: Vec<_> = x_candidates.iter().collect();
        x_sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        let mut y_sorted: Vec<_> = y_candidates.iter().collect();
        y_sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));

        let missing = || {
            SurfaceError::AnalysisError(
                "derivative_at: fewer than two candidates after filtering".to_string(),
            )
        };
        let (Some(x0), Some(x1)) = (x_sorted.first(), x_sorted.get(1)) else {
            return Err(missing());
        };
        let (Some(y0), Some(y1)) = (y_sorted.first(), y_sorted.get(1)) else {
            return Err(missing());
        };

        // Prevent division by zero
        let dx = if x0.x == x1.x {
            Decimal::ZERO
        } else {
            let rise = d_sub(x1.z, x0.z, op).map_err(analysis_err)?;
            let run = d_sub(x1.x, x0.x, op).map_err(analysis_err)?;
            d_div(rise, run, op).map_err(analysis_err)?
        };

        let dy = if y0.y == y1.y {
            Decimal::ZERO
        } else {
            let rise = d_sub(y1.z, y0.z, op).map_err(analysis_err)?;
            let run = d_sub(y1.y, y0.y, op).map_err(analysis_err)?;
            d_div(rise, run, op).map_err(analysis_err)?
        };

        Ok(vec![dx, dy])
    }

    fn extrema(&self) -> Result<(Point3D, Point3D), Self::Error> {
        if self.points.is_empty() {
            return Err(SurfaceError::invalid_parameters(
                "extrema",
                "Surface has no points",
            ));
        }

        let min_point = self
            .points
            .iter()
            .min_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| {
                SurfaceError::AnalysisError("extrema: empty point set in min_by".to_string())
            })?;

        let max_point = self
            .points
            .iter()
            .max_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| {
                SurfaceError::AnalysisError("extrema: empty point set in max_by".to_string())
            })?;

        Ok((min_point, max_point))
    }

    fn measure_under(&self, base_value: &Decimal) -> Result<Decimal, Self::Error> {
        if self.points.len() < 3 {
            return Ok(Decimal::ZERO);
        }

        // Approximate volume using triangular prisms
        let mut volume = Decimal::ZERO;
        let points: Vec<_> = self.points.iter().collect();

        let op = "Surface::measure_under";

        // For each possible triangle in the surface
        for window in points.windows(3) {
            // Calculate area of triangle
            let (Some(p1), Some(p2), Some(p3)) = (window.first(), window.get(1), window.get(2))
            else {
                return Err(SurfaceError::AnalysisError(
                    "measure_under: triangle window is shorter than three points".to_string(),
                ));
            };

            let cross_a = d_mul(
                d_sub(p2.x, p1.x, op).map_err(analysis_err)?,
                d_sub(p3.y, p1.y, op).map_err(analysis_err)?,
                op,
            )
            .map_err(analysis_err)?;
            let cross_b = d_mul(
                d_sub(p3.x, p1.x, op).map_err(analysis_err)?,
                d_sub(p2.y, p1.y, op).map_err(analysis_err)?,
                op,
            )
            .map_err(analysis_err)?;
            let cross = d_sub(cross_a, cross_b, op).map_err(analysis_err)?;
            let area = d_div(cross.abs(), dec!(2), op).map_err(analysis_err)?;

            // Average height from base_value
            let heights = d_sum_iter(
                [
                    d_sub(p1.z, *base_value, op).map_err(analysis_err)?,
                    d_sub(p2.z, *base_value, op).map_err(analysis_err)?,
                    d_sub(p3.z, *base_value, op).map_err(analysis_err)?,
                ],
                op,
            )
            .map_err(analysis_err)?;
            let avg_height = d_div(heights, dec!(3), op).map_err(analysis_err)?;

            let prism = d_mul(area, avg_height, op).map_err(analysis_err)?;
            volume = d_add(volume, prism, op).map_err(analysis_err)?;
        }

        Ok(volume.abs())
    }
}

#[cfg(test)]
mod tests_surface_basic {
    use super::*;
    use rust_decimal_macros::dec;

    // Helper function to create test points
    fn create_test_points() -> BTreeSet<Point3D> {
        BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(1.5)),
        ])
    }

    #[test]
    fn test_surface_new() {
        let points = create_test_points();
        let surface = Surface::new(points.clone());

        // Check points are correctly stored
        assert_eq!(surface.points, points);

        // Check x range calculation
        assert_eq!(surface.x_range.0, dec!(0.0));
        assert_eq!(surface.x_range.1, dec!(1.0));

        // Check y range calculation
        assert_eq!(surface.y_range.0, dec!(0.0));
        assert_eq!(surface.y_range.1, dec!(1.0));
    }

    /// Projecting out `x` maps `(x, y, z)` to `(y, z)`, which is multi-valued
    /// on a grid: the test surface has two points at `y = 0` and two at
    /// `y = 1`, and all five survive the projection.
    ///
    /// The membership probes below are unchanged, but they are stricter than
    /// they were: `Point2D` used to compare on `x` alone, so `p == (0, 0)`
    /// also matched `(0, 1)`. It now means what it says.
    #[test]
    fn test_get_curve_x_axis() {
        let points = create_test_points();
        let surface = Surface::new(points);
        let curve = surface.get_curve(Axis::X);

        // Check curve points
        let curve_points: Vec<Point2D> = curve.points.into_iter().collect();

        // Verify the points are mapped correctly for X-axis curve
        assert!(
            curve_points
                .iter()
                .any(|p| p == &Point2D::new(dec!(0.0), dec!(0.0)))
        );
        assert!(
            curve_points
                .iter()
                .any(|p| p == &Point2D::new(dec!(1.0), dec!(1.0)))
        );

        let points = surface.get_f64_points();
        assert_eq!(points.len(), 5);
        assert_eq!(points[0].0, 0.0);
        assert_eq!(points[0].1, 0.0);
        assert_eq!(points[0].2, 0.0);

        let default = Surface::default();
        assert_eq!(default.points.len(), 0);
        assert_eq!(default.x_range, (Decimal::ZERO, Decimal::ZERO));
        assert_eq!(default.y_range, (Decimal::ZERO, Decimal::ZERO));

        let graph_data = surface.graph_data();
        assert!(matches!(
            graph_data,
            GraphData::GraphSurface(Surface3D { .. })
        ));
    }

    /// Projecting out `y` maps `(x, y, z)` to `(x, z)`, multi-valued on a
    /// grid for the same reason as the X-axis case above. The probes are
    /// unchanged and now mean exact membership.
    #[test]
    fn test_get_curve_y_axis() {
        let points = create_test_points();
        let surface = Surface::new(points);
        let curve = surface.get_curve(Axis::Y);

        // Check curve points
        let curve_points: Vec<Point2D> = curve.points.into_iter().collect();

        // Verify the points are mapped correctly for Y-axis curve
        assert!(
            curve_points
                .iter()
                .any(|p| p == &Point2D::new(dec!(0.0), dec!(0.0)))
        );
        assert!(
            curve_points
                .iter()
                .any(|p| p == &Point2D::new(dec!(1.0), dec!(2.0)))
        );
    }

    /// Projecting out `z` maps `(x, y, z)` to `(x, y)`: the xy-footprint of
    /// the surface, which on a grid has several ordinates per abscissa by
    /// construction. The probes are unchanged and now mean exact membership.
    #[test]
    fn test_get_curve_z_axis() {
        let points = create_test_points();
        let surface = Surface::new(points);
        let curve = surface.get_curve(Axis::Z);

        // Check curve points
        let curve_points: Vec<Point2D> = curve.points.into_iter().collect();

        // Verify the points are mapped correctly for Z-axis curve
        assert!(
            curve_points
                .iter()
                .any(|p| p == &Point2D::new(dec!(0.0), dec!(0.0)))
        );
        assert!(
            curve_points
                .iter()
                .any(|p| p == &Point2D::new(dec!(1.0), dec!(1.0)))
        );
    }

    #[test]
    fn test_one_dimensional_spline_interpolation_basic() {
        let surface = Surface::new(create_test_points());

        // Create test points for interpolation
        let test_points = vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(0.5), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
        ];

        // Test interpolation at different points
        let test_cases = vec![
            (dec!(0.25), dec!(0.5)), // Midpoint
            (dec!(0.0), dec!(0.0)),  // Start point
            (dec!(1.0), dec!(2.0)),  // End point
            (dec!(0.75), dec!(1.5)), // Another point
        ];

        for (target, expected) in test_cases {
            let result = surface
                .one_dimensional_spline_interpolation(&test_points, target, |p| p.x, |p| p.z)
                .unwrap();

            // Allow small deviation due to interpolation
            assert!(
                (result - expected).abs() < dec!(0.1),
                "Failed for target {target}, expected {expected}, got {result}"
            );
        }
    }

    #[test]
    fn test_one_dimensional_spline_interpolation_insufficient_points() {
        let surface = Surface::new(create_test_points());

        // Single point is insufficient for interpolation
        let test_points = vec![Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0))];

        let result =
            surface.one_dimensional_spline_interpolation(&test_points, dec!(0.5), |p| p.x, |p| p.z);

        assert!(matches!(
            result,
            Err(InterpolationError::Spline(msg)) if msg.contains("Insufficient points")
        ));
    }

    #[test]
    fn test_one_dimensional_spline_interpolation_out_of_range() {
        let surface = Surface::new(create_test_points());

        let test_points = vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
        ];

        // Test points outside the point range
        let out_of_range_cases = vec![
            (dec!(-0.5), dec!(0.0)), // Below minimum
            (dec!(1.5), dec!(2.0)),  // Above maximum
        ];

        for (target, expected) in out_of_range_cases {
            let result = surface
                .one_dimensional_spline_interpolation(&test_points, target, |p| p.x, |p| p.z)
                .unwrap();

            // Should return endpoints for out-of-range values
            assert_eq!(result, expected, "Failed for out-of-range target {target}");
        }
    }
}

#[cfg(test)]
mod tests_surface_geometric_object {
    use super::*;
    use crate::geometrics::ResultPoint;
    use rust_decimal_macros::dec;

    // Helper function to create test points
    fn create_test_points() -> Vec<Point3D> {
        vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]
    }

    #[test]
    fn test_get_points() {
        let points = create_test_points();
        let surface = Surface::from_vector(points.clone());

        let retrieved_points: Vec<&Point3D> = surface.get_points().into_iter().collect();

        assert_eq!(retrieved_points.len(), points.len());
        for point in &points {
            assert!(retrieved_points.contains(&point));
        }
    }

    #[test]
    fn test_from_vector() {
        let points = create_test_points();
        let surface = Surface::from_vector(points.clone());

        assert_eq!(surface.points.len(), points.len());

        // Check x and y ranges
        assert_eq!(surface.x_range.0, dec!(0.0));
        assert_eq!(surface.x_range.1, dec!(1.0));
        assert_eq!(surface.y_range.0, dec!(0.0));
        assert_eq!(surface.y_range.1, dec!(1.0));
    }

    #[test]
    fn test_construct_from_data() {
        let points = BTreeSet::from_iter(create_test_points());
        let result = Surface::construct(ConstructionMethod::FromData { points });

        assert!(result.is_ok());
        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 4);
    }

    #[test]
    fn test_construct_from_data_empty() {
        let points: BTreeSet<Point3D> = BTreeSet::new();
        let result = Surface::construct(ConstructionMethod::FromData { points });

        assert!(matches!(
            result,
            Err(SurfaceError::Point3DError { reason: _ })
        ));
    }

    #[test]
    fn test_construct_parametric() {
        // Create a simple parametric function that creates a basic surface
        let parametric_func: Box<dyn Fn(Point2D) -> ResultPoint<Point3D> + Send + Sync> =
            Box::new(move |t: Point2D| -> ResultPoint<Point3D> {
                Ok(Point3D::new(
                    t.x,
                    t.y,
                    t.x * t.y, // Simple z = x * y surface
                ))
            });

        let params = ConstructionParams::D3 {
            x_start: dec!(0.0),
            x_end: dec!(1.0),
            y_start: dec!(0.0),
            y_end: dec!(1.0),
            x_steps: 2,
            y_steps: 2,
        };

        let result = Surface::construct(ConstructionMethod::Parametric {
            f: parametric_func,
            params,
        });

        assert!(result.is_ok());
        let surface = result.unwrap();

        // Should have (x_steps + 1) * (y_steps + 1) points
        assert_eq!(surface.points.len(), 9); // 3x3 grid
    }

    #[test]
    fn test_construct_parametric_invalid_params() {
        let parametric_func: Box<dyn Fn(Point2D) -> ResultPoint<Point3D> + Send + Sync> =
            Box::new(move |_: Point2D| -> ResultPoint<Point3D> {
                Ok(Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)))
            });

        // Use incorrect parameters to trigger error
        let params = ConstructionParams::D2 {
            t_start: Decimal::ZERO,
            t_end: Decimal::ONE,
            steps: 2,
        };

        let result = Surface::construct(ConstructionMethod::Parametric {
            f: parametric_func,
            params,
        });

        assert!(matches!(result, Err(SurfaceError::ConstructionError(_))));
    }

    #[test]
    fn test_construct_parametric_error_handling() {
        // Parametric function that sometimes fails
        let parametric_func: Box<dyn Fn(Point2D) -> ResultPoint<Point3D> + Send + Sync> =
            Box::new(move |t: Point2D| -> ResultPoint<Point3D> {
                if t.x > dec!(0.5) && t.y > dec!(0.5) {
                    Err(crate::error::ChainError::invalid_parameters(
                        "parametric_f",
                        "Test error",
                    ))
                } else {
                    Ok(Point3D::new(t.x, t.y, t.x * t.y))
                }
            });

        let params = ConstructionParams::D3 {
            x_start: dec!(0.0),
            x_end: dec!(1.0),
            y_start: dec!(0.0),
            y_end: dec!(1.0),
            x_steps: 2,
            y_steps: 2,
        };

        let result = Surface::construct(ConstructionMethod::Parametric {
            f: parametric_func,
            params,
        });

        assert!(matches!(result, Err(SurfaceError::ConstructionError(_))));
    }

    #[test]
    fn test_range_calculation() {
        let points = create_test_points();
        let surface = Surface::from_vector(points);

        // Verify x and y ranges
        assert_eq!(surface.x_range.0, dec!(0.0));
        assert_eq!(surface.x_range.1, dec!(1.0));
        assert_eq!(surface.y_range.0, dec!(0.0));
        assert_eq!(surface.y_range.1, dec!(1.0));
    }
}

#[cfg(test)]
mod tests_surface_linear_interpolation {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_point_out_of_range() {
        let surface = create_test_surface();
        let result = surface.linear_interpolate(Point2D::new(dec!(-1.0), dec!(0.5)));
        assert!(matches!(
            result,
            Err(InterpolationError::Linear(msg)) if msg.contains("outside the surface's range")
        ));
    }

    #[test]
    fn test_exact_point_match() {
        let surface = create_test_surface();
        let result = surface
            .linear_interpolate(Point2D::new(dec!(0.0), dec!(0.0)))
            .unwrap();
        assert_eq!(result.z, dec!(0.0));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface
            .linear_interpolate(Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();
        assert_eq!(result.z, dec!(1.0));
    }

    #[test]
    fn test_quarter_point_interpolation() {
        let surface = create_test_surface();
        let result = surface
            .linear_interpolate(Point2D::new(dec!(0.25), dec!(0.25)))
            .unwrap();
        // El valor debe estar entre 0.0 y 1.0
        assert!(result.z > dec!(0.0) && result.z < dec!(1.0));
    }

    #[test]
    fn test_degenerate_triangle() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(1.0), dec!(1.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface.linear_interpolate(Point2D::new(dec!(1.0), dec!(1.0)));
        assert!(matches!(
            result,
            Err(InterpolationError::Linear(msg)) if msg.contains("Degenerate triangle")
        ));
    }

    #[test]
    fn test_boundary_interpolation() {
        let surface = create_test_surface();
        // Test interpolation on the edge
        let result = surface
            .linear_interpolate(Point2D::new(dec!(0.0), dec!(0.5)))
            .unwrap();
        assert_eq!(result.z, dec!(0.5));
    }

    #[test]
    fn test_uniform_gradient() {
        // Crear una superficie con un gradiente uniforme
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]);
        let surface = Surface::new(points);

        // Interpolation at any point should maintain the gradient
        let result = surface
            .linear_interpolate(Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();
        assert_eq!(result.z, dec!(1.0));
    }

    #[test]
    fn test_interpolation_precision() {
        let surface = create_test_surface();
        let result = surface
            .linear_interpolate(Point2D::new(dec!(0.333333), dec!(0.333333)))
            .unwrap();
        // Verify that the result has the expected precision
        assert!(result.z >= dec!(0.0) && result.z <= dec!(2.0));
    }
}

#[cfg(test)]
mod tests_surface_bilinear_interpolation {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)), // Bottom-left
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)), // Bottom-right
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)), // Top-left
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)), // Top-right
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_insufficient_points() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(2.0), dec!(2.0), dec!(2.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.5), dec!(0.5)));
        assert!(matches!(
            result,
            Err(InterpolationError::Bilinear(msg)) if msg.contains("Need at least four points")
        ));
    }

    #[test]
    fn test_point_out_of_range() {
        let surface = create_test_surface();
        let result = surface.bilinear_interpolate(Point2D::new(dec!(-1.0), dec!(0.5)));
        assert!(matches!(
            result,
            Err(InterpolationError::Bilinear(msg)) if msg.contains("outside the surface's range")
        ));
    }

    #[test]
    fn test_exact_point_match() {
        let surface = create_test_surface();
        let result = surface
            .bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.0)))
            .unwrap();
        assert_eq!(result.z, dec!(0.0));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface
            .bilinear_interpolate(Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();
        // At the midpoint, we expect the average of surrounding values
        assert_eq!(result.z, dec!(1.0));
    }

    #[test]
    fn test_quarter_point_interpolation() {
        let surface = create_test_surface();
        let result = surface
            .bilinear_interpolate(Point2D::new(dec!(0.25), dec!(0.25)))
            .unwrap();
        // Value should be between 0.0 and 1.0
        assert!(result.z > dec!(0.0) && result.z < dec!(1.0));
    }

    /// Four points stacked on one xy-coordinate are an invalid quadrilateral,
    /// and `bilinear_interpolate` says so.
    ///
    /// This test used to assert the *"Need at least four points"* message
    /// instead, and it reached it because `BTreeSet::from_iter` sorts and then
    /// deduplicates adjacent elements with `PartialEq`, not with `Ord`. With
    /// `Point3D` comparing equal on `(x, y)`, these four points collapsed to
    /// **one** on the way into the set, so the length guard fired and the
    /// invalid-quadrilateral check below it was unreachable. The same four
    /// values inserted one by one produced a set of four, because `insert`
    /// dispatches on `Ord` — the same container, two different contents,
    /// decided by which constructor was used.
    ///
    /// `PartialEq` now agrees with `Ord`, so the set holds all four and the
    /// check the test is named after is the one that runs.
    #[test]
    fn test_invalid_quadrilateral() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(0.0), dec!(0.0), dec!(3.0)),
        ]);
        assert_eq!(points.len(), 4, "all four heights are distinct points");

        let surface = Surface::new(points);
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.0)));
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(InterpolationError::Bilinear(msg)) if msg.contains("Invalid quadrilateral")
        ));
    }

    #[test]
    fn test_boundary_interpolation() {
        let surface = create_test_surface();
        // Test interpolation at edge
        let result = surface
            .bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.5)))
            .unwrap();
        assert_eq!(result.z, dec!(0.5));
    }

    #[test]
    fn test_uniform_gradient() {
        // Create a surface with uniform gradient
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface
            .bilinear_interpolate(Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();
        assert_eq!(result.z, dec!(1.0));
    }

    #[test]
    fn test_interpolation_precision() {
        let surface = create_test_surface();
        let result = surface
            .bilinear_interpolate(Point2D::new(dec!(0.333333), dec!(0.333333)))
            .unwrap();
        // Verify that the result has the expected precision
        assert!(result.z >= dec!(0.0) && result.z <= dec!(2.0));
    }

    #[test]
    fn test_corners_interpolation() {
        let surface = create_test_surface();

        // Test all four corners
        let bl = surface
            .bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.0)))
            .unwrap();
        let br = surface
            .bilinear_interpolate(Point2D::new(dec!(1.0), dec!(0.0)))
            .unwrap();
        let tl = surface
            .bilinear_interpolate(Point2D::new(dec!(0.0), dec!(1.0)))
            .unwrap();
        let tr = surface
            .bilinear_interpolate(Point2D::new(dec!(1.0), dec!(1.0)))
            .unwrap();

        assert_eq!(bl.z, dec!(0.0));
        assert_eq!(br.z, dec!(1.0));
        assert_eq!(tl.z, dec!(1.0));
        assert_eq!(tr.z, dec!(2.0));
    }
}

#[cfg(test)]
mod tests_surface_cubic_interpolation {
    use super::*;
    use rust_decimal_macros::dec;

    /// Helper function to create a test surface with a more complex point distribution
    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(1.5)),
            Point3D::new(dec!(0.2), dec!(0.8), dec!(0.7)),
            Point3D::new(dec!(0.8), dec!(0.2), dec!(0.7)),
            Point3D::new(dec!(0.3), dec!(0.3), dec!(0.3)),
            Point3D::new(dec!(0.7), dec!(0.7), dec!(1.7)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_insufficient_points() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(2.0), dec!(2.0), dec!(2.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface.cubic_interpolate(Point2D::new(dec!(0.5), dec!(0.5)));

        assert!(matches!(
            result,
            Err(InterpolationError::Cubic(msg)) if msg.contains("Need at least nine points")
        ));
    }

    #[test]
    fn test_point_out_of_range() {
        let surface = create_test_surface();
        let result = surface.cubic_interpolate(Point2D::new(dec!(2.0), dec!(2.0)));

        assert!(matches!(
            result,
            Err(InterpolationError::Cubic(msg)) if msg.contains("outside the surface's range")
        ));
    }

    #[test]
    fn test_exact_point_match() {
        let surface = create_test_surface();
        let result = surface
            .cubic_interpolate(Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();

        assert_eq!(result.z, dec!(1.5));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface
            .cubic_interpolate(Point2D::new(dec!(0.4), dec!(0.4)))
            .unwrap();

        // Verify that the interpolated z is between the surrounding points
        assert!(result.z > dec!(0.3) && result.z < dec!(1.5));
    }

    #[test]
    fn test_interpolation_consistency() {
        let surface = create_test_surface();

        // Test multiple interpolation points
        let test_points = vec![
            Point2D::new(dec!(0.2), dec!(0.2)),
            Point2D::new(dec!(0.6), dec!(0.6)),
            Point2D::new(dec!(0.8), dec!(0.3)),
        ];

        for point in test_points {
            let result = surface.cubic_interpolate(point).unwrap();

            // Verify z is within reasonable bounds
            assert!(
                result.z >= dec!(0.0) && result.z <= dec!(2.0),
                "Failed for point {point:?}"
            );

            // Verify the interpolated point is on the surface
            assert_eq!(result.x, point.x);
            assert_eq!(result.y, point.y);
        }
    }

    #[test]
    fn test_boundary_interpolation() {
        let surface = create_test_surface();

        // Test interpolation near surface boundaries
        let boundary_points = vec![
            Point2D::new(dec!(0.0), dec!(0.5)),
            Point2D::new(dec!(0.5), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(0.5)),
            Point2D::new(dec!(0.5), dec!(1.0)),
        ];

        for point in boundary_points {
            let result = surface.cubic_interpolate(point).unwrap();

            // Verify z is interpolated correctly
            assert!(
                result.z > dec!(0.0) && result.z < dec!(2.0),
                "Failed for boundary point {point:?}"
            );
        }
    }

    #[test]
    fn test_interpolation_precision() {
        let surface = create_test_surface();
        let result = surface
            .cubic_interpolate(Point2D::new(dec!(0.333333), dec!(0.333333)))
            .unwrap();

        // Verify precision and reasonable interpolation
        assert!(result.z > dec!(0.0) && result.z < dec!(2.0));
    }

    #[test]
    fn test_repeated_interpolation() {
        let surface = create_test_surface();

        // Interpolate the same point multiple times to check consistency
        let point = Point2D::new(dec!(0.4), dec!(0.4));
        let results: Vec<Decimal> = (0..5)
            .map(|_| surface.cubic_interpolate(point).unwrap().z)
            .collect();

        // Check that results are very close to each other
        let max_diff = results
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            - results
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        assert!(
            max_diff < dec!(0.001),
            "Interpolation results should be consistent"
        );
    }

    #[test]
    fn test_extreme_point_locations() {
        let surface = create_test_surface();

        // Test points very close to existing points
        let extreme_points = vec![
            Point2D::new(dec!(0.001), dec!(0.001)),
            Point2D::new(dec!(0.999), dec!(0.999)),
        ];

        for point in extreme_points {
            let result = surface.cubic_interpolate(point).unwrap();

            // Verify z is interpolated reasonably
            assert!(
                result.z >= dec!(0.0) && result.z <= dec!(2.0),
                "Failed for extreme point {point:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests_surface_spline_interpolation {
    use super::*;
    use rust_decimal_macros::dec;

    /// Helper function to create a test surface with a more complex point distribution
    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(1.5)),
            Point3D::new(dec!(0.2), dec!(0.8), dec!(0.7)),
            Point3D::new(dec!(0.8), dec!(0.2), dec!(0.7)),
            Point3D::new(dec!(0.3), dec!(0.3), dec!(0.3)),
            Point3D::new(dec!(0.7), dec!(0.7), dec!(1.7)),
            Point3D::new(dec!(0.4), dec!(0.6), dec!(1.1)),
            Point3D::new(dec!(0.6), dec!(0.4), dec!(1.2)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_insufficient_points() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface.spline_interpolate(Point2D::new(dec!(0.5), dec!(0.5)));

        assert!(matches!(
            result,
            Err(InterpolationError::Spline(msg)) if msg.contains("Need at least nine points")
        ));
    }

    #[test]
    fn test_point_out_of_range() {
        let surface = create_test_surface();
        let result = surface.spline_interpolate(Point2D::new(dec!(2.0), dec!(2.0)));

        assert!(matches!(
            result,
            Err(InterpolationError::Spline(msg)) if msg.contains("outside the surface's range")
        ));
    }

    #[test]
    fn test_exact_point_match() {
        let surface = create_test_surface();
        let result = surface
            .spline_interpolate(Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();

        assert_eq!(result.z, dec!(1.5));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface
            .spline_interpolate(Point2D::new(dec!(0.4), dec!(0.4)))
            .unwrap();

        // Verify that the interpolated z is between the surrounding points
        assert!(result.z > dec!(0.3) && result.z < dec!(1.5));
    }

    #[test]
    fn test_interpolation_consistency() {
        let surface = create_test_surface();

        // Test multiple interpolation points
        let test_points = vec![
            Point2D::new(dec!(0.2), dec!(0.2)),
            Point2D::new(dec!(0.6), dec!(0.6)),
            Point2D::new(dec!(0.8), dec!(0.3)),
        ];

        for point in test_points {
            let result = surface.spline_interpolate(point).unwrap();

            // Verify z is within reasonable bounds
            assert!(
                result.z >= dec!(0.0) && result.z <= dec!(2.0),
                "Failed for point {point:?}"
            );

            // Verify the interpolated point is on the surface
            assert_eq!(result.x, point.x);
            assert_eq!(result.y, point.y);
        }
    }

    #[test]
    fn test_boundary_interpolation() {
        let surface = create_test_surface();

        // Test interpolation near surface boundaries
        let boundary_points = vec![
            Point2D::new(dec!(0.0), dec!(0.5)),
            Point2D::new(dec!(0.5), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(0.5)),
            Point2D::new(dec!(0.5), dec!(1.0)),
        ];

        for point in boundary_points {
            let result = surface.spline_interpolate(point).unwrap();

            // Verify z is interpolated correctly
            assert!(
                result.z > dec!(0.0) && result.z < dec!(2.0),
                "Failed for boundary point {point:?}"
            );
        }
    }

    #[test]
    fn test_interpolation_precision() {
        let surface = create_test_surface();
        let result = surface
            .spline_interpolate(Point2D::new(dec!(0.333333), dec!(0.333333)))
            .unwrap();

        // Verify precision and reasonable interpolation
        assert!(result.z > dec!(0.0) && result.z < dec!(2.0));
    }

    #[test]
    fn test_repeated_interpolation() {
        let surface = create_test_surface();

        // Interpolate the same point multiple times to check consistency
        let point = Point2D::new(dec!(0.4), dec!(0.4));
        let results: Vec<Decimal> = (0..5)
            .map(|_| surface.spline_interpolate(point).unwrap().z)
            .collect();

        // Check that results are very close to each other
        let max_diff = results
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            - results
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        assert!(
            max_diff < dec!(0.001),
            "Interpolation results should be consistent"
        );
    }

    #[test]
    fn test_extreme_point_locations() {
        let surface = create_test_surface();

        // Test points very close to existing points
        let extreme_points = vec![
            Point2D::new(dec!(0.001), dec!(0.001)),
            Point2D::new(dec!(0.999), dec!(0.999)),
        ];

        for point in extreme_points {
            let result = surface.spline_interpolate(point).unwrap();

            // Verify z is interpolated reasonably
            assert!(
                result.z >= dec!(0.0) && result.z <= dec!(2.0),
                "Failed for extreme point {point:?}"
            );
        }
    }

    #[test]
    fn test_one_dimensional_spline_interpolation() {
        let surface = create_test_surface();

        // Create test points for one-dimensional interpolation
        let points = vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(0.5), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
        ];

        // Test interpolation at different points
        let test_points = vec![
            (dec!(0.25), dec!(0.5)), // Midpoint
            (dec!(0.0), dec!(0.0)),  // Start point
            (dec!(1.0), dec!(2.0)),  // End point
            (dec!(0.75), dec!(1.5)), // Another point
        ];

        for (target, expected) in test_points {
            let result = surface
                .one_dimensional_spline_interpolation(&points, target, |p| p.x, |p| p.z)
                .unwrap();

            // Allow small deviation due to interpolation
            assert!(
                (result - expected).abs() < dec!(0.1),
                "Failed for target {target}, expected {expected}, got {result}"
            );
        }
    }

    #[test]
    fn test_interpolation_edge_cases() {
        let surface = create_test_surface();

        // Test edge cases like very small intervals
        let edge_points = vec![
            Point2D::new(dec!(0.001), dec!(0.001)),
            Point2D::new(dec!(0.999), dec!(0.999)),
            Point2D::new(dec!(0.5), dec!(0.5)),
        ];

        for point in edge_points {
            let result = surface.spline_interpolate(point);
            assert!(result.is_ok(), "Failed for point {point:?}");

            let interpolated_point = result.unwrap();
            assert_eq!(interpolated_point.x, point.x);
            assert_eq!(interpolated_point.y, point.y);
        }
    }
}

#[cfg(test)]
mod tests_surface_arithmetic {
    use super::*;
    use crate::error::OperationErrorKind;
    use rust_decimal_macros::dec;

    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(0.5), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.5), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_merge_empty_surfaces() {
        let result = Surface::merge(&[], MergeOperation::Add);
        assert!(matches!(
            result,
            Err(SurfaceError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "merge_surfaces" && reason.contains("No surfaces")
        ));
    }

    #[test]
    fn test_merge_single_surface() {
        let surface = create_test_surface();
        let result = Surface::merge(&[&surface], MergeOperation::Add).unwrap();
        assert_eq!(result.points.len(), surface.points.len());
    }

    #[test]
    fn test_merge_add() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();
        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Add).unwrap();

        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid_point.z, dec!(2.0));
    }

    #[test]
    fn test_merge_subtract() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();
        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Subtract).unwrap();

        // Test point should have z-value of 0 (1.0 - 1.0)
        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid_point.z, dec!(0.0));
    }

    fn constant_surface(height: Decimal) -> Surface {
        let points =
            BTreeSet::from_iter([dec!(0.0), dec!(0.5), dec!(1.0)].into_iter().flat_map(|x| {
                [dec!(0.0), dec!(0.5), dec!(1.0)]
                    .into_iter()
                    .map(move |y| Point3D::new(x, y, height))
            }));
        Surface::new(points)
    }

    /// Division is not associative, so the divisors have to be folded in
    /// sequence. Three surfaces at 8, 2 and 2 must give `8 / 2 / 2 = 2`; a
    /// reducer that keeps only its left input drops the trailing divisors and
    /// lands somewhere between 4 and 8.
    #[test]
    fn test_merge_divide_folds_every_divisor() {
        let eight = constant_surface(dec!(8));
        let two_a = constant_surface(dec!(2));
        let two_b = constant_surface(dec!(2));

        let result = Surface::merge(&[&eight, &two_a, &two_b], MergeOperation::Divide).unwrap();

        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert!(
            (mid_point.z - dec!(2)).abs() < dec!(0.0001),
            "expected 8 / 2 / 2 = 2, got {}",
            mid_point.z
        );
    }

    /// A divisor that fails the checked division has to surface as an error
    /// rather than being discarded together with the fold result that
    /// produced it. `1e20 / 1e-20` is `1e40`, well past `Decimal::MAX`, while
    /// both operands stay far enough inside the range that the interpolation
    /// feeding the fold cannot fail first.
    #[test]
    fn test_merge_divide_reports_overflow_instead_of_dropping_it() {
        let tiny = constant_surface(Decimal::new(1, 20));
        let big = constant_surface(dec!(100000000000000000000));

        let result = Surface::merge(&[&big, &tiny], MergeOperation::Divide);
        assert!(
            matches!(result, Err(SurfaceError::ConstructionError(_))),
            "a quotient outside the Decimal range must be reported, got {result:?}"
        );
    }

    #[test]
    fn test_incompatible_ranges() {
        let surface1 = Surface::new(BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ]));

        let surface2 = Surface::new(BTreeSet::from_iter(vec![
            Point3D::new(dec!(2.0), dec!(2.0), dec!(1.0)),
            Point3D::new(dec!(3.0), dec!(3.0), dec!(1.0)),
        ]));

        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Add);
        assert!(matches!(
            result,
            Err(SurfaceError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "merge_surfaces" && reason.contains("incompatible ranges")
        ));
    }

    #[test]
    fn test_merge_with() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();

        let result1 = surface1.merge_with(&surface2, MergeOperation::Add).unwrap();
        let result2 = Surface::merge(&[&surface1, &surface2], MergeOperation::Add).unwrap();

        assert_eq!(result1.points.len(), result2.points.len());

        // Compare some interpolated points
        let test_point = Point2D::new(dec!(0.5), dec!(0.5));
        let z1 = result1
            .interpolate(test_point, InterpolationType::Cubic)
            .unwrap();
        let z2 = result2
            .interpolate(test_point, InterpolationType::Cubic)
            .unwrap();
        assert_eq!(z1.z, z2.z);
    }

    #[test]
    fn test_merge_multiply() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();
        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Multiply).unwrap();

        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid_point.z, dec!(1.0)); // 1.0 * 1.0 = 1.0
    }

    #[test]
    fn test_merge_divide() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();
        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Divide).unwrap();

        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid_point.z, dec!(1.0)); // 1.0 / 1.0 = 1.0
    }

    #[test]
    fn test_merge_max() {
        let surface1 = create_test_surface(); // z=1.0 everywhere

        // Create surface2 with z=2.0 everywhere
        let points2 = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(0.0), dec!(0.5), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(2.0)),
            Point3D::new(dec!(1.0), dec!(0.5), dec!(2.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(1.0), dec!(2.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]);
        let surface2 = Surface::new(points2);

        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Max).unwrap();

        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid_point.z, dec!(2.0));
    }

    #[test]
    fn test_merge_min() {
        let surface1 = create_test_surface();
        let mut surface2 = create_test_surface();

        // Modify one point in surface2 to be lower
        surface2
            .points
            .insert(Point3D::new(dec!(0.5), dec!(0.5), dec!(0.5)));

        let result = Surface::merge(&[&surface1, &surface2], MergeOperation::Min).unwrap();

        let mid_point = result
            .interpolate(Point2D::new(dec!(0.5), dec!(0.5)), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid_point.z, dec!(0.5));
    }
}

#[cfg(test)]
mod tests_metrics {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.5), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(3.0)),
            Point3D::new(dec!(0.0), dec!(0.5), dec!(2.0)),
            Point3D::new(dec!(0.5), dec!(0.5), dec!(3.0)),
            Point3D::new(dec!(1.0), dec!(0.5), dec!(4.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(3.0)),
            Point3D::new(dec!(0.5), dec!(1.0), dec!(4.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(5.0)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_basic_metrics() {
        let surface = create_test_surface();
        let metrics = surface.compute_basic_metrics().unwrap();

        assert_eq!(metrics.mean, dec!(3.0));
        assert_eq!(metrics.median, dec!(3.0));
        assert_eq!(metrics.std_dev, dec!(1.1547005383792515290182975610));
    }

    #[test]
    fn test_shape_metrics() {
        let surface = create_test_surface();
        let metrics = surface.compute_shape_metrics().unwrap();

        assert!(metrics.skewness.abs() < dec!(0.001));
        assert!((metrics.kurtosis - dec!(2.25)).abs() < dec!(0.001));
    }

    #[test]
    fn test_range_metrics() {
        let surface = create_test_surface();
        let metrics = surface.compute_range_metrics().unwrap();

        assert_eq!(metrics.min.y, dec!(1.0));
        assert_eq!(metrics.max.y, dec!(5.0));
        assert_eq!(metrics.range, dec!(4.0));

        let (q1, q2, q3) = metrics.quartiles;
        assert_eq!(q1, dec!(2.0));
        assert_eq!(q2, dec!(3.0));
        assert_eq!(q3, dec!(4.0));
        assert_eq!(metrics.interquartile_range, dec!(2.0));
    }

    #[test]
    fn test_trend_metrics() {
        let surface = create_test_surface();
        let metrics = surface.compute_trend_metrics().unwrap();

        // We have a linear trend with slope 2.0
        assert!((metrics.slope - dec!(2.0)).abs() < dec!(0.001));
        assert!((metrics.intercept - dec!(2.0)).abs() < dec!(0.001));
    }

    /// A flat surface has no uncertainty, but it is not worthless. Only the
    /// fields that are genuinely undefined at zero dispersion may be zeroed;
    /// the parametric VaR still has its deterministic limit at the mean.
    #[test]
    fn test_risk_metrics_flat_surface_keeps_deterministic_var() {
        let points = BTreeSet::from_iter((0..3i64).flat_map(|i| {
            (0..3i64).map(move |j| Point3D::new(Decimal::from(i), Decimal::from(j), dec!(5)))
        }));
        let surface = Surface::new(points);
        let metrics = surface.compute_risk_metrics().unwrap();

        // Measured, and genuinely zero.
        assert_eq!(metrics.volatility, Decimal::ZERO);
        // `mean - 1.645 * 0` is the mean, not zero: the surface is worth 5.
        assert_eq!(metrics.value_at_risk, dec!(5));
        // No sample falls below the VaR, so the conditional mean has an empty
        // tail and the function's own empty-tail rule gives zero.
        assert_eq!(metrics.expected_shortfall, Decimal::ZERO);
        // Not implemented yet, zero either way.
        assert_eq!(metrics.beta, Decimal::ZERO);
        // `mean / 0` is the one genuinely undefined field.
        assert_eq!(metrics.sharpe_ratio, Decimal::ZERO);
    }

    /// The same shape with a negative mean: the VaR limit follows the mean
    /// wherever it sits, so a sign error cannot hide behind a positive value.
    #[test]
    fn test_risk_metrics_flat_negative_surface_keeps_deterministic_var() {
        let points = BTreeSet::from_iter((0..3i64).flat_map(|i| {
            (0..3i64).map(move |j| Point3D::new(Decimal::from(i), Decimal::from(j), dec!(-2.5)))
        }));
        let surface = Surface::new(points);
        let metrics = surface.compute_risk_metrics().unwrap();

        assert_eq!(metrics.volatility, Decimal::ZERO);
        assert_eq!(metrics.value_at_risk, dec!(-2.5));
        assert_eq!(metrics.sharpe_ratio, Decimal::ZERO);
    }
}

#[cfg(test)]
mod tests_trend_metrics {
    use super::*;
    use crate::assert_decimal_eq;
    use rust_decimal_macros::dec;

    // Helper function to create a test surface
    fn create_linear_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
            Point3D::new(dec!(2.0), dec!(2.0), dec!(4.0)),
            Point3D::new(dec!(3.0), dec!(3.0), dec!(6.0)),
            Point3D::new(dec!(4.0), dec!(4.0), dec!(8.0)),
        ]);
        Surface::new(points)
    }

    // Helper function to create a non-linear surface
    fn create_non_linear_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(3.0)),
            Point3D::new(dec!(2.0), dec!(2.0), dec!(2.0)),
            Point3D::new(dec!(3.0), dec!(3.0), dec!(5.0)),
            Point3D::new(dec!(4.0), dec!(4.0), dec!(4.0)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_compute_trend_metrics_linear_surface() {
        let surface = create_linear_surface();
        let metrics = surface.compute_trend_metrics().unwrap();

        // Check slope (should be 2.0 for a perfectly linear surface)
        assert_decimal_eq!(metrics.slope, dec!(2.0), dec!(0.001));

        // Check intercept (should be close to 0)
        assert_decimal_eq!(metrics.intercept, dec!(0.0), dec!(0.001));

        // R-squared should be very close to 1 for a perfect linear relationship
        assert_decimal_eq!(metrics.r_squared, dec!(1.0), dec!(0.001));

        // Check moving average points
        assert_eq!(metrics.moving_average.len(), 4);
    }

    #[test]
    fn test_compute_trend_metrics_non_linear_surface() {
        let surface = create_non_linear_surface();
        let metrics = surface.compute_trend_metrics().unwrap();

        // R-squared should be less than 1 for a non-perfect linear relationship
        assert!(metrics.r_squared < dec!(1.0));

        // Slope and intercept will vary based on the non-linear surface
        assert!(metrics.slope != dec!(0.0));
        assert!(metrics.intercept != dec!(0.0));
    }

    #[test]
    fn test_moving_average_calculation() {
        let surface = create_linear_surface();
        let metrics = surface.compute_trend_metrics().unwrap();

        // Verify moving average calculation
        let window_sizes = [3, 5, 7];

        // Calculate total points safely
        let surface_points_count = surface.points.len();

        let expected_total_points = window_sizes
            .iter()
            .map(|&window| {
                // Safely handle cases where window might be larger than points
                if window > surface_points_count {
                    0
                } else {
                    surface_points_count
                        .saturating_sub(window)
                        .saturating_add(1)
                }
            })
            .sum::<usize>();

        // Assert with more informative message
        assert_eq!(
            metrics.moving_average.len(),
            expected_total_points,
            "Mismatch in moving average points calculation"
        );

        // Verify x and y values in moving average
        for point in &metrics.moving_average {
            assert!(point.x >= dec!(0.0), "x value should be non-negative");
            assert!(point.y >= dec!(0.0), "y value should be non-negative");
        }
    }

    #[test]
    fn test_edge_cases() {
        // Surface with a single point
        let single_point_surface = Surface::new(BTreeSet::from_iter(vec![Point3D::new(
            dec!(1.0),
            dec!(1.0),
            dec!(1.0),
        )]));

        let metrics = single_point_surface.compute_trend_metrics();
        assert!(metrics.is_ok());

        // Surface with identical points
        let identical_points_surface = Surface::new(BTreeSet::from_iter(vec![
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ]));

        let metrics = identical_points_surface.compute_trend_metrics().unwrap();

        // For identical points, R-squared should be 1
        assert_decimal_eq!(metrics.r_squared, dec!(1.0), dec!(0.001));
        assert_decimal_eq!(metrics.slope, dec!(0.0), dec!(0.001));
    }
}

#[cfg(test)]
mod tests_axis_operations {
    use super::*;
    use rust_decimal_macros::dec;

    // Create a test Surface with predefined points
    fn create_test_surface() -> Surface {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(3.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(4.0)),
        ]);
        Surface::new(points)
    }

    #[test]
    fn test_contains_point() {
        let surface = create_test_surface();
        assert!(surface.contains_point(&Point2D::new(dec!(0.0), dec!(0.0))));
        assert!(!surface.contains_point(&Point2D::new(dec!(2.0), dec!(2.0))));
    }

    #[test]
    fn test_get_index_values() {
        let surface = create_test_surface();
        let indexes = surface.get_index_values();
        assert_eq!(indexes.len(), 4);
        assert!(indexes.contains(&Point2D::new(dec!(0.0), dec!(0.0))));
        assert!(indexes.contains(&Point2D::new(dec!(1.0), dec!(1.0))));
    }

    #[test]
    fn test_get_values() {
        let surface = create_test_surface();
        let values = surface.get_values(Point2D::new(dec!(0.0), dec!(0.0)));
        assert_eq!(values.len(), 1);
        assert_eq!(*values[0], dec!(1.0));
    }

    #[test]
    fn test_get_closest_point() {
        let surface = create_test_surface();
        let point = surface
            .get_closest_point(&Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();
        assert_eq!(point.x, dec!(0.0));
        assert_eq!(point.y, dec!(0.0));
        assert_eq!(point.z, dec!(1.0));
    }

    #[test]
    fn test_get_point() {
        let surface = create_test_surface();
        let point = surface
            .get_point(&Point2D::new(dec!(0.0), dec!(0.0)))
            .unwrap();
        assert_eq!(point.x, dec!(0.0));
        assert_eq!(point.y, dec!(0.0));
        assert_eq!(point.z, dec!(1.0));

        assert!(
            surface
                .get_point(&Point2D::new(dec!(2.0), dec!(2.0)))
                .is_none()
        );
    }

    /// Merging a 2x2 grid with itself yields all four xy-coordinates.
    ///
    /// This test used to assert `merged.len() == 2`, and it passed. That was
    /// not the specification, it was data loss: `merge_indexes` funnels the
    /// index values through a `HashSet<Input>`, `Input` is `Point2D` for a
    /// surface, and `Point2D` hashed and compared on `x` alone — so both
    /// cells of every column collapsed into one and half the grid vanished
    /// before `merge_axis_interpolate` ever saw it. `Point2D` now carries
    /// both coordinates in `Eq` and `Hash`, so the axis survives the merge.
    #[test]
    fn test_merge_indexes() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();
        let merged = surface1.merge_indexes(surface2.get_index_values());

        assert_eq!(merged.len(), 4);
        for expected in [
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ] {
            assert!(merged.contains(&expected), "missing index {expected}");
        }
    }
}

#[cfg(test)]
mod tests_surface_geometric_transformations {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_surface() -> Surface {
        Surface::new(BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
        ]))
    }

    mod test_translate {
        use super::*;

        #[test]
        fn test_translate_positive() {
            let surface = create_test_surface();
            let result = surface
                .translate(vec![&dec!(1.0), &dec!(1.0), &dec!(1.0)])
                .unwrap();

            let translated_points: Vec<_> = result.points.iter().collect();
            assert_eq!(translated_points[0].x, dec!(1.0));
            assert_eq!(translated_points[0].y, dec!(1.0));
            assert_eq!(translated_points[0].z, dec!(1.0));
        }

        #[test]
        fn test_translate_negative() {
            let surface = create_test_surface();
            let result = surface
                .translate(vec![&dec!(-1.0), &dec!(-1.0), &dec!(-1.0)])
                .unwrap();

            let translated_points: Vec<_> = result.points.iter().collect();
            assert_eq!(translated_points[0].x, dec!(-1.0));
            assert_eq!(translated_points[0].y, dec!(-1.0));
            assert_eq!(translated_points[0].z, dec!(-1.0));
        }

        #[test]
        fn test_translate_zero() {
            let surface = create_test_surface();
            let result = surface
                .translate(vec![&dec!(0.0), &dec!(0.0), &dec!(0.0)])
                .unwrap();
            assert_eq!(surface.points, result.points);
        }

        #[test]
        fn test_translate_wrong_dimensions() {
            let surface = create_test_surface();
            let result = surface.translate(vec![&dec!(1.0), &dec!(1.0)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_translate_preserves_distances() {
            let surface = create_test_surface();
            let result = surface
                .translate(vec![&dec!(1.0), &dec!(1.0), &dec!(1.0)])
                .unwrap();

            let original_points: Vec<_> = surface.points.iter().collect();
            let translated_points: Vec<_> = result.points.iter().collect();

            let orig_dist = ((original_points[1].x - original_points[0].x).powi(2)
                + (original_points[1].y - original_points[0].y).powi(2)
                + (original_points[1].z - original_points[0].z).powi(2))
            .sqrt();

            let trans_dist = ((translated_points[1].x - translated_points[0].x).powi(2)
                + (translated_points[1].y - translated_points[0].y).powi(2)
                + (translated_points[1].z - translated_points[0].z).powi(2))
            .sqrt();

            assert_eq!(orig_dist, trans_dist);
        }
    }

    mod test_scale {
        use super::*;

        #[test]
        fn test_scale_uniform() {
            let surface = create_test_surface();
            let result = surface
                .scale(vec![&dec!(2.0), &dec!(2.0), &dec!(2.0)])
                .unwrap();
            assert_eq!(result[1].x, dec!(0.0));
            assert_eq!(result[1].y, dec!(2.0));
            assert_eq!(result[1].z, dec!(2.0));
        }

        #[test]
        fn test_scale_non_uniform() {
            let surface = create_test_surface();
            let result = surface
                .scale(vec![&dec!(2.0), &dec!(3.0), &dec!(4.0)])
                .unwrap();

            assert_eq!(result[0].x, dec!(0.0));
            assert_eq!(result[0].y, dec!(0.0));
            assert_eq!(result[0].z, dec!(0.0));
            assert_eq!(result[1].x, dec!(0.0));
            assert_eq!(result[1].y, dec!(3.0));
            assert_eq!(result[1].z, dec!(4.0));
            assert_eq!(result[2].x, dec!(2.0));
            assert_eq!(result[2].y, dec!(0.0));
            assert_eq!(result[2].z, dec!(4.0));
            assert_eq!(result[2].x, dec!(2.0));
            assert_eq!(result[2].y, dec!(0.0));
            assert_eq!(result[2].z, dec!(4.0));
        }

        #[test]
        fn test_scale_zero() {
            let surface = create_test_surface();
            let result = surface
                .scale(vec![&dec!(0.0), &dec!(0.0), &dec!(0.0)])
                .unwrap();

            assert!(
                result
                    .points
                    .iter()
                    .all(|p| p.x == dec!(0.0) && p.y == dec!(0.0) && p.z == dec!(0.0))
            );
        }

        #[test]
        fn test_scale_wrong_dimensions() {
            let surface = create_test_surface();
            let result = surface.scale(vec![&dec!(2.0), &dec!(2.0)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_scale_negative() {
            let surface = create_test_surface();
            let result = surface
                .scale(vec![&dec!(-1.0), &dec!(-1.0), &dec!(-1.0)])
                .unwrap();

            let scaled_points: Vec<_> = result.points.iter().collect();
            assert_eq!(scaled_points[1].x, dec!(-1.0));
            assert_eq!(scaled_points[1].y, dec!(0.0));
            assert_eq!(scaled_points[1].z, dec!(-1.0));
        }
    }

    mod test_intersect_with {
        use super::*;

        #[test]
        fn test_surfaces_intersect() {
            let surface1 = create_test_surface();
            let surface2 = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
                Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
            ]));

            let intersections = surface1.intersect_with(&surface2).unwrap();
            assert_eq!(intersections.len(), 2);
        }

        #[test]
        fn test_no_intersection() {
            let surface1 = create_test_surface();
            let surface2 = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(10.0), dec!(10.0), dec!(10.0)),
                Point3D::new(dec!(11.0), dec!(11.0), dec!(11.0)),
            ]));

            let intersections = surface1.intersect_with(&surface2).unwrap();
            assert!(intersections.is_empty());
        }

        #[test]
        fn test_multiple_intersections() {
            let surface1 = create_test_surface();
            let surface2 = create_test_surface();

            let intersections = surface1.intersect_with(&surface2).unwrap();
            assert_eq!(intersections.len(), surface1.points.len());
        }

        #[test]
        fn test_self_intersection() {
            let surface = create_test_surface();
            let intersections = surface.intersect_with(&surface).unwrap();
            assert_eq!(intersections.len(), surface.points.len());
        }

        #[test]
        fn test_empty_surfaces() {
            let surface1 = Surface::new(BTreeSet::new());
            let surface2 = Surface::new(BTreeSet::new());

            let intersections = surface1.intersect_with(&surface2).unwrap();
            assert!(intersections.is_empty());
        }
    }

    mod test_derivative_at {
        use super::*;

        #[test]
        fn test_planar_derivative() {
            let surface = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
                Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
                Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            ]));

            let derivatives = surface
                .derivative_at(&Point3D::new(dec!(0.5), dec!(0.5), dec!(0.5)))
                .unwrap();
            assert_eq!(derivatives.len(), 2);
            assert_eq!(derivatives[0], Decimal::MAX); // ∂z/∂x
            assert_eq!(derivatives[1], dec!(1.0)); // ∂z/∂y
        }

        #[test]
        fn test_non_planar_derivative() {
            let surface = create_test_surface();
            let derivatives = surface
                .derivative_at(&Point3D::new(dec!(0.5), dec!(0.5), dec!(1.0)))
                .unwrap();
            assert_eq!(derivatives.len(), 2);
        }

        #[test]
        fn test_out_of_range() {
            let surface = create_test_surface();
            let result = surface.derivative_at(&Point3D::new(dec!(10.0), dec!(10.0), dec!(10.0)));
            assert!(result.is_err());
        }

        #[test]
        fn test_at_corner() {
            let surface = create_test_surface();
            let derivatives = surface
                .derivative_at(&Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)))
                .unwrap();
            assert_eq!(derivatives.len(), 2);
        }

        #[test]
        fn test_single_point_surface() {
            let surface = Surface::new(BTreeSet::from_iter(vec![Point3D::new(
                dec!(1.0),
                dec!(1.0),
                dec!(1.0),
            )]));
            let result = surface.derivative_at(&Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)));
            assert!(result.is_err());
        }
    }

    mod test_extrema {
        use super::*;

        #[test]
        fn test_find_extrema() {
            let surface = create_test_surface();
            let (min, max) = surface.extrema().unwrap();
            assert_eq!(min.z, dec!(0.0));
            assert_eq!(max.z, dec!(2.0));
        }

        #[test]
        fn test_empty_surface() {
            let surface = Surface::new(BTreeSet::new());
            let result = surface.extrema();
            assert!(result.is_err());
        }

        #[test]
        fn test_single_point() {
            let surface = Surface::new(BTreeSet::from_iter(vec![Point3D::new(
                dec!(1.0),
                dec!(1.0),
                dec!(1.0),
            )]));

            let (min, max) = surface.extrema().unwrap();
            assert_eq!(min, max);
        }

        #[test]
        fn test_flat_surface() {
            let surface = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
                Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
                Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            ]));

            let (min, max) = surface.extrema().unwrap();
            assert_eq!(min.z, max.z);
        }

        #[test]
        fn test_multiple_extrema() {
            let surface = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
                Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),
                Point3D::new(dec!(2.0), dec!(2.0), dec!(0.0)),
            ]));

            let (min, max) = surface.extrema().unwrap();
            assert_eq!(min.z, dec!(0.0));
            assert_eq!(max.z, dec!(2.0));
        }
    }

    mod test_measure_under {
        use super::*;

        #[test]
        fn test_volume_under_planar() {
            let surface = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
                Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),
                Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            ]));

            let volume = surface.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(volume, dec!(0.5)); // Area of triangle * height
        }

        #[test]
        fn test_volume_empty_surface() {
            let surface = Surface::new(BTreeSet::new());
            let volume = surface.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(volume, dec!(0.0));
        }

        #[test]
        fn test_volume_single_triangle() {
            let surface = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
                Point3D::new(dec!(1.0), dec!(0.0), dec!(0.0)),
                Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),
            ]));

            let volume = surface.measure_under(&dec!(0.0)).unwrap();
            assert!(volume > dec!(0.0));
        }

        #[test]
        fn test_volume_with_base_value() {
            let surface = create_test_surface();
            let volume1 = surface.measure_under(&dec!(0.0)).unwrap();
            let volume2 = surface.measure_under(&dec!(1.0)).unwrap();
            assert!(volume1 > volume2);
        }

        #[test]
        fn test_negative_volume() {
            let surface = Surface::new(BTreeSet::from_iter(vec![
                Point3D::new(dec!(0.0), dec!(0.0), dec!(-1.0)),
                Point3D::new(dec!(1.0), dec!(0.0), dec!(-1.0)),
                Point3D::new(dec!(0.0), dec!(1.0), dec!(-1.0)),
            ]));

            let volume = surface.measure_under(&dec!(0.0)).unwrap();
            assert!(volume > dec!(0.0));
        }
    }
}

#[cfg(test)]
mod tests_surface_serde {
    use super::*;
    use rust_decimal_macros::dec;

    // Helper function to create a test surface
    fn create_test_surface() -> Surface {
        let mut points = BTreeSet::new();
        points.insert(Point3D {
            x: dec!(1.0),
            y: dec!(2.0),
            z: dec!(3.0),
        });
        points.insert(Point3D {
            x: dec!(4.0),
            y: dec!(5.0),
            z: dec!(6.0),
        });
        points.insert(Point3D {
            x: dec!(7.0),
            y: dec!(8.0),
            z: dec!(9.0),
        });

        Surface {
            points,
            x_range: (dec!(1.0), dec!(7.0)),
            y_range: (dec!(2.0), dec!(8.0)),
        }
    }

    #[test]
    fn test_basic_serialization() {
        let surface = create_test_surface();
        let serialized = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();

        assert_eq!(surface.points, deserialized.points);
        assert_eq!(surface.x_range, deserialized.x_range);
        assert_eq!(surface.y_range, deserialized.y_range);
    }

    #[test]
    fn test_pretty_print() {
        let surface = create_test_surface();
        let serialized = serde_json::to_string_pretty(&surface).unwrap();

        // Verify pretty print format
        assert!(serialized.contains('\n'));
        assert!(serialized.contains("  "));

        // Verify deserialization still works
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();
        assert_eq!(surface.points, deserialized.points);
    }

    #[test]
    fn test_empty_surface() {
        let surface = Surface {
            points: BTreeSet::new(),
            x_range: (dec!(0.0), dec!(0.0)),
            y_range: (dec!(0.0), dec!(0.0)),
        };

        let serialized = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.points.is_empty());
        assert_eq!(deserialized.x_range, (dec!(0.0), dec!(0.0)));
        assert_eq!(deserialized.y_range, (dec!(0.0), dec!(0.0)));
    }

    #[test]
    fn test_surface_with_negative_values() {
        let mut points = BTreeSet::new();
        points.insert(Point3D {
            x: dec!(-1.0),
            y: dec!(-2.0),
            z: dec!(-3.0),
        });
        points.insert(Point3D {
            x: dec!(-4.0),
            y: dec!(-5.0),
            z: dec!(-6.0),
        });

        let surface = Surface {
            points,
            x_range: (dec!(-4.0), dec!(-1.0)),
            y_range: (dec!(-5.0), dec!(-2.0)),
        };

        let serialized = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();

        assert_eq!(surface.points, deserialized.points);
        assert_eq!(surface.x_range, deserialized.x_range);
        assert_eq!(surface.y_range, deserialized.y_range);
    }

    #[test]
    fn test_surface_with_high_precision() {
        let mut points = BTreeSet::new();
        points.insert(Point3D {
            x: dec!(1.12345678901234567890),
            y: dec!(2.12345678901234567890),
            z: dec!(3.12345678901234567890),
        });
        points.insert(Point3D {
            x: dec!(4.12345678901234567890),
            y: dec!(5.12345678901234567890),
            z: dec!(6.12345678901234567890),
        });

        let surface = Surface {
            points,
            x_range: (dec!(1.12345678901234567890), dec!(4.12345678901234567890)),
            y_range: (dec!(2.12345678901234567890), dec!(5.12345678901234567890)),
        };

        let serialized = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();

        assert_eq!(surface.points, deserialized.points);
        assert_eq!(surface.x_range, deserialized.x_range);
        assert_eq!(surface.y_range, deserialized.y_range);
    }

    #[test]
    fn test_invalid_json() {
        // Missing required fields
        let json_str = r#"{"points": []}"#;
        let result = serde_json::from_str::<Surface>(json_str);
        assert!(result.is_err());

        // Invalid points format
        let json_str = r#"{"points": [1, 2, 3], "x_range": [0, 1], "y_range": [0, 1]}"#;
        let result = serde_json::from_str::<Surface>(json_str);
        assert!(result.is_err());

        // Invalid range format
        let json_str = r#"{"points": [], "x_range": "invalid", "y_range": [0, 1]}"#;
        let result = serde_json::from_str::<Surface>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_structure() {
        let surface = create_test_surface();
        let serialized = serde_json::to_string(&surface).unwrap();
        let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Check structure
        assert!(json.is_object());
        assert!(json.get("points").is_some());
        assert!(json.get("x_range").is_some());
        assert!(json.get("y_range").is_some());

        // Check points is an array
        assert!(json.get("points").unwrap().is_array());

        // Check ranges are arrays of 2 elements
        let x_range = json.get("x_range").unwrap().as_array().unwrap();
        let y_range = json.get("y_range").unwrap().as_array().unwrap();
        assert_eq!(x_range.len(), 2);
        assert_eq!(y_range.len(), 2);
    }

    #[test]
    fn test_multiple_surfaces() {
        let surface1 = create_test_surface();
        let mut surface2 = create_test_surface();
        surface2.x_range = (dec!(8.0), dec!(14.0));
        surface2.y_range = (dec!(9.0), dec!(15.0));

        let surfaces = vec![surface1, surface2];
        let serialized = serde_json::to_string(&surfaces).unwrap();
        let deserialized: Vec<Surface> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(surfaces.len(), deserialized.len());
        assert_eq!(surfaces[0].points, deserialized[0].points);
        assert_eq!(surfaces[1].points, deserialized[1].points);
    }

    #[test]
    fn test_ordering_preservation() {
        let surface = create_test_surface();
        let serialized = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();

        // Convert points to vectors to check ordering
        let original_points: Vec<_> = surface.points.into_iter().collect();
        let deserialized_points: Vec<_> = deserialized.points.into_iter().collect();

        // Check if points maintain their order
        assert_eq!(original_points, deserialized_points);
    }

    #[test]
    fn test_surface_with_extremes() {
        let mut points = BTreeSet::new();
        points.insert(Point3D {
            x: Decimal::MAX,
            y: Decimal::MAX,
            z: Decimal::MAX,
        });
        points.insert(Point3D {
            x: Decimal::MIN,
            y: Decimal::MIN,
            z: Decimal::MIN,
        });

        let surface = Surface {
            points,
            x_range: (Decimal::MIN, Decimal::MAX),
            y_range: (Decimal::MIN, Decimal::MAX),
        };

        let serialized = serde_json::to_string(&surface).unwrap();
        let deserialized: Surface = serde_json::from_str(&serialized).unwrap();

        assert_eq!(surface.points, deserialized.points);
        assert_eq!(surface.x_range, deserialized.x_range);
        assert_eq!(surface.y_range, deserialized.y_range);
    }

    #[test]
    fn test_surface_points_array_format() {
        // Test that points can be deserialized from array format
        let json_str = r#"{
            "points": [
                [1.0, 2.0, 3.0],
                [4.0, 5.0, 6.0]
            ],
            "x_range": [1.0, 4.0],
            "y_range": [2.0, 5.0]
        }"#;

        let result = serde_json::from_str::<Surface>(json_str);
        assert!(result.is_ok());

        let surface = result.unwrap();
        assert_eq!(surface.points.len(), 2);
    }
}
