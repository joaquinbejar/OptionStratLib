//! Errors owned by the math layer.
//!
//! Each failure here is raised by a generic math type: a curve, a surface, an
//! interpolation or a statistic extracted from them. Errors raised by core
//! types (`DecimalError`, `PositionError`, …) stay in `optionstratlib-core`,
//! and the variants below that wrap them carry those types unchanged.

/// ### Curve Errors (`CurveError`)
/// Handles:
/// * Curve construction from points or parametric functions
/// * Point lookup and segment search
/// * Curve arithmetic and merging
/// * Metric extraction over a curve
pub mod curves;

/// ### Interpolation Errors (`InterpolationError`)
/// Manages:
/// * Data point validation
/// * Interpolation method errors
/// * Boundary conditions
/// * Mathematical approximation issues
mod interpolation;

/// ### Metrics Errors (`MetricsError`)
/// Statistical, shape, range and trend metrics extracted from curves and
/// surfaces.
mod metrics;

/// ### Surface Errors (`SurfaceError`)
/// Covers:
/// * Surface construction failures
/// * Dimensional and data completeness errors
/// * Surface arithmetic and interpolation
mod surfaces;

pub use curves::{CurveError, CurvesResult};
pub use interpolation::InterpolationError;
pub use metrics::MetricsError;
pub use surfaces::SurfaceError;
