/******************************************************************************
   Author: Joaquín Béjar García 
   Email: jb@taunais.com
   Date: 20/1/25
******************************************************************************/

use crate::surfaces::analysis::SurfaceAnalysisResult;
use crate::surfaces::types::{Surface, SurfaceType, Point3D, SurfaceInterpolationType};
use crate::error::SurfaceError;
use rust_decimal::Decimal;
use crate::curves::Curve;

/// The `SurfaceOperations` trait defines a comprehensive set of operations for 3D mathematical surfaces.
///
/// # Methods
///
/// ## Core Operations
/// - `generate_surface(&self, points: Vec<Point3D>, surface_type: SurfaceType) -> Result<Surface, SurfaceError>`
///   Creates a new surface from 3D points and specified type.
///
/// - `interpolate(&self, point: Point3D, surface: &Surface, interpolation: SurfaceInterpolationType) -> Option<Decimal>`
///   Calculates surface height (z-value) at given (x,y) point using specified interpolation method.
///
/// - `analyze_surface(&self, surface: &Surface) -> SurfaceAnalysisResult`
///   Performs statistical analysis on the surface.
///
/// ## Surface Transformations
/// - `merge_surfaces(&self, surfaces: Vec<&Surface>, operation: SurfaceMergeOperation) -> Result<Surface, SurfaceError>`
///   Combines multiple surfaces using specified operation (add, subtract, multiply, etc.).
///
/// - `slice_surface(&self, surface: &Surface, plane: Plane) -> Result<Curve, SurfaceError>`
///   Extracts intersection curve between surface and given plane.
///
/// - `translate_surface(&self, surface: &Surface, dx: Decimal, dy: Decimal, dz: Decimal) -> Result<Surface, SurfaceError>`
///   Shifts surface by dx, dy, and dz.
///
/// - `scale_surface(&self, surface: &Surface, sx: Decimal, sy: Decimal, sz: Decimal) -> Result<Surface, SurfaceError>`
///   Scales surface by factors sx, sy, and sz.
///
/// - `rotate_surface(&self, surface: &Surface, angle_x: Decimal, angle_y: Decimal, angle_z: Decimal) -> Result<Surface, SurfaceError>`
///   Rotates surface around x, y, and z axes.
///
/// ## Analysis Operations
/// - `find_intersections(&self, surface1: &Surface, surface2: &Surface) -> Result<Vec<Point3D>, SurfaceError>`
///   Locates points where two surfaces intersect.
///
/// - `normal_at(&self, surface: &Surface, point: Point3D) -> Result<Point3D, SurfaceError>`
///   Calculates normal vector at given point on surface.
///
/// - `partial_derivative_x(&self, surface: &Surface, point: Point3D) -> Result<Decimal, SurfaceError>`
///   Calculates partial derivative with respect to x at given point.
///
/// - `partial_derivative_y(&self, surface: &Surface, point: Point3D) -> Result<Decimal, SurfaceError>`
///   Calculates partial derivative with respect to y at given point.
///
/// - `get_extrema(&self, surface: &Surface) -> Result<(Point3D, Point3D), SurfaceError>`
///   Finds minimum and maximum points on surface.
///
/// - `surface_area(&self, surface: &Surface) -> Result<Decimal, SurfaceError>`
///   Calculates total surface area.
///
/// - `volume_under_surface(&self, surface: &Surface, z0: Decimal) -> Result<Decimal, SurfaceError>`
///   Calculates volume between surface and z=z0 plane.
pub trait SurfaceOperations {
    fn generate_surface(
        &self,
        points: Vec<Point3D>,
        surface_type: SurfaceType,
    ) -> Result<Surface, SurfaceError>;

    fn interpolate(
        &self,
        point: Point3D,
        surface: &Surface,
        interpolation: SurfaceInterpolationType,
    ) -> Option<Decimal>;

    fn analyze_surface(&self, surface: &Surface) -> SurfaceAnalysisResult;

    fn merge_surfaces(
        &self,
        surfaces: Vec<&Surface>,
        operation: SurfaceMergeOperation,
    ) -> Result<Surface, SurfaceError>;

    fn slice_surface(
        &self,
        surface: &Surface,
        plane: Plane,
    ) -> Result<Curve, SurfaceError>;

    fn translate_surface(
        &self,
        surface: &Surface,
        dx: Decimal,
        dy: Decimal,
        dz: Decimal,
    ) -> Result<Surface, SurfaceError>;

    fn scale_surface(
        &self,
        surface: &Surface,
        sx: Decimal,
        sy: Decimal,
        sz: Decimal,
    ) -> Result<Surface, SurfaceError>;

    fn rotate_surface(
        &self,
        surface: &Surface,
        angle_x: Decimal,
        angle_y: Decimal,
        angle_z: Decimal,
    ) -> Result<Surface, SurfaceError>;

    fn find_intersections(
        &self,
        surface1: &Surface,
        surface2: &Surface,
    ) -> Result<Vec<Point3D>, SurfaceError>;

    fn normal_at(
        &self,
        surface: &Surface,
        point: Point3D,
    ) -> Result<Point3D, SurfaceError>;

    fn partial_derivative_x(
        &self,
        surface: &Surface,
        point: Point3D,
    ) -> Result<Decimal, SurfaceError>;

    fn partial_derivative_y(
        &self,
        surface: &Surface,
        point: Point3D,
    ) -> Result<Decimal, SurfaceError>;

    fn get_extrema(
        &self,
        surface: &Surface,
    ) -> Result<(Point3D, Point3D), SurfaceError>;

    fn surface_area(
        &self,
        surface: &Surface,
    ) -> Result<Decimal, SurfaceError>;

    fn volume_under_surface(
        &self,
        surface: &Surface,
        z0: Decimal,
    ) -> Result<Decimal, SurfaceError>;
}

/// Enum defining available operations for merging surfaces
#[derive(Debug, Clone, Copy)]
pub enum SurfaceMergeOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Min,
    Max,
}

/// Struct representing a plane in 3D space using the equation ax + by + cz + d = 0
#[derive(Debug, Clone)]
pub struct Plane {
    pub a: Decimal,
    pub b: Decimal,
    pub c: Decimal,
    pub d: Decimal,
}

impl Plane {
    pub fn new(a: Decimal, b: Decimal, c: Decimal, d: Decimal) -> Self {
        Self { a, b, c, d }
    }

    pub fn from_points(p1: Point3D, p2: Point3D, p3: Point3D) -> Result<Self, SurfaceError> {
        // Calculate vector v1 = p2 - p1
        let v1x = p2.x - p1.x;
        let v1y = p2.y - p1.y;
        let v1z = p2.z - p1.z;

        // Calculate vector v2 = p3 - p1
        let v2x = p3.x - p1.x;
        let v2y = p3.y - p1.y;
        let v2z = p3.z - p1.z;

        // Normal vector is cross product of v1 x v2
        let a = v1y * v2z - v1z * v2y;
        let b = v1z * v2x - v1x * v2z;
        let c = v1x * v2y - v1y * v2x;

        // d coefficient = -(ax + by + cz) using p1
        let d = -(a * p1.x + b * p1.y + c * p1.z);

        Ok(Self::new(a, b, c, d))
    }
}