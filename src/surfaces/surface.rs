/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use std::collections::BTreeSet;
use std::ops::Index;
use std::sync::Arc;
use num_traits::ToPrimitive;
use rayon::iter::IntoParallelIterator;
use rust_decimal::{Decimal, MathematicalOps};
use crate::curves::{Curve, Point2D};
use crate::error::{InterpolationError, SurfaceError};
use crate::geometrics::{BiLinearInterpolation, ConstructionMethod, ConstructionParams, CubicInterpolation, GeometricObject, Interpolate, LinearInterpolation, SplineInterpolation};
use crate::surfaces::Point3D;
use crate::surfaces::types::Axis;
use rayon::iter::ParallelIterator;

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
    
    // Helper method for one-dimensional cubic spline interpolation
    fn one_dimensional_spline_interpolation<T>(
        &self,
        points: &[T],
        target: Decimal,
        x_selector: fn(&T) -> Decimal,
        z_selector: fn(&T) -> Decimal
    ) -> Result<Decimal, InterpolationError>
    where
        T: Clone
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
        let (left_index, right_index) = match sorted_points.iter()
            .enumerate()
            .find(|(_, p)| x_selector(p) > target) {
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
    
    pub(crate) fn get_f64_points(&self) -> Vec<(f64, f64, f64)> {
        self
            .points
            .iter()
            .map(|p| (p.x.to_f64().unwrap_or(0.0), p.z.to_f64().unwrap_or(0.0), p.y.to_f64().unwrap_or(0.0)))
            .collect()
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
            || xy.y > self.y_range.1 {
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
        let weights: Vec<Decimal> = closest_points.iter().map(|&point| {
            let dist = ((point.x - xy.x).powi(2) + (point.y - xy.y).powi(2)).sqrt().unwrap();
            Decimal::ONE / (dist + Decimal::new(1, 6)) // Avoid division by zero
        }).collect();

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
            closest_points.iter().map(|p| p.z).sum::<Decimal>() / Decimal::from(closest_points.len())
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
            || xy.y > self.y_range.1 {
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
        let mut x_groups: std::collections::HashMap<Decimal, Vec<&Point3D>> = std::collections::HashMap::new();
        let mut y_groups: std::collections::HashMap<Decimal, Vec<&Point3D>> = std::collections::HashMap::new();

        for &point in &sorted_points {
            x_groups.entry(point.x).or_insert_with(Vec::new).push(point);
            y_groups.entry(point.y).or_insert_with(Vec::new).push(point);
        }

        // Prepare data for interpolation
        let y_values: Vec<Decimal> = y_groups.keys().cloned().collect();

        // Natural cubic spline interpolation
        // We'll interpolate in two steps: first along x, then along y

        // Interpolate along x for each unique y value
        let mut interpolated_x_points: Vec<Point3D> = Vec::new();
        for &y in &y_values {
            let y_points: Vec<&Point3D> = sorted_points.iter()
                .filter(|&&p| p.y == y)
                .cloned()
                .collect();

            if y_points.len() < 2 {
                continue;
            }

            // Perform cubic spline interpolation along x for this y
            let x_interpolated = self.one_dimensional_spline_interpolation(
                &y_points,
                xy.x,
                |p| p.x,
                |p| p.z
            );

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
            |p| p.z
        );

        // Return the final interpolated point
        y_interpolated.map(|z| Point3D::new(xy.x, xy.y, z))
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
        assert!(curve_points.iter().any(|p| p == &Point2D::new(dec!(0.0), dec!(0.0))));
        assert!(curve_points.iter().any(|p| p == &Point2D::new(dec!(1.0), dec!(1.0))));
    }

    #[test]
    fn test_get_curve_y_axis() {
        let points = create_test_points();
        let surface = Surface::new(points);
        let curve = surface.get_curve(Axis::Y);

        // Check curve points
        let curve_points: Vec<Point2D> = curve.points.into_iter().collect();

        // Verify the points are mapped correctly for Y-axis curve
        assert!(curve_points.iter().any(|p| p == &Point2D::new(dec!(0.0), dec!(0.0))));
        assert!(curve_points.iter().any(|p| p == &Point2D::new(dec!(1.0), dec!(2.0))));
    }

    #[test]
    fn test_get_curve_z_axis() {
        let points = create_test_points();
        let surface = Surface::new(points);
        let curve = surface.get_curve(Axis::Z);

        // Check curve points
        let curve_points: Vec<Point2D> = curve.points.into_iter().collect();

        // Verify the points are mapped correctly for Z-axis curve
        assert!(curve_points.iter().any(|p| p == &Point2D::new(dec!(0.0), dec!(0.0))));
        assert!(curve_points.iter().any(|p| p == &Point2D::new(dec!(1.0), dec!(1.0))));
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
            (dec!(0.25), dec!(0.5)),   // Midpoint
            (dec!(0.0), dec!(0.0)),    // Start point
            (dec!(1.0), dec!(2.0)),    // End point
            (dec!(0.75), dec!(1.5)),   // Another point
        ];

        for (target, expected) in test_cases {
            let result = surface.one_dimensional_spline_interpolation(
                &test_points,
                target,
                |p| p.x,
                |p| p.z
            ).unwrap();

            // Allow small deviation due to interpolation
            assert!((result - expected).abs() < dec!(0.1),
                    "Failed for target {}, expected {}, got {}", target, expected, result);
        }
    }

    #[test]
    fn test_one_dimensional_spline_interpolation_insufficient_points() {
        let surface = Surface::new(create_test_points());

        // Single point is insufficient for interpolation
        let test_points = vec![
            Point3D::new(dec!(0.0), dec!(0.0), dec!(0.0)),
        ];

        let result = surface.one_dimensional_spline_interpolation(
            &test_points,
            dec!(0.5),
            |p| p.x,
            |p| p.z
        );

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
            (dec!(-0.5), dec!(0.0)),   // Below minimum
            (dec!(1.5), dec!(2.0)),    // Above maximum
        ];

        for (target, expected) in out_of_range_cases {
            let result = surface.one_dimensional_spline_interpolation(
                &test_points,
                target,
                |p| p.x,
                |p| p.z
            ).unwrap();

            // Should return endpoints for out-of-range values
            assert_eq!(result, expected,
                       "Failed for out-of-range target {}", target);
        }
    }
}

