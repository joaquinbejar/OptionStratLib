/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use crate::error::SurfaceError;
use crate::surfaces::analysis::metrics::{
    BasicMetrics, SurfaceMetrics, RangeMetrics, ShapeMetrics, SpatialMetrics,
};


/// Trait for extracting metrics from surfaces.
pub trait SurfaceMetricsExtractor {
    /// Computes basic height-based statistical metrics.
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, SurfaceError>;

    /// Computes shape-related metrics including curvature analysis.
    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, SurfaceError>;

    /// Computes spatial metrics like surface area and volume.
    fn compute_spatial_metrics(&self) -> Result<SpatialMetrics, SurfaceError>;

    /// Computes range-based metrics for heights.
    fn compute_range_metrics(&self) -> Result<RangeMetrics, SurfaceError>;

    /// Computes all surface metrics.
    fn compute_surface_metrics(&self) -> Result<SurfaceMetrics, SurfaceError> {
        let basic = self.compute_basic_metrics()?;
        let shape = self.compute_shape_metrics()?;
        let spatial = self.compute_spatial_metrics()?;
        let range = self.compute_range_metrics()?;

        Ok(SurfaceMetrics::new(basic, shape, spatial, range))
    }
}