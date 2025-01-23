/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/1/25
******************************************************************************/
use crate::curves::Point2D;
use crate::error::SurfaceError;
use crate::geometrics::HasX;
use crate::model::positive::is_positive;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::cmp::Ordering;
use std::collections::BTreeSet;

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
#[derive(Debug, Clone, Copy, Hash)]
pub struct Point3D {
    pub x: Decimal,
    pub y: Decimal,
    pub z: Decimal,
}

impl PartialEq for Point3D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Point3D {}

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
    pub fn to_tuple<
        T: From<Decimal> + 'static,
        U: From<Decimal> + 'static,
        V: From<Decimal> + 'static,
    >(
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

    pub fn point2d(&self) -> Box<Point2D> {
        Box::new(Point2D::new(self.x, self.y))
    }
}

impl From<&Point3D> for Point3D {
    fn from(point: &Point3D) -> Self {
        *point
    }
}

impl HasX for Point3D {
    fn get_x(&self) -> Decimal {
        self.x
    }
}

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

/// Represents the three possible axes in a 3D space.
///
/// This enumeration is commonly used to define or manipulate directions
/// or dimensions within a 3D coordinate system.
///
/// ## Variants
/// - `X`: The axis representing the horizontal direction.
/// - `Y`: The axis representing the vertical direction or elevation.
/// - `Z`: The axis representing depth or the third dimension.
pub enum Axis {
    X,
    Y,
    Z,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;
    use crate::surfaces::Surface;
    use rust_decimal_macros::dec;

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
        let points = BTreeSet::from_iter(vec![
            Point3D::from_f64_tuple(0.0, 0.0, 0.0).unwrap(),
            Point3D::from_f64_tuple(1.0, 1.0, 1.0).unwrap(),
        ]);
        let surface = Surface::new(points.clone());

        assert_eq!(surface.points, points);
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

    #[test]
    fn test_equal() {
        let p1 = Point3D::from_f64_tuple(1.0, 2.0, 3.0).unwrap();
        let p2 = Point3D::from_f64_tuple(1.0, 2.0, 3.0).unwrap();
        let p3 = Point3D::from_f64_tuple(1.0, 2.0, 4.0).unwrap();
        let p4 = Point3D::from_f64_tuple(1.0, 3.0, 3.0).unwrap();
        let p5 = Point3D::from_f64_tuple(2.0, 2.0, 3.0).unwrap();
        assert_eq!(p1, p2);
        assert_eq!(p1, p3);
        assert_ne!(p1, p4);
        assert_ne!(p1, p5);
    }
}
