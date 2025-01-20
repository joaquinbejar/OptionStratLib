/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/1/25
******************************************************************************/
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use rayon::iter::IntoParallelIterator;
use crate::curves::{Curve, Point2D};
use crate::error::{CurvesError, SurfaceError};
use crate::model::positive::is_positive;
use crate::surfaces::construction::SurfaceConstructionMethod;

/// Represents a point in three-dimensional space with `x`, `y` and `z` coordinates.
///
/// # Overview
/// The `Point3D` struct defines a point in a 3D Cartesian coordinate system.
/// All coordinates (`x`, `y`, and `z`) are stored as `Decimal` values to provide high precision,
/// making it suitable for applications requiring accurate numerical calculations.
///
/// # Fields 
/// - **x**: The x-coordinate of the point, represented as a `Decimal`
/// - **y**: The y-coordinate of the point, represented as a `Decimal`
/// - **z**: The z-coordinate of the point, represented as a `Decimal`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point3D {
    pub x: Decimal,
    pub y: Decimal,
    pub z: Decimal,
}

impl PartialOrd for Point3D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point3D {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x.cmp(&other.x) {
            Ordering::Equal => match self.y.cmp(&other.y) {
                Ordering::Equal => self.z.cmp(&other.z),
                y_ordering => y_ordering,
            },
            x_ordering => x_ordering,
        }
    }
}

impl Point3D {
    /// Creates a new instance of `Point3D` using the specified coordinates.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate, which implements `Into<Decimal>`
    /// - `y`: The y-coordinate, which implements `Into<Decimal>`
    /// - `z`: The z-coordinate, which implements `Into<Decimal>`
    pub fn new<T: Into<Decimal>, U: Into<Decimal>, V: Into<Decimal>>(x: T, y: U, z: V) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    /// Converts the Point3D to a tuple of three values.
    ///
    /// # Type Parameters
    /// - `T`: Type for x-coordinate
    /// - `U`: Type for y-coordinate
    /// - `V`: Type for z-coordinate
    pub fn to_tuple<T: From<Decimal> + 'static, U: From<Decimal> + 'static, V: From<Decimal> + 'static>(
        &self,
    ) -> Result<(T, U, V), SurfaceError> {
        if is_positive::<T>() && self.x <= Decimal::ZERO {
            return Err(SurfaceError::Point3DError {
                reason: "x must be positive for type T",
            });
        }

        if is_positive::<U>() && self.y <= Decimal::ZERO {
            return Err(SurfaceError::Point3DError {
                reason: "y must be positive for type U",
            });
        }

        if is_positive::<V>() && self.z <= Decimal::ZERO {
            return Err(SurfaceError::Point3DError {
                reason: "z must be positive for type V",
            });
        }

        Ok((T::from(self.x), U::from(self.y), V::from(self.z)))
    }

    /// Creates a Point3D from a tuple of three values.
    pub fn from_tuple<T: Into<Decimal>, U: Into<Decimal>, V: Into<Decimal>>(
        x: T,
        y: U,
        z: V,
    ) -> Result<Self, SurfaceError> {
        Ok(Self::new(x, y, z))
    }

    /// Converts the Point3D to a tuple of f64 values.
    pub fn to_f64_tuple(&self) -> Result<(f64, f64, f64), SurfaceError> {
        let x = self.x.to_f64();
        let y = self.y.to_f64();
        let z = self.z.to_f64();

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Ok((x, y, z)),
            _ => Err(SurfaceError::Point3DError {
                reason: "Error converting Decimal to f64",
            }),
        }
    }

    /// Creates a Point3D from a tuple of f64 values.
    pub fn from_f64_tuple(x: f64, y: f64, z: f64) -> Result<Self, SurfaceError> {
        let x = Decimal::from_f64(x);
        let y = Decimal::from_f64(y);
        let z = Decimal::from_f64(z);

        match (x, y, z) {
            (Some(x), Some(y), Some(z)) => Ok(Self::new(x, y, z)),
            _ => Err(SurfaceError::Point3DError {
                reason: "Error converting f64 to Decimal",
            }),
        }
    }
}

/// Defines different types of surfaces that can be analyzed or constructed.
#[derive(Debug, Clone)]
pub enum SurfaceType {
    /// Surface representing volatility in three dimensions
    Volatility,
    /// Surface showing delta sensitivity
    Delta,
    /// Surface showing gamma sensitivity
    Gamma,
    /// Surface showing theta decay
    Theta,
    /// Surface showing vega sensitivity
    Vega,
    /// Custom surface type with description
    Other(String),
}

