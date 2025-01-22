/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use std::collections::BTreeSet;
use std::ops::Index;
use std::sync::Arc;
use rayon::iter::IntoParallelIterator;
use rust_decimal::{Decimal, MathematicalOps};
use crate::curves::{Curve, Point2D};
use crate::error::{InterpolationError, SurfaceError};
use crate::geometrics::{BiLinearInterpolation, ConstructionMethod, ConstructionParams, CubicInterpolation, GeometricObject, Interpolate, LinearInterpolation, SplineInterpolation};
use crate::surfaces::Point3D;
use crate::surfaces::types::Axis;
use rayon::iter::ParallelIterator;
use rust_decimal_macros::dec;

/// Represents a mathematical surface in 3D space
#[derive(Debug, Clone)]
pub struct Surface {
    /// Collection of 3D points defining the surface
    pub points: BTreeSet<Point3D>,
    pub x_range: (Decimal, Decimal),
    pub y_range: (Decimal, Decimal),

}

impl Surface {
    
    pub fn new(points: BTreeSet<Point3D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        Self {
            points,
            x_range,
            y_range,
        }
    }
    
    pub fn get_curve(&self, axis: Axis) -> Curve {
        let points = self.points.iter().map(|p| {
            match axis {
                Axis::X => Point2D::new(p.y, p.z),
                Axis::Y => Point2D::new(p.x, p.z),
                Axis::Z => Point2D::new(p.x, p.y),
            }
        }).collect();
        
        Curve::new(points)
    }

}

impl GeometricObject<Point3D, Point2D> for Surface {
    type Error = SurfaceError;

    fn get_points(&self) -> BTreeSet<&Point3D> {
        self.points.iter().collect()
    }

    fn from_vector<T>(points: Vec<T>) -> Self
    where
        T: Into<Point3D> + Clone,
    {
        let points: BTreeSet<Point3D> = points.into_iter().map(|p| p.into()).collect();

        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        
        Surface { points, x_range , y_range}
    }



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
                        ))
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

impl Index<usize> for Surface {
    type Output = Point3D;

    fn index(&self, index: usize) -> &Self::Output {
        self.points.iter().nth(index).expect("Index out of bounds")
    }
}

impl Interpolate<Point3D, Point2D> for Surface { }

