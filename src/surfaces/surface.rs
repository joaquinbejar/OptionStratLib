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
use crate::error::{InterpolationError, MetricsError, SurfaceError};
use crate::geometrics::{
    Arithmetic, AxisOperations, BasicMetrics, BiLinearInterpolation, ConstructionMethod,
    ConstructionParams, CubicInterpolation, GeometricObject, GeometricTransformations, Interpolate,
    InterpolationType, LinearInterpolation, MergeAxisInterpolate, MergeOperation, MetricsExtractor,
    RangeMetrics, RiskMetrics, ShapeMetrics, SplineInterpolation, TrendMetrics,
};
use crate::surfaces::Point3D;
use crate::surfaces::types::Axis;
use crate::utils::Len;

use num_traits::ToPrimitive;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ops::Index;
use std::sync::Arc;
use crate::visualization::{Graph, GraphData, Series2D, Surface3D, TraceMode};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        sorted_points.sort_by(|a, b| x_selector(a).partial_cmp(&x_selector(b)).unwrap());

        // Ensure we have at least 2 points
        if sorted_points.len() < 2 {
            return Err(InterpolationError::Spline(
                "Insufficient points for interpolation".to_string(),
            ));
        }

        // Handle out-of-range cases
        if target <= x_selector(&sorted_points[0]) {
            return Ok(z_selector(&sorted_points[0]));
        }

        if target >= x_selector(&sorted_points[sorted_points.len() - 1]) {
            return Ok(z_selector(&sorted_points[sorted_points.len() - 1]));
        }

        // Find the segment where the target falls
        let (left_index, right_index) = match sorted_points
            .iter()
            .enumerate()
            .find(|(_, p)| x_selector(p) > target)
        {
            Some((index, _)) => (index - 1, index),
            None => (sorted_points.len() - 2, sorted_points.len() - 1),
        };

        // Get the points for interpolation
        let x0 = x_selector(&sorted_points[left_index]);
        let x1 = x_selector(&sorted_points[right_index]);
        let z0 = z_selector(&sorted_points[left_index]);
        let z1 = z_selector(&sorted_points[right_index]);

        // Linear interpolation
        let interpolated_z = z0 + (z1 - z0) * ((target - x0) / (x1 - x0));

        Ok(interpolated_z)
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
        GraphData::Surface(Surface3D {
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
    /// use std::collections::BTreeSet;
    /// use optionstratlib::geometrics::{ConstructionMethod, GeometricObject};
    /// use optionstratlib::surfaces::{Point3D, Surface};
    /// let points = BTreeSet::from_iter(vec![
    ///     Point3D::new(0, 0, 0),
    ///     Point3D::new(1, 0, 1),
    ///     Point3D::new(0, 1, 1),
    /// ]);
    /// let surface = Surface::construct(ConstructionMethod::FromData { points }).unwrap();
    /// ```
    ///
    /// ## Creating from a parametric function
    /// ```rust,no_run
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
    /// let surface = Surface::construct(ConstructionMethod::Parametric { f, params }).unwrap();
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
                let x_step = (x_end - x_start) / Decimal::from(x_steps);
                let y_step = (y_end - y_start) / Decimal::from(y_steps);

                // Wrap f in an Arc so it can be shared across threads
                let f = Arc::new(f);

                let points: Result<BTreeSet<Point3D>, SurfaceError> = (0..=x_steps)
                    .into_par_iter()
                    .flat_map(|i| {
                        let x = x_start + x_step * Decimal::from(i);
                        let f = Arc::clone(&f);
                        (0..=y_steps).into_par_iter().map(move |j| {
                            let y = y_start + y_step * Decimal::from(j);
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
/// This implementation will panic with the message "Index out of bounds" if the provided index
/// is greater than or equal to the number of points in the surface.
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
    fn index(&self, index: usize) -> &Self::Output {
        self.points.iter().nth(index).expect("Index out of bounds")
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

        let mut nearest_points: Vec<&Point3D> = self.points.iter().collect();
        nearest_points.sort_by(|a, b| {
            let dist_a = (a.x - xy.x).powi(2) + (a.y - xy.y).powi(2);
            let dist_b = (b.x - xy.x).powi(2) + (b.y - xy.y).powi(2);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        let p1 = nearest_points[0];
        let p2 = nearest_points[1];
        let p3 = nearest_points[2];

        let denominator = (p2.y - p3.y) * (p1.x - p3.x) + (p3.x - p2.x) * (p1.y - p3.y);
        let w1 = ((p2.y - p3.y) * (xy.x - p3.x) + (p3.x - p2.x) * (xy.y - p3.y)) / denominator;
        let w2 = ((p3.y - p1.y) * (xy.x - p3.x) + (p1.x - p3.x) * (xy.y - p3.y)) / denominator;
        let w3 = Decimal::ONE - w1 - w2;

        let z = w1 * p1.z + w2 * p2.z + w3 * p3.z;

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
        let mut sorted_points: Vec<&Point3D> = self.points.iter().collect();
        sorted_points.sort_by(|a, b| {
            let dist_a = (a.x - xy.x).powi(2) + (a.y - xy.y).powi(2);
            let dist_b = (b.x - xy.x).powi(2) + (b.y - xy.y).powi(2);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        let closest_points = &sorted_points[0..4];

        // Sort points to create a quadrilateral
        let mut quad_points: Vec<&Point3D> = closest_points.to_vec();
        quad_points.sort_by(|a, b| {
            let a_key = (a.y, a.x);
            let b_key = (b.y, b.x);
            a_key.partial_cmp(&b_key).unwrap()
        });

        // Get the four points for interpolation
        let q11 = quad_points[0]; // Bottom-left point
        let q12 = quad_points[1]; // Bottom-right point
        let q21 = quad_points[2]; // Top-left point
        let q22 = quad_points[3]; // Top-right point

        // Calculate normalized coordinates
        let x_ratio = (xy.x - q11.x) / (q12.x - q11.x);
        let y_ratio = (xy.y - q11.y) / (q21.y - q11.y);

        // Perform bilinear interpolation
        let z = (Decimal::ONE - x_ratio) * (Decimal::ONE - y_ratio) * q11.z
            + x_ratio * (Decimal::ONE - y_ratio) * q12.z
            + (Decimal::ONE - x_ratio) * y_ratio * q21.z
            + x_ratio * y_ratio * q22.z;

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
        let mut sorted_points: Vec<&Point3D> = self.points.iter().collect();
        sorted_points.sort_by(|a, b| {
            let dist_a = (a.x - xy.x).powi(2) + (a.y - xy.y).powi(2);
            let dist_b = (b.x - xy.x).powi(2) + (b.y - xy.y).powi(2);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        let closest_points = &sorted_points[0..9];

        // Cubic interpolation requires solving a system of equations
        // We'll use a weighted cubic interpolation approach

        // Calculate weights based on distance
        let weights: Vec<Decimal> = closest_points
            .iter()
            .map(|&point| {
                let dist = ((point.x - xy.x).powi(2) + (point.y - xy.y).powi(2))
                    .sqrt()
                    .unwrap();
                Decimal::ONE / (dist + Decimal::new(1, 6)) // Avoid division by zero
            })
            .collect();

        // Weighted cubic interpolation
        let mut numerator_z = Decimal::ZERO;
        let mut denominator = Decimal::ZERO;

        for (&point, &weight) in closest_points.iter().zip(weights.iter()) {
            // Cubic weight function
            let cubic_weight = weight.powi(3);
            numerator_z += point.z * cubic_weight;
            denominator += cubic_weight;
        }

        // Prevent division by zero
        let interpolated_z = if denominator != Decimal::ZERO {
            numerator_z / denominator
        } else {
            // Fallback to average if weights are problematic
            closest_points.iter().map(|p| p.z).sum::<Decimal>()
                / Decimal::from(closest_points.len())
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
            a_key.partial_cmp(&b_key).unwrap()
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

        let mean = z_values.iter().sum::<Decimal>() / Decimal::from(z_values.len());

        let mut sorted = z_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = sorted[sorted.len() / 2];

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

        let std_dev = (z_values
            .iter()
            .map(|x| (*x - mean).powu(2))
            .sum::<Decimal>()
            / Decimal::from(z_values.len()))
        .sqrt()
        .unwrap_or(Decimal::ZERO);

        Ok(BasicMetrics {
            mean,
            median,
            mode,
            std_dev,
        })
    }

    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError> {
        let z_values: Vec<Decimal> = self.points.iter().map(|p| p.z).collect();
        let mean = z_values.iter().sum::<Decimal>() / Decimal::from(z_values.len());
        let std_dev = (z_values
            .iter()
            .map(|x| (*x - mean).powu(2))
            .sum::<Decimal>()
            / Decimal::from(z_values.len()))
        .sqrt()
        .unwrap_or(Decimal::ONE);

        let n = Decimal::from(z_values.len());

        let skewness = z_values
            .iter()
            .map(|x| (*x - mean).powu(3))
            .sum::<Decimal>()
            / (n * std_dev.powu(3));

        let kurtosis = z_values
            .iter()
            .map(|x| (*x - mean).powu(4))
            .sum::<Decimal>()
            / (n * std_dev.powu(4));

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
        let mut sorted = z_values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted.first().copied().unwrap_or(Decimal::ZERO);
        let max = sorted.last().copied().unwrap_or(Decimal::ZERO);

        let len = sorted.len();
        let q1 = sorted[len / 4];
        let q2 = sorted[len / 2];
        let q3 = sorted[3 * len / 4];

        let range = max - min;
        let iqr = q3 - q1;

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

        let sum_x: Decimal = x_vals.iter().sum();
        let sum_z: Decimal = z_vals.iter().sum();

        // Check for identical points to avoid division by zero
        let is_identical_points = z_vals.iter().all(|&z| z == z_vals[0]);

        let (slope, intercept, r_squared) = if is_identical_points {
            // All points are the same
            (Decimal::ZERO, z_vals[0], Decimal::ONE)
        } else {
            let sum_xz: Decimal = x_vals.iter().zip(&z_vals).map(|(x, z)| *x * *z).sum();
            let sum_xx: Decimal = x_vals.iter().map(|x| *x * *x).sum();

            let slope = (n * sum_xz - sum_x * sum_z) / (n * sum_xx - sum_x * sum_x);
            let intercept = (sum_z - slope * sum_x) / n;

            // R-squared Calculation
            let mean_z = sum_z / n;
            let sst: Decimal = z_vals.iter().map(|z| (*z - mean_z).powu(2)).sum();

            let ssr: Decimal = z_vals
                .iter()
                .zip(&x_vals)
                .map(|(z, x)| {
                    let z_predicted = slope * *x + intercept;
                    (*z - z_predicted).powu(2)
                })
                .sum();

            let r_squared = if sst == Decimal::ZERO {
                Decimal::ONE
            } else {
                Decimal::ONE - (ssr / sst)
            };

            (slope, intercept, r_squared)
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
        let z_values: Vec<Decimal> = self.points.iter().map(|p| p.z).collect();

        let mean = z_values.iter().sum::<Decimal>() / Decimal::from(z_values.len());
        let volatility = (z_values
            .iter()
            .map(|x| (*x - mean).powu(2))
            .sum::<Decimal>()
            / Decimal::from(z_values.len()))
        .sqrt()
        .unwrap_or(Decimal::ZERO);

        // Value at Risk (95% confidence) using parametric method
        let z_score = dec!(1.645); // 95% confidence interval
        let var = mean - z_score * volatility;

        // Expected Shortfall (Conditional VaR) calculation
        let expected_shortfall = z_values.iter().filter(|&x| *x < var).sum::<Decimal>()
            / Decimal::from(z_values.iter().filter(|&x| *x < var).count() as u64);

        // Beta calculation with optional market volatility
        let beta = Decimal::ZERO; // TODO: Implement beta calculation

        // Sharpe Ratio (assuming risk-free rate of 0)
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

impl Arithmetic<Surface> for Surface {
    type Error = SurfaceError;

    fn merge(surfaces: &[&Surface], operation: MergeOperation) -> Result<Surface, Self::Error> {
        if surfaces.is_empty() {
            return Err(SurfaceError::invalid_parameters(
                "merge_surfaces",
                "No surfaces provided for merging",
            ));
        }

        if surfaces.len() == 1 {
            return Ok(surfaces[0].clone());
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
        let x_step = (max_x - min_x) / Decimal::from(steps);
        let y_step = (max_y - min_y) / Decimal::from(steps);

        let result_points: Result<Vec<Point3D>, SurfaceError> = (0..=steps)
            .into_par_iter()
            .flat_map(|i| {
                let x = min_x + x_step * Decimal::from(i);
                (0..=steps).into_par_iter().map(move |j| {
                    let y = min_y + y_step * Decimal::from(j);
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
                        MergeOperation::Add => z_values.par_iter().sum(),
                        MergeOperation::Subtract => {
                            let first = z_values.first().cloned().unwrap_or(Decimal::ZERO);
                            let remaining_sum: Decimal = z_values.iter().skip(1).sum();
                            first - remaining_sum
                        }
                        MergeOperation::Multiply => z_values.par_iter().product(),
                        MergeOperation::Divide => {
                            let first = z_values.first().cloned().unwrap_or(Decimal::ONE);
                            z_values
                                .par_iter()
                                .skip(1)
                                .fold(
                                    || first,
                                    |acc, &val| if val == Decimal::ZERO { acc } else { acc / val },
                                )
                                .reduce(|| first, |a, _b| a)
                        }
                        MergeOperation::Max => z_values
                            .par_iter()
                            .cloned()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(Decimal::ZERO),
                        MergeOperation::Min => z_values
                            .par_iter()
                            .cloned()
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
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
        self.points
            .iter()
            .min_by(|a, b| {
                let dist_a = ((a.x - x.x).powi(2) + (a.y - x.y).powi(2)).sqrt().unwrap();
                let dist_b = ((b.x - x.x).powi(2) + (b.y - x.y).powi(2)).sqrt().unwrap();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
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
                interpolated_self_points.insert(
                    *self
                        .points
                        .iter()
                        .find(|p| p.x == xy.x && p.y == xy.y)
                        .unwrap(),
                );
            } else {
                let interpolated_point = self.interpolate(*xy, interpolation)?;
                interpolated_self_points.insert(interpolated_point);
            }

            if other.contains_point(xy) {
                interpolated_other_points.insert(
                    *other
                        .points
                        .iter()
                        .find(|p| p.x == xy.x && p.y == xy.y)
                        .unwrap(),
                );
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

        let translated_points = self
            .points
            .iter()
            .map(|point| {
                Point3D::new(
                    point.x + *deltas[0],
                    point.y + *deltas[1],
                    point.z + *deltas[2],
                )
            })
            .collect();

        Ok(Surface::new(translated_points))
    }

    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if factors.len() != 3 {
            return Err(SurfaceError::invalid_parameters(
                "scale",
                "Expected 3 factors for 3D scaling",
            ));
        }

        let scaled_points = self
            .points
            .iter()
            .map(|point| {
                Point3D::new(
                    point.x * *factors[0],
                    point.y * *factors[1],
                    point.z * *factors[2],
                )
            })
            .collect();

        Ok(Surface::new(scaled_points))
    }

    fn intersect_with(&self, other: &Self) -> Result<Vec<Point3D>, Self::Error> {
        let mut intersections = Vec::new();
        let epsilon = Decimal::new(1, 6); // 0.000001 tolerance

        for p1 in self.points.iter() {
            for p2 in other.points.iter() {
                if (p1.x - p2.x).abs() < epsilon
                    && (p1.y - p2.y).abs() < epsilon
                    && (p1.z - p2.z).abs() < epsilon
                {
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

        // For surfaces with exactly 2 or 3 points, use a simple approach
        if self.points.len() <= 3 {
            // let points: Vec<_> = self.points.iter().collect();

            // Ensure points are not identical
            if self[0] == self[1] {
                return Err(SurfaceError::invalid_parameters(
                    "derivative_at",
                    "Points are identical, cannot calculate derivatives",
                ));
            }

            // Calculate derivatives using the first two points
            let dx = if (self[1].x - self[0].x) == Decimal::ZERO {
                Decimal::MAX
            } else {
                (self[1].z - self[0].z) / (self[1].x - self[0].x)
            };

            let dy = if (self[1].y - self[0].y) == Decimal::ZERO {
                Decimal::MAX
            } else {
                (self[1].z - self[0].z) / (self[1].y - self[0].y)
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

        let x_points: BTreeSet<Point3D> = self
            .get_points()
            .into_iter()
            .filter(|p| (p.x - point.x).abs() < tolerance)
            .cloned()
            .collect();

        let y_points: BTreeSet<Point3D> = self
            .get_points()
            .into_iter()
            .filter(|p| (p.y - point.y).abs() < tolerance)
            .cloned()
            .collect();

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
        x_sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

        let mut y_sorted: Vec<_> = y_candidates.iter().collect();
        y_sorted.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // Prevent division by zero
        let dx = if x_sorted[0].x == x_sorted[1].x {
            Decimal::ZERO
        } else {
            (x_sorted[1].z - x_sorted[0].z) / (x_sorted[1].x - x_sorted[0].x)
        };

        let dy = if y_sorted[0].y == y_sorted[1].y {
            Decimal::ZERO
        } else {
            (y_sorted[1].z - y_sorted[0].z) / (y_sorted[1].y - y_sorted[0].y)
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
            .min_by(|a, b| a.z.partial_cmp(&b.z).unwrap())
            .cloned()
            .unwrap();

        let max_point = self
            .points
            .iter()
            .max_by(|a, b| a.z.partial_cmp(&b.z).unwrap())
            .cloned()
            .unwrap();

        Ok((min_point, max_point))
    }

    fn measure_under(&self, base_value: &Decimal) -> Result<Decimal, Self::Error> {
        if self.points.len() < 3 {
            return Ok(Decimal::ZERO);
        }

        // Approximate volume using triangular prisms
        let mut volume = Decimal::ZERO;
        let points: Vec<_> = self.points.iter().collect();

        // For each possible triangle in the surface
        for window in points.windows(3) {
            // Calculate area of triangle
            let p1 = window[0];
            let p2 = window[1];
            let p3 = window[2];

            let area =
                ((p2.x - p1.x) * (p3.y - p1.y) - (p3.x - p1.x) * (p2.y - p1.y)).abs() / dec!(2);

            // Average height from base_value
            let avg_height =
                ((p1.z - *base_value) + (p2.z - *base_value) + (p3.z - *base_value)) / dec!(3);

            volume += area * avg_height;
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
    }

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
                "Failed for target {}, expected {}, got {}",
                target,
                expected,
                result
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
            assert_eq!(
                result, expected,
                "Failed for out-of-range target {}",
                target
            );
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
                    Err(Box::from("Test error".to_string()))
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
        // Test interpolación en el borde
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

        // La interpolación en cualquier punto debe mantener el gradiente
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
        // Verificar que el resultado tiene la precisión esperada
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

    #[test]
    fn test_invalid_quadrilateral() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(0.0), dec!(0.0), dec!(1.0)),
            Point3D::new(dec!(0.0), dec!(0.0), dec!(2.0)),
            Point3D::new(dec!(0.0), dec!(0.0), dec!(3.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.0)));
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(InterpolationError::Bilinear(msg)) if msg.contains("Need at least four points for bilinear interpolation")
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
                "Failed for point {:?}",
                point
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
                "Failed for boundary point {:?}",
                point
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
                "Failed for extreme point {:?}",
                point
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
                "Failed for point {:?}",
                point
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
                "Failed for boundary point {:?}",
                point
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
                "Failed for extreme point {:?}",
                point
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
                "Failed for target {}, expected {}, got {}",
                target,
                expected,
                result
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
            assert!(result.is_ok(), "Failed for point {:?}", point);

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

    #[test]
    fn test_merge_indexes() {
        let surface1 = create_test_surface();
        let surface2 = create_test_surface();
        let merged = surface1.merge_indexes(surface2.get_index_values());

        assert_eq!(merged.len(), 2);
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