/// Represents an interpolation method for surfaces
#[derive(Debug, Clone)]
pub enum SurfaceInterpolationType {
    /// Linear interpolation between points
    Linear,
    /// Bilinear interpolation
    Bilinear,
    /// Bicubic interpolation
    Bicubic,
    /// Thin plate spline interpolation
    ThinPlateSpline,
    /// Custom interpolation method
    Custom(String),
}

/// Defines methods for constructing surfaces


/// Configuration for constructing and analyzing surfaces
#[derive(Debug, Clone)]
pub struct SurfaceConfig {
    /// Type of surface
    pub surface_type: SurfaceType,
    /// Interpolation method
    pub interpolation: SurfaceInterpolationType,
    /// Construction method
    pub construction_method: SurfaceConstructionMethod,
    /// Additional parameters
    pub extra_params: HashMap<String, Decimal>,
}

/// Represents a mathematical surface in 3D space
#[derive(Debug, Clone)]
pub struct Surface {
    /// Collection of 3D points defining the surface
    pub points: BTreeSet<Point3D>,
    pub x_range: (Decimal, Decimal),
    pub y_range: (Decimal, Decimal),

}

impl Surface {
    /// Creates a new surface with the given points and type
    pub fn new(points: BTreeSet<Point3D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        Self {
            points,
            x_range,
            y_range,
        }
    }

    pub fn from_vector(points: Vec<Point3D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        let points = points.into_iter().collect();
        Surface { points, x_range , y_range }
    }