impl LinearInterpolation<Point3D, Point2D> for Surface {
    fn linear_interpolate(&self, xy: Point2D) -> Result<Point3D, InterpolationError> {
        // First check that we have enough points
        if self.points.len() < 3 {
            return Err(InterpolationError::Linear(
                "Need at least three points for linear interpolation".to_string(),
            ));
        }

        // Check that the point is within the surface's range
        if xy.x < self.x_range.0
            || xy.x > self.x_range.1
            || xy.y < self.y_range.0
            || xy.y > self.y_range.1 {
            return Err(InterpolationError::Linear(
                "Point is outside the surface's range".to_string(),
            ));
        }

        // Get all points sorted by distance
        let mut nearest_points: Vec<&Point3D> = self.points
            .iter()
            .collect();

        nearest_points.sort_by(|a, b| {
            let dist_a = (a.x - xy.x).powi(2) + (a.y - xy.y).powi(2);
            let dist_b = (b.x - xy.x).powi(2) + (b.y - xy.y).powi(2);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        // Need at least three points for interpolation
        if nearest_points.len() < 3 {
            return Err(InterpolationError::Linear(
                "Could not find three suitable points for interpolation".to_string(),
            ));
        }

        let p1 = nearest_points[0];
        let p2 = nearest_points[1];
        let p3 = nearest_points[2];

        // Primero verificamos si los puntos son colineales
        let v1x = p2.x - p1.x;
        let v1y = p2.y - p1.y;
        let v2x = p3.x - p1.x;
        let v2y = p3.y - p1.y;

        // Calculamos el producto cruz para ver si los vectores son paralelos
        let cross_product = v1x * v2y - v1y * v2x;

        if cross_product.abs() < Decimal::new(1, 3) {
            return Err(InterpolationError::Linear(
                "Degenerate triangle detected".to_string(),
            ));
        }

        // Ahora podemos revisar si hay coincidencia exacta
        if let Some(point) = self.points.iter().find(|p| p.x == xy.x && p.y == xy.y) {
            return Ok(*point);
        }

        // Calculate barycentric coordinates
        let denominator = (p2.y - p3.y) * (p1.x - p3.x) + (p3.x - p2.x) * (p1.y - p3.y);
        let w1 = ((p2.y - p3.y) * (xy.x - p3.x) + (p3.x - p2.x) * (xy.y - p3.y)) / denominator;
        let w2 = ((p3.y - p1.y) * (xy.x - p3.x) + (p1.x - p3.x) * (xy.y - p3.y)) / denominator;
        let w3 = Decimal::ONE - w1 - w2;

        // Calculate interpolated z value
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
            || xy.y > self.y_range.1 {
            return Err(InterpolationError::Bilinear(
                "Point is outside the surface's range".to_string(),
            ));
        }

        // Check for invalid quadrilateral: all points have the same x and y but different z
        let xy_points: Vec<&Point3D> = self.points.iter()
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
        let z = (Decimal::ONE - x_ratio) * (Decimal::ONE - y_ratio) * q11.z +
            x_ratio * (Decimal::ONE - y_ratio) * q12.z +
            (Decimal::ONE - x_ratio) * y_ratio * q21.z +
            x_ratio * y_ratio * q22.z;

        Ok(Point3D::new(xy.x, xy.y, z))
    }
}

impl CubicInterpolation<Point3D, Point2D> for Surface {
    fn cubic_interpolate(&self, x: Point2D) -> Result<Point3D, InterpolationError> {
        todo!()
    }
}

impl SplineInterpolation<Point3D, Point2D> for Surface {
    fn spline_interpolate(&self, x: Point2D) -> Result<Point3D, InterpolationError> {
        todo!()
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
    fn test_insufficient_points() {
        let points = BTreeSet::from_iter(vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
            Point3D::new(dec!(1.0), dec!(1.0), dec!(1.0)),
        ]);
        let surface = Surface::new(points);
        let result = surface.linear_interpolate(Point2D::new(dec!(0.5), dec!(0.5)));
        assert!(matches!(
            result,
            Err(InterpolationError::Linear(msg)) if msg.contains("Need at least three points")
        ));
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
        let result = surface.linear_interpolate(Point2D::new(dec!(0.0), dec!(0.0))).unwrap();
        assert_eq!(result.z, dec!(0.0));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface.linear_interpolate(Point2D::new(dec!(0.5), dec!(0.5))).unwrap();
        assert_eq!(result.z, dec!(1.0));
    }

    #[test]
    fn test_quarter_point_interpolation() {
        let surface = create_test_surface();
        let result = surface.linear_interpolate(Point2D::new(dec!(0.25), dec!(0.25))).unwrap();
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
        // Probamos con un punto que está en la misma línea que los tres puntos
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
        let result = surface.linear_interpolate(Point2D::new(dec!(0.0), dec!(0.5))).unwrap();
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
        let result = surface.linear_interpolate(Point2D::new(dec!(0.5), dec!(0.5))).unwrap();
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
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),  // Bottom-left
            Point3D::new(dec!(1.0), dec!(0.0), dec!(1.0)),  // Bottom-right
            Point3D::new(dec!(0.0), dec!(1.0), dec!(1.0)),  // Top-left
            Point3D::new(dec!(1.0), dec!(1.0), dec!(2.0)),  // Top-right
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
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.0))).unwrap();
        assert_eq!(result.z, dec!(0.0));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.5), dec!(0.5))).unwrap();
        // At the midpoint, we expect the average of surrounding values
        assert_eq!(result.z, dec!(1.0));
    }

    #[test]
    fn test_quarter_point_interpolation() {
        let surface = create_test_surface();
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.25), dec!(0.25))).unwrap();
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
        assert!(matches!(
            result,
            Err(InterpolationError::Bilinear(msg)) if msg.contains("Invalid quadrilateral")
        ));
    }

    #[test]
    fn test_boundary_interpolation() {
        let surface = create_test_surface();
        // Test interpolation at edge
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.5))).unwrap();
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
        let result = surface.bilinear_interpolate(Point2D::new(dec!(0.5), dec!(0.5))).unwrap();
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
        let bl = surface.bilinear_interpolate(Point2D::new(dec!(0.0), dec!(0.0))).unwrap();
        let br = surface.bilinear_interpolate(Point2D::new(dec!(1.0), dec!(0.0))).unwrap();
        let tl = surface.bilinear_interpolate(Point2D::new(dec!(0.0), dec!(1.0))).unwrap();
        let tr = surface.bilinear_interpolate(Point2D::new(dec!(1.0), dec!(1.0))).unwrap();

        assert_eq!(bl.z, dec!(0.0));
        assert_eq!(br.z, dec!(1.0));
        assert_eq!(tl.z, dec!(1.0));
        assert_eq!(tr.z, dec!(2.0));
    }
}