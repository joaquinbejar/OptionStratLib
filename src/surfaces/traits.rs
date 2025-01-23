use crate::curves::{Curve, Point2D};
use crate::error::SurfaceError;
use crate::geometrics::{AnalysisResult, MergeOperation};
use crate::surfaces::types::{Point3D, SurfaceInterpolationType};
use crate::surfaces::Surface;
use rust_decimal::Decimal;

pub trait SurfaceOperations {
    fn generate_surface(&self, points: Vec<Point3D>) -> Result<Surface, SurfaceError>;

    fn interpolate(
        &self,
        point: Point3D,
        surface: &Surface,
        interpolation: SurfaceInterpolationType,
    ) -> Option<Decimal>;

    fn analyze_surface(&self, surface: &Surface) -> AnalysisResult;

    fn merge_surfaces(
        &self,
        surfaces: Vec<&Surface>,
        operation: MergeOperation,
    ) -> Result<Surface, SurfaceError>;

    fn slice_surface(
        &self,
        surface: &Surface,
        x1: Point2D,
        x2: Point2D,
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

    fn normal_at(&self, surface: &Surface, point: Point3D) -> Result<Point3D, SurfaceError>;

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

    fn get_extrema(&self, surface: &Surface) -> Result<(Point3D, Point3D), SurfaceError>;

    fn surface_area(&self, surface: &Surface) -> Result<Decimal, SurfaceError>;

    fn volume_under_surface(&self, surface: &Surface, z0: Decimal)
        -> Result<Decimal, SurfaceError>;
}
