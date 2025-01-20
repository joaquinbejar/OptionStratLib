
use rust_decimal::Decimal;
use crate::surfaces::analysis::statistics::SurfaceAnalysisResult;
use crate::surfaces::types::Point3D;

/// Basic statistical metrics for surfaces.
#[derive(Clone, Copy, Debug)]
pub struct BasicMetrics {
    pub mean_height: Decimal,
    pub median_height: Decimal,
    pub mode_height: Decimal,
    pub std_dev_height: Decimal,
}

/// Metrics describing the shape and form of the surface.
#[derive(Clone, Debug)]
pub struct ShapeMetrics {
    pub mean_curvature: Decimal,
    pub gaussian_curvature: Decimal,
    pub peaks: Vec<Point3D>,
    pub valleys: Vec<Point3D>,
    pub saddle_points: Vec<Point3D>,
}

/// Metrics describing the spatial characteristics of the surface.
#[derive(Clone, Debug)]
pub struct SpatialMetrics {
    pub surface_area: Decimal,
    pub volume: Decimal,
    pub roughness: Decimal,
    pub normal_vectors: Vec<(Point3D, Point3D)>, // (point, normal_vector)
}

/// Metrics describing the range and distribution of heights.
#[derive(Clone, Copy, Debug)]
pub struct RangeMetrics {
    pub min_height: Point3D,
    pub max_height: Point3D,
    pub height_range: Decimal,
    pub quartiles: (Decimal, Decimal, Decimal),
    pub interquartile_range: Decimal,
}

/// Comprehensive metrics for surface analysis.
#[derive(Clone, Debug)]
pub struct SurfaceMetrics {
    pub basic: BasicMetrics,
    pub shape: ShapeMetrics,
    pub spatial: SpatialMetrics,
    pub range: RangeMetrics,
}

impl SurfaceMetrics {
    pub fn new(
        basic: BasicMetrics,
        shape: ShapeMetrics,
        spatial: SpatialMetrics,
        range: RangeMetrics,
    ) -> Self {
        Self {
            basic,
            shape,
            spatial,
            range,
        }
    }

    pub fn surface_analysis_result(&self) -> Result<SurfaceAnalysisResult, crate::error::surface::SurfaceError> {
        Ok(SurfaceAnalysisResult {
            statistics: self.basic,
            shape_metrics: self.shape.clone(),
        })
    }
}