    /// Calculates the range of x values in the curve.
    ///
    /// This function computes the minimum and maximum x values from an iterator of `Decimal`
    /// inputs, representing x-coordinates of points. It returns a tuple containing the
    /// minimum and maximum x values. The computation is efficient and involves a single
    /// traversal of the iterator.
    ///
    /// # Parameters
    ///
    /// - `iter` (`Iterator<Item = Decimal>`): An iterator over x-coordinates of points.
    ///
    /// # Returns
    ///
    /// - `(Decimal, Decimal)`: A tuple where:
    ///   - The first value is the minimum x-coordinate.
    ///   - The second value is the maximum x-coordinate.
    ///
    /// # Behavior
    ///
    /// - Iterates over the input to compute the x-range in a fold operation.
    /// - Returns `(Decimal::MAX, Decimal::MIN)` for an empty iterator (although such
    ///   cases are expected to be handled elsewhere).
    pub fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
    where
        I: Iterator<Item = Decimal>,
    {
        iter.fold((Decimal::MAX, Decimal::MIN), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }

    /// Constructs a curve using the specified construction method and returns the result.
    ///
    /// This function supports two distinct curve construction strategies:
    /// - **FromData**: Directly constructs a curve using pre-defined 2D points.
    /// - **Parametric**: Algorithmically builds a curve based on a parameterized
    ///   function over a given range and number of steps.
    ///
    /// # Parameters
    ///
    /// - `method` (`CurveConstructionMethod`): Specifies the strategy for constructing the curve.
    ///   Options include `FromData` (explicit points) or `Parametric` (function-based).
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The successfully constructed curve.
    /// - `Err(CurvesError)`: Indicates errors during construction.
    ///
    /// # Behavior
    ///
    /// ## FromData
    ///
    /// - Validates that the input points vector is not empty.
    /// - Returns an error (`CurvesError::Point2DError`) if the points vector is empty.
    /// - Constructs the curve using the provided points.
    ///
    /// ## Parametric
    ///
    /// - Divides the range `[t_start, t_end]` into `steps` intervals.
    /// - Computes points by evaluating a parameterized function `f` at each step using parallel
    ///   processing (`rayon`).
    /// - Fails gracefully with a `CurvesError` if the function `f` encounters issues.
    ///
    /// # Errors
    ///
    /// - **FromData**:
    ///   - Returns an error if an empty set of points is provided.
    /// - **Parametric**:
    ///   - Generates an error if the function `f` produces invalid results.
    ///
    /// # Details
    ///
    /// - Efficiently computes points in the parametric mode using parallel processing
    ///   provided by the `rayon` crate.
    ///
    /// # See Also
    ///
    /// - [`CurveConstructionMethod`]: Enum defining the supported construction strategies.
    /// - [`CurvesError`]: Represents possible errors encountered during curve construction.
    /// - [`Point2D`]: The data type representing a 2D point.
    pub fn construct(method: SurfaceConstructionMethod) -> Result<Self, SurfaceError> {
        match method {
            SurfaceConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(SurfaceError::Point3DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Surface::new(points))
            }

            SurfaceConstructionMethod::Parametric {
                f,
                x_start,
                x_end,
                y_start,
                y_end,
                x_steps,
                y_steps,
            } => {
                let step_size = (x_end - x_start) / Decimal::from(x_steps);
                let points: Result<BTreeSet<Point3D>, SurfaceError> = (0..=x_steps)
                    .into_par_iter()
                    .map(|i| {
                        let t = x_start + step_size * Decimal::from(i);
                        f(t).map_err(|e| CurvesError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(Surface::new)
            }
        }
    }

    pub fn vector(&self) -> Vec<&Point3D> {
        self.points.iter().collect()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::pos;

    #[test]
    fn test_point3d_new() {
        let point = Point3D::new(dec!(1.0), dec!(2.0), dec!(3.0));
        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
        assert_eq!(point.z, dec!(3.0));
    }

    #[test]
    fn test_point3d_to_tuple() {
        let result = Point3D::from_f64_tuple(1.0, 2.0, 3.0);
        assert!(result.is_ok());
        let point = result.unwrap();
        let result: Result<(Decimal, Decimal, Decimal), _> = point.to_tuple();
        assert!(result.is_ok());
        let (x, y, z) = result.unwrap();
        assert_eq!(x, dec!(1.0));
        assert_eq!(y, dec!(2.0));
        assert_eq!(z, dec!(3.0));
    }

    #[test]
    fn test_point3d_from_tuple() {
        let result = Point3D::from_tuple(dec!(1.0), pos!(2.0), pos!(3.0));
        assert!(result.is_ok());
        let point = result.unwrap();
        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
        assert_eq!(point.z, dec!(3.0));
    }

    #[test]
    fn test_point3d_to_f64_tuple() {
        let result = Point3D::from_f64_tuple(1.0, 2.0, 3.0);
        assert!(result.is_ok());
        let point = result.unwrap();
        let result = point.to_f64_tuple();
        assert!(result.is_ok());
        let (x, y, z) = result.unwrap();
        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 3.0);
    }

    #[test]
    fn test_point3d_from_f64_tuple() {
        let result = Point3D::from_f64_tuple(1.0, 2.0, 3.0);
        assert!(result.is_ok());
        let point = result.unwrap();
        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
        assert_eq!(point.z, dec!(3.0));
    }

    #[test]
    fn test_point3d_ordering() {
        let p1 = Point3D::from_f64_tuple(1.0, 2.0, 3.0).unwrap();
        let p2 = Point3D::from_f64_tuple(1.0, 2.0, 4.0).unwrap();
        let p3 = Point3D::from_f64_tuple(1.0, 3.0, 1.0).unwrap();
        let p4 = Point3D::from_f64_tuple(2.0, 1.0, 1.0).unwrap();

        assert!(p1 < p2); // Same x,y, different z
        assert!(p1 < p3); // Same x, different y
        assert!(p1 < p4); // Different x
    }

    #[test]
    fn test_surface_new() {
        let points = BTreeSet::from_iter( vec![
            Point3D::from_f64_tuple(0.0, 0.0, 0.0).unwrap(),
            Point3D::from_f64_tuple(1.0, 1.0, 1.0).unwrap(),
        ]);
        let surface = Surface::new(points.clone(), SurfaceType::Volatility);

        assert_eq!(surface.points, points);
        assert!(matches!(surface.surface_type, SurfaceType::Volatility));
        assert!(surface.config.is_none());
    }

    #[test]
    fn test_surface_with_config() {
        let points = BTreeSet::from_iter(vec![
            Point3D::from_f64_tuple(0.0, 0.0, 0.0).unwrap(),
            Point3D::from_f64_tuple(1.0, 1.0, 1.0).unwrap(),
        ]);

        let config = SurfaceConfig {
            surface_type: SurfaceType::Volatility,
            interpolation: SurfaceInterpolationType::Linear,
            construction_method: SurfaceConstructionMethod::FromData,
            extra_params: HashMap::new(),
        };

        let surface = Surface::with_config(
            points.clone(),
            SurfaceType::Volatility,
            config
        );

        assert_eq!(surface.points, points);
        assert!(matches!(surface.surface_type, SurfaceType::Volatility));
        assert!(surface.config.is_some());
    }

    #[test]
    fn test_error_handling_invalid_f64() {
        let result = Point3D::from_f64_tuple(f64::INFINITY, 2.0, 3.0);
        assert!(result.is_err());
        match result {
            Err(SurfaceError::Point3DError { reason }) => {
                assert_eq!(reason, "Error converting f64 to Decimal");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}