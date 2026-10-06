#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code.
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-math
//!
//! Generic numerical containers and algorithms of OptionStratLib: 2D curves,
//! 3D surfaces, the geometry, construction and interpolation machinery they
//! share, and the statistics extracted from them. Nothing here knows about
//! option chains, Greeks, pricing or plotting; it depends only on
//! `optionstratlib-core` and general numeric crates.
//!
//! - [`curves`]: [`curves::Curve`], [`curves::Point2D`] and the curve traits.
//! - [`surfaces`]: [`surfaces::Surface`], [`surfaces::Point3D`] and the
//!   surface traits.
//! - [`geometrics`]: construction, interpolation (linear, bilinear, cubic,
//!   spline), arithmetic and metric extraction shared by curves and surfaces.
//! - [`error`]: [`error::CurveError`], [`error::SurfaceError`],
//!   [`error::InterpolationError`] and [`error::MetricsError`].
//!
//! Coordinates are `rust_decimal::Decimal`; arithmetic on them is checked.
//! Option-specific projections of curves and surfaces (`BasicCurves`,
//! `BasicSurfaces`) belong to the analytics layer, and plotting to
//! `optionstratlib-visualization`; the `optionstratlib` facade provides both.
//!
//! ## Imports: no prelude
//!
//! This crate has no `prelude` module, by decision (#518). Each module root
//! re-exports what a caller needs: `curves` (`Curve`, `Point2D`, `Curvable`,
//! `StatisticalCurve`), `surfaces` (`Surface`, `Point3D`, `Surfacable`),
//! `geometrics` (construction, interpolation and the shared traits) and
//! `error`.
//!
//! Measured when the crate was extracted, counting explicit
//! `optionstratlib::{curves,surfaces,geometrics}::…` imports: the examples
//! name no math item explicitly (they use the facade's `prelude::*`), and
//! tests and benches name 20 distinct math items in 4 files, most of them in
//! one property test that imports 15 `geometrics` items. That use is narrow
//! and specialised; a crate glob would serve none of it better than the
//! module roots already do.
//!
//! ```rust
//! use optionstratlib_math::curves::{Curvable, Curve, Point2D, StatisticalCurve};
//! use optionstratlib_math::error::{CurveError, SurfaceError};
//! use optionstratlib_math::geometrics::{
//!     ConstructionMethod, ConstructionParams, GeometricObject, Interpolate, InterpolationType,
//! };
//! use optionstratlib_math::surfaces::{Point3D, Surfacable, Surface};
//! ```
//!
//! ## Behaviour that lives outside this crate
//!
//! Some capabilities take a math type as input but belong to a higher layer,
//! so they are implemented there, not here (#517):
//!
//! | Capability | Where | Owner |
//! | --- | --- | --- |
//! | `BasicCurves`, `BasicSurfaces`: projecting an option set onto a `Curve` or `Surface` (prices options, reads Greeks) | `optionstratlib::analytics` | analytics; final adapter in M4-01 (#529) |
//! | `Graph` and `Plottable` for `Curve`, `Vec<Curve>` and `Surface` | `optionstratlib_visualization::visualization`, also `optionstratlib::visualization` | visualization (#542) |
//!
//! Rendering failures are `GraphError`s of the visualization layer; the math
//! errors carry no rendering variant.
//!
//! ## Features
//!
//! - `schema` (off by default): derives `utoipa::ToSchema` on the math types
//!   and enables `optionstratlib-core/schema`.

/// Geometry shared by curves and surfaces: construction, interpolation,
/// arithmetic and metric extraction.
pub mod geometrics;

/// Two-dimensional curves: `Curve`, `Point2D` and their traits.
pub mod curves;

/// Three-dimensional surfaces: `Surface`, `Point3D` and their traits.
pub mod surfaces;

/// Errors raised by the math types.
pub mod error;

/// Version of the `optionstratlib-math` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
