/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/1/25
******************************************************************************/
use crate::curves::Point2D;
use crate::error::SurfaceError;
use crate::geometrics::HasX;
use num_traits::FromPrimitive;
use positive::is_positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Display;
use utoipa::ToSchema;

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
///
/// # Examples
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::surfaces::Point3D;
///
/// // Create a new 3D point
/// let point = Point3D {
///     x: dec!(1.5),
///     y: dec!(2.0),
///     z: dec!(-3.25),
/// };
/// ```
///
/// # Usage
/// `Point3D` is primarily used within the surface module to represent vertices
/// of 3D surfaces and for various geometric calculations. The high-precision `Decimal`
/// type ensures accuracy in scientific and engineering applications.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct Point3D {
    /// The x-coordinate in the Cartesian system
    pub x: Decimal,
    /// The y-coordinate in the Cartesian system
    pub y: Decimal,
    /// The z-coordinate in the Cartesian system
    pub z: Decimal,
}

impl Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {}, z: {})", self.x, self.y, self.z)
    }
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
        T: TryFrom<Decimal> + 'static,
        U: TryFrom<Decimal> + 'static,
        V: TryFrom<Decimal> + 'static,
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

        let x = T::try_from(self.x).map_err(|_| SurfaceError::Point3DError {
            reason: "failed to convert x",
        })?;
        let y = U::try_from(self.y).map_err(|_| SurfaceError::Point3DError {
            reason: "failed to convert y",
        })?;
        let z = V::try_from(self.z).map_err(|_| SurfaceError::Point3DError {
            reason: "failed to convert z",
        })?;

        Ok((x, y, z))
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

    /// Converts this `Point3D` instance to a `Point2D` by projecting onto the XY plane.
    ///
    /// # Overview
    /// This method creates a new `Point2D` instance using only the `x` and `y` coordinates
    /// of the current `Point3D` object, effectively projecting the 3D point onto the XY plane.
    /// The `z` coordinate is discarded in this operation.
    ///
    /// # Returns
    /// A heap-allocated `Point2D` instance (`Box<Point2D>`) containing the `x` and `y` coordinates
    /// from this `Point3D` object.
    ///
    /// # Example
    /// ```
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::surfaces::Point3D;
    ///
    /// let point3d = Point3D { x: dec!(1.5), y: dec!(2.0), z: dec!(3.5) };
    /// let point2d = point3d.point2d();
    /// assert_eq!(point2d.x, dec!(1.5));
    /// assert_eq!(point2d.y, dec!(2.0));
    /// ```
    ///
    /// # Use Cases
    /// This method is useful when:
    /// - Working with visualization that requires 2D projections of 3D points
    /// - Analyzing specific planes within a 3D model
    /// - Converting between coordinate systems from 3D to 2D
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

    use crate::surfaces::Surface;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    #[test]
    fn test_point3d_new() {
        let point = Point3D::new(dec!(1.0), dec!(2.0), dec!(3.0));
        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
        assert_eq!(point.z, dec!(3.0));

        let display = format!("{point}");
        assert_eq!(display, "(x: 1.0, y: 2.0, z: 3.0)");

        let point2d = point.point2d();
        assert_eq!(point2d.x, dec!(1.0));
        assert_eq!(point2d.y, dec!(2.0));

        let has_x = point.get_x();
        assert_eq!(has_x, dec!(1.0));
    }

    #[test]
    fn test_point3d_negative() {
        let point = Point3D::new(dec!(-1.0), dec!(-2.0), dec!(-3.0));
        assert_eq!(point.x, dec!(-1.0));
        assert_eq!(point.y, dec!(-2.0));
        assert_eq!(point.z, dec!(-3.0));

        let display = format!("{point}");
        assert_eq!(display, "(x: -1.0, y: -2.0, z: -3.0)");

        let tuple = point.to_tuple::<Positive, Decimal, Decimal>();
        assert!(tuple.is_err());
        let tuple = point.to_tuple::<Decimal, Positive, Decimal>();
        assert!(tuple.is_err());
        let tuple = point.to_tuple::<Decimal, Decimal, Positive>();
        assert!(tuple.is_err());
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
        let result = Point3D::from_tuple(dec!(1.0), Positive::TWO, pos_or_panic!(3.0));
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

#[cfg(test)]
mod tests_point3d_serde {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::Value;

    #[test]
    fn test_basic_serialization() {
        let point = Point3D {
            x: dec!(1.5),
            y: dec!(2.5),
            z: dec!(3.5),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point.x, deserialized.x);
        assert_eq!(point.y, deserialized.y);
        assert_eq!(point.z, deserialized.z);
    }

    #[test]
    fn test_zero_values() {
        let point = Point3D {
            x: dec!(0.0),
            y: dec!(0.0),
            z: dec!(0.0),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_negative_values() {
        let point = Point3D {
            x: dec!(-1.5),
            y: dec!(-2.5),
            z: dec!(-3.5),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_high_precision_values() {
        let point = Point3D {
            x: dec!(1.12345678901234567890),
            y: dec!(2.12345678901234567890),
            z: dec!(3.12345678901234567890),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point.x, deserialized.x);
        assert_eq!(point.y, deserialized.y);
        assert_eq!(point.z, deserialized.z);
    }

    #[test]
    fn test_json_structure() {
        let point = Point3D {
            x: dec!(1.5),
            y: dec!(2.5),
            z: dec!(3.5),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let json_value: Value = serde_json::from_str(&serialized).unwrap();

        // Verify JSON structure
        assert!(json_value.is_object());
        assert_eq!(json_value.as_object().unwrap().len(), 3);
        assert!(json_value.get("x").is_some());
        assert!(json_value.get("y").is_some());
        assert!(json_value.get("z").is_some());
    }

    #[test]
    fn test_pretty_print() {
        let point = Point3D {
            x: dec!(1.5),
            y: dec!(2.5),
            z: dec!(3.5),
        };

        let serialized = serde_json::to_string_pretty(&point).unwrap();

        // Verify pretty print format
        assert!(serialized.contains('\n'));
        assert!(serialized.contains("  "));

        // Verify we can still deserialize pretty-printed JSON
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_deserialize_from_integers() {
        let json_str = r#"{"x": 1, "y": 2, "z": 3}"#;
        let point: Point3D = serde_json::from_str(json_str).unwrap();

        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
        assert_eq!(point.z, dec!(3.0));
    }

    #[test]
    fn test_deserialize_from_strings() {
        let json_str = r#"{"x": "1.5", "y": "2.5", "z": "3.5"}"#;
        let point: Point3D = serde_json::from_str(json_str).unwrap();

        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
        assert_eq!(point.z, dec!(3.5));
    }

    #[test]
    fn test_invalid_json() {
        // Missing field
        let json_str = r#"{"x": 1.5, "y": 2.5}"#;
        let result = serde_json::from_str::<Point3D>(json_str);
        assert!(result.is_err());

        // Invalid number format
        let json_str = r#"{"x": "invalid", "y": 2.5, "z": 3.5}"#;
        let result = serde_json::from_str::<Point3D>(json_str);
        assert!(result.is_err());

        // Wrong data type
        let json_str = r#"{"x": true, "y": 2.5, "z": 3.5}"#;
        let result = serde_json::from_str::<Point3D>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_values() {
        let point = Point3D {
            x: Decimal::MAX,
            y: Decimal::MAX,
            z: Decimal::MAX,
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_min_values() {
        let point = Point3D {
            x: Decimal::MIN,
            y: Decimal::MIN,
            z: Decimal::MIN,
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point3D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_json_to_vec() {
        let points = vec![
            Point3D {
                x: dec!(1.0),
                y: dec!(2.0),
                z: dec!(3.0),
            },
            Point3D {
                x: dec!(4.0),
                y: dec!(5.0),
                z: dec!(6.0),
            },
        ];

        let serialized = serde_json::to_string(&points).unwrap();
        let deserialized: Vec<Point3D> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(points, deserialized);
    }

    #[test]
    fn test_array() {
        let json_str = "[1.5,  2.5, 3.5]";
        let result = serde_json::from_str::<Point3D>(json_str);
        assert!(result.is_ok());
    }
}
