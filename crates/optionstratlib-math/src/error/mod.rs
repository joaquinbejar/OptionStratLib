//! Errors owned by the math layer.
//!
//! Each failure here is raised by a generic math type: a curve, a surface, an
//! interpolation or a statistic extracted from them. Errors raised by core
//! types (`DecimalError`, `PositionError`, …) stay in `optionstratlib-core`,
//! and the variants below that wrap them carry those types unchanged.
//!
//! # Canonical paths (#550)
//!
//! Every error type is exported flat from this module, for example
//! `optionstratlib_math::error::CurveError`. The file module `curves` is
//! private: it held nothing the flat paths do not, and was public only
//! because 0.21 exposed it.
//!
//! ```rust
//! use optionstratlib_math::error::{CurveError, CurvesResult};
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_math::error::curves::CurveError;
//! ```
//!
//! ```compile_fail,E0603
//! use optionstratlib_math::error::curves::CurvesResult;
//! ```

/// ### Curve Errors (`CurveError`)
/// Handles:
/// * Curve construction from points or parametric functions
/// * Point lookup and segment search
/// * Curve arithmetic and merging
/// * Metric extraction over a curve
mod curves;

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