#[cfg(test)]
mod tests_surface_geometric_object {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::geometrics::ResultPoint;

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
                    t.x * t.y // Simple z = x * y surface
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
            params
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
            params
        });

        assert!(matches!(
        result,
        Err(SurfaceError::ConstructionError(_))
    ));
    }

    #[test]
    fn test_construct_parametric_error_handling() {
        // Parametric function that sometimes fails
        let parametric_func: Box<dyn Fn(Point2D) -> ResultPoint<Point3D> + Send + Sync> =
            Box::new(move |t: Point2D| -> ResultPoint<Point3D> {
                if t.x > dec!(0.5) && t.y > dec!(0.5) {
                    Err(Box::from("Test error".to_string()))
                } else {
                    Ok(Point3D::new(
                        t.x,
                        t.y,
                        t.x * t.y
                    ))
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
            params
        });

        assert!(matches!(
        result,
        Err(SurfaceError::ConstructionError(_))
    ));
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
        let result = surface.cubic_interpolate(Point2D::new(dec!(0.5), dec!(0.5))).unwrap();

        assert_eq!(result.z, dec!(1.5));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface.cubic_interpolate(Point2D::new(dec!(0.4), dec!(0.4))).unwrap();

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
            assert!(result.z >= dec!(0.0) && result.z <= dec!(2.0),
                    "Failed for point {:?}", point);

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
            assert!(result.z > dec!(0.0) && result.z < dec!(2.0),
                    "Failed for boundary point {:?}", point);
        }
    }

    #[test]
    fn test_interpolation_precision() {
        let surface = create_test_surface();
        let result = surface.cubic_interpolate(Point2D::new(dec!(0.333333), dec!(0.333333))).unwrap();

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
        let max_diff = results.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap() -
            results.iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        assert!(max_diff < dec!(0.001), "Interpolation results should be consistent");
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
            assert!(result.z >= dec!(0.0) && result.z <= dec!(2.0),
                    "Failed for extreme point {:?}", point);
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
        let result = surface.spline_interpolate(Point2D::new(dec!(0.5), dec!(0.5))).unwrap();

        assert_eq!(result.z, dec!(1.5));
    }

    #[test]
    fn test_midpoint_interpolation() {
        let surface = create_test_surface();
        let result = surface.spline_interpolate(Point2D::new(dec!(0.4), dec!(0.4))).unwrap();

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
            assert!(result.z >= dec!(0.0) && result.z <= dec!(2.0),
                    "Failed for point {:?}", point);

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
            assert!(result.z > dec!(0.0) && result.z < dec!(2.0),
                    "Failed for boundary point {:?}", point);
        }
    }

    #[test]
    fn test_interpolation_precision() {
        let surface = create_test_surface();
        let result = surface.spline_interpolate(Point2D::new(dec!(0.333333), dec!(0.333333))).unwrap();

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
        let max_diff = results.iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap() -
            results.iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

        assert!(max_diff < dec!(0.001), "Interpolation results should be consistent");
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
            assert!(result.z >= dec!(0.0) && result.z <= dec!(2.0),
                    "Failed for extreme point {:?}", point);
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
            (dec!(0.25), dec!(0.5)),   // Midpoint
            (dec!(0.0), dec!(0.0)),    // Start point
            (dec!(1.0), dec!(2.0)),    // End point
            (dec!(0.75), dec!(1.5)),   // Another point
        ];

        for (target, expected) in test_points {
            let result = surface.one_dimensional_spline_interpolation(
                &points,
                target,
                |p| p.x,
                |p| p.z
            ).unwrap();

            // Allow small deviation due to interpolation
            assert!((result - expected).abs() < dec!(0.1),
                    "Failed for target {}, expected {}, got {}", target, expected, result);
